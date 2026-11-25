use std::collections::{HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tracing::warn;
use vterm_common::SessionId;

const SCREEN_LOG_QUEUE_CAPACITY: usize = 1024;
const SCREEN_LOG_FLUSH_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Clone)]
pub(super) struct ScreenLogWriter {
    tx: mpsc::SyncSender<ScreenLogCommand>,
    dropped_commands: Arc<AtomicU64>,
    closed_sessions: Arc<std::sync::Mutex<HashSet<SessionId>>>,
}

enum ScreenLogCommand {
    Append {
        session_id: SessionId,
        lines: Vec<String>,
    },
    Close(SessionId),
}

impl ScreenLogWriter {
    pub(super) fn new() -> Self {
        let (tx, rx) = mpsc::sync_channel(SCREEN_LOG_QUEUE_CAPACITY);
        let dropped_commands = Arc::new(AtomicU64::new(0));
        std::thread::Builder::new()
            .name("vterm-screen-log".to_string())
            .spawn(move || run_screen_log_writer(rx))
            .expect("failed to spawn screen log writer");
        Self {
            tx,
            dropped_commands,
            closed_sessions: Arc::new(std::sync::Mutex::new(HashSet::new())),
        }
    }

    pub(super) fn append(&self, session_id: SessionId, lines: Vec<String>) {
        if lines.is_empty() {
            return;
        }
        let closed_sessions = match self.closed_sessions.try_lock() {
            Ok(closed_sessions) => closed_sessions,
            Err(std::sync::TryLockError::WouldBlock) => {
                self.record_dropped_batch(session_id, "session close in progress");
                return;
            }
            Err(std::sync::TryLockError::Poisoned(_)) => return,
        };
        if closed_sessions.contains(&session_id) {
            return;
        }
        match self
            .tx
            .try_send(ScreenLogCommand::Append { session_id, lines })
        {
            Ok(()) => {}
            Err(mpsc::TrySendError::Full(_)) => {
                self.record_dropped_batch(session_id, "queue full");
            }
            Err(mpsc::TrySendError::Disconnected(_)) => {
                warn!("Screen log writer stopped before session {session_id} output was recorded");
            }
        }
        drop(closed_sessions);
    }

    fn record_dropped_batch(&self, session_id: SessionId, reason: &str) {
        let dropped = self.dropped_commands.fetch_add(1, Ordering::Relaxed) + 1;
        if dropped == 1 || dropped.is_power_of_two() {
            warn!(
                "Dropped {} screen log batches (latest session {}, reason: {})",
                dropped, session_id, reason
            );
        }
    }

    pub(super) fn close(&self, session_id: SessionId) {
        // Session close is not on the output hot path. Ensure the close marker
        // is ordered after prior appends so the writer is flushed and removed.
        let Ok(mut closed_sessions) = self.closed_sessions.lock() else {
            return;
        };
        closed_sessions.insert(session_id);
        let _ = self.tx.send(ScreenLogCommand::Close(session_id));
    }
}

fn run_screen_log_writer(rx: mpsc::Receiver<ScreenLogCommand>) {
    let mut writers = HashMap::<SessionId, BufWriter<File>>::new();
    let mut next_flush = Instant::now() + SCREEN_LOG_FLUSH_INTERVAL;
    loop {
        let wait = next_flush.saturating_duration_since(Instant::now());
        match rx.recv_timeout(wait) {
            Ok(command) => match command {
                ScreenLogCommand::Append { session_id, lines } => {
                    if let Err(error) = append_lines(&mut writers, session_id, &lines) {
                        writers.remove(&session_id);
                        warn!(
                            "Failed to append screen history log for session {session_id}: {error}"
                        );
                    }
                }
                ScreenLogCommand::Close(session_id) => {
                    if let Some(mut writer) = writers.remove(&session_id) {
                        if let Err(error) = writer.flush() {
                            warn!(
                                "Failed to flush screen history log for session {session_id}: {error}"
                            );
                        }
                    }
                }
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                flush_writers(&mut writers);
                break;
            }
        }

        if Instant::now() >= next_flush {
            flush_writers(&mut writers);
            next_flush = Instant::now() + SCREEN_LOG_FLUSH_INTERVAL;
        }
    }
}

fn flush_writers(writers: &mut HashMap<SessionId, BufWriter<File>>) {
    for (session_id, writer) in writers {
        if let Err(error) = writer.flush() {
            warn!("Failed to flush screen history log for session {session_id}: {error}");
        }
    }
}

fn append_lines(
    writers: &mut HashMap<SessionId, BufWriter<File>>,
    session_id: SessionId,
    lines: &[String],
) -> std::io::Result<()> {
    if !writers.contains_key(&session_id) {
        let path = super::session_log_dir(session_id).join("screen.log");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        writers.insert(session_id, BufWriter::new(file));
    }

    let writer = writers
        .get_mut(&session_id)
        .expect("screen log writer inserted above");
    for line in lines {
        writer.write_all(line.as_bytes())?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}
