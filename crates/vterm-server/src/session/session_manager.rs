use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, RwLock as StdRwLock};
use std::time::Instant;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, watch, Mutex, RwLock};
use tokio::time::{sleep, Duration};
use tracing::{debug, info, trace, warn};

use vterm_common::{
    EnvironmentProfileInfo, FileSearchResult, KnownAppInfo, PathNameMatch, RecentProjectDirectory,
    SessionApp, SessionAppStatus, SessionId, SessionInfo, TabGroup, TabGroupId, TabId, TabInfo,
    TabSplitPlacement, TerminalCommandHistory, TerminalLayout, TerminalLayoutDirection,
    TerminalProfile, TerminalSize, TAB_GROUP_ALL_ID, TAB_GROUP_UNGROUPED_ID,
};
use vterm_core::emulator::{Color, Terminal};
use vterm_core::folder_stream::{
    FolderDecodeError, FolderFrame, FolderFrameDecoder, FolderFrameWriter, FolderStreamReader,
    FolderWalkState,
};
use vterm_core::protocol::{Message, TempFileUsage, VoiceInputOptimizationContext};
use vterm_transport::{encode_message, EncodedMessage, OutboundBody, ReceivedBody};

use crate::config::{ConfigManager, EnvironmentNode, TerminalEnvVar};
use crate::fs_watcher::FsWatcherManager;
use crate::session::terminal_observer::AppDetectionMode;
use crate::session::util::{default_terminal_os, infer_terminal_os};
use crate::session::TerminalObserverManager;
use crate::voice::VoicePolishContext;

use super::environment_script::write_environment_script;
use super::file_ops::{
    create_path as fs_create_path, delete_paths as fs_delete_paths, paste_paths as fs_paste_paths,
    rename_path as fs_rename_path, CreatePathRequest, DeletePathsRequest, PastePathsRequest,
    RenamePathRequest,
};
use super::find_files::{find_files_stream, FindFilesOptions};
use super::git::git_read_worktree_file_diff_ranges;
use super::path_listing::{
    annotate_path_content, read_path_content, read_path_content_fast, resolve_filesystem_path,
    resolve_listing_target, ListingTarget,
};
use super::quick_command_script::write_quick_command_script;
use super::recent_commands::spawn_recent_command_scan;
use super::recent_projects::{spawn_recent_project_scan, RecentProjectScanState};
use super::screen_log::ScreenLogWriter;
use super::search_path_names::{search_path_names_stream, SearchPathNamesOptions};
use super::session_core::{ConnectionTx, ScreenListenerTx, Session, ShellKind};
use super::session_error::SessionError;
use super::session_io::{SerialTransport, SessionIo};
use super::shell_integration;
use super::terminal_profiles::{
    annotate_terminal_profile, available_terminal_profiles, discover_shell_profiles,
    local_server_name, resolve_terminal_launch, LaunchTarget,
};

const FIND_FILES_RESULT_CHUNK_FILES: usize = 50;
const SESSION_RESIZE_DEBOUNCE_MS: u64 = 10;
/// Beyond this many attached viewers per session, viewer registrations are
/// likely leaked from dead connections (see the connection idle timeout).
const MAX_EXPECTED_VIEWERS: u32 = 4;
const SESSION_OUTPUT_QUEUE_CAPACITY: usize = 256;
const SESSION_OUTPUT_BATCH_MAX_BYTES: usize = 256 * 1024;
const VOICE_POLISH_FILE_CONTEXT_LINES: u32 = 100;
const VOICE_POLISH_MAX_FILE_BYTES: u64 = 5 * 1024 * 1024;
const VOICE_POLISH_COMMAND_HISTORY_LINES: usize = 200;

pub(crate) struct PendingPathAnnotation {
    session_id: SessionId,
    path: String,
    target: ListingTarget,
    listing_generation: u64,
}

/// Project an `EnvironmentNode` into the wire-friendly `EnvironmentNodeInfo`
/// shape (Groups recurse, Leaves drop their variables). Used to populate
/// `SessionInfo.environments_tree` for the terminal client's tree picker.
fn environment_node_to_info(node: &EnvironmentNode) -> vterm_common::EnvironmentNodeInfo {
    match node {
        EnvironmentNode::Group { id, name, children } => vterm_common::EnvironmentNodeInfo::Group {
            id: id.clone(),
            name: name.clone(),
            children: children.iter().map(environment_node_to_info).collect(),
        },
        EnvironmentNode::Leaf { id, name, .. } => vterm_common::EnvironmentNodeInfo::Leaf {
            id: id.clone(),
            name: name.clone(),
        },
    }
}

fn base_terminal_env(
    shell_kind: Option<ShellKind>,
    program: &str,
    integration_enabled: bool,
    requested_env: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut env = if integration_enabled && shell_kind == Some(ShellKind::Bash) {
        shell_integration::bash_login_environment(program, requested_env)
    } else {
        HashMap::new()
    };
    env.extend(requested_env.clone());
    env
}

fn apply_configured_env_vars(env: &mut HashMap<String, String>, env_vars: &[TerminalEnvVar]) {
    for env_var in env_vars {
        // The config may hold in-progress rows with blank or padded names
        // (kept verbatim for the webui editor); never export those.
        let name = env_var.name.trim();
        if env_var.enabled && !name.is_empty() {
            env.insert(name.to_string(), env_var.value.clone());
        }
    }
}

fn apply_vterm_terminal_env(env: &mut HashMap<String, String>) {
    const VTERM_VERSION: &str = env!("CARGO_PKG_VERSION");

    env.insert("VTERM".to_string(), "1".to_string());
    env.insert("VTERM_APP".to_string(), "vibe-terminal".to_string());
    env.insert("VTERM_VERSION".to_string(), VTERM_VERSION.to_string());

    // Kimi Code currently gates OSC 9 / OSC 9;4 emission behind a short
    // TERM_PROGRAM allow-list. Advertise a compatible terminal program.
    env.insert("TERM_PROGRAM".to_string(), "WezTerm".to_string());
    env.insert(
        "TERM_PROGRAM_VERSION".to_string(),
        format!("vibe-terminal-{VTERM_VERSION}"),
    );
    // Qwen Code also gates OSC 9;4 on ConEmuPID/WT_SESSION. Preserve an
    // explicitly configured value while enabling its progress reporting.
    env.entry("ConEmuPID".to_string())
        .or_insert_with(|| "1".to_string());

    // The client emulator parses full xterm SGR (16/256/true-color fg+bg), so
    // the PTY should advertise those capabilities: programs gate colored output
    // on TERM/COLORTERM, and a server spawned by a GUI app (Finder, ZCode,
    // launchd, systemd) inherits no TERM or a "dumb" placeholder, silently
    // disabling all colors in the session. Only fill in when neither the
    // client-provided env nor the server's own process environment carries a
    // usable value, so an existing TERM's terminfo behavior stays unchanged.
    apply_terminal_capability_env(
        env,
        std::env::var("TERM").ok().as_deref(),
        std::env::var("COLORTERM").ok().as_deref(),
    );
}

fn apply_terminal_capability_env(
    env: &mut HashMap<String, String>,
    process_term: Option<&str>,
    process_colorterm: Option<&str>,
) {
    let usable_term =
        |term: Option<&str>| term.is_some_and(|term| !term.trim().is_empty() && term != "dumb");
    if !usable_term(env.get("TERM").map(String::as_str)) && !usable_term(process_term) {
        env.insert("TERM".to_string(), "xterm-256color".to_string());
    }
    let has_colorterm = |value: Option<&str>| value.is_some_and(|v| !v.trim().is_empty());
    if !has_colorterm(env.get("COLORTERM").map(String::as_str)) && !has_colorterm(process_colorterm)
    {
        env.insert("COLORTERM".to_string(), "truecolor".to_string());
    }
}

async fn read_text_file_window(
    path: &Path,
    start_line: u32,
    end_line: u32,
) -> Result<(u32, u32, String), String> {
    let metadata = tokio::fs::metadata(path)
        .await
        .map_err(|error| format!("metadata failed: {error}"))?;
    if !metadata.is_file() {
        return Err("not a file".to_string());
    }
    if metadata.len() > VOICE_POLISH_MAX_FILE_BYTES {
        return Err(format!(
            "file too large: {} > {} bytes",
            metadata.len(),
            VOICE_POLISH_MAX_FILE_BYTES
        ));
    }

    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|error| format!("read failed: {error}"))?;
    Ok(extract_line_window(
        &content,
        start_line,
        end_line,
        VOICE_POLISH_FILE_CONTEXT_LINES,
    ))
}

fn read_git_file_at_revision(
    working_path: &str,
    file_path: &str,
    git_hash: &str,
) -> Result<(String, String), String> {
    if git_hash.trim().is_empty() {
        return Err("empty git revision".to_string());
    }
    if file_path.trim().is_empty() {
        return Err("empty file path".to_string());
    }

    super::git::git_check_repo(working_path).map_err(|error| error.to_string())?;
    let repo_root = super::git::git_repo_root(working_path).map_err(|error| error.to_string())?;
    let repo_path = normalize_git_context_path(&repo_root, file_path);
    let spec = format!("{}:{}", git_hash.trim(), repo_path);

    let mut command = std::process::Command::new("git");
    command
        .args(["show", "--textconv", &spec])
        .current_dir(&repo_root);
    vterm_common::command_no_window(&mut command);
    let output = command
        .output()
        .map_err(|error| format!("git show failed: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "git show failed".to_string()
        } else {
            stderr
        });
    }

    Ok((
        repo_path,
        String::from_utf8_lossy(&output.stdout).into_owned(),
    ))
}

fn normalize_git_context_path(repo_root: &str, file_path: &str) -> String {
    let path = Path::new(file_path);
    if path.is_absolute() {
        if let Ok(relative) = path.strip_prefix(repo_root) {
            return path_to_slash_string(relative);
        }
    }
    file_path.trim_start_matches("./").replace('\\', "/")
}

fn path_to_slash_string(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            std::path::Component::Normal(part) => Some(part.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn extract_line_window(
    content: &str,
    start_line: u32,
    end_line: u32,
    padding: u32,
) -> (u32, u32, String) {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return (0, 0, String::new());
    }

    let total = lines.len() as u32;
    let requested_start = start_line.max(1).min(total);
    let requested_end = end_line.max(requested_start).min(total);
    let from = requested_start.saturating_sub(padding).max(1);
    let to = requested_end.saturating_add(padding).min(total);

    let mut output = String::new();
    for line_no in from..=to {
        let idx = (line_no - 1) as usize;
        output.push_str(&format!("{line_no}: {}\n", lines[idx]));
    }
    (from, to, output)
}

fn debug_raw_history_path(session_id: SessionId, role: &str, size: TerminalSize) -> PathBuf {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".vibeterminal").join(format!(
        "debug_{}_{}_buffer_{}x{}.raw",
        session_id, role, size.rows, size.cols
    ))
}

fn send_find_files_result_chunk(
    tx: &ConnectionTx,
    session_id: SessionId,
    request_id: &str,
    query: &str,
    chunk_index: &mut u32,
    results: Vec<FileSearchResult>,
    is_final: bool,
    error: Option<String>,
) {
    info!(
        "FindFiles sending chunk session={} request_id={} query_bytes={} chunk={} final={} files={} matches={} error={}",
        session_id,
        request_id,
        query.len(),
        *chunk_index,
        is_final,
        results.len(),
        results
            .iter()
            .map(|result| result.matches.len())
            .sum::<usize>(),
        error.is_some()
    );
    let _ = tx.send(Message::FindFilesResult {
        session_id,
        request_id: request_id.to_string(),
        query: query.to_string(),
        chunk_index: *chunk_index,
        is_final,
        results,
        error,
    });
    *chunk_index = (*chunk_index).saturating_add(1);
}

/// Send a single `SearchPathNamesResult` message. The name search streams at
/// most one results chunk before the final marker, so (unlike the find-files
/// helper) there is no chunk counter to maintain.
fn send_search_path_names_result(
    tx: &ConnectionTx,
    session_id: SessionId,
    request_id: &str,
    root_path: &str,
    query: &str,
    results: Vec<PathNameMatch>,
    is_final: bool,
    error: Option<String>,
) {
    info!(
        "SearchPathNames sending result session={} request_id={} query_bytes={} final={} matches={} error={}",
        session_id,
        request_id,
        query.len(),
        is_final,
        results.len(),
        error.is_some()
    );
    let _ = tx.send(Message::SearchPathNamesResult {
        session_id,
        request_id: request_id.to_string(),
        root_path: root_path.to_string(),
        query: query.to_string(),
        is_final,
        results,
        error,
    });
}

/// Log and apply any resize that was deferred for this session before PTY I/O continues.
fn flush_pending_resize_for_session(
    session_id: SessionId,
    session: &mut Session,
) -> Result<(), SessionError> {
    if let Some((old_size, new_size)) = session.flush_pending_resize()? {
        //         tracing::info!(
        //             "[Server] resize_session: session={} pty {}x{} -> {}x{}",
        //             session_id,
        //             old_size.rows,
        //             old_size.cols,
        //             new_size.rows,
        //             new_size.cols
        //         );
    }
    Ok(())
}

/// Choose app detection strategy based on transport and shell capabilities.
fn app_detection_mode_for_session(session: &Session) -> AppDetectionMode {
    if matches!(session.io, SessionIo::Serial(_)) || session.shell_kind == Some(ShellKind::Cmd) {
        AppDetectionMode::LegacyPrompt
    } else {
        AppDetectionMode::ShellIntegration
    }
}

/// Incremental writer for one `UploadStart` body (see
/// [`SessionManager::upload_sink`]): decodes the framed entry stream
/// chunk-by-chunk and writes the uploaded items under the resolved
/// destination as the bytes arrive. Once `failed` is set the sink only
/// drains the stream.
pub struct FolderUploadSink {
    task_id: String,
    session_id: SessionId,
    fs_dir: PathBuf,
    decoder: FolderFrameDecoder,
    current: Option<(tokio::fs::File, u64)>,
    files_done: u64,
    bytes_done: u64,
    failed: Option<String>,
}

impl FolderUploadSink {
    /// A sink that only drains the stream and reports `error` at `finish`
    /// (used when the request cannot start, e.g. session access denied).
    pub fn failed(task_id: String, session_id: SessionId, error: String) -> Self {
        Self {
            task_id,
            session_id,
            fs_dir: PathBuf::new(),
            decoder: FolderFrameDecoder::new(),
            current: None,
            files_done: 0,
            bytes_done: 0,
            failed: Some(error),
        }
    }

    /// Feed one transport body chunk. Decode/write errors fail the task but
    /// keep draining: body frames continue until the stream ends.
    pub async fn push(&mut self, chunk: &[u8]) {
        if self.failed.is_some() {
            return;
        }
        if let Err(error) = self.consume(chunk).await {
            warn!(task_id = %self.task_id, %error, "upload: stream failed mid-transfer");
            self.failed = Some(error);
        }
    }

    async fn consume(&mut self, chunk: &[u8]) -> Result<(), String> {
        let frames = self.decoder.push(chunk).map_err(|error| match error {
            FolderDecodeError::Aborted(reason) => format!("cancelled: {reason}"),
            FolderDecodeError::Malformed(detail) => format!("bad upload stream: {detail}"),
        })?;
        for frame in frames {
            match frame {
                FolderFrame::FileBegin { rel_path, size } => {
                    let rel_norm = rel_path.replace('/', std::path::MAIN_SEPARATOR_STR);
                    let target = vterm_core::folder_stream::unique_destination_path(
                        self.fs_dir.join(&rel_norm),
                    );
                    debug!(
                        %rel_path,
                        size,
                        target = %target.display(),
                        "upload: file begin"
                    );
                    if let Some(parent) = target.parent() {
                        tokio::fs::create_dir_all(parent)
                            .await
                            .map_err(|e| format!("mkdir failed: {e}"))?;
                    }
                    let file = tokio::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&target)
                        .await
                        .map_err(|e| format!("create failed: {e}"))?;
                    self.current = Some((file, size));
                }
                FolderFrame::Chunk(data) => {
                    if let Some((file, remaining)) = self.current.as_mut() {
                        file.write_all(&data)
                            .await
                            .map_err(|e| format!("write failed: {e}"))?;
                        *remaining = remaining.saturating_sub(data.len() as u64);
                        self.bytes_done += data.len() as u64;
                    }
                }
                FolderFrame::FileEnd { .. } => {
                    self.files_done += 1;
                    self.current = None;
                }
                FolderFrame::End => {}
                FolderFrame::Abort { reason } => {
                    return Err(format!("cancelled: {reason}"));
                }
                FolderFrame::Skip { rel_path, reason } => {
                    warn!(%rel_path, %reason, "upload: file skipped");
                }
            }
        }
        Ok(())
    }

    /// Produce the `UploadResult` once the stream ended.
    pub fn finish(self) -> Message {
        info!(
            task_id = %self.task_id,
            files = self.files_done,
            bytes = self.bytes_done,
            error = self.failed.as_deref().unwrap_or(""),
            "upload: streaming finished"
        );
        Message::UploadResult {
            task_id: self.task_id,
            session_id: self.session_id,
            files: self.files_done,
            bytes: self.bytes_done,
            error: self.failed,
        }
    }
}

#[derive(Clone)]
pub struct SessionManager {
    pub(super) sessions: Arc<RwLock<HashMap<SessionId, Arc<Mutex<Session>>>>>,
    tab_layouts: Arc<RwLock<HashMap<TabId, TerminalLayout>>>,
    next_group_id: Arc<AtomicU32>,
    tab_groups: Arc<RwLock<HashMap<TabGroupId, String>>>,
    next_tab_group_id: Arc<AtomicU32>,
    session_sequence: Arc<AtomicU64>,
    state_change_notifier: Arc<RwLock<Option<mpsc::UnboundedSender<()>>>>,
    terminal_observer: Arc<RwLock<Option<Arc<TerminalObserverManager>>>>,
    fs_watcher_manager: Arc<RwLock<Option<Arc<FsWatcherManager>>>>,
    /// Serializes rebase git mutations (apply/resolve/continue/skip/
    /// abort) so an optimistic `git add` and a `git rebase --continue`
    /// can never collide on .git/index.lock.
    git_rebase_mutex: Arc<Mutex<()>>,
    pub(super) config_manager: Arc<ConfigManager>,
    local_terminal_profiles: Arc<Vec<TerminalProfile>>,
    pub(super) recent_project_directories: Arc<StdRwLock<Vec<RecentProjectDirectory>>>,
    pub(super) recent_project_scan_state: watch::Sender<RecentProjectScanState>,
    recent_terminal_commands: Arc<StdRwLock<Vec<TerminalCommandHistory>>>,
    pub(super) active_ai_conversations: Arc<Mutex<HashMap<SessionId, String>>>,
    pub(super) ai_conversation_message_counts: Arc<Mutex<HashMap<SessionId, usize>>>,
    file_search_cancellations: Arc<Mutex<HashMap<SessionId, Arc<AtomicBool>>>>,
    analysis_cancellations: Arc<Mutex<HashMap<SessionId, Arc<AtomicBool>>>>,
    rebase_ai_cancellations: Arc<Mutex<HashMap<SessionId, Arc<AtomicBool>>>>,
    screen_log_writer: ScreenLogWriter,
}

impl SessionManager {
    async fn terminate_session_transport(session_id: SessionId, session: Arc<Mutex<Session>>) {
        let result = tokio::task::spawn_blocking(move || {
            let mut session = session.blocking_lock();
            session.io.terminate()
        })
        .await;

        match result {
            Ok(Ok(())) => {}
            Ok(Err(error)) => warn!(
                "Failed to terminate session transport for session {}: {}",
                session_id, error
            ),
            Err(error) => warn!(
                "Session transport termination task failed for session {}: {}",
                session_id, error
            ),
        }
    }

    /// Start or replace the debounce window for PTY resize so only the newest
    /// requested size survives a window-resize burst.
    fn spawn_deferred_resize_task(
        session_id: SessionId,
        session: Arc<Mutex<Session>>,
        generation: u64,
    ) {
        tokio::spawn(async move {
            sleep(Duration::from_millis(SESSION_RESIZE_DEBOUNCE_MS)).await;
            let mut session = session.lock().await;
            if session.pending_resize_generation() != generation {
                return;
            }
            if let Err(error) = flush_pending_resize_for_session(session_id, &mut session) {
                warn!(
                    "Failed to apply deferred PTY resize for session {}: {}",
                    session_id, error
                );
            }
        });
    }

