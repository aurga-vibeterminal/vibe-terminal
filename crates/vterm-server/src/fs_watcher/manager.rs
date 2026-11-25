use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Component, Path, PathBuf},
    sync::{Arc, LazyLock, Mutex},
};

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use notify::{Config, RecommendedWatcher, Watcher};
use tokio::sync::broadcast;
use vterm_common::{ErrorCode, SessionId};
use vterm_core::protocol::{FsWatchConfig, FsWatchFileContent, FsWatchGit, Message};

use crate::session::{ConnectionTx, SessionError, SessionManager};

use super::{FileSystemNotification, FsWatcherError, WatchMode, WatchRequest, WatchedPath};

const DEFAULT_EVENT_BUFFER: usize = 4096;
const FS_WATCH_DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(500);

/// Environment variable opting in to the logs of this module. Unset (the
/// default) or empty keeps the server quiet; any other value, e.g.
/// `VTERM_FS_WATCHER_LOG=1`, enables them. vterm reads the same
/// variable for its `[fs_watcher]` logs, so one setting covers both sides.
const FS_WATCHER_LOG_ENV: &str = "VTERM_FS_WATCHER_LOG";

static FS_WATCHER_LOG_ENABLED: LazyLock<bool> =
    LazyLock::new(|| std::env::var(FS_WATCHER_LOG_ENV).is_ok_and(|value| !value.is_empty()));

/// Trace this module only when [`FS_WATCHER_LOG_ENV`] opted in.
macro_rules! fs_log {
    (debug, $($arg:tt)+) => {
        if *FS_WATCHER_LOG_ENABLED {
            tracing::debug!($($arg)+);
        }
    };
    (info, $($arg:tt)+) => {
        if *FS_WATCHER_LOG_ENABLED {
            tracing::info!($($arg)+);
        }
    };
    (warn, $($arg:tt)+) => {
        if *FS_WATCHER_LOG_ENABLED {
            tracing::warn!($($arg)+);
        }
    };
}

#[derive(Debug, Default)]
struct WatchState {
    paths: HashMap<PathBuf, WatchMode>,
}

#[derive(Debug, Clone)]
struct FileContentWatch {
    request: FsWatchFileContent,
    watched_path: PathBuf,
}

#[derive(Debug, Clone)]
struct GitWatch {
    request: FsWatchGit,
    repo_root: PathBuf,
    ignore_matcher: IgnoreMatcher,
    /// Linux only: the non-ignored top-level directories of `repo_root`
    /// registered with one recursive watch each (the root itself is watched
    /// non-recursively). Ignored directories such as `target/` or
    /// `node_modules/` are skipped so large repositories do not waste inotify
    /// watches on trees nobody looks at. Empty on macOS/Windows, where the
    /// whole root is covered by a single native recursive watch.
    watched_dirs: Vec<PathBuf>,
    /// Linux only: whether `.git` itself is registered with a recursive
    /// watch. Always false on macOS/Windows (covered by the root watch).
    watching_git_dir: bool,
}

/// Linux splits a Git watch into a non-recursive watch on the repository
/// root plus one recursive watch per non-ignored top-level directory:
/// inotify has no native recursive mode, and a single recursive registration
/// would also descend into ignored trees such as `target/` or
/// `node_modules/`, burning watch descriptors. macOS (FSEvents) and Windows
/// (ReadDirectoryChangesW) have natively recursive backends, so there one
/// recursive watch on the repository root is both cheaper and simpler;
/// ignore filtering happens at event-classification time instead.
const GIT_WATCH_TOP_LEVEL_DIRS: bool = cfg!(target_os = "linux");

#[derive(Debug, Clone)]
struct DirectoryWatch {
    requested_path: String,
    watched_path: PathBuf,
    ignore_matcher: IgnoreMatcher,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QuickCommandsWatch {
    repo_root: PathBuf,
    vterm_dir: PathBuf,
    commands_path: PathBuf,
    watching_vterm_dir: bool,
}

#[derive(Debug, Clone)]
struct IgnoreMatcher {
    root: PathBuf,
    gitignores: Vec<ScopedGitignore>,
}

#[derive(Debug, Clone)]
struct ScopedGitignore {
    root: PathBuf,
    gitignore: Gitignore,
}

#[derive(Debug, Clone)]
struct SessionWatchState {
    /// Owner terminal connection for this session's watches. `SetFsWatchers`
    /// rides on the terminal connection, so the last client message on that
    /// connection wins; the owner's stream end/disconnect removes the state.
    tx: ConnectionTx,
    file_content: Option<FileContentWatch>,
    git: Option<GitWatch>,
    /// The client's desired git watch request. `git` holds the resolved
    /// watch and is dropped when the session path is not inside a
    /// repository, but the desire must survive so a later path change can
    /// re-resolve it (see `refresh_session_git_watch`).
    git_request: Option<FsWatchGit>,
    directories: Vec<DirectoryWatch>,
    quick_commands: Option<QuickCommandsWatch>,
}

#[derive(Debug)]
struct Inner {
    watcher: RecommendedWatcher,
    path_refs: HashMap<PathBuf, PathRef>,
    sessions: HashMap<SessionId, SessionWatchState>,
}

#[derive(Debug, Default, Clone, Copy)]
struct PathRef {
    recursive: usize,
    non_recursive: usize,
}

impl PathRef {
    fn total(self) -> usize {
        self.recursive + self.non_recursive
    }

    fn effective_mode(self) -> WatchMode {
        if self.recursive > 0 {
            WatchMode::Recursive
        } else {
            WatchMode::NonRecursive
        }
    }
}

pub struct FsWatcherManager {
    inner: Mutex<Inner>,
    legacy_state: Arc<Mutex<WatchState>>,
    events: broadcast::Sender<FileSystemNotification>,
}

#[derive(Debug, Default)]
struct SessionWatchUpdate {
    file_content: Option<Option<FileContentWatch>>,
    git: Option<Option<GitWatch>>,
    /// The client's desired git request. Carried independently of the
    /// resolved `git` watch so the desire survives a clearing resolution.
    git_request: Option<Option<FsWatchGit>>,
    directories: Option<Vec<DirectoryWatch>>,
    quick_commands: Option<Option<QuickCommandsWatch>>,
}

#[derive(Debug, Default)]
struct TriggeredWatches {
    files: Vec<(ConnectionTx, SessionId, FsWatchFileContent)>,
    git: Vec<(ConnectionTx, SessionId, FsWatchGit)>,
    git_history: Vec<SessionId>,
    git_branches: Vec<SessionId>,
    directories: Vec<(ConnectionTx, SessionId, String)>,
    quick_commands: Vec<SessionId>,
}

impl TriggeredWatches {
    fn is_empty(&self) -> bool {
        self.files.is_empty()
            && self.git.is_empty()
            && self.git_history.is_empty()
            && self.git_branches.is_empty()
            && self.directories.is_empty()
            && self.quick_commands.is_empty()
    }

    fn merge(&mut self, other: TriggeredWatches) {
        for file in other.files {
            if !self
                .files
                .iter()
                .any(|(_, session_id, request)| *session_id == file.1 && request == &file.2)
            {
                self.files.push(file);
            }
        }

        for git in other.git {
            if !self
                .git
                .iter()
                .any(|(_, session_id, request)| *session_id == git.1 && request == &git.2)
            {
                self.git.push(git);
            }
        }

        for session_id in other.git_history {
            if !self.git_history.contains(&session_id) {
                self.git_history.push(session_id);
            }
        }

        for session_id in other.git_branches {
            if !self.git_branches.contains(&session_id) {
                self.git_branches.push(session_id);
            }
        }

        for directory in other.directories {
            if !self
                .directories
                .iter()
                .any(|(_, session_id, path)| *session_id == directory.1 && path == &directory.2)
            {
                self.directories.push(directory);
            }
        }

        for session_id in other.quick_commands {
            if !self.quick_commands.contains(&session_id) {
                self.quick_commands.push(session_id);
            }
        }
    }
}

impl FsWatcherManager {
    pub fn new() -> Result<Self, FsWatcherError> {
        let legacy_state = Arc::new(Mutex::new(WatchState::default()));
        let (events, _) = broadcast::channel(DEFAULT_EVENT_BUFFER);
        let callback_state = Arc::clone(&legacy_state);
        let callback_events = events.clone();

        let watcher = RecommendedWatcher::new(
            move |result: notify::Result<notify::Event>| match result {
                Ok(event) => {
                    let watched_roots = callback_state
                        .lock()
                        .map(|state| state.paths.keys().cloned().collect::<Vec<_>>())
                        .unwrap_or_default();
                    fs_log!(
                        debug,
                        kind = ?event.kind,
                        paths = ?event.paths,
                        watched_roots = ?watched_roots,
                        "file system watcher callback received notify event"
                    );
                    let notification =
                        FileSystemNotification::from_notify_event(event, &watched_roots);
                    let kind = notification.kind.clone();
                    let paths = notification.paths.clone();
                    let watched_root = notification.watched_root.clone();
                    match callback_events.send(notification) {
                        Ok(receiver_count) => fs_log!(
                            debug,
                            kind = ?kind,
                            paths = ?paths,
                            watched_root = ?watched_root,
                            receiver_count,
                            "file system watcher callback published notification"
                        ),
                        Err(error) => fs_log!(
                            warn,
                            kind = ?kind,
                            paths = ?paths,
                            watched_root = ?watched_root,
                            error = %error,
                            "file system watcher callback dropped notification"
                        ),
                    }
                }
                Err(error) => fs_log!(warn, error = %error, "file system watcher emitted an error"),
            },
            Config::default(),
        )?;

        Ok(Self {
            inner: Mutex::new(Inner {
                watcher,
                path_refs: HashMap::new(),
                sessions: HashMap::new(),
            }),
            legacy_state,
            events,
        })
    }

    pub async fn run(self: Arc<Self>, session_manager: Arc<SessionManager>) {
        let mut rx = self.events.subscribe();
        let mut pending = TriggeredWatches::default();
        let mut flush_at = None;
        let mut lagged_events = 0u64;

        loop {
            if pending.is_empty() {
                let notification = match rx.recv().await {
                    Ok(notification) => notification,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        lagged_events = lagged_events.saturating_add(skipped);
                        pending.merge(self.all_session_watches());
                        flush_at = Some(tokio::time::Instant::now() + FS_WATCH_DEBOUNCE);
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        fs_log!(
                            warn,
                            "file system watcher dispatcher stopped because event channel closed"
                        );
                        break;
                    }
                };
                let triggered = self.triggered_watches(&notification);
                log_triggered_watches(&notification, &triggered);
                if !triggered.is_empty() {
                    pending.merge(triggered);
                    flush_at = Some(tokio::time::Instant::now() + FS_WATCH_DEBOUNCE);
                }
                continue;
            }

            let deadline =
                flush_at.unwrap_or_else(|| tokio::time::Instant::now() + FS_WATCH_DEBOUNCE);
            tokio::select! {
                result = rx.recv() => {
                    match result {
                        Ok(notification) => {
                            let triggered = self.triggered_watches(&notification);
                            log_triggered_watches(&notification, &triggered);
                            pending.merge(triggered);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            lagged_events = lagged_events.saturating_add(skipped);
                            pending.merge(self.all_session_watches());
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            fs_log!(warn, "file system watcher dispatcher stopped because event channel closed");
                            let triggered = std::mem::take(&mut pending);
                            self.dispatch_triggered_watches(&session_manager, triggered).await;
                            break;
                        }
                    }
                }
                _ = tokio::time::sleep_until(deadline) => {
                    let triggered = std::mem::take(&mut pending);
                    flush_at = None;
                    if lagged_events > 0 {
                        fs_log!(
                            warn,
                            skipped = lagged_events,
                            "file system watcher dispatcher lagged; coalescing into a full watched-state refresh"
                        );
                        lagged_events = 0;
                    }
                    self.dispatch_triggered_watches(&session_manager, triggered).await;
                }
            }
        }
    }

