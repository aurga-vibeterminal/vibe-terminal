//! In-flight git operation tracking.
//!
//! Every git process spawned by this module goes through [`GitCommand`] (the
//! return type of `super::git_command()`), which registers each invocation in
//! a process-wide registry until the child exits. This exists to diagnose
//! `index.lock` contention ("Unable to create '.../.git/index.lock': File
//! exists"): when a git command fails that way, the log names every other git
//! operation this process is still running — or states that the lock was left
//! by something outside this process. Operations that run longer than
//! [`SLOW_OP_THRESHOLD`] are also logged when they finish, so a hung git
//! process is identifiable after the fact.
//!
//! The logs are opt-in: nothing is emitted unless the [`TRACKER_LOG_ENV`]
//! environment variable is set to a non-empty value.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::io;
use std::panic::Location;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, ExitStatus, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use tracing::warn;

/// Invocations still running after this long are logged when they finish.
const SLOW_OP_THRESHOLD: Duration = Duration::from_secs(30);

/// Environment variable opting in to the `[git]` tracking logs below. Unset
/// (the default) or empty keeps the server quiet; any other value, e.g.
/// `TERMINAL_SERVER_GIT_TRACKER_LOG=1`, enables them.
const TRACKER_LOG_ENV: &str = "TERMINAL_SERVER_GIT_TRACKER_LOG";

static TRACKING_ENABLED: LazyLock<bool> =
    LazyLock::new(|| std::env::var(TRACKER_LOG_ENV).is_ok_and(|value| !value.is_empty()));

struct GitOpInfo {
    summary: String,
    cwd: String,
    caller: &'static Location<'static>,
    started: Instant,
}

static NEXT_OP_ID: AtomicU64 = AtomicU64::new(1);
static IN_FLIGHT: LazyLock<Mutex<HashMap<u64, GitOpInfo>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Render a command as `"git <args…>"` plus its working directory, for logs.
fn summarize(command: &Command) -> (String, String) {
    let mut summary = command.get_program().to_string_lossy().into_owned();
    for arg in command.get_args() {
        summary.push(' ');
        summary.push_str(&arg.to_string_lossy());
    }
    let cwd = command
        .get_current_dir()
        .map(|dir| dir.display().to_string())
        .unwrap_or_else(|| "<inherit>".to_string());
    (summary, cwd)
}

/// RAII handle keeping one invocation registered until it is reaped.
struct GitOpGuard {
    id: u64,
    summary: String,
    cwd: String,
    started: Instant,
}

fn register(command: &Command, caller: &'static Location<'static>) -> GitOpGuard {
    let id = NEXT_OP_ID.fetch_add(1, Ordering::Relaxed);
    let (summary, cwd) = summarize(command);
    let started = Instant::now();
    if let Ok(mut ops) = IN_FLIGHT.lock() {
        if *TRACKING_ENABLED {
            for (other_id, other) in ops.iter() {
                if other.cwd == cwd {
                    warn!(
                        "[git] op#{id} `{summary}` starting in {cwd} while op#{other_id} `{}` has \
                         been running for {:?} (from {}:{})",
                        other.summary,
                        other.started.elapsed(),
                        other.caller.file(),
                        other.caller.line(),
                    );
                }
            }
        }
        ops.insert(
            id,
            GitOpInfo {
                summary: summary.clone(),
                cwd: cwd.clone(),
                caller,
                started,
            },
        );
    }
    // A poisoned registry mutex must not break git itself — tracking just
    // degrades for this invocation.
    GitOpGuard {
        id,
        summary,
        cwd,
        started,
    }
}

