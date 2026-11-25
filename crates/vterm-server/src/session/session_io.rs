use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver as StdReceiver, RecvTimeoutError, Sender as StdSender};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use tokio::sync::mpsc;
use tracing::{error, warn};

use vterm_common::{SessionId, TerminalSize};
use vterm_core::pty::{PortablePty, Pty};

use crate::config::{UartFlowControl, UartParity, UartPortConfig, UartStopBits};

use super::session_error::SessionError;

const SERIAL_READ_TIMEOUT_MS: u64 = 10;
/// Reads at or below this size are treated as interactive echo and flushed
/// to the session output queue immediately, instead of waiting for the
/// idle-flush deadline.
const SERIAL_INTERACTIVE_READ_BYTES: usize = 128;
const SERIAL_READ_BUFFER_SIZE: usize = 4096;
const SERIAL_FLUSH_IDLE_MS: u64 = 8;
const SERIAL_PENDING_OUTPUT_MAX_BYTES: usize = 8 * 1024 * 1024;
/// Idle poll interval of the serial I/O thread: when no input is pending it
/// parks on the write queue for this long before polling the port again.
const SERIAL_POLL_IDLE_MS: u64 = 1;
const PTY_OUTPUT_QUEUE_TIMEOUT: Duration = Duration::from_secs(2);

pub enum SessionIo {
    Pty(PortablePty, mpsc::Sender<Vec<u8>>),
    Serial(SerialTransport),
}

pub(super) struct SerialWrite {
    pub data: Vec<u8>,
    pub enqueued_at: std::time::Instant,
}

#[derive(Clone)]
pub(super) enum SessionIoWriter {
    Pty(mpsc::Sender<Vec<u8>>),
    Serial(StdSender<SerialWrite>),
}

impl SessionIoWriter {
    pub(super) async fn write(&self, data: &[u8]) -> Result<(), SessionError> {
        match self {
            Self::Pty(writer) => writer
                .send(data.to_vec())
                .await
                .map_err(|_| SessionError::PtyError("PTY writer closed".to_string())),
            // std mpsc send never blocks: a keystroke is handed to the serial
            // I/O thread immediately.
            Self::Serial(writer) => writer
                .send(SerialWrite {
                    data: data.to_vec(),
                    enqueued_at: std::time::Instant::now(),
                })
                .map_err(|_| SessionError::PtyError("serial writer closed".to_string())),
        }
    }
}

impl SessionIo {
    /// Create a PTY-backed I/O transport with a dedicated writer thread.
    /// `session_id` feeds the writer thread's latency probe.
    pub(super) fn new_pty(pty: PortablePty, session_id: SessionId) -> Self {
        let writer = pty.spawn_writer(Some(session_id));
        Self::Pty(pty, writer)
    }
}

pub struct SerialTransport {
    port: Arc<StdMutex<Option<Box<dyn serialport::SerialPort>>>>,
    writer_tx: StdSender<SerialWrite>,
    writer_rx: StdMutex<Option<StdReceiver<SerialWrite>>>,
    running: Arc<AtomicBool>,
}

impl SerialTransport {
    pub(super) fn open(port_config: &UartPortConfig) -> Result<Self, SessionError> {
        let builder = serialport::new(&port_config.port, port_config.baud_rate)
            .data_bits(serial_data_bits(port_config.data_bits)?)
            .parity(serial_parity(&port_config.parity))
            .stop_bits(serial_stop_bits(&port_config.stop_bits))
            .flow_control(serial_flow_control(&port_config.flow_control))
            .timeout(Duration::from_millis(SERIAL_READ_TIMEOUT_MS));

        let mut port = builder
            .open()
            .map_err(|error| SessionError::PtyError(error.to_string()))?;

        // Terminal emulators (PuTTY) assert DTR/RTS on open; serialport
        // leaves DTR untouched and drives RTS low with FlowControl::None.
        // Some USB-CDC devices gate their console on DTR being asserted,
        // so match the terminal-emulator behavior. RTS stays
        // driver-managed when hardware flow control is on.
        if let Err(error) = port.write_data_terminal_ready(true) {
            warn!("Failed to assert DTR on {}: {}", port_config.port, error);
        }
        if port_config.flow_control == UartFlowControl::None {
            if let Err(error) = port.write_request_to_send(true) {
                warn!("Failed to assert RTS on {}: {}", port_config.port, error);
            }
        }

        let (writer_tx, writer_rx) = std::sync::mpsc::channel();
        Ok(Self {
            port: Arc::new(StdMutex::new(Some(port))),
            writer_tx,
            writer_rx: StdMutex::new(Some(writer_rx)),
            running: Arc::new(AtomicBool::new(true)),
        })
    }