    pub async fn update_session_watches(
        &self,
        session_id: SessionId,
        watches: FsWatchConfig,
        session_manager: &SessionManager,
        tx: ConnectionTx,
    ) -> Result<(), FsWatcherError> {
        fs_log!(
            info,
            session_id = %session_id,
            file_content = ?watches.file_content.as_ref().map(|watch| watch.path.as_str()),
            git = watches.git.is_some(),
            directories = ?watches.directories,
            "updating fs watchers for session"
        );
        let update = self
            .build_session_watch_update(session_id, watches, session_manager)
            .await?;
        self.apply_session_watch_update(session_id, update, tx)
    }

    /// Re-resolve the repository derived from a session's current CWD.  When it
    /// changes, move the `.vterm/commands.json` watch and publish the commands
    /// from the new repository to every viewer of that session.
    pub async fn refresh_session_quick_commands(
        &self,
        session_id: SessionId,
        session_manager: &SessionManager,
    ) -> Result<(), FsWatcherError> {
        let tx = {
            let inner = self.inner.lock().expect("fs watcher state poisoned");
            let Some(state) = inner.sessions.get(&session_id) else {
                return Ok(());
            };
            state.tx.clone()
        };

        let quick_commands = self
            .build_quick_commands_watch_update(session_id, session_manager)
            .await?;
        let Some(quick_commands) = quick_commands else {
            return Ok(());
        };

        // Partial update: only the quick-commands watch is replaced. It is
        // disjoint from the client-driven file/git/directory fields, so no
        // ordering against `update_session_watches` is required.
        self.apply_session_watch_update(
            session_id,
            SessionWatchUpdate {
                quick_commands: Some(quick_commands),
                ..Default::default()
            },
            tx,
        )?;

        fs_log!(
            debug,
            session_id = %session_id,
            "dispatching quick commands refresh after session CWD repository change"
        );
        self.dispatch_quick_commands_refresh(session_manager, session_id)
            .await;
        Ok(())
    }

    /// Re-resolve the git watch derived from the session's current viewing
    /// path (falling back to the CWD). A `SetFsWatchers` carrying the git
    /// watch can be applied while the path-adoption listing is still in
    /// flight, so the watch may have been resolved — and dropped, when the
    /// stale path is not inside a repository — against the pre-listing
    /// path. Call this once the session's path is authoritative to restore
    /// or move the watch.
    pub async fn refresh_session_git_watch(
        &self,
        session_id: SessionId,
        session_manager: &SessionManager,
    ) -> Result<(), FsWatcherError> {
        let (tx, watches) = {
            let inner = self.inner.lock().expect("fs watcher state poisoned");
            let Some(state) = inner.sessions.get(&session_id) else {
                return Ok(());
            };
            if state.git_request.is_none() {
                return Ok(());
            }
            // Rebuild the client's desired config from the applied state.
            // The file/directory requests round-trip unchanged (build marks
            // them no-ops); only the git request is re-resolved against the
            // current session path.
            (
                state.tx.clone(),
                FsWatchConfig {
                    file_content: state
                        .file_content
                        .as_ref()
                        .map(|watch| watch.request.clone()),
                    git: state.git_request.clone(),
                    directories: state
                        .directories
                        .iter()
                        .map(|watch| watch.requested_path.clone())
                        .collect(),
                },
            )
        };

        let update = self
            .build_session_watch_update(session_id, watches, session_manager)
            .await?;
        self.apply_session_watch_update(session_id, update, tx)
    }

    pub fn remove_session(&self, session_id: SessionId) {
        let mut inner = self.inner.lock().expect("fs watcher state poisoned");
        let Some(existing) = inner.sessions.remove(&session_id) else {
            return;
        };
        unwatch_session_paths(&mut inner, existing);
        sync_legacy_paths(&self.legacy_state, &inner.path_refs);
    }

    /// Remove a session's watches only if `tx` is still the owner terminal
    /// connection that set them. A late `StreamEnd`/disconnect from an old
    /// connection must not tear down watches a newer connection installed.
    pub fn remove_session_if_owner(&self, session_id: SessionId, tx: &ConnectionTx) {
        let is_owner = {
            let inner = self.inner.lock().expect("fs watcher state poisoned");
            inner
                .sessions
                .get(&session_id)
                .is_some_and(|state| state.tx.same_channel(tx))
        };
        if is_owner {
            self.remove_session(session_id);
        }
    }

