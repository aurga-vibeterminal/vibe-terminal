//! Startup diagnostics for vterm-server: mirror tracing output into a
//! per-run log file under the build flavor's log directory while keeping the
//! console output the server has always had. File logging follows the
//! persisted `server.log_to_file` setting ([`init`]'s `log_to_file`);
//! console output is unconditional. All sink I/O is offloaded to
//! [`vterm_common::AsyncLogWriter`]'s dedicated writer thread so logging
//! never blocks connection or session threads; exit paths flush through
//! [`flush_and_exit`]. Panics and native crashes are captured into
//! `fatal_*.log` files in the same directory by
//! [`vterm_common::diagnostics::install_crash_handlers`].

use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use chrono::Local;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init(log_to_file: bool) -> (PathBuf, Option<PathBuf>) {
    let log_dir = server_log_dir();
    // Panics and (Windows SEH / unix fatal signals) crashes land in
    // `fatal_*.log` files. Unlike the GUI client the server does *not*
    // abort on panic: Tokio isolates a panicking task, and aborting would
    // take down every live session over one session's bug. Crash logs are
    // independent of `log_to_file`: they exist precisely for the cases the
    // regular log cannot capture.
    vterm_common::diagnostics::install_crash_handlers(
        log_dir.clone(),
        env!("CARGO_PKG_VERSION"),
        false,
    );
    let tracing_log = init_tracing(&log_dir, log_to_file);
    // Cleanup still runs with file logging disabled: it purges the files
    // written before the setting was turned off.
    spawn_log_cleanup(log_dir.clone());
    (log_dir, tracing_log)
}

/// Per-run log files accumulate forever; a background thread drops files
/// past [`vterm_common::LOG_RETENTION`] so startup I/O stays off the accept
/// path. The current run's file is never a candidate (it was just created).
fn spawn_log_cleanup(log_dir: PathBuf) {
    std::thread::Builder::new()
        .name("vterm-log-cleanup".into())
        .spawn(move || {
            let (removed, failed) =
                vterm_common::cleanup_old_logs(&log_dir, vterm_common::LOG_RETENTION);
            if removed > 0 || failed > 0 {
                tracing::info!(
                    removed,
                    failed,
                    dir = %log_dir.display(),
                    "Cleaned up old server log files"
                );
            }
        })
        .ok();
}

fn server_log_dir() -> PathBuf {
    home_dir()
        .join(if cfg!(DEVELOPMENT) {
            ".vibeterminal-dev"
        } else {
            ".vibeterminal"
        })
        .join("logs")
        .join("vterm-server")
}

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(std::env::temp_dir)
}

fn file_timestamp() -> String {
    Local::now().format("%Y-%m-%d_%H-%M-%S_%3f").to_string()
}

/// Third-party infrastructure whose `debug`/`info` output is handshake,
/// protocol and directory-scan chatter rather than actionable diagnostics:
/// - rustls / tungstenite / tokio_tungstenite: TLS + WebSocket handshake
///   internals (cipher negotiation, session-cache probes, frame events)
/// - hyper / h2 / hyper_util / reqwest: outbound HTTP connection state
/// - globset / ignore: gitignore glob compilation and matching emitted by
///   directory walks (recent-project scan, file search) — tens of lines
///   per walked directory
/// - rmcp: MCP session lifecycle (per-client InitializeRequest dumps)
/// Capped at `warn` so the default `debug` log stays focused on this
/// crate's own output. `RUST_LOG` always overrides — e.g.
/// `RUST_LOG=debug,rustls=debug` to bring the TLS internals back while
/// debugging handshake issues.
const NOISY_INFRA_TARGETS: &str = "rustls=warn,tungstenite=warn,tokio_tungstenite=warn,\
hyper=warn,h2=warn,hyper_util=warn,reqwest=warn,\
globset=warn,ignore=warn,rmcp=warn";

fn env_filter() -> tracing_subscriber::EnvFilter {
    // Default to `debug` so the log carries useful diagnostic detail. The
    // highest-frequency, low-value chatter (latency probes, history fetches,
    // relay syncs, git refresh traces) is logged at `trace` instead, so it
    // stays out of the default output; set `RUST_LOG=trace` to see it.
    // Noisy third-party crates (see [`NOISY_INFRA_TARGETS`]) are capped at
    // `warn` in the default filter.
    tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        tracing_subscriber::EnvFilter::new(format!("debug,{NOISY_INFRA_TARGETS}"))
    })
}

/// The process-wide async log writer, kept so exit paths can flush pending
/// log lines before `std::process::exit` kills the writer thread.
static LOG_WRITER: std::sync::OnceLock<vterm_common::AsyncLogWriter> = std::sync::OnceLock::new();

/// Flush pending log lines and terminate. `std::process::exit` kills threads
/// without running destructors, so every server exit path must go through
/// here to keep its final log lines (shutdown, duplicate-instance, error
/// diagnostics) on disk.
pub fn flush_and_exit(code: i32) -> ! {
    if let Some(writer) = LOG_WRITER.get() {
        writer.flush_blocking();
    }
    std::process::exit(code)
}

fn init_tracing(log_dir: &Path, log_to_file: bool) -> Option<PathBuf> {
    if log_to_file {
        let log_path = log_dir.join(format!("vterm-server_{}.log", file_timestamp()));
        if let Ok(file) = open_log_file(&log_path) {
            // File first, then the console mirror the server has always had. All
            // sink I/O runs on the AsyncLogWriter's dedicated thread so logging
            // never blocks the accept or session threads.
            let writer = vterm_common::AsyncLogWriter::new(vec![
                Box::new(file),
                Box::new(std::io::stdout()),
            ]);
            let _ = LOG_WRITER.set(writer.clone());
            tracing_subscriber::fmt()
                .with_ansi(false)
                .with_env_filter(env_filter())
                .with_writer(move || writer.clone())
                .init();
            return Some(log_path);
        }

        eprintln!(
            "Failed to open log file {}; logging to the console only",
            log_path.display()
        );
    }

    tracing_subscriber::fmt()
        .with_env_filter(env_filter())
        .init();
    None
}

fn open_log_file(path: &Path) -> std::io::Result<File> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    OpenOptions::new().create(true).append(true).open(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drive the real installed hook through an actual panic and assert the
    /// fatal log lands with message and backtrace. `catch_unwind` works here
    /// precisely because the server policy does not abort (unlike the
    /// client's).
    #[test]
    fn panic_hook_writes_fatal_log_with_backtrace() {
        let dir = tempfile::tempdir().unwrap();
        vterm_common::diagnostics::install_crash_handlers(
            dir.path().to_path_buf(),
            env!("CARGO_PKG_VERSION"),
            false,
        );

        let panicked = std::panic::catch_unwind(|| panic!("fatal-log smoke test"));
        assert!(panicked.is_err());

        let mut saw_fatal_with_details = false;
        for entry in std::fs::read_dir(dir.path()).unwrap().flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with("fatal_") || !name.ends_with(".log") {
                continue;
            }
            let content = std::fs::read_to_string(entry.path()).unwrap();
            if content.contains("message: fatal-log smoke test") && content.contains("backtrace:") {
                saw_fatal_with_details = true;
            }
        }
        assert!(
            saw_fatal_with_details,
            "no fatal_*.log with the panic message and backtrace was written"
        );
    }
}
