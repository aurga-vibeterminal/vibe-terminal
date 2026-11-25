use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::time::Duration;
use tracing::{debug, info, warn};
use vterm_common::{RecentProjectDirectory, SessionId};
use vterm_core::protocol::Message;

use crate::fs_watcher::{FileSystemEventKind, FileSystemNotification, FsWatcherManager, WatchMode};
use crate::session::recent_projects::RecentProjectScanState;

use super::session_manager::SessionManager;
impl SessionManager {
    pub fn start_ai_conversation_global_watch(&self, watcher: Arc<FsWatcherManager>) {
        tokio::spawn(global_watch_loop(self.clone(), watcher));
    }

    pub async fn list_ai_conversations(
        &self,
        session_id: SessionId,
    ) -> Vec<vterm_common::AiConversationInfo> {
        let Some(session) = self.sessions.read().await.get(&session_id).cloned() else {
            return Vec::new();
        };
        let session = session.lock().await;
        let app = session.app.clone();
        let Some(working_path) = session.current_working_path.clone() else {
            return Vec::new();
        };
        drop(session);
        tokio::task::spawn_blocking(move || {
            super::conversation_history::list_ai_conversations(app, &working_path)
        })
        .await
        .unwrap_or_default()
    }

    pub async fn read_ai_conversation(
        &self,
        session_id: SessionId,
        conversation_id: String,
    ) -> Vec<vterm_common::AiConversationMessage> {
        let Some(session) = self.sessions.read().await.get(&session_id).cloned() else {
            return Vec::new();
        };
        let session = session.lock().await;
        let app = session.app.clone();
        let Some(working_path) = session.current_working_path.clone() else {
            return Vec::new();
        };
        drop(session);
        self.active_ai_conversations
            .lock()
            .await
            .insert(session_id, conversation_id.clone());
        info!(
            session_id = %session_id,
            app = %app.0,
            conversation_id = %conversation_id,
            working_path = %working_path,
            "activated ai conversation"
        );
        let messages = tokio::task::spawn_blocking(move || {
            super::conversation_history::read_ai_conversation(&app, &working_path, &conversation_id)
        })
        .await
        .unwrap_or_default();
        // The client receives the full snapshot via `AiConversationMessages`; seed
        // the message-count baseline so subsequent file-watch refreshes can send
        // incremental updates instead of always re-pushing everything.
        self.ai_conversation_message_counts
            .lock()
            .await
            .insert(session_id, messages.len());
        messages
    }

    pub async fn stop_watching_ai_conversation(
        &self,
        session_id: SessionId,
        conversation_id: &str,
    ) {
        let mut active = self.active_ai_conversations.lock().await;
        if active.get(&session_id).map(String::as_str) == Some(conversation_id) {
            active.remove(&session_id);
            drop(active);
            self.ai_conversation_message_counts
                .lock()
                .await
                .remove(&session_id);
            debug!(
                session_id = %session_id,
                conversation_id,
                "stopped watching ai conversation"
            );
        }
    }

    pub(super) async fn clear_active_ai_conversation(&self, session_id: SessionId) {
        let mut active = self.active_ai_conversations.lock().await;
        if let Some(conversation_id) = active.remove(&session_id) {
            self.ai_conversation_message_counts
                .lock()
                .await
                .remove(&session_id);
            debug!(
                session_id = %session_id,
                conversation_id = %conversation_id,
                "cleared active ai conversation"
            );
        }
    }
}