    fn clear_session_git_watch(&self, session_id: SessionId) {
        let mut inner = self.inner.lock().expect("fs watcher state poisoned");
        let Some(mut state) = inner.sessions.remove(&session_id) else {
            return;
        };
        if let Some(git) = state.git.take() {
            remove_git_watch_path_refs(&mut inner, &git);
        }
        if state.file_content.is_some()
            || state.git_request.is_some()
            || !state.directories.is_empty()
            || state.quick_commands.is_some()
        {
            inner.sessions.insert(session_id, state);
        }
        sync_legacy_paths(&self.legacy_state, &inner.path_refs);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<FileSystemNotification> {
        self.events.subscribe()
    }

    pub fn watched_paths(&self) -> Vec<WatchedPath> {
        let inner = self.inner.lock().expect("fs watcher state poisoned");
        inner
            .path_refs
            .iter()
            .map(|(path, refs)| WatchedPath {
                path: path.clone(),
                mode: refs.effective_mode(),
            })
            .collect()
    }

    pub fn add_watch_ref(
        &self,
        path: impl AsRef<Path>,
        mode: WatchMode,
    ) -> Result<PathBuf, FsWatcherError> {
        let path = normalize_existing_path(path.as_ref())?;
        ensure_watch_path_allowed(&path, mode, true)?;
        let mut inner = self.inner.lock().expect("fs watcher state poisoned");
        add_path_ref(&mut inner, &path, mode)?;
        sync_legacy_paths(&self.legacy_state, &inner.path_refs);
        Ok(path)
    }

    pub fn remove_watch_ref(&self, path: impl AsRef<Path>, mode: WatchMode) {
        let mut inner = self.inner.lock().expect("fs watcher state poisoned");
        remove_path_ref(&mut inner, path.as_ref(), mode);
        sync_legacy_paths(&self.legacy_state, &inner.path_refs);
    }
    pub fn watch(&self, request: WatchRequest) -> Result<(), FsWatcherError> {
        let path = normalize_existing_path(&request.path)?;
        ensure_watch_path_allowed(&path, request.mode, true)?;
        let mut inner = self.inner.lock().expect("fs watcher state poisoned");
        if inner.path_refs.contains_key(&path) {
            return Err(FsWatcherError::AlreadyWatched(path));
        }
        add_path_ref(&mut inner, &path, request.mode)?;
        sync_legacy_paths(&self.legacy_state, &inner.path_refs);
        Ok(())
    }

    pub fn unwatch(&self, path: impl AsRef<Path>) -> Result<(), FsWatcherError> {
        let path = normalize_existing_path(path.as_ref())?;
        let mut inner = self.inner.lock().expect("fs watcher state poisoned");
        if !inner.path_refs.contains_key(&path) {
            return Err(FsWatcherError::NotWatched(path));
        }
        remove_all_path_refs(&mut inner, &path);
        sync_legacy_paths(&self.legacy_state, &inner.path_refs);
        Ok(())
    }

    async fn build_session_watch_update(
        &self,
        session_id: SessionId,
        watches: FsWatchConfig,
        session_manager: &SessionManager,
    ) -> Result<SessionWatchUpdate, FsWatcherError> {
        let current = {
            let inner = self.inner.lock().expect("fs watcher state poisoned");
            inner.sessions.get(&session_id).cloned()
        };

        let file_content = match watches.file_content {
            Some(request) => {
                let unchanged = current
                    .as_ref()
                    .and_then(|state| state.file_content.as_ref())
                    .is_some_and(|watch| watch.request == request);
                if unchanged {
                    None
                } else {
                    let path = session_manager
                        .resolve_fs_watch_path(session_id, &request.path)
                        .await
                        .map_err(|error| FsWatcherError::Session(error.to_string()))?;
                    let watched_path = normalize_existing_path(&path)?;
                    fs_log!(
                        info,
                        session_id = %session_id,
                        requested_path = %request.path,
                        watched_path = %watched_path.display(),
                        "resolved fs watcher file content path"
                    );
                    Some(Some(FileContentWatch {
                        request,
                        watched_path,
                    }))
                }
            }
            None => current
                .as_ref()
                .and_then(|state| state.file_content.as_ref())
                .map(|_| None),
        };

        // Record the desired request even when the resolution below drops
        // the watch: the session path can become valid later, and
        // `refresh_session_git_watch` needs the desire to re-resolve it.
        let git_request = match &watches.git {
            Some(request) => Some(Some(request.clone())),
            None => current
                .as_ref()
                .and_then(|state| state.git_request.as_ref())
                .map(|_| None),
        };

        let git = match watches.git {
            Some(request) => match session_manager.resolve_git_watch_path(session_id).await {
                Ok(path) => {
                    let path = normalize_existing_path(&path)?;
                    if !path.is_dir() {
                        return Err(FsWatcherError::NotDirectory(path));
                    }
                    ensure_watch_path_allowed(&path, WatchMode::Recursive, true)?;
                    fs_log!(
                        info,
                        session_id = %session_id,
                        watched_path = %path.display(),
                        "resolved fs watcher git path"
                    );
                    let ignore_matcher = current
                        .as_ref()
                        .and_then(|state| state.git.as_ref())
                        .filter(|watch| watch.repo_root == path)
                        .map(|watch| watch.ignore_matcher.clone())
                        .unwrap_or_else(|| build_ignore_matcher(&path));
                    Some(Some(GitWatch {
                        request,
                        ignore_matcher,
                        repo_root: path,
                        watched_dirs: Vec::new(),
                        watching_git_dir: false,
                    }))
                }
                Err(SessionError::FileSystemError(message))
                    if message == "Not a git repository" =>
                {
                    fs_log!(
                        info,
                        session_id = %session_id,
                        "clearing fs watcher git path because session is not inside a git repository"
                    );
                    current
                        .as_ref()
                        .and_then(|state| state.git.as_ref())
                        .map(|_| None)
                }
                Err(error) => return Err(FsWatcherError::Session(error.to_string())),
            },
            None => current
                .as_ref()
                .and_then(|state| state.git.as_ref())
                .map(|_| None),
        };

        let requested_directories = normalize_requested_directories(watches.directories);
        let directories = {
            let current_requests = current
                .as_ref()
                .map(|state| {
                    state
                        .directories
                        .iter()
                        .map(|watch| watch.requested_path.clone())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if current_requests == requested_directories {
                None
            } else {
                let mut resolved = Vec::with_capacity(requested_directories.len());
                for requested_path in requested_directories {
                    let path = session_manager
                        .resolve_fs_watch_path(session_id, &requested_path)
                        .await
                        .map_err(|error| FsWatcherError::Session(error.to_string()))?;
                    // Tree-mode expansion state can briefly retain a path after
                    // its parent has changed (for example while a generated
                    // `.vterm/launch.json` is opened). A missing directory is
                    // not a watcher setup failure: ignore that stale entry and
                    // still install all other valid watches in this update.
                    let path = match normalize_existing_path(&path) {
                        Ok(path) => path,
                        Err(FsWatcherError::PathNotFound(path)) => {
                            fs_log!(
                                debug,
                                session_id = %session_id,
                                requested_path = %requested_path,
                                path = %path.display(),
                                "skipping fs watcher directory that no longer exists"
                            );
                            continue;
                        }
                        Err(error) => return Err(error),
                    };
                    if !path.is_dir() {
                        return Err(FsWatcherError::NotDirectory(path));
                    }
                    ensure_watch_path_allowed(&path, WatchMode::NonRecursive, false)?;
                    fs_log!(
                        info,
                        session_id = %session_id,
                        requested_path = %requested_path,
                        watched_path = %path.display(),
                        "resolved fs watcher directory path"
                    );
                    resolved.push(DirectoryWatch {
                        requested_path,
                        ignore_matcher: build_shallow_ignore_matcher(&path),
                        watched_path: path,
                    });
                }
                Some(resolved)
            }
        };

        let quick_commands = self
            .build_quick_commands_watch_update(session_id, session_manager)
            .await?;

        Ok(SessionWatchUpdate {
            file_content,
            git,
            git_request,
            directories,
            quick_commands,
        })
    }

    async fn build_quick_commands_watch_update(
        &self,
        session_id: SessionId,
        session_manager: &SessionManager,
    ) -> Result<Option<Option<QuickCommandsWatch>>, FsWatcherError> {
        let current = {
            let inner = self.inner.lock().expect("fs watcher state poisoned");
            inner.sessions.get(&session_id).cloned()
        };

        Ok(
            match crate::debug::command_paths(session_manager, session_id).await {
                Ok((repo_root, _)) => {
                    let repo_root = normalize_existing_path(&repo_root)?;
                    if !repo_root.is_dir() {
                        return Err(FsWatcherError::NotDirectory(repo_root));
                    }
                    ensure_watch_path_allowed(&repo_root, WatchMode::NonRecursive, true)?;
                    let vterm_dir = repo_root.join(".vterm");
                    let next = QuickCommandsWatch {
                        commands_path: vterm_dir.join("commands.json"),
                        watching_vterm_dir: vterm_dir.is_dir(),
                        repo_root,
                        vterm_dir,
                    };
                    let unchanged = current
                        .as_ref()
                        .and_then(|state| state.quick_commands.as_ref())
                        == Some(&next);
                    if unchanged {
                        None
                    } else {
                        fs_log!(
                            info,
                            session_id = %session_id,
                            repo_root = %next.repo_root.display(),
                            commands_path = %next.commands_path.display(),
                            watching_vterm_dir = next.watching_vterm_dir,
                            "resolved quick commands watch paths"
                        );
                        Some(Some(next))
                    }
                }
                Err(error) => {
                    fs_log!(
                        warn,
                        session_id = %session_id,
                        error = %error,
                        "quick commands watcher is unavailable for session"
                    );
                    current
                        .as_ref()
                        .and_then(|state| state.quick_commands.as_ref())
                        .map(|_| None)
                }
            },
        )
    }

    fn apply_session_watch_update(
        &self,
        session_id: SessionId,
        update: SessionWatchUpdate,
        tx: ConnectionTx,
    ) -> Result<(), FsWatcherError> {
        let mut inner = self.inner.lock().expect("fs watcher state poisoned");
        let mut state = inner
            .sessions
            .remove(&session_id)
            .unwrap_or(SessionWatchState {
                tx: tx.clone(),
                file_content: None,
                git: None,
                git_request: None,
                directories: Vec::new(),
                quick_commands: None,
            });
        state.tx = tx;

        let apply_result = (|| -> Result<(), FsWatcherError> {
            if let Some(file_content) = update.file_content {
                if let Some(existing) = state.file_content.take() {
                    remove_path_ref(&mut inner, &existing.watched_path, WatchMode::NonRecursive);
                }
                if let Some(next) = file_content {
                    add_path_ref(&mut inner, &next.watched_path, WatchMode::NonRecursive)?;
                    state.file_content = Some(next);
                }
            }

            if let Some(git_request) = update.git_request {
                state.git_request = git_request;
            }

            if let Some(git) = update.git {
                match (state.git.take(), git) {
                    (Some(mut existing), Some(next)) if existing.repo_root == next.repo_root => {
                        // The request and ignore matcher may have changed, but the
                        // watched directories did not. Keep the existing OS watches
                        // in place and only replace the session metadata, carrying
                        // over the watched-directory bookkeeping.
                        existing.request = next.request;
                        existing.ignore_matcher = next.ignore_matcher;
                        state.git = Some(existing);
                    }
                    (existing, next) => {
                        if let Some(existing) = existing {
                            remove_git_watch_path_refs(&mut inner, &existing);
                        }
                        if let Some(mut next) = next {
                            add_git_watch_path_refs(&mut inner, &mut next)?;
                            state.git = Some(next);
                        }
                    }
                }
            }

            if let Some(directories) = update.directories {
                for existing in state.directories.drain(..) {
                    remove_path_ref(&mut inner, &existing.watched_path, WatchMode::NonRecursive);
                }
                for next in directories {
                    add_path_ref(&mut inner, &next.watched_path, WatchMode::NonRecursive)?;
                    state.directories.push(next);
                }
            }

            if let Some(quick_commands) = update.quick_commands {
                if let Some(existing) = state.quick_commands.take() {
                    remove_quick_commands_path_refs(&mut inner, &existing);
                }
                if let Some(next) = quick_commands {
                    add_path_ref(&mut inner, &next.repo_root, WatchMode::NonRecursive)?;
                    if next.watching_vterm_dir {
                        if let Err(error) =
                            add_path_ref(&mut inner, &next.vterm_dir, WatchMode::NonRecursive)
                        {
                            remove_path_ref(&mut inner, &next.repo_root, WatchMode::NonRecursive);
                            return Err(error);
                        }
                    }
                    state.quick_commands = Some(next);
                }
            }
            Ok(())
        })();

        if let Err(error) = apply_result {
            // `state` was temporarily removed from `inner.sessions` so its path
            // refs can be updated without aliasing `inner`. If an OS watch
            // registration fails (notably ENOSPC on Linux), returning early
            // without cleanup would orphan every unchanged ref owned by this
            // session. Repeated updates could then permanently exhaust the
            // inotify limit even after the session stopped using those paths.
            unwatch_session_paths(&mut inner, state);
            sync_legacy_paths(&self.legacy_state, &inner.path_refs);
            return Err(error);
        }

        let file_content_path = state
            .file_content
            .as_ref()
            .map(|watch| watch.watched_path.display().to_string());
        let git_path = state
            .git
            .as_ref()
            .map(|watch| watch.repo_root.display().to_string());
        let directory_paths = state
            .directories
            .iter()
            .map(|watch| {
                format!(
                    "{} => {}",
                    watch.requested_path,
                    watch.watched_path.display()
                )
            })
            .collect::<Vec<_>>();
        let quick_commands_path = state
            .quick_commands
            .as_ref()
            .map(|watch| watch.commands_path.display().to_string());
        if state.file_content.is_some()
            || state.git.is_some()
            || state.git_request.is_some()
            || !state.directories.is_empty()
            || state.quick_commands.is_some()
        {
            inner.sessions.insert(session_id, state);
        }
        sync_legacy_paths(&self.legacy_state, &inner.path_refs);
        let active_path_refs = inner
            .path_refs
            .iter()
            .map(|(path, refs)| {
                format!(
                    "{} (recursive={}, non_recursive={}, effective={:?})",
                    path.display(),
                    refs.recursive,
                    refs.non_recursive,
                    refs.effective_mode()
                )
            })
            .collect::<Vec<_>>();
        fs_log!(
            info,
            session_id = %session_id,
            file_content_path = ?file_content_path,
            git_path = ?git_path,
            directory_paths = ?directory_paths,
            quick_commands_path = ?quick_commands_path,
            active_path_refs = ?active_path_refs,
            "applied fs watcher session state"
        );
        Ok(())
    }

    fn all_session_watches(&self) -> TriggeredWatches {
        let inner = self.inner.lock().expect("fs watcher state poisoned");
        let mut triggered = TriggeredWatches::default();
        for (session_id, state) in &inner.sessions {
            if let Some(file) = &state.file_content {
                triggered
                    .files
                    .push((state.tx.clone(), *session_id, file.request.clone()));
            }
            if let Some(git) = &state.git {
                triggered
                    .git
                    .push((state.tx.clone(), *session_id, git.request.clone()));
                triggered.git_history.push(*session_id);
                triggered.git_branches.push(*session_id);
            }
            for directory in &state.directories {
                triggered.directories.push((
                    state.tx.clone(),
                    *session_id,
                    directory.requested_path.clone(),
                ));
            }
            if state.quick_commands.is_some() {
                triggered.quick_commands.push(*session_id);
            }
        }
        triggered
    }

    fn triggered_watches(&self, notification: &FileSystemNotification) -> TriggeredWatches {
        let mut inner = self.inner.lock().expect("fs watcher state poisoned");
        if reconcile_quick_commands_watch_dirs(&mut inner) {
            sync_legacy_paths(&self.legacy_state, &inner.path_refs);
        }
        let matcher_refreshed = refresh_git_ignore_matchers(&mut inner, notification);
        let mut git_dirs_changed = reconcile_git_watch_dirs(&mut inner, notification);
        for session_id in matcher_refreshed {
            git_dirs_changed |= resync_git_watch_dirs(&mut inner, session_id);
        }
        if git_dirs_changed {
            sync_legacy_paths(&self.legacy_state, &inner.path_refs);
        }
        let mut triggered = TriggeredWatches::default();
        let mut emitted_dirs = HashSet::<(SessionId, String)>::new();

        for (session_id, state) in &inner.sessions {
            if let Some(file) = state.file_content.as_ref() {
                if event_touches_exact_path(notification, &file.watched_path) {
                    triggered
                        .files
                        .push((state.tx.clone(), *session_id, file.request.clone()));
                }
            }

            if let Some(git) = state.git.as_ref() {
                let git_trigger = event_touches_git_watch(notification, git);
                if git_trigger.changed_files {
                    triggered
                        .git
                        .push((state.tx.clone(), *session_id, git.request.clone()));
                }
                if git_trigger.history {
                    triggered.git_history.push(*session_id);
                }
                if git_trigger.branches {
                    triggered.git_branches.push(*session_id);
                }
            }

            for directory in &state.directories {
                if event_touches_directory_path(notification, &directory.watched_path)
                    && emitted_dirs.insert((*session_id, directory.requested_path.clone()))
                {
                    triggered.directories.push((
                        state.tx.clone(),
                        *session_id,
                        directory.requested_path.clone(),
                    ));
                }
            }

            if let Some(quick_commands) = state.quick_commands.as_ref() {
                if event_touches_quick_commands(notification, quick_commands) {
                    triggered.quick_commands.push(*session_id);
                }
            }
        }

        triggered
    }

    fn has_session_directory_watch(&self, session_id: SessionId, requested_path: &str) -> bool {
        let requested_path = comparable_path(Path::new(requested_path));
        let inner = self.inner.lock().expect("fs watcher state poisoned");
        inner.sessions.get(&session_id).is_some_and(|state| {
            state.directories.iter().any(|directory| {
                comparable_path(Path::new(&directory.requested_path)) == requested_path
                    || comparable_path(&directory.watched_path) == requested_path
            })
        })
    }

    async fn dispatch_triggered_watches(
        &self,
        session_manager: &SessionManager,
        triggered: TriggeredWatches,
    ) {
        let mut cleared_git_sessions = HashSet::new();
        for (tx, session_id, request) in triggered.files {
            fs_log!(
                debug,
                session_id = %session_id,
                path = %request.path,
                "dispatching watched file content refresh"
            );
            let message = match session_manager
                .read_file(session_id, request.path.clone(), request.max_size)
                .await
            {
                Ok(message) => message,
                Err(error) => Message::FileContent {
                    session_id,
                    path: request.path,
                    data: Vec::new(),
                    diff_ranges: Vec::new(),
                    error: Some(error.to_string()),
                },
            };
            let _ = tx.send(message);
        }

        for (_tx, session_id, request) in triggered.git {
            fs_log!(
                debug,
                session_id = %session_id,
                commit = ?request.commit,
                "dispatching watched git refresh"
            );
            match session_manager
                .git_changed_files(session_id, request.commit.clone())
                .await
            {
                Ok(message) => {
                    send_message_to_session_viewers(session_manager, session_id, message).await;
                }
                Err(SessionError::FileSystemError(message))
                    if message == "Not a git repository" =>
                {
                    self.clear_session_git_watch(session_id);
                    cleared_git_sessions.insert(session_id);
                    fs_log!(
                        info,
                        session_id = %session_id,
                        "cleared watched Git repository after it stopped being a repository"
                    );
                }
                Err(error) => {
                    fs_log!(warn, session_id = %session_id, error = %error, "failed to refresh watched git changes")
                }
            }
        }

        for session_id in triggered.git_branches {
            if cleared_git_sessions.contains(&session_id) {
                continue;
            }
            fs_log!(
                debug,
                session_id = %session_id,
                "dispatching watched Git branch refresh"
            );
            match session_manager.git_branches(session_id).await {
                Ok(message) => {
                    send_message_to_session_viewers(session_manager, session_id, message).await;
                }
                Err(error) => fs_log!(
                    warn,
                    session_id = %session_id,
                    error = %error,
                    "failed to refresh watched Git branches"
                ),
            }
        }

        for session_id in triggered.git_history {
            if cleared_git_sessions.contains(&session_id) {
                continue;
            }
            fs_log!(
                debug,
                session_id = %session_id,
                "dispatching watched Git history refresh"
            );
            match session_manager.git_log(session_id, None, None, None).await {
                Ok(message) => {
                    send_message_to_session_viewers(session_manager, session_id, message).await;
                }
                Err(error) => fs_log!(
                    warn,
                    session_id = %session_id,
                    error = %error,
                    "failed to refresh watched Git history"
                ),
            }
        }

        for (tx, session_id, path) in triggered.directories {
            if !self.has_session_directory_watch(session_id, &path) {
                fs_log!(
                    debug,
                    session_id = %session_id,
                    path = %path,
                    "skipping debounced refresh for a directory that is no longer watched"
                );
                continue;
            }
            fs_log!(
                debug,
                session_id = %session_id,
                path = %path,
                "dispatching watched directory refresh"
            );
            match session_manager
                .list_watched_path_content(session_id, path.clone())
                .await
            {
                Ok(message) => {
                    if self.has_session_directory_watch(session_id, &path) {
                        let _ = tx.send(message);
                    } else {
                        fs_log!(
                            debug,
                            session_id = %session_id,
                            path = %path,
                            "dropping refresh for a directory removed while it was being listed"
                        );
                    }
                }
                Err(error) => {
                    if self.has_session_directory_watch(session_id, &path) {
                        let _ = tx.send(Message::Error {
                            code: ErrorCode::ServerError,
                            message: error.to_string(),
                        });
                    }
                }
            }
        }

        for session_id in triggered.quick_commands {
            self.dispatch_quick_commands_refresh(session_manager, session_id)
                .await;
        }
    }

    async fn dispatch_quick_commands_refresh(
        &self,
        session_manager: &SessionManager,
        session_id: SessionId,
    ) {
        fs_log!(debug, session_id = %session_id, "dispatching quick commands refresh");
        let message = crate::debug::command_list(session_manager, session_id).await;
        let Some(session) = session_manager.get_session(session_id).await else {
            fs_log!(
                debug,
                session_id = %session_id,
                "skipping quick commands refresh for removed session"
            );
            return;
        };
        let viewers = session.lock().await.viewers.clone();
        for viewer in viewers {
            let _ = viewer.send(message.clone());
        }
    }
}

pub(crate) async fn send_message_to_session_viewers(
    session_manager: &SessionManager,
    session_id: SessionId,
    message: Message,
) {
    let Some(session) = session_manager.get_session(session_id).await else {
        return;
    };
    let viewers = session.lock().await.viewers.clone();
    for viewer in viewers {
        let _ = viewer.send(message.clone());
    }
}

fn log_triggered_watches(notification: &FileSystemNotification, triggered: &TriggeredWatches) {
    if !triggered.is_empty() {
        fs_log!(
            debug,
            kind = ?notification.kind,
            paths = ?notification.paths,
            file_count = triggered.files.len(),
            git_count = triggered.git.len(),
            git_history_count = triggered.git_history.len(),
            git_branches_count = triggered.git_branches.len(),
            directory_count = triggered.directories.len(),
            quick_commands_count = triggered.quick_commands.len(),
            "file system notification matched fs watchers"
        );
    } else {
        fs_log!(
            debug,
            kind = ?notification.kind,
            paths = ?notification.paths,
            "file system notification did not match fs watchers"
        );
    }
}
fn add_path_ref(inner: &mut Inner, path: &Path, mode: WatchMode) -> Result<(), FsWatcherError> {
    let previous_refs = inner.path_refs.get(path).copied().unwrap_or_default();
    let previous_total = previous_refs.total();
    let previous_mode = (previous_total > 0).then(|| previous_refs.effective_mode());
    let mut next_refs = previous_refs;
    match mode {
        WatchMode::Recursive => next_refs.recursive += 1,
        WatchMode::NonRecursive => next_refs.non_recursive += 1,
    }
    let next_mode = next_refs.effective_mode();

    if previous_total == 0 {
        //         info!(
        //             path = %path.display(),
        //             mode = ?next_mode,
        //             "registering file system watch path"
        //         );
        inner.watcher.watch(path, next_mode.into())?;
    //         info!(path = %path.display(), mode = ?next_mode, "registered file system watch path");
    } else if previous_mode != Some(next_mode) {
        //         info!(
        //             path = %path.display(),
        //             previous_mode = ?previous_mode,
        //             next_mode = ?next_mode,
        //             "updating file system watch path mode"
        //         );
        if let Err(error) = inner.watcher.unwatch(path) {
            fs_log!(warn, path = %path.display(), error = %error, "failed to unwatch file system path before mode update");
        }
        if let Err(error) = inner.watcher.watch(path, next_mode.into()) {
            if let Some(previous_mode) = previous_mode {
                if let Err(restore_error) = inner.watcher.watch(path, previous_mode.into()) {
                    fs_log!(
                        warn,
                        path = %path.display(),
                        mode = ?previous_mode,
                        error = %restore_error,
                        "failed to restore file system watch path after mode update failure"
                    );
                }
            }
            return Err(error.into());
        }
    //         info!(path = %path.display(), mode = ?next_mode, "updated file system watch path mode");
    } else {
        fs_log!(
            info,
            path = %path.display(),
            mode = ?next_mode,
            recursive_refs = next_refs.recursive,
            non_recursive_refs = next_refs.non_recursive,
            "reusing file system watch path"
        );
    }
    inner.path_refs.insert(path.to_path_buf(), next_refs);
    //     info!(
    //         path = %path.display(),
    //         recursive_refs = next_refs.recursive,
    //         non_recursive_refs = next_refs.non_recursive,
    //         effective_mode = ?next_refs.effective_mode(),
    //         "file system watch path ref added"
    //     );
    Ok(())
}

fn remove_path_ref(inner: &mut Inner, path: &Path, mode: WatchMode) {
    let Some(refs) = inner.path_refs.get_mut(path) else {
        fs_log!(info, path = %path.display(), "file system watch path ref remove skipped; path not registered");
        return;
    };

    let previous_mode = refs.effective_mode();
    match mode {
        WatchMode::Recursive => {
            refs.recursive = refs.recursive.saturating_sub(1);
        }
        WatchMode::NonRecursive => {
            refs.non_recursive = refs.non_recursive.saturating_sub(1);
        }
    }

    //     info!(
    //         path = %path.display(),
    //         recursive_refs = refs.recursive,
    //         non_recursive_refs = refs.non_recursive,
    //         "file system watch path ref removed"
    //     );

    if refs.total() == 0 {
        inner.path_refs.remove(path);
        //         info!(path = %path.display(), "unregistering file system watch path");
        if let Err(error) = inner.watcher.unwatch(path) {
            fs_log!(warn, path = %path.display(), error = %error, "failed to unwatch file system path");
        }
    //        else {
    //             info!(path = %path.display(), "unregistered file system watch path");
    //         }
    } else {
        let next_mode = refs.effective_mode();
        if previous_mode != next_mode {
            fs_log!(
                info,
                path = %path.display(),
                previous_mode = ?previous_mode,
                next_mode = ?next_mode,
                "downgrading file system watch path mode"
            );
            if let Err(error) = inner.watcher.unwatch(path) {
                fs_log!(warn, path = %path.display(), error = %error, "failed to unwatch file system path before mode downgrade");
            } else if let Err(error) = inner.watcher.watch(path, next_mode.into()) {
                fs_log!(warn, path = %path.display(), mode = ?next_mode, error = %error, "failed to rewatch file system path after mode downgrade");
            }
        }
    }
}

fn remove_all_path_refs(inner: &mut Inner, path: &Path) {
    if inner.path_refs.remove(path).is_none() {
        fs_log!(info, path = %path.display(), "file system watch path remove skipped; path not registered");
        return;
    }
    //     info!(path = %path.display(), "unregistering file system watch path");
    if let Err(error) = inner.watcher.unwatch(path) {
        fs_log!(warn, path = %path.display(), error = %error, "failed to unwatch file system path");
    }
    //     else {
    //         info!(path = %path.display(), "unregistered file system watch path");
    //     }
}

/// Register every directory the Git watch needs with the OS watcher.
///
/// Instead of one recursive watch on the repository root (which would register
/// every ignored directory such as `target/` or `node_modules/` and can exhaust
/// the Linux inotify watch limit), this registers a recursive watch on `.git`
/// plus one non-recursive watch per non-ignored working-tree directory.
fn add_git_watch_path_refs(inner: &mut Inner, watch: &mut GitWatch) -> Result<(), FsWatcherError> {
    // Roll back on failure (notably ENOSPC on Linux) so a partially applied
    // registration does not orphan path refs owned by no session.
    let mut added: Vec<(PathBuf, WatchMode)> = Vec::new();
    let result = (|| -> Result<(), FsWatcherError> {
        if !GIT_WATCH_TOP_LEVEL_DIRS {
            // macOS/Windows: one natively recursive watch on the repository
            // root covers the working tree and `.git`.
            add_path_ref(inner, &watch.repo_root, WatchMode::Recursive)?;
            added.push((watch.repo_root.clone(), WatchMode::Recursive));
            return Ok(());
        }
        let git_dir = watch.repo_root.join(".git");
        if git_dir.is_dir() {
            add_path_ref(inner, &git_dir, WatchMode::Recursive)?;
            added.push((git_dir, WatchMode::Recursive));
            watch.watching_git_dir = true;
        }
        add_path_ref(inner, &watch.repo_root, WatchMode::NonRecursive)?;
        added.push((watch.repo_root.clone(), WatchMode::NonRecursive));
        for dir in collect_top_level_watchable_dirs(&watch.repo_root, &watch.ignore_matcher) {
            add_path_ref(inner, &dir, WatchMode::Recursive)?;
            added.push((dir.clone(), WatchMode::Recursive));
            watch.watched_dirs.push(dir);
        }
        Ok(())
    })();
    if result.is_err() {
        for (path, mode) in added.into_iter().rev() {
            remove_path_ref(inner, &path, mode);
        }
        watch.watched_dirs.clear();
        watch.watching_git_dir = false;
    }
    result
}

fn remove_git_watch_path_refs(inner: &mut Inner, watch: &GitWatch) {
    if !GIT_WATCH_TOP_LEVEL_DIRS {
        remove_path_ref(inner, &watch.repo_root, WatchMode::Recursive);
        return;
    }
    if watch.watching_git_dir {
        remove_path_ref(inner, &watch.repo_root.join(".git"), WatchMode::Recursive);
    }
    remove_path_ref(inner, &watch.repo_root, WatchMode::NonRecursive);
    for dir in &watch.watched_dirs {
        remove_path_ref(inner, dir, WatchMode::Recursive);
    }
}

/// List the immediate subdirectories of `root` that are not ignored by the
/// matcher, skipping `.git` (which is watched separately). Used on Linux to
/// register one recursive watch per top-level directory; the recursive watch
/// tracks deeper subdirectories on its own, so no tree walk is needed.
fn collect_top_level_watchable_dirs(root: &Path, matcher: &IgnoreMatcher) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) => {
            fs_log!(
                debug,
                directory = %root.display(),
                error = %error,
                "failed to scan repository root for fs watcher directories"
            );
            return dirs;
        }
    };
    for entry in entries.flatten() {
        if entry.file_name() == ".git" {
            continue;
        }
        let is_dir = entry
            .file_type()
            .map(|file_type| file_type.is_dir())
            .unwrap_or(false);
        if !is_dir {
            continue;
        }
        let path = entry.path();
        if path_is_ignored(&path, matcher) {
            continue;
        }
        dirs.push(path);
    }
    dirs
}

