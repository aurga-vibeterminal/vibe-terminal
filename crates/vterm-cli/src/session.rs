use crate::client::VTermClient;
use crate::terminal::Terminal;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use vterm_common::SessionId;
use vterm_core::protocol::Message;

/// Device query sequences the vterm server emulator answers by itself.
/// They are stripped from the output stream before it reaches the host
/// terminal: if the host terminal also saw them it would inject its own
/// reply (e.g. `ESC [ 6 ; 20 ; 10 t` for a `CSI 16 t` cell-size query) into
/// our stdin, where the event-based input re-encoding would mangle it into
/// literal typed text inside the application.
const ANSWERED_QUERIES: &[&[u8]] = &[
    b"\x1b[5n",
    b"\x1b[6n", // DSR status / cursor position
    b"\x1b[c",
    b"\x1b[0c", // primary DA
    b"\x1b[>c",
    b"\x1b[>0c", // secondary DA
    b"\x1b[?u",  // kitty keyboard flags query
    b"\x1b[11t",
    b"\x1b[13t",
    b"\x1b[14t",
    b"\x1b[15t",
    b"\x1b[16t",
    b"\x1b[18t",
    b"\x1b[19t",
    // XTWINOPS window/cell size queries
];

enum QueryMatch {
    /// A full query sequence of this length starts here — strip it.
    Full(usize),
    /// The bytes are a strict prefix of a query — wait for more output.
    Prefix,
    /// Not a query — pass through.
    None,
}

fn match_answered_query(rest: &[u8]) -> QueryMatch {
    let mut prefix = false;
    for query in ANSWERED_QUERIES {
        if rest.len() >= query.len() && &rest[..query.len()] == *query {
            return QueryMatch::Full(query.len());
        }
        if rest.len() < query.len() && query.starts_with(rest) {
            prefix = true;
        }
    }
    if prefix {
        QueryMatch::Prefix
    } else {
        QueryMatch::None
    }
}

/// Incremental filter removing `ANSWERED_QUERIES` sequences from the session
/// output stream, keeping incomplete escape prefixes buffered across chunks.
struct DeviceQueryFilter {
    pending: Vec<u8>,
}

impl DeviceQueryFilter {
    fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    fn push(&mut self, data: &[u8]) -> Vec<u8> {
        let mut work = std::mem::take(&mut self.pending);
        work.extend_from_slice(data);
        let mut out = Vec::with_capacity(work.len());
        let mut i = 0;
        while i < work.len() {
            if work[i] != 0x1b {
                let start = i;
                while i < work.len() && work[i] != 0x1b {
                    i += 1;
                }
                out.extend_from_slice(&work[start..i]);
                continue;
            }
            match match_answered_query(&work[i..]) {
                QueryMatch::Full(len) => i += len,
                QueryMatch::Prefix => {
                    self.pending = work[i..].to_vec();
                    break;
                }
                QueryMatch::None => {
                    out.push(0x1b);
                    i += 1;
                }
            }
        }
        out
    }

    /// Return any buffered incomplete escape sequence (call at stream end).
    fn flush(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.pending)
    }
}