impl GitOpGuard {
    /// When this invocation failed because a git lock file already exists,
    /// log the other operations this process is still running. An empty
    /// in-flight list means the lock came from outside this process (an
    /// editor, a terminal git, or an earlier crashed run).
    fn report_if_lock_contention(&self, output: &Output) {
        if output.status.success() || !*TRACKING_ENABLED {
            return;
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains(".lock") || !stderr.contains("File exists") {
            return;
        }
        let others: Vec<String> = match IN_FLIGHT.lock() {
            Ok(ops) => ops
                .iter()
                .filter(|(id, _)| **id != self.id)
                .map(|(id, op)| {
                    format!(
                        "op#{id} `{}` in {} running for {:?} (from {}:{})",
                        op.summary,
                        op.cwd,
                        op.started.elapsed(),
                        op.caller.file(),
                        op.caller.line(),
                    )
                })
                .collect(),
            Err(_) => Vec::new(),
        };
        if others.is_empty() {
            warn!(
                "[git] op#{} `{}` in {} failed on a stale lock with no other git op in flight \
                 in this process — the lock was likely left by an external git process or an \
                 earlier crash: {}",
                self.id,
                self.summary,
                self.cwd,
                stderr.trim(),
            );
        } else {
            warn!(
                "[git] op#{} `{}` in {} failed on lock contention; in-flight git op(s): [{}]; \
                 error: {}",
                self.id,
                self.summary,
                self.cwd,
                others.join("; "),
                stderr.trim(),
            );
        }
    }
}

impl Drop for GitOpGuard {
    fn drop(&mut self) {
        if let Ok(mut ops) = IN_FLIGHT.lock() {
            ops.remove(&self.id);
        }
        let elapsed = self.started.elapsed();
        if *TRACKING_ENABLED && elapsed >= SLOW_OP_THRESHOLD {
            warn!(
                "[git] op#{} `{}` in {} took {:?} to finish",
                self.id, self.summary, self.cwd, elapsed,
            );
        }
    }
}

/// `std::process::Command` wrapper returned by `super::git_command()`.
/// Mirrors the builder/terminal methods used by this module so call sites are
/// unchanged, and tracks every spawned process in the in-flight registry.
pub(super) struct GitCommand {
    inner: Command,
}

impl GitCommand {
    pub(super) fn new(inner: Command) -> Self {
        Self { inner }
    }

    pub(super) fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.inner.arg(arg);
        self
    }

    pub(super) fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.inner.args(args);
        self
    }

    pub(super) fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.inner.current_dir(dir);
        self
    }

    pub(super) fn env<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.inner.env(key, value);
        self
    }

    pub(super) fn stdin<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.inner.stdin(cfg);
        self
    }

    pub(super) fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.inner.stdout(cfg);
        self
    }

    pub(super) fn stderr<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.inner.stderr(cfg);
        self
    }

    #[track_caller]
    pub(super) fn output(&mut self) -> io::Result<Output> {
        let guard = register(&self.inner, Location::caller());
        let output = self.inner.output();
        if let Ok(output) = &output {
            guard.report_if_lock_contention(output);
        }
        output
    }

    /// Like [`Self::output`], but kills the child and returns
    /// [`io::ErrorKind::TimedOut`] when it runs longer than `timeout`.
    /// Used for network-bound operations (fetch/pull/push) so a stalled
    /// remote cannot park a `spawn_blocking` thread forever.
    #[track_caller]
    pub(super) fn output_with_timeout(&mut self, timeout: Duration) -> io::Result<Output> {
        let guard = register(&self.inner, Location::caller());
        self.inner.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = self.inner.spawn()?;
        let started = Instant::now();
        loop {
            if child.try_wait()?.is_some() {
                let output = child.wait_with_output()?;
                guard.report_if_lock_contention(&output);
                return Ok(output);
            }
            if started.elapsed() >= timeout {
                let _ = child.kill();
                let _ = child.wait();
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("git operation timed out after {timeout:?}"),
                ));
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    #[track_caller]
    pub(super) fn status(&mut self) -> io::Result<ExitStatus> {
        let _guard = register(&self.inner, Location::caller());
        self.inner.status()
    }

    #[track_caller]
    pub(super) fn spawn(&mut self) -> io::Result<GitChild> {
        let guard = register(&self.inner, Location::caller());
        // On spawn failure the guard drops here and unregisters immediately.
        let mut child = self.inner.spawn()?;
        let stdin = child.stdin.take();
        Ok(GitChild {
            inner: child,
            stdin,
            _guard: guard,
        })
    }
}

/// `std::process::Child` wrapper keeping the op registered until the child is
/// reaped (or the handle is dropped).
pub(super) struct GitChild {
    inner: Child,
    /// Piped stdin of the child, mirroring `Child::stdin`.
    pub stdin: Option<ChildStdin>,
    _guard: GitOpGuard,
}

impl GitChild {
    pub(super) fn wait_with_output(self) -> io::Result<Output> {
        let output = self.inner.wait_with_output();
        if let Ok(output) = &output {
            self._guard.report_if_lock_contention(output);
        }
        output
    }
}