/// Rebuild ignore matchers whose `.gitignore` (or `.git/info/exclude`)
/// changed. Returns the sessions whose matcher was rebuilt so their watched
/// directories can be re-synced (newly ignored directories dropped, newly
/// unignored directories picked up).
fn refresh_git_ignore_matchers(
    inner: &mut Inner,
    notification: &FileSystemNotification,
) -> Vec<SessionId> {
    let mut refreshed = Vec::new();
    for (session_id, state) in &mut inner.sessions {
        let Some(watch) = state.git.as_mut() else {
            continue;
        };
        let info_exclude = watch.repo_root.join(".git").join("info").join("exclude");
        let changed = notification.paths.iter().any(|path| {
            path_is_within(path, &watch.repo_root)
                && (path.file_name().is_some_and(|name| name == ".gitignore")
                    || paths_equal(path, &info_exclude))
        });
        if changed {
            watch.ignore_matcher = build_ignore_matcher(&watch.repo_root);
            refreshed.push(*session_id);
        }
    }
    refreshed
}

/// Keep each Git watch's per-directory OS watches in sync with the directory
/// tree: register directories that appeared (created or renamed in) and drop
/// refs for watched directories that vanished (deleted or renamed away).
fn reconcile_git_watch_dirs(inner: &mut Inner, notification: &FileSystemNotification) -> bool {
    let session_ids = inner.sessions.keys().copied().collect::<Vec<_>>();
    let mut changed = false;
    for session_id in session_ids {
        let Some(mut state) = inner.sessions.remove(&session_id) else {
            continue;
        };
        if let Some(watch) = state.git.as_mut() {
            changed |= reconcile_git_watch_dirs_for(inner, session_id, watch, notification);
        }
        inner.sessions.insert(session_id, state);
    }
    changed
}

