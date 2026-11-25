//! Unlock the local desktop session when a client starts a screen stream
//! (Linux, systemd-logind).
//!
//! GNOME's compositor (mutter) forbids screen casting while the lock screen
//! is up — it refuses new sessions and tears down running streams, so a
//! client connecting to a locked host sees nothing. The daemon therefore
//! always unlocks its own session via `loginctl` before the capture starts:
//! connecting remotely then means "unlock this machine", which is exactly
//! the trade-off this behavior documents. Everything is a no-op off Linux
//! / without loginctl.

use anyhow::{Context, Result};
use tracing::{debug, info, warn};

/// How long to wait for the session to report unlocked after `unlock-session`.
#[cfg(target_os = "linux")]
const UNLOCK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
#[cfg(target_os = "linux")]
const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(200);

/// The sessions whose lock state matters: `$XDG_SESSION_ID` when the process
/// inherited one; otherwise every session owned by our uid from
/// `loginctl list-sessions`. A daemon spawned outside the login session (IDE
/// task runner, setsid, a lingering tmux) has no `XDG_SESSION_ID`, and
/// `loginctl show-session` without an id then answers empty garbage instead
/// of the real lock state — so the fallback enumerates ours explicitly.
#[cfg(target_os = "linux")]
async fn candidate_sessions() -> Vec<String> {
    if let Ok(id) = std::env::var("XDG_SESSION_ID") {
        let id = id.trim();
        if !id.is_empty() {
            return vec![id.to_string()];
        }
    }
    let our_uid = match tokio::process::Command::new("id").arg("-u").output().await {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => return Vec::new(),
    };
    let Ok(output) = tokio::process::Command::new("loginctl")
        .args(["list-sessions", "--no-legend"])
        .output()
        .await
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    parse_session_list(&String::from_utf8_lossy(&output.stdout), &our_uid)
}

/// The session ids of `uid` from `loginctl list-sessions --no-legend` output
/// (columns: SESSION UID USER SEAT [TTY] ...).
#[cfg(target_os = "linux")]
fn parse_session_list(output: &str, uid: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let mut cols = line.split_whitespace();
            let session = cols.next()?;
            (cols.next()? == uid).then(|| session.to_string())
        })
        .collect()
}

/// One session's `LockedHint`: `Some(bool)` on a definitive yes/no answer,
/// `None` when indeterminate (command failed or answered empty — an empty
/// answer is NOT "unlocked").
#[cfg(target_os = "linux")]
async fn locked_hint(session: &str) -> Option<bool> {
    let output = tokio::process::Command::new("loginctl")
        .args(["show-session", session, "--property=LockedHint", "--value"])
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    match String::from_utf8_lossy(&output.stdout).trim() {
        "yes" => Some(true),
        "no" => Some(false),
        _ => None,
    }
}

/// Unlock the caller's session if it is locked, waiting for the compositor
/// to drop the lock screen (mutter only permits screen casting again once
/// `LockedHint` clears). Returns `true` when the session ends up unlocked
/// (or already was), `false` on timeout / indeterminate state.
#[cfg(target_os = "linux")]
pub async fn ensure_unlocked() -> Result<bool> {
    let sessions = candidate_sessions().await;
    if sessions.is_empty() {
        debug!("no logind session found for this process; skipping the unlock attempt");
        return Ok(false);
    }
    let mut locked = Vec::new();
    let mut determinate = false;
    for session in &sessions {
        match locked_hint(session).await {
            Some(true) => {
                determinate = true;
                locked.push(session.clone());
            }
            Some(false) => determinate = true,
            None => {}
        }
    }
    if locked.is_empty() {
        if !determinate {
            debug!("logind LockedHint indeterminate; skipping the unlock attempt");
            return Ok(false);
        }
        return Ok(true);
    }

    info!(
        ?locked,
        "session is locked; unlocking for the incoming screen stream"
    );
    for session in &locked {
        let status = tokio::process::Command::new("loginctl")
            .args(["unlock-session", session])
            .output()
            .await
            .context("failed to run loginctl unlock-session")?
            .status;
        if !status.success() {
            warn!("loginctl unlock-session {session} exited with {status}");
        }
    }

    let deadline = std::time::Instant::now() + UNLOCK_TIMEOUT;
    loop {
        tokio::time::sleep(POLL_INTERVAL).await;
        let mut still_locked = false;
        for session in &locked {
            if locked_hint(session).await == Some(true) {
                still_locked = true;
            }
        }
        if !still_locked {
            return Ok(true);
        }
        if std::time::Instant::now() >= deadline {
            warn!(
                "session still locked {}s after unlock-session",
                UNLOCK_TIMEOUT.as_secs()
            );
            return Ok(false);
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub async fn ensure_unlocked() -> Result<bool> {
    Ok(true)
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::parse_session_list;

    #[test]
    fn parse_session_list_filters_by_uid() {
        let output = " 2 1000 user seat0 tty2\n 3 1000 user -\n 7 1001 other seat0\n";
        assert_eq!(parse_session_list(output, "1000"), vec!["2", "3"]);
        assert_eq!(parse_session_list(output, "1001"), vec!["7"]);
        assert!(parse_session_list(output, "0").is_empty());
        assert!(parse_session_list("", "1000").is_empty());
    }
}