pub(super) async fn global_watch_loop(
    session_manager: SessionManager,
    fs_watcher_manager: Arc<FsWatcherManager>,
) {
    // Watch roots paired with the mode used to register them; `remove_watch_ref`
    // must receive the same mode as `add_watch_ref` or the per-mode ref counter
    // is left unbalanced (see FsWatcherManager path-ref accounting).
    let mut watch_refs = Vec::<(PathBuf, WatchMode)>::new();
    for root in super::conversation_history::ai_conversation_global_watch_roots() {
        let mode = if root.file_name().and_then(|name| name.to_str()) == Some("sessions")
            || root.file_name().and_then(|name| name.to_str()) == Some("projects")
        {
            WatchMode::Recursive
        } else {
            WatchMode::NonRecursive
        };
        match fs_watcher_manager.add_watch_ref(&root, mode) {
            Ok(path) => watch_refs.push((path, mode)),
            Err(error) => debug!(
                path = %root.display(),
                error = %error,
                "skipping global ai conversation watch root"
            ),
        }
    }

    refresh_recent_projects_from_ai_history(&session_manager, false).await;
    let mut rx = fs_watcher_manager.subscribe();
    loop {
        let pending = match next_ai_conversation_watch_paths(&mut rx, &watch_refs).await {
            Some(pending) if pending.paths.is_empty() => continue,
            Some(pending) => pending,
            None => break,
        };

        let pending = match debounce_ai_conversation_watch(&mut rx, &watch_refs, pending).await {
            Some(pending) => pending,
            None => break,
        };

        // info!(
        //     changed_paths = ?pending_paths,
        //     "refreshing ai conversations after debounced history changes"
        // );
        if pending.recent_projects_may_have_changed
            && ai_history_changes_affect_recent_projects(&pending.paths)
        {
            refresh_recent_projects_from_ai_history(&session_manager, true).await;
        }
        refresh_ai_conversations_for_viewers(&session_manager).await;
    }

    // Release the watch refs so the FsWatcherManager ref accounting stays
    // balanced. Today this is only reached if the broadcast channel closes
    // (which doesn't happen during normal process lifetime), but wiring it up
    // makes the accounting correct and supports a future shutdown path.
    for (path, mode) in &watch_refs {
        fs_watcher_manager.remove_watch_ref(path, *mode);
    }
}

/// Return whether the debounced changes affect a data source used to build the
/// recent-project list. Conversation rollouts and UI/cache files must not
/// cause a project-list broadcast.
fn ai_history_changes_affect_recent_projects(changed_paths: &[String]) -> bool {
    if changed_paths.iter().any(|path| path == "<lagged>") {
        // A lagged watcher cannot identify the changed file, so reconcile the
        // small project list once rather than risk keeping stale entries.
        return true;
    }

    let Some(home) = super::conversation_history::user_home_dir() else {
        return false;
    };
    let project_sources = [
        home.join(".claude").join("history.jsonl"),
        home.join(".codex").join("state_5.sqlite"),
        home.join(".zcode").join("cli").join("db").join("db.sqlite"),
        home.join(".kimi-code").join("session_index.jsonl"),
        home.join(".pi").join("agent").join("trust.json"),
    ];
    let pi_sessions = home.join(".pi").join("agent").join("sessions");
    let qwen_projects = super::recent_projects::qwen_projects_root(&home);
    let dsh_sessions = super::recent_projects::dsh_sessions_root(&home);
    let opencode_data = super::recent_projects::opencode_data_dir(&home);
    let opencode_databases = super::recent_projects::opencode_database_paths(&home);

    changed_paths.iter().any(|changed_path| {
        let changed_path = Path::new(changed_path);
        if ai_path_touches_target(changed_path, &pi_sessions)
            || ai_path_touches_target(changed_path, &qwen_projects)
            || ai_path_touches_target(changed_path, &dsh_sessions)
            || ai_path_touches_target(changed_path, &opencode_data)
            || opencode_databases.iter().any(|database| {
                let changed = comparable_ai_watch_path(changed_path);
                let database = comparable_ai_watch_path(database);
                changed == database || changed.starts_with(&(database + "-"))
            })
        {
            return true;
        }
        let changed_path = comparable_ai_watch_path(changed_path);
        project_sources
            .iter()
            .any(|source| changed_path == comparable_ai_watch_path(source))
    })
}
async fn refresh_recent_projects_from_ai_history(session_manager: &SessionManager, log_diff: bool) {
    let directories =
        tokio::task::spawn_blocking(super::recent_projects::scan_recent_project_directories)
            .await
            .unwrap_or_default();
    let previous = session_manager
        .recent_project_directories
        .read()
        .map(|guard| guard.clone())
        .unwrap_or_default();
    if previous == directories {
        debug!(
            count = directories.len(),
            "recent project directories unchanged after ai history refresh"
        );
        return;
    }

    // Recency timestamps advance for every prompt in several agents. Keep the
    // in-memory timestamps fresh, but only publish when the visible list of
    // project paths (including its ordering) actually changed.
    let visible_list_changed = recent_project_paths_changed(&previous, &directories);
    if log_diff && visible_list_changed {
        let diff = recent_project_directory_diff(&previous, &directories);
        info!(
            added = ?diff.added,
            removed = ?diff.removed,
            updated = ?diff.updated,
            "recent project directories changed after ai history refresh"
        );
    }

    if let Ok(mut guard) = session_manager.recent_project_directories.write() {
        *guard = directories.clone();
    }
    if !visible_list_changed {
        debug!(
            count = directories.len(),
            "recent project paths unchanged after ai history refresh"
        );
        return;
    }
    let _ = session_manager
        .recent_project_scan_state
        .send(RecentProjectScanState::Ready(directories));
}