    /// Spawn the single serial I/O thread.
    ///
    /// One thread owns the one port handle and alternates between draining
    /// the write queue and polling the RX FIFO with `bytes_to_read()`.
    /// This is deliberate: the previous design parked a second thread in a
    /// synchronous blocking `ReadFile` on a cloned handle, and with the
    /// CP2102/silabser driver that permanently pending read blocks
    /// `WriteFile` on the other handle for hundreds of milliseconds to
    /// seconds on an idle line (measured 420-2627ms per 1-byte write vs
    /// 0ms with polling). Keystrokes are always drained before reads, so
    /// input goes out the instant it is queued.
    pub(super) fn spawn_io(
        &self,
        session_id: SessionId,
        output_tx: mpsc::Sender<Vec<u8>>,
        manager: super::session_manager::SessionManager,
    ) {
        let Some(mut write_rx) = self.writer_rx.lock().ok().and_then(|mut rx| rx.take()) else {
            return;
        };
        let Some(mut port) = self.port.lock().ok().and_then(|mut port| port.take()) else {
            return;
        };
        let running = Arc::clone(&self.running);
        let runtime = tokio::runtime::Handle::current();

        std::thread::spawn(move || {
            let mut buffer = [0u8; SERIAL_READ_BUFFER_SIZE];
            let mut pending_output = Vec::with_capacity(SERIAL_READ_BUFFER_SIZE * 2);
            let mut last_read_at = std::time::Instant::now();

            while running.load(Ordering::Relaxed) {
                // Writes first: drain every queued keystroke before polling
                // the port, so input latency never waits on read activity.
                while let Ok(request) = write_rx.try_recv() {
                    write_serial(&mut port, session_id, &request);
                }

                let mut output_channel_open = true;
                match port.bytes_to_read() {
                    Ok(0) => {
                        if !pending_output.is_empty()
                            && last_read_at.elapsed() >= Duration::from_millis(SERIAL_FLUSH_IDLE_MS)
                        {
                            output_channel_open = try_flush_output(&mut pending_output, &output_tx);
                        }
                        // Nothing to read: park briefly on the write queue so
                        // an incoming keystroke still goes out immediately,
                        // without busy-spinning the poll loop.
                        match write_rx.recv_timeout(Duration::from_millis(SERIAL_POLL_IDLE_MS)) {
                            Ok(request) => write_serial(&mut port, session_id, &request),
                            Err(RecvTimeoutError::Timeout) => {}
                            Err(RecvTimeoutError::Disconnected) => break,
                        }
                    }
                    Ok(pending_bytes) => {
                        let want = (pending_bytes as usize).min(buffer.len());
                        match port.read(&mut buffer[..want]) {
                            Ok(0) => {}
                            Ok(bytes_read) => {
                                pending_output.extend_from_slice(&buffer[..bytes_read]);
                                last_read_at = std::time::Instant::now();
                                if pending_output.len() >= SERIAL_READ_BUFFER_SIZE
                                    || bytes_read >= SERIAL_READ_BUFFER_SIZE / 2
                                    || bytes_read <= SERIAL_INTERACTIVE_READ_BYTES
                                {
                                    // Interactive echo (small reads) flushes
                                    // immediately; bulk reads flush at the
                                    // batch thresholds; mid-size reads wait
                                    // for the idle deadline above.
                                    output_channel_open =
                                        try_flush_output(&mut pending_output, &output_tx);
                                }
                            }
                            Err(error) if error.kind() == std::io::ErrorKind::TimedOut => {}
                            Err(error) => {
                                let _ = try_flush_output(&mut pending_output, &output_tx);
                                warn!(
                                    "Serial reader stopped for session {}: {}",
                                    session_id, error
                                );
                                let close_manager = manager.clone();
                                runtime.block_on(async move {
                                    let _ = close_manager.close_session(session_id).await;
                                });
                                break;
                            }
                        }
                    }
                    Err(error) => {
                        let _ = try_flush_output(&mut pending_output, &output_tx);
                        warn!(
                            "Serial port status poll failed for session {}: {}",
                            session_id, error
                        );
                        let close_manager = manager.clone();
                        runtime.block_on(async move {
                            let _ = close_manager.close_session(session_id).await;
                        });
                        break;
                    }
                }

                if !output_channel_open {
                    warn!(
                        "Serial output processor stopped for session {}; closing transport",
                        session_id
                    );
                    let close_manager = manager.clone();
                    runtime.block_on(async move {
                        let _ = close_manager.close_session(session_id).await;
                    });
                    break;
                }

                // Never block the hardware reader on the async session queue.
                // Retain a bounded software backlog during short processing
                // stalls; if the consumer cannot recover before this limit,
                // close explicitly instead of silently losing UART bytes.
                if pending_output.len() > SERIAL_PENDING_OUTPUT_MAX_BYTES {
                    error!(
                        "Serial output backlog exceeded {} bytes for session {}; closing to avoid silent data loss",
                        SERIAL_PENDING_OUTPUT_MAX_BYTES, session_id
                    );
                    let close_manager = manager.clone();
                    runtime.block_on(async move {
                        let _ = close_manager.close_session(session_id).await;
                    });
                    break;
                }
            }
        });
    }