fn reconcile_git_watch_dirs_for(
    inner: &mut Inner,
    session_id: SessionId,
    watch: &mut GitWatch,
    notification: &FileSystemNotification,
) -> bool {
    if !GIT_WATCH_TOP_LEVEL_DIRS {
        // macOS/Windows: the single recursive root watch already covers new
        // subdirectories and a late-appearing `.git`; nothing to reconcile.
        return false;
    }
    let mut changed = false;
    let git_dir = watch.repo_root.join(".git");

    // `.git` can appear after the watch was installed (for example `git init`
    // in an existing directory).
    if !watch.watching_git_dir && git_dir.is_dir() {
        match add_path_ref(inner, &git_dir, WatchMode::Recursive) {
            Ok(()) => {
                watch.watching_git_dir = true;
                changed = true;
            }
            Err(error) => fs_log!(
                warn,
                session_id = %session_id,
                path = %git_dir.display(),
                error = %error,
                "failed to watch .git directory"
            ),
        }
    }

    for path in &notification.paths {
        if !path_is_within(path, &watch.repo_root) || path_is_within(path, &git_dir) {
            continue;
        }

        // Dropped refs for watched top-level directories that vanished
        // (deleted or renamed away). Deeper directories are owned by the
        // recursive watch itself and need no bookkeeping here.
        let stale = watch
            .watched_dirs
            .iter()
            .filter(|dir| path_is_within(dir, path) && !dir.exists())
            .cloned()
            .collect::<Vec<_>>();
        for dir in stale {
            remove_path_ref(inner, &dir, WatchMode::Recursive);
            watch.watched_dirs.retain(|watched| watched != &dir);
            changed = true;
        }

        // Only top-level directories need explicit registration; the
        // recursive watch on each of them tracks deeper entries by itself.
        let is_top_level = path
            .parent()
            .is_some_and(|parent| paths_equal(parent, &watch.repo_root));
        if !is_top_level
            || !path.is_dir()
            || path_is_ignored(path, &watch.ignore_matcher)
            || watch.watched_dirs.contains(path)
        {
            continue;
        }
        // The recursive registration also covers a whole subtree that may
        // already exist inside the new directory (e.g. an unpacked archive).
        match add_path_ref(inner, path, WatchMode::Recursive) {
            Ok(()) => {
                watch.watched_dirs.push(path.clone());
                changed = true;
            }
            Err(error) => {
                fs_log!(
                    warn,
                    session_id = %session_id,
                    path = %path.display(),
                    error = %error,
                    "failed to watch new directory"
                );
            }
        }
    }
    changed
}

/// Re-sync a Git watch's per-directory OS watches after its ignore matcher was
/// rebuilt: drop refs for top-level directories that are now ignored (or
/// gone) and register directories that are no longer ignored. A no-op on
/// macOS/Windows, where the rebuilt matcher only affects event filtering.
fn resync_git_watch_dirs(inner: &mut Inner, session_id: SessionId) -> bool {
    if !GIT_WATCH_TOP_LEVEL_DIRS {
        return false;
    }
    let Some(mut state) = inner.sessions.remove(&session_id) else {
        return false;
    };
    let mut changed = false;
    if let Some(watch) = state.git.as_mut() {
        let desired = collect_top_level_watchable_dirs(&watch.repo_root, &watch.ignore_matcher);
        let stale = watch
            .watched_dirs
            .iter()
            .filter(|dir| !desired.contains(dir) || !dir.exists())
            .cloned()
            .collect::<Vec<_>>();
        for dir in stale {
            remove_path_ref(inner, &dir, WatchMode::Recursive);
            watch.watched_dirs.retain(|watched| watched != &dir);
            changed = true;
        }
        for dir in desired {
            if watch.watched_dirs.contains(&dir) {
                continue;
            }
            match add_path_ref(inner, &dir, WatchMode::Recursive) {
                Ok(()) => {
                    watch.watched_dirs.push(dir);
                    changed = true;
                }
                Err(error) => {
                    fs_log!(
                        warn,
                        session_id = %session_id,
                        path = %dir.display(),
                        error = %error,
                        "failed to watch newly unignored directory"
                    );
                    break;
                }
            }
        }
    }
    inner.sessions.insert(session_id, state);
    changed
}

fn reconcile_quick_commands_watch_dirs(inner: &mut Inner) -> bool {
    let changes = inner
        .sessions
        .iter()
        .filter_map(|(session_id, state)| {
            let watch = state.quick_commands.as_ref()?;
            let should_watch = watch.vterm_dir.is_dir();
            (watch.watching_vterm_dir != should_watch).then_some((*session_id, should_watch))
        })
        .collect::<Vec<_>>();

    for (session_id, should_watch) in &changes {
        let Some(mut state) = inner.sessions.remove(session_id) else {
            continue;
        };
        if let Some(watch) = state.quick_commands.as_mut() {
            if *should_watch {
                match add_path_ref(inner, &watch.vterm_dir, WatchMode::NonRecursive) {
                    Ok(()) => watch.watching_vterm_dir = true,
                    Err(error) => {
                        remove_path_ref(inner, &watch.vterm_dir, WatchMode::NonRecursive);
                        fs_log!(
                            warn,
                            session_id = %session_id,
                            path = %watch.vterm_dir.display(),
                            error = %error,
                            "failed to watch newly created .vterm directory"
                        );
                    }
                }
            } else {
                remove_path_ref(inner, &watch.vterm_dir, WatchMode::NonRecursive);
                watch.watching_vterm_dir = false;
            }
        }
        inner.sessions.insert(*session_id, state);
    }

    !changes.is_empty()
}

fn remove_quick_commands_path_refs(inner: &mut Inner, watch: &QuickCommandsWatch) {
    if watch.watching_vterm_dir {
        remove_path_ref(inner, &watch.vterm_dir, WatchMode::NonRecursive);
    }
    remove_path_ref(inner, &watch.repo_root, WatchMode::NonRecursive);
}

fn unwatch_session_paths(inner: &mut Inner, state: SessionWatchState) {
    if let Some(file) = state.file_content {
        remove_path_ref(inner, &file.watched_path, WatchMode::NonRecursive);
    }
    if let Some(git) = state.git {
        remove_git_watch_path_refs(inner, &git);
    }
    for directory in state.directories {
        remove_path_ref(inner, &directory.watched_path, WatchMode::NonRecursive);
    }
    if let Some(quick_commands) = state.quick_commands {
        remove_quick_commands_path_refs(inner, &quick_commands);
    }
}

fn sync_legacy_paths(legacy_state: &Arc<Mutex<WatchState>>, path_refs: &HashMap<PathBuf, PathRef>) {
    let mut state = legacy_state.lock().expect("fs watcher state poisoned");
    state.paths = path_refs
        .iter()
        .map(|(path, refs)| (path.clone(), refs.effective_mode()))
        .collect();
}

fn normalize_existing_path(path: &Path) -> Result<PathBuf, FsWatcherError> {
    if !path.exists() {
        return Err(FsWatcherError::PathNotFound(path.to_path_buf()));
    }

    Ok(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()))
}

fn normalize_requested_directories(paths: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();
    for path in paths {
        let path = path.trim().to_string();
        if path.is_empty() || !seen.insert(path.clone()) {
            continue;
        }
        normalized.push(path);
    }
    normalized
}