async fn refresh_ai_conversations_for_viewers(session_manager: &SessionManager) {
    let sessions = {
        let sessions = session_manager.sessions.read().await;
        sessions
            .iter()
            .map(|(session_id, session)| (*session_id, session.clone()))
            .collect::<Vec<_>>()
    };

    for (session_id, session) in sessions {
        let (app, working_path, viewers) = {
            let session = session.lock().await;
            (
                session.app.clone(),
                session.current_working_path.clone(),
                session.viewers.clone(),
            )
        };
        if viewers.is_empty()
            || !(app.0 == "claude"
                || app.0 == "codex"
                || app.0 == "kimi"
                || app.0 == "pi"
                || app.0 == "opencode"
                || app.0 == "dsh")
        {
            continue;
        }
        let Some(working_path) = working_path else {
            continue;
        };

        let active_conversation = session_manager
            .active_ai_conversations
            .lock()
            .await
            .get(&session_id)
            .cloned();
        let Some(conversation_id) = active_conversation else {
            continue;
        };
        let messages = tokio::task::spawn_blocking({
            let app = app.clone();
            let working_path = working_path.clone();
            let conversation_id = conversation_id.clone();
            move || {
                super::conversation_history::read_ai_conversation(
                    &app,
                    &working_path,
                    &conversation_id,
                )
            }
        })
        .await
        .unwrap_or_default();

        // Send only what changed. The client applies updates as a splice from
        // `start_index` (truncate then extend), so:
        //   - grown since baseline -> append the tail
        //   - shrank/reordered     -> full replace from index 0
        //   - unchanged            -> skip (no message, no churn)
        // Without a baseline, fall back to a full push and record one.
        let mut counts = session_manager.ai_conversation_message_counts.lock().await;
        let previous = counts.get(&session_id).copied();
        let count = messages.len();
        let update = match previous {
            Some(prev) if count > prev => Some((prev, messages[prev..].to_vec())),
            Some(prev) if count < prev => Some((0, messages.clone())),
            None => Some((0, messages.clone())),
            _ => None,
        };
        if let Some((start_index, update_messages)) = update {
            counts.insert(session_id, count);
            drop(counts);
            for viewer in &viewers {
                let _ = viewer.send(Message::AiConversationMessagesUpdate {
                    session_id,
                    conversation_id: conversation_id.clone(),
                    start_index,
                    messages: update_messages.clone(),
                });
            }
        }
    }
}

const AI_CONVERSATION_REFRESH_QUIET: Duration = Duration::from_secs(1);
const AI_CONVERSATION_REFRESH_MAX: Duration = Duration::from_secs(2);

struct PendingAiHistoryChanges {
    paths: Vec<String>,
    recent_projects_may_have_changed: bool,
}