    pub(super) fn terminate(&self) -> Result<(), SessionError> {
        // The I/O thread polls the running flag at least every
        // SERIAL_POLL_IDLE_MS and drops the port on exit.
        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }
}

fn write_serial(
    port: &mut Box<dyn serialport::SerialPort>,
    session_id: SessionId,
    request: &SerialWrite,
) {
    let queue_wait_ms = request.enqueued_at.elapsed().as_millis() as u64;
    let write_started = std::time::Instant::now();
    if let Err(error) = port.write_all(&request.data) {
        warn!("Serial write failed for session {}: {}", session_id, error);
        return;
    }
    let write_ms = write_started.elapsed().as_millis() as u64;
    let total_ms = request.enqueued_at.elapsed().as_millis() as u64;
    if vterm_common::latency::sample("server:serial-write", session_id, total_ms as f64, 5.0, 3.0) {
        vterm_common::latency_log!(
            stage = "server:serial-write",
            session = %session_id,
            bytes = request.data.len(),
            queue_wait_ms,
            write_ms,
            "[latency] serial write path was slow"
        );
    }
}

fn try_flush_output(pending_output: &mut Vec<u8>, output_tx: &mpsc::Sender<Vec<u8>>) -> bool {
    if pending_output.is_empty() {
        return true;
    }

    let data = std::mem::take(pending_output);
    match output_tx.try_send(data) {
        Ok(()) => true,
        Err(mpsc::error::TrySendError::Full(data)) => {
            *pending_output = data;
            true
        }
        Err(mpsc::error::TrySendError::Closed(_)) => false,
    }
}