    pub fn new(config_manager: Arc<ConfigManager>) -> Self {
        let (recent_project_scan_state, _recent_project_scan_rx) =
            watch::channel(RecentProjectScanState::Loading);
        let local_terminal_profiles = discover_shell_profiles();
        info!(
            profile_count = local_terminal_profiles.len(),
            "Discovered local terminal profiles during server startup"
        );
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            tab_layouts: Arc::new(RwLock::new(HashMap::new())),
            next_group_id: Arc::new(AtomicU32::new(1)),
            tab_groups: Arc::new(RwLock::new(HashMap::new())),
            next_tab_group_id: Arc::new(AtomicU32::new(1)),
            session_sequence: Arc::new(AtomicU64::new(1)),
            state_change_notifier: Arc::new(RwLock::new(None)),
            terminal_observer: Arc::new(RwLock::new(None)),
            fs_watcher_manager: Arc::new(RwLock::new(None)),
            git_rebase_mutex: Arc::new(Mutex::new(())),
            config_manager: config_manager.clone(),
            local_terminal_profiles: Arc::new(local_terminal_profiles),
            recent_project_directories: Arc::new(StdRwLock::new(Vec::new())),
            recent_project_scan_state,
            recent_terminal_commands: Arc::new(StdRwLock::new(Vec::new())),
            active_ai_conversations: Arc::new(Mutex::new(HashMap::new())),
            ai_conversation_message_counts: Arc::new(Mutex::new(HashMap::new())),
            file_search_cancellations: Arc::new(Mutex::new(HashMap::new())),
            analysis_cancellations: Arc::new(Mutex::new(HashMap::new())),
            rebase_ai_cancellations: Arc::new(Mutex::new(HashMap::new())),
            screen_log_writer: ScreenLogWriter::new(),
        }
    }

    pub fn start_recent_project_scan(&self) {
        spawn_recent_project_scan(
            Arc::clone(&self.recent_project_directories),
            self.recent_project_scan_state.clone(),
        );
    }

    pub fn subscribe_recent_project_scan(&self) -> watch::Receiver<RecentProjectScanState> {
        self.recent_project_scan_state.subscribe()
    }

    pub fn start_recent_command_scan(&self) {
        spawn_recent_command_scan(Arc::clone(&self.recent_terminal_commands));
    }

    pub fn recent_project_directories(&self) -> Vec<RecentProjectDirectory> {
        self.recent_project_directories
            .read()
            .map(|directories| directories.clone())
            .unwrap_or_default()
    }

    pub fn remove_recent_project_directory(&self, path: &str) -> Vec<RecentProjectDirectory> {
        self.recent_project_directories
            .write()
            .map(|mut directories| {
                directories.retain(|directory| directory.path != path);
                directories.clone()
            })
            .unwrap_or_default()
    }

    pub fn recent_terminal_commands(&self) -> Vec<TerminalCommandHistory> {
        self.recent_terminal_commands
            .read()
            .map(|commands| commands.clone())
            .unwrap_or_default()
    }

    pub async fn known_apps(&self) -> Vec<KnownAppInfo> {
        match self.terminal_observer().await {
            Some(observer) => observer.known_apps().await,
            None => Vec::new(),
        }
    }

    pub async fn set_state_change_notifier(&self, notifier: mpsc::UnboundedSender<()>) {
        *self.state_change_notifier.write().await = Some(notifier);
    }

    pub async fn set_terminal_observer(&self, observer: Arc<TerminalObserverManager>) {
        let existing_sessions: Vec<_> = {
            let sessions = self.sessions.read().await;
            sessions.values().cloned().collect()
        };

        for session_arc in existing_sessions {
            let (
                session_id,
                title,
                session_name,
                detection_mode,
                terminal_os,
                notification,
                auto_confirm,
            ) = {
                let session = session_arc.lock().await;
                (
                    session.id,
                    session.title.clone(),
                    session.session_name.clone(),
                    app_detection_mode_for_session(&session),
                    session.terminal_os,
                    session.notification_enabled,
                    session.auto_confirm_enabled,
                )
            };
            let handle = observer.spawn_observer_actor(
                &session_arc,
                session_id,
                title,
                session_name,
                detection_mode,
                terminal_os,
                notification,
                auto_confirm,
            );
            session_arc.lock().await.observer = Some(handle);
        }

        *self.terminal_observer.write().await = Some(observer);
    }

    pub async fn set_fs_watcher_manager(&self, watcher: Arc<FsWatcherManager>) {
        *self.fs_watcher_manager.write().await = Some(watcher);
    }

    pub async fn fs_watcher_manager(&self) -> Option<Arc<FsWatcherManager>> {
        self.fs_watcher_manager.read().await.clone()
    }

    async fn terminal_observer(&self) -> Option<Arc<TerminalObserverManager>> {
        self.terminal_observer.read().await.clone()
    }

    async fn notify_state_changed(&self) {
        let notifier = self.state_change_notifier.read().await.clone();
        if let Some(notifier) = notifier {
            let _ = notifier.send(());
        }
    }

    fn allocate_group_id(&self) -> u32 {
        loop {
            let group_id = self.next_group_id.fetch_add(1, Ordering::Relaxed);
            if group_id != 0 {
                return group_id;
            }
        }
    }

    fn allocate_tab_group_id(&self) -> TabGroupId {
        loop {
            let group_id = self.next_tab_group_id.fetch_add(1, Ordering::Relaxed);
            if group_id != TAB_GROUP_UNGROUPED_ID && group_id != TAB_GROUP_ALL_ID {
                return group_id;
            }
        }
    }

    async fn resolve_group_id_for_new_session(
        &self,
        sibling_session_id: Option<SessionId>,
    ) -> Result<u32, SessionError> {
        let Some(sibling_session_id) = sibling_session_id else {
            return Ok(0);
        };

        let sibling = self
            .get_session(sibling_session_id)
            .await
            .ok_or(SessionError::SessionNotFound(sibling_session_id))?;
        let mut sibling = sibling.lock().await;
        if sibling.group_id == 0 {
            sibling.group_id = self.allocate_group_id();
        }
        Ok(sibling.group_id)
    }

    async fn resolve_tab_group_id_for_new_session(
        &self,
        tab_group_id: Option<TabGroupId>,
    ) -> TabGroupId {
        let Some(group_id) = tab_group_id else {
            return TAB_GROUP_UNGROUPED_ID;
        };
        if group_id == TAB_GROUP_UNGROUPED_ID {
            return TAB_GROUP_UNGROUPED_ID;
        }
        if group_id == TAB_GROUP_ALL_ID {
            return TAB_GROUP_UNGROUPED_ID;
        }
        if self.tab_groups.read().await.contains_key(&group_id) {
            group_id
        } else {
            TAB_GROUP_UNGROUPED_ID
        }
    }

    async fn maybe_reset_last_sibling_group(&self, closed_group_id: u32) {
        if closed_group_id == 0 {
            return;
        }

        let session_arcs: Vec<_> = {
            let sessions = self.sessions.read().await;
            sessions.values().cloned().collect()
        };
        let mut grouped = Vec::new();
        for session in session_arcs {
            let group_id = session.lock().await.group_id;
            if group_id == closed_group_id {
                grouped.push(session);
            }
        }

        if grouped.len() == 1 {
            grouped[0].lock().await.group_id = 0;
        }
    }

    async fn rehome_tab_after_session_removed(&self, removed_tab_id: TabId) {
        let sessions = self.sessions.read().await;
        let session_arcs = sessions.values().cloned().collect::<Vec<_>>();
        drop(sessions);

        let mut tab_sessions = Vec::new();
        for session in session_arcs {
            if session.lock().await.tab_id == removed_tab_id {
                tab_sessions.push(session);
            }
        }

        let Some(first_session) = tab_sessions.first() else {
            return;
        };
        let new_tab_id = first_session.lock().await.id;

        for session in tab_sessions {
            session.lock().await.tab_id = new_tab_id;
        }
    }

    pub async fn create_session(
        &self,
        size: TerminalSize,
        shell: String,
        profile_id: Option<String>,
        env: HashMap<String, String>,
        cwd: Option<String>,
        sibling_session_id: Option<SessionId>,
        duplicate_session_id: Option<SessionId>,
        tab_group_id: Option<TabGroupId>,
        session_name: Option<String>,
        app: Option<String>,
    ) -> Result<SessionId, SessionError> {
        self.create_session_with_theme_colors(
            size,
            shell,
            profile_id,
            env,
            cwd,
            sibling_session_id,
            duplicate_session_id,
            tab_group_id,
            session_name,
            app,
            None,
            None,
        )
        .await
    }

    pub async fn create_session_with_theme_colors(
        &self,
        size: TerminalSize,
        shell: String,
        profile_id: Option<String>,
        env: HashMap<String, String>,
        cwd: Option<String>,
        sibling_session_id: Option<SessionId>,
        duplicate_session_id: Option<SessionId>,
        tab_group_id: Option<TabGroupId>,
        session_name: Option<String>,
        app: Option<String>,
        background: Option<[u8; 3]>,
        foreground: Option<[u8; 3]>,
    ) -> Result<SessionId, SessionError> {
        // Normalize the requested display name: trim, drop if empty.
        let session_name = session_name.and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
        let app = app.and_then(|value| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        });

        // Inherit cwd (and profile_id, unless the caller specified one) from
        // the source session for split/duplicate. An explicit `profile_id`
        // wins so callers (e.g. mobile split) can override the inherited
        // profile; `None` falls back to the source session's profile.
        let source_id = sibling_session_id.or(duplicate_session_id);
        let (resolved_profile_id, resolved_cwd) = if let Some(source_id) = source_id {
            if let Some(source) = self.get_session(source_id).await {
                let s = source.lock().await;
                (
                    profile_id
                        .clone()
                        .or_else(|| Some(s.profile_id.clone().unwrap_or_default())),
                    cwd.or_else(|| s.current_working_path.clone()),
                )
            } else {
                (profile_id, cwd)
            }
        } else {
            (profile_id, cwd)
        };
        let config = self.config_manager.snapshot().await;
        let launch = resolve_terminal_launch(
            &config,
            self.local_terminal_profiles.as_slice(),
            resolved_profile_id.as_deref(),
            &shell,
        )?;
        if app.is_some() && matches!(&launch, LaunchTarget::Uart(_)) {
            return Err(SessionError::PtyError(
                "an initial app command can only be used with a shell terminal".to_string(),
            ));
        }
        let env_count = env.len();
        let requested_profile = resolved_profile_id.as_deref().unwrap_or("<none>");
        let requested_shell = if shell.trim().is_empty() {
            "<default>"
        } else {
            shell.as_str()
        };
        let session_id = SessionId::new();
        let launch_result = match launch {
            LaunchTarget::Shell(launch) => {
                let terminal_os = Some(infer_terminal_os(&launch.program, &launch.args));
                let shell_kind = infer_shell_kind(&launch.program, &launch.title, terminal_os);

                // Prepare shell integration (OSC 7 CWD reporting)
                let integration = shell_integration::prepare_integration(
                    session_id,
                    shell_kind,
                    &launch.program,
                    &launch.args,
                    resolved_cwd.as_deref(),
                    &env,
                )?;
                let mut merged_env =
                    base_terminal_env(shell_kind, &launch.program, integration.is_some(), &env);
                apply_configured_env_vars(&mut merged_env, &config.terminal.env_vars);
                // A lingering (boot-time) server predates the graphical
                // login, so its own environment — inherited by the PTY
                // child — has no display/session-bus variables. Fill them
                // in from the user manager environment so GUI applications
                // can start from terminal sessions.
                #[cfg(target_os = "linux")]
                crate::desktop_env::apply_graphical_env(&mut merged_env);
                let mut final_args = launch.args.clone();
                let mut cleanup_paths = Vec::new();
                if let Some(ref integ) = integration {
                    for (key, value) in integ.env_additions() {
                        merged_env.insert(key.clone(), value.clone());
                    }
                    if let Some(args) = integ.args_override() {
                        final_args = args.to_vec();
                    }
                    cleanup_paths = integ.temp_paths().to_vec();
                }
                apply_vterm_terminal_env(&mut merged_env);

                info!(
                    "Creating terminal session: requested_profile={}, requested_shell={}, resolved_title={}, program={}, args={:?}, terminal_os={:?}, cwd={:?}, env_count={}",
                    requested_profile,
                    requested_shell,
                    launch.title,
                    launch.program,
                    final_args,
                    terminal_os,
                    resolved_cwd,
                    merged_env.len()
                );
                let pty = vterm_core::pty::PortablePty::new_command(
                    size,
                    &launch.program,
                    &final_args,
                    &merged_env,
                    resolved_cwd.as_deref(),
                )
                .map_err(|error| SessionError::PtyError(error.to_string()))?;
                (
                    launch.title,
                    None,
                    None,
                    None,
                    terminal_os,
                    shell_kind,
                    SessionIo::new_pty(pty, session_id),
                    cleanup_paths,
                    launch.program,
                    launch.args,
                )
            }
            LaunchTarget::Uart(launch) => {
                let uart_session_name =
                    format!("{}({})", launch.port_config.name, launch.port_config.port);
                info!(
                    "Creating UART session: requested_profile={}, requested_shell={}, resolved_title={}, port={}, baud_rate={}",
                    requested_profile,
                    requested_shell,
                    launch.title,
                    launch.port_config.port,
                    launch.port_config.baud_rate
                );
                (
                    launch.title,
                    Some(launch.port_config.terminal_type.clone()),
                    launch.port_config.encoding.clone(),
                    Some(uart_session_name),
                    None,
                    None,
                    SessionIo::Serial(SerialTransport::open(&launch.port_config)?),
                    Vec::new(),
                    String::new(),
                    Vec::new(),
                )
            }
        };
        let (
            title,
            terminal_type,
            encoding,
            uart_session_name,
            terminal_os,
            shell_kind,
            io,
            cleanup_paths,
            launch_program,
            launch_args,
        ) = launch_result;
        let group_id = self
            .resolve_group_id_for_new_session(sibling_session_id)
            .await?;
        let tab_id = if let Some(sibling_session_id) = sibling_session_id {
            self.get_session(sibling_session_id)
                .await
                .ok_or(SessionError::SessionNotFound(sibling_session_id))?
                .lock()
                .await
                .tab_id
        } else {
            session_id
        };
        let tab_group_id = self
            .resolve_tab_group_id_for_new_session(tab_group_id)
            .await;
        let observer_title = title.clone();
        let sequence = self.session_sequence.fetch_add(1, Ordering::Relaxed);
        let mut session = Session::new(
            session_id,
            tab_id,
            group_id,
            size,
            title,
            terminal_type,
            encoding,
            terminal_os,
            shell_kind,
            resolved_cwd.clone(),
            io,
            resolved_profile_id,
            sequence,
        );
        session.tab_group_id = tab_group_id;
        session.temp_cleanup_paths = cleanup_paths;
        session.launch_program = launch_program;
        session.launch_args = launch_args;
        session.auto_confirm_enabled = config.terminal.auto_confirm;
        session.terminal.set_theme_default_colors(
            foreground.map(|rgb| Color::Rgb(rgb[0], rgb[1], rgb[2])),
            background.map(|rgb| Color::Rgb(rgb[0], rgb[1], rgb[2])),
        );
        let session_name = uart_session_name.or(session_name);
        session.session_name = session_name.clone();
        let session = Arc::new(Mutex::new(session));

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, Arc::clone(&session));
        }
        self.register_session_in_tab_layout(tab_id, session_id, sibling_session_id)
            .await;

        if let Some(manager) = self.terminal_observer().await {
            let (detection_mode, terminal_os, notification, auto_confirm) = {
                let s = session.lock().await;
                (
                    app_detection_mode_for_session(&s),
                    s.terminal_os,
                    s.notification_enabled,
                    s.auto_confirm_enabled,
                )
            };
            let handle = manager.spawn_observer_actor(
                &session,
                session_id,
                observer_title,
                session_name,
                detection_mode,
                terminal_os,
                notification,
                auto_confirm,
            );
            session.lock().await.observer = Some(handle);
        }

        self.spawn_session_reader(session_id, session).await?;
        if let Some(app) = app {
            // The PTY was created with `resolved_cwd`, so this command runs in
            // the requested directory without shell-specific quoting rules.
            let mut command = app.into_bytes();
            command.push(b'\r');
            let session = self
                .get_session(session_id)
                .await
                .ok_or(SessionError::SessionNotFound(session_id))?;
            let writer = session.lock().await.io.writer();
            writer.write(&command).await?;
        }
        self.broadcast_session_list().await?;
        info!(
            "Session created: {} with size {}x{}",
            session_id, size.rows, size.cols
        );
        Ok(session_id)
    }

    pub async fn split_terminal(
        &self,
        target_session_id: SessionId,
        placement: TabSplitPlacement,
        size: TerminalSize,
        profile_id: Option<String>,
    ) -> Result<SessionId, SessionError> {
        let session_id = self
            .create_session(
                size,
                String::new(),
                profile_id,
                HashMap::new(),
                None,
                Some(target_session_id),
                None,
                None,
                None,
                None,
            )
            .await?;
        self.replace_terminal_with_split(target_session_id, session_id, placement)
            .await?;
        self.broadcast_session_list().await?;
        Ok(session_id)
    }

    async fn register_session_in_tab_layout(
        &self,
        tab_id: TabId,
        session_id: SessionId,
        sibling_session_id: Option<SessionId>,
    ) {
        let mut layouts = self.tab_layouts.write().await;
        let new_terminal = TerminalLayout::Terminal { session_id };
        let layout = layouts
            .entry(tab_id)
            .or_insert_with(|| new_terminal.clone());
        if layout_contains_session(layout, session_id) {
            return;
        }
        if sibling_session_id.is_some() {
            *layout = TerminalLayout::Split {
                direction: TerminalLayoutDirection::Horizontal,
                first_percent: 50.0,
                first: Box::new(layout.clone()),
                second: Box::new(new_terminal),
            };
        } else {
            *layout = new_terminal;
        }
    }

    async fn replace_terminal_with_split(
        &self,
        target_session_id: SessionId,
        new_session_id: SessionId,
        placement: TabSplitPlacement,
    ) -> Result<(), SessionError> {
        let target = self
            .get_session(target_session_id)
            .await
            .ok_or(SessionError::SessionNotFound(target_session_id))?;
        let tab_id = target.lock().await.tab_id;
        let mut layouts = self.tab_layouts.write().await;
        let layout = layouts
            .entry(tab_id)
            .or_insert_with(|| TerminalLayout::Terminal {
                session_id: target_session_id,
            });
        if layout_contains_session(layout, new_session_id) {
            let mut session_ids = Vec::new();
            collect_layout_sessions(layout, &mut session_ids);
            session_ids.retain(|session_id| *session_id != new_session_id);
            if let Some(pruned) = prune_layout_to_sessions(layout.clone(), &session_ids) {
                *layout = pruned;
            }
        }
        if replace_terminal_leaf(layout, target_session_id, new_session_id, placement) {
            Ok(())
        } else {
            Err(SessionError::SessionNotFound(target_session_id))
        }
    }

    pub async fn update_tab_layout(
        &self,
        tab_id: TabId,
        layout: TerminalLayout,
    ) -> Result<(), SessionError> {
        let layout_session_ids = {
            let mut ids = Vec::new();
            collect_layout_sessions(&layout, &mut ids);
            ids
        };
        let session_arcs = {
            let sessions = self.sessions.read().await;
            sessions.values().cloned().collect::<Vec<_>>()
        };
        let mut tab_session_ids = Vec::new();
        for session in session_arcs {
            let session = session.lock().await;
            if session.tab_id == tab_id {
                tab_session_ids.push(session.id);
            }
        }
        let mut sorted_layout_ids = layout_session_ids.clone();
        sorted_layout_ids.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        sorted_layout_ids.dedup();
        let mut sorted_tab_ids = tab_session_ids;
        sorted_tab_ids.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        sorted_tab_ids.dedup();

        if sorted_layout_ids != sorted_tab_ids {
            return Err(SessionError::InvalidState(
                "tab layout sessions do not match tab sessions".to_string(),
            ));
        }

        self.tab_layouts.write().await.insert(tab_id, layout);
        self.broadcast_session_list().await?;
        Ok(())
    }

    async fn spawn_session_reader(
        &self,
        session_id: SessionId,
        session: Arc<Mutex<Session>>,
    ) -> Result<(), SessionError> {
        let (output_tx, mut output_rx) = mpsc::channel::<Vec<u8>>(SESSION_OUTPUT_QUEUE_CAPACITY);
        let output_manager = self.clone();
        let reader_session_id = session_id;
        tokio::spawn(async move {
            while let Some(mut data) = output_rx.recv().await {
                let mut chunks = 1usize;
                while data.len() < SESSION_OUTPUT_BATCH_MAX_BYTES {
                    let Ok(next) = output_rx.try_recv() else {
                        break;
                    };
                    data.extend_from_slice(&next);
                    chunks += 1;
                }
                // Echo-path probe: only batches following a recent keystroke
                // are sampled; background PTY streams never log here.
                if let Some(echo_ms) = vterm_common::latency::echo_ms_since_input(reader_session_id)
                {
                    if vterm_common::latency::sample(
                        "server:pty-output-dequeued",
                        reader_session_id,
                        echo_ms as f64,
                        50.0,
                        3.0,
                    ) {
                        vterm_common::latency_log!(
                            stage = "server:pty-output-dequeued",
                            session = %reader_session_id,
                            echo_ms,
                            bytes = data.len(),
                            chunks,
                            "[latency] keystroke echo dequeued late from PTY queue"
                        );
                    }
                }
                if let Err(error) = output_manager
                    .process_session_output(session_id, data)
                    .await
                {
                    if matches!(error, SessionError::SessionNotFound(_)) {
                        break;
                    }
                    warn!(
                        "Failed to process output for session {}; continuing output drain: {}",
                        session_id, error
                    );
                }
            }
        });

        let session = session.lock().await;
        session
            .io
            .spawn_reader(session_id, output_tx, self.clone(), session.io_generation)
            .await
    }

    pub async fn get_session(&self, session_id: SessionId) -> Option<Arc<Mutex<Session>>> {
        let sessions = self.sessions.read().await;
        sessions.get(&session_id).cloned()
    }

    pub async fn add_viewer(
        &self,
        session_id: SessionId,
        viewer_tx: ConnectionTx,
        initial_size: Option<TerminalSize>,
    ) -> Result<Vec<Message>, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let (messages, resize_generation) = {
            let mut session = session.lock().await;
            let count = session.add_viewer(viewer_tx.clone());
            info!("Added viewer to session {} ({} viewers)", session_id, count);
            if count > MAX_EXPECTED_VIEWERS {
                // Legitimate setups rarely exceed a handful of viewers per
                // session (desktop + web UI + a second window). More usually
                // means dead connections leaked their viewer registration.
                warn!(
                    "Session {} has {} viewers — possible leaked viewer registrations",
                    session_id, count
                );
            }
            // A viewer that reports its grid at attach time is the most
            // recently active one; snap the PTY to it immediately instead of
            // waiting for the follow-up Resize (avoids a visible flap and a
            // spurious SIGWINCH/redraw cycle).
            let generation = initial_size.and_then(|size| {
                session.update_viewer_size(&viewer_tx, size);
                session.defer_resize(size)
            });
            (session.attach_messages(), generation)
        };
        if let Some(generation) = resize_generation {
            Self::spawn_deferred_resize_task(session_id, session.clone(), generation);
        }

        self.broadcast_session_list().await?;
        Ok(messages)
    }

    /// Register an in-process screen listener for a session.
    pub async fn add_screen_listener(
        &self,
        session_id: SessionId,
        listener: ScreenListenerTx,
    ) -> Result<(), SessionError> {
        let sessions = self.sessions.read().await;
        if let Some(session_arc) = sessions.get(&session_id) {
            let mut session = session_arc.lock().await;
            session.screen_listeners.push(listener);
            Ok(())
        } else {
            Err(SessionError::SessionNotFound(session_id))
        }
    }

    /// Remove a previously-registered screen listener (by same_channel identity).
    pub async fn remove_screen_listener(&self, session_id: SessionId, listener: &ScreenListenerTx) {
        let sessions = self.sessions.read().await;
        if let Some(session_arc) = sessions.get(&session_id) {
            let mut session = session_arc.lock().await;
            session
                .screen_listeners
                .retain(|tx| !tx.same_channel(listener));
        }
    }

    pub async fn session_initial_buffer_infos(
        &self,
        session_id: SessionId,
    ) -> Result<Vec<Message>, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let messages = {
            let session = session.lock().await;
            session.attach_messages()
        };
        Ok(messages)
    }

    pub async fn remove_viewer(
        &self,
        session_id: SessionId,
        viewer_tx: &ConnectionTx,
    ) -> Result<u32, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let (count, resize_generation) = {
            let mut session = session.lock().await;
            let count = session.remove_viewer(viewer_tx);
            info!(
                "Removed viewer from session {} ({} viewers)",
                session_id, count
            );
            // The departed viewer may have been the most recently active one;
            // fall back to the remaining most recent viewer's size (if any).
            let generation = session
                .active_viewer_size()
                .and_then(|active| session.defer_resize(active));
            (count, generation)
        };
        if let Some(generation) = resize_generation {
            Self::spawn_deferred_resize_task(session_id, session.clone(), generation);
        }

        if count == 0 {
            self.clear_active_ai_conversation(session_id).await;
        }
        self.broadcast_session_list().await?;
        Ok(count)
    }

    pub async fn close_session(&self, session_id: SessionId) -> Result<(), SessionError> {
        let session = {
            let mut sessions = self.sessions.write().await;
            sessions.remove(&session_id)
        };

        match session {
            Some(session) => {
                self.screen_log_writer.close(session_id);
                self.clear_active_ai_conversation(session_id).await;
                let (viewers, group_id, tab_id) = {
                    let session = session.lock().await;
                    let viewers = session.viewers.clone();
                    let group_id = session.group_id;
                    let tab_id = session.tab_id;
                    (viewers, group_id, tab_id)
                };
                Self::terminate_session_transport(session_id, Arc::clone(&session)).await;

                for viewer in viewers {
                    let _ = viewer.send(Message::SessionClosed { session_id });
                }

                // Clean up observer — send AttentionResolved if there was an active prompt
                {
                    let mut s = session.lock().await;
                    if let Some(observer) = s.observer.take() {
                        observer.resolve_attention();
                        observer.shutdown();
                    }
                    // Clean up shell integration temp files
                    for path in &s.temp_cleanup_paths {
                        if path.is_dir() {
                            let _ = std::fs::remove_dir_all(path);
                        } else {
                            let _ = std::fs::remove_file(path);
                        }
                    }
                }

                self.rehome_tab_after_session_removed(tab_id).await;
                self.maybe_reset_last_sibling_group(group_id).await;
                self.broadcast_session_list().await?;
                info!("Session closed: {}", session_id);
                Ok(())
            }
            None => Err(SessionError::SessionNotFound(session_id)),
        }
    }

    /// Applications known to kill their ConPTY-hosted parent shell on exit
    /// (see anomalyco/opencode#26480: opencode's server child corrupts the
    /// shared pseudoconsole and the shell crashes on its next console
    /// access). When such an app was running in a session whose PTY child
    /// just died, the session respawns a fresh shell instead of closing.
    const RESPAWN_ON_EXIT_APPS: &[&str] = &["opencode"];

    fn should_respawn_on_app_exit(app: &SessionApp) -> bool {
        Self::RESPAWN_ON_EXIT_APPS.contains(&app.0.as_str())
    }

    /// Single funnel for PTY-child death (exit watcher, read EOF, reader
    /// error). Exit events from a replaced PTY carry an older
    /// `io_generation` and are ignored.
    pub async fn handle_child_exit(
        &self,
        session_id: SessionId,
        io_generation: u64,
    ) -> Result<(), SessionError> {
        let respawn_app = {
            let session = self.get_session(session_id).await;
            match session {
                Some(session) => {
                    let s = session.lock().await;
                    if s.io_generation != io_generation {
                        return Ok(());
                    }
                    (matches!(s.io, SessionIo::Pty(..)) && Self::should_respawn_on_app_exit(&s.app))
                        .then(|| s.app.0.clone())
                }
                None => None,
            }
        };
        if let Some(app) = respawn_app {
            info!(
                "Session {}: parent shell died while '{}' was running; respawning shell instead of closing",
                session_id, app
            );
            return self.respawn_session_shell(session_id).await;
        }
        self.close_session(session_id).await
    }

    /// Replace the session's dead PTY with a fresh shell launched the same
    /// way the session was originally created, keeping the tab alive. Used
    /// when an application killed its parent shell (see
    /// `RESPAWN_ON_EXIT_APPS`) — the terminal must survive the app.
    async fn respawn_session_shell(&self, session_id: SessionId) -> Result<(), SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;
        let (program, args, cwd, size, shell_kind, observer) = {
            let s = session.lock().await;
            (
                s.launch_program.clone(),
                s.launch_args.clone(),
                s.current_working_path.clone(),
                s.size,
                s.shell_kind,
                s.observer.clone(),
            )
        };
        if program.is_empty() {
            // Not a shell-launched session (e.g. UART); nothing to respawn.
            return self.close_session(session_id).await;
        }
        // The prompt the user faced died with the old shell.
        if let Some(observer) = &observer {
            observer.resolve_attention();
            // The dead app never emitted an exit marker — clear the
            // observer's stale app state or every later launch goes
            // undetected (shell integration only detects while no app is
            // active).
            observer.reset_app_tracking();
            // Re-install the sequence callback in case a previous app exit
            // cleared it.
            observer.sync_sequence_callback();
        }

        // Same launch recipe as `create_session`, minus the client-supplied
        // env (not retained; empty for typical sessions).
        let env = HashMap::new();
        let integration = shell_integration::prepare_integration(
            session_id,
            shell_kind,
            &program,
            &args,
            cwd.as_deref(),
            &env,
        )?;
        let mut merged_env = base_terminal_env(shell_kind, &program, integration.is_some(), &env);
        let config = self.config_manager.snapshot().await;
        apply_configured_env_vars(&mut merged_env, &config.terminal.env_vars);
        #[cfg(target_os = "linux")]
        crate::desktop_env::apply_graphical_env(&mut merged_env);
        let mut final_args = args.clone();
        let mut cleanup_paths = Vec::new();
        if let Some(ref integ) = integration {
            for (key, value) in integ.env_additions() {
                merged_env.insert(key.clone(), value.clone());
            }
            if let Some(args) = integ.args_override() {
                final_args = args.to_vec();
            }
            cleanup_paths = integ.temp_paths().to_vec();
        }
        apply_vterm_terminal_env(&mut merged_env);

        let pty = vterm_core::pty::PortablePty::new_command(
            size,
            &program,
            &final_args,
            &merged_env,
            cwd.as_deref(),
        )
        .map_err(|error| SessionError::PtyError(error.to_string()))?;

        let old_io = {
            let mut s = session.lock().await;
            s.io_generation += 1;
            s.temp_cleanup_paths.extend(cleanup_paths);
            s.app = SessionApp::none();
            s.app_status = SessionAppStatus::None;
            s.app_status_summary = None;
            std::mem::replace(&mut s.io, SessionIo::new_pty(pty, session_id))
        };
        // The old PTY child is already dead (that is why we are here); drop
        // the transport without `terminate`, which would wait on the exit
        // watcher we may be running on.
        drop(old_io);

        // Reset the server-side terminal model and any client state the app
        // left behind (alt screen, mouse tracking, cursor visibility, SGR),
        // as if the new shell had emitted the sequence itself.
        self.process_session_output(
            session_id,
            b"\x1b[?1049l\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[?25h\x1b[0m\r\n".to_vec(),
        )
        .await?;

        self.spawn_session_reader(session_id, Arc::clone(&session))
            .await?;
        self.broadcast_session_list().await?;
        info!("Session {}: shell respawned after app exit", session_id);
        Ok(())
    }

    pub async fn close_tab(&self, tab_id: TabId) -> Result<Vec<SessionId>, SessionError> {
        let closing = {
            let mut sessions = self.sessions.write().await;
            let closing_ids = sessions
                .iter()
                .filter_map(|(session_id, session)| {
                    session
                        .try_lock()
                        .ok()
                        .and_then(|session| (session.tab_id == tab_id).then_some(*session_id))
                })
                .collect::<Vec<_>>();
            closing_ids
                .into_iter()
                .filter_map(|session_id| {
                    sessions
                        .remove(&session_id)
                        .map(|session| (session_id, session))
                })
                .collect::<Vec<_>>()
        };

        if closing.is_empty() {
            return Err(SessionError::SessionNotFound(tab_id));
        }

        let mut closed_group_ids = Vec::new();
        let mut closed_session_ids = Vec::new();
        for (session_id, session) in closing {
            self.screen_log_writer.close(session_id);
            self.clear_active_ai_conversation(session_id).await;
            closed_session_ids.push(session_id);
            let (viewers, group_id) = {
                let session = session.lock().await;
                let viewers = session.viewers.clone();
                let group_id = session.group_id;
                (viewers, group_id)
            };
            Self::terminate_session_transport(session_id, Arc::clone(&session)).await;

            for viewer in viewers {
                let _ = viewer.send(Message::SessionClosed { session_id });
            }

            {
                let mut s = session.lock().await;
                if let Some(observer) = s.observer.take() {
                    observer.resolve_attention();
                    observer.shutdown();
                }
                for path in &s.temp_cleanup_paths {
                    if path.is_dir() {
                        let _ = std::fs::remove_dir_all(path);
                    } else {
                        let _ = std::fs::remove_file(path);
                    }
                }
            }

            closed_group_ids.push(group_id);
            info!("Session closed with tab {}: {}", tab_id, session_id);
        }

        self.tab_layouts.write().await.remove(&tab_id);
        closed_group_ids.sort_unstable();
        closed_group_ids.dedup();
        for group_id in closed_group_ids {
            self.maybe_reset_last_sibling_group(group_id).await;
        }
        self.broadcast_session_list().await?;
        Ok(closed_session_ids)
    }

    /// Detach a session from its split group by resetting its group_id to 0.
    /// If this leaves a single sibling in the group, that sibling's group_id is also reset.
    pub async fn detach_session(&self, session_id: SessionId) -> Result<(), SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let old_group_id = {
            let mut s = session.lock().await;
            let old = s.group_id;
            s.group_id = 0;
            s.tab_id = session_id;
            old
        };

        if old_group_id != 0 {
            self.maybe_reset_last_sibling_group(old_group_id).await;
        }

        self.broadcast_session_list().await?;
        info!(
            "Session detached from group: {} (was group {})",
            session_id, old_group_id
        );
        Ok(())
    }

    pub async fn rename_session(
        &self,
        session_id: SessionId,
        session_name: Option<String>,
    ) -> Result<(), SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let (title, session_name) = {
            let mut session = session.lock().await;
            session.session_name = session_name.and_then(|value| {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            });
            (session.title.clone(), session.session_name.clone())
        };

        {
            let s = session.lock().await;
            if let Some(observer) = &s.observer {
                observer.update_metadata(title, session_name);
            }
        }

        self.broadcast_session_list().await?;
        Ok(())
    }

    pub async fn set_notification_enabled(
        &self,
        session_id: SessionId,
        enabled: bool,
    ) -> Result<(), SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        {
            let mut s = session.lock().await;
            s.notification_enabled = enabled;
            if let Some(observer) = &s.observer {
                observer.set_notification_enabled(enabled);
            }
        }

        self.broadcast_session_list().await?;
        Ok(())
    }

    pub async fn set_session_encoding(
        &self,
        session_id: SessionId,
        encoding: String,
    ) -> Result<(), SessionError> {
        let encoding = encoding.trim().to_ascii_lowercase();
        if encoding.is_empty() {
            return Err(SessionError::Unsupported(
                "terminal encoding cannot be empty".to_string(),
            ));
        }
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        {
            let mut session = session.lock().await;
            if !matches!(&session.io, SessionIo::Serial(_)) {
                return Err(SessionError::Unsupported(
                    "encoding can only be changed for UART sessions".to_string(),
                ));
            }
            // Host colors and the current OSC overrides may predate the
            // retained history window; seed both, then let replayed OSC
            // sequences override them. Historical query replies are stale.
            let theme_fg = session.terminal.theme_default_foreground();
            let theme_bg = session.terminal.theme_default_background();
            let (osc_fg, osc_bg) = session.terminal.osc_default_colors();
            let mut terminal = Terminal::new_with_type_and_encoding(
                session.size,
                session.terminal_type.as_deref(),
                Some(&encoding),
            );
            terminal.set_theme_default_colors(theme_fg, theme_bg);
            terminal.set_osc_default_colors(osc_fg, osc_bg);
            terminal.process(&session.raw_history.snapshot());
            terminal.take_pending_query_responses();
            terminal.mark_dirty();
            session.terminal = terminal;
            session.encoding = Some(encoding);
        }

        self.broadcast_session_list().await?;
        Ok(())
    }

    /// Record the viewer's Host default colors without disturbing a running
    /// application's OSC overrides. Used during session/stream establishment.
    pub async fn set_terminal_theme_colors(
        &self,
        session_id: SessionId,
        background: Option<[u8; 3]>,
        foreground: Option<[u8; 3]>,
    ) -> Result<(), SessionError> {
        self.update_terminal_theme_colors(session_id, background, foreground, false)
            .await
    }

    /// Apply an explicit Host theme selection. Clearing OSC overrides makes
    /// the new theme effective immediately and ensures later OSC 10/11 probes
    /// receive the newly selected colors.
    pub async fn apply_terminal_theme_colors(
        &self,
        session_id: SessionId,
        background: Option<[u8; 3]>,
        foreground: Option<[u8; 3]>,
    ) -> Result<(), SessionError> {
        self.update_terminal_theme_colors(session_id, background, foreground, true)
            .await
    }

    async fn update_terminal_theme_colors(
        &self,
        session_id: SessionId,
        background: Option<[u8; 3]>,
        foreground: Option<[u8; 3]>,
        reset_osc_overrides: bool,
    ) -> Result<(), SessionError> {
        let to_color = |rgb: [u8; 3]| Color::Rgb(rgb[0], rgb[1], rgb[2]);
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let mut session = session.lock().await;
        session
            .terminal
            .set_theme_default_colors(foreground.map(to_color), background.map(to_color));
        if reset_osc_overrides {
            session.terminal.set_osc_default_colors(None, None);
        }
        Ok(())
    }

    /// Clear a UART session's server-side screen: drop the retained
    /// raw-history window, reset the emulator grid, and forget the screen
    /// log. Stream offsets keep advancing (the client-side "clear screen"
    /// relies on the same monotonic offsets), so attaching viewers receive
    /// a blank screen plus only post-clear output instead of the old
    /// buffer. UART sessions only — a PTY's screen belongs to the shell
    /// running inside it.
    pub async fn clear_session_screen(&self, session_id: SessionId) -> Result<(), SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        {
            let mut session = session.lock().await;
            if !matches!(&session.io, SessionIo::Serial(_)) {
                return Err(SessionError::Unsupported(
                    "screen clearing is only supported for UART sessions".to_string(),
                ));
            }
            session.raw_history.clear_retained();
            let theme_fg = session.terminal.theme_default_foreground();
            let theme_bg = session.terminal.theme_default_background();
            let (osc_fg, osc_bg) = session.terminal.osc_default_colors();
            let mut terminal = Terminal::new_with_type_and_encoding(
                session.size,
                session.terminal_type.as_deref(),
                session.encoding.as_deref(),
            );
            terminal.set_theme_default_colors(theme_fg, theme_bg);
            terminal.set_osc_default_colors(osc_fg, osc_bg);
            terminal.mark_dirty();
            session.terminal = terminal;
            session.screen_history_lines.clear();
            session.screen_history_logged_rows = 0;
        }

        Ok(())
    }

    /// Clear a terminal's server-side screen buffer: blank the visible grid
    /// and purge history so screen reads start from a clean buffer. Viewer
    /// clients render the raw output stream in their own emulators and are
    /// not affected.
    ///
    /// UART sessions defer to [`SessionManager::clear_session_screen`] — the
    /// same path as the client's `ClearSessionScreen` message — which drops
    /// the retained raw-history window (so a re-attach cannot replay
    /// pre-clear output), hard-resets the emulator, and forgets the screen
    /// log. Every other session type feeds the standard `clear` sequences
    /// (`ESC[H ESC[2J ESC[3J`) through the live emulator instead, blanking
    /// the grid and purging scrollback while preserving session state —
    /// terminal type, encoding, and any running alternate-screen app (ED 3
    /// skips the scrollback purge while an alternate-screen app owns the
    /// display, like xterm).
    pub async fn clear_terminal_buffer(&self, session_id: SessionId) -> Result<(), SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let (is_uart, screen_listeners) = {
            let s = session.lock().await;
            (
                matches!(&s.io, SessionIo::Serial(_)),
                s.screen_listeners.clone(),
            )
        };

        if is_uart {
            // Lock is released above, so the delegated call can take it.
            self.clear_session_screen(session_id).await?;
        } else {
            let mut session = session.lock().await;
            session.terminal.process(b"\x1b[H\x1b[2J\x1b[3J");
            session.screen_history_lines.clear();
            session.screen_history_logged_rows = 0;
        }

        // Let in-process screen subscribers (e.g. MCP streaming resources) see
        // the blank screen immediately instead of waiting for the next output.
        for listener in &screen_listeners {
            let _ = listener.send(session_id);
        }
        Ok(())
    }

    pub async fn set_auto_confirm(
        &self,
        session_id: SessionId,
        enabled: bool,
    ) -> Result<(), SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        {
            let mut s = session.lock().await;
            s.auto_confirm_enabled = enabled;
            if let Some(observer) = &s.observer {
                observer.set_auto_confirm(enabled);
            }
        }

        self.broadcast_session_list().await?;
        Ok(())
    }

    pub async fn update_session_observation(
        &self,
        session_id: SessionId,
        app: SessionApp,
        app_status: SessionAppStatus,
        app_status_summary: Option<String>,
    ) -> Result<(), SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let changed = {
            let mut session = session.lock().await;
            if session.app == app
                && session.app_status == app_status
                && session.app_status_summary == app_status_summary
            {
                false
            } else {
                session.app = app;
                session.app_status = app_status;
                session.app_status_summary = app_status_summary;
                true
            }
        };

        if changed {
            self.broadcast_session_list().await?;
        }

        Ok(())
    }

    pub async fn process_session_output(
        &self,
        session_id: SessionId,
        data: Vec<u8>,
    ) -> Result<(), SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let process_started = std::time::Instant::now();
        // The terminal-model update (raw history append + vte parse +
        // scrollback extraction) is CPU-bound and scales with batch size.
        // Run it on the blocking pool instead of a tokio worker: with N
        // sessions streaming, worker threads stayed saturated with parsing
        // and starved connection I/O, heartbeats and other sessions' drain
        // tasks. The session lock is still held for the whole update — byte
        // order and model consistency are unchanged — but holding it no
        // longer occupies an async worker.
        let echo_probe = vterm_common::latency::echo_ms_since_input(session_id).is_some();
        let parse_session = Arc::clone(&session);
        let (
            data,
            viewers,
            start_offset,
            observer,
            observed_terminal_state,
            cwd_changed,
            screen_history_lines,
            screen_listeners,
            query_responses,
        ) = tokio::task::spawn_blocking(move || {
            let lock_started = std::time::Instant::now();
            let mut session = parse_session.blocking_lock();
            let lock_wait_ms = lock_started.elapsed().as_millis() as u64;
            if echo_probe
                && vterm_common::latency::sample(
                    "server:output-session-lock",
                    session_id,
                    lock_wait_ms as f64,
                    5.0,
                    3.0,
                )
            {
                vterm_common::latency_log!(
                    stage = "server:output-session-lock",
                    session = %session_id,
                    lock_wait_ms,
                    "[latency] session lock contended before output processing"
                );
            }
            // Raw history and the terminal model represent the same byte
            // stream. Update both while holding one lock so no other caller can
            // observe different output orders between the two representations.
            let start_offset = session.raw_history.append(&data);
            let observer = session.observer.clone();
            let (cwd_changed, observed_terminal_state) =
                session.apply_terminal_output(&data, observer.is_some());
            // Device query replies generated while parsing (e.g. the kitty
            // keyboard flags report) must go back to the application's stdin.
            let query_responses = session.terminal.take_pending_query_responses();
            let screen_history_lines = session.take_new_screen_history_lines();
            (
                data,
                session.viewers.clone(),
                start_offset,
                observer,
                observed_terminal_state,
                cwd_changed,
                screen_history_lines,
                session.screen_listeners.clone(),
                query_responses,
            )
        })
        .await
        .map_err(|error| SessionError::PtyError(format!("output parse task failed: {error}")))?;
        // Shared with the observer actor without copying the batch; viewers
        // still get their own Vec clones as before.
        let data = Arc::new(data);

        if !query_responses.is_empty() {
            // If the application is blocked waiting for this reply (e.g. a
            // keyboard-protocol or cell-size query), any delay here lands
            // directly on the echo path — log the round trip when slow.
            if let Some(echo_ms) = vterm_common::latency::echo_ms_since_input(session_id) {
                let preview: String = query_responses
                    .iter()
                    .take(24)
                    .map(|b| match *b {
                        0x1b => "\\e".to_string(),
                        0x20..=0x7e => (*b as char).to_string(),
                        b => format!("\\x{b:02x}"),
                    })
                    .collect();
                if vterm_common::latency::sample(
                    "server:query-response-written",
                    session_id,
                    echo_ms as f64,
                    20.0,
                    3.0,
                ) {
                    vterm_common::latency_log!(
                        stage = "server:query-response-written",
                        session = %session_id,
                        echo_ms,
                        bytes = query_responses.len(),
                        preview = %preview,
                        "[latency] device query reply written back to PTY late"
                    );
                }
            }
            let writer = {
                let session = session.lock().await;
                session.io.writer()
            };
            writer.write(&query_responses).await?;
        }

        // Viewer delivery remains ahead of logging, observer work, and CWD
        // notifications, but follows the atomic history/model update above.
        for viewer in viewers {
            if viewer
                .send(Message::Output {
                    session_id,
                    start_offset,
                    data: (*data).clone(),
                })
                .is_err()
            {
                warn!(
                    "Session {} failed to enqueue output for a viewer bytes={}",
                    session_id,
                    data.len()
                );
            }
        }
        let lock_held_ms = process_started.elapsed().as_millis() as u64;
        if let Some(echo_ms) = vterm_common::latency::echo_ms_since_input(session_id) {
            if vterm_common::latency::sample(
                "server:output-dispatched",
                session_id,
                echo_ms as f64,
                50.0,
                3.0,
            ) {
                vterm_common::latency_log!(
                    stage = "server:output-dispatched",
                    session = %session_id,
                    echo_ms,
                    bytes = data.len(),
                    start_offset,
                    lock_held_ms,
                    "[latency] keystroke echo dispatched to viewers late"
                );
            }
        }

        self.screen_log_writer
            .append(session_id, screen_history_lines);

        if let (Some(observer), Some(observed_terminal_state)) = (observer, observed_terminal_state)
        {
            // Fire-and-forget: the observer actor owns its state and applies
            // updates in order, so a slow QuickJS handler can no longer stall
            // this drain task (previously: up to 100ms per batch on the
            // observer lock timeout).
            observer.record_output(Arc::clone(&data), observed_terminal_state);
        }

        // Notify in-process screen subscribers (e.g. MCP streaming resources).
        for listener in &screen_listeners {
            let _ = listener.send(session_id);
        }

        if cwd_changed {
            let manager = self.clone();
            tokio::spawn(async move {
                if let Err(error) = manager.broadcast_session_list().await {
                    warn!(
                        session_id = %session_id,
                        error = %error,
                        "failed to broadcast session list after session CWD change"
                    );
                }
                if let Some(watcher) = manager.fs_watcher_manager.read().await.clone() {
                    if let Err(error) = watcher
                        .refresh_session_quick_commands(session_id, &manager)
                        .await
                    {
                        warn!(
                            session_id = %session_id,
                            error = %error,
                            "failed to refresh quick commands after session CWD change"
                        );
                    }
                    // The git watch prefers the viewing path, so a CWD change
                    // only moves it while no listing has been adopted yet.
                    if let Err(error) = watcher
                        .refresh_session_git_watch(session_id, &manager)
                        .await
                    {
                        warn!(
                            session_id = %session_id,
                            error = %error,
                            "failed to refresh git watch after session CWD change"
                        );
                    }
                }
            });
        }

        Ok(())
    }

    pub async fn forward_input(
        &self,
        session_id: SessionId,
        _viewer_tx: Option<&ConnectionTx>,
        data: Vec<u8>,
    ) -> Result<(), SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let is_newline_only = data == b"\r";
        let input_received_at = std::time::Instant::now();
        let (observer, current_line_text) = {
            let lock_started = std::time::Instant::now();
            let s = session.lock().await;
            let lock_wait_ms = lock_started.elapsed().as_millis() as u64;
            if vterm_common::latency::sample(
                "server:input-session-lock",
                session_id,
                lock_wait_ms as f64,
                5.0,
                3.0,
            ) {
                vterm_common::latency_log!(
                    stage = "server:input-session-lock",
                    session = %session_id,
                    lock_wait_ms,
                    "[latency] session lock contended for input"
                );
            }
            let observer = s.observer.clone();
            let current_line_text =
                (is_newline_only && observer.is_some()).then(|| s.terminal.current_line_text());
            (observer, current_line_text)
        };

        if let Some(observer) = observer {
            // Fire-and-forget: the observer actor applies the input in order
            // and syncs the sequence callback itself on app detection. The
            // keystroke path no longer waits on the observer lock — which
            // had no timeout and could sit behind a QuickJS handler.
            observer.record_input(is_newline_only, current_line_text);
        }

        let writer = {
            let lock_started = std::time::Instant::now();
            let mut session = session.lock().await;
            let lock_wait_ms = lock_started.elapsed().as_millis() as u64;
            if vterm_common::latency::sample(
                "server:input-writer-session-lock",
                session_id,
                lock_wait_ms as f64,
                5.0,
                3.0,
            ) {
                vterm_common::latency_log!(
                    stage = "server:input-writer-session-lock",
                    session = %session_id,
                    lock_wait_ms,
                    "[latency] session lock contended before PTY write"
                );
            }
            flush_pending_resize_for_session(session_id, &mut session)?;
            session.io.writer()
        };
        // Stamp before the channel send: the writer thread may pick the
        // batch up within microseconds, and the echo budget should include
        // the channel wait + writer wake anyway.
        vterm_common::latency::note_input(session_id);
        writer.write(&data).await?;

        let total_ms = input_received_at.elapsed().as_millis() as u64;
        if vterm_common::latency::sample(
            "server:input-pty-written",
            session_id,
            total_ms as f64,
            20.0,
            3.0,
        ) {
            vterm_common::latency_log!(
                stage = "server:input-pty-written",
                session = %session_id,
                bytes = data.len(),
                total_ms,
                "[latency] input path to PTY was slow"
            );
        }

        Ok(())
    }

    pub async fn apply_environment(
        &self,
        session_id: SessionId,
        environment_id: &str,
        enabled: bool,
    ) -> Result<(), SessionError> {
        let config = self.config_manager.snapshot().await;
        let node = config
            .environment
            .root
            .find_node(environment_id)
            .ok_or_else(|| SessionError::PtyError("environment profile not found".to_string()))?
            .clone();
        // Only Leaf nodes (environment-variable collections) are applicable.
        // Groups are pure folders and carry no variables.
        if !node.is_leaf() {
            return Err(SessionError::PtyError(
                "environment profile must be a leaf node".to_string(),
            ));
        }
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let (terminal_os, shell_kind) = {
            let session = session.lock().await;
            let Some(terminal_os) = session.terminal_os else {
                return Err(SessionError::PtyError(
                    "environment profiles require a shell session".to_string(),
                ));
            };
            (terminal_os, session.shell_kind)
        };

        let invocation =
            write_environment_script(session_id, terminal_os, shell_kind, &node, enabled)?;
        let writer = {
            let mut session = session.lock().await;
            flush_pending_resize_for_session(session_id, &mut session)?;
            session.io.writer()
        };
        writer.write(&invocation).await?;
        Ok(())
    }

    /// Generate and source a one-shot script in the session's native shell.
    /// The script always changes to `repo_root` before evaluating `command`.
    pub async fn run_quick_command(
        &self,
        session_id: SessionId,
        repo_root: &std::path::Path,
        command: &str,
        env: &std::collections::HashMap<String, String>,
    ) -> Result<(), SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let (terminal_os, shell_kind) = {
            let session = session.lock().await;
            let Some(terminal_os) = session.terminal_os else {
                return Err(SessionError::PtyError(
                    "quick commands require a shell session".to_string(),
                ));
            };
            (terminal_os, session.shell_kind)
        };

        let invocation = write_quick_command_script(
            session_id,
            terminal_os,
            shell_kind,
            repo_root,
            command,
            env,
        )?;
        let writer = {
            let mut session = session.lock().await;
            flush_pending_resize_for_session(session_id, &mut session)?;
            session.io.writer()
        };
        writer.write(&invocation).await?;
        Ok(())
    }

    pub async fn fetch_history_chunk(
        &self,
        session_id: SessionId,
        before_offset: u64,
        max_bytes: usize,
    ) -> Result<Message, SessionError> {
        let fetch_started = Instant::now();
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;
        let session = session.lock().await;
        let message = session.history_chunk_message(before_offset, max_bytes);
        if let Message::HistoryRawChunk {
            start_offset,
            end_offset,
            ref data,
            ..
        } = message
        {
            trace!(
                "session_manager history read session={} before_offset={} requested_max_bytes={} start_offset={} end_offset={} payload_bytes={} total_ms={}",
                session_id,
                before_offset,
                max_bytes,
                start_offset,
                end_offset,
                data.len(),
                fetch_started.elapsed().as_millis()
            );
        }
        Ok(message)
    }

    pub async fn export_raw_history(&self, session_id: SessionId) -> Result<PathBuf, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let (size, raw_history) = {
            let session = session.lock().await;
            (session.size, session.raw_history.snapshot())
        };

        let path = debug_raw_history_path(session_id, "server", size);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        }
        fs::write(&path, &raw_history)
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        info!(
            "exported raw history session={} bytes={} path={}",
            session_id,
            raw_history.len(),
            path.display()
        );
        Ok(path)
    }

    pub async fn voice_input_polish_context(
        &self,
        context: &VoiceInputOptimizationContext,
    ) -> VoicePolishContext {
        match context {
            VoiceInputOptimizationContext::Transparent { session_id } => {
                self.voice_command_context(*session_id).await
            }
            VoiceInputOptimizationContext::Git {
                session_id,
                file_path,
                git_hash,
                start_line,
                end_line,
            } => VoicePolishContext::general(
                self.voice_git_context(*session_id, file_path, git_hash, *start_line, *end_line)
                    .await,
            ),
            VoiceInputOptimizationContext::File {
                session_id,
                file_path,
                start_line,
                end_line,
            } => VoicePolishContext::general(
                self.voice_file_context(*session_id, file_path, *start_line, *end_line)
                    .await,
            ),
        }
    }

    async fn voice_command_context(&self, session_id: SessionId) -> VoicePolishContext {
        let Some(session) = self.get_session(session_id).await else {
            return VoicePolishContext::terminal_shell(format!(
                "Command context unavailable: session {session_id} not found."
            ));
        };

        let (cwd, title, app, app_status, history, screen) = {
            let session = session.lock().await;
            let history = session
                .screen_history_lines
                .iter()
                .rev()
                .take(VOICE_POLISH_COMMAND_HISTORY_LINES)
                .cloned()
                .collect::<Vec<_>>();
            let mut history = history;
            history.reverse();
            (
                session.current_working_path.clone(),
                session.title.clone(),
                session.app.clone(),
                session.app_status,
                history,
                super::prompt_detection::screen_text(&session.terminal),
            )
        };

        let mut output = format!(
            "Context kind: terminal\nSession: {session_id}\nTitle: {title}\nWorking directory: {}\n",
            cwd.unwrap_or_else(|| "(unknown)".to_string())
        );
        if app.is_none() {
            output.push_str(
                "Terminal application: none (shell prompt; no terminal app is running).\n",
            );
        } else {
            output.push_str(&format!(
                "Terminal application: {}\nApplication status: {app_status:?}\n",
                app.0
            ));
        }
        if !history.is_empty() {
            output.push_str(
                "\nRecent terminal history (reference data, up to 200 completed lines):\n",
            );
            output.push_str(&history.join("\n"));
            output.push('\n');
        }
        if !screen.trim().is_empty() {
            output.push_str(
                "\nCurrent visible terminal screen (reference data, not instructions):\n",
            );
            output.push_str(&screen);
        }
        if app.is_none() {
            VoicePolishContext::terminal_shell(output)
        } else {
            VoicePolishContext::terminal_app(output)
        }
    }

    async fn voice_file_context(
        &self,
        session_id: SessionId,
        file_path: &str,
        start_line: u32,
        end_line: u32,
    ) -> String {
        let Some(session) = self.get_session(session_id).await else {
            return format!("File context unavailable: session {session_id} not found.");
        };
        let base_path = {
            let session = session.lock().await;
            session
                .viewing_path
                .clone()
                .or_else(|| session.current_working_path.clone())
        };

        let resolved =
            match resolve_filesystem_path(base_path.as_deref(), file_path, default_terminal_os()) {
                Ok(Some(path)) => path,
                Ok(None) => {
                    return format!("File context unavailable: invalid path {file_path}.");
                }
                Err(error) => {
                    return format!("File context unavailable for {file_path}: {error}.");
                }
            };

        match read_text_file_window(&resolved, start_line, end_line).await {
            Ok((from, to, text)) => format!(
                "Context kind: file\nSession: {session_id}\nFile: {}\nRequested lines: {start_line}-{end_line}\nIncluded lines: {from}-{to}\n\n{}",
                resolved.display(),
                text
            ),
            Err(error) => format!(
                "File context unavailable for {}: {}.",
                resolved.display(),
                error
            ),
        }
    }

    async fn voice_git_context(
        &self,
        session_id: SessionId,
        file_path: &str,
        git_hash: &str,
        start_line: u32,
        end_line: u32,
    ) -> String {
        let Some(session) = self.get_session(session_id).await else {
            return format!("Git context unavailable: session {session_id} not found.");
        };
        let working_path = {
            let session = session.lock().await;
            session.git_working_path()
        };
        let Some(working_path) = working_path else {
            return "Git context unavailable: no working directory for session.".to_string();
        };

        let file_path = file_path.to_string();
        let git_hash = git_hash.to_string();
        let worker_file_path = file_path.clone();
        let worker_git_hash = git_hash.clone();
        let loaded = tokio::task::spawn_blocking(move || {
            read_git_file_at_revision(&working_path, &worker_file_path, &worker_git_hash)
        })
        .await;

        match loaded {
            Ok(Ok((repo_path, content))) => {
                let (from, to, text) = extract_line_window(
                    &content,
                    start_line,
                    end_line,
                    VOICE_POLISH_FILE_CONTEXT_LINES,
                );
                format!(
                    "Context kind: git\nSession: {session_id}\nFile: {repo_path}\nGit revision: {git_hash}\nRequested lines: {start_line}-{end_line}\nIncluded lines: {from}-{to}\n\n{text}"
                )
            }
            Ok(Err(error)) => format!("Git context unavailable for {file_path}: {error}."),
            Err(error) => format!("Git context task failed for {file_path}: {error}."),
        }
    }

    pub async fn resolve_fs_watch_path(
        &self,
        session_id: SessionId,
        path: &str,
    ) -> Result<PathBuf, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let base_path = {
            let session = session.lock().await;
            session
                .viewing_path
                .clone()
                .or_else(|| session.current_working_path.clone())
        };

        let resolved = resolve_filesystem_path(base_path.as_deref(), path, default_terminal_os())
            .map_err(SessionError::FileSystemError)?;
        resolved.ok_or_else(|| SessionError::FileSystemError("Invalid path".to_string()))
    }

    pub async fn resolve_git_watch_path(
        &self,
        session_id: SessionId,
    ) -> Result<PathBuf, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = {
            let session = session.lock().await;
            session.git_working_path()
        };

        let working_path = working_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;
        tokio::task::spawn_blocking(move || super::git::git_repo_root(&working_path))
            .await
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?
            .map(PathBuf::from)
    }
    pub async fn read_file(
        &self,
        session_id: SessionId,
        path: String,
        max_size: u64,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let base_path = {
            let session = session.lock().await;
            session
                .viewing_path
                .clone()
                .or_else(|| session.current_working_path.clone())
        };

        let runtime_os = default_terminal_os();
        let resolved = resolve_filesystem_path(base_path.as_deref(), &path, runtime_os)
            .map_err(SessionError::FileSystemError)?;

        let file_path = match resolved {
            Some(p) => p,
            None => {
                return Ok(Message::FileContent {
                    session_id,
                    path,
                    data: Vec::new(),
                    diff_ranges: Vec::new(),
                    error: Some("Invalid path".to_string()),
                });
            }
        };

        let metadata = tokio::fs::metadata(&file_path).await;
        let file_len = match metadata {
            Ok(meta) => {
                if !meta.is_file() {
                    return Ok(Message::FileContent {
                        session_id,
                        path,
                        data: Vec::new(),
                        diff_ranges: Vec::new(),
                        error: Some("Not a file".to_string()),
                    });
                }
                if meta.len() > max_size {
                    return Ok(Message::FileContent {
                        session_id,
                        path,
                        data: Vec::new(),
                        diff_ranges: Vec::new(),
                        error: Some(format!(
                            "File too large: {} > {} bytes",
                            meta.len(),
                            max_size
                        )),
                    });
                }
                meta.len()
            }
            Err(e) => {
                return Ok(Message::FileContent {
                    session_id,
                    path,
                    data: Vec::new(),
                    diff_ranges: Vec::new(),
                    error: Some(format!("Failed to read file: {}", e)),
                });
            }
        };

        let data = match tokio::fs::read(&file_path).await {
            Ok(content) => content,
            Err(e) => {
                return Ok(Message::FileContent {
                    session_id,
                    path,
                    data: Vec::new(),
                    diff_ranges: Vec::new(),
                    error: Some(format!("Failed to read file: {}", e)),
                });
            }
        };

        let diff_ranges = if file_len < 1024 * 1024 {
            base_path
                .as_deref()
                .map(
                    |working_path| match git_read_worktree_file_diff_ranges(working_path, &path) {
                        Ok(ranges) => ranges,
                        Err(error) => {
                            debug!(
                                "file editor git diff ranges unavailable for {}: {}",
                                path, error
                            );
                            Vec::new()
                        }
                    },
                )
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(Message::FileContent {
            session_id,
            path,
            data,
            diff_ranges,
            error: None,
        })
    }

    /// Prepare a file response whose body is read incrementally by the
    /// transport producer instead of allocating the entire file first.
    pub async fn read_file_transport(
        &self,
        session_id: SessionId,
        path: String,
        max_size: u64,
    ) -> Result<EncodedMessage, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;
        let base_path = {
            let session = session.lock().await;
            session
                .viewing_path
                .clone()
                .or_else(|| session.current_working_path.clone())
        };
        let file_path =
            match resolve_filesystem_path(base_path.as_deref(), &path, default_terminal_os())
                .map_err(SessionError::FileSystemError)?
            {
                Some(path) => path,
                None => {
                    return encode_message(Message::FileContent {
                        session_id,
                        path,
                        data: Vec::new(),
                        diff_ranges: Vec::new(),
                        error: Some("Invalid path".to_string()),
                    })
                    .map_err(|error| SessionError::FileSystemError(error.to_string()));
                }
            };
        let metadata = match tokio::fs::metadata(&file_path).await {
            Ok(metadata) if metadata.is_file() => metadata,
            Ok(_) => {
                return encode_message(Message::FileContent {
                    session_id,
                    path,
                    data: Vec::new(),
                    diff_ranges: Vec::new(),
                    error: Some("Not a file".to_string()),
                })
                .map_err(|error| SessionError::FileSystemError(error.to_string()));
            }
            Err(error) => {
                return encode_message(Message::FileContent {
                    session_id,
                    path,
                    data: Vec::new(),
                    diff_ranges: Vec::new(),
                    error: Some(format!("Failed to read file: {error}")),
                })
                .map_err(|error| SessionError::FileSystemError(error.to_string()));
            }
        };
        if metadata.len() > max_size {
            return encode_message(Message::FileContent {
                session_id,
                path,
                data: Vec::new(),
                diff_ranges: Vec::new(),
                error: Some(format!(
                    "File too large: {} > {} bytes",
                    metadata.len(),
                    max_size
                )),
            })
            .map_err(|error| SessionError::FileSystemError(error.to_string()));
        }
        let diff_ranges = if metadata.len() < 1024 * 1024 {
            base_path
                .as_deref()
                .and_then(|working_path| {
                    git_read_worktree_file_diff_ranges(working_path, &path).ok()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let mut encoded = encode_message(Message::FileContent {
            session_id,
            path: path.clone(),
            data: Vec::new(),
            diff_ranges,
            error: None,
        })
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        encoded.body = OutboundBody::from_file(&file_path)
            .await
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        {
            let mut session = session.lock().await;
            session.record_recent_file(path);
        }
        let _ = self.broadcast_session_list().await;
        Ok(encoded)
    }
    pub async fn session_workspace_path(
        &self,
        session_id: SessionId,
    ) -> Result<Option<String>, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let session = session.lock().await;
        Ok(session.git_working_path())
    }

    /// Terminal quick commands follow the shell's actual CWD rather than the
    /// file browser's potentially pinned viewing path.
    pub async fn session_current_working_path(
        &self,
        session_id: SessionId,
    ) -> Result<Option<String>, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;
        let current_working_path = session.lock().await.current_working_path.clone();
        Ok(current_working_path)
    }

    pub async fn save_file(
        &self,
        session_id: SessionId,
        path: String,
        data: Vec<u8>,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let base_path = {
            let session = session.lock().await;
            session
                .viewing_path
                .clone()
                .or_else(|| session.current_working_path.clone())
        };

        let runtime_os = default_terminal_os();
        let resolved = resolve_filesystem_path(base_path.as_deref(), &path, runtime_os)
            .map_err(SessionError::FileSystemError)?;

        let Some(file_path) = resolved else {
            return Ok(Message::SaveFileResult {
                session_id,
                path,
                error: Some("Invalid path".to_string()),
            });
        };

        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        }

        tokio::fs::write(&file_path, &data)
            .await
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

        {
            let mut session = session.lock().await;
            session.record_recent_file(path.clone());
        }
        let _ = self.broadcast_session_list().await;

        Ok(Message::SaveFileResult {
            session_id,
            path,
            error: None,
        })
    }

    /// Persist an assembled transport body without materializing a large file
    /// in memory. The body is copied into a sibling staging file and flushed
    /// before it replaces the destination.
    pub async fn save_file_body(
        &self,
        session_id: SessionId,
        path: String,
        body: ReceivedBody,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;
        let base_path = {
            let session = session.lock().await;
            session
                .viewing_path
                .clone()
                .or_else(|| session.current_working_path.clone())
        };
        let resolved = resolve_filesystem_path(base_path.as_deref(), &path, default_terminal_os())
            .map_err(SessionError::FileSystemError)?;
        let Some(file_path) = resolved else {
            return Ok(Message::SaveFileResult {
                session_id,
                path,
                error: Some("Invalid path".to_string()),
            });
        };
        let parent = file_path.parent().ok_or_else(|| {
            SessionError::FileSystemError("Destination has no parent directory".to_string())
        })?;
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        persist_body_to_file(&file_path, body)
            .await
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

        {
            let mut session = session.lock().await;
            session.record_recent_file(path.clone());
        }
        let _ = self.broadcast_session_list().await;
        Ok(Message::SaveFileResult {
            session_id,
            path,
            error: None,
        })
    }

    pub async fn upload_temp_file(
        &self,
        session_id: SessionId,
        task_id: String,
        file_name: String,
        content_type: String,
        usage: TempFileUsage,
        body: ReceivedBody,
    ) -> Message {
        let result: Result<(PathBuf, u64), String> = async {
            let file_name = sanitize_temp_file_name(&file_name)?;
            let directory = std::env::temp_dir()
                .join("vterm-paste")
                .join(uuid::Uuid::new_v4().to_string());
            tokio::fs::create_dir_all(&directory)
                .await
                .map_err(|error| format!("Failed to create temporary directory: {error}"))?;
            let path = directory.join(file_name);
            let bytes = match &body {
                ReceivedBody::Empty => return Err("Temporary file is empty".to_string()),
                ReceivedBody::Memory(data) => {
                    tokio::fs::write(&path, data)
                        .await
                        .map_err(|error| format!("Failed to write temporary file: {error}"))?;
                    data.len() as u64
                }
                ReceivedBody::TempFile(spool) => tokio::fs::copy(spool.path(), &path)
                    .await
                    .map_err(|error| format!("Failed to write temporary file: {error}"))?,
            };
            if let TempFileUsage::InsertIntoTerminal { bracketed_paste } = usage {
                let input = temp_file_terminal_input(&path, bracketed_paste);
                self.forward_input(session_id, None, input)
                    .await
                    .map_err(|error| format!("Failed to insert temporary file path: {error}"))?;
            }
            info!(%task_id, ?session_id, %content_type, bytes, path = %path.display(), "temporary file uploaded");
            Ok((path, bytes))
        }
        .await;
        let (path, bytes, error) = match result {
            Ok((path, bytes)) => (Some(path.to_string_lossy().into_owned()), bytes, None),
            Err(error) => {
                warn!(%task_id, ?session_id, %error, "temporary file upload failed");
                (None, 0, Some(error))
            }
        };
        Message::TempFileUploadResult {
            task_id,
            session_id,
            path,
            bytes,
            error,
        }
    }

    /// Receive a framed upload stream (see `vterm_core::folder_stream`) —
    /// the non-streaming fallback that consumes a complete spooled body —
    /// and write every entry under `dst_dir`, creating parent directories
    /// from each entry's '/'-separated relative path. The whole transfer is
    /// ONE task: an entry-level failure skips that file; a stream Abort
    /// fails the task.
    pub async fn upload_body(
        &self,
        session_id: SessionId,
        task_id: String,
        dst_dir: String,
        body: ReceivedBody,
    ) -> Result<Message, SessionError> {
        info!(
            %task_id,
            ?session_id,
            %dst_dir,
            body = match &body {
                ReceivedBody::Empty => "empty".to_string(),
                ReceivedBody::Memory(bytes) => format!("memory({} bytes)", bytes.len()),
                ReceivedBody::TempFile(spool) => format!("tempfile({} bytes)", spool.len()),
            },
            "upload: body assembled, writing files"
        );
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;
        let base_path = {
            let session = session.lock().await;
            session
                .viewing_path
                .clone()
                .or_else(|| session.current_working_path.clone())
        };

        let finish_task_id = task_id.clone();
        let finish = |files: u64, bytes: u64, error: Option<String>| {
            Ok(Message::UploadResult {
                task_id: finish_task_id,
                session_id,
                files,
                bytes,
                error,
            })
        };

        let fs_dir =
            match resolve_filesystem_path(base_path.as_deref(), &dst_dir, default_terminal_os())
                .map_err(SessionError::FileSystemError)?
            {
                Some(path) => path,
                None => return finish(0, 0, Some("Invalid destination path".to_string())),
            };
        info!(%task_id, fs_dir = %fs_dir.display(), "upload: destination resolved");
        tokio::fs::create_dir_all(&fs_dir)
            .await
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

        struct FolderWriteState {
            decoder: FolderFrameDecoder,
            current: Option<(tokio::fs::File, u64)>,
            files_done: u64,
            bytes_done: u64,
        }
        async fn consume_chunk(
            state: &mut FolderWriteState,
            fs_dir: &Path,
            chunk: &[u8],
        ) -> Result<(), String> {
            for frame in state.decoder.push(chunk).map_err(|error| match error {
                FolderDecodeError::Aborted(reason) => format!("cancelled: {reason}"),
                FolderDecodeError::Malformed(detail) => format!("bad upload stream: {detail}"),
            })? {
                match frame {
                    FolderFrame::FileBegin { rel_path, size } => {
                        let rel_norm = rel_path.replace('/', std::path::MAIN_SEPARATOR_STR);
                        let target = vterm_core::folder_stream::unique_destination_path(
                            fs_dir.join(&rel_norm),
                        );
                        debug!(
                            %rel_path,
                            size,
                            target = %target.display(),
                            "upload: file begin"
                        );
                        if let Some(parent) = target.parent() {
                            tokio::fs::create_dir_all(parent)
                                .await
                                .map_err(|e| format!("mkdir failed: {e}"))?;
                        }
                        let file = tokio::fs::OpenOptions::new()
                            .write(true)
                            .create_new(true)
                            .open(&target)
                            .await
                            .map_err(|e| format!("create failed: {e}"))?;
                        state.current = Some((file, size));
                    }
                    FolderFrame::Chunk(data) => {
                        if let Some((file, remaining)) = state.current.as_mut() {
                            file.write_all(&data)
                                .await
                                .map_err(|e| format!("write failed: {e}"))?;
                            *remaining = remaining.saturating_sub(data.len() as u64);
                            state.bytes_done += data.len() as u64;
                        }
                    }
                    FolderFrame::FileEnd { .. } => {
                        state.files_done += 1;
                        state.current = None;
                    }
                    FolderFrame::End => {
                        // Stream complete; remaining frames are ignored.
                    }
                    FolderFrame::Abort { reason } => {
                        return Err(format!("cancelled: {reason}"));
                    }
                    FolderFrame::Skip { .. } => {}
                }
            }
            Ok(())
        }

        let mut write_state = FolderWriteState {
            decoder: FolderFrameDecoder::new(),
            current: None,
            files_done: 0,
            bytes_done: 0,
        };
        let outcome: Result<(), String> = match &body {
            ReceivedBody::Empty => Ok(()),
            ReceivedBody::Memory(bytes) => consume_chunk(&mut write_state, &fs_dir, bytes).await,
            ReceivedBody::TempFile(spool) => {
                // Stream-parse from the spool file instead of buffering it.
                let mut file = match tokio::fs::File::open(spool.path()).await {
                    Ok(file) => file,
                    Err(error) => return Err(SessionError::FileSystemError(error.to_string())),
                };
                let mut buf = vec![0u8; 256 * 1024];
                loop {
                    let n = match file.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(error) => return Err(SessionError::FileSystemError(error.to_string())),
                    };
                    if let Err(error) = consume_chunk(&mut write_state, &fs_dir, &buf[..n]).await {
                        return finish(write_state.files_done, write_state.bytes_done, Some(error));
                    }
                }
                Ok(())
            }
        };

        match outcome {
            Ok(()) => {
                info!(
                    %task_id,
                    files = write_state.files_done,
                    bytes = write_state.bytes_done,
                    "upload: completed"
                );
                finish(write_state.files_done, write_state.bytes_done, None)
            }
            Err(error) => {
                warn!(
                    %task_id,
                    %error,
                    files = write_state.files_done,
                    bytes = write_state.bytes_done,
                    "upload: failed"
                );
                finish(write_state.files_done, write_state.bytes_done, Some(error))
            }
        }
    }

    /// Create an incremental writer for an `UploadStart` body: the
    /// request connection feeds body chunks as they arrive (`push`) and
    /// the uploaded items land under the resolved destination immediately,
    /// instead of spooling the whole body first (`upload_body`). Startup
    /// failures are deferred into the sink so the caller keeps draining the
    /// body and reports the error at `finish`.
    pub async fn upload_sink(
        &self,
        session_id: SessionId,
        task_id: String,
        dst_dir: String,
    ) -> FolderUploadSink {
        let start = async {
            let session = self
                .get_session(session_id)
                .await
                .ok_or_else(|| "Session not found".to_string())?;
            let base_path = {
                let session = session.lock().await;
                session
                    .viewing_path
                    .clone()
                    .or_else(|| session.current_working_path.clone())
            };
            let fs_dir =
                resolve_filesystem_path(base_path.as_deref(), &dst_dir, default_terminal_os())
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "Invalid destination path".to_string())?;
            tokio::fs::create_dir_all(&fs_dir)
                .await
                .map_err(|error| format!("Failed to create destination: {error}"))?;
            Ok::<PathBuf, String>(fs_dir)
        };
        let (fs_dir, failed) = match start.await {
            Ok(fs_dir) => {
                info!(
                    %task_id,
                    ?session_id,
                    fs_dir = %fs_dir.display(),
                    "upload: streaming started"
                );
                (fs_dir, None)
            }
            Err(error) => {
                warn!(%task_id, ?session_id, %error, "upload: cannot start streaming");
                (PathBuf::new(), Some(error))
            }
        };
        FolderUploadSink {
            task_id,
            session_id,
            fs_dir,
            decoder: FolderFrameDecoder::new(),
            current: None,
            files_done: 0,
            bytes_done: 0,
            failed,
        }
    }

    /// Stream the requested files/folders back to the client as one framed
    /// entry stream (one transfer task). Every requested path is resolved
    /// against the session's base path; successfully resolved paths become
    /// roots of the lazy walk (files stream as single entries, directories
    /// recursively, each under its own name). Paths that fail resolution
    /// produce Skip frames chained in front of the walk stream. If every
    /// path fails, the reply is an error `DownloadResult` with no body.
    pub async fn download_transport(
        &self,
        session_id: SessionId,
        task_id: String,
        paths: Vec<String>,
    ) -> Result<EncodedMessage, SessionError> {
        info!(%task_id, ?session_id, ?paths, "download: request received");
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;
        let base_path = {
            let session = session.lock().await;
            session
                .viewing_path
                .clone()
                .or_else(|| session.current_working_path.clone())
        };

        let finish_error = |error: String| {
            warn!(%task_id, %error, "download: failed before streaming");
            encode_message(Message::DownloadResult {
                task_id: task_id.clone(),
                session_id,
                files: 0,
                bytes: 0,
                error: Some(error),
            })
            .map_err(|error| SessionError::FileSystemError(error.to_string()))
        };

        let mut roots: Vec<PathBuf> = Vec::new();
        let mut skip_writer = FolderFrameWriter::new();
        for path in &paths {
            let resolved =
                match resolve_filesystem_path(base_path.as_deref(), path, default_terminal_os())
                    .map_err(SessionError::FileSystemError)?
                {
                    Some(path) => path,
                    None => {
                        let name = Path::new(path)
                            .file_name()
                            .map(|name| name.to_string_lossy().into_owned())
                            .unwrap_or_else(|| path.clone());
                        skip_writer.skip(&name, "invalid path");
                        continue;
                    }
                };
            roots.push(resolved);
        }
        if roots.is_empty() {
            return finish_error("Invalid path".to_string());
        }

        let abort = Arc::new(AtomicBool::new(false));
        let state = Arc::new(std::sync::Mutex::new(FolderWalkState::default()));
        let reader = FolderStreamReader::new(roots, abort, state);
        let skip_bytes = skip_writer.take();
        let combined = std::io::Cursor::new(skip_bytes).chain(reader);
        info!(%task_id, "download: streaming items");
        let mut encoded = encode_message(Message::DownloadResult {
            task_id,
            session_id,
            files: 0,
            bytes: 0,
            error: None,
        })
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        encoded.body = OutboundBody::Reader {
            len: None,
            reader: Box::pin(combined),
        };
        Ok(encoded)
    }

    pub async fn create_path(
        &self,
        session_id: SessionId,
        path: String,
        is_dir: bool,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let base_path = {
            let session = session.lock().await;
            session
                .viewing_path
                .clone()
                .or_else(|| session.current_working_path.clone())
        };

        let runtime_os = default_terminal_os();
        let path_clone = path.clone();
        let result = tokio::task::spawn_blocking(move || {
            fs_create_path(CreatePathRequest {
                base_path,
                path: path_clone,
                is_dir,
                runtime_os,
            })
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()));

        match result {
            Ok(Ok(_target)) => Ok(Message::CreatePathResult {
                session_id,
                path,
                error: None,
            }),
            Ok(Err(error)) => Ok(Message::CreatePathResult {
                session_id,
                path,
                error: Some(error.to_string()),
            }),
            Err(error) => Ok(Message::CreatePathResult {
                session_id,
                path,
                error: Some(error.to_string()),
            }),
        }
    }

    pub async fn delete_paths(
        &self,
        session_id: SessionId,
        paths: Vec<String>,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let base_path = {
            let session = session.lock().await;
            session
                .viewing_path
                .clone()
                .or_else(|| session.current_working_path.clone())
        };

        let runtime_os = default_terminal_os();
        let result = tokio::task::spawn_blocking(move || {
            fs_delete_paths(DeletePathsRequest {
                base_path,
                paths,
                runtime_os,
            })
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()));

        match result {
            Ok(Ok(())) => Ok(Message::DeletePathsResult {
                session_id,
                error: None,
            }),
            Ok(Err(error)) => Ok(Message::DeletePathsResult {
                session_id,
                error: Some(error.to_string()),
            }),
            Err(error) => Ok(Message::DeletePathsResult {
                session_id,
                error: Some(error.to_string()),
            }),
        }
    }

    pub async fn paste_paths(
        &self,
        session_id: SessionId,
        src_paths: Vec<String>,
        dst_dir: String,
        is_cut: bool,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let base_path = {
            let session = session.lock().await;
            session
                .viewing_path
                .clone()
                .or_else(|| session.current_working_path.clone())
        };

        let runtime_os = default_terminal_os();
        let result = tokio::task::spawn_blocking(move || {
            fs_paste_paths(PastePathsRequest {
                base_path,
                src_paths,
                dst_dir,
                is_cut,
                runtime_os,
            })
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()));

        match result {
            Ok(Ok(())) => Ok(Message::PastePathsResult {
                session_id,
                error: None,
            }),
            Ok(Err(error)) => Ok(Message::PastePathsResult {
                session_id,
                error: Some(error.to_string()),
            }),
            Err(error) => Ok(Message::PastePathsResult {
                session_id,
                error: Some(error.to_string()),
            }),
        }
    }

    pub async fn rename_path(
        &self,
        session_id: SessionId,
        old_path: String,
        new_path: String,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let base_path = {
            let session = session.lock().await;
            session
                .viewing_path
                .clone()
                .or_else(|| session.current_working_path.clone())
        };

        let runtime_os = default_terminal_os();
        let result = tokio::task::spawn_blocking(move || {
            fs_rename_path(RenamePathRequest {
                base_path,
                old_path,
                new_path,
                runtime_os,
            })
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()));

        match result {
            Ok(Ok(())) => Ok(Message::RenamePathResult {
                session_id,
                error: None,
            }),
            Ok(Err(error)) => Ok(Message::RenamePathResult {
                session_id,
                error: Some(error.to_string()),
            }),
            Err(error) => Ok(Message::RenamePathResult {
                session_id,
                error: Some(error.to_string()),
            }),
        }
    }

    pub async fn list_path_content(
        &self,
        session_id: SessionId,
        path: String,
    ) -> Result<(Message, Option<PendingPathAnnotation>), SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let (base_path, requested_path, viewing_path, current_working_path, listing_generation) = {
            let mut session = session.lock().await;
            session.path_listing_generation = session.path_listing_generation.wrapping_add(1);
            let base_path = session
                .viewing_path
                .clone()
                .or_else(|| session.current_working_path.clone());
            (
                base_path,
                path.trim().to_string(),
                session.viewing_path.clone(),
                session.current_working_path.clone(),
                session.path_listing_generation,
            )
        };

        let runtime_os = default_terminal_os();
        info!(
            %session_id,
            ?requested_path,
            ?viewing_path,
            ?current_working_path,
            selected_base_path = ?base_path,
            ?runtime_os,
            listing_generation,
            "[explorer] server resolving path listing"
        );
        let listing_target =
            resolve_listing_target(base_path.as_deref(), &requested_path, runtime_os)
                .map_err(SessionError::FileSystemError)?;
        let listing_path = listing_target.real_path.clone();
        let items = tokio::task::spawn_blocking({
            let listing_target = listing_target.clone();
            move || read_path_content_fast(&listing_target)
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))??;
        info!(
            %session_id,
            ?requested_path,
            listing_path = ?listing_path,
            filesystem_path = ?listing_target.filesystem_path,
            items = items.len(),
            listing_generation,
            "[explorer] server returning path listing"
        );

        {
            let mut session = session.lock().await;
            if session.path_listing_generation == listing_generation {
                session.viewing_path = Some(listing_path.clone());
            }
        }

        let annotation =
            (!items.is_empty() && listing_target.filesystem_path.is_some()).then(|| {
                PendingPathAnnotation {
                    session_id,
                    path: listing_path.clone(),
                    target: listing_target,
                    listing_generation,
                }
            });

        Ok((
            Message::PathContentList {
                session_id,
                path: listing_path,
                items,
                listing_generation,
            },
            annotation,
        ))
    }

    /// Finish the Git portion of an interactive directory listing. The
    /// directory is sampled again after the Git subprocesses finish so a slow
    /// annotation can never replace a newer file set with its stale snapshot.
    pub(crate) async fn finish_path_content_annotation(
        &self,
        pending: PendingPathAnnotation,
    ) -> Result<Option<Message>, SessionError> {
        let PendingPathAnnotation {
            session_id,
            path,
            target,
            listing_generation,
        } = pending;
        let annotated = tokio::task::spawn_blocking({
            let target = target.clone();
            move || {
                let mut items = read_path_content_fast(&target)?;
                if !annotate_path_content(&target, &mut items) {
                    return Ok(None);
                }
                let current = read_path_content_fast(&target)?;
                let unchanged = current.len() == items.len()
                    && current.iter().zip(&items).all(|(left, right)| {
                        left.name == right.name && left.is_dir == right.is_dir
                    });
                Ok(unchanged.then_some(items))
            }
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))??;

        let Some(items) = annotated else {
            return Ok(None);
        };
        let Some(session) = self.get_session(session_id).await else {
            return Ok(None);
        };
        if session.lock().await.viewing_path.as_deref() != Some(path.as_str()) {
            return Ok(None);
        }

        Ok(Some(Message::PathContentList {
            session_id,
            path,
            items,
            listing_generation,
        }))
    }

    pub async fn list_watched_path_content(
        &self,
        session_id: SessionId,
        path: String,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let (base_path, listing_generation) = {
            let mut session = session.lock().await;
            session.path_listing_generation = session.path_listing_generation.wrapping_add(1);
            (
                session
                    .viewing_path
                    .clone()
                    .or_else(|| session.current_working_path.clone()),
                session.path_listing_generation,
            )
        };

        let requested_path = path.trim().to_string();
        let listing_target = super::path_listing::resolve_existing_listing_target(
            base_path.as_deref(),
            &requested_path,
            default_terminal_os(),
        )
        .map_err(SessionError::FileSystemError)?;
        let Some(listing_target) = listing_target else {
            // A deleted watched directory must not fall back to the filesystem
            // root; report an empty listing at its original path instead.
            return Ok(Message::PathContentList {
                session_id,
                path: requested_path,
                items: Vec::new(),
                listing_generation,
            });
        };
        let listing_path = listing_target.real_path.clone();
        let items = tokio::task::spawn_blocking({
            let listing_target = listing_target.clone();
            move || read_path_content(listing_target)
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))??;

        Ok(Message::PathContentList {
            session_id,
            path: listing_path,
            items,
            listing_generation,
        })
    }
    pub async fn spawn_find_files(
        &self,
        session_id: SessionId,
        request_id: String,
        root_path: String,
        query: String,
        include: String,
        exclude: String,
        case_sensitive: bool,
        whole_word: bool,
        use_regex: bool,
        max_results: usize,
        tx: ConnectionTx,
    ) -> Result<(), SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let base_path = if root_path.is_empty() {
            let session = session.lock().await;
            session
                .viewing_path
                .clone()
                .or_else(|| session.current_working_path.clone())
        } else {
            Some(root_path)
        };
        let base_path = base_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;

        let runtime_os = default_terminal_os();
        let root = resolve_filesystem_path(None, &base_path, runtime_os)
            .map_err(SessionError::FileSystemError)?
            .ok_or_else(|| SessionError::FileSystemError("Invalid search path".to_string()))?;

        info!(
            "FindFiles starting session={} request_id={} query_bytes={} include='{}' exclude='{}' case_sensitive={} whole_word={} regex={} max_results={} root={}",
            session_id,
            request_id,
            query.len(),
            include,
            exclude,
            case_sensitive,
            whole_word,
            use_regex,
            max_results,
            root.display()
        );

        let cancellation = Arc::new(AtomicBool::new(false));
        {
            let mut cancellations = self.file_search_cancellations.lock().await;
            if let Some(previous) = cancellations.insert(session_id, cancellation.clone()) {
                info!(
                    "FindFiles cancelling previous search for session={} before request_id={}",
                    session_id, request_id
                );
                previous.store(true, Ordering::SeqCst);
            }
        }

        let cancellations = self.file_search_cancellations.clone();
        let search_query = query.clone();
        let runtime_handle = tokio::runtime::Handle::current();
        tokio::task::spawn_blocking(move || {
            let mut chunk_index = 0u32;
            let mut chunk = Vec::new();

            let search_result = find_files_stream(
                FindFilesOptions {
                    root,
                    query: search_query,
                    include,
                    exclude,
                    case_sensitive,
                    whole_word,
                    use_regex,
                    max_results,
                },
                |result| {
                    chunk.push(result);
                    if chunk.len() >= FIND_FILES_RESULT_CHUNK_FILES {
                        let results = std::mem::take(&mut chunk);
                        send_find_files_result_chunk(
                            &tx,
                            session_id,
                            &request_id,
                            &query,
                            &mut chunk_index,
                            results,
                            false,
                            None,
                        );
                    }
                    Ok(())
                },
                || cancellation.load(Ordering::SeqCst),
            );

            match search_result {
                Ok(completed) => {
                    let results = std::mem::take(&mut chunk);
                    info!(
                        "FindFiles completed scan session={} request_id={} completed={} final_files={}",
                        session_id,
                        request_id,
                        completed,
                        results.len()
                    );
                    send_find_files_result_chunk(
                        &tx,
                        session_id,
                        &request_id,
                        &query,
                        &mut chunk_index,
                        results,
                        true,
                        if completed {
                            None
                        } else {
                            Some("Search cancelled".to_string())
                        },
                    );
                }
                Err(error) => {
                    warn!(
                        "FindFiles failed session={} request_id={} error={}",
                        session_id, request_id, error
                    );
                    send_find_files_result_chunk(
                        &tx,
                        session_id,
                        &request_id,
                        &query,
                        &mut chunk_index,
                        Vec::new(),
                        true,
                        Some(error),
                    );
                }
            }

            let cancellations = cancellations.clone();
            runtime_handle.spawn(async move {
                let mut cancellations = cancellations.lock().await;
                if cancellations
                    .get(&session_id)
                    .is_some_and(|current| Arc::ptr_eq(current, &cancellation))
                {
                    cancellations.remove(&session_id);
                }
            });
        });

        Ok(())
    }

    /// Spawn a recursive *filename* search (case-insensitive substring) under
    /// `root_path`. Unlike `spawn_find_files` (which greps file contents), this
    /// never reads file bodies and matches only entry names. Results are sent
    /// as a single chunk followed by a final `is_final` marker. A new search
    /// for the same session cancels any in-flight one (shared cancellation map).
    pub async fn spawn_search_path_names(
        &self,
        session_id: SessionId,
        request_id: String,
        root_path: String,
        query: String,
        max_results: usize,
        tx: ConnectionTx,
    ) -> Result<(), SessionError> {
        let runtime_os = default_terminal_os();
        let root = resolve_filesystem_path(None, &root_path, runtime_os)
            .map_err(SessionError::FileSystemError)?
            .ok_or_else(|| SessionError::FileSystemError("Invalid search path".to_string()))?;

        info!(
            "SearchPathNames starting session={} request_id={} query_bytes={} max_results={} root={}",
            session_id,
            request_id,
            query.len(),
            max_results,
            root.display()
        );

        let cancellation = Arc::new(AtomicBool::new(false));
        {
            let mut cancellations = self.file_search_cancellations.lock().await;
            if let Some(previous) = cancellations.insert(session_id, cancellation.clone()) {
                info!(
                    "SearchPathNames cancelling previous search for session={} before request_id={}",
                    session_id, request_id
                );
                previous.store(true, Ordering::SeqCst);
            }
        }

        let cancellations = self.file_search_cancellations.clone();
        let search_query = query.clone();
        let runtime_handle = tokio::runtime::Handle::current();
        tokio::task::spawn_blocking(move || {
            let mut results: Vec<PathNameMatch> = Vec::new();
            let search_result = search_path_names_stream(
                SearchPathNamesOptions {
                    root,
                    query: search_query.clone(),
                    max_results,
                },
                |m| {
                    results.push(m);
                    Ok(())
                },
                || cancellation.load(Ordering::SeqCst),
            );

            match search_result {
                Ok(completed) => {
                    info!(
                        "SearchPathNames completed scan session={} request_id={} completed={} matches={}",
                        session_id,
                        request_id,
                        completed,
                        results.len()
                    );
                    let error = if completed {
                        None
                    } else {
                        Some("Search cancelled".to_string())
                    };
                    send_search_path_names_result(
                        &tx,
                        session_id,
                        &request_id,
                        &root_path,
                        &query,
                        results,
                        true,
                        error,
                    );
                }
                Err(error) => {
                    warn!(
                        "SearchPathNames failed session={} request_id={} error={}",
                        session_id, request_id, error
                    );
                    send_search_path_names_result(
                        &tx,
                        session_id,
                        &request_id,
                        &root_path,
                        &query,
                        Vec::new(),
                        true,
                        Some(error),
                    );
                }
            }

            let cancellations = cancellations.clone();
            runtime_handle.spawn(async move {
                let mut cancellations = cancellations.lock().await;
                if cancellations
                    .get(&session_id)
                    .is_some_and(|current| Arc::ptr_eq(current, &cancellation))
                {
                    cancellations.remove(&session_id);
                }
            });
        });

        Ok(())
    }

    pub async fn git_changed_files(
        &self,
        session_id: SessionId,
        commit: Option<String>,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = {
            let session = session.lock().await;
            session.git_working_path()
        };

        let working_path = working_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;

        let response_commit = commit.clone();
        //         info!(
        //             %session_id,
        //             commit = ?response_commit,
        //             working_path = ?working_path,
        //             "[git] server resolving changed-files working path"
        //         );
        let result = tokio::task::spawn_blocking(move || {
            let repo_root = super::git::git_repo_root(&working_path)?;
            let items = super::git::git_read_changed_files(&working_path, commit.as_deref())?;
            Ok::<_, SessionError>((repo_root, items))
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

        let (repo_root, items) = match result {
            Ok((repo_root, items)) => (Some(repo_root), items),
            Err(SessionError::FileSystemError(message)) if message == "Not a git repository" => {
                (None, Vec::new())
            }
            Err(error) => return Err(error),
        };
        //         info!(
        //             %session_id,
        //             commit = ?response_commit,
        //             repo_root = ?repo_root,
        //             items = items.len(),
        //             "[git] server returning changed files"
        //         );
        session.lock().await.git_repo_root.clone_from(&repo_root);

        Ok(Message::GitChangedFilesList {
            session_id,
            commit: response_commit,
            repo_root,
            items,
        })
    }

    pub async fn git_log(
        &self,
        session_id: SessionId,
        path: Option<String>,
        commit: Option<String>,
        branch: Option<String>,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = {
            let session = session.lock().await;
            session.git_working_path()
        };

        let working_path = working_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;

        let response_commit = commit.clone();
        let response_branch = branch.clone();
        let result = tokio::task::spawn_blocking(move || {
            let repo_root = super::git::git_repo_root(&working_path)?;
            let commits = super::git::git_read_log(
                &working_path,
                path.as_deref(),
                commit.as_deref(),
                branch.as_deref(),
            )?;
            Ok::<_, SessionError>((repo_root, commits))
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

        let (repo_root, commits) = match result {
            Ok((repo_root, commits)) => (Some(repo_root), commits),
            Err(SessionError::FileSystemError(message)) if message == "Not a git repository" => {
                (None, Vec::new())
            }
            Err(error) => return Err(error),
        };

        Ok(Message::GitLogList {
            session_id,
            commit: response_commit,
            branch: response_branch,
            repo_root,
            commits,
        })
    }

    pub async fn git_diff(
        &self,
        session_id: SessionId,
        path: String,
        commit: Option<String>,
        cached: bool,
        compare_with_worktree: bool,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = {
            let session = session.lock().await;
            session.git_working_path()
        };

        let working_path = working_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;

        let diff = tokio::task::spawn_blocking({
            let path = path.clone();
            move || {
                super::git::git_read_diff(
                    &working_path,
                    &path,
                    commit.as_deref(),
                    cached,
                    compare_with_worktree,
                )
            }
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))??;

        Ok(Message::GitDiffResult {
            session_id,
            path,
            diff,
        })
    }

    pub async fn git_stage(
        &self,
        session_id: SessionId,
        paths: Vec<String>,
        repo_root: Option<String>,
    ) -> Result<Message, SessionError> {
        if paths.is_empty() {
            return Ok(Message::GitStageResult {
                session_id,
                error: None,
            });
        }
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        // A client-pinned repo root (from the changed-files list being
        // displayed) wins over the session's live working path; the working
        // path is only needed when the pin is absent.
        let working_path = if repo_root.is_some() {
            String::new()
        } else {
            let session = session.lock().await;
            session.git_mutation_working_path().ok_or_else(|| {
                SessionError::FileSystemError("No working path available".to_string())
            })?
        };

        let error = tokio::task::spawn_blocking(move || {
            super::git::git_do_stage(&working_path, &paths, repo_root.as_deref()).err()
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?
        .map(|e| e.to_string());

        Ok(Message::GitStageResult { session_id, error })
    }

    pub async fn git_stash_list(&self, session_id: SessionId) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = {
            let session = session.lock().await;
            session.git_working_path()
        };

        let working_path = working_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;

        let result =
            tokio::task::spawn_blocking(move || super::git::git_read_stash_list(&working_path))
                .await
                .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

        let (repo_root, stashes) = match result {
            Ok((repo_root, stashes)) => (Some(repo_root), stashes),
            Err(SessionError::FileSystemError(message)) if message == "Not a git repository" => {
                (None, Vec::new())
            }
            Err(error) => return Err(error),
        };

        Ok(Message::GitStashListResult {
            session_id,
            repo_root,
            stashes,
        })
    }

    pub async fn git_stash_push(
        &self,
        session_id: SessionId,
        message: Option<String>,
        paths: Vec<String>,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = {
            let session = session.lock().await;
            session.git_mutation_working_path()
        };
        let working_path = working_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;

        let error = tokio::task::spawn_blocking(move || {
            super::git::git_do_stash_push(&working_path, message.as_deref(), &paths).err()
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?
        .map(|e| e.to_string());

        Ok(Message::GitStashResult { session_id, error })
    }

    pub async fn git_stash_apply(
        &self,
        session_id: SessionId,
        index: u32,
    ) -> Result<Message, SessionError> {
        self.git_stash_ref_op(session_id, "apply", index).await
    }

    pub async fn git_stash_pop(
        &self,
        session_id: SessionId,
        index: u32,
    ) -> Result<Message, SessionError> {
        self.git_stash_ref_op(session_id, "pop", index).await
    }

    pub async fn git_stash_drop(
        &self,
        session_id: SessionId,
        index: u32,
    ) -> Result<Message, SessionError> {
        self.git_stash_ref_op(session_id, "drop", index).await
    }

    async fn git_stash_ref_op(
        &self,
        session_id: SessionId,
        operation: &'static str,
        index: u32,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = {
            let session = session.lock().await;
            session.git_mutation_working_path()
        };
        let working_path = working_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;

        let error = tokio::task::spawn_blocking(move || {
            let result = match operation {
                "apply" => super::git::git_do_stash_apply(&working_path, index),
                "pop" => super::git::git_do_stash_pop(&working_path, index),
                _ => super::git::git_do_stash_drop(&working_path, index),
            };
            result.err()
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?
        .map(|e| e.to_string());

        Ok(Message::GitStashResult { session_id, error })
    }

    pub async fn git_submodule_list(&self, session_id: SessionId) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = {
            let session = session.lock().await;
            session.git_working_path()
        };

        let working_path = working_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;

        let result =
            tokio::task::spawn_blocking(move || super::git::git_read_submodule_list(&working_path))
                .await
                .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

        let (repo_root, submodules) = match result {
            Ok((repo_root, submodules)) => (Some(repo_root), submodules),
            Err(SessionError::FileSystemError(message)) if message == "Not a git repository" => {
                (None, Vec::new())
            }
            Err(error) => return Err(error),
        };

        Ok(Message::GitSubmoduleListResult {
            session_id,
            repo_root,
            submodules,
        })
    }

    pub async fn git_unstage(
        &self,
        session_id: SessionId,
        paths: Vec<String>,
        repo_root: Option<String>,
    ) -> Result<Message, SessionError> {
        if paths.is_empty() {
            return Ok(Message::GitUnstageResult {
                session_id,
                error: None,
            });
        }
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = if repo_root.is_some() {
            String::new()
        } else {
            let session = session.lock().await;
            session.git_mutation_working_path().ok_or_else(|| {
                SessionError::FileSystemError("No working path available".to_string())
            })?
        };

        let error = tokio::task::spawn_blocking(move || {
            super::git::git_do_unstage(&working_path, &paths, repo_root.as_deref()).err()
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?
        .map(|e| e.to_string());

        Ok(Message::GitUnstageResult { session_id, error })
    }

    pub async fn git_discard(
        &self,
        session_id: SessionId,
        paths: Vec<String>,
        repo_root: Option<String>,
    ) -> Result<Message, SessionError> {
        if paths.is_empty() {
            return Ok(Message::GitDiscardResult {
                session_id,
                error: None,
            });
        }
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = if repo_root.is_some() {
            String::new()
        } else {
            let session = session.lock().await;
            session.git_mutation_working_path().ok_or_else(|| {
                SessionError::FileSystemError("No working path available".to_string())
            })?
        };

        let error = tokio::task::spawn_blocking(move || {
            super::git::git_do_discard(&working_path, &paths, repo_root.as_deref()).err()
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?
        .map(|e| e.to_string());

        Ok(Message::GitDiscardResult { session_id, error })
    }

    pub async fn git_revert(
        &self,
        session_id: SessionId,
        path: String,
        commit: String,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = {
            let session = session.lock().await;
            session.git_mutation_working_path()
        };
        let working_path = working_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;

        let error = tokio::task::spawn_blocking({
            let path = path.clone();
            let commit = commit.clone();
            move || super::git::git_do_revert_file(&working_path, &path, &commit).err()
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?
        .map(|e| e.to_string());

        Ok(Message::GitRevertResult {
            session_id,
            path,
            error,
        })
    }

    pub async fn git_commit(
        &self,
        session_id: SessionId,
        message: String,
        expected_detached_head: Option<String>,
        target_branch: Option<String>,
        create_branch: bool,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = {
            let session = session.lock().await;
            session.git_mutation_working_path()
        };
        let working_path = working_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;

        let error = tokio::task::spawn_blocking(move || {
            match expected_detached_head {
                Some(expected_head) => super::git::git_do_detached_commit(
                    &working_path,
                    &message,
                    &expected_head,
                    target_branch.as_deref(),
                    create_branch,
                ),
                None => super::git::git_do_commit(&working_path, &message),
            }
            .err()
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?
        .map(|e| e.to_string());

        Ok(Message::GitCommitResult { session_id, error })
    }

    pub async fn git_fetch(&self, session_id: SessionId) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = {
            let session = session.lock().await;
            session.git_mutation_working_path()
        };
        let working_path = working_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;

        let error =
            tokio::task::spawn_blocking(move || super::git::git_do_fetch(&working_path).err())
                .await
                .map_err(|error| SessionError::FileSystemError(error.to_string()))?
                .map(|e| e.to_string());

        Ok(Message::GitFetchResult { session_id, error })
    }

    pub async fn git_pull(&self, session_id: SessionId) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = {
            let session = session.lock().await;
            session.git_mutation_working_path()
        };
        let working_path = working_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;

        let error =
            tokio::task::spawn_blocking(move || super::git::git_do_pull(&working_path).err())
                .await
                .map_err(|error| SessionError::FileSystemError(error.to_string()))?
                .map(|e| e.to_string());

        Ok(Message::GitPullResult { session_id, error })
    }

    pub async fn git_push(
        &self,
        session_id: SessionId,
        push_all: bool,
        force: bool,
        force_with_lease: bool,
        set_upstream: bool,
        tags: bool,
        dry_run: bool,
        remote: String,
        branch: String,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = {
            let session = session.lock().await;
            session.git_mutation_working_path()
        };
        let working_path = working_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;

        let error = tokio::task::spawn_blocking(move || {
            super::git::git_do_push(
                &working_path,
                push_all,
                force,
                force_with_lease,
                set_upstream,
                tags,
                dry_run,
                &remote,
                &branch,
            )
            .err()
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?
        .map(|e| e.to_string());

        Ok(Message::GitPushResult { session_id, error })
    }

    pub async fn git_branches(&self, session_id: SessionId) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = {
            let session = session.lock().await;
            session.git_working_path()
        };

        let working_path = working_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;

        let result = tokio::task::spawn_blocking(move || {
            let (current, detached_head, branches) = super::git::git_read_branches(&working_path)?;
            let remote_hash = super::git::git_read_remote_tracking_hash(&working_path, &current)?;
            let tags = super::git::git_read_tags(&working_path)?;
            Ok::<_, SessionError>((current, detached_head, branches, remote_hash, tags))
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

        let (current, detached_head, branches, remote_tracking_hash, tags) = match result {
            Ok(values) => values,
            Err(SessionError::FileSystemError(message)) if message == "Not a git repository" => {
                (None, None, Vec::new(), None, Vec::new())
            }
            Err(error) => return Err(error),
        };

        Ok(Message::GitBranchesResult {
            session_id,
            current,
            detached_head,
            branches,
            remote_tracking_hash,
            tags,
        })
    }

    pub async fn git_checkout(
        &self,
        session_id: SessionId,
        ref_: String,
        create_branch: bool,
        branch_name: String,
        track: bool,
        force: bool,
        merge: bool,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = {
            let session = session.lock().await;
            session.git_mutation_working_path()
        };
        let working_path = working_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;

        let error = tokio::task::spawn_blocking(move || {
            super::git::git_do_checkout(
                &working_path,
                &ref_,
                create_branch,
                &branch_name,
                track,
                force,
                merge,
            )
            .err()
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?
        .map(|e| e.to_string());

        Ok(Message::GitCheckoutResult { session_id, error })
    }

    pub async fn git_delete_branch(
        &self,
        session_id: SessionId,
        branch: String,
        is_worktree: bool,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = {
            let session = session.lock().await;
            session.git_mutation_working_path()
        };
        let working_path = working_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;

        let error = tokio::task::spawn_blocking(move || {
            super::git::git_do_delete_branch(&working_path, &branch, is_worktree).err()
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?
        .map(|e| e.to_string());

        Ok(Message::GitDeleteBranchResult { session_id, error })
    }

    pub async fn git_create_branch(
        &self,
        session_id: SessionId,
        source_ref: String,
        branch: String,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = {
            let session = session.lock().await;
            session.git_mutation_working_path()
        };
        let working_path = working_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;

        let error = tokio::task::spawn_blocking(move || {
            super::git::git_do_create_branch(&working_path, &source_ref, &branch).err()
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?
        .map(|e| e.to_string());

        Ok(Message::GitCreateBranchResult { session_id, error })
    }

    pub async fn git_merge(
        &self,
        session_id: SessionId,
        branch: String,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let working_path = {
            let session = session.lock().await;
            session.git_mutation_working_path()
        };
        let working_path = working_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;

        let error = tokio::task::spawn_blocking(move || {
            super::git::git_do_merge(&working_path, &branch).err()
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?
        .map(|e| e.to_string());

        Ok(Message::GitMergeResult { session_id, error })
    }

    /* ── Interactive rebase ────────────────────────────── */

    async fn rebase_working_path(&self, session_id: SessionId) -> Result<String, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;
        let working_path = {
            let session = session.lock().await;
            session.git_mutation_working_path()
        };
        working_path
            .ok_or_else(|| SessionError::FileSystemError("No working path available".to_string()))
    }

    pub async fn git_rebase_start(
        &self,
        session_id: SessionId,
        target_branch: String,
    ) -> Result<Message, SessionError> {
        let working_path = self.rebase_working_path(session_id).await?;
        let result = tokio::task::spawn_blocking(move || {
            super::git::git_rebase_todo(&working_path, &target_branch)
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        match result {
            Ok((source_branch, commits)) => Ok(Message::GitRebaseStartResult {
                session_id,
                error: None,
                source_branch,
                commits,
            }),
            Err(error) => Ok(Message::GitRebaseStartResult {
                session_id,
                error: Some(error.to_string()),
                source_branch: None,
                commits: vec![],
            }),
        }
    }

    pub async fn git_rebase_apply(
        &self,
        session_id: SessionId,
        target_branch: String,
        commits: Vec<vterm_common::GitRebaseTodoCommit>,
    ) -> Result<Message, SessionError> {
        let working_path = self.rebase_working_path(session_id).await?;
        let _guard = self.git_rebase_mutex.lock().await;
        let result = tokio::task::spawn_blocking(move || {
            super::git::git_rebase_apply(&working_path, &target_branch, &commits)
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        match result {
            Ok(state) => Ok(Message::GitRebaseApplyResult {
                session_id,
                error: None,
                state: Some(state),
            }),
            Err(error) => Ok(Message::GitRebaseApplyResult {
                session_id,
                error: Some(error.to_string()),
                state: None,
            }),
        }
    }

    pub async fn git_rebase_conflict_content(
        &self,
        session_id: SessionId,
        path: String,
    ) -> Result<Message, SessionError> {
        let working_path = self.rebase_working_path(session_id).await?;
        let path_clone = path.clone();
        let result = tokio::task::spawn_blocking(move || {
            super::git::git_rebase_conflict_content(&working_path, &path_clone)
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        match result {
            Ok(content) => Ok(Message::GitRebaseConflictContentResult {
                session_id,
                path,
                base: content.base,
                ours: content.ours,
                theirs: content.theirs,
                error: None,
                ours_commit: content.ours_commit,
                theirs_commit: content.theirs_commit,
            }),
            Err(error) => Ok(Message::GitRebaseConflictContentResult {
                session_id,
                path,
                base: None,
                ours: None,
                theirs: None,
                error: Some(error.to_string()),
                ours_commit: None,
                theirs_commit: None,
            }),
        }
    }

    pub async fn git_rebase_resolve(
        &self,
        session_id: SessionId,
        path: String,
        content: String,
    ) -> Result<Message, SessionError> {
        let working_path = self.rebase_working_path(session_id).await?;
        let _guard = self.git_rebase_mutex.lock().await;
        let path_clone = path.clone();
        let error = tokio::task::spawn_blocking(move || {
            super::git::git_rebase_resolve(&working_path, &path_clone, &content).err()
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?
        .map(|e| e.to_string());
        Ok(Message::GitRebaseResolveResult {
            session_id,
            path,
            error,
        })
    }

    pub async fn git_rebase_continue(
        &self,
        session_id: SessionId,
    ) -> Result<Message, SessionError> {
        let working_path = self.rebase_working_path(session_id).await?;
        let _guard = self.git_rebase_mutex.lock().await;
        let result =
            tokio::task::spawn_blocking(move || super::git::git_rebase_continue(&working_path))
                .await
                .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        match result {
            Ok(state) => Ok(Message::GitRebaseContinueResult {
                session_id,
                error: None,
                state: Some(state),
            }),
            Err(error) => Ok(Message::GitRebaseContinueResult {
                session_id,
                error: Some(error.to_string()),
                state: None,
            }),
        }
    }

    pub async fn git_rebase_skip(&self, session_id: SessionId) -> Result<Message, SessionError> {
        let working_path = self.rebase_working_path(session_id).await?;
        let _guard = self.git_rebase_mutex.lock().await;
        let result =
            tokio::task::spawn_blocking(move || super::git::git_rebase_skip(&working_path))
                .await
                .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        match result {
            Ok(state) => Ok(Message::GitRebaseSkipResult {
                session_id,
                error: None,
                state: Some(state),
            }),
            Err(error) => Ok(Message::GitRebaseSkipResult {
                session_id,
                error: Some(error.to_string()),
                state: None,
            }),
        }
    }

    pub async fn git_rebase_abort(&self, session_id: SessionId) -> Result<Message, SessionError> {
        let working_path = self.rebase_working_path(session_id).await?;
        let _guard = self.git_rebase_mutex.lock().await;
        let error =
            tokio::task::spawn_blocking(move || super::git::git_rebase_abort(&working_path).err())
                .await
                .map_err(|error| SessionError::FileSystemError(error.to_string()))?
                .map(|e| e.to_string());
        Ok(Message::GitRebaseAbortResult { session_id, error })
    }

    pub async fn git_create_worktree(
        &self,
        session_id: SessionId,
        branch: String,
        name: String,
    ) -> Result<Message, SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let (working_path, size) = {
            let s = session.lock().await;
            let path = s.git_working_path();
            (path, s.size)
        };
        let working_path = working_path.ok_or_else(|| {
            SessionError::FileSystemError("No working path available".to_string())
        })?;

        let wp = working_path.clone();
        let br = branch.clone();
        let nm = name.clone();
        let worktree_path = tokio::task::spawn_blocking(move || {
            let result = super::git::git_create_worktree(&wp, &br, &nm);
            if let Err(ref e) = result {
                info!(
                    "[git_worktree] create failed: working_path={}, branch={}, name={}, error={}",
                    wp, br, nm, e
                );
            }
            result
        })
        .await
        .map_err(|error| SessionError::FileSystemError(error.to_string()))??;

        // Create a terminal session in the new worktree directory,
        // named after the worktree so the tab reflects it.
        let new_session_id = self
            .create_session(
                size,
                String::new(),
                None,
                HashMap::new(),
                Some(worktree_path.clone()),
                None,
                None,
                None,
                Some(name.clone()),
                None,
            )
            .await?;

        Ok(Message::GitCreateWorktreeResult {
            session_id,
            worktree_path: Some(worktree_path),
            new_session_id: Some(new_session_id),
            error: None,
        })
    }

    /// Generate a commit message using the agent (vterm-agent with shell tools).
    pub async fn git_generate_commit_message_agent(
        &self,
        session_id: SessionId,
        addon_host: Arc<crate::addon::AddonHost>,
    ) -> Message {
        info!(
            "[git-commit-agent] Starting agent-based commit message generation for session {}",
            session_id
        );

        let session = match self.get_session(session_id.clone()).await {
            Some(s) => s,
            None => {
                warn!("[git-commit-agent] Session {} not found", session_id);
                return Message::GitGenerateCommitMessageResult {
                    session_id,
                    message: None,
                    error: Some("session not found".to_string()),
                };
            }
        };
        let working_path = {
            let s = session.lock().await;
            s.git_working_path()
        };
        drop(session);
        let working_path = match working_path {
            Some(p) => p,
            None => {
                warn!(
                    "[git-commit-agent] No working path for session {}",
                    session_id
                );
                return Message::GitGenerateCommitMessageResult {
                    session_id,
                    message: None,
                    error: Some("no working path".to_string()),
                };
            }
        };

        let config = self.config_manager.snapshot().await;

        let git_result =
            super::git::git_generate_commit_message_with_agent(&working_path, addon_host, &config)
                .await;

        Message::GitGenerateCommitMessageResult {
            session_id,
            message: git_result.message,
            error: git_result.error,
        }
    }

    pub async fn spawn_git_analyze_diff(
        &self,
        session_id: SessionId,
        path: String,
        commit: Option<String>,
        cached: bool,
        compare_with_worktree: bool,
        addon_host: Arc<crate::addon::AddonHost>,
        tx: ConnectionTx,
    ) -> Result<(), SessionError> {
        let working_path = {
            let sessions = self.sessions.read().await;
            let Some(session) = sessions.get(&session_id) else {
                let _ = tx.send(Message::GitAnalyzeDiffResult {
                    session_id,
                    hunks: Vec::new(),
                    error: Some("session not found".to_string()),
                    done: true,
                });
                return Ok(());
            };
            let session = session.lock().await;
            session.git_mutation_working_path()
        };
        let working_path = match working_path {
            Some(p) => p,
            None => {
                let _ = tx.send(Message::GitAnalyzeDiffResult {
                    session_id,
                    hunks: Vec::new(),
                    error: Some("no working path".to_string()),
                    done: true,
                });
                return Ok(());
            }
        };

        // Cancel previous analysis for this session
        let cancellation = Arc::new(AtomicBool::new(false));
        {
            let mut cancellations = self.analysis_cancellations.lock().await;
            if let Some(previous) = cancellations.insert(session_id, cancellation.clone()) {
                info!(
                    "[code-analysis] Cancelling previous analysis for session {}",
                    session_id
                );
                previous.store(true, Ordering::SeqCst);
            }
        }

        let config = self.config_manager.snapshot().await;
        let cancellations_map = self.analysis_cancellations.clone();
        let runtime_handle = tokio::runtime::Handle::current();

        tokio::task::spawn_blocking(move || {
            super::git::git_analyze_hunks_streaming(
                session_id,
                &working_path,
                &path,
                commit.as_deref(),
                cached,
                compare_with_worktree,
                &addon_host,
                &config,
                &cancellation,
                &tx,
            );

            // Clean up cancellation token if still current
            let cancellations_map = cancellations_map.clone();
            runtime_handle.spawn(async move {
                let mut cancellations = cancellations_map.lock().await;
                if cancellations
                    .get(&session_id)
                    .is_some_and(|current| Arc::ptr_eq(current, &cancellation))
                {
                    cancellations.remove(&session_id);
                }
            });
        });

        Ok(())
    }

    /// Start an AI rebase conflict-resolution run for the session. The run
    /// streams `GitRebaseAiResolveResult` events over `tx`; a new run
    /// supersedes (cancels) the session's previous one.
    pub async fn spawn_git_rebase_ai_resolve(
        &self,
        session_id: SessionId,
        run_id: u32,
        instruction: String,
        ours_label: Option<String>,
        theirs_label: Option<String>,
        targets: Vec<vterm_common::GitRebaseAiTarget>,
        addon_host: Arc<crate::addon::AddonHost>,
        tx: ConnectionTx,
    ) -> Result<(), SessionError> {
        {
            let sessions = self.sessions.read().await;
            if !sessions.contains_key(&session_id) {
                let _ = tx.send(Message::GitRebaseAiResolveResult {
                    session_id,
                    run_id,
                    hunk: None,
                    error: Some("session not found".to_string()),
                    done: true,
                });
                return Ok(());
            }
        }

        // A new run cancels the session's previous in-flight run.
        let cancellation = Arc::new(AtomicBool::new(false));
        {
            let mut cancellations = self.rebase_ai_cancellations.lock().await;
            if let Some(previous) = cancellations.insert(session_id, cancellation.clone()) {
                info!(
                    "[rebase-ai] Cancelling previous AI resolve run for session {}",
                    session_id
                );
                previous.store(true, Ordering::SeqCst);
            }
        }

        // The agent reads the repository through its tools, so resolve the
        // repo root up front and fail the run early when it is unavailable.
        let repo_root = match self.rebase_working_path(session_id).await {
            Ok(working_path) => match super::git::git_repo_root(&working_path) {
                Ok(root) => root,
                Err(error) => {
                    let _ = tx.send(Message::GitRebaseAiResolveResult {
                        session_id,
                        run_id,
                        hunk: None,
                        error: Some(error.to_string()),
                        done: true,
                    });
                    return Ok(());
                }
            },
            Err(error) => {
                let _ = tx.send(Message::GitRebaseAiResolveResult {
                    session_id,
                    run_id,
                    hunk: None,
                    error: Some(error.to_string()),
                    done: true,
                });
                return Ok(());
            }
        };

        let config = self.config_manager.snapshot().await;
        let cancellations_map = self.rebase_ai_cancellations.clone();

        tokio::spawn(async move {
            super::git::git_rebase_ai_resolve_streaming(
                session_id,
                run_id,
                &instruction,
                ours_label.as_deref(),
                theirs_label.as_deref(),
                &targets,
                &repo_root,
                addon_host,
                &config,
                &cancellation,
                &tx,
            )
            .await;

            // Clean up cancellation token if still current
            let mut cancellations = cancellations_map.lock().await;
            if cancellations
                .get(&session_id)
                .is_some_and(|current| Arc::ptr_eq(current, &cancellation))
            {
                cancellations.remove(&session_id);
            }
        });

        Ok(())
    }

    /// Stop the session's in-flight AI resolve run (if any). The run ends
    /// after the hunk currently being analyzed and sends its final
    /// `done = true` event itself, so there is no separate response here.
    pub async fn cancel_git_rebase_ai_resolve(&self, session_id: SessionId) {
        let cancellations = self.rebase_ai_cancellations.lock().await;
        if let Some(cancellation) = cancellations.get(&session_id) {
            info!(
                "[rebase-ai] Cancelling AI resolve run for session {}",
                session_id
            );
            cancellation.store(true, Ordering::SeqCst);
        }
    }

    pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>, SessionError> {
        Ok(self.collect_session_info().await)
    }

    /// One `collect_session_info` sweep feeding the session, tab-group and
    /// tab-info lists together. Broadcast paths use this so each broadcast
    /// costs a single O(N) session-lock sweep instead of three.
    pub async fn session_lists_snapshot(&self) -> (Vec<SessionInfo>, Vec<TabGroup>, Vec<TabInfo>) {
        let sessions = self.collect_session_info().await;
        let server_id = self.config_manager.snapshot().await.server.server_id;
        let tabs = self
            .tab_infos_from_sessions(Some(server_id.clone()), &sessions)
            .await;
        let groups = self
            .tab_groups_from_parts(Some(server_id), &sessions, &tabs)
            .await;
        (sessions, groups, tabs)
    }

    pub async fn list_tab_groups(&self) -> Vec<TabGroup> {
        let sessions = self.collect_session_info().await;
        let server_id = self.config_manager.snapshot().await.server.server_id;
        self.tab_groups_from_sessions(Some(server_id), &sessions)
            .await
    }

    pub async fn list_tab_infos(&self) -> Vec<TabInfo> {
        let sessions = self.collect_session_info().await;
        let server_id = self.config_manager.snapshot().await.server.server_id;
        self.tab_infos_from_sessions(Some(server_id), &sessions)
            .await
    }

    pub async fn create_tab_group(
        &self,
        name: String,
        initial_session_id: Option<SessionId>,
    ) -> Result<TabGroupId, SessionError> {
        let group_id = self.allocate_tab_group_id();
        self.tab_groups.write().await.insert(group_id, name);

        if let Some(session_id) = initial_session_id {
            self.set_session_tab_group(session_id, group_id).await?;
        } else {
            self.broadcast_session_list().await?;
        }

        Ok(group_id)
    }

    pub async fn delete_tab_group(&self, group_id: TabGroupId) -> Result<(), SessionError> {
        if group_id == TAB_GROUP_UNGROUPED_ID || group_id == TAB_GROUP_ALL_ID {
            return Ok(());
        }

        self.tab_groups.write().await.remove(&group_id);
        let sessions = self.sessions.read().await;
        let session_arcs: Vec<_> = sessions.values().cloned().collect();
        drop(sessions);

        for session in session_arcs {
            let mut session = session.lock().await;
            if session.tab_group_id == group_id {
                session.tab_group_id = TAB_GROUP_UNGROUPED_ID;
            }
        }

        self.broadcast_session_list().await?;
        Ok(())
    }

    pub async fn rename_tab_group(
        &self,
        group_id: TabGroupId,
        name: String,
    ) -> Result<(), SessionError> {
        if group_id == TAB_GROUP_UNGROUPED_ID || group_id == TAB_GROUP_ALL_ID {
            return Ok(());
        }

        if let Some(group_name) = self.tab_groups.write().await.get_mut(&group_id) {
            *group_name = name;
            self.broadcast_session_list().await?;
        }

        Ok(())
    }

    pub async fn move_terminal_to_group(
        &self,
        session_id: SessionId,
        group_id: TabGroupId,
    ) -> Result<(), SessionError> {
        self.set_session_tab_group(session_id, group_id).await?;
        Ok(())
    }

    pub async fn move_session_to_tab(
        &self,
        session_id: SessionId,
        tab_id: TabId,
    ) -> Result<(), SessionError> {
        if session_id == tab_id {
            return Ok(());
        }

        let sessions = self.sessions.read().await;
        let moving = sessions
            .get(&session_id)
            .cloned()
            .ok_or(SessionError::SessionNotFound(session_id))?;
        let target = sessions
            .get(&tab_id)
            .cloned()
            .ok_or(SessionError::SessionNotFound(tab_id))?;
        drop(sessions);

        let old_tab_id = moving.lock().await.tab_id;
        let target_tab_id = target.lock().await.tab_id;
        let old_group_id = moving.lock().await.group_id;
        let mut target_group_id = target.lock().await.group_id;
        if target_group_id == 0 {
            target_group_id = self.allocate_group_id();
            target.lock().await.group_id = target_group_id;
        }

        {
            let mut moving = moving.lock().await;
            moving.tab_id = target_tab_id;
            moving.group_id = target_group_id;
        }
        if old_tab_id != target_tab_id {
            self.rehome_tab_after_session_removed(old_tab_id).await;
        }
        if old_group_id != 0 && old_group_id != target_group_id {
            self.maybe_reset_last_sibling_group(old_group_id).await;
        }

        self.broadcast_session_list().await?;
        Ok(())
    }

    async fn set_session_tab_group(
        &self,
        session_id: SessionId,
        group_id: TabGroupId,
    ) -> Result<(), SessionError> {
        if group_id == TAB_GROUP_ALL_ID {
            return Ok(());
        }
        if group_id != TAB_GROUP_UNGROUPED_ID
            && !self.tab_groups.read().await.contains_key(&group_id)
        {
            return Ok(());
        }

        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;
        let tab_id = session.lock().await.tab_id;

        let sessions = self.sessions.read().await;
        let session_arcs = sessions.values().cloned().collect::<Vec<_>>();
        drop(sessions);

        for session in session_arcs {
            let mut session = session.lock().await;
            let same_tab = session.tab_id == tab_id;
            if same_tab {
                session.tab_group_id = group_id;
            }
        }
        self.broadcast_session_list().await?;
        Ok(())
    }

    async fn tab_groups_from_sessions(
        &self,
        server_id: Option<String>,
        sessions: &[SessionInfo],
    ) -> Vec<TabGroup> {
        let tabs = self
            .tab_infos_from_sessions(server_id.clone(), sessions)
            .await;
        self.tab_groups_from_parts(server_id, sessions, &tabs).await
    }

    async fn tab_groups_from_parts(
        &self,
        server_id: Option<String>,
        sessions: &[SessionInfo],
        tabs: &[TabInfo],
    ) -> Vec<TabGroup> {
        let all_tab_ids = tabs.iter().map(|tab| tab.tab_id).collect::<Vec<_>>();
        let mut ungrouped_tab_ids = sessions
            .iter()
            .filter(|session| session.tab_group_id == TAB_GROUP_UNGROUPED_ID)
            .map(|session| session.tab_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        ungrouped_tab_ids.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));

        let mut groups = vec![
            TabGroup {
                server_id: server_id.clone(),
                id: TAB_GROUP_ALL_ID,
                name: "All".to_string(),
                builtin: true,
                tab_ids: all_tab_ids,
            },
            TabGroup {
                server_id: server_id.clone(),
                id: TAB_GROUP_UNGROUPED_ID,
                name: "Ungrouped".to_string(),
                builtin: true,
                tab_ids: ungrouped_tab_ids,
            },
        ];

        let group_names = self.tab_groups.read().await;
        let mut custom_groups = group_names
            .iter()
            .map(|(id, name)| {
                let mut tab_ids = sessions
                    .iter()
                    .filter(|session| session.tab_group_id == *id)
                    .map(|session| session.tab_id)
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>();
                tab_ids.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));

                TabGroup {
                    server_id: server_id.clone(),
                    id: *id,
                    name: name.clone(),
                    builtin: false,
                    tab_ids,
                }
            })
            .collect::<Vec<_>>();
        custom_groups
            .sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));
        groups.extend(custom_groups);
        groups
    }

    async fn tab_infos_from_sessions(
        &self,
        server_id: Option<String>,
        sessions: &[SessionInfo],
    ) -> Vec<TabInfo> {
        let mut tab_sessions: HashMap<TabId, Vec<SessionInfo>> = HashMap::new();

        for session in sessions {
            tab_sessions
                .entry(session.tab_id)
                .or_default()
                .push(session.clone());
        }

        let mut tab_sessions = tab_sessions.into_values().collect::<Vec<_>>();
        tab_sessions
            .sort_by_key(|sessions| sessions.first().map(|s| s.sequence).unwrap_or(u64::MAX));
        let mut tabs = Vec::new();
        for mut tab_sessions in tab_sessions {
            tab_sessions.sort_by_key(|s| s.sequence);
            let tab_id = tab_sessions[0].tab_id;
            let layout = self
                .layout_for_tab_sessions_with_state(tab_id, &tab_sessions)
                .await;
            tabs.push(TabInfo {
                server_id: server_id.clone(),
                tab_id,
                sessions: tab_sessions,
                layout,
            });
        }

        tabs.sort_by_key(|tab| tab.sessions.first().map(|s| s.sequence).unwrap_or(u64::MAX));
        tabs
    }

    fn layout_for_tab_sessions(sessions: &[SessionInfo]) -> TerminalLayout {
        let mut iter = sessions.iter();
        let first = iter
            .next()
            .map(|session| TerminalLayout::Terminal {
                session_id: session.id,
            })
            .unwrap_or_else(|| TerminalLayout::Terminal {
                session_id: SessionId::default(),
            });

        iter.fold(first, |layout, session| TerminalLayout::Split {
            direction: TerminalLayoutDirection::Horizontal,
            first_percent: 50.0,
            first: Box::new(layout),
            second: Box::new(TerminalLayout::Terminal {
                session_id: session.id,
            }),
        })
    }

    async fn layout_for_tab_sessions_with_state(
        &self,
        tab_id: TabId,
        sessions: &[SessionInfo],
    ) -> TerminalLayout {
        let session_ids = sessions
            .iter()
            .map(|session| session.id)
            .collect::<Vec<_>>();
        let mut layouts = self.tab_layouts.write().await;
        if let Some(layout) = layouts.get(&tab_id).cloned() {
            if let Some(pruned) = prune_layout_to_sessions(layout, &session_ids) {
                let mut layout_sessions = Vec::new();
                collect_layout_sessions(&pruned, &mut layout_sessions);
                let missing: Vec<_> = session_ids
                    .iter()
                    .filter(|id| !layout_sessions.contains(id))
                    .collect();
                if !missing.is_empty() {
                    warn!(
                        "TabInfo layout for tab={} is missing sessions: {:?}, using fallback",
                        tab_id, missing
                    );
                } else {
                    layouts.insert(tab_id, pruned.clone());
                    return pruned;
                }
            } else {
                warn!(
                    "TabInfo layout for tab={} was pruned to None (session mismatch), using fallback",
                    tab_id
                );
            }
        }
        let fallback = Self::layout_for_tab_sessions(sessions);
        layouts.insert(tab_id, fallback.clone());
        fallback
    }

    pub async fn list_terminal_profiles(&self) -> Vec<TerminalProfile> {
        let config = self.config_manager.snapshot().await;
        let server_id = config.server.server_id.clone();
        let server_name = local_server_name(&config);
        available_terminal_profiles(&config, self.local_terminal_profiles.as_slice())
            .into_iter()
            .map(|profile| annotate_terminal_profile(profile, &server_id, server_name.as_deref()))
            .collect()
    }

    pub async fn broadcast_session_list(&self) -> Result<(), SessionError> {
        #[cfg(debug_assertions)]
        self.debug_assert_tab_info_integrity().await;
        self.notify_state_changed().await;
        Ok(())
    }

    /// Post-file-open bookkeeping: remember the file in the session's recent
    /// list and broadcast the state change. Kept out of `read_file` so the
    /// FileContent response is not delayed by it (both parts are invisible to
    /// the fetching client's first paint).
    pub async fn record_recent_file_and_broadcast(
        &self,
        session_id: SessionId,
        path: String,
    ) -> Result<(), SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;
        {
            let mut session = session.lock().await;
            session.record_recent_file(path);
        }
        self.broadcast_session_list().await
    }

    #[cfg(debug_assertions)]
    async fn debug_assert_tab_info_integrity(&self) {
        let sessions = self.collect_session_info().await;
        let tabs = self.tab_infos_from_sessions(None, &sessions).await;
        let mut tab_layout_sessions = std::collections::HashMap::<TabId, Vec<SessionId>>::new();
        for tab in &tabs {
            let mut layout_sessions = Vec::new();
            collect_layout_sessions(&tab.layout, &mut layout_sessions);
            layout_sessions.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
            layout_sessions.dedup();
            tab_layout_sessions.insert(tab.tab_id, layout_sessions);
        }

        for tab in &tabs {
            let mut expected = tab
                .sessions
                .iter()
                .map(|session| session.id)
                .collect::<Vec<_>>();
            expected.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
            expected.dedup();
            let actual = tab_layout_sessions
                .get(&tab.tab_id)
                .cloned()
                .unwrap_or_default();
            if expected != actual {
                let missing: Vec<_> = expected.iter().filter(|id| !actual.contains(id)).collect();
                let extra: Vec<_> = actual.iter().filter(|id| !expected.contains(id)).collect();
                warn!(
                    "TabInfo integrity mismatch tab={} expected={:?} layout={:?} missing={:?} extra={:?}",
                    tab.tab_id, expected, actual, missing, extra
                );
            }
        }

        for session in &sessions {
            if !tabs.iter().any(|tab| tab.tab_id == session.tab_id) {
                warn!(
                    "TabInfo integrity orphan terminal session={} tab_id={}",
                    session.id, session.tab_id
                );
            }
        }

        debug!(
            "TabInfo integrity check: {} sessions, {} tabs, all terminals have TabInfo归宿",
            sessions.len(),
            tabs.len()
        );
    }

    async fn collect_session_info(&self) -> Vec<SessionInfo> {
        let config = self.config_manager.snapshot().await;
        let server_id = config.server.server_id.clone();
        let server_name = local_server_name(&config);
        let environments = config
            .environment
            .root
            .collect_leaves()
            .into_iter()
            .map(|node| EnvironmentProfileInfo {
                id: node.id().to_string(),
                name: node.name().to_string(),
            })
            .collect::<Vec<_>>();
        let environments_tree: Vec<vterm_common::EnvironmentNodeInfo> = config
            .environment
            .root
            .children
            .iter()
            .map(environment_node_to_info)
            .collect();
        let sessions = self.sessions.read().await;
        let session_arcs: Vec<_> = sessions.values().cloned().collect();
        drop(sessions);

        let mut items = Vec::with_capacity(session_arcs.len());
        for session in session_arcs {
            let mut info = session.lock().await.info();
            info.server_id = Some(server_id.clone());
            info.server_name = server_name.clone();
            info.environments = environments.clone();
            info.environments_tree = environments_tree.clone();
            items.push(info);
        }

        items.sort_by_key(|s| s.sequence);
        items
    }

    pub async fn resize_session(
        &self,
        session_id: SessionId,
        viewer_tx: &ConnectionTx,
        size: TerminalSize,
    ) -> Result<(), SessionError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or(SessionError::SessionNotFound(session_id))?;

        let generation = {
            let mut session = session.lock().await;
            if session.has_viewer(viewer_tx) {
                // Track the viewer's grid so `remove_viewer` can fall back to
                // a remaining viewer's size when it detaches.
                session.update_viewer_size(viewer_tx, size);
            }
            // The session emulator mirrors the last valid viewer's view size:
            // the viewer that reported a size most recently controls the PTY.
            // Non-viewer callers (e.g. the MCP resize_terminal tool) resize
            // authoritatively through the same path.
            session.defer_resize(size)
        };

        if let Some(generation) = generation {
            Self::spawn_deferred_resize_task(session_id, session.clone(), generation);
        }

        Ok(())
    }

    #[cfg(test)]
    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }
}

fn layout_contains_session(layout: &TerminalLayout, session_id: SessionId) -> bool {
    match layout {
        TerminalLayout::Terminal { session_id: id } => *id == session_id,
        TerminalLayout::Split { first, second, .. } => {
            layout_contains_session(first, session_id)
                || layout_contains_session(second, session_id)
        }
    }
}

fn collect_layout_sessions(layout: &TerminalLayout, session_ids: &mut Vec<SessionId>) {
    match layout {
        TerminalLayout::Terminal { session_id } => session_ids.push(*session_id),
        TerminalLayout::Split { first, second, .. } => {
            collect_layout_sessions(first, session_ids);
            collect_layout_sessions(second, session_ids);
        }
    }
}

fn replace_terminal_leaf(
    layout: &mut TerminalLayout,
    target_session_id: SessionId,
    new_session_id: SessionId,
    placement: TabSplitPlacement,
) -> bool {
    match layout {
        TerminalLayout::Terminal { session_id } if *session_id == target_session_id => {
            let (direction, first_session_id, second_session_id) = match placement {
                TabSplitPlacement::Left => (
                    TerminalLayoutDirection::Horizontal,
                    new_session_id,
                    target_session_id,
                ),
                TabSplitPlacement::Right => (
                    TerminalLayoutDirection::Horizontal,
                    target_session_id,
                    new_session_id,
                ),
                TabSplitPlacement::Top => (
                    TerminalLayoutDirection::Vertical,
                    new_session_id,
                    target_session_id,
                ),
                TabSplitPlacement::Bottom => (
                    TerminalLayoutDirection::Vertical,
                    target_session_id,
                    new_session_id,
                ),
            };
            *layout = TerminalLayout::Split {
                direction,
                first_percent: 50.0,
                first: Box::new(TerminalLayout::Terminal {
                    session_id: first_session_id,
                }),
                second: Box::new(TerminalLayout::Terminal {
                    session_id: second_session_id,
                }),
            };
            true
        }
        TerminalLayout::Terminal { .. } => false,
        TerminalLayout::Split { first, second, .. } => {
            replace_terminal_leaf(first, target_session_id, new_session_id, placement)
                || replace_terminal_leaf(second, target_session_id, new_session_id, placement)
        }
    }
}

fn prune_layout_to_sessions(
    layout: TerminalLayout,
    session_ids: &[SessionId],
) -> Option<TerminalLayout> {
    match layout {
        TerminalLayout::Terminal { session_id } => session_ids
            .contains(&session_id)
            .then_some(TerminalLayout::Terminal { session_id }),
        TerminalLayout::Split {
            direction,
            first_percent,
            first,
            second,
        } => match (
            prune_layout_to_sessions(*first, session_ids),
            prune_layout_to_sessions(*second, session_ids),
        ) {
            (Some(first), Some(second)) => Some(TerminalLayout::Split {
                direction,
                first_percent,
                first: Box::new(first),
                second: Box::new(second),
            }),
            (Some(only), None) | (None, Some(only)) => Some(only),
            (None, None) => None,
        },
    }
}

fn infer_shell_kind(
    program: &str,
    title: &str,
    terminal_os: Option<vterm_common::SessionTerminalOs>,
) -> Option<ShellKind> {
    match terminal_os {
        Some(vterm_common::SessionTerminalOs::Windows) => {
            let value = format!("{program} {title}").to_ascii_lowercase();
            if value.contains("powershell") || value.contains("pwsh") {
                Some(ShellKind::PowerShell)
            } else {
                Some(ShellKind::Cmd)
            }
        }
        Some(vterm_common::SessionTerminalOs::Linux | vterm_common::SessionTerminalOs::Macos) => {
            let basename = std::path::Path::new(program)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(program)
                .to_ascii_lowercase();
            match basename.as_str() {
                "zsh" => Some(ShellKind::Zsh),
                "bash" => Some(ShellKind::Bash),
                "fish" => Some(ShellKind::Fish),
                _ => Some(ShellKind::Sh),
            }
        }
        None => None,
    }
}

#[cfg(windows)]
fn replace_file_on_windows(staging_path: &Path, file_path: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    let staging: Vec<u16> = staging_path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let destination: Vec<u16> = file_path.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: both UTF-16 buffers are NUL-terminated and remain alive for the call.
    let replaced = unsafe {
        MoveFileExW(
            staging.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if replaced == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// Atomically persist `body` to `file_path`: write a sibling staging file,
/// flush and sync it, then rename over the destination so readers never see
/// a partially-written file. On failure the staging file is removed.
///
/// On Unix the existing destination's permission bits are copied onto the
/// staging file before the rename: the staging file is created with default
/// (umask-based, non-executable) permissions, and the rename would otherwise
/// silently strip attributes such as the executable bit from scripts being
/// saved by the file editor.
async fn persist_body_to_file(file_path: &Path, body: ReceivedBody) -> std::io::Result<()> {
    let parent = file_path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Destination has no parent directory",
        )
    })?;
    let staging_path = parent.join(format!(".vterm-save-{}.tmp", uuid::Uuid::new_v4()));

    let write_result: std::io::Result<()> = async {
        let mut staging = tokio::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&staging_path)
            .await?;
        match body {
            ReceivedBody::Empty => {}
            ReceivedBody::Memory(bytes) => staging.write_all(&bytes).await?,
            ReceivedBody::TempFile(spooled) => {
                let mut source = tokio::fs::File::open(spooled.path()).await?;
                tokio::io::copy(&mut source, &mut staging).await?;
            }
        }
        staging.flush().await?;
        staging.sync_data().await?;
        drop(staging);

        // `metadata.permissions()` carries the full mode bits on Unix.
        // A missing destination (new file) keeps the staging defaults.
        #[cfg(unix)]
        if let Ok(metadata) = tokio::fs::metadata(file_path).await {
            if metadata.is_file() {
                tokio::fs::set_permissions(&staging_path, metadata.permissions()).await?;
            }
        }

        #[cfg(windows)]
        return replace_file_on_windows(&staging_path, file_path);
        #[cfg(not(windows))]
        return tokio::fs::rename(&staging_path, file_path).await;
    }
    .await;
    if write_result.is_err() {
        let _ = tokio::fs::remove_file(&staging_path).await;
    }
    write_result
}

fn sanitize_temp_file_name(file_name: &str) -> Result<String, String> {
    if file_name.is_empty()
        || file_name == "."
        || file_name == ".."
        || file_name.contains(['/', '\\', '\0'])
    {
        return Err("Invalid temporary file name".to_string());
    }
    Ok(file_name.to_string())
}

fn temp_file_terminal_input(path: &Path, bracketed_paste: bool) -> Vec<u8> {
    let path = path.to_string_lossy();
    #[cfg(target_os = "windows")]
    let quoted = format!("\"{}\"", path.replace('"', "\\\""));
    #[cfg(not(target_os = "windows"))]
    let quoted = format!("'{}'", path.replace('\'', "'\\''"));
    let mut input = Vec::new();
    if bracketed_paste {
        input.extend_from_slice(b"\x1b[200~");
    }
    input.extend_from_slice(quoted.as_bytes());
    input.push(b' ');
    if bracketed_paste {
        input.extend_from_slice(b"\x1b[201~");
    }
    input
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ConfigManager, TerminalServerConfig, UartPortConfig};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use vterm_common::{SessionTerminalOs, TerminalProfile};
    use vterm_core::protocol::Message;

    use super::super::path_listing::{
        read_path_content, resolve_listing_target, windows_drive_root_path, ListingTarget,
    };
    use super::super::raw_history::RawHistoryBuffer;
    use super::super::terminal_profiles::{default_shell, find_profile_alias};

    async fn test_session_manager() -> SessionManager {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("session-tests-{unique_suffix}.json"));
        let manager = ConfigManager::load_from_path(PathBuf::from(&path))
            .await
            .unwrap();
        SessionManager::new(Arc::new(manager))
    }

    #[test]
    fn temporary_file_name_cannot_escape_its_directory() {
        assert_eq!(
            sanitize_temp_file_name("clipboard.jpg").unwrap(),
            "clipboard.jpg"
        );
        for invalid in [
            "",
            ".",
            "..",
            "../secret",
            "folder/image.jpg",
            "folder\\image.jpg",
        ] {
            assert!(
                sanitize_temp_file_name(invalid).is_err(),
                "accepted {invalid}"
            );
        }
    }

    #[test]
    fn temporary_file_terminal_input_quotes_path_without_submitting() {
        let path = Path::new("temporary image.jpg");
        let plain = temp_file_terminal_input(path, false);
        assert!(!plain.contains(&b'\r'));
        assert!(!plain.contains(&b'\n'));
        assert!(plain.ends_with(b" "));
        let bracketed = temp_file_terminal_input(path, true);
        assert!(bracketed.starts_with(b"\x1b[200~"));
        assert!(bracketed.ends_with(b" \x1b[201~"));
        assert!(!bracketed.contains(&b'\r'));
    }

    #[tokio::test]
    async fn temporary_file_upload_inserts_path_into_real_terminal() {
        let manager = test_session_manager().await;
        let session_id = match manager
            .create_session(
                TerminalSize::new(24, 80),
                default_shell(),
                None,
                HashMap::new(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
        {
            Ok(session_id) => session_id,
            Err(SessionError::PtyError(_)) => return,
            Err(error) => panic!("unexpected session creation error: {error}"),
        };
        let result = manager
            .upload_temp_file(
                session_id,
                "test-upload".to_string(),
                "clipboard test.jpg".to_string(),
                "image/jpeg".to_string(),
                TempFileUsage::InsertIntoTerminal {
                    bracketed_paste: false,
                },
                ReceivedBody::Memory(vec![1, 2, 3, 4].into()),
            )
            .await;
        let Message::TempFileUploadResult {
            path, bytes, error, ..
        } = result
        else {
            panic!("unexpected response");
        };
        assert_eq!(error, None);
        assert_eq!(bytes, 4);
        let path = PathBuf::from(path.unwrap());
        assert_eq!(tokio::fs::read(&path).await.unwrap(), vec![1, 2, 3, 4]);

        let session = manager.get_session(session_id).await.unwrap();
        let mut inserted = false;
        for _ in 0..50 {
            if session
                .lock()
                .await
                .terminal
                .current_line_text()
                .contains("clipboard test.jpg")
            {
                inserted = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert!(inserted, "uploaded path did not reach the terminal");
        manager.close_session(session_id).await;
        let _ = tokio::fs::remove_dir_all(path.parent().unwrap()).await;
    }

    #[test]
    fn vterm_terminal_env_enables_qwen_progress_reporting() {
        let mut env = HashMap::new();
        apply_vterm_terminal_env(&mut env);
        assert_eq!(env.get("ConEmuPID").map(String::as_str), Some("1"));
        assert_eq!(env.get("TERM_PROGRAM").map(String::as_str), Some("WezTerm"));

        let mut configured = HashMap::from([("ConEmuPID".to_string(), "42".to_string())]);
        apply_vterm_terminal_env(&mut configured);
        assert_eq!(configured.get("ConEmuPID").map(String::as_str), Some("42"));
    }

    #[test]
    fn terminal_capability_env_fills_missing_term_and_colorterm() {
        let mut env = HashMap::new();
        apply_terminal_capability_env(&mut env, None, None);
        assert_eq!(env.get("TERM").map(String::as_str), Some("xterm-256color"));
        assert_eq!(env.get("COLORTERM").map(String::as_str), Some("truecolor"));
    }

    #[test]
    fn terminal_capability_env_overrides_dumb_term() {
        // A GUI-launched server inherits TERM=dumb (e.g. from the spawning
        // editor), which silently disables all program color output even though
        // the client emulator renders full xterm-256color.
        let mut env = HashMap::new();
        apply_terminal_capability_env(&mut env, Some("dumb"), None);
        assert_eq!(env.get("TERM").map(String::as_str), Some("xterm-256color"));
    }

    #[test]
    fn terminal_capability_env_keeps_existing_term_untouched() {
        let mut env = HashMap::new();
        env.insert("TERM".to_string(), "xterm-kitty".to_string());
        apply_terminal_capability_env(&mut env, None, None);
        assert_eq!(env.get("TERM").map(String::as_str), Some("xterm-kitty"));

        let mut env = HashMap::new();
        apply_terminal_capability_env(&mut env, Some("screen-256color"), Some("24bit"));
        assert!(!env.contains_key("TERM"));
        assert!(!env.contains_key("COLORTERM"));
    }

    #[tokio::test]
    async fn persist_body_to_file_replaces_existing_file() {
        use vterm_transport::{BodyAssembler, TransportLimits};

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("notes.txt");
        std::fs::write(&file_path, b"old").unwrap();
        let data: &[u8] = b"new contents";
        let mut assembler = BodyAssembler::new(Some(data.len() as u64), TransportLimits::default())
            .await
            .unwrap();
        assembler.push(data).await.unwrap();
        let body = assembler.finish().await.unwrap();

        persist_body_to_file(&file_path, body).await.unwrap();

        assert_eq!(std::fs::read(&file_path).unwrap(), data);
    }

    #[tokio::test]
    async fn upload_auto_renames_instead_of_overwriting_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("note.txt"), b"original").unwrap();
        let mut writer = vterm_core::folder_stream::FolderFrameWriter::new();
        writer.file_begin("note.txt", 4);
        writer.chunk(b"new!");
        writer.file_end(4);
        writer.end();
        let mut sink = FolderUploadSink {
            task_id: "test-upload".to_string(),
            session_id: SessionId::new(),
            fs_dir: dir.path().to_path_buf(),
            decoder: FolderFrameDecoder::new(),
            current: None,
            files_done: 0,
            bytes_done: 0,
            failed: None,
        };

        sink.push(&writer.take()).await;

        assert!(sink.failed.is_none());
        assert_eq!(
            std::fs::read(dir.path().join("note.txt")).unwrap(),
            b"original"
        );
        assert_eq!(
            std::fs::read(dir.path().join("note copy.txt")).unwrap(),
            b"new!"
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn persist_body_to_file_preserves_executable_bit() {
        use std::os::unix::fs::PermissionsExt;
        use vterm_transport::{BodyAssembler, TransportLimits};

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("script.sh");
        std::fs::write(&file_path, b"#!/bin/sh\necho old\n").unwrap();
        std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o755)).unwrap();

        let data: &[u8] = b"#!/bin/sh\necho new\n";
        let mut assembler = BodyAssembler::new(Some(data.len() as u64), TransportLimits::default())
            .await
            .unwrap();
        assembler.push(data).await.unwrap();
        let body = assembler.finish().await.unwrap();

        persist_body_to_file(&file_path, body).await.unwrap();

        let mode = std::fs::metadata(&file_path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o755, "save must preserve the executable bit");
        assert_eq!(std::fs::read(&file_path).unwrap(), data);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn persist_body_to_file_new_file_keeps_default_permissions() {
        use std::os::unix::fs::PermissionsExt;
        use vterm_transport::{BodyAssembler, TransportLimits};

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("notes.txt");

        let data: &[u8] = b"hello";
        let mut assembler = BodyAssembler::new(Some(data.len() as u64), TransportLimits::default())
            .await
            .unwrap();
        assembler.push(data).await.unwrap();
        let body = assembler.finish().await.unwrap();

        persist_body_to_file(&file_path, body).await.unwrap();

        let mode = std::fs::metadata(&file_path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode & 0o111, 0, "new files must not be executable");
        assert_eq!(std::fs::read(&file_path).unwrap(), data);
    }

    #[tokio::test]
    async fn test_session_creation() {
        let manager = test_session_manager().await;
        let session_id = match manager
            .create_session(
                TerminalSize::new(24, 80),
                default_shell(),
                None,
                HashMap::new(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
        {
            Ok(session_id) => session_id,
            Err(SessionError::PtyError(_)) => return,
            Err(error) => panic!("unexpected session creation error: {error}"),
        };
        assert_eq!(manager.session_count().await, 1);

        let session = manager.get_session(session_id).await;
        assert!(session.is_some());
    }

    #[test]
    fn should_respawn_on_app_exit_only_for_known_apps() {
        assert!(SessionManager::should_respawn_on_app_exit(&SessionApp(
            "opencode".to_string()
        )));
        assert!(!SessionManager::should_respawn_on_app_exit(&SessionApp(
            "vim".to_string()
        )));
        assert!(!SessionManager::should_respawn_on_app_exit(
            &SessionApp::none()
        ));
    }

    #[tokio::test]
    async fn child_exit_with_opencode_respawns_shell_instead_of_closing() {
        let manager = test_session_manager().await;
        let session_id = match manager
            .create_session(
                TerminalSize::new(24, 80),
                default_shell(),
                None,
                HashMap::new(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
        {
            Ok(session_id) => session_id,
            Err(SessionError::PtyError(_)) => return,
            Err(error) => panic!("unexpected session creation error: {error}"),
        };

        // Simulate "opencode was running when the parent shell died".
        {
            let session = manager.get_session(session_id).await.unwrap();
            let mut s = session.lock().await;
            s.app = SessionApp("opencode".to_string());
            assert_eq!(s.io_generation, 0);
            assert!(!s.launch_program.is_empty());
        }

        manager.handle_child_exit(session_id, 0).await.unwrap();

        // The session survived with a respawned shell and cleared app state.
        let session = manager.get_session(session_id).await;
        assert!(session.is_some(), "session must survive the app exit");
        {
            let s = session.as_ref().unwrap().lock().await;
            assert_eq!(s.io_generation, 1);
            assert_eq!(s.app, SessionApp::none());
            assert_eq!(s.app_status, SessionAppStatus::None);
        }

        // A stale exit event from the replaced PTY is ignored.
        manager.handle_child_exit(session_id, 0).await.unwrap();
        assert!(manager.get_session(session_id).await.is_some());

        // Without a known app running, child exit closes the session.
        manager.handle_child_exit(session_id, 1).await.unwrap();
        assert!(manager.get_session(session_id).await.is_none());
    }

    #[tokio::test]
    async fn clear_terminal_buffer_empties_screen_and_scrollback() {
        let manager = test_session_manager().await;
        let session_id = match manager
            .create_session(
                TerminalSize::new(24, 80),
                default_shell(),
                None,
                HashMap::new(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
        {
            Ok(session_id) => session_id,
            Err(SessionError::PtyError(_)) => return,
            Err(error) => panic!("unexpected session creation error: {error}"),
        };
        // Let the shell's startup output (banner/prompt) settle so it cannot
        // race the post-clear assertions.
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;

        // 60 lines on a 24-row grid force lines into scrollback.
        let mut filler = Vec::new();
        for i in 0..60 {
            filler.extend_from_slice(format!("line-{i}\r\n").as_bytes());
        }
        manager
            .process_session_output(session_id, filler)
            .await
            .unwrap();
        {
            let session = manager.get_session(session_id).await.unwrap();
            let s = session.lock().await;
            assert!(
                !s.terminal.scrollback().is_empty(),
                "filler output must produce scrollback"
            );
        }

        manager.clear_terminal_buffer(session_id).await.unwrap();

        let session = manager.get_session(session_id).await.unwrap();
        let s = session.lock().await;
        assert!(
            s.terminal.scrollback().is_empty(),
            "clear must purge scrollback"
        );
        assert!(
            s.terminal.unwrapped_screen_text().trim().is_empty(),
            "clear must blank the visible grid"
        );
        assert_eq!(
            s.screen_history_logged_rows, 0,
            "clear must reset the screen-history log delta"
        );
    }

    #[test]
    fn replace_terminal_leaf_builds_nested_split_layout() {
        let first_id = SessionId::new();
        let second_id = SessionId::new();
        let third_id = SessionId::new();
        let mut layout = TerminalLayout::Split {
            direction: TerminalLayoutDirection::Horizontal,
            first_percent: 50.0,
            first: Box::new(TerminalLayout::Terminal {
                session_id: first_id,
            }),
            second: Box::new(TerminalLayout::Terminal {
                session_id: second_id,
            }),
        };

        assert!(replace_terminal_leaf(
            &mut layout,
            second_id,
            third_id,
            TabSplitPlacement::Bottom,
        ));

        let TerminalLayout::Split { second, .. } = layout else {
            panic!("expected root split");
        };
        let TerminalLayout::Split {
            direction,
            first,
            second,
            ..
        } = *second
        else {
            panic!("expected nested split");
        };
        assert_eq!(direction, TerminalLayoutDirection::Vertical);
        assert!(
            matches!(*first, TerminalLayout::Terminal { session_id } if session_id == second_id)
        );
        assert!(
            matches!(*second, TerminalLayout::Terminal { session_id } if session_id == third_id)
        );
    }

    #[test]
    fn replace_terminal_leaf_supports_before_placement() {
        let first_id = SessionId::new();
        let second_id = SessionId::new();
        let mut layout = TerminalLayout::Terminal {
            session_id: first_id,
        };

        assert!(replace_terminal_leaf(
            &mut layout,
            first_id,
            second_id,
            TabSplitPlacement::Left,
        ));

        let TerminalLayout::Split {
            direction,
            first,
            second,
            ..
        } = layout
        else {
            panic!("expected split layout");
        };
        assert_eq!(direction, TerminalLayoutDirection::Horizontal);
        assert!(
            matches!(*first, TerminalLayout::Terminal { session_id } if session_id == second_id)
        );
        assert!(
            matches!(*second, TerminalLayout::Terminal { session_id } if session_id == first_id)
        );
    }

    #[test]
    fn prune_layout_to_sessions_removes_duplicate_split_session_before_replacement() {
        let first_id = SessionId::new();
        let second_id = SessionId::new();
        let layout = TerminalLayout::Split {
            direction: TerminalLayoutDirection::Horizontal,
            first_percent: 50.0,
            first: Box::new(TerminalLayout::Terminal {
                session_id: first_id,
            }),
            second: Box::new(TerminalLayout::Terminal {
                session_id: second_id,
            }),
        };

        let pruned = prune_layout_to_sessions(layout, &[first_id]).unwrap();

        assert!(
            matches!(pruned, TerminalLayout::Terminal { session_id } if session_id == first_id)
        );
    }

    #[test]
    fn prune_layout_to_sessions_collapses_removed_terminal() {
        let first_id = SessionId::new();
        let second_id = SessionId::new();
        let layout = TerminalLayout::Split {
            direction: TerminalLayoutDirection::Horizontal,
            first_percent: 50.0,
            first: Box::new(TerminalLayout::Terminal {
                session_id: first_id,
            }),
            second: Box::new(TerminalLayout::Terminal {
                session_id: second_id,
            }),
        };

        let pruned = prune_layout_to_sessions(layout, &[first_id]).unwrap();

        assert!(
            matches!(pruned, TerminalLayout::Terminal { session_id } if session_id == first_id)
        );
    }

    #[tokio::test]
    async fn test_list_sessions_does_not_create_default_session() {
        let manager = test_session_manager().await;

        let sessions = manager.list_sessions().await.unwrap();

        assert!(sessions.is_empty());
        assert_eq!(manager.session_count().await, 0);
    }

    #[tokio::test]
    async fn test_add_remove_viewer() {
        let manager = test_session_manager().await;
        let (viewer_tx, _viewer_rx) = mpsc::unbounded_channel();

        let session_id = match manager
            .create_session(
                TerminalSize::new(24, 80),
                default_shell(),
                None,
                HashMap::new(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
        {
            Ok(session_id) => session_id,
            Err(SessionError::PtyError(_)) => return,
            Err(error) => panic!("unexpected session creation error: {error}"),
        };

        let messages = manager
            .add_viewer(session_id, viewer_tx.clone(), None)
            .await
            .unwrap();
        assert_eq!(messages.len(), 2);
        assert!(matches!(messages[0], Message::HistorySnapshot { .. }));
        assert!(matches!(messages[1], Message::SessionBufferInfo { .. }));

        let count = manager.remove_viewer(session_id, &viewer_tx).await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_rename_session_persists_normalized_session_name() {
        let manager = test_session_manager().await;
        let session_id = match manager
            .create_session(
                TerminalSize::new(24, 80),
                default_shell(),
                None,
                HashMap::new(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
        {
            Ok(session_id) => session_id,
            Err(SessionError::PtyError(_)) => return,
            Err(error) => panic!("unexpected session creation error: {error}"),
        };

        manager
            .rename_session(session_id, Some("  Renamed Session  ".to_string()))
            .await
            .unwrap();

        let renamed = manager
            .list_sessions()
            .await
            .unwrap()
            .into_iter()
            .find(|session| session.id == session_id)
            .unwrap();
        assert_eq!(renamed.session_name.as_deref(), Some("Renamed Session"));

        manager
            .rename_session(session_id, Some("   ".to_string()))
            .await
            .unwrap();

        let cleared = manager
            .list_sessions()
            .await
            .unwrap()
            .into_iter()
            .find(|session| session.id == session_id)
            .unwrap();
        assert!(cleared.session_name.is_none());
    }

    #[tokio::test]
    async fn test_list_sessions_include_local_server_identity() {
        let manager = test_session_manager().await;
        let server_id = manager.config_manager.snapshot().await.server.server_id;
        let session_id = match manager
            .create_session(
                TerminalSize::new(24, 80),
                default_shell(),
                None,
                HashMap::new(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
        {
            Ok(session_id) => session_id,
            Err(SessionError::PtyError(_)) => return,
            Err(error) => panic!("unexpected session creation error: {error}"),
        };

        let session = manager
            .list_sessions()
            .await
            .unwrap()
            .into_iter()
            .find(|session| session.id == session_id)
            .unwrap();
        assert_eq!(session.server_id.as_deref(), Some(server_id.as_str()));
    }

    #[tokio::test]
    async fn test_create_session_with_sibling_assigns_shared_group_id() {
        let manager = test_session_manager().await;
        let first_session_id = match manager
            .create_session(
                TerminalSize::new(24, 80),
                default_shell(),
                None,
                HashMap::new(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
        {
            Ok(session_id) => session_id,
            Err(SessionError::PtyError(_)) => return,
            Err(error) => panic!("unexpected session creation error: {error}"),
        };
        let second_session_id = match manager
            .create_session(
                TerminalSize::new(24, 80),
                default_shell(),
                None,
                HashMap::new(),
                None,
                Some(first_session_id),
                None,
                None,
                None,
                None,
            )
            .await
        {
            Ok(session_id) => session_id,
            Err(SessionError::PtyError(_)) => return,
            Err(error) => panic!("unexpected session creation error: {error}"),
        };

        let sessions = manager.list_sessions().await.unwrap();
        let first = sessions
            .iter()
            .find(|session| session.id == first_session_id)
            .unwrap();
        let second = sessions
            .iter()
            .find(|session| session.id == second_session_id)
            .unwrap();
        assert_ne!(first.group_id, 0);
        assert_eq!(first.group_id, second.group_id);
        assert_eq!(first.tab_id, first_session_id);
        assert_eq!(second.tab_id, first_session_id);

        let tabs = manager.list_tab_infos().await;
        let tab = tabs
            .iter()
            .find(|tab| tab.tab_id == first_session_id)
            .expect("split sessions should be attached to first session tab");
        assert_eq!(tab.sessions.len(), 2);
    }

    #[tokio::test]
    async fn test_create_session_tab_group_resolution_defaults_unknown_group() {
        let manager = test_session_manager().await;
        let group_id = manager
            .create_tab_group("Work".to_string(), None)
            .await
            .expect("group should be created");

        assert_eq!(
            manager
                .resolve_tab_group_id_for_new_session(Some(group_id))
                .await,
            group_id
        );
        assert_eq!(
            manager
                .resolve_tab_group_id_for_new_session(Some(group_id + 10_000))
                .await,
            TAB_GROUP_UNGROUPED_ID
        );
        assert_eq!(
            manager
                .resolve_tab_group_id_for_new_session(Some(TAB_GROUP_ALL_ID))
                .await,
            TAB_GROUP_UNGROUPED_ID
        );
    }

    #[tokio::test]
    async fn test_close_session_resets_group_id_when_one_sibling_remains() {
        let manager = test_session_manager().await;
        let first_session_id = match manager
            .create_session(
                TerminalSize::new(24, 80),
                default_shell(),
                None,
                HashMap::new(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
        {
            Ok(session_id) => session_id,
            Err(SessionError::PtyError(_)) => return,
            Err(error) => panic!("unexpected session creation error: {error}"),
        };
        let second_session_id = match manager
            .create_session(
                TerminalSize::new(24, 80),
                default_shell(),
                None,
                HashMap::new(),
                None,
                Some(first_session_id),
                None,
                None,
                None,
                None,
            )
            .await
        {
            Ok(session_id) => session_id,
            Err(SessionError::PtyError(_)) => return,
            Err(error) => panic!("unexpected session creation error: {error}"),
        };

        manager.close_session(first_session_id).await.unwrap();

        let remaining = manager
            .list_sessions()
            .await
            .unwrap()
            .into_iter()
            .find(|session| session.id == second_session_id)
            .unwrap();
        assert_eq!(remaining.group_id, 0);
        assert_eq!(remaining.tab_id, second_session_id);
    }

    #[tokio::test]
    async fn test_list_terminal_profiles_include_local_server_identity() {
        let manager = test_session_manager().await;
        let config = manager.config_manager.snapshot().await;
        let server_id = config.server.server_id.clone();
        let server_name = local_server_name(&config);

        let profiles = manager.list_terminal_profiles().await;
        assert!(!profiles.is_empty());
        assert!(profiles
            .iter()
            .all(|profile| profile.server_id.as_deref() == Some(server_id.as_str())));
        assert!(profiles
            .iter()
            .all(|profile| profile.server_name == server_name));
    }

    #[test]
    fn test_raw_history_buffer_reads_latest_window_by_offset() {
        let mut history = RawHistoryBuffer::new_default();
        history.append(b"hello");
        history.append(b" world");

        let (start_offset, data) = history.read_before(history.end_offset(), 5);
        assert_eq!(start_offset, 6);
        assert_eq!(data, b"world");
    }

    #[test]
    fn test_raw_history_buffer_clear_retained_keeps_offsets_monotonic() {
        let mut history = RawHistoryBuffer::new_default();
        history.append(b"hello");
        history.append(b" world");
        assert_eq!(history.end_offset(), 11);

        history.clear_retained();

        // The window is empty but positioned at the old end: an attaching
        // viewer learns there is nothing to replay, and later appends
        // continue the offset space without rewinding.
        assert_eq!(history.start_offset(), 11);
        assert_eq!(history.end_offset(), 11);
        assert!(history.snapshot().is_empty());
        let (start_offset, data) = history.read_before(history.end_offset(), 64);
        assert_eq!(start_offset, 11);
        assert!(data.is_empty());

        let start = history.append(b"again");
        assert_eq!(start, 11);
        assert_eq!(history.snapshot(), b"again");
    }

    #[test]
    fn test_read_path_content_marks_directories_compactly() {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("path-list-{unique_suffix}"));
        std::fs::create_dir_all(root.join("dir")).unwrap();
        std::fs::write(root.join("file.txt"), b"test").unwrap();

        let items = read_path_content(ListingTarget {
            real_path: root.to_string_lossy().into_owned(),
            filesystem_path: Some(root.clone()),
            runtime_os: SessionTerminalOs::Windows,
        })
        .unwrap();

        assert!(items
            .iter()
            .any(|item| item.name == "dir" && item.is_dir == Some(true)));
        assert!(items
            .iter()
            .any(|item| item.name == "file.txt" && item.is_dir.is_none()));

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_resolve_listing_target_falls_back_to_windows_root_for_missing_path() {
        let target = resolve_listing_target(
            None,
            "Z:\\definitely-missing-vterm-path",
            SessionTerminalOs::Windows,
        )
        .unwrap();

        assert!(target.real_path.is_empty());
        assert!(target.filesystem_path.is_none());
        assert_eq!(target.runtime_os, SessionTerminalOs::Windows);
    }

    #[test]
    fn test_resolve_listing_target_falls_back_to_unix_root_for_missing_path() {
        let target = resolve_listing_target(
            None,
            "/definitely-missing-vterm-path",
            SessionTerminalOs::Linux,
        )
        .unwrap();

        assert_eq!(target.real_path, "/");
        assert_eq!(target.filesystem_path, Some(PathBuf::from("/")));
        assert_eq!(target.runtime_os, SessionTerminalOs::Linux);
    }

    #[test]
    fn test_watched_listing_target_skips_missing_path_instead_of_root_fallback() {
        let target = super::super::path_listing::resolve_existing_listing_target(
            None,
            "/definitely-missing-vterm-path",
            SessionTerminalOs::Linux,
        )
        .unwrap();

        assert!(target.is_none());
    }

    #[test]
    fn test_windows_drive_root_path_normalizes_drive_requests() {
        assert_eq!(windows_drive_root_path("c:"), Some("C:\\".to_string()));
        assert_eq!(windows_drive_root_path("D:\\"), Some("D:\\".to_string()));
        assert_eq!(windows_drive_root_path("e:/"), Some("E:\\".to_string()));
        assert_eq!(windows_drive_root_path("folder"), None);
    }

    #[tokio::test]
    async fn test_resize_follows_last_viewer_size() {
        let manager = test_session_manager().await;
        let (first_viewer_tx, _first_viewer_rx) = mpsc::unbounded_channel();
        let (second_viewer_tx, _second_viewer_rx) = mpsc::unbounded_channel();

        let session_id = match manager
            .create_session(
                TerminalSize::new(24, 80),
                default_shell(),
                None,
                HashMap::new(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
        {
            Ok(session_id) => session_id,
            Err(SessionError::PtyError(_)) => return,
            Err(error) => panic!("unexpected session creation error: {error}"),
        };

        manager
            .add_viewer(session_id, first_viewer_tx.clone(), None)
            .await
            .unwrap();
        manager
            .add_viewer(session_id, second_viewer_tx.clone(), None)
            .await
            .unwrap();

        // The first viewer grows: it becomes the most recently active viewer,
        // so the PTY follows its grid even though the second viewer is still
        // at the (smaller) session default.
        manager
            .resize_session(session_id, &first_viewer_tx, TerminalSize::new(30, 100))
            .await
            .unwrap();
        let session = manager.get_session(session_id).await.unwrap();
        {
            let session = session.lock().await;
            assert_eq!(session.pending_resize, Some(TerminalSize::new(30, 100)));
        }

        // The second viewer grows: it is now the most recently active viewer,
        // so the PTY follows it — the last valid viewer wins, not the
        // minimum.
        manager
            .resize_session(session_id, &second_viewer_tx, TerminalSize::new(32, 120))
            .await
            .unwrap();
        {
            let session = session.lock().await;
            assert_eq!(session.pending_resize, Some(TerminalSize::new(32, 120)));
        }

        // The first viewer shrinks: it is the most recent again, so the PTY
        // follows its (smaller) grid.
        manager
            .resize_session(session_id, &first_viewer_tx, TerminalSize::new(20, 60))
            .await
            .unwrap();
        {
            let session = session.lock().await;
            assert_eq!(session.pending_resize, Some(TerminalSize::new(20, 60)));
        }
    }

    #[tokio::test]
    async fn test_remove_viewer_grows_pty_back() {
        let manager = test_session_manager().await;
        let (first_viewer_tx, _first_viewer_rx) = mpsc::unbounded_channel();
        let (second_viewer_tx, _second_viewer_rx) = mpsc::unbounded_channel();

        let session_id = match manager
            .create_session(
                TerminalSize::new(24, 80),
                default_shell(),
                None,
                HashMap::new(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
        {
            Ok(session_id) => session_id,
            Err(SessionError::PtyError(_)) => return,
            Err(error) => panic!("unexpected session creation error: {error}"),
        };

        manager
            .add_viewer(session_id, first_viewer_tx.clone(), None)
            .await
            .unwrap();
        manager
            .add_viewer(session_id, second_viewer_tx.clone(), None)
            .await
            .unwrap();
        manager
            .resize_session(session_id, &first_viewer_tx, TerminalSize::new(20, 60))
            .await
            .unwrap();

        let session = manager.get_session(session_id).await.unwrap();
        {
            let mut session = session.lock().await;
            assert_eq!(session.pending_resize, Some(TerminalSize::new(20, 60)));
            session.pending_resize = None;
            session.size = TerminalSize::new(20, 60);
        }

        // The most recently active viewer leaves: the PTY falls back to the
        // remaining viewer's size.
        manager
            .remove_viewer(session_id, &first_viewer_tx)
            .await
            .unwrap();
        {
            let mut session = session.lock().await;
            assert_eq!(session.pending_resize, Some(TerminalSize::new(24, 80)));
        }
    }

    #[test]
    fn test_available_terminal_profiles_combine_cached_shells_and_uart_entries() {
        let mut config = TerminalServerConfig::default();
        config.uart.ports.push(UartPortConfig {
            id: "uart-1".to_string(),
            name: "Console".to_string(),
            port: "COM3".to_string(),
            ..UartPortConfig::default()
        });
        let cached_shell_profiles = vec![TerminalProfile {
            id: "cached-shell".to_string(),
            title: "Cached Shell".to_string(),
            program: "shell".to_string(),
            args: Vec::new(),
            is_default: true,
            server_id: None,
            server_name: None,
        }];

        let profiles = available_terminal_profiles(&config, &cached_shell_profiles);

        assert!(profiles.iter().any(|profile| profile.id == "cached-shell"));
        assert!(profiles
            .iter()
            .any(|profile| profile.id == "uart:uart-1" && profile.title == "UART: Console (COM3)"));
    }

    #[test]
    fn test_find_profile_alias_matches_windows_shell_aliases() {
        let profiles = vec![
            TerminalProfile {
                id: "windows-powershell".to_string(),
                title: "Windows PowerShell".to_string(),
                program: r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe".to_string(),
                args: Vec::new(),
                is_default: true,
                server_id: None,
                server_name: None,
            },
            TerminalProfile {
                id: "command-prompt".to_string(),
                title: "Command Prompt".to_string(),
                program: r"C:\Windows\System32\cmd.exe".to_string(),
                args: Vec::new(),
                is_default: false,
                server_id: None,
                server_name: None,
            },
            TerminalProfile {
                id: "powershell-7".to_string(),
                title: "PowerShell".to_string(),
                program: r"C:\Program Files\PowerShell\7\pwsh.exe".to_string(),
                args: Vec::new(),
                is_default: false,
                server_id: None,
                server_name: None,
            },
        ];

        assert_eq!(
            find_profile_alias(&profiles, "cmd").map(|profile| profile.id.as_str()),
            Some("command-prompt")
        );
        assert_eq!(
            find_profile_alias(&profiles, "powershell").map(|profile| profile.id.as_str()),
            Some("windows-powershell")
        );
        assert_eq!(
            find_profile_alias(&profiles, "pwsh").map(|profile| profile.id.as_str()),
            Some("powershell-7")
        );
    }
}