fn event_may_change_recent_projects(kind: FileSystemEventKind) -> bool {
    matches!(
        kind,
        FileSystemEventKind::Created | FileSystemEventKind::Removed | FileSystemEventKind::Renamed
    )
}

async fn next_ai_conversation_watch_paths(
    rx: &mut tokio::sync::broadcast::Receiver<FileSystemNotification>,
    watch_refs: &[(PathBuf, WatchMode)],
) -> Option<PendingAiHistoryChanges> {
    loop {
        match rx.recv().await {
            Ok(notification) => {
                let paths = ai_notification_touched_paths(&notification, watch_refs);
                if !paths.is_empty() {
                    return Some(PendingAiHistoryChanges {
                        paths,
                        recent_projects_may_have_changed: event_may_change_recent_projects(
                            notification.kind,
                        ),
                    });
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                warn!(
                    skipped,
                    "global ai conversation watcher lagged; refreshing full state"
                );
                return Some(PendingAiHistoryChanges {
                    paths: vec!["<lagged>".to_string()],
                    recent_projects_may_have_changed: true,
                });
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
        }
    }
}

async fn debounce_ai_conversation_watch(
    rx: &mut tokio::sync::broadcast::Receiver<FileSystemNotification>,
    watch_refs: &[(PathBuf, WatchMode)],
    initial: PendingAiHistoryChanges,
) -> Option<PendingAiHistoryChanges> {
    let mut pending_paths = initial.paths.into_iter().collect::<HashSet<_>>();
    let mut recent_projects_may_have_changed = initial.recent_projects_may_have_changed;
    let max_deadline = tokio::time::Instant::now() + AI_CONVERSATION_REFRESH_MAX;
    let mut quiet_deadline = tokio::time::Instant::now() + AI_CONVERSATION_REFRESH_QUIET;

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(notification) => {
                        let paths = ai_notification_touched_paths(&notification, watch_refs);
                        if !paths.is_empty() {
                            pending_paths.extend(paths);
                            recent_projects_may_have_changed |=
                                event_may_change_recent_projects(notification.kind);
                            quiet_deadline = tokio::time::Instant::now() + AI_CONVERSATION_REFRESH_QUIET;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!(
                            skipped,
                            "global ai conversation watcher lagged while debouncing; refreshing full state"
                        );
                        pending_paths.insert("<lagged>".to_string());
                        recent_projects_may_have_changed = true;
                        quiet_deadline = tokio::time::Instant::now() + AI_CONVERSATION_REFRESH_QUIET;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
                }
            }
            _ = tokio::time::sleep_until(quiet_deadline) => {
                let mut paths = pending_paths.into_iter().collect::<Vec<_>>();
                paths.sort();
                return Some(PendingAiHistoryChanges {
                    paths,
                    recent_projects_may_have_changed,
                });
            }
            _ = tokio::time::sleep_until(max_deadline) => {
                let mut paths = pending_paths.into_iter().collect::<Vec<_>>();
                paths.sort();
                return Some(PendingAiHistoryChanges {
                    paths,
                    recent_projects_may_have_changed,
                });
            }
        }
    }
}

fn ai_notification_touched_paths(
    notification: &FileSystemNotification,
    watch_refs: &[(PathBuf, WatchMode)],
) -> Vec<String> {
    let mut paths = notification
        .paths
        .iter()
        .filter(|path| !is_ignored_ai_history_event_path(path))
        .filter(|path| {
            watch_refs
                .iter()
                .any(|(watch_ref, _)| ai_path_touches_target(path, watch_ref))
        })
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    paths
}

fn is_ignored_ai_history_event_path(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let name = name.to_ascii_lowercase();
    name.ends_with(".sqlite-shm")
        || name.ends_with(".sqlite-wal")
        || name.ends_with(".sqlite-journal")
}

struct RecentProjectDirectoryDiff {
    added: Vec<RecentProjectDirectory>,
    removed: Vec<RecentProjectDirectory>,
    updated: Vec<(RecentProjectDirectory, RecentProjectDirectory)>,
}

