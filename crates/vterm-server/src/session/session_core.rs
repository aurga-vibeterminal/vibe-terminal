use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};
use tracing::info;

use vterm_common::{
    EnvironmentProfileInfo, SessionApp, SessionAppStatus, SessionId, SessionInfo,
    SessionTerminalOs, TerminalSize, TAB_GROUP_UNGROUPED_ID,
};
use vterm_core::{emulator::Terminal, protocol::Message};

use crate::session::terminal_observer::ObserverHandle;
use crate::session::util::{
    default_home_cwd, default_terminal_os, normalize_working_path, prompt_working_path,
};

use super::raw_history::RawHistoryBuffer;
use super::session_error::SessionError;
use super::session_io::SessionIo;
use crate::session::ObservedTerminalState;

/// Type alias for sender channel to a connection
pub type ConnectionTx = mpsc::UnboundedSender<Message>;

/// Notification channel for in-process screen subscribers (MCP streaming).
/// Sent the session's SessionId whenever the session receives new PTY output.
pub type ScreenListenerTx = tokio::sync::mpsc::UnboundedSender<SessionId>;

const SCREEN_HISTORY_MAX_LINES: usize = 1000;

fn select_git_working_path(current: Option<&str>, viewing: Option<&str>) -> Option<String> {
    viewing
        .filter(|path| !path.trim().is_empty())
        .or_else(|| current.filter(|path| !path.trim().is_empty()))
        .map(str::to_owned)
}

/// Maximum number of recent files retained per session (most-recent-first).
/// The file-editor dropdown scrolls when the list exceeds the visible area.
const MAX_RECENT_FILES: usize = 100;

/// Represents an active terminal session.
pub struct Session {
    pub id: SessionId,
    pub tab_id: SessionId,
    pub group_id: u32,
    pub tab_group_id: u32,
    pub sequence: u64,
    pub title: String,
    pub session_name: Option<String>,
    pub terminal_type: Option<String>,
    pub encoding: Option<String>,
    pub terminal_os: Option<SessionTerminalOs>,
    pub shell_kind: Option<ShellKind>,
    pub current_working_path: Option<String>,
    pub viewing_path: Option<String>,
    pub git_repo_root: Option<String>,
    /// Incremented when a directory-listing request starts. Only the latest
    /// request may move `viewing_path` when it finishes.
    pub path_listing_generation: u64,
    pub app: SessionApp,
    pub app_status: SessionAppStatus,
    /// One-line summary of the pending confirmation/selection prompt.
    /// `None` when `app_status` is not a confirm/select state.
    pub app_status_summary: Option<String>,
    pub viewers: Vec<ConnectionTx>,
    pub screen_listeners: Vec<ScreenListenerTx>,
    pub size: TerminalSize,
    pub pending_resize: Option<TerminalSize>,
    pub pending_resize_generation: u64,
    /// Grid sizes reported by each attached viewer, ordered by activity: the
    /// most recently active viewer (latest attach or resize) is last. The
    /// session emulator mirrors that viewer's view size (see
    /// `active_viewer_size`).
    pub viewer_sizes: Vec<(ConnectionTx, TerminalSize)>,
    pub raw_history: RawHistoryBuffer,
    pub terminal: Terminal,
    pub screen_history_lines: VecDeque<String>,
    pub screen_history_logged_rows: usize,
    pub io: SessionIo,
    pub profile_id: Option<String>,
    pub recent_files: Vec<String>,
    pub observer: Option<ObserverHandle>,
    pub notification_enabled: bool,
    pub auto_confirm_enabled: bool,
    pub temp_cleanup_paths: Vec<PathBuf>,
    /// Program the session's PTY child was launched with (empty for UART).
    /// Used to respawn the shell when an application kills its parent shell
    /// (e.g. opencode corrupting the ConPTY-hosted shell on exit).
    pub launch_program: String,
    pub launch_args: Vec<String>,
    /// Incremented each time the session's PTY is replaced (respawn). Exit
    /// events from a stale PTY carry an older generation and are ignored.
    pub io_generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellKind {
    Cmd,
    PowerShell,
    Zsh,
    Bash,
    Fish,
    Sh,
}

impl Session {
    /// Git follows the directory currently viewed in the left panel, falling
    /// back to the terminal cwd only while no viewing path is available.
    pub fn git_working_path(&self) -> Option<String> {
        select_git_working_path(
            self.current_working_path.as_deref(),
            self.viewing_path.as_deref(),
        )
    }