fn send_output_with_timeout(
    output_tx: &mpsc::Sender<Vec<u8>>,
    mut data: Vec<u8>,
    timeout: Duration,
) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    let started_at = deadline - timeout;
    let byte_count = data.len();
    let mut retries = 0u32;
    loop {
        match output_tx.try_send(data) {
            Ok(()) => {
                if retries > 0 {
                    vterm_common::latency_log!(
                        stage = "server:pty-read-queue-full",
                        bytes = byte_count,
                        retries,
                        waited_ms = started_at.elapsed().as_millis() as u64,
                        "[latency] PTY output queue was full; reader thread stalled"
                    );
                }
                return true;
            }
            Err(mpsc::error::TrySendError::Full(returned)) => {
                data = returned;
                retries += 1;
                if std::time::Instant::now() >= deadline {
                    vterm_common::latency_log!(
                        stage = "server:pty-read-queue-timeout",
                        bytes = data.len(),
                        retries,
                        "[latency] PTY output queue stayed full past timeout"
                    );
                    return false;
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            Err(mpsc::error::TrySendError::Closed(_)) => return false,
        }
    }
}

impl SessionIo {
    pub(super) async fn spawn_reader(
        &self,
        session_id: SessionId,
        output_tx: mpsc::Sender<Vec<u8>>,
        manager: super::session_manager::SessionManager,
        io_generation: u64,
    ) -> Result<(), SessionError> {
        match self {
            SessionIo::Pty(pty, _) => {
                let reader = pty.reader();
                let runtime = tokio::runtime::Handle::current();
                let close_manager = manager.clone();
                pty.spawn_exit_watcher(move || {
                    let close_manager = close_manager.clone();
                    runtime.block_on(async move {
                        let _ = close_manager
                            .handle_child_exit(session_id, io_generation)
                            .await;
                    });
                });
                let runtime = tokio::runtime::Handle::current();

                std::thread::spawn(move || {
                    let mut buffer = [0u8; 4096];

                    loop {
                        // Echo-window probes: partition the reader side of the
                        // keystroke echo. `entry` = echo budget already spent
                        // when this loop iteration started (None while the
                        // thread was parked inside the previous read);
                        // `lock_wait_ms` = reader mutex contention;
                        // `read_block_ms` = time inside the blocking ReadFile
                        // — i.e. how long ConPTY/app took to produce data.
                        let entry_ms = vterm_common::latency::echo_ms_since_input(session_id);
                        let lock_started = std::time::Instant::now();
                        let mut lock_wait_ms = 0u64;
                        let mut read_block_ms = 0u64;
                        let read_result = {
                            let mut reader = match reader.lock() {
                                Ok(reader) => reader,
                                Err(error) => {
                                    error!(
                                        "Failed to lock PTY reader for session {}: {}",
                                        session_id, error
                                    );
                                    break;
                                }
                            };
                            lock_wait_ms = lock_started.elapsed().as_millis() as u64;
                            let read_started = std::time::Instant::now();
                            let result = reader.read(&mut buffer);
                            read_block_ms = read_started.elapsed().as_millis() as u64;
                            result
                        };

                        match read_result {
                            Ok(0) => {
                                let close_manager = manager.clone();
                                runtime.block_on(async move {
                                    let _ = close_manager
                                        .handle_child_exit(session_id, io_generation)
                                        .await;
                                });
                                break;
                            }
                            Ok(bytes_read) => {
                                vterm_common::latency::note_first_output(session_id);
                                // Splits the PTY-echo segment: this timestamp
                                // is taken where the blocking read returns. If
                                // it is already late, ConPTY/shell produced
                                // the echo late; if it is early while
                                // `server:pty-output-dequeued` is late, the
                                // delay is in the queue/consumer-task wake.
                                if let Some(echo_ms) =
                                    vterm_common::latency::echo_ms_since_input(session_id)
                                {
                                    if vterm_common::latency::sample(
                                        "server:pty-read-returned",
                                        session_id,
                                        echo_ms as f64,
                                        50.0,
                                        3.0,
                                    ) {
                                        vterm_common::latency_log!(
                                            stage = "server:pty-read-returned",
                                            session = %session_id,
                                            echo_ms,
                                            bytes = bytes_read,
                                            entry_ms = entry_ms.unwrap_or(0),
                                            lock_wait_ms,
                                            read_block_ms,
                                            "[latency] PTY read returned keystroke echo late"
                                        );
                                    }
                                }
                                let data = buffer[..bytes_read].to_vec();
                                if !send_output_with_timeout(
                                    &output_tx,
                                    data,
                                    PTY_OUTPUT_QUEUE_TIMEOUT,
                                ) {
                                    error!(
                                        "PTY output queue remained unavailable for {:?} in session {}; closing transport",
                                        PTY_OUTPUT_QUEUE_TIMEOUT, session_id
                                    );
                                    let close_manager = manager.clone();
                                    runtime.block_on(async move {
                                        let _ = close_manager
                                            .handle_child_exit(session_id, io_generation)
                                            .await;
                                    });
                                    break;
                                }
                            }
                            Err(error) => {
                                warn!("PTY reader stopped for session {}: {}", session_id, error);
                                let close_manager = manager.clone();
                                runtime.block_on(async move {
                                    let _ = close_manager
                                        .handle_child_exit(session_id, io_generation)
                                        .await;
                                });
                                break;
                            }
                        }
                    }
                });
                Ok(())
            }
            SessionIo::Serial(serial) => {
                serial.spawn_io(session_id, output_tx, manager);
                Ok(())
            }
        }
    }

    pub(super) fn writer(&self) -> SessionIoWriter {
        match self {
            SessionIo::Pty(_, writer) => SessionIoWriter::Pty(writer.clone()),
            SessionIo::Serial(serial) => SessionIoWriter::Serial(serial.writer_tx.clone()),
        }
    }

    pub(super) fn resize(&self, size: TerminalSize) -> Result<(), SessionError> {
        match self {
            SessionIo::Pty(pty, _) => pty
                .resize(size)
                .map_err(|error| SessionError::PtyError(error.to_string())),
            SessionIo::Serial(_) => Ok(()),
        }
    }

    pub(super) fn terminate(&mut self) -> Result<(), SessionError> {
        match self {
            SessionIo::Pty(pty, _) => pty
                .terminate()
                .map_err(|error| SessionError::PtyError(error.to_string())),
            SessionIo::Serial(serial) => serial.terminate(),
        }
    }
}

fn serial_data_bits(bits: u8) -> Result<serialport::DataBits, SessionError> {
    match bits {
        5 => Ok(serialport::DataBits::Five),
        6 => Ok(serialport::DataBits::Six),
        7 => Ok(serialport::DataBits::Seven),
        8 => Ok(serialport::DataBits::Eight),
        _ => Err(SessionError::PtyError(format!(
            "Unsupported UART data bits value: {}",
            bits
        ))),
    }
}

fn serial_parity(parity: &UartParity) -> serialport::Parity {
    match parity {
        UartParity::None => serialport::Parity::None,
        UartParity::Odd => serialport::Parity::Odd,
        UartParity::Even => serialport::Parity::Even,
    }
}

fn serial_stop_bits(stop_bits: &UartStopBits) -> serialport::StopBits {
    match stop_bits {
        UartStopBits::One => serialport::StopBits::One,
        UartStopBits::Two => serialport::StopBits::Two,
    }
}

fn serial_flow_control(flow_control: &UartFlowControl) -> serialport::FlowControl {
    match flow_control {
        UartFlowControl::None => serialport::FlowControl::None,
        UartFlowControl::Software => serialport::FlowControl::Software,
        UartFlowControl::Hardware => serialport::FlowControl::Hardware,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{send_output_with_timeout, try_flush_output};
    use tokio::sync::mpsc;

    #[test]
    fn full_output_queue_preserves_pending_bytes_without_blocking() {
        let (tx, _rx) = mpsc::channel(1);
        tx.try_send(vec![1]).unwrap();
        let mut pending = vec![2, 3, 4];

        assert!(try_flush_output(&mut pending, &tx));
        assert_eq!(pending, vec![2, 3, 4]);
    }

    #[test]
    fn closed_output_queue_is_reported_to_reader() {
        let (tx, rx) = mpsc::channel(1);
        drop(rx);
        let mut pending = vec![1, 2, 3];

        assert!(!try_flush_output(&mut pending, &tx));
    }

    #[test]
    fn pty_output_wait_is_bounded_when_queue_stays_full() {
        let (tx, _rx) = mpsc::channel(1);
        tx.try_send(vec![1]).unwrap();

        assert!(!send_output_with_timeout(&tx, vec![2], Duration::ZERO));
    }
}