/// Run a terminal session
pub async fn run_session(client: &mut VTermClient, session_id: SessionId) -> Result<i32> {
    // Start streaming
    client.start_stream(session_id).await?;

    // Create channels for communication
    let (input_tx, mut input_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    let (output_tx, output_rx) = mpsc::unbounded_channel::<Vec<u8>>();

    // Spawn stdin reader task
    let stdin_handle = tokio::spawn(read_stdin(input_tx));

    // Spawn stdout writer task
    let stdout_handle = tokio::spawn(write_stdout(output_rx));

    // Main I/O loop
    let mut exit_code = 0i32;
    // Once stdin reaches EOF we stop sending input but keep draining
    // output until the session closes (e.g. `echo cmd | vtermc ...`).
    let mut input_done = false;

    loop {
        tokio::select! {
            // Handle input from stdin
            Some(data) = input_rx.recv(), if !input_done => {
                if data.is_empty() {
                    // EOF
                    input_done = true;
                } else {
                    client.send_input(session_id, data).await?;
                }
            }

            // Handle messages from server
            msg = client.receive_message() => {
                match msg {
                    Some(Message::Output { data, .. }) => {
                        if output_tx.send(data).is_err() {
                            break;
                        }
                    }
                    Some(Message::SessionClosed { .. }) => {
                        tracing::debug!("Session closed by server");
                        break;
                    }
                    Some(Message::Error { code, message }) => {
                        tracing::error!("Server error: {} - {}", code, message);
                        exit_code = 1;
                        break;
                    }
                    Some(_) => {}
                    None => {
                        tracing::debug!("Connection closed by server");
                        break;
                    }
                }
            }

            // Check for terminal resize
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {
                // Check for resize (handled separately)
            }
        }
    }

    // Cleanup
    drop(stdout_handle);
    drop(stdin_handle);

    Ok(exit_code)
}

/// Read from stdin and send to channel
async fn read_stdin(tx: mpsc::UnboundedSender<Vec<u8>>) {
    let mut stdin = tokio::io::stdin();
    let mut buffer = [0u8; 1024];

    loop {
        match stdin.read(&mut buffer).await {
            Ok(0) => {
                // EOF
                let _ = tx.send(vec![]);
                break;
            }
            Ok(n) => {
                if tx.send(buffer[..n].to_vec()).is_err() {
                    break;
                }
            }
            Err(e) => {
                tracing::error!("Stdin read error: {}", e);
                break;
            }
        }
    }
}

/// Write to stdout from channel
async fn write_stdout(mut rx: mpsc::UnboundedReceiver<Vec<u8>>) {
    let mut stdout = tokio::io::stdout();

    while let Some(data) = rx.recv().await {
        if let Err(e) = stdout.write_all(&data).await {
            tracing::error!("Stdout write error: {}", e);
            break;
        }
        if let Err(e) = stdout.flush().await {
            tracing::error!("Stdout flush error: {}", e);
            break;
        }
    }
}

/// Run session with raw mode and resize handling
pub async fn run_interactive_session(
    client: &mut VTermClient,
    session_id: SessionId,
) -> Result<i32> {
    // Get initial size
    let size = Terminal::size()?;
    client.resize(session_id, size).await?;

    // Enable raw mode
    Terminal::enable_raw_mode()?;
    let _guard = RawModeGuard;

    // Start streaming
    client.start_stream(session_id).await?;

    // Shadow emulator tracking the session's screen state. The remote byte
    // stream is forwarded to the host console verbatim, so the host console's
    // cursor — which its IME candidate window anchors to — ends up wherever
    // each frame finished. When the application hides the real cursor and
    // draws its own (Kimi Code and similar TUIs), that anchor is wrong; the
    // shadow emulator lets us spot the drawn cursor and park the (hidden)
    // host cursor on it instead.
    let mut shadow = vterm_core::Terminal::new(size);
    // The host console answers device queries directly (the byte stream is
    // forwarded verbatim); a mirror that buffers replies would only grow.
    shadow.set_query_responder(false);

    // Create channels
    let (input_tx, mut input_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    let (output_tx, output_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    let (shutdown_tx, shutdown_rx) = mpsc::unbounded_channel::<()>();

    // Spawn tasks
    // Use spawn_blocking for the input task since crossterm events are synchronous
    let input_handle =
        tokio::task::spawn_blocking(|| read_input_raw_blocking(input_tx, shutdown_rx));
    let output_handle = tokio::spawn(write_stdout(output_rx));

    // Track terminal size for resize detection
    let mut last_size = size;

    // Main loop
    let mut exit_code = 0i32;
    let mut query_filter = DeviceQueryFilter::new();
    // Drawn-cursor anchoring runs only once the output stream goes quiet:
    // per-message timing depends on how the server chunks frames, so a scan
    // mid-burst can see a half-drawn screen (zero or several inverse cells).
    let anchor_scan = tokio::time::sleep(tokio::time::Duration::from_millis(40));
    tokio::pin!(anchor_scan);
    let mut anchor_pending = false;

    loop {
        tokio::select! {
            // Handle input
            Some(data) = input_rx.recv() => {
                if data.is_empty() {
                    break;
                }
                if let Err(e) = client.send_input(session_id, data).await {
                    tracing::error!("Failed to send input: {}", e);
                    exit_code = 1;
                    break;
                }
            }

            // Handle output from server
            msg = client.receive_message() => {
                match msg {
                    Some(Message::Output { data, .. }) => {
                        let data = query_filter.push(&data);
                        if !data.is_empty() {
                            shadow.process(&data);
                            if output_tx.send(data).is_err() {
                                break;
                            }
                            anchor_pending = true;
                            anchor_scan
                                .as_mut()
                                .reset(tokio::time::Instant::now() + tokio::time::Duration::from_millis(40));
                        }
                    }
                    Some(Message::SessionClosed { .. }) => {
                        break;
                    }
                    Some(Message::Error { code, message }) => {
                        eprintln!("\r\nServer error: {} - {}\r", code, message);
                        exit_code = 1;
                        break;
                    }
                    Some(_) => {}
                    None => {
                        tracing::debug!("Connection closed by server");
                        break;
                    }
                }
            }

            // Output went quiet — the screen is settled; re-anchor now.
            () = &mut anchor_scan, if anchor_pending => {
                anchor_pending = false;
                if let Some((row, col)) = drawn_cursor_anchor(&shadow) {
                    // Park the host console's cursor on the TUI's drawn
                    // cursor so the host's IME candidate window follows it.
                    // 1-based CUP; invisible while the application keeps the
                    // cursor hidden.
                    let cup = format!("\x1b[{};{}H", row + 1, col + 1);
                    if output_tx.send(cup.into_bytes()).is_err() {
                        break;
                    }
                }
            }

            // Check for terminal resize
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(250)) => {
                if let Ok(new_size) = Terminal::size() {
                    if new_size.rows != last_size.rows || new_size.cols != last_size.cols {
                        last_size = new_size;
                        shadow.resize(new_size);
                        if let Err(e) = client.resize(session_id, new_size).await {
                            tracing::warn!("Failed to send resize: {}", e);
                        }
                    }
                }
            }
        }
    }

    // Cleanup
    let rest = query_filter.flush();
    if !rest.is_empty() {
        let _ = output_tx.send(rest);
    }
    drop(output_tx);
    // Signal input task to shutdown
    let _ = shutdown_tx.send(());
    output_handle.await.ok();
    input_handle.await.ok();

    Ok(exit_code)
}

/// Locate the cursor a TUI application drew itself when the real cursor is
/// hidden (DECTCEM off): apps like Kimi Code leave the real cursor wherever
/// the frame finished (e.g. the bottom status line) and paint the input
/// cursor as a single reverse-video cell. Returns that cell's `(row, col)` —
/// but only when exactly one inverse cell exists on the visible screen, so
/// menus/selections rendered with reverse video don't produce a bogus anchor.
fn drawn_cursor_anchor(terminal: &vterm_core::Terminal) -> Option<(usize, usize)> {
    if terminal.cursor_visible() {
        return None;
    }
    let total = terminal.total_line_count();
    let start = total.saturating_sub(terminal.size().rows as usize);
    let mut found: Option<(usize, usize)> = None;
    for line_index in start..total {
        let Some(line) = terminal.line_at(line_index) else {
            continue;
        };
        // The visible screen is the last `rows` lines; line_index within the
        // grid maps to a row relative to the screen top.
        for (col, cell) in line.iter().enumerate() {
            if cell
                .attributes
                .contains(vterm_common::CellAttributes::INVERSE)
            {
                if found.is_some() {
                    return None;
                }
                found = Some((line_index - start, col));
            }
        }
    }
    found
}

/// Read input in raw mode (captures all keys including special keys)
/// This runs in a blocking task since crossterm events are synchronous
fn read_input_raw_blocking(
    tx: mpsc::UnboundedSender<Vec<u8>>,
    mut shutdown_rx: mpsc::UnboundedReceiver<()>,
) {
    loop {
        // Check for shutdown signal (non-blocking)
        match shutdown_rx.try_recv() {
            Ok(()) | Err(mpsc::error::TryRecvError::Disconnected) => break,
            Err(mpsc::error::TryRecvError::Empty) => {}
        }

        // Poll for events with a short timeout
        // This allows us to periodically check for shutdown signal
        if event::poll(std::time::Duration::from_millis(50)).unwrap_or(false) {
            match event::read() {
                Ok(Event::Key(key)) => {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    match key.code {
                        KeyCode::Char(c) => {
                            let bytes =
                                if key.modifiers.contains(KeyModifiers::CONTROL) && c.is_ascii() {
                                    // Ctrl+letter arrives as Char with the CONTROL
                                    // modifier; the terminal expects the control
                                    // byte (Ctrl+A = 0x01 … Ctrl+Z = 0x1a).
                                    vec![(c as u8) & 0x1f]
                                } else {
                                    // Regular characters — including CJK and other
                                    // multi-byte codepoints — go out as UTF-8.
                                    c.encode_utf8(&mut [0; 4]).as_bytes().to_vec()
                                };
                            if tx.send(bytes).is_err() {
                                break;
                            }
                        }
                        KeyCode::Enter => {
                            if tx.send(vec![b'\r']).is_err() {
                                break;
                            }
                        }
                        KeyCode::Tab => {
                            if tx.send(vec![b'\t']).is_err() {
                                break;
                            }
                        }
                        KeyCode::Backspace => {
                            if tx.send(vec![0x7f]).is_err() {
                                break;
                            }
                        }
                        KeyCode::Esc => {
                            if tx.send(vec![0x1b]).is_err() {
                                break;
                            }
                        }
                        KeyCode::Up => {
                            if tx.send(vec![0x1b, b'[', b'A']).is_err() {
                                break;
                            }
                        }
                        KeyCode::Down => {
                            if tx.send(vec![0x1b, b'[', b'B']).is_err() {
                                break;
                            }
                        }
                        KeyCode::Right => {
                            if tx.send(vec![0x1b, b'[', b'C']).is_err() {
                                break;
                            }
                        }
                        KeyCode::Left => {
                            if tx.send(vec![0x1b, b'[', b'D']).is_err() {
                                break;
                            }
                        }
                        KeyCode::Home => {
                            if tx.send(vec![0x1b, b'[', b'H']).is_err() {
                                break;
                            }
                        }
                        KeyCode::End => {
                            if tx.send(vec![0x1b, b'[', b'F']).is_err() {
                                break;
                            }
                        }
                        KeyCode::PageUp => {
                            if tx.send(vec![0x1b, b'[', b'5', b'~']).is_err() {
                                break;
                            }
                        }
                        KeyCode::PageDown => {
                            if tx.send(vec![0x1b, b'[', b'6', b'~']).is_err() {
                                break;
                            }
                        }
                        KeyCode::Delete => {
                            if tx.send(vec![0x1b, b'[', b'3', b'~']).is_err() {
                                break;
                            }
                        }
                        KeyCode::Insert => {
                            if tx.send(vec![0x1b, b'[', b'2', b'~']).is_err() {
                                break;
                            }
                        }
                        KeyCode::F(n) => {
                            let seq = match n {
                                1 => vec![0x1b, b'O', b'P'],
                                2 => vec![0x1b, b'O', b'Q'],
                                3 => vec![0x1b, b'O', b'R'],
                                4 => vec![0x1b, b'O', b'S'],
                                5 => vec![0x1b, b'[', b'1', b'5', b'~'],
                                6 => vec![0x1b, b'[', b'1', b'7', b'~'],
                                7 => vec![0x1b, b'[', b'1', b'8', b'~'],
                                8 => vec![0x1b, b'[', b'1', b'9', b'~'],
                                9 => vec![0x1b, b'[', b'2', b'0', b'~'],
                                10 => vec![0x1b, b'[', b'2', b'1', b'~'],
                                11 => vec![0x1b, b'[', b'2', b'3', b'~'],
                                12 => vec![0x1b, b'[', b'2', b'4', b'~'],
                                _ => continue,
                            };
                            if tx.send(seq).is_err() {
                                break;
                            }
                        }
                        KeyCode::Null => {}
                        _ => {} // Ignore other key codes (BackTab, CapsLock, etc.)
                    }
                }
                Ok(Event::Paste(data)) => {
                    if tx.send(data.into_bytes()).is_err() {
                        break;
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Event read error: {}", e);
                    break;
                }
            }
        }
    }
}

/// Guard to ensure raw mode is disabled on drop
struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = Terminal::disable_raw_mode();
    }
}