    pub fn git_mutation_working_path(&self) -> Option<String> {
        self.git_repo_root
            .clone()
            .or_else(|| self.git_working_path())
    }

    pub fn new(
        id: SessionId,
        tab_id: SessionId,
        group_id: u32,
        size: TerminalSize,
        title: String,
        terminal_type: Option<String>,
        encoding: Option<String>,
        terminal_os: Option<SessionTerminalOs>,
        shell_kind: Option<ShellKind>,
        current_working_path: Option<String>,
        io: SessionIo,
        profile_id: Option<String>,
        sequence: u64,
    ) -> Self {
        let current_working_path = current_working_path.or_else(|| default_home_cwd(terminal_os));
        Self {
            id,
            tab_id,
            group_id,
            tab_group_id: TAB_GROUP_UNGROUPED_ID,
            sequence,
            title,
            session_name: None,
            terminal_type: terminal_type.clone(),
            encoding: encoding.clone(),
            terminal_os,
            shell_kind,
            current_working_path: current_working_path.clone(),
            viewing_path: current_working_path,
            git_repo_root: None,
            path_listing_generation: 0,
            app: SessionApp::none(),
            app_status: SessionAppStatus::None,
            app_status_summary: None,
            viewers: Vec::new(),
            screen_listeners: Vec::new(),
            size,
            pending_resize: None,
            pending_resize_generation: 0,
            viewer_sizes: Vec::new(),
            raw_history: RawHistoryBuffer::new_default(),
            terminal: Terminal::new_with_type_and_encoding(
                size,
                terminal_type.as_deref(),
                encoding.as_deref(),
            ),
            screen_history_lines: VecDeque::with_capacity(SCREEN_HISTORY_MAX_LINES),
            screen_history_logged_rows: 0,
            io,
            profile_id,
            recent_files: Vec::new(),
            observer: None,
            notification_enabled: true,
            auto_confirm_enabled: true,
            temp_cleanup_paths: Vec::new(),
            launch_program: String::new(),
            launch_args: Vec::new(),
            io_generation: 0,
        }
    }

    pub fn add_viewer(&mut self, viewer_tx: ConnectionTx) -> u32 {
        if !self.viewers.iter().any(|tx| tx.same_channel(&viewer_tx)) {
            self.viewers.push(viewer_tx.clone());
            // A newly attached viewer becomes the most recently active one,
            // seeded with the session's current size until it reports its own.
            self.viewer_sizes.push((viewer_tx.clone(), self.size));
        }
        self.viewers.len() as u32
    }

    pub fn remove_viewer(&mut self, viewer_tx: &ConnectionTx) -> u32 {
        self.viewers.retain(|tx| !tx.same_channel(viewer_tx));
        self.viewer_sizes
            .retain(|(tx, _)| !tx.same_channel(viewer_tx));
        self.viewers.len() as u32
    }

    pub fn viewer_count(&self) -> u32 {
        self.viewers.len() as u32
    }

    pub fn buffer_info_message(&self) -> Message {
        Message::SessionBufferInfo {
            session_id: self.id,
            start_offset: self.raw_history.start_offset(),
            end_offset: self.raw_history.end_offset(),
        }
    }