fn ensure_watch_path_allowed(
    path: &Path,
    _mode: WatchMode,
    _reject_system_directory: bool,
) -> Result<(), FsWatcherError> {
    if is_forbidden_watch_root(path) || is_system_directory(path) {
        return Err(FsWatcherError::ForbiddenWatchRoot(path.to_path_buf()));
    }
    Ok(())
}

fn is_forbidden_watch_root(path: &Path) -> bool {
    path.parent().is_none()
}

fn is_system_directory(path: &Path) -> bool {
    let components = path.components().collect::<Vec<_>>();

    #[cfg(unix)]
    {
        if is_unix_top_level_system_directory(&components) {
            return true;
        }
    }

    #[cfg(windows)]
    {
        if is_windows_filesystem_root(&components) {
            return true;
        }
    }

    false
}

#[cfg(unix)]
fn is_unix_top_level_system_directory(components: &[Component<'_>]) -> bool {
    let [Component::RootDir, Component::Normal(name)] = components else {
        return false;
    };
    matches!(
        name.to_str(),
        Some(
            "bin"
                | "boot"
                | "dev"
                | "etc"
                | "home"
                | "lib"
                | "lib64"
                | "media"
                | "mnt"
                | "opt"
                | "proc"
                | "root"
                | "run"
                | "sbin"
                | "srv"
                | "sys"
                | "tmp"
                | "usr"
                | "var"
        )
    )
}

#[cfg(windows)]
fn is_windows_filesystem_root(components: &[Component<'_>]) -> bool {
    let normal_components = components
        .iter()
        .filter(|component| matches!(component, Component::Normal(_)))
        .count();
    normal_components == 0
        && components
            .iter()
            .any(|component| matches!(component, Component::Prefix(_) | Component::RootDir))
}

fn build_ignore_matcher(root: &Path) -> IgnoreMatcher {
    let mut gitignores = Vec::new();
    let info_exclude = root.join(".git").join("info").join("exclude");
    if info_exclude.is_file() {
        add_scoped_gitignore(&mut gitignores, root, &info_exclude);
    }
    add_gitignore_files(&mut gitignores, root);
    IgnoreMatcher {
        root: root.to_path_buf(),
        gitignores,
    }
}

fn build_shallow_ignore_matcher(root: &Path) -> IgnoreMatcher {
    let mut gitignores = Vec::new();
    let gitignore = root.join(".gitignore");
    if gitignore.is_file() {
        add_scoped_gitignore(&mut gitignores, root, &gitignore);
    }
    IgnoreMatcher {
        root: root.to_path_buf(),
        gitignores,
    }
}

fn add_scoped_gitignore(gitignores: &mut Vec<ScopedGitignore>, root: &Path, path: &Path) {
    let mut builder = GitignoreBuilder::new(root);
    if let Some(error) = builder.add(path) {
        fs_log!(
            warn,
            path = %path.display(),
            error = %error,
            "failed to add fs watcher gitignore file"
        );
    }
    let gitignore = match builder.build() {
        Ok(gitignore) => gitignore,
        Err(error) => {
            fs_log!(
                warn,
                root = %root.display(),
                error = %error,
                "failed to build fs watcher gitignore matcher"
            );
            return;
        }
    };
    gitignores.push(ScopedGitignore {
        root: root.to_path_buf(),
        gitignore,
    });
}

fn add_gitignore_files(gitignores: &mut Vec<ScopedGitignore>, directory: &Path) {
    let gitignore = directory.join(".gitignore");
    if gitignore.is_file() {
        add_scoped_gitignore(gitignores, directory, &gitignore);
    }

    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            fs_log!(
                debug,
                directory = %directory.display(),
                error = %error,
                "failed to scan directory for fs watcher gitignore files"
            );
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name();
        if file_name == ".git" {
            continue;
        }

        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(error) => {
                fs_log!(
                    debug,
                    path = %path.display(),
                    error = %error,
                    "failed to read file type while scanning fs watcher gitignore files"
                );
                continue;
            }
        };

        if file_type.is_dir() {
            add_gitignore_files(gitignores, &path);
        }
    }
}

fn path_is_ignored(path: &Path, matcher: &IgnoreMatcher) -> bool {
    if event_path_is_git_metadata(path, &matcher.root) {
        return true;
    }

    if !path_is_within(path, &matcher.root) {
        return false;
    }

    for scoped in matcher.gitignores.iter().rev() {
        if !path_is_within(path, &scoped.root) {
            continue;
        }
        let matched = scoped
            .gitignore
            .matched_path_or_any_parents(path, path.is_dir());
        if !matched.is_none() {
            return matched.is_ignore();
        }
    }
    false
}

fn event_touches_exact_path(notification: &FileSystemNotification, target: &Path) -> bool {
    let target = comparable_path(target);
    notification
        .paths
        .iter()
        .any(|path| comparable_path(path) == target)
}

fn event_touches_quick_commands(
    notification: &FileSystemNotification,
    watch: &QuickCommandsWatch,
) -> bool {
    event_touches_exact_path(notification, &watch.commands_path)
        || event_touches_exact_path(notification, &watch.vterm_dir)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct GitWatchTrigger {
    /// Working-tree or index change → push the changed-files list.
    changed_files: bool,
    /// A commit moved a ref → push the commit history.
    history: bool,
    /// The branch set changed or another branch was checked out → push the
    /// branch list.
    branches: bool,
}

fn event_touches_git_watch(
    notification: &FileSystemNotification,
    watch: &GitWatch,
) -> GitWatchTrigger {
    use crate::fs_watcher::FileSystemEventKind as Kind;

    let git_dir = watch.repo_root.join(".git");
    let index = git_dir.join("index");
    let head = git_dir.join("HEAD");
    let commit_editmsg = git_dir.join("COMMIT_EDITMSG");
    let packed_refs = git_dir.join("packed-refs");
    let info_exclude = git_dir.join("info").join("exclude");
    let refs = git_dir.join("refs");
    let mut trigger = GitWatchTrigger::default();

    for path in &notification.paths {
        if !path_is_within(path, &watch.repo_root) {
            continue;
        }

        if path_is_within(path, &git_dir) {
            if is_git_lock_path(path) {
                continue;
            }
            if paths_equal(path, &index) || paths_equal(path, &info_exclude) {
                // Staging state only affects the changed-files list.
                trigger.changed_files = true;
            } else if paths_equal(path, &git_dir) && matches!(notification.kind, Kind::Removed) {
                // The repository itself vanished; every view must refresh.
                trigger.changed_files = true;
                trigger.history = true;
                trigger.branches = true;
            } else if paths_equal(path, &head) {
                // HEAD moves on checkout: the working tree, the current
                // branch and its history all change at once.
                trigger.changed_files = true;
                trigger.history = true;
                trigger.branches = true;
            } else if paths_equal(path, &commit_editmsg) {
                // Written on every commit (including --amend and merge).
                trigger.history = true;
            } else if paths_equal(path, &packed_refs) {
                trigger.branches = true;
            } else if path_is_within(path, &refs) {
                match notification.kind {
                    // A commit rewrites an existing ref in place.
                    Kind::Modified => trigger.history = true,
                    // Branch/tag creation and deletion.
                    Kind::Created | Kind::Removed => trigger.branches = true,
                    // Git replaces refs by renaming a temp file over them,
                    // which is ambiguous between "commit" and "new branch" —
                    // refresh both.
                    Kind::Renamed => {
                        trigger.history = true;
                        trigger.branches = true;
                    }
                    Kind::Other => {}
                }
            }
            // Anything else under `.git` (objects, reflogs, hooks, …) is
            // internal noise and never triggers a push.
            continue;
        }

        if !path_is_ignored(path, &watch.ignore_matcher) {
            trigger.changed_files = true;
        }
    }

    trigger
}

fn is_git_lock_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".lock"))
}

fn paths_equal(left: &Path, right: &Path) -> bool {
    comparable_path(left) == comparable_path(right)
}

fn event_touches_path(notification: &FileSystemNotification, target: &Path) -> bool {
    notification
        .paths
        .iter()
        .any(|path| event_touches_path_for_path(path, target))
}

fn event_touches_path_for_path(path: &Path, target: &Path) -> bool {
    let target = comparable_path(target);
    let path = comparable_path(path);
    path == target || path.starts_with(&(target + "\\"))
}

fn event_touches_directory_path(notification: &FileSystemNotification, target: &Path) -> bool {
    notification.paths.iter().any(|path| {
        event_touches_direct_child(path, target) && !event_path_is_git_metadata(path, target)
    })
}

fn event_touches_direct_child(path: &Path, directory: &Path) -> bool {
    if comparable_path(path) == comparable_path(directory) {
        return true;
    }
    path.parent()
        .is_some_and(|parent| comparable_path(parent) == comparable_path(directory))
}

fn event_path_is_git_metadata(path: &Path, watched_directory: &Path) -> bool {
    path_is_within(path, &watched_directory.join(".git"))
}

fn path_is_within(path: &Path, directory: &Path) -> bool {
    let path = comparable_path(path);
    let directory = comparable_path(directory);
    path == directory || path.starts_with(&(directory + "\\"))
}

