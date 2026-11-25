use anyhow::{Context, Result};
use crossterm::event::{DisableBracketedPaste, EnableBracketedPaste};
use crossterm::execute;
use crossterm::terminal;
use vterm_common::TerminalSize;

/// Terminal information and control
pub struct Terminal;

impl Terminal {
    /// Get current terminal size
    pub fn size() -> Result<TerminalSize> {
        let (cols, rows) = terminal::size().context("Failed to get terminal size")?;

        Ok(TerminalSize {
            rows: rows as u16,
            cols: cols as u16,
        })
    }

    /// Enable raw mode for the terminal
    pub fn enable_raw_mode() -> Result<()> {
        terminal::enable_raw_mode().context("Failed to enable raw mode")?;
        execute!(std::io::stdout(), EnableBracketedPaste)
            .context("Failed to enable bracketed paste")?;
        Ok(())
    }

    /// Disable raw mode for the terminal
    pub fn disable_raw_mode() -> Result<()> {
        execute!(std::io::stdout(), DisableBracketedPaste)
            .context("Failed to disable bracketed paste")?;
        terminal::disable_raw_mode().context("Failed to disable raw mode")?;
        Ok(())
    }

    /// Check if stdout is a TTY
    pub fn is_tty() -> bool {
        terminal::is_raw_mode_enabled().unwrap_or(false) || atty::is(atty::Stream::Stdout)
    }

    /// Check if stdin is a TTY
    pub fn is_stdin_tty() -> bool {
        atty::is(atty::Stream::Stdin)
    }
}

/// Install a resize signal handler
#[cfg(unix)]
pub fn on_resize<F>(mut callback: F) -> Result<()>
where
    F: FnMut(TerminalSize) + Send + 'static,
{
    use signal_hook::consts::SIGWINCH;
    use signal_hook::iterator::Signals;

    let mut signals = Signals::new(&[SIGWINCH]).context("Failed to create signal handler")?;

    std::thread::spawn(move || {
        for _ in signals.forever() {
            if let Ok(size) = Terminal::size() {
                callback(size);
            }
        }
    });

    Ok(())
}

/// On Windows, we poll for size changes
#[cfg(windows)]
pub fn on_resize<F>(mut callback: F) -> Result<()>
where
    F: FnMut(TerminalSize) + Send + 'static,
{
    use std::thread;
    use std::time::Duration;

    thread::spawn(move || {
        let mut last_size = Terminal::size().unwrap_or(TerminalSize { rows: 24, cols: 80 });

        loop {
            thread::sleep(Duration::from_millis(500));

            if let Ok(new_size) = Terminal::size() {
                if new_size.rows != last_size.rows || new_size.cols != last_size.cols {
                    last_size = new_size;
                    callback(new_size);
                }
            }
        }
    });

    Ok(())
}

/// Clean up terminal on exit
pub struct TerminalGuard;

impl TerminalGuard {
    pub fn new() -> Result<Self> {
        Terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = Terminal::disable_raw_mode();
    }
}

/// Simple atty replacement for checking if stream is a TTY
mod atty {
    #[derive(Clone, Copy)]
    pub enum Stream {
        Stdin,
        Stdout,
        Stderr,
    }

    #[cfg(unix)]
    pub fn is(stream: Stream) -> bool {
        use libc::{isatty, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
        unsafe {
            let fd = match stream {
                Stream::Stdin => STDIN_FILENO,
                Stream::Stdout => STDOUT_FILENO,
                Stream::Stderr => STDERR_FILENO,
            };
            isatty(fd) == 1
        }
    }

    #[cfg(windows)]
    pub fn is(stream: Stream) -> bool {
        use windows_sys::Win32::System::Console::{
            GetConsoleMode, GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
        };
        unsafe {
            let handle = match stream {
                Stream::Stdin => GetStdHandle(STD_INPUT_HANDLE),
                Stream::Stdout => GetStdHandle(STD_OUTPUT_HANDLE),
                Stream::Stderr => GetStdHandle(STD_ERROR_HANDLE),
            };

            let mut mode = 0;
            GetConsoleMode(handle, &mut mode) != 0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_size() {
        // This will fail if not run in a terminal
        let size = Terminal::size();
        println!("Terminal size: {:?}", size);
    }
}