    /// Initial messages for a viewer attaching to this session. A rendered
    /// snapshot of the current screen always precedes the buffer info: the
    /// client initially loads only the tail of the raw history, so without
    /// the snapshot, mode-setting sequences that scrolled out of that window
    /// (mouse tracking, bracketed paste, ...) would be lost on the rebuilt
    /// emulator — and when the retained raw history was trimmed, replaying
    /// the trimmed byte window cannot even reconstruct the visible screen
    /// (full-screen TUI apps would come up partially blank). The server
    /// therefore sends its authoritative grid and mode state rendered as
    /// ANSI bytes, replayable exactly at `base_offset`.
    pub fn attach_messages(&self) -> Vec<Message> {
        vec![
            Message::HistorySnapshot {
                session_id: self.id,
                base_offset: self.raw_history.end_offset(),
                data: self.terminal.render_snapshot(),
            },
            self.buffer_info_message(),
        ]
    }

    pub fn history_chunk_message(&self, before_offset: u64, max_bytes: usize) -> Message {
        let (start_offset, data) = self.raw_history.read_before(before_offset, max_bytes);
        Message::HistoryRawChunk {
            session_id: self.id,
            start_offset,
            end_offset: start_offset + data.len() as u64,
            data,
        }
    }

    pub fn info(&self) -> SessionInfo {
        SessionInfo {
            id: self.id,
            tab_id: self.tab_id,
            group_id: self.group_id,
            tab_group_id: self.tab_group_id,
            sequence: self.sequence,
            title: self.title.clone(),
            session_name: self.session_name.clone(),
            size: self.size,
            viewer_count: self.viewer_count(),
            current_working_path: self.current_working_path.clone(),
            viewing_path: self.viewing_path.clone(),
            terminal_type: self.terminal_type.clone(),
            encoding: self.encoding.clone(),
            is_uart: matches!(&self.io, SessionIo::Serial(_)),
            terminal_os: self.terminal_os,
            runtime_os: Some(default_terminal_os()),
            app: self.app.clone(),
            app_status: self.app_status,
            app_status_summary: self.app_status_summary.clone(),
            server_id: None,
            server_name: None,
            environments: Vec::<EnvironmentProfileInfo>::new(),
            environments_tree: Vec::<vterm_common::EnvironmentNodeInfo>::new(),
            recent_files: self.recent_files.clone(),
            notification_enabled: self.notification_enabled,
            auto_confirm_enabled: self.auto_confirm_enabled,
        }
    }

    pub fn has_viewer(&self, viewer_tx: &ConnectionTx) -> bool {
        self.viewers.iter().any(|tx| tx.same_channel(viewer_tx))
    }

    /// Effective PTY size: the grid reported by the most recently active
    /// viewer (latest attach or resize). The session emulator — the mirror
    /// that observers, MCP tooling, and reattaching viewers read — follows
    /// that viewer's view size, so the last valid viewer always wins and a
    /// stale or secondary viewer can never pin the size. Returns `None` when
    /// no viewer is attached.
    pub fn active_viewer_size(&self) -> Option<TerminalSize> {
        self.viewer_sizes.last().map(|(_, size)| *size)
    }

    /// Record a viewer's reported grid size and mark it as the most recently
    /// active viewer so the session emulator follows it.
    pub fn update_viewer_size(&mut self, viewer_tx: &ConnectionTx, size: TerminalSize) {
        let pos = self
            .viewer_sizes
            .iter()
            .position(|(tx, _)| tx.same_channel(viewer_tx));
        if let Some(pos) = pos {
            let mut entry = self.viewer_sizes.remove(pos);
            entry.1 = size;
            self.viewer_sizes.push(entry);
        }
    }

    /// Remember the newest requested size and invalidate older debounce tasks.
    pub fn defer_resize(&mut self, size: TerminalSize) -> Option<u64> {
        if self.size == size && self.pending_resize.is_none() {
            return None;
        }

        self.pending_resize = Some(size);
        self.pending_resize_generation = self.pending_resize_generation.wrapping_add(1);
        Some(self.pending_resize_generation)
    }