fn comparable_path(path: &Path) -> String {
    let text = path.to_string_lossy().replace('/', "\\");
    let text = text
        .strip_prefix(r"\\?\UNC\")
        .map(|rest| format!(r"\\{}", rest))
        .or_else(|| text.strip_prefix(r"\\?\").map(str::to_string))
        .unwrap_or(text);
    let text = text.trim_end_matches('\\').to_string();
    if cfg!(windows) {
        text.to_lowercase()
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_missing_path() {
        let path = PathBuf::from("definitely-missing-fs-watcher-root");
        let error = normalize_existing_path(&path).unwrap_err();

        assert!(matches!(error, FsWatcherError::PathNotFound(_)));
    }

    #[test]
    fn failed_session_update_does_not_orphan_path_refs() {
        let session_id = SessionId::new();
        let root = std::env::temp_dir().join(format!(
            "vterm-fs-watcher-failed-update-{}-{session_id}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let root = normalize_existing_path(&root).unwrap();
        let missing = root.join("missing-file");
        let manager = FsWatcherManager::new().unwrap();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

        {
            let mut inner = manager.inner.lock().unwrap();
            add_path_ref(&mut inner, &root, WatchMode::NonRecursive).unwrap();
            inner.sessions.insert(
                session_id,
                SessionWatchState {
                    tx: tx.clone(),
                    file_content: None,
                    git: None,
                    git_request: None,
                    directories: vec![DirectoryWatch {
                        requested_path: root.display().to_string(),
                        watched_path: root.clone(),
                        ignore_matcher: build_shallow_ignore_matcher(&root),
                    }],
                    quick_commands: None,
                },
            );
        }

        let result = manager.apply_session_watch_update(
            session_id,
            SessionWatchUpdate {
                file_content: Some(Some(FileContentWatch {
                    request: FsWatchFileContent {
                        path: missing.display().to_string(),
                        max_size: 1024,
                    },
                    watched_path: missing.clone(),
                })),
                ..Default::default()
            },
            tx,
        );

        assert!(matches!(result, Err(FsWatcherError::Watcher(_))));
        let inner = manager.inner.lock().unwrap();
        assert!(!inner.path_refs.contains_key(&missing));
        assert!(!inner.path_refs.contains_key(&root));
        assert!(!inner.sessions.contains_key(&session_id));
        drop(inner);
        std::fs::remove_dir(root).unwrap();
    }

    /// Insert a session state with an installed git watch plus the recorded
    /// client desire — the state a `SetFsWatchers{git: Some}` leaves behind
    /// once its repository resolved successfully.
    fn install_git_watch(
        manager: &FsWatcherManager,
        session_id: SessionId,
        root: &Path,
        tx: ConnectionTx,
    ) {
        let mut inner = manager.inner.lock().unwrap();
        let mut watch = GitWatch {
            request: FsWatchGit { commit: None },
            repo_root: root.to_path_buf(),
            ignore_matcher: build_ignore_matcher(root),
            watched_dirs: Vec::new(),
            watching_git_dir: false,
        };
        add_git_watch_path_refs(&mut inner, &mut watch).unwrap();
        inner.sessions.insert(
            session_id,
            SessionWatchState {
                tx,
                file_content: None,
                git: Some(watch),
                git_request: Some(FsWatchGit { commit: None }),
                directories: Vec::new(),
                quick_commands: None,
            },
        );
    }

    fn temp_watch_root(label: &str, session_id: SessionId) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "vterm-fs-watcher-{label}-{}-{session_id}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        normalize_existing_path(&root).unwrap()
    }

    #[test]
    fn clearing_git_watch_keeps_desired_request_for_later_heal() {
        let session_id = SessionId::new();
        let root = temp_watch_root("git-clear", session_id);
        let manager = FsWatcherManager::new().unwrap();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        install_git_watch(&manager, session_id, &root, tx.clone());

        // What build_session_watch_update produces for SetFsWatchers
        // {git: Some} when the session path is not inside a repository: the
        // resolved watch is dropped while the client's desire is recorded.
        manager
            .apply_session_watch_update(
                session_id,
                SessionWatchUpdate {
                    git: Some(None),
                    git_request: Some(Some(FsWatchGit { commit: None })),
                    ..Default::default()
                },
                tx,
            )
            .unwrap();

        {
            let inner = manager.inner.lock().unwrap();
            let state = inner
                .sessions
                .get(&session_id)
                .expect("the desired request must keep the session state alive");
            assert!(state.git.is_none());
            assert!(state.git_request.is_some());
            assert!(!inner.path_refs.contains_key(&root));
        }
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn turning_git_watch_off_drops_desired_request() {
        let session_id = SessionId::new();
        let root = temp_watch_root("git-off", session_id);
        let manager = FsWatcherManager::new().unwrap();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        install_git_watch(&manager, session_id, &root, tx.clone());

        // The client asking for no git watch clears both the resolved watch
        // and the desire, leaving nothing to keep the session state alive.
        manager
            .apply_session_watch_update(
                session_id,
                SessionWatchUpdate {
                    git: Some(None),
                    git_request: Some(None),
                    ..Default::default()
                },
                tx,
            )
            .unwrap();

        {
            let inner = manager.inner.lock().unwrap();
            assert!(!inner.sessions.contains_key(&session_id));
            assert!(!inner.path_refs.contains_key(&root));
        }
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn comparable_path_strips_windows_extended_prefix() {
        let regular = PathBuf::from(r"D:\repo\src");
        let extended = PathBuf::from(r"\\?\D:\repo\src\main.rs");
        let notification = FileSystemNotification {
            watched_root: None,
            paths: vec![extended],
            kind: crate::fs_watcher::FileSystemEventKind::Removed,
            received_at: std::time::SystemTime::now(),
        };

        assert!(event_touches_path(&notification, &regular));
    }

    #[test]
    fn directory_watch_ignores_git_metadata_changes() {
        let regular = PathBuf::from(r"D:\repo");
        let git_event = PathBuf::from(r"\\?\D:\repo\.git\index.lock");
        let notification = FileSystemNotification {
            watched_root: None,
            paths: vec![git_event],
            kind: crate::fs_watcher::FileSystemEventKind::Modified,
            received_at: std::time::SystemTime::now(),
        };

        assert!(event_touches_path(&notification, &regular));
        assert!(!event_touches_directory_path(&notification, &regular));
    }

    #[test]
    fn directory_watch_still_triggers_gitignored_working_tree_changes() {
        let session_id = SessionId::new();
        let root = temp_watch_root("directory-ignored", session_id);
        std::fs::write(root.join(".gitignore"), "*.tmp\n").unwrap();
        let notification = |path| FileSystemNotification {
            watched_root: None,
            paths: vec![path],
            kind: crate::fs_watcher::FileSystemEventKind::Modified,
            received_at: std::time::SystemTime::now(),
        };

        let ignored_event = notification(root.join("scratch.tmp"));
        assert!(event_touches_directory_path(&ignored_event, &root));

        let git_event = notification(root.join(".git").join("index.lock"));
        assert!(!event_touches_directory_path(&git_event, &root));

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn git_watch_ignores_transient_metadata_locks() {
        let repo_root = PathBuf::from("repo");
        let git_dir = repo_root.join(".git");
        let refs_dir = git_dir.join("refs");
        let watch = GitWatch {
            request: FsWatchGit { commit: None },
            repo_root: repo_root.clone(),
            ignore_matcher: build_ignore_matcher(&repo_root),
            watched_dirs: Vec::new(),
            watching_git_dir: false,
        };
        let notification = |path| FileSystemNotification {
            watched_root: None,
            paths: vec![path],
            kind: crate::fs_watcher::FileSystemEventKind::Modified,
            received_at: std::time::SystemTime::now(),
        };

        assert_eq!(
            event_touches_git_watch(&notification(git_dir.join("index.lock")), &watch),
            GitWatchTrigger::default()
        );
        // A Git command that updates index/refs may also produce a directory
        // notification for `.git` itself. Treating that coarse notification as
        // a repository change feeds branch/log reads back into the watcher.
        assert_eq!(
            event_touches_git_watch(&notification(git_dir.clone()), &watch),
            GitWatchTrigger::default()
        );
        assert_eq!(
            event_touches_git_watch(
                &FileSystemNotification {
                    watched_root: None,
                    paths: vec![git_dir.clone()],
                    kind: crate::fs_watcher::FileSystemEventKind::Removed,
                    received_at: std::time::SystemTime::now(),
                },
                &watch,
            ),
            GitWatchTrigger {
                changed_files: true,
                history: true,
                branches: true,
            }
        );
        assert_eq!(
            event_touches_git_watch(
                &notification(refs_dir.join("heads").join("main.lock")),
                &watch
            ),
            GitWatchTrigger::default()
        );
    }

    #[test]
    fn git_watch_classifies_status_history_and_branch_events() {
        let repo_root = PathBuf::from("repo");
        let git_dir = repo_root.join(".git");
        let refs_dir = git_dir.join("refs");
        let watch = GitWatch {
            request: FsWatchGit { commit: None },
            repo_root: repo_root.clone(),
            ignore_matcher: build_ignore_matcher(&repo_root),
            watched_dirs: Vec::new(),
            watching_git_dir: false,
        };
        let notification = |path| FileSystemNotification {
            watched_root: None,
            paths: vec![path],
            kind: crate::fs_watcher::FileSystemEventKind::Modified,
            received_at: std::time::SystemTime::now(),
        };

        // Staging only affects the changed-files list.
        assert_eq!(
            event_touches_git_watch(&notification(git_dir.join("index")), &watch),
            GitWatchTrigger {
                changed_files: true,
                history: false,
                branches: false,
            }
        );
        // Checkout moves HEAD: files, history and branches all refresh.
        assert_eq!(
            event_touches_git_watch(&notification(git_dir.join("HEAD")), &watch),
            GitWatchTrigger {
                changed_files: true,
                history: true,
                branches: true,
            }
        );
        // A commit rewrites COMMIT_EDITMSG and the current branch ref.
        assert_eq!(
            event_touches_git_watch(&notification(git_dir.join("COMMIT_EDITMSG")), &watch),
            GitWatchTrigger {
                changed_files: false,
                history: true,
                branches: false,
            }
        );
        assert_eq!(
            event_touches_git_watch(&notification(refs_dir.join("heads").join("main")), &watch),
            GitWatchTrigger {
                changed_files: false,
                history: true,
                branches: false,
            }
        );
        // Branch/tag creation and deletion change the branch list.
        for kind in [
            crate::fs_watcher::FileSystemEventKind::Created,
            crate::fs_watcher::FileSystemEventKind::Removed,
        ] {
            assert_eq!(
                event_touches_git_watch(
                    &FileSystemNotification {
                        watched_root: None,
                        paths: vec![refs_dir.join("heads").join("feature")],
                        kind,
                        received_at: std::time::SystemTime::now(),
                    },
                    &watch,
                ),
                GitWatchTrigger {
                    changed_files: false,
                    history: false,
                    branches: true,
                }
            );
        }
        // Refs are replaced via rename, which is ambiguous between a commit
        // and a new branch: refresh both.
        assert_eq!(
            event_touches_git_watch(
                &FileSystemNotification {
                    watched_root: None,
                    paths: vec![refs_dir.join("heads").join("main")],
                    kind: crate::fs_watcher::FileSystemEventKind::Renamed,
                    received_at: std::time::SystemTime::now(),
                },
                &watch,
            ),
            GitWatchTrigger {
                changed_files: false,
                history: true,
                branches: true,
            }
        );
        assert_eq!(
            event_touches_git_watch(&notification(git_dir.join("packed-refs")), &watch),
            GitWatchTrigger {
                changed_files: false,
                history: false,
                branches: true,
            }
        );
        assert_eq!(
            event_touches_git_watch(&notification(repo_root.join("src").join("main.rs")), &watch),
            GitWatchTrigger {
                changed_files: true,
                history: false,
                branches: false,
            }
        );
        // Internal Git noise never triggers a push.
        assert_eq!(
            event_touches_git_watch(&notification(git_dir.join("objects/ab/cdef")), &watch),
            GitWatchTrigger::default()
        );
        assert_eq!(
            event_touches_git_watch(&notification(git_dir.join("logs").join("HEAD")), &watch),
            GitWatchTrigger::default()
        );
    }

    #[test]
    fn quick_commands_watch_matches_file_and_vterm_directory_events() {
        let repo_root = PathBuf::from(r"D:\repo");
        let vterm_dir = repo_root.join(".vterm");
        let watch = QuickCommandsWatch {
            commands_path: vterm_dir.join("commands.json"),
            repo_root,
            vterm_dir: vterm_dir.clone(),
            watching_vterm_dir: true,
        };
        let notification = |path| FileSystemNotification {
            watched_root: None,
            paths: vec![path],
            kind: crate::fs_watcher::FileSystemEventKind::Modified,
            received_at: std::time::SystemTime::now(),
        };

        assert!(event_touches_quick_commands(
            &notification(watch.commands_path.clone()),
            &watch
        ));
        assert!(event_touches_quick_commands(
            &notification(vterm_dir),
            &watch
        ));
        assert!(!event_touches_quick_commands(
            &notification(watch.repo_root.join("src/main.rs")),
            &watch
        ));
    }

    #[test]
    fn quick_commands_watch_follows_vterm_directory_lifecycle() {
        let session_id = SessionId::new();
        let repo_root = std::env::temp_dir().join(format!(
            "vterm-quick-commands-watch-{}-{}",
            std::process::id(),
            session_id
        ));
        std::fs::create_dir_all(&repo_root).unwrap();
        let repo_root = normalize_existing_path(&repo_root).unwrap();
        let vterm_dir = repo_root.join(".vterm");
        let manager = FsWatcherManager::new().unwrap();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

        {
            let mut inner = manager.inner.lock().unwrap();
            add_path_ref(&mut inner, &repo_root, WatchMode::NonRecursive).unwrap();
            inner.sessions.insert(
                session_id,
                SessionWatchState {
                    tx,
                    file_content: None,
                    git: None,
                    git_request: None,
                    directories: Vec::new(),
                    quick_commands: Some(QuickCommandsWatch {
                        commands_path: vterm_dir.join("commands.json"),
                        repo_root: repo_root.clone(),
                        vterm_dir: vterm_dir.clone(),
                        watching_vterm_dir: false,
                    }),
                },
            );
        }

        std::fs::create_dir(&vterm_dir).unwrap();
        {
            let mut inner = manager.inner.lock().unwrap();
            assert!(reconcile_quick_commands_watch_dirs(&mut inner));
            assert!(inner.path_refs.contains_key(&vterm_dir));
            assert!(
                inner.sessions[&session_id]
                    .quick_commands
                    .as_ref()
                    .unwrap()
                    .watching_vterm_dir
            );
        }

        std::fs::remove_dir(&vterm_dir).unwrap();
        {
            let mut inner = manager.inner.lock().unwrap();
            assert!(reconcile_quick_commands_watch_dirs(&mut inner));
            assert!(!inner.path_refs.contains_key(&vterm_dir));
            assert!(
                !inner.sessions[&session_id]
                    .quick_commands
                    .as_ref()
                    .unwrap()
                    .watching_vterm_dir
            );
        }

        manager.remove_session(session_id);
        std::fs::remove_dir(repo_root).unwrap();
    }

    #[test]
    fn ignore_matcher_ignores_gitignored_paths() {
        let root = std::env::temp_dir().join(format!(
            "vterm-fs-watcher-ignore-test-{}",
            std::process::id()
        ));
        let ignored_dir = root.join("target");
        let nested_dir = root.join("nested");
        std::fs::create_dir_all(&ignored_dir).unwrap();
        std::fs::create_dir_all(&nested_dir).unwrap();
        std::fs::write(root.join(".gitignore"), "target/\n*.tmp\n").unwrap();
        std::fs::write(nested_dir.join(".gitignore"), "/nested-only.txt\n").unwrap();

        let matcher = build_ignore_matcher(&root);
        assert!(path_is_ignored(&ignored_dir.join("debug.log"), &matcher));
        assert!(path_is_ignored(&root.join("scratch.tmp"), &matcher));
        assert!(path_is_ignored(
            &nested_dir.join("nested-only.txt"),
            &matcher
        ));
        assert!(!path_is_ignored(&root.join("nested-only.txt"), &matcher));
        assert!(!path_is_ignored(&root.join("src/main.rs"), &matcher));

        let watch = GitWatch {
            request: FsWatchGit { commit: None },
            repo_root: root.clone(),
            ignore_matcher: matcher,
            watched_dirs: Vec::new(),
            watching_git_dir: false,
        };
        let notification = |path| FileSystemNotification {
            watched_root: Some(root.clone()),
            paths: vec![path],
            kind: crate::fs_watcher::FileSystemEventKind::Created,
            received_at: std::time::SystemTime::now(),
        };
        assert_eq!(
            event_touches_git_watch(&notification(ignored_dir.join("debug.log")), &watch),
            GitWatchTrigger::default()
        );
        assert!(
            event_touches_git_watch(&notification(root.join("src/main.rs")), &watch).changed_files
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn applying_git_watch_reuses_same_root_and_replaces_changed_root() {
        let test_id = SessionId::new();
        let base = std::env::temp_dir().join(format!(
            "vterm-git-watch-apply-{}-{}",
            std::process::id(),
            test_id
        ));
        let first_root = base.join("first");
        let second_root = base.join("second");
        std::fs::create_dir_all(&first_root).unwrap();
        std::fs::create_dir_all(&second_root).unwrap();
        let first_root = normalize_existing_path(&first_root).unwrap();
        let second_root = normalize_existing_path(&second_root).unwrap();
        let manager = FsWatcherManager::new().unwrap();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

        {
            let mut inner = manager.inner.lock().unwrap();
            let mut watch = GitWatch {
                request: FsWatchGit { commit: None },
                repo_root: first_root.clone(),
                ignore_matcher: build_ignore_matcher(&first_root),
                watched_dirs: Vec::new(),
                watching_git_dir: false,
            };
            add_git_watch_path_refs(&mut inner, &mut watch).unwrap();
            inner.sessions.insert(
                test_id,
                SessionWatchState {
                    tx: tx.clone(),
                    file_content: None,
                    git: Some(watch),
                    git_request: None,
                    directories: Vec::new(),
                    quick_commands: None,
                },
            );
        }

        manager
            .apply_session_watch_update(
                test_id,
                SessionWatchUpdate {
                    git: Some(Some(GitWatch {
                        request: FsWatchGit { commit: None },
                        repo_root: first_root.clone(),
                        ignore_matcher: build_ignore_matcher(&first_root),
                        watched_dirs: Vec::new(),
                        watching_git_dir: false,
                    })),
                    ..Default::default()
                },
                tx.clone(),
            )
            .unwrap();
        {
            let inner = manager.inner.lock().unwrap();
            if GIT_WATCH_TOP_LEVEL_DIRS {
                assert_eq!(inner.path_refs[&first_root].non_recursive, 1);
                assert_eq!(inner.path_refs[&first_root].recursive, 0);
            } else {
                assert_eq!(inner.path_refs[&first_root].recursive, 1);
                assert_eq!(inner.path_refs[&first_root].non_recursive, 0);
            }
            // Re-applying the same root keeps the existing watched-directory
            // bookkeeping instead of registering the OS watches again. The
            // test repository has no subdirectories, so there is nothing to
            // track beyond the root.
            assert!(inner.sessions[&test_id]
                .git
                .as_ref()
                .unwrap()
                .watched_dirs
                .is_empty());
        }

        manager
            .apply_session_watch_update(
                test_id,
                SessionWatchUpdate {
                    git: Some(Some(GitWatch {
                        request: FsWatchGit { commit: None },
                        repo_root: second_root.clone(),
                        ignore_matcher: build_ignore_matcher(&second_root),
                        watched_dirs: Vec::new(),
                        watching_git_dir: false,
                    })),
                    ..Default::default()
                },
                tx.clone(),
            )
            .unwrap();
        {
            let inner = manager.inner.lock().unwrap();
            assert!(!inner.path_refs.contains_key(&first_root));
            if GIT_WATCH_TOP_LEVEL_DIRS {
                assert_eq!(inner.path_refs[&second_root].non_recursive, 1);
                assert_eq!(inner.path_refs[&second_root].recursive, 0);
            } else {
                assert_eq!(inner.path_refs[&second_root].recursive, 1);
                assert_eq!(inner.path_refs[&second_root].non_recursive, 0);
            }
        }

        manager
            .apply_session_watch_update(
                test_id,
                SessionWatchUpdate {
                    git: Some(None),
                    ..Default::default()
                },
                tx,
            )
            .unwrap();
        assert!(!manager
            .inner
            .lock()
            .unwrap()
            .path_refs
            .contains_key(&second_root));

        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn git_watch_skips_ignored_directories() {
        let test_id = SessionId::new();
        let root = std::env::temp_dir().join(format!(
            "vterm-git-watch-prune-{}-{}",
            std::process::id(),
            test_id
        ));
        std::fs::create_dir_all(root.join(".git")).unwrap();
        std::fs::create_dir_all(root.join("target").join("debug")).unwrap();
        std::fs::create_dir_all(root.join("src").join("nested")).unwrap();
        std::fs::write(root.join(".gitignore"), "target/\n").unwrap();
        let root = normalize_existing_path(&root).unwrap();
        let manager = FsWatcherManager::new().unwrap();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

        let mut watch = GitWatch {
            request: FsWatchGit { commit: None },
            repo_root: root.clone(),
            ignore_matcher: build_ignore_matcher(&root),
            watched_dirs: Vec::new(),
            watching_git_dir: false,
        };
        {
            let mut inner = manager.inner.lock().unwrap();
            add_git_watch_path_refs(&mut inner, &mut watch).unwrap();
            if GIT_WATCH_TOP_LEVEL_DIRS {
                // Linux: `.git` recursive, root non-recursive, one recursive
                // watch per non-ignored top-level directory. The recursive
                // `src` watch covers `src/nested` without its own ref, and
                // the ignored `target` tree is never registered.
                assert!(watch.watching_git_dir);
                assert_eq!(inner.path_refs[&root.join(".git")].recursive, 1);
                assert_eq!(inner.path_refs[&root].non_recursive, 1);
                assert_eq!(inner.path_refs[&root].recursive, 0);
                assert_eq!(inner.path_refs[&root.join("src")].recursive, 1);
                assert!(!inner
                    .path_refs
                    .contains_key(&root.join("src").join("nested")));
            } else {
                // macOS/Windows: a single native recursive watch on the root.
                assert!(!watch.watching_git_dir);
                assert_eq!(inner.path_refs[&root].recursive, 1);
                assert_eq!(inner.path_refs.len(), 1);
            }
            assert!(!inner.path_refs.contains_key(&root.join("target")));
            assert!(!inner
                .path_refs
                .contains_key(&root.join("target").join("debug")));
            inner.sessions.insert(
                test_id,
                SessionWatchState {
                    tx,
                    file_content: None,
                    git: Some(watch),
                    git_request: None,
                    directories: Vec::new(),
                    quick_commands: None,
                },
            );
        }

        // A new top-level directory needs its own recursive watch on Linux;
        // elsewhere the root watch already covers it.
        let created = root.join("fresh");
        let notification = FileSystemNotification {
            watched_root: None,
            paths: vec![created.clone()],
            kind: crate::fs_watcher::FileSystemEventKind::Created,
            received_at: std::time::SystemTime::now(),
        };
        std::fs::create_dir(&created).unwrap();
        {
            let mut inner = manager.inner.lock().unwrap();
            assert_eq!(
                reconcile_git_watch_dirs(&mut inner, &notification),
                GIT_WATCH_TOP_LEVEL_DIRS
            );
            if GIT_WATCH_TOP_LEVEL_DIRS {
                assert_eq!(inner.path_refs[&created].recursive, 1);
            } else {
                assert!(!inner.path_refs.contains_key(&created));
            }
        }
        std::fs::remove_dir(&created).unwrap();
        {
            let mut inner = manager.inner.lock().unwrap();
            assert_eq!(
                reconcile_git_watch_dirs(&mut inner, &notification),
                GIT_WATCH_TOP_LEVEL_DIRS
            );
            assert!(!inner.path_refs.contains_key(&created));
        }

        // A new nested directory is tracked by the recursive watch on its
        // top-level ancestor (Linux) or the root watch (macOS/Windows); it
        // never gets its own ref.
        let nested = root.join("src").join("fresh");
        let nested_notification = FileSystemNotification {
            watched_root: None,
            paths: vec![nested.clone()],
            kind: crate::fs_watcher::FileSystemEventKind::Created,
            received_at: std::time::SystemTime::now(),
        };
        std::fs::create_dir(&nested).unwrap();
        {
            let mut inner = manager.inner.lock().unwrap();
            assert!(!reconcile_git_watch_dirs(&mut inner, &nested_notification));
            assert!(!inner.path_refs.contains_key(&nested));
        }
        std::fs::remove_dir(&nested).unwrap();

        manager.remove_session(test_id);
        assert!(manager.inner.lock().unwrap().path_refs.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn normalizes_directory_requests() {
        assert_eq!(
            normalize_requested_directories(vec![
                "".to_string(),
                " src ".to_string(),
                "src".to_string(),
                "tests".to_string(),
            ]),
            vec!["src".to_string(), "tests".to_string()]
        );
    }
}
