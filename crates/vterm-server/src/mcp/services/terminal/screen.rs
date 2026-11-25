//! Pure screen-snapshot and line-level diff logic for the MCP server.
//!
//! These helpers form the reusable core that the MCP tools (Task 5) and
//! resources (Task 6) will consume. Until those land, the public items are
//! only exercised by tests in this module, so we silence `dead_code` here.
#![allow(dead_code)]

use vterm_core::emulator::Terminal;

/// A snapshot of a terminal's visible screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenSnapshot {
    pub text: String,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub rows: u16,
    pub cols: u16,
}

/// One line-level change between two snapshots. Row indices refer to the
/// target (new) snapshot's line numbering.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase", tag = "op")]
pub enum LineChange {
    /// Replace the contents of a row.
    Replace { row: usize, text: String },
    /// Append a new row at the end.
    Append { text: String },
}

/// Compute the line-level diff from `prev` to `curr`. Each line is compared
/// independently; a row that differs (or exists only in `curr`) becomes a
/// `Replace` (for shared indices) or `Append` (for extra trailing lines).
/// If the screens differ on more than `full_sync_threshold` fraction of rows
/// (0.0–1.0), returns `None` to signal "send full text instead".
pub fn diff_lines(
    prev: &ScreenSnapshot,
    curr: &ScreenSnapshot,
    full_sync_threshold: f32,
) -> Option<Vec<LineChange>> {
    let prev_lines: Vec<&str> = prev.text.split('\n').collect();
    let curr_lines: Vec<&str> = curr.text.split('\n').collect();
    let max_len = prev_lines.len().max(curr_lines.len());
    let mut changes = Vec::new();
    let mut changed = 0usize;
    for i in 0..max_len {
        match (prev_lines.get(i), curr_lines.get(i)) {
            (Some(p), Some(c)) if p == c => {}
            (Some(_), Some(c)) => {
                changed += 1;
                changes.push(LineChange::Replace {
                    row: i,
                    text: (*c).to_string(),
                });
            }
            (None, Some(c)) => {
                changed += 1;
                changes.push(LineChange::Append {
                    text: (*c).to_string(),
                });
            }
            _ => {}
        }
    }
    let denom = max_len.max(1) as f32;
    if !curr_lines.is_empty() && (changed as f32 / denom) > full_sync_threshold {
        return None;
    }
    Some(changes)
}

/// Build a snapshot from a live `Terminal`, using its visible (unwrapped) text.
pub fn snapshot_from_terminal(terminal: &Terminal) -> ScreenSnapshot {
    let text = terminal.unwrapped_screen_text();
    let (cursor_row, cursor_col) = terminal.cursor_pos();
    let size = terminal.size();
    ScreenSnapshot {
        text,
        cursor_row,
        cursor_col,
        rows: size.rows,
        cols: size.cols,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vterm_common::TerminalSize;

    fn snap(text: &str, rows: u16, cols: u16) -> ScreenSnapshot {
        ScreenSnapshot {
            text: text.to_string(),
            cursor_row: 0,
            cursor_col: 0,
            rows,
            cols,
        }
    }

    #[test]
    fn diff_no_change_returns_empty() {
        let a = snap("a\nb\nc", 3, 10);
        assert_eq!(diff_lines(&a, &a, 0.5).unwrap(), vec![]);
    }

    #[test]
    fn diff_replace_one_line() {
        let prev = snap("a\nb\nc", 3, 10);
        let curr = snap("a\nB\nc", 3, 10);
        let changes = diff_lines(&prev, &curr, 0.5).unwrap();
        assert_eq!(
            changes,
            vec![LineChange::Replace {
                row: 1,
                text: "B".to_string()
            }]
        );
    }

    #[test]
    fn diff_append_line() {
        let prev = snap("a\nb", 2, 10);
        let curr = snap("a\nb\nc", 3, 10);
        let changes = diff_lines(&prev, &curr, 0.5).unwrap();
        assert_eq!(
            changes,
            vec![LineChange::Append {
                text: "c".to_string()
            }]
        );
    }

    #[test]
    fn diff_full_sync_when_too_many_changes() {
        let prev = snap("a\nb\nc\nd", 4, 10);
        let curr = snap("X\nY\nZ\nW", 4, 10);
        assert!(diff_lines(&prev, &curr, 0.5).is_none());
    }

    #[test]
    fn diff_threshold_allows_half() {
        let prev = snap("a\nb\nc\nd", 4, 10);
        let curr = snap("a\nB\nc\nD", 4, 10);
        // 2/4 == 0.5, not > 0.5, so diff is returned
        assert!(diff_lines(&prev, &curr, 0.5).is_some());
    }

    #[test]
    fn snapshot_captures_grid_text_and_size() {
        let mut term = Terminal::new(TerminalSize::new(3, 10));
        term.process(b"hello\r\nworld");
        let snap = snapshot_from_terminal(&term);
        assert!(snap.text.contains("hello"), "text was: {}", snap.text);
        assert_eq!(snap.rows, 3);
        assert_eq!(snap.cols, 10);
    }
}