    /// Return the generation id associated with the latest deferred resize.
    pub fn pending_resize_generation(&self) -> u64 {
        self.pending_resize_generation
    }

    pub fn apply_terminal_output(
        &mut self,
        data: &[u8],
        capture_observed_state: bool,
    ) -> (bool, Option<ObservedTerminalState>) {
        self.terminal.process(data);

        let normalized_terminal_cwd = self
            .terminal
            .cwd()
            .and_then(|path| normalize_working_path(path, self.terminal_os));
        let current_line_text = (capture_observed_state || normalized_terminal_cwd.is_none())
            .then(|| self.terminal.current_line_text());
        let cwd_changed = self
            .refresh_current_working_path(normalized_terminal_cwd, current_line_text.as_deref());
        let observed_terminal_state = capture_observed_state.then(|| ObservedTerminalState {
            terminal_title: self.terminal.title().to_string(),
            current_line_text: current_line_text.unwrap_or_default(),
        });

        (cwd_changed, observed_terminal_state)
    }

    pub fn take_new_screen_history_lines(&mut self) -> Vec<String> {
        if self.screen_history_logged_rows > self.terminal.scrollback().len() {
            self.screen_history_logged_rows = self.terminal.scrollback().len();
        }

        let (logged_rows, lines) = self
            .terminal
            .completed_scrollback_text_from(self.screen_history_logged_rows);
        self.screen_history_logged_rows = logged_rows;

        for line in &lines {
            self.screen_history_lines.push_back(line.clone());
            while self.screen_history_lines.len() > SCREEN_HISTORY_MAX_LINES {
                self.screen_history_lines.pop_front();
            }
        }

        lines
    }

    pub fn resize_terminal(&mut self, size: TerminalSize) {
        self.terminal.resize(size);
    }

    /// Apply the latest deferred resize to both the server terminal model and the PTY.
    pub fn flush_pending_resize(
        &mut self,
    ) -> Result<Option<(TerminalSize, TerminalSize)>, SessionError> {
        let Some(size) = self.pending_resize.take() else {
            return Ok(None);
        };
        if self.size == size {
            return Ok(None);
        }

        let old_size = self.size;
        self.resize_terminal(size);
        self.io
            .resize(size)
            .map_err(|error| SessionError::PtyError(error.to_string()))?;
        self.size = size;
        Ok(Some((old_size, size)))
    }

    pub fn record_recent_file(&mut self, path: String) {
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);
        self.recent_files.truncate(MAX_RECENT_FILES);
    }

    fn refresh_current_working_path(
        &mut self,
        normalized_terminal_cwd: Option<String>,
        current_line_text: Option<&str>,
    ) -> bool {
        let osc7_changed = self.terminal.take_cwd_dirty();
        let next_cwd = normalized_terminal_cwd.or_else(|| {
            current_line_text.and_then(|line| prompt_working_path(line, self.terminal_os))
        });
        if self.current_working_path == next_cwd {
            return false;
        }

        if !osc7_changed && next_cwd.is_none() {
            return false;
        }

        let follow_cwd =
            self.viewing_path.is_none() || self.viewing_path == self.current_working_path;
        info!(
            "Session {} current working path changed: {:?} -> {:?}",
            self.id, self.current_working_path, next_cwd
        );
        self.current_working_path = next_cwd.clone();
        if follow_cwd {
            self.viewing_path = next_cwd;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::select_git_working_path;

    #[test]
    fn git_working_path_prefers_viewing_path_and_falls_back_to_cwd() {
        assert_eq!(
            select_git_working_path(Some("/repo"), Some("/repo/submodule")),
            Some("/repo/submodule".to_string())
        );
        assert_eq!(
            select_git_working_path(Some("/repo"), None),
            Some("/repo".to_string())
        );
        assert_eq!(
            select_git_working_path(Some("/repo"), Some("")),
            Some("/repo".to_string())
        );
    }
}