fn recent_project_paths_changed(
    previous: &[RecentProjectDirectory],
    next: &[RecentProjectDirectory],
) -> bool {
    previous.len() != next.len()
        || previous
            .iter()
            .zip(next)
            .any(|(previous, next)| previous.path != next.path)
}

fn recent_project_directory_diff(
    previous: &[RecentProjectDirectory],
    next: &[RecentProjectDirectory],
) -> RecentProjectDirectoryDiff {
    let previous_by_path = previous
        .iter()
        .map(|directory| (directory.path.as_str(), directory))
        .collect::<HashMap<_, _>>();
    let next_by_path = next
        .iter()
        .map(|directory| (directory.path.as_str(), directory))
        .collect::<HashMap<_, _>>();

    let added = next
        .iter()
        .filter(|directory| !previous_by_path.contains_key(directory.path.as_str()))
        .cloned()
        .collect();
    let removed = previous
        .iter()
        .filter(|directory| !next_by_path.contains_key(directory.path.as_str()))
        .cloned()
        .collect();
    let updated = next
        .iter()
        .filter_map(|directory| {
            previous_by_path
                .get(directory.path.as_str())
                .filter(|previous| {
                    previous.accessed_at_unix_secs != directory.accessed_at_unix_secs
                })
                .map(|previous| ((*previous).clone(), directory.clone()))
        })
        .collect();

    RecentProjectDirectoryDiff {
        added,
        removed,
        updated,
    }
}
fn ai_path_touches_target(path: &Path, target: &Path) -> bool {
    let target = comparable_ai_watch_path(target);
    let path = comparable_ai_watch_path(path);
    path == target || path.starts_with(&(target + "\\"))
}

fn comparable_ai_watch_path(path: &Path) -> String {
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

    fn recent(path: &str, accessed_at_unix_secs: u64) -> RecentProjectDirectory {
        RecentProjectDirectory {
            path: path.to_string(),
            accessed_at_unix_secs,
        }
    }

    #[test]
    fn dsh_log_changes_trigger_recent_project_scan() {
        let Some(home) = super::super::conversation_history::user_home_dir() else {
            return;
        };
        let path = super::super::recent_projects::dsh_sessions_root(&home)
            .join("--project--")
            .join("dsh-id")
            .join("session.v4.jsonl.zstd");
        assert!(ai_history_changes_affect_recent_projects(&[path
            .to_string_lossy()
            .into_owned(),]));
    }

    #[test]
    fn modified_history_does_not_trigger_recent_project_scan() {
        assert!(!event_may_change_recent_projects(
            FileSystemEventKind::Modified
        ));
        assert!(event_may_change_recent_projects(
            FileSystemEventKind::Created
        ));
        assert!(event_may_change_recent_projects(
            FileSystemEventKind::Removed
        ));
        assert!(event_may_change_recent_projects(
            FileSystemEventKind::Renamed
        ));
    }

    #[test]
    fn timestamp_only_recent_project_change_is_not_visible() {
        let previous = vec![
            recent(r"D:\projects\one", 10),
            recent(r"D:\projects\two", 5),
        ];
        let next = vec![
            recent(r"D:\projects\one", 20),
            recent(r"D:\projects\two", 7),
        ];
        assert!(!recent_project_paths_changed(&previous, &next));
    }

    #[test]
    fn recent_project_addition_or_reorder_is_visible() {
        let previous = vec![
            recent(r"D:\projects\one", 10),
            recent(r"D:\projects\two", 5),
        ];
        let added = vec![
            recent(r"D:\projects\three", 20),
            recent(r"D:\projects\one", 10),
            recent(r"D:\projects\two", 5),
        ];
        let reordered = vec![
            recent(r"D:\projects\two", 15),
            recent(r"D:\projects\one", 10),
        ];
        assert!(recent_project_paths_changed(&previous, &added));
        assert!(recent_project_paths_changed(&previous, &reordered));
    }
}
