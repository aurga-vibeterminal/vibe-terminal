//! MCP resource logic: terminal screen snapshot + throttled live-stream diffs.
//!
//! When live streaming is wired in (currently deferred — see the design spec),
//! the rmcp `ServerHandler` (in `handler.rs`) would subscribe a caller to
//! `terminal://<id>/screen/live` by spawning `stream_screen` with an mpsc
//! sender it drains into the SSE response.
//!
//! Until that wiring lands, this module's public items are only exercised by
//! tests, so we silence `dead_code` here.
#![allow(dead_code)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use tokio::sync::mpsc;
use vterm_common::SessionId;

use crate::session::SessionManager as VtSessionManager;

use super::screen::{diff_lines, snapshot_from_terminal, LineChange, ScreenSnapshot};

/// Push the initial full snapshot then incremental diffs (or full resyncs).
/// Above 50% changed rows, send a `Full` frame instead of a giant diff.
const FULL_SYNC_THRESHOLD: f32 = 0.5;

/// One frame pushed to a `terminal://<id>/screen/live` subscriber.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ScreenUpdate {
    /// Initial snapshot or a full resync (when too much changed).
    Full {
        terminal_id: String,
        text: String,
        cursor_row: usize,
        cursor_col: usize,
        rows: u16,
        cols: u16,
        timestamp: String,
    },
    /// Incremental line-level changes since the last frame.
    Update {
        terminal_id: String,
        changes: Vec<LineChange>,
        cursor_row: usize,
        cursor_col: usize,
        timestamp: String,
    },
}

/// Run a streaming subscription for one terminal. Pushes `ScreenUpdate` frames
/// to `out` as the terminal receives PTY output, with throttling. Returns when
/// the session closes, the subscriber drops `out`, or the listener channel ends.
///
/// `throttle_ms` is the minimum interval between pushes; bursts within a window
/// coalesce into one push (latest snapshot wins).
pub async fn stream_screen(
    session_manager: Arc<VtSessionManager>,
    session_id: SessionId,
    throttle_ms: u32,
    out: mpsc::UnboundedSender<ScreenUpdate>,
) {
    let throttle = Duration::from_millis(throttle_ms.max(1) as u64);
    // Full UUID string for `terminal_id` fields. NOTE: SessionId's Display impl
    // truncates to 8 hex chars, but MCP clients need the full UUID to round-trip
    // against `parse_session_id` (Uuid::parse_str) — use the inner Uuid.
    let terminal_id = session_id.0.to_string();

    // Register a screen-listener sink (Task 4 wired this into process_session_output).
    let (listener_tx, mut listener_rx) = mpsc::unbounded_channel::<SessionId>();
    if (session_manager
        .add_screen_listener(session_id, listener_tx.clone())
        .await)
        .is_err()
    {
        return; // session already gone
    }

    // Send an initial full snapshot so the subscriber has a starting state.
    let mut prev: Option<ScreenSnapshot> =
        match current_snapshot(&session_manager, session_id).await {
            Some(snap) => {
                let _ = out.send(full_frame(&terminal_id, &snap));
                Some(snap)
            }
            None => None,
        };

    let mut last_push = Instant::now();
    while let Some(_session_id) = listener_rx.recv().await {
        // Throttle: if we pushed too recently, drain bursts in the window and skip.
        if last_push.elapsed() < throttle {
            while listener_rx.try_recv().is_ok() {}
            continue;
        }

        let curr = match current_snapshot(&session_manager, session_id).await {
            Some(s) => s,
            None => break, // session closed
        };

        let frame = build_frame(&terminal_id, &prev, &curr);
        prev = Some(curr);
        if out.send(frame).is_err() {
            break; // subscriber gone
        }
        last_push = Instant::now();
    }

    // Cleanup: drop our listener so it doesn't leak.
    let _ = session_manager
        .remove_screen_listener(session_id, &listener_tx)
        .await;
}

fn build_frame(
    terminal_id: &str,
    prev: &Option<ScreenSnapshot>,
    curr: &ScreenSnapshot,
) -> ScreenUpdate {
    match prev {
        Some(p) => match diff_lines(p, curr, FULL_SYNC_THRESHOLD) {
            // Empty-changes still emit so cursor updates get through.
            Some(changes) => ScreenUpdate::Update {
                terminal_id: terminal_id.to_string(),
                changes,
                cursor_row: curr.cursor_row,
                cursor_col: curr.cursor_col,
                timestamp: now_iso(),
            },
            None => full_frame(terminal_id, curr),
        },
        None => full_frame(terminal_id, curr),
    }
}

fn full_frame(terminal_id: &str, curr: &ScreenSnapshot) -> ScreenUpdate {
    ScreenUpdate::Full {
        terminal_id: terminal_id.to_string(),
        text: curr.text.clone(),
        cursor_row: curr.cursor_row,
        cursor_col: curr.cursor_col,
        rows: curr.rows,
        cols: curr.cols,
        timestamp: now_iso(),
    }
}

async fn current_snapshot(
    session_manager: &Arc<VtSessionManager>,
    session_id: SessionId,
) -> Option<ScreenSnapshot> {
    let session = session_manager.get_session(session_id).await?;
    let s = session.lock().await;
    Some(snapshot_from_terminal(&s.terminal))
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_frame_no_prev_is_full() {
        let curr = ScreenSnapshot {
            text: "hi".into(),
            cursor_row: 0,
            cursor_col: 2,
            rows: 1,
            cols: 10,
        };
        let id = SessionId::new().0.to_string();
        let frame = build_frame(&id, &None, &curr);
        assert!(matches!(frame, ScreenUpdate::Full { .. }));
    }

    #[test]
    fn terminal_id_is_full_uuid() {
        // Guards against the SessionId-truncation regression (commit 9b2b385a).
        let session = SessionId::new();
        let curr = ScreenSnapshot {
            text: "x".into(),
            cursor_row: 0,
            cursor_col: 1,
            rows: 1,
            cols: 10,
        };
        let id_str = session.0.to_string();
        let frame = full_frame(&id_str, &curr);
        match frame {
            ScreenUpdate::Full { terminal_id, .. } => {
                assert_eq!(terminal_id, id_str);
                assert_eq!(terminal_id.len(), 36, "expected full hyphenated UUID");
            }
            _ => panic!("expected Full"),
        }
    }
}
