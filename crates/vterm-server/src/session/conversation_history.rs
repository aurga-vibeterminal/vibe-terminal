use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OpenFlags};
use serde_json::Value;
use tracing::{debug, warn};
use vterm_common::{
    AiConversationInfo, AiConversationMessage, AiConversationMessageKind, AiConversationRole,
    SessionApp,
};

const MAX_CONVERSATIONS: usize = 100;
const MAX_SCAN_FILES: usize = 1_000;
const MAX_SCAN_DEPTH: usize = 8;
const MAX_FILE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_KIMI_RECORD_BYTES: usize = 2 * 1024 * 1024;

struct CodexConversationTarget {
    rollout_path: PathBuf,
    model_provider: String,
}

pub(super) fn list_ai_conversations(
    app: SessionApp,
    working_path: &str,
) -> Vec<AiConversationInfo> {
    match app.0.as_str() {
        "claude" => list_claude_conversations(working_path),
        "codex" => list_codex_conversations(working_path),
        "kimi" => list_kimi_conversations(working_path),
        "qwen" => list_qwen_conversations(working_path),
        "pi" => list_pi_conversations(working_path),
        "opencode" => list_opencode_conversations(working_path),
        "dsh" => user_home_dir()
            .map(|home| list_dsh_conversations(&home, working_path))
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

pub(super) fn read_ai_conversation(
    app: &SessionApp,
    working_path: &str,
    conversation_id: &str,
) -> Vec<AiConversationMessage> {
    if app.0 == "opencode" {
        let Some(home) = user_home_dir() else {
            return Vec::new();
        };
        return read_opencode_conversation(&home, working_path, conversation_id);
    }
    let Some(path) = resolve_conversation_path(app, working_path, conversation_id) else {
        return Vec::new();
    };
    match app.0.as_str() {
        "claude" => parse_claude_conversation_file(&path),
        "codex" => {
            let model_provider = codex_conversation_target(working_path, conversation_id)
                .map(|target| target.model_provider)
                .unwrap_or_default();
            parse_codex_conversation_file(&path, &model_provider)
        }
        "kimi" => parse_kimi_wire_file(&path),
        "qwen" => parse_qwen_conversation_file(&path),
        "pi" => parse_pi_conversation_file(&path),
        "dsh" => parse_dsh_conversation_file(&path),
        _ => parse_conversation_file_with_model(&path, ""),
    }
}

pub(super) fn ai_conversation_global_watch_roots() -> Vec<PathBuf> {
    let Some(home) = user_home_dir() else {
        return Vec::new();
    };
    let mut roots = Vec::new();
    for path in [
        home.join(".claude"),
        home.join(".claude").join("projects"),
        home.join(".codex"),
        home.join(".codex").join("sessions"),
        // ZCode CLI stores project sessions in a SQLite db. Watching the
        // directory (non-recursive) catches db.sqlite modifications (including
        // SQLite's atomic replace-on-vacuum) and triggers a recent-projects
        // rescan. Only the `session.directory` column is read — no AI
        // conversation content.
        home.join(".zcode").join("cli").join("db"),
        // Kimi Code appends sessions to session_index.jsonl (watched via the
        // non-recursive `.kimi-code` root) and stores each session's
        // transcript at sessions/<wd>/<session>/agents/main/wire.jsonl
        // (watched via the recursive `sessions` root).
        home.join(".kimi-code"),
        home.join(".kimi-code").join("sessions"),
        super::recent_projects::qwen_projects_root(&home),
        // Pi stores trust decisions and cwd-encoded JSONL transcripts here.
        home.join(".pi").join("agent"),
        home.join(".pi").join("agent").join("sessions"),
        // OpenCode stores project/session/message projections in SQLite.
        super::recent_projects::opencode_data_dir(&home),
        super::recent_projects::dsh_sessions_root(&home),
    ] {
        if path.exists() {
            roots.push(path);
        }
    }
    for db_path in super::recent_projects::opencode_database_paths(&home) {
        if let Some(parent) = db_path.parent().filter(|parent| parent.exists()) {
            roots.push(parent.to_path_buf());
        }
    }
    roots.sort();
    roots.dedup();
    roots
}
fn list_dsh_conversations(home: &Path, working_path: &str) -> Vec<AiConversationInfo> {
    list_dsh_conversations_at_root(
        &super::recent_projects::dsh_sessions_root(home),
        working_path,
    )
}

fn list_dsh_conversations_at_root(
    sessions_root: &Path,
    working_path: &str,
) -> Vec<AiConversationInfo> {
    super::recent_projects::dsh_session_artifacts(sessions_root)
        .into_iter()
        .filter(|artifact| {
            artifact.origin.as_deref() != Some("subagent")
                && is_same_dsh_project(&artifact.cwd, working_path)
        })
        .map(|artifact| {
            let brief =
                dsh_conversation_brief(&artifact.path).unwrap_or_else(|| artifact.id.clone());
            AiConversationInfo {
                id: artifact.id,
                time: format_unix_timestamp_secs(artifact.modified_secs),
                brief,
            }
        })
        .take(MAX_CONVERSATIONS)
        .collect()
}

fn is_same_dsh_project(left: &str, right: &str) -> bool {
    let normalize = |path: &str| {
        normalize_path_text(&expand_home_path_text(path))
            .trim_end_matches('/')
            .to_string()
    };
    normalize(left) == normalize(right)
}

fn dsh_conversation_brief(path: &Path) -> Option<String> {
    let bytes = super::recent_projects::read_dsh_log(path)?;
    let mut title = None;
    let mut first_prompt = None;
    for line in String::from_utf8_lossy(&bytes).lines() {
        let Ok(value) = serde_json::from_str::<Value>(line.trim()) else {
            continue;
        };
        if value.get("type").and_then(Value::as_str) == Some("session/title") {
            if let Some(candidate) = value
                .get("data")
                .and_then(|data| data.get("title"))
                .and_then(Value::as_str)
                .filter(|candidate| !candidate.trim().is_empty())
            {
                title = Some(brief_text(candidate));
            }
            continue;
        }
        if first_prompt.is_none() {
            first_prompt = parse_dsh_message_value(&value, 0)
                .into_iter()
                .find(|message| message.role == AiConversationRole::User)
                .map(|message| brief_text(&message.markdown));
        }
    }
    title.or(first_prompt).filter(|brief| !brief.is_empty())
}

fn parse_dsh_conversation_file(path: &Path) -> Vec<AiConversationMessage> {
    let Some(bytes) = super::recent_projects::read_dsh_log(path) else {
        return Vec::new();
    };
    String::from_utf8_lossy(&bytes)
        .lines()
        .enumerate()
        .filter_map(|(idx, line)| {
            serde_json::from_str::<Value>(line.trim())
                .ok()
                .map(|value| (idx, value))
        })
        .flat_map(|(idx, value)| parse_dsh_message_value(&value, idx))
        .collect()
}

fn dsh_message_time(data: &Value) -> String {
    data.get("stream")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|record| record.get("time").or_else(|| record.get("time0")))
        .find_map(Value::as_u64)
        .map(timestamp_millis_to_secs)
        .map(format_unix_timestamp_secs)
        .unwrap_or_default()
}

fn parse_dsh_message_value(value: &Value, idx: usize) -> Vec<AiConversationMessage> {
    let event_type = value.get("type").and_then(Value::as_str);
    if event_type != Some("user/message") && event_type != Some("assistant/message") {
        return Vec::new();
    }
    let Some(data) = value.get("data") else {
        return Vec::new();
    };
    let message = if event_type == Some("assistant/message") {
        data.get("message")
    } else {
        Some(data)
    };
    let Some(message) = message else {
        return Vec::new();
    };
    let role = if event_type == Some("user/message") {
        if message.get("role").and_then(Value::as_str) != Some("user")
            || message
                .get("source")
                .and_then(|source| source.get("kind"))
                .and_then(Value::as_str)
                != Some("user")
        {
            return Vec::new();
        }
        AiConversationRole::User
    } else {
        if message.get("role").and_then(Value::as_str) != Some("assistant") {
            return Vec::new();
        }
        AiConversationRole::Model
    };
    let id = message
        .get("id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("dsh-{idx}"));
    let time = dsh_message_time(data);
    let model = if role == AiConversationRole::Model {
        message
            .get("source")
            .and_then(|source| source.get("model"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    } else {
        String::new()
    };
    let Some(content) = message.get("content").and_then(Value::as_array) else {
        return Vec::new();
    };
    if role == AiConversationRole::User {
        let text = content
            .iter()
            .filter(|block| block.get("type").and_then(Value::as_str) == Some("text"))
            .filter_map(|block| block.get("text").and_then(Value::as_str))
            .filter(|text| !text.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        if text.is_empty() {
            return Vec::new();
        }
        return vec![AiConversationMessage {
            id,
            role,
            kind: AiConversationMessageKind::Text,
            time,
            model,
            markdown: text,
        }];
    }
    content
        .iter()
        .enumerate()
        .filter_map(|(content_idx, block)| {
            let (kind, text) = match block.get("type").and_then(Value::as_str) {
                Some("text") => (
                    AiConversationMessageKind::Text,
                    block.get("text").and_then(Value::as_str),
                ),
                Some("reasoning") => (
                    AiConversationMessageKind::Thinking,
                    block.get("text").and_then(Value::as_str),
                ),
                _ => return None,
            };
            let text = text?.trim();
            if text.is_empty() {
                return None;
            }
            Some(AiConversationMessage {
                id: format!("{id}-{content_idx}"),
                role: role.clone(),
                kind,
                time: time.clone(),
                model: model.clone(),
                markdown: text.to_string(),
            })
        })
        .collect()
}

fn list_claude_conversations(working_path: &str) -> Vec<AiConversationInfo> {
    let Some(home) = user_home_dir() else {
        warn!(
            "Claude conversation history unavailable: USERPROFILE/HOME is not set; working_path={}",
            working_path
        );
        return Vec::new();
    };
    let history_path = home.join(".claude").join("history.jsonl");
    let Ok(contents) = fs::read_to_string(&history_path) else {
        warn!(
            "Claude conversation history unavailable: failed to read {}; working_path={}",
            history_path.display(),
            working_path
        );
        return Vec::new();
    };
    let working_path = expand_home_path_text(working_path);
    let target_project = normalize_path_text(&working_path);
    // info!(
    //     "Listing Claude conversations from {}; working_path={}; normalized_project={}",
    //     history_path.display(),
    //     working_path,
    //     target_project
    // );
    let mut by_session = HashMap::<String, (String, u64)>::new();
    let mut parsed_lines = 0usize;
    let mut invalid_json_lines = 0usize;
    let mut missing_project_lines = 0usize;
    let mut project_mismatch_lines = 0usize;
    let mut missing_session_lines = 0usize;
    let mut matched_lines = 0usize;
    for line in contents.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line.trim()) else {
            invalid_json_lines += 1;
            continue;
        };
        parsed_lines += 1;
        let Some(project) = value.get("project").and_then(Value::as_str) else {
            missing_project_lines += 1;
            continue;
        };
        let normalized_project = normalize_path_text(project);
        if !is_path_within_project(&normalized_project, &target_project) {
            project_mismatch_lines += 1;
            debug!(
                "Skipping Claude conversation history row for different project: project={}; normalized_project={}; target_project={}",
                project, normalized_project, target_project
            );
            continue;
        }
        let Some(session_id) = value.get("sessionId").and_then(Value::as_str) else {
            missing_session_lines += 1;
            continue;
        };
        matched_lines += 1;
        let display = value
            .get("display")
            .and_then(Value::as_str)
            .map(brief_text)
            .unwrap_or_else(|| session_id.to_string());
        let timestamp = value
            .get("timestamp")
            .and_then(Value::as_u64)
            .map(timestamp_millis_to_secs)
            .unwrap_or(0);
        by_session
            .entry(session_id.to_string())
            .and_modify(|existing| {
                if timestamp >= existing.1 {
                    *existing = (display.clone(), timestamp);
                }
            })
            .or_insert((display, timestamp));
    }

    let mut sessions = by_session
        .into_iter()
        .map(|(id, (brief, timestamp))| (id, brief, timestamp))
        .collect::<Vec<_>>();
    sessions.sort_by_key(|(_, _, timestamp)| std::cmp::Reverse(*timestamp));
    sessions.truncate(MAX_CONVERSATIONS);

    let items = sessions
        .into_iter()
        .map(|(id, brief, timestamp)| AiConversationInfo {
            id,
            time: format_unix_timestamp_secs(timestamp),
            brief,
        })
        .collect::<Vec<_>>();
    // info!(
    //     "Listed {} Claude conversations for {}; parsed_lines={}; matched_lines={}; invalid_json_lines={}; missing_project_lines={}; project_mismatch_lines={}; missing_session_lines={}",
    //     items.len(),
    //     working_path,
    //     parsed_lines,
    //     matched_lines,
    //     invalid_json_lines,
    //     missing_project_lines,
    //     project_mismatch_lines,
    //     missing_session_lines
    // );
    // for item in &items {
    //     info!(
    //         "Claude conversation listed: session_id={}; time={}; brief={}",
    //         item.id, item.time, item.brief
    //     );
    // }
    items
}

fn list_codex_conversations(working_path: &str) -> Vec<AiConversationInfo> {
    let Some(home) = user_home_dir() else {
        return Vec::new();
    };
    let working_path = expand_home_path_text(working_path);
    if let Some(items) = list_codex_conversations_from_db(&home, &working_path) {
        if !items.is_empty() {
            return items;
        }
        debug!(
            working_path = %working_path,
            "Codex conversation database returned no matching rows; falling back to session files"
        );
    }
    let root = home.join(".codex").join("sessions");
    collect_jsonl_files(&root)
        .into_iter()
        .filter(|path| conversation_file_matches_working_path(path, &working_path))
        .filter_map(|path| conversation_info_from_file(&path))
        .take(MAX_CONVERSATIONS)
        .collect()
}

struct KimiSessionIndexEntry {
    session_id: String,
    session_dir: PathBuf,
    work_dir: String,
}

impl KimiSessionIndexEntry {
    fn wire_path(&self) -> PathBuf {
        self.session_dir
            .join("agents")
            .join("main")
            .join("wire.jsonl")
    }
}

/// Read `~/.kimi-code/session_index.jsonl`. The file is append-only, so
/// newer sessions are at the end.
fn read_kimi_session_index() -> Vec<KimiSessionIndexEntry> {
    let Some(home) = user_home_dir() else {
        return Vec::new();
    };
    let index_path = home.join(".kimi-code").join("session_index.jsonl");
    let Ok(contents) = fs::read_to_string(&index_path) else {
        debug!(
            "Kimi Code session index not available at {}",
            index_path.display()
        );
        return Vec::new();
    };
    let mut entries = Vec::new();
    for line in contents.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line.trim()) else {
            continue;
        };
        let (Some(session_id), Some(session_dir), Some(work_dir)) = (
            value.get("sessionId").and_then(Value::as_str),
            value.get("sessionDir").and_then(Value::as_str),
            value.get("workDir").and_then(Value::as_str),
        ) else {
            continue;
        };
        entries.push(KimiSessionIndexEntry {
            session_id: session_id.to_string(),
            session_dir: PathBuf::from(session_dir),
            work_dir: work_dir.to_string(),
        });
    }
    entries
}

fn list_kimi_conversations(working_path: &str) -> Vec<AiConversationInfo> {
    let target_project = normalize_path_text(&expand_home_path_text(working_path));
    kimi_conversation_infos(&read_kimi_session_index(), &target_project)
}

fn qwen_project_directory_name(working_path: &str) -> String {
    let working_path = expand_home_path_text(working_path);
    #[cfg(windows)]
    let working_path = working_path.to_lowercase();
    working_path
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect()
}

fn qwen_chats_directory(home: &Path, working_path: &str) -> PathBuf {
    super::recent_projects::qwen_projects_root(home)
        .join(qwen_project_directory_name(working_path))
        .join("chats")
}

fn normalize_qwen_project_path(path: &str) -> String {
    let path = strip_windows_extended_path_prefix(&expand_home_path_text(path)).replace('\\', "/");
    #[cfg(windows)]
    let path = path.to_lowercase();
    path.trim_end_matches('/').to_string()
}

fn is_same_qwen_project(left: &str, right: &str) -> bool {
    let left = normalize_qwen_project_path(left);
    let right = normalize_qwen_project_path(right);
    left == right
        || left
            .rsplit_once("/.qwen/worktrees/")
            .is_some_and(|(project, _)| project == right)
}

fn list_qwen_conversations(working_path: &str) -> Vec<AiConversationInfo> {
    let Some(home) = user_home_dir() else {
        return Vec::new();
    };
    list_qwen_conversations_at_root(&qwen_chats_directory(&home, working_path), working_path)
}

fn list_qwen_conversations_at_root(
    chats_root: &Path,
    working_path: &str,
) -> Vec<AiConversationInfo> {
    super::recent_projects::qwen_session_artifacts(chats_root)
        .into_iter()
        .filter(|artifact| is_same_qwen_project(&artifact.cwd, working_path))
        .map(|artifact| {
            let brief =
                qwen_conversation_brief(&artifact.path).unwrap_or_else(|| artifact.id.clone());
            AiConversationInfo {
                id: artifact.id,
                time: format_unix_timestamp_secs(artifact.modified_secs),
                brief,
            }
        })
        .take(MAX_CONVERSATIONS)
        .collect()
}

fn qwen_conversation_brief(path: &Path) -> Option<String> {
    let reader = open_conversation_tail(path)?;
    let mut title = None;
    let mut first_prompt = None;
    for line in reader.lines().map_while(Result::ok) {
        let Ok(value) = serde_json::from_str::<Value>(line.trim()) else {
            continue;
        };
        if value.get("type").and_then(Value::as_str) == Some("system")
            && value.get("subtype").and_then(Value::as_str) == Some("custom_title")
        {
            title = value
                .get("systemPayload")
                .and_then(|payload| payload.get("customTitle"))
                .and_then(Value::as_str)
                .map(brief_text)
                .filter(|title| !title.is_empty());
            continue;
        }
        if first_prompt.is_none() {
            first_prompt = parse_qwen_message_value(&value, 0)
                .into_iter()
                .find(|message| message.role == AiConversationRole::User)
                .map(|message| brief_text(&message.markdown));
        }
    }
    title.or(first_prompt).filter(|brief| !brief.is_empty())
}

#[derive(Debug)]
struct PiSessionHeader {
    id: String,
    cwd: String,
}

fn pi_session_directory(home: &Path, working_path: &str) -> PathBuf {
    let resolved = pi_resolved_working_path(working_path);
    home.join(".pi")
        .join("agent")
        .join("sessions")
        .join(super::recent_projects::pi_session_directory_name(&resolved))
}

/// Match Pi's `resolvePath`: expand home syntax and make the path absolute
/// without resolving symlinks.
fn pi_resolved_working_path(working_path: &str) -> String {
    let expanded = expand_home_path_text(working_path);
    std::path::absolute(&expanded)
        .map(|path| strip_windows_extended_path_prefix(&path.to_string_lossy()))
        .unwrap_or(expanded)
}

fn parse_pi_session_header(path: &Path) -> Option<PiSessionHeader> {
    let file = fs::File::open(path).ok()?;
    for line in BufReader::new(file).lines().map_while(Result::ok).take(8) {
        let Ok(value) = serde_json::from_str::<Value>(line.trim()) else {
            continue;
        };
        let is_v3 = value.get("type").and_then(Value::as_str) == Some("session");
        let is_v4 = value.get("kind").and_then(Value::as_str) == Some("header")
            && value.get("version").and_then(Value::as_u64) == Some(4);
        if !is_v3 && !is_v4 {
            continue;
        }
        return Some(PiSessionHeader {
            id: value.get("id")?.as_str()?.to_string(),
            cwd: value.get("cwd")?.as_str()?.to_string(),
        });
    }
    None
}

fn list_pi_conversations(working_path: &str) -> Vec<AiConversationInfo> {
    let Some(home) = user_home_dir() else {
        return Vec::new();
    };
    let target = normalize_path_text(&pi_resolved_working_path(working_path));
    list_jsonl_files(&pi_session_directory(&home, working_path))
        .into_iter()
        .filter_map(|path| {
            let header = parse_pi_session_header(&path)?;
            if normalize_path_text(&header.cwd) != target {
                return None;
            }
            let messages = parse_pi_conversation_file(&path);
            let brief = messages
                .iter()
                .find(|message| message.role == AiConversationRole::User)
                .map(|message| brief_text(&message.markdown))
                .unwrap_or_else(|| "Conversation".to_string());
            Some(AiConversationInfo {
                id: header.id,
                time: metadata_time(&path),
                brief,
            })
        })
        .take(MAX_CONVERSATIONS)
        .collect()
}

fn list_opencode_conversations(working_path: &str) -> Vec<AiConversationInfo> {
    let Some(home) = user_home_dir() else {
        return Vec::new();
    };
    let (directory_clause, directory_values) = opencode_directory_match(working_path);
    let mut by_id = HashMap::<String, (u64, AiConversationInfo)>::new();
    for db_path in super::recent_projects::opencode_database_paths(&home) {
        let Ok(connection) = open_sqlite_readonly(&db_path) else {
            continue;
        };
        for (id, title, time_updated) in
            query_opencode_session_rows(&connection, &directory_clause, &directory_values)
        {
            let timestamp = time_updated.max(0) as u64;
            let title = title.unwrap_or_default();
            let info = AiConversationInfo {
                id: id.clone(),
                time: format_unix_timestamp_secs(timestamp_millis_to_secs(timestamp)),
                brief: if title.trim().is_empty() {
                    "Conversation".to_string()
                } else {
                    brief_text(&title)
                },
            };
            by_id
                .entry(id)
                .and_modify(|current| {
                    if timestamp > current.0 {
                        *current = (timestamp, info.clone());
                    }
                })
                .or_insert((timestamp, info));
        }
    }
    let mut items = by_id.into_values().collect::<Vec<_>>();
    items.sort_by_key(|item| std::cmp::Reverse(item.0));
    items
        .into_iter()
        .take(MAX_CONVERSATIONS)
        .map(|item| item.1)
        .collect()
}

/// Session rows from one OpenCode database. Current versions store sessions
/// in `session_v2`; the v1 `session` table remains as a migration leftover,
/// so query both and let the caller dedup by id (migrated sessions appear in
/// both). `title` is nullable in `session_v2`.
fn query_opencode_session_rows(
    connection: &Connection,
    directory_clause: &str,
    directory_values: &[String],
) -> Vec<(String, Option<String>, i64)> {
    let mut rows = Vec::new();
    for table in ["session_v2", "session"] {
        let Ok(mut statement) = connection.prepare(&format!(
            "SELECT id, title, time_updated \
             FROM {table} \
             WHERE parent_id IS NULL AND time_archived IS NULL \
               AND ({directory_clause}) \
             ORDER BY time_updated DESC, id DESC \
             LIMIT ?"
        )) else {
            continue;
        };
        let mut params = directory_values
            .iter()
            .cloned()
            .map(rusqlite::types::Value::from)
            .collect::<Vec<_>>();
        params.push((MAX_CONVERSATIONS as i64).into());
        let Ok(mapped) = statement.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, i64>(2)?,
            ))
        }) else {
            continue;
        };
        rows.extend(mapped.flatten());
    }
    rows
}

fn opencode_directory_match(working_path: &str) -> (String, Vec<String>) {
    #[cfg(windows)]
    {
        let bare = strip_windows_extended_path_prefix(&expand_home_path_text(working_path));
        let backslash = bare.replace('/', "\\");
        let slash = backslash.replace('\\', "/");
        let prefixed = format!(r"\\?\{backslash}");
        let prefixed_slash = prefixed.replace('\\', "/");
        let mut values = Vec::with_capacity(4);
        for value in [backslash, slash, prefixed, prefixed_slash] {
            if !values
                .iter()
                .any(|existing: &String| existing.eq_ignore_ascii_case(&value))
            {
                values.push(value);
            }
        }
        let clause = std::iter::repeat_n("directory = ? COLLATE NOCASE", values.len())
            .collect::<Vec<_>>()
            .join(" OR ");
        (clause, values)
    }
    #[cfg(not(windows))]
    {
        (
            "directory = ?".to_string(),
            vec![expand_home_path_text(working_path)],
        )
    }
}

fn kimi_conversation_infos(
    entries: &[KimiSessionIndexEntry],
    target_project: &str,
) -> Vec<AiConversationInfo> {
    let mut candidates = Vec::new();
    for (index_position, entry) in entries.iter().enumerate() {
        if !is_path_within_project(&normalize_path_text(&entry.work_dir), &target_project) {
            continue;
        }
        let wire_path = entry.wire_path();
        // session_index.jsonl is append-only, but an older session can become
        // active again and receive newer events. Use the transcript's mtime as
        // the source of truth for both the displayed time and list ordering.
        // A missing transcript sorts last instead of appearing as newly active.
        let modified_secs = metadata_modified_secs_opt(&wire_path).unwrap_or(0);
        candidates.push((modified_secs, index_position, entry, wire_path));
    }
    // Preserve the previous newest-index-first behavior as a deterministic
    // tie-breaker on file systems with coarse timestamp resolution.
    candidates.sort_by_key(|(modified_secs, index_position, _, _)| {
        std::cmp::Reverse((*modified_secs, *index_position))
    });
    candidates
        .into_iter()
        .take(MAX_CONVERSATIONS)
        .map(|(modified_secs, _, entry, wire_path)| {
            let brief = kimi_first_user_text(&wire_path)
                .map(|text| brief_text(&text))
                .unwrap_or_else(|| "Conversation".to_string());
            AiConversationInfo {
                id: entry.session_id.clone(),
                time: format_unix_timestamp_secs(modified_secs),
                brief,
            }
        })
        .collect()
}

/// Extract the first user prompt from a Kimi Code wire.jsonl transcript,
/// used as the conversation brief.
fn kimi_first_user_text(path: &Path) -> Option<String> {
    let contents = fs::read_to_string(path).ok()?;
    for line in contents.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line.trim()) else {
            continue;
        };
        if value.get("type").and_then(Value::as_str) != Some("turn.prompt") {
            continue;
        }
        let parts = value.get("input").and_then(Value::as_array)?;
        let text = parts
            .iter()
            .filter(|part| part.get("type").and_then(Value::as_str) == Some("text"))
            .filter_map(|part| part.get("text").and_then(Value::as_str))
            .find(|text| !text.trim().is_empty())?;
        return Some(text.to_string());
    }
    None
}

/// SQL predicate + bound values selecting Codex threads whose `cwd` matches
/// the terminal `working_path`.
///
/// On Windows the database sometimes stores `cwd` with the `\\?\`
/// extended-length path prefix while the terminal path lacks it (or vice
/// versa), so a plain `cwd = ?` equality misses rows. We normalize the query
/// value by stripping the prefix, then ask the database to match either the
/// bare path or its `\\?\`-prefixed form — `cwd = ? COLLATE NOCASE OR
/// cwd = ? COLLATE NOCASE`. Both sides are plain `column = literal`, so
/// (unlike a `substr`/`lower` expression on the column) SQLite can use any
/// index on `cwd` and never recomputes a function per row. `COLLATE NOCASE`
/// handles the case-insensitive Windows file system. The prefixed value is
/// rebuilt from the stripped value rather than trusting the caller's input
/// form, so the query is correct regardless of which side carries the prefix.
///
/// On other platforms `cwd` is stored verbatim, so the original exact
/// `cwd = ? COLLATE NOCASE` equality is used. The returned clause contains one
/// `?` placeholder per returned value, in order.
fn codex_cwd_match(working_path: &str) -> (String, Vec<String>) {
    #[cfg(windows)]
    {
        let bare = strip_windows_extended_path_prefix(&expand_home_path_text(working_path));
        let prefixed = format!(r"\\?\{bare}");
        (
            "cwd = ? COLLATE NOCASE OR cwd = ? COLLATE NOCASE".to_string(),
            vec![bare, prefixed],
        )
    }
    #[cfg(not(windows))]
    {
        let value = expand_home_path_text(working_path);
        ("cwd = ? COLLATE NOCASE".to_string(), vec![value])
    }
}

fn list_codex_conversations_from_db(
    home: &Path,
    working_path: &str,
) -> Option<Vec<AiConversationInfo>> {
    let db_path = home.join(".codex").join("state_5.sqlite");
    let connection = open_sqlite_readonly(&db_path)
        .map_err(|error| {
            debug!(
                "Codex conversation database unavailable at {}: {}",
                db_path.display(),
                error
            );
            error
        })
        .ok()?;
    let (cwd_clause, cwd_values) = codex_cwd_match(working_path);
    let mut statement = connection
        .prepare(&format!(
            "SELECT id, title, updated_at, updated_at_ms \
             FROM threads \
             WHERE archived = 0 AND cwd IS NOT NULL AND ({cwd_clause}) \
             ORDER BY COALESCE(updated_at_ms, updated_at * 1000) DESC, id DESC \
             LIMIT ?"
        ))
        .map_err(|error| {
            warn!(
                "Failed to prepare Codex conversation query from {}: {}",
                db_path.display(),
                error
            );
            error
        })
        .ok()?;
    let mut params = Vec::with_capacity(cwd_values.len() + 1);
    params.extend(cwd_values.into_iter().map(rusqlite::types::Value::from));
    params.push((MAX_CONVERSATIONS as i64).into());
    let rows = statement
        .query_map(rusqlite::params_from_iter(params.iter()), |row| {
            let id: String = row.get(0)?;
            let title: Option<String> = row.get(1)?;
            let updated_at: Option<i64> = row.get(2)?;
            let updated_at_ms: Option<i64> = row.get(3)?;
            Ok((id, title.unwrap_or_default(), updated_at, updated_at_ms))
        })
        .map_err(|error| {
            warn!(
                "Failed to query Codex conversations from {}: {}",
                db_path.display(),
                error
            );
            error
        })
        .ok()?;

    let mut items = Vec::new();
    for row in rows {
        let (id, title, updated_at, updated_at_ms) = match row {
            Ok(row) => row,
            Err(error) => {
                warn!("Failed to read Codex conversation row: {}", error);
                continue;
            }
        };
        let timestamp = updated_at_ms
            .or_else(|| updated_at.map(|value| value.saturating_mul(1000)))
            .unwrap_or_default();
        items.push(AiConversationInfo {
            id,
            time: format_unix_timestamp_secs(timestamp_millis_to_secs(timestamp.max(0) as u64)),
            brief: if title.trim().is_empty() {
                "Conversation".to_string()
            } else {
                brief_text(&title)
            },
        });
        if items.len() >= MAX_CONVERSATIONS {
            break;
        }
    }
    Some(items)
}
fn resolve_conversation_path(
    app: &SessionApp,
    working_path: &str,
    conversation_id: &str,
) -> Option<PathBuf> {
    let home = user_home_dir()?;
    let working_path = expand_home_path_text(working_path);
    if app.0 == "claude" {
        return resolve_claude_conversation_path(&home, &working_path, conversation_id);
    }
    if app.0 == "codex" {
        let root = fs::canonicalize(home.join(".codex").join("sessions")).ok()?;
        let path = if let Some(target) = codex_conversation_target(&working_path, conversation_id) {
            fs::canonicalize(target.rollout_path).ok()?
        } else {
            fs::canonicalize(PathBuf::from(conversation_id)).ok()?
        };
        return (path.starts_with(&root) && path.is_file()).then_some(path);
    }
    if app.0 == "kimi" {
        return resolve_kimi_conversation_path(&home, &working_path, conversation_id);
    }
    if app.0 == "qwen" {
        return resolve_qwen_conversation_path(&home, &working_path, conversation_id);
    }
    if app.0 == "pi" {
        return resolve_pi_conversation_path(&home, &working_path, conversation_id);
    }
    if app.0 == "dsh" {
        return resolve_dsh_conversation_path(&home, &working_path, conversation_id);
    }
    None
}

fn resolve_qwen_conversation_path(
    home: &Path,
    working_path: &str,
    session_id: &str,
) -> Option<PathBuf> {
    resolve_qwen_conversation_path_at_root(
        &qwen_chats_directory(home, working_path),
        working_path,
        session_id,
    )
}

fn resolve_qwen_conversation_path_at_root(
    chats_root: &Path,
    working_path: &str,
    session_id: &str,
) -> Option<PathBuf> {
    if !(32..=36).contains(&session_id.len())
        || !session_id
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() || ch == '-')
    {
        return None;
    }
    let root = fs::canonicalize(chats_root).ok()?;
    let artifact = super::recent_projects::qwen_session_artifact(
        chats_root.join(format!("{session_id}.jsonl")),
    )?;
    if !artifact.id.eq_ignore_ascii_case(session_id)
        || !is_same_qwen_project(&artifact.cwd, working_path)
    {
        return None;
    }
    let path = fs::canonicalize(artifact.path).ok()?;
    (path.starts_with(&root) && path.is_file()).then_some(path)
}

fn resolve_dsh_conversation_path(
    home: &Path,
    working_path: &str,
    session_id: &str,
) -> Option<PathBuf> {
    resolve_dsh_conversation_path_at_root(
        &super::recent_projects::dsh_sessions_root(home),
        working_path,
        session_id,
    )
}

fn resolve_dsh_conversation_path_at_root(
    sessions_root: &Path,
    working_path: &str,
    session_id: &str,
) -> Option<PathBuf> {
    let root = fs::canonicalize(sessions_root).ok()?;
    let artifact = super::recent_projects::dsh_session_artifacts(sessions_root)
        .into_iter()
        .find(|artifact| {
            artifact.id == session_id
                && artifact.origin.as_deref() != Some("subagent")
                && is_same_dsh_project(&artifact.cwd, working_path)
        })?;
    let path = fs::canonicalize(artifact.path).ok()?;
    (path.starts_with(&root) && path.is_file()).then_some(path)
}

fn resolve_pi_conversation_path(
    home: &Path,
    working_path: &str,
    session_id: &str,
) -> Option<PathBuf> {
    if session_id.is_empty()
        || !session_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        return None;
    }
    let session_dir = pi_session_directory(home, working_path);
    let root = fs::canonicalize(&session_dir).ok()?;
    let target = normalize_path_text(&pi_resolved_working_path(working_path));
    list_jsonl_files(&session_dir).into_iter().find_map(|path| {
        let header = parse_pi_session_header(&path)?;
        if header.id != session_id || normalize_path_text(&header.cwd) != target {
            return None;
        }
        let path = fs::canonicalize(path).ok()?;
        (path.starts_with(&root) && path.is_file()).then_some(path)
    })
}

fn resolve_kimi_conversation_path(
    home: &Path,
    working_path: &str,
    session_id: &str,
) -> Option<PathBuf> {
    if !session_id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return None;
    }
    let root = fs::canonicalize(home.join(".kimi-code").join("sessions")).ok()?;
    let target_project = normalize_path_text(working_path);
    let entry = read_kimi_session_index().into_iter().rev().find(|entry| {
        entry.session_id == session_id
            && is_path_within_project(&normalize_path_text(&entry.work_dir), &target_project)
    })?;
    let path = fs::canonicalize(entry.wire_path()).ok()?;
    (path.starts_with(&root) && path.is_file()).then_some(path)
}

fn codex_conversation_target(
    working_path: &str,
    conversation_id: &str,
) -> Option<CodexConversationTarget> {
    let home = user_home_dir()?;
    let db_path = home.join(".codex").join("state_5.sqlite");
    let connection = open_sqlite_readonly(&db_path).ok()?;
    let (cwd_clause, cwd_values) = codex_cwd_match(working_path);
    let mut statement = connection
        .prepare(&format!(
            "SELECT rollout_path, model_provider \
             FROM threads \
             WHERE id = ? AND archived = 0 AND cwd IS NOT NULL AND ({cwd_clause}) \
             LIMIT 1"
        ))
        .ok()?;
    let mut params: Vec<rusqlite::types::Value> = Vec::with_capacity(1 + cwd_values.len());
    params.push(conversation_id.to_string().into());
    params.extend(cwd_values.into_iter().map(rusqlite::types::Value::from));
    statement
        .query_row(rusqlite::params_from_iter(params.iter()), |row| {
            let rollout_path: String = row.get(0)?;
            let model_provider: String = row.get(1)?;
            Ok(CodexConversationTarget {
                rollout_path: PathBuf::from(rollout_path),
                model_provider,
            })
        })
        .ok()
}
fn resolve_claude_conversation_path(
    home: &Path,
    working_path: &str,
    session_id: &str,
) -> Option<PathBuf> {
    if !session_id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return None;
    }
    let root = home
        .join(".claude")
        .join("projects")
        .join(claude_project_dir_name(working_path));
    let root = fs::canonicalize(root).ok()?;
    let direct = root.join(format!("{session_id}.jsonl"));
    if direct.is_file() {
        return Some(direct);
    }
    list_jsonl_files(&root)
        .into_iter()
        .find(|path| path.file_stem().and_then(|stem| stem.to_str()) == Some(session_id))
}

fn conversation_info_from_file(path: &Path) -> Option<AiConversationInfo> {
    let messages = parse_conversation_file_with_model(path, "");
    let brief = messages
        .iter()
        .find(|message| message.role == AiConversationRole::User)
        .or_else(|| messages.first())
        .map(|message| brief_text(&message.markdown))
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("Conversation")
                .to_string()
        });
    let time = messages
        .iter()
        .find_map(|message| (!message.time.is_empty()).then(|| message.time.clone()))
        .unwrap_or_else(|| metadata_time(path));
    Some(AiConversationInfo {
        id: path.to_string_lossy().into_owned(),
        time,
        brief,
    })
}

fn parse_conversation_file_with_model(
    path: &Path,
    model_provider: &str,
) -> Vec<AiConversationMessage> {
    let Some(reader) = open_conversation_tail(path) else {
        return Vec::new();
    };
    let mut messages = Vec::new();
    for (idx, line) in reader.lines().map_while(Result::ok).enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
            continue;
        };
        if let Some(message) = parse_message_value(&value, idx, model_provider) {
            messages.push(message);
        }
    }
    messages
}

fn parse_claude_conversation_file(path: &Path) -> Vec<AiConversationMessage> {
    let Some(reader) = open_conversation_tail(path) else {
        return Vec::new();
    };
    let mut messages = Vec::new();
    for (idx, line) in reader.lines().map_while(Result::ok).enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
            continue;
        };
        messages.extend(parse_claude_message_value(&value, idx));
    }
    messages
}

fn parse_codex_conversation_file(path: &Path, model_provider: &str) -> Vec<AiConversationMessage> {
    let Some(reader) = open_conversation_tail(path) else {
        return Vec::new();
    };
    let mut messages = Vec::new();
    for (idx, line) in reader.lines().map_while(Result::ok).enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
            continue;
        };
        if let Some(message) = parse_codex_message_value(&value, idx, model_provider) {
            messages.push(message);
        }
    }
    messages
}

fn parse_qwen_conversation_file(path: &Path) -> Vec<AiConversationMessage> {
    let Some(reader) = open_conversation_tail(path) else {
        return Vec::new();
    };
    let mut messages = Vec::new();
    for (idx, line) in reader.lines().map_while(Result::ok).enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
            continue;
        };
        messages.extend(parse_qwen_message_value(&value, idx));
    }
    messages
}

fn parse_qwen_message_value(value: &Value, idx: usize) -> Vec<AiConversationMessage> {
    if value.get("subtype").and_then(Value::as_str).is_some()
        || value.get("isSidechain").and_then(Value::as_bool) == Some(true)
    {
        return Vec::new();
    }
    let role = match value.get("type").and_then(Value::as_str) {
        Some("user") => AiConversationRole::User,
        Some("assistant") => AiConversationRole::Model,
        _ => return Vec::new(),
    };
    let id = value
        .get("uuid")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("qwen-{idx}"));
    let time = value
        .get("timestamp")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let model = if role == AiConversationRole::Model {
        value
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    } else {
        String::new()
    };

    if role == AiConversationRole::User {
        if let Some(text) = value
            .get("systemPayload")
            .and_then(|payload| payload.get("displayText"))
            .and_then(Value::as_str)
        {
            return if text.trim().is_empty() {
                Vec::new()
            } else {
                vec![AiConversationMessage {
                    id,
                    role,
                    kind: AiConversationMessageKind::Text,
                    time,
                    model,
                    markdown: text.to_string(),
                }]
            };
        }
        let text = value
            .get("message")
            .and_then(|message| message.get("parts"))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|part| part.get("text").and_then(Value::as_str))
            .filter(|text| !text.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        return if text.is_empty() {
            Vec::new()
        } else {
            vec![AiConversationMessage {
                id,
                role,
                kind: AiConversationMessageKind::Text,
                time,
                model,
                markdown: text,
            }]
        };
    }

    value
        .get("message")
        .and_then(|message| message.get("parts"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(|(content_idx, part)| {
            let text = part.get("text").and_then(Value::as_str)?.trim();
            if text.is_empty() {
                return None;
            }
            let kind = if part.get("thought").and_then(Value::as_bool) == Some(true) {
                AiConversationMessageKind::Thinking
            } else {
                AiConversationMessageKind::Text
            };
            Some(AiConversationMessage {
                id: format!("{id}-{content_idx}"),
                role: role.clone(),
                kind,
                time: time.clone(),
                model: model.clone(),
                markdown: text.to_string(),
            })
        })
        .collect()
}

fn parse_pi_conversation_file(path: &Path) -> Vec<AiConversationMessage> {
    let Some(reader) = open_conversation_tail(path) else {
        return Vec::new();
    };
    let mut messages = Vec::new();
    for (idx, line) in reader.lines().map_while(Result::ok).enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
            continue;
        };
        messages.extend(parse_pi_message_value(&value, idx));
    }
    messages
}

fn read_opencode_conversation(
    home: &Path,
    working_path: &str,
    session_id: &str,
) -> Vec<AiConversationMessage> {
    if !session_id.starts_with("ses_")
        || !session_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
    {
        return Vec::new();
    }
    let (directory_clause, directory_values) = opencode_directory_match(working_path);
    for db_path in super::recent_projects::opencode_database_paths(home) {
        let Ok(connection) = open_sqlite_readonly(&db_path) else {
            continue;
        };
        let mut params = Vec::<rusqlite::types::Value>::with_capacity(directory_values.len() + 1);
        params.push(session_id.to_string().into());
        params.extend(
            directory_values
                .iter()
                .cloned()
                .map(rusqlite::types::Value::from),
        );
        // Current versions store sessions in `session_v2`; the v1 `session`
        // table remains as a migration leftover, so accept either.
        let exists = ["session_v2", "session"].iter().any(|table| {
            connection
                .query_row(
                    &format!(
                        "SELECT 1 FROM {table} \
                         WHERE id = ? AND parent_id IS NULL AND time_archived IS NULL \
                           AND ({directory_clause}) LIMIT 1"
                    ),
                    rusqlite::params_from_iter(params.iter()),
                    |_| Ok(()),
                )
                .is_ok()
        });
        if !exists {
            continue;
        }

        // Current OpenCode versions use session_message. During the storage
        // migration older releases used message + part, so retain that as a
        // fallback when the current projection has no rows.
        if let Some(messages) = parse_opencode_current_messages(&connection, session_id) {
            if !messages.is_empty() {
                return messages;
            }
        }
        if let Some(messages) = parse_opencode_legacy_messages(&connection, session_id) {
            return messages;
        }
    }
    Vec::new()
}

fn parse_opencode_current_messages(
    connection: &Connection,
    session_id: &str,
) -> Option<Vec<AiConversationMessage>> {
    let mut statement = connection
        .prepare(
            "SELECT id, type, time_created, data \
             FROM session_message WHERE session_id = ?1 \
             ORDER BY seq ASC, id ASC",
        )
        .ok()?;
    let rows = statement
        .query_map([session_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .ok()?;
    let mut messages = Vec::new();
    for (id, message_type, time_created, data) in rows.flatten() {
        let Ok(data) = serde_json::from_str::<Value>(&data) else {
            continue;
        };
        let time = opencode_message_time(&data, time_created);
        match message_type.as_str() {
            "user" => {
                let text = data.get("text").and_then(Value::as_str).unwrap_or_default();
                if !text.trim().is_empty() {
                    messages.push(AiConversationMessage {
                        id,
                        role: AiConversationRole::User,
                        kind: AiConversationMessageKind::Text,
                        time,
                        model: String::new(),
                        markdown: text.to_string(),
                    });
                }
            }
            "assistant" => {
                let model = data
                    .get("model")
                    .and_then(|model| model.get("id"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                for (idx, content) in data
                    .get("content")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .enumerate()
                {
                    let kind = match content.get("type").and_then(Value::as_str) {
                        Some("text") => AiConversationMessageKind::Text,
                        Some("reasoning") => AiConversationMessageKind::Thinking,
                        _ => continue,
                    };
                    let text = content
                        .get("text")
                        .and_then(Value::as_str)
                        .unwrap_or_default();
                    if text.trim().is_empty() {
                        continue;
                    }
                    let content_id = content
                        .get("id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                        .unwrap_or_else(|| format!("{id}-{idx}"));
                    messages.push(AiConversationMessage {
                        id: content_id,
                        role: AiConversationRole::Model,
                        kind,
                        time: time.clone(),
                        model: model.clone(),
                        markdown: text.to_string(),
                    });
                }
            }
            _ => {}
        }
    }
    Some(messages)
}

fn parse_opencode_legacy_messages(
    connection: &Connection,
    session_id: &str,
) -> Option<Vec<AiConversationMessage>> {
    let mut message_statement = connection
        .prepare(
            "SELECT id, time_created, data FROM message \
             WHERE session_id = ?1 ORDER BY time_created ASC, id ASC",
        )
        .ok()?;
    let mut part_statement = connection
        .prepare(
            "SELECT id, data FROM part WHERE message_id = ?1 \
             ORDER BY time_created ASC, id ASC",
        )
        .ok()?;
    let rows = message_statement
        .query_map([session_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .ok()?;
    let mut messages = Vec::new();
    for (message_id, time_created, data) in rows.flatten() {
        let Ok(data) = serde_json::from_str::<Value>(&data) else {
            continue;
        };
        let Some(role) = data.get("role").and_then(Value::as_str) else {
            continue;
        };
        let (role, model) = match role {
            "user" => (AiConversationRole::User, String::new()),
            "assistant" => (
                AiConversationRole::Model,
                data.get("modelID")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            ),
            _ => continue,
        };
        let time = opencode_message_time(&data, time_created);
        let Ok(parts) = part_statement.query_map([&message_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }) else {
            continue;
        };
        for (part_id, part) in parts.flatten() {
            let Ok(part) = serde_json::from_str::<Value>(&part) else {
                continue;
            };
            if part.get("ignored").and_then(Value::as_bool) == Some(true) {
                continue;
            }
            let kind = match part.get("type").and_then(Value::as_str) {
                Some("text") => AiConversationMessageKind::Text,
                Some("reasoning") => AiConversationMessageKind::Thinking,
                _ => continue,
            };
            let text = part.get("text").and_then(Value::as_str).unwrap_or_default();
            if text.trim().is_empty() {
                continue;
            }
            messages.push(AiConversationMessage {
                id: part_id,
                role: role.clone(),
                kind,
                time: time.clone(),
                model: model.clone(),
                markdown: text.to_string(),
            });
        }
    }
    Some(messages)
}

fn opencode_message_time(data: &Value, fallback_millis: i64) -> String {
    let timestamp = data
        .get("time")
        .and_then(|time| time.get("created"))
        .and_then(Value::as_u64)
        .map(timestamp_millis_to_secs)
        .unwrap_or_else(|| timestamp_millis_to_secs(fallback_millis.max(0) as u64));
    format_unix_timestamp_secs(timestamp)
}

/// Parse both Pi coding-agent v3 rows (`type: message`) and JSONL repository
/// v4 rows (`kind: entry, type: message`). Their message payload is shared.
fn parse_pi_message_value(value: &Value, idx: usize) -> Vec<AiConversationMessage> {
    if value.get("type").and_then(Value::as_str) != Some("message") {
        return Vec::new();
    }
    if let Some(kind) = value.get("kind").and_then(Value::as_str) {
        if kind != "entry" {
            return Vec::new();
        }
    }
    let Some(message) = value.get("message") else {
        return Vec::new();
    };
    let Some(role) = message.get("role").and_then(Value::as_str) else {
        return Vec::new();
    };
    let entry_id = value
        .get("id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("pi-message-{idx}"));
    let time = pi_message_time(value, message);

    match role {
        "user" => {
            let text = pi_text_content(message.get("content"));
            if text.trim().is_empty() {
                return Vec::new();
            }
            vec![AiConversationMessage {
                id: entry_id,
                role: AiConversationRole::User,
                kind: AiConversationMessageKind::Text,
                time,
                model: String::new(),
                markdown: text,
            }]
        }
        "assistant" => {
            let model = message
                .get("model")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let Some(content) = message.get("content") else {
                return Vec::new();
            };
            let blocks = content
                .as_array()
                .cloned()
                .unwrap_or_else(|| vec![content.clone()]);
            blocks
                .iter()
                .enumerate()
                .filter_map(|(content_idx, block)| {
                    let (kind, text) = match block.get("type").and_then(Value::as_str) {
                        Some("text") => (
                            AiConversationMessageKind::Text,
                            block.get("text").and_then(Value::as_str),
                        ),
                        Some("thinking") => (
                            AiConversationMessageKind::Thinking,
                            block.get("thinking").and_then(Value::as_str),
                        ),
                        _ => return None,
                    };
                    let text = text?.trim();
                    if text.is_empty() {
                        return None;
                    }
                    Some(AiConversationMessage {
                        id: format!("{entry_id}-{content_idx}"),
                        role: AiConversationRole::Model,
                        kind,
                        time: time.clone(),
                        model: model.clone(),
                        markdown: text.to_string(),
                    })
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn pi_text_content(content: Option<&Value>) -> String {
    match content {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Array(blocks)) => blocks
            .iter()
            .filter(|block| block.get("type").and_then(Value::as_str) == Some("text"))
            .filter_map(|block| block.get("text").and_then(Value::as_str))
            .filter(|text| !text.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn pi_message_time(entry: &Value, message: &Value) -> String {
    if let Some(timestamp) = message.get("timestamp").and_then(Value::as_u64) {
        return format_unix_timestamp_secs(timestamp_millis_to_secs(timestamp));
    }
    match entry.get("timestamp") {
        Some(Value::String(timestamp)) => timestamp.clone(),
        Some(Value::Number(timestamp)) => timestamp
            .as_u64()
            .map(timestamp_millis_to_secs)
            .map(format_unix_timestamp_secs)
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn parse_codex_message_value(
    value: &Value,
    idx: usize,
    model_provider: &str,
) -> Option<AiConversationMessage> {
    if value.get("type").and_then(Value::as_str) != Some("event_msg") {
        return None;
    }
    let payload = value.get("payload")?;
    let payload_type = payload.get("type").and_then(Value::as_str)?;
    let timestamp = value
        .get("timestamp")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    // Older rollouts emit `user_message`/`agent_message` events; newer ones
    // wrap the same content in `item_completed` items typed
    // `UserMessage`/`AgentMessage` with a `content[].text` array.
    let (role, text) = match payload_type {
        "user_message" => (
            AiConversationRole::User,
            payload.get("message").and_then(Value::as_str)?.to_string(),
        ),
        "agent_message" => (
            AiConversationRole::Model,
            payload.get("message").and_then(Value::as_str)?.to_string(),
        ),
        "item_completed" => {
            let item = payload.get("item")?;
            let role = match item.get("type").and_then(Value::as_str) {
                Some("UserMessage") => AiConversationRole::User,
                Some("AgentMessage") => AiConversationRole::Model,
                _ => return None,
            };
            let text = item
                .get("content")
                .and_then(Value::as_array)?
                .iter()
                .filter_map(|part| part.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("\n");
            (role, text)
        }
        _ => return None,
    };
    if text.trim().is_empty() {
        return None;
    }
    let model = match role {
        AiConversationRole::User => String::new(),
        AiConversationRole::Model => model_provider.to_string(),
    };
    Some(AiConversationMessage {
        id: format!("codex-{idx}"),
        role,
        kind: AiConversationMessageKind::Text,
        time: timestamp,
        model,
        markdown: text,
    })
}

fn open_conversation_tail(path: &Path) -> Option<BufReader<fs::File>> {
    let mut file = fs::File::open(path).ok()?;
    let metadata = file.metadata().ok()?;
    if !metadata.is_file() {
        return None;
    }

    let file_len = metadata.len();
    if file_len <= MAX_FILE_BYTES {
        return Some(BufReader::new(file));
    }

    let tail_start = file_len - MAX_FILE_BYTES;
    file.seek(SeekFrom::Start(tail_start - 1)).ok()?;
    let mut previous_byte = [0_u8; 1];
    file.read_exact(&mut previous_byte).ok()?;
    file.seek(SeekFrom::Start(tail_start)).ok()?;

    let mut reader = BufReader::new(file);
    if previous_byte[0] != b'\n' {
        let mut partial_line = Vec::new();
        reader.read_until(b'\n', &mut partial_line).ok()?;
    }
    Some(reader)
}

fn parse_kimi_wire_file(path: &Path) -> Vec<AiConversationMessage> {
    let Some(reader) = open_conversation_tail(path) else {
        return Vec::new();
    };
    let mut messages: Vec<(usize, AiConversationMessage)> = Vec::new();
    // Model names are not part of the content lines; each assistant step is
    // followed by a `usage.record` line carrying the `model` that produced it.
    let mut usage_records: Vec<(usize, String)> = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let Ok(line) = line else {
            continue;
        };
        if line.len() > MAX_KIMI_RECORD_BYTES {
            continue;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
            continue;
        };
        if value.get("type").and_then(Value::as_str) == Some("usage.record") {
            if let Some(model) = value.get("model").and_then(Value::as_str) {
                if !model.trim().is_empty() {
                    usage_records.push((idx, model.to_string()));
                }
            }
            continue;
        }
        messages.extend(
            parse_kimi_wire_value(&value, idx)
                .into_iter()
                .map(|message| (idx, message)),
        );
    }
    // Backfill the model onto assistant messages from the nearest usage
    // record: the first one at or after the message's line (usage is recorded
    // after the step it describes), falling back to the last one before it.
    messages
        .into_iter()
        .map(|(idx, mut message)| {
            if message.role == AiConversationRole::Model {
                let record = usage_records
                    .iter()
                    .find(|(record_idx, _)| *record_idx >= idx)
                    .or_else(|| {
                        usage_records
                            .iter()
                            .rev()
                            .find(|(record_idx, _)| *record_idx < idx)
                    });
                if let Some((_, model)) = record {
                    message.model = model.clone();
                }
            }
            message
        })
        .collect()
}

/// Parse one line of a Kimi Code wire.jsonl transcript.
///
/// Relevant record types:
/// - `turn.prompt` — user input (`input` is an array of text parts)
/// - `context.append_loop_event` with event type `content.part` — assistant
///   output; part type `text` is a reply, `think` is reasoning
/// - `usage.record` — token usage; its `model` field names the model that
///   produced the preceding assistant step (handled by the caller)
///
/// (`context.append_message` also carries the user message, but it duplicates
/// `turn.prompt`, so only the latter is used.)
fn parse_kimi_wire_value(value: &Value, idx: usize) -> Vec<AiConversationMessage> {
    let timestamp = value
        .get("time")
        .and_then(Value::as_u64)
        .map(timestamp_millis_to_secs)
        .map(format_unix_timestamp_secs)
        .unwrap_or_default();
    match value.get("type").and_then(Value::as_str) {
        Some("turn.prompt") => {
            let text = value
                .get("input")
                .and_then(Value::as_array)
                .map(|parts| {
                    parts
                        .iter()
                        .filter(|part| part.get("type").and_then(Value::as_str) == Some("text"))
                        .filter_map(|part| part.get("text").and_then(Value::as_str))
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_default();
            if text.trim().is_empty() {
                return Vec::new();
            }
            vec![AiConversationMessage {
                id: format!("kimi-{idx}"),
                role: AiConversationRole::User,
                kind: AiConversationMessageKind::Text,
                time: timestamp,
                model: String::new(),
                markdown: text,
            }]
        }
        Some("context.append_loop_event") => {
            let Some(event) = value.get("event") else {
                return Vec::new();
            };
            if event.get("type").and_then(Value::as_str) != Some("content.part") {
                return Vec::new();
            }
            let Some(part) = event.get("part") else {
                return Vec::new();
            };
            let (kind, text) = match part.get("type").and_then(Value::as_str) {
                Some("text") => (
                    AiConversationMessageKind::Text,
                    part.get("text").and_then(Value::as_str),
                ),
                Some("think") => (
                    AiConversationMessageKind::Thinking,
                    part.get("think").and_then(Value::as_str),
                ),
                _ => return Vec::new(),
            };
            let Some(text) = text else {
                return Vec::new();
            };
            if text.trim().is_empty() {
                return Vec::new();
            }
            let id = event
                .get("uuid")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| format!("kimi-{idx}"));
            vec![AiConversationMessage {
                id,
                role: AiConversationRole::Model,
                kind,
                time: timestamp,
                model: String::new(),
                markdown: text.to_string(),
            }]
        }
        _ => Vec::new(),
    }
}

fn parse_claude_message_value(value: &Value, idx: usize) -> Vec<AiConversationMessage> {
    let Some(top_type) = value.get("type").and_then(Value::as_str) else {
        return Vec::new();
    };
    let timestamp = value
        .get("timestamp")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let uuid = value
        .get("uuid")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| format!("message-{idx}"));

    if top_type == "user" {
        let Some(content) = value
            .get("message")
            .and_then(|message| message.get("content"))
            .and_then(Value::as_str)
        else {
            return Vec::new();
        };
        if content.trim().is_empty() {
            return Vec::new();
        }
        return vec![AiConversationMessage {
            id: uuid,
            role: AiConversationRole::User,
            kind: AiConversationMessageKind::Text,
            time: timestamp,
            model: String::new(),
            markdown: content.to_string(),
        }];
    }

    let Some(message) = value.get("message") else {
        return Vec::new();
    };
    if message.get("type").and_then(Value::as_str) != Some("message")
        || message.get("role").and_then(Value::as_str) != Some("assistant")
    {
        return Vec::new();
    }

    let model = message
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let Some(contents) = message.get("content").and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for (content_idx, content) in contents.iter().enumerate() {
        let Some(content_type) = content.get("type").and_then(Value::as_str) else {
            continue;
        };
        let (kind, text) = match content_type {
            "thinking" => (
                AiConversationMessageKind::Thinking,
                content.get("thinking").and_then(Value::as_str),
            ),
            "text" => (
                AiConversationMessageKind::Text,
                content.get("text").and_then(Value::as_str),
            ),
            _ => continue,
        };
        let Some(text) = text else {
            continue;
        };
        if text.trim().is_empty() {
            continue;
        }
        out.push(AiConversationMessage {
            id: format!("{uuid}-{content_idx}"),
            role: AiConversationRole::Model,
            kind,
            time: timestamp.clone(),
            model: model.to_string(),
            markdown: text.to_string(),
        });
    }
    out
}

fn parse_message_value(
    value: &Value,
    idx: usize,
    model_provider: &str,
) -> Option<AiConversationMessage> {
    let role = find_string_for_keys(value, &["role", "type", "speaker"])?;
    let role = match role.to_ascii_lowercase().as_str() {
        "user" | "human" => AiConversationRole::User,
        "assistant" | "model" | "agent" => AiConversationRole::Model,
        _ => return None,
    };
    let markdown = extract_text(value)?;
    if markdown.trim().is_empty() {
        return None;
    }
    let kind = if find_string_for_keys(value, &["type", "kind"])
        .is_some_and(|kind| kind.eq_ignore_ascii_case("thinking"))
    {
        AiConversationMessageKind::Thinking
    } else {
        AiConversationMessageKind::Text
    };
    Some(AiConversationMessage {
        id: find_string_for_keys(value, &["uuid", "id"])
            .unwrap_or_else(|| format!("message-{idx}")),
        role,
        kind,
        time: find_string_for_keys(value, &["timestamp", "created_at", "time"]).unwrap_or_default(),
        model: if role == AiConversationRole::Model {
            model_provider.to_string()
        } else {
            String::new()
        },
        markdown,
    })
}

fn extract_text(value: &Value) -> Option<String> {
    for key in ["content", "text", "message"] {
        if let Some(text) = value.get(key).and_then(value_to_text) {
            return Some(text);
        }
    }
    match value {
        Value::Object(map) => {
            for nested in map.values() {
                if let Some(text) = extract_text(nested) {
                    return Some(text);
                }
            }
            None
        }
        _ => None,
    }
}

fn value_to_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Array(items) => {
            let parts = items
                .iter()
                .filter_map(value_to_text)
                .filter(|text| !text.trim().is_empty())
                .collect::<Vec<_>>();
            (!parts.is_empty()).then(|| parts.join("\n"))
        }
        Value::Object(map) => {
            if let Some(text) = map.get("text").and_then(Value::as_str) {
                return Some(text.to_string());
            }
            if let Some(text) = map.get("content").and_then(value_to_text) {
                return Some(text);
            }
            None
        }
        _ => None,
    }
}

fn find_string_for_keys(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            for key in keys {
                if let Some(text) = map.get(*key).and_then(Value::as_str) {
                    return Some(text.to_string());
                }
            }
            for nested in map.values() {
                if let Some(text) = find_string_for_keys(nested, keys) {
                    return Some(text);
                }
            }
            None
        }
        Value::Array(items) => items
            .iter()
            .find_map(|item| find_string_for_keys(item, keys)),
        _ => None,
    }
}

fn conversation_file_matches_working_path(path: &Path, working_path: &str) -> bool {
    let Ok(contents) = fs::read_to_string(path) else {
        return false;
    };
    let needle = normalize_path_text(working_path);
    contents.lines().take(100).any(|line| {
        serde_json::from_str::<Value>(line)
            .ok()
            .and_then(|value| find_string_for_keys(&value, &["cwd", "current_dir", "working_dir"]))
            .is_some_and(|path| is_path_within_project(&normalize_path_text(&path), &needle))
    })
}

fn list_jsonl_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = fs::read_dir(dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("jsonl"))
        .collect::<Vec<_>>();
    sort_paths_by_mtime_desc(&mut files);
    files
}

fn collect_jsonl_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut queue = VecDeque::from([(root.to_path_buf(), 0usize)]);
    while let Some((dir, depth)) = queue.pop_front() {
        if depth > MAX_SCAN_DEPTH || out.len() >= MAX_SCAN_FILES {
            continue;
        }
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if metadata.is_dir() {
                queue.push_back((path, depth + 1));
            } else if metadata.is_file()
                && path.extension().and_then(|ext| ext.to_str()) == Some("jsonl")
            {
                out.push(path);
            }
        }
    }
    sort_paths_by_mtime_desc(&mut out);
    out
}

fn sort_paths_by_mtime_desc(paths: &mut [PathBuf]) {
    paths.sort_by_key(|path| std::cmp::Reverse(metadata_modified_secs(path)));
}

fn claude_project_dir_name(path: &str) -> String {
    path.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect()
}

fn normalize_path_text(path: &str) -> String {
    strip_windows_extended_path_prefix(&expand_home_path_text(path))
        .replace('\\', "/")
        .to_ascii_lowercase()
}

fn strip_windows_extended_path_prefix(path: &str) -> String {
    if let Some(rest) = path.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{}", rest);
    }
    if let Some(rest) = path.strip_prefix(r"\\?\") {
        return rest.to_string();
    }
    path.to_string()
}
fn expand_home_path_text(path: &str) -> String {
    if path == "~" {
        if let Some(home) = user_home_dir() {
            return home.to_string_lossy().into_owned();
        }
    }

    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = user_home_dir() {
            return home.join(rest).to_string_lossy().into_owned();
        }
    }

    #[cfg(windows)]
    if let Some(rest) = path.strip_prefix(r"~\") {
        if let Some(home) = user_home_dir() {
            return home.join(rest).to_string_lossy().into_owned();
        }
    }

    path.to_string()
}

/// Check whether `path` is the same directory as `project` or a subdirectory
/// of it. Uses normalized path prefix matching with a separator boundary to
/// avoid false positives (e.g. `/proj-1` matching `/proj-12`).
///
/// The match is one-directional: only `path` being inside `project` counts.
/// A `path` that is an *ancestor* of `project` (e.g. `/home/user` vs project
/// `/home/user/proj`) deliberately does not match, so a conversation whose cwd
/// is a parent directory is not surfaced for every nested project session.
fn is_path_within_project(path: &str, project: &str) -> bool {
    if path == project {
        return true;
    }
    let with_slash = |p: &str| {
        if p.ends_with('/') {
            p.to_string()
        } else {
            format!("{}/", p)
        }
    };
    path.starts_with(&with_slash(project))
}

fn brief_text(text: &str) -> String {
    let mut brief = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if let Some((idx, _)) = brief.char_indices().nth(80) {
        brief.truncate(idx);
    }
    brief
}

fn metadata_time(path: &Path) -> String {
    format_unix_timestamp_secs(metadata_modified_secs(path))
}

fn metadata_modified_secs_opt(path: &Path) -> Option<u64> {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
}

fn metadata_modified_secs(path: &Path) -> u64 {
    metadata_modified_secs_opt(path).unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0)
    })
}

fn timestamp_millis_to_secs(timestamp: u64) -> u64 {
    if timestamp > 10_000_000_000 {
        timestamp / 1000
    } else {
        timestamp
    }
}

fn format_unix_timestamp_secs(timestamp: u64) -> String {
    if timestamp == 0 {
        return String::new();
    }
    let days = (timestamp / 86_400) as i64;
    let seconds_of_day = timestamp % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}")
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    (year as i32, month as u32, day as u32)
}

pub(super) fn user_home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

fn open_sqlite_readonly(path: &Path) -> rusqlite::Result<Connection> {
    Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_qwen_messages_keeps_display_text_thinking_text_and_model() {
        let user = serde_json::json!({
            "uuid": "qwen-user",
            "sessionId": "session",
            "timestamp": "2026-08-15T10:00:00.000Z",
            "type": "user",
            "cwd": "/tmp/project",
            "message": {"role": "user", "parts": [{"text": "expanded prompt"}]},
            "systemPayload": {"displayText": "original prompt"}
        });
        let assistant = serde_json::json!({
            "uuid": "qwen-assistant",
            "sessionId": "session",
            "timestamp": "2026-08-15T10:00:01.000Z",
            "type": "assistant",
            "model": "qwen3-coder-plus",
            "message": {"role": "model", "parts": [
                {"text": "inspect first", "thought": true},
                {"text": "done"},
                {"functionCall": {"name": "read_file", "args": {}}}
            ]}
        });
        let subtype = serde_json::json!({
            "uuid": "qwen-command",
            "type": "user",
            "subtype": "slash_command",
            "message": {"role": "user", "parts": [{"text": "ignored"}]}
        });
        let sidechain = serde_json::json!({
            "uuid": "qwen-sidechain",
            "type": "assistant",
            "isSidechain": true,
            "message": {"role": "model", "parts": [{"text": "ignored"}]}
        });

        let users = parse_qwen_message_value(&user, 0);
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].id, "qwen-user");
        assert_eq!(users[0].role, AiConversationRole::User);
        assert_eq!(users[0].markdown, "original prompt");
        let messages = parse_qwen_message_value(&assistant, 1);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].id, "qwen-assistant-0");
        assert_eq!(messages[0].kind, AiConversationMessageKind::Thinking);
        assert_eq!(messages[0].markdown, "inspect first");
        assert_eq!(messages[0].model, "qwen3-coder-plus");
        assert_eq!(messages[1].kind, AiConversationMessageKind::Text);
        assert_eq!(messages[1].markdown, "done");
        assert!(parse_qwen_message_value(&subtype, 2).is_empty());
        assert!(parse_qwen_message_value(&sidechain, 3).is_empty());
    }

    #[test]
    fn qwen_listing_and_resolution_are_scoped_to_the_current_project() {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("project");
        let other = temp.path().join("other");
        fs::create_dir(&project).unwrap();
        fs::create_dir(&other).unwrap();
        let chats = temp.path().join("chats");
        fs::create_dir(&chats).unwrap();
        let session_id = "550e8400-e29b-41d4-a716-446655440000";
        let records = [
            serde_json::json!({
                "uuid": "user", "sessionId": session_id,
                "timestamp": "2026-08-15T10:00:00.000Z", "type": "user",
                "cwd": project.to_string_lossy(),
                "message": {"role": "user", "parts": [{"text": "first prompt"}]}
            }),
            serde_json::json!({
                "uuid": "title", "sessionId": session_id,
                "timestamp": "2026-08-15T10:00:02.000Z", "type": "system",
                "subtype": "custom_title", "cwd": project.to_string_lossy(),
                "systemPayload": {"customTitle": "Qwen title"}
            }),
        ];
        let path = chats.join(format!("{session_id}.jsonl"));
        fs::write(
            &path,
            records
                .iter()
                .map(Value::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .unwrap();
        let other_id = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
        let other_record = serde_json::json!({
            "uuid": "other", "sessionId": other_id, "type": "user",
            "cwd": other.to_string_lossy(),
            "message": {"role": "user", "parts": [{"text": "other project"}]}
        });
        fs::write(
            chats.join(format!("{other_id}.jsonl")),
            format!("{other_record}\n"),
        )
        .unwrap();

        let items = list_qwen_conversations_at_root(&chats, &project.to_string_lossy());

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, session_id);
        assert_eq!(items[0].brief, "Qwen title");
        assert_eq!(
            resolve_qwen_conversation_path_at_root(&chats, &project.to_string_lossy(), session_id),
            fs::canonicalize(&path).ok()
        );
        assert!(resolve_qwen_conversation_path_at_root(
            &chats,
            &other.to_string_lossy(),
            session_id
        )
        .is_none());
        assert!(resolve_qwen_conversation_path_at_root(
            &chats,
            &project.to_string_lossy(),
            "../other"
        )
        .is_none());
    }

    #[test]
    fn parse_dsh_messages_keeps_real_prompts_text_reasoning_and_model() {
        let runtime = serde_json::json!({
            "type": "user/message",
            "data": {
                "role": "user", "id": "runtime", "source": {"kind": "runtime-context"},
                "content": [{"type": "text", "text": "runtime only"}]
            }
        });
        let user = serde_json::json!({
            "type": "user/message",
            "data": {
                "role": "user", "id": "user-1", "source": {"kind": "user"},
                "content": [{"type": "text", "text": "hello dsh"}]
            }
        });
        let assistant = serde_json::json!({
            "type": "assistant/message",
            "data": {
                "message": {
                    "role": "assistant", "id": "assistant-1",
                    "source": {"kind": "model", "model": "deepseek-v4-flash"},
                    "content": [
                        {"type": "reasoning", "text": "inspect first"},
                        {"type": "text", "text": "done"},
                        {"type": "tool-call", "name": "ignored"}
                    ]
                },
                "stream": [{"type": "chunk", "time": 1_786_788_001_000_u64}]
            }
        });
        let tool = serde_json::json!({"type": "tool/result", "data": {}});

        assert!(parse_dsh_message_value(&runtime, 0).is_empty());
        assert!(parse_dsh_message_value(&tool, 0).is_empty());
        let users = parse_dsh_message_value(&user, 1);
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].id, "user-1");
        assert_eq!(users[0].role, AiConversationRole::User);
        assert_eq!(users[0].markdown, "hello dsh");
        let messages = parse_dsh_message_value(&assistant, 2);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].id, "assistant-1-0");
        assert_eq!(messages[0].kind, AiConversationMessageKind::Thinking);
        assert_eq!(messages[0].markdown, "inspect first");
        assert_eq!(messages[0].model, "deepseek-v4-flash");
        assert_eq!(messages[0].time, "2026-08-15 10:00");
        assert_eq!(messages[1].kind, AiConversationMessageKind::Text);
        assert_eq!(messages[1].markdown, "done");
    }

    #[test]
    fn dsh_listing_scopes_to_exact_project_without_filtering_content() {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("project");
        let other = temp.path().join("other");
        let nested = project.join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::create_dir(&other).unwrap();
        let root = temp.path().join("sessions");
        let write_session = |id: &str, cwd: &Path, origin: Option<&str>, events: &[Value]| {
            let session_dir = root.join("--project--").join(id);
            fs::create_dir_all(&session_dir).unwrap();
            let header = serde_json::json!({
                "type": "session", "version": 4, "id": id, "cwd": cwd.to_string_lossy(),
                "origin": origin,
            });
            let log = std::iter::once(header)
                .chain(events.iter().cloned())
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            let path = session_dir.join("session.v4.jsonl");
            fs::write(&path, log).unwrap();
            path
        };
        let user_event = |text: &str| serde_json::json!({"type":"user/message","data":{"role":"user","id":"user","source":{"kind":"user"},"content":[{"type":"text","text":text}]}});
        let path = write_session(
            "dsh-id",
            &project,
            None,
            &[
                serde_json::json!({"type":"user/message","data":{"role":"user","id":"runtime","source":{"kind":"runtime-context"},"content":[{"type":"text","text":"runtime"}]}}),
                user_event("first prompt"),
                serde_json::json!({"type":"session/title","data":{"title":"Latest DSH title"}}),
            ],
        );
        write_session(
            "assistant-id",
            &project,
            None,
            &[
                serde_json::json!({"type":"assistant/message","data":{"message":{"role":"assistant","id":"assistant","source":{"kind":"model"},"content":[{"type":"text","text":"reply"}]}}}),
            ],
        );
        for (id, cwd) in [("other-id", &other), ("nested-id", &nested)] {
            write_session(id, cwd, None, &[user_event("other project")]);
        }
        write_session("empty-id", &project, None, &[]);
        write_session(
            "title-only-id",
            &project,
            None,
            &[serde_json::json!({"type":"session/title","data":{"title":"No conversation"}})],
        );
        write_session(
            "runtime-only-id",
            &project,
            None,
            &[
                serde_json::json!({"type":"user/message","data":{"role":"user","id":"runtime","source":{"kind":"runtime-context"},"content":[{"type":"text","text":"runtime"}]}}),
            ],
        );
        write_session(
            "child-id",
            &project,
            Some("subagent"),
            &[user_event("subagent")],
        );

        let items = list_dsh_conversations_at_root(&root, &project.to_string_lossy());
        let mut ids = items
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>();
        ids.sort_unstable();
        assert_eq!(
            ids,
            [
                "assistant-id",
                "dsh-id",
                "empty-id",
                "runtime-only-id",
                "title-only-id",
            ]
        );
        assert_eq!(
            items.iter().find(|item| item.id == "dsh-id").unwrap().brief,
            "Latest DSH title"
        );
        assert_eq!(
            items
                .iter()
                .find(|item| item.id == "empty-id")
                .unwrap()
                .brief,
            "empty-id"
        );
        assert!(is_same_dsh_project(
            r"D:\Projects\Test\",
            r"d:\projects\test"
        ));
        assert!(!is_same_dsh_project(
            r"D:\Projects\Test\Nested",
            r"D:\Projects\Test"
        ));
        assert_eq!(
            resolve_dsh_conversation_path_at_root(&root, &project.to_string_lossy(), "dsh-id"),
            fs::canonicalize(&path).ok()
        );
        assert!(resolve_dsh_conversation_path_at_root(
            &root,
            &project.to_string_lossy(),
            "child-id"
        )
        .is_none());
        assert!(
            resolve_dsh_conversation_path_at_root(&root, &other.to_string_lossy(), "dsh-id")
                .is_none()
        );
        assert!(resolve_dsh_conversation_path_at_root(
            &root,
            &project.to_string_lossy(),
            "nested-id"
        )
        .is_none());
        assert!(resolve_dsh_conversation_path_at_root(
            &root,
            &project.to_string_lossy(),
            "../dsh-id"
        )
        .is_none());
    }

    /// v2 databases keep sessions in `session_v2`; the v1 `session` table is
    /// a migration leftover. The list query must read both (dedup happens in
    /// the caller), skip archived and subagent rows, and tolerate NULL titles.
    #[test]
    fn opencode_session_rows_merge_v2_and_v1_tables() {
        let connection = Connection::open_in_memory().unwrap();
        for table in ["session_v2", "session"] {
            connection
                .execute(
                    &format!(
                        "CREATE TABLE {table} (id TEXT, title TEXT, directory TEXT, \
                         parent_id TEXT, time_archived INTEGER, time_updated INTEGER)"
                    ),
                    [],
                )
                .unwrap();
        }
        let dir = if cfg!(windows) {
            r"D:\projects\vibe"
        } else {
            "/home/dev/projects/vibe"
        };
        connection
            .execute(
                "INSERT INTO session_v2 VALUES ('ses_v2', NULL, ?1, NULL, NULL, 2000)",
                [dir],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO session_v2 VALUES ('ses_archived', 'gone', ?1, NULL, 1, 3000)",
                [dir],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO session_v2 VALUES ('ses_child', 'sub', ?1, 'ses_v2', NULL, 4000)",
                [dir],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO session VALUES ('ses_v1', 'legacy', ?1, NULL, NULL, 1000)",
                [dir],
            )
            .unwrap();

        let (clause, values) = opencode_directory_match(dir);
        let rows = query_opencode_session_rows(&connection, &clause, &values);

        let ids = rows.iter().map(|row| row.0.as_str()).collect::<Vec<_>>();
        assert!(ids.contains(&"ses_v2"));
        assert!(ids.contains(&"ses_v1"));
        assert!(!ids.contains(&"ses_archived"));
        assert!(!ids.contains(&"ses_child"));
        let v2 = rows.iter().find(|row| row.0 == "ses_v2").unwrap();
        assert_eq!(v2.1, None);
        assert_eq!(v2.2, 2000);
    }

    /// A v1-only database has no `session_v2` table; the query must fall back
    /// to `session` instead of failing.
    #[test]
    fn opencode_session_rows_fall_back_to_v1_table() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute(
                "CREATE TABLE session (id TEXT, title TEXT, directory TEXT, \
                 parent_id TEXT, time_archived INTEGER, time_updated INTEGER)",
                [],
            )
            .unwrap();
        let dir = if cfg!(windows) {
            r"D:\projects\vibe"
        } else {
            "/home/dev/projects/vibe"
        };
        connection
            .execute(
                "INSERT INTO session VALUES ('ses_v1', 'legacy', ?1, NULL, NULL, 1000)",
                [dir],
            )
            .unwrap();

        let (clause, values) = opencode_directory_match(dir);
        let rows = query_opencode_session_rows(&connection, &clause, &values);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "ses_v1");
        assert_eq!(rows[0].1.as_deref(), Some("legacy"));
    }

    /// Build an in-memory SQLite database whose `threads` table mirrors the
    /// columns the Codex query selects, seeded with the given (id, cwd) rows.
    fn codex_test_db(rows: &[(&str, &str)]) -> Connection {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute(
                "CREATE TABLE threads (id TEXT, title TEXT, cwd TEXT, archived INTEGER, \
                 updated_at INTEGER, updated_at_ms INTEGER, rollout_path TEXT, model_provider TEXT)",
                [],
            )
            .unwrap();
        for (id, cwd) in rows {
            connection
                .execute(
                    "INSERT INTO threads (id, cwd, archived) VALUES (?1, ?2, 0)",
                    rusqlite::params![id, cwd],
                )
                .unwrap();
        }
        connection
    }

    #[cfg(windows)]
    #[test]
    fn opencode_directory_match_accepts_forward_slashes() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute("CREATE TABLE session (id TEXT, directory TEXT)", [])
            .unwrap();
        connection
            .execute(
                "INSERT INTO session VALUES ('ses_target', 'D:/projects/acme/vibe-terminal')",
                [],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO session VALUES ('ses_other', 'D:/projects/acme/other')",
                [],
            )
            .unwrap();

        let (clause, values) = opencode_directory_match(r"D:\projects\acme\vibe-terminal");
        let mut statement = connection
            .prepare(&format!("SELECT id FROM session WHERE {clause}"))
            .unwrap();
        let ids = statement
            .query_map(rusqlite::params_from_iter(values.iter()), |row| {
                row.get::<_, String>(0)
            })
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["ses_target"]);
    }

    #[test]
    fn codex_cwd_match_matches_both_prefix_forms() {
        // On Windows the DB may store `\\?\C:\...` while the terminal path is
        // bare `C:\...`, or the reverse; on Posix paths are stored verbatim.
        // In both cases the rows for the same project match, regardless of
        // which side carries the prefix, and the comparison is case-insensitive.
        let (target, sibling) = if cfg!(windows) {
            (
                r"C:\Users\dev\projects\vibe",
                r"C:\Users\dev\projects\other",
            )
        } else {
            ("/home/dev/projects/vibe", "/home/dev/projects/other")
        };

        // Seed two rows for the target (bare and prefixed, different case) plus
        // an unrelated sibling project.
        let (bare, prefixed, mixed_case) = if cfg!(windows) {
            (
                target.to_string(),
                format!(r"\\?\{target}"),
                target.to_uppercase(),
            )
        } else {
            // No extended-length prefix on Posix; reuse the same value so the
            // "prefixed" row still matches, and vary case for NOCASE coverage.
            (
                target.to_string(),
                target.to_string(),
                target.to_uppercase(),
            )
        };
        let connection = codex_test_db(&[
            ("bare", &bare),
            ("prefixed", &prefixed),
            ("mixed_case", &mixed_case),
            ("other", sibling),
        ]);

        let (clause, values) = codex_cwd_match(target);
        let mut statement = connection
            .prepare(&format!(
                "SELECT id FROM threads WHERE archived = 0 AND cwd IS NOT NULL AND ({clause}) \
                 ORDER BY id"
            ))
            .unwrap();
        let ids: Vec<String> = statement
            .query_map(rusqlite::params_from_iter(values.iter()), |row| {
                row.get::<_, String>(0)
            })
            .unwrap()
            .map(|row| row.unwrap())
            .collect();

        assert_eq!(ids, vec!["bare", "mixed_case", "prefixed"]);
    }

    #[test]
    fn codex_cwd_match_rejects_sibling_project() {
        // Two distinct project directories on the same system must not be
        // confused for each other, on both platforms.
        let (target, sibling) = if cfg!(windows) {
            (
                r"C:\Users\dev\projects\vibe",
                r"C:\Users\dev\projects\other",
            )
        } else {
            ("/home/dev/projects/vibe", "/home/dev/projects/other")
        };

        let connection = codex_test_db(&[("sibling", sibling)]);
        let (clause, values) = codex_cwd_match(target);
        let mut statement = connection
            .prepare(&format!(
                "SELECT COUNT(*) FROM threads WHERE archived = 0 AND cwd IS NOT NULL AND ({clause})"
            ))
            .unwrap();
        let count: i64 = statement
            .query_row(rusqlite::params_from_iter(values.iter()), |row| row.get(0))
            .unwrap();

        assert_eq!(count, 0, "sibling project directory should not match");
    }

    #[test]
    fn codex_cwd_match_clause_builder_shape() {
        let target = if cfg!(windows) {
            r"C:\Users\dev\proj"
        } else {
            "/home/dev/proj"
        };
        let (clause, values) = codex_cwd_match(target);

        if cfg!(windows) {
            // Bare value first, then its `\\?\`-prefixed form; both bound as
            // plain `cwd = ?` equalities joined by OR.
            assert_eq!(
                clause,
                "cwd = ? COLLATE NOCASE OR cwd = ? COLLATE NOCASE".to_string()
            );
            assert_eq!(values, vec![target.to_string(), format!(r"\\?\{target}")]);
        } else {
            assert_eq!(clause, "cwd = ? COLLATE NOCASE");
            assert_eq!(values, vec![target]);
        }
    }

    #[test]
    fn is_path_within_project_matches_equal_and_subdirectory() {
        assert!(is_path_within_project("/home/dev/proj", "/home/dev/proj"));
        assert!(is_path_within_project(
            "/home/dev/proj/src",
            "/home/dev/proj"
        ));
    }

    #[test]
    fn is_path_within_project_rejects_sibling_and_unrelated() {
        assert!(!is_path_within_project("/home/dev/other", "/home/dev/proj"));
        // Separator-boundary check: `/proj-1` must not match `/proj`.
        assert!(!is_path_within_project("/home/proj-12", "/home/proj"));
    }

    #[test]
    fn is_path_within_project_rejects_parent_directory() {
        // Regression: a cwd that is an ancestor of the project must not match.
        // Previously the bidirectional check caused `/home/dev` to match every
        // nested project like `/home/dev/proj`.
        assert!(!is_path_within_project("/home/dev", "/home/dev/proj"));
    }

    #[test]
    fn brief_text_truncates_on_utf8_char_boundary() {
        let text = "中文".repeat(60);
        let brief = brief_text(&text);

        assert_eq!(brief.chars().count(), 80);
        assert!(brief.is_char_boundary(brief.len()));
    }

    #[test]
    fn format_unix_timestamp_secs_returns_readable_utc_date() {
        assert_eq!(format_unix_timestamp_secs(0), "");
        assert_eq!(format_unix_timestamp_secs(86_400), "1970-01-02 00:00");
    }

    #[test]
    fn normalize_path_text_expands_home_prefix() {
        let Some(home) = user_home_dir() else {
            return;
        };
        let expected = home
            .join("Documents/projects/vibe-terminal")
            .to_string_lossy()
            .replace('\\', "/")
            .to_ascii_lowercase();

        assert_eq!(
            normalize_path_text("~/Documents/projects/vibe-terminal"),
            expected
        );
    }

    #[test]
    fn claude_project_dir_name_uses_expanded_home_path() {
        let Some(home) = user_home_dir() else {
            return;
        };
        let expanded = home.join("Documents/projects/vibe-terminal");

        assert_eq!(
            claude_project_dir_name(&expand_home_path_text("~/Documents/projects/vibe-terminal")),
            claude_project_dir_name(&expanded.to_string_lossy())
        );
    }

    #[test]
    fn parse_pi_v3_user_message() {
        let value = serde_json::json!({
            "type": "message",
            "id": "entry-user",
            "timestamp": "2026-08-15T10:00:00.000Z",
            "message": {
                "role": "user",
                "content": [
                    { "type": "text", "text": "hello pi" },
                    { "type": "image", "data": "ignored", "mimeType": "image/png" }
                ],
                "timestamp": 1786788000000_u64
            }
        });

        let messages = parse_pi_message_value(&value, 0);

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, "entry-user");
        assert_eq!(messages[0].role, AiConversationRole::User);
        assert_eq!(messages[0].kind, AiConversationMessageKind::Text);
        assert_eq!(messages[0].markdown, "hello pi");
        assert_eq!(messages[0].model, "");
    }

    #[test]
    fn parse_pi_session_header_supports_v3_and_v4() {
        let temp = tempfile::tempdir().unwrap();
        let v3 = temp.path().join("v3.jsonl");
        let v4 = temp.path().join("v4.jsonl");
        fs::write(
            &v3,
            r#"{"type":"session","version":3,"id":"v3-id","timestamp":"2026-08-15T10:00:00Z","cwd":"/tmp/project"}"#,
        )
        .unwrap();
        fs::write(
            &v4,
            r#"{"kind":"header","version":4,"id":"v4.id","createdAt":1786788000000,"cwd":"/tmp/project"}"#,
        )
        .unwrap();

        let v3 = parse_pi_session_header(&v3).unwrap();
        let v4 = parse_pi_session_header(&v4).unwrap();

        assert_eq!(v3.id, "v3-id");
        assert_eq!(v3.cwd, "/tmp/project");
        assert_eq!(v4.id, "v4.id");
        assert_eq!(v4.cwd, "/tmp/project");
    }

    #[test]
    fn parse_pi_v4_assistant_thinking_and_text() {
        let value = serde_json::json!({
            "kind": "entry",
            "seq": 2,
            "type": "message",
            "id": "entry-assistant",
            "parentId": "entry-user",
            "timestamp": 1786788000000_u64,
            "message": {
                "role": "assistant",
                "model": "claude-sonnet-4-5",
                "content": [
                    { "type": "thinking", "thinking": "inspect the project" },
                    { "type": "text", "text": "Done." },
                    { "type": "toolCall", "id": "call-1", "name": "read", "arguments": {} }
                ],
                "timestamp": 1786788000000_u64
            }
        });

        let messages = parse_pi_message_value(&value, 0);

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].id, "entry-assistant-0");
        assert_eq!(messages[0].role, AiConversationRole::Model);
        assert_eq!(messages[0].kind, AiConversationMessageKind::Thinking);
        assert_eq!(messages[0].markdown, "inspect the project");
        assert_eq!(messages[0].model, "claude-sonnet-4-5");
        assert_eq!(messages[1].id, "entry-assistant-1");
        assert_eq!(messages[1].kind, AiConversationMessageKind::Text);
        assert_eq!(messages[1].markdown, "Done.");
    }

    #[test]
    fn parse_opencode_current_message_projection() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE session_message (\
                    id TEXT, session_id TEXT, type TEXT, seq INTEGER, \
                    time_created INTEGER, data TEXT\
                 );",
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO session_message VALUES (?1, 'ses_test', 'user', 1, ?2, ?3)",
                rusqlite::params![
                    "msg_user",
                    1_786_788_000_000_i64,
                    r#"{"time":{"created":1786788000000},"text":"hello OpenCode","files":[],"agents":[]}"#
                ],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO session_message VALUES (?1, 'ses_test', 'assistant', 2, ?2, ?3)",
                rusqlite::params![
                    "msg_assistant",
                    1_786_788_001_000_i64,
                    r#"{"time":{"created":1786788001000},"model":{"id":"gpt-5","providerID":"openai"},"content":[{"type":"reasoning","id":"reason-1","text":"inspect"},{"type":"text","id":"text-1","text":"Done."}]}"#
                ],
            )
            .unwrap();

        let messages = parse_opencode_current_messages(&connection, "ses_test").unwrap();

        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].role, AiConversationRole::User);
        assert_eq!(messages[0].markdown, "hello OpenCode");
        assert_eq!(messages[1].kind, AiConversationMessageKind::Thinking);
        assert_eq!(messages[1].model, "gpt-5");
        assert_eq!(messages[2].kind, AiConversationMessageKind::Text);
        assert_eq!(messages[2].markdown, "Done.");
    }

    #[test]
    fn parse_opencode_legacy_message_and_part_projection() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE message (\
                    id TEXT, session_id TEXT, time_created INTEGER, data TEXT\
                 );\
                 CREATE TABLE part (\
                    id TEXT, message_id TEXT, time_created INTEGER, data TEXT\
                 );",
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO message VALUES ('msg_user', 'ses_test', ?1, ?2)",
                rusqlite::params![
                    1_786_788_000_000_i64,
                    r#"{"role":"user","time":{"created":1786788000000}}"#
                ],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO message VALUES ('msg_assistant', 'ses_test', ?1, ?2)",
                rusqlite::params![
                    1_786_788_001_000_i64,
                    r#"{"role":"assistant","modelID":"claude-sonnet-4","time":{"created":1786788001000}}"#
                ],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO part VALUES ('part_user', 'msg_user', ?1, ?2)",
                rusqlite::params![
                    1_786_788_000_000_i64,
                    r#"{"type":"text","text":"legacy prompt"}"#
                ],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO part VALUES ('part_reasoning', 'msg_assistant', ?1, ?2)",
                rusqlite::params![
                    1_786_788_001_000_i64,
                    r#"{"type":"reasoning","text":"legacy thought"}"#
                ],
            )
            .unwrap();

        let messages = parse_opencode_legacy_messages(&connection, "ses_test").unwrap();

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, AiConversationRole::User);
        assert_eq!(messages[0].markdown, "legacy prompt");
        assert_eq!(messages[1].role, AiConversationRole::Model);
        assert_eq!(messages[1].kind, AiConversationMessageKind::Thinking);
        assert_eq!(messages[1].model, "claude-sonnet-4");
    }

    #[test]
    fn parse_claude_user_message_with_string_content() {
        let value = serde_json::json!({
            "type": "user",
            "uuid": "user-1",
            "timestamp": "2026-05-09T07:37:46.163Z",
            "message": {
                "role": "user",
                "content": "hello claude"
            }
        });

        let messages = parse_claude_message_value(&value, 0);

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, "user-1");
        assert_eq!(messages[0].role, AiConversationRole::User);
        assert_eq!(messages[0].kind, AiConversationMessageKind::Text);
        assert_eq!(messages[0].model, "");
        assert_eq!(messages[0].markdown, "hello claude");
    }

    #[test]
    fn parse_claude_assistant_thinking_and_text_messages() {
        let value = serde_json::json!({
            "type": "assistant",
            "uuid": "assistant-1",
            "timestamp": "2026-05-09T07:37:50.754Z",
            "message": {
                "id": "msg_1",
                "type": "message",
                "role": "assistant",
                "model": "glm-5.1",
                "content": [
                    {
                        "type": "thinking",
                        "thinking": "I should inspect the file."
                    },
                    {
                        "type": "text",
                        "text": "The button is hidden by layout."
                    },
                    {
                        "type": "tool_use",
                        "name": "Read"
                    }
                ]
            }
        });

        let messages = parse_claude_message_value(&value, 0);

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].id, "assistant-1-0");
        assert_eq!(messages[0].role, AiConversationRole::Model);
        assert_eq!(messages[0].kind, AiConversationMessageKind::Thinking);
        assert_eq!(messages[0].model, "glm-5.1");
        assert_eq!(messages[0].markdown, "I should inspect the file.");
        assert_eq!(messages[1].id, "assistant-1-1");
        assert_eq!(messages[1].role, AiConversationRole::Model);
        assert_eq!(messages[1].kind, AiConversationMessageKind::Text);
        assert_eq!(messages[1].model, "glm-5.1");
        assert_eq!(messages[1].markdown, "The button is hidden by layout.");
    }

    #[test]
    fn parse_claude_ignores_tool_results_and_system_rows() {
        let tool_result = serde_json::json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": [
                    {
                        "type": "tool_result",
                        "content": "ignored"
                    }
                ]
            }
        });
        let system = serde_json::json!({
            "type": "system",
            "content": "ignored"
        });

        assert!(parse_claude_message_value(&tool_result, 0).is_empty());
        assert!(parse_claude_message_value(&system, 1).is_empty());
    }

    #[test]
    fn parse_codex_user_and_agent_event_messages() {
        let user = serde_json::json!({
            "timestamp": "2026-05-09T08:15:17.444Z",
            "type": "event_msg",
            "payload": {
                "type": "user_message",
                "message": "add model field",
                "images": [],
                "local_images": [],
                "text_elements": []
            }
        });
        let agent = serde_json::json!({
            "timestamp": "2026-05-09T08:15:27.786Z",
            "type": "event_msg",
            "payload": {
                "type": "agent_message",
                "message": "I will update the DTO.",
                "phase": "commentary",
                "memory_citation": null
            }
        });

        let user_message = parse_codex_message_value(&user, 0, "openai").unwrap();
        let agent_message = parse_codex_message_value(&agent, 1, "openai").unwrap();

        assert_eq!(user_message.role, AiConversationRole::User);
        assert_eq!(user_message.model, "");
        assert_eq!(user_message.markdown, "add model field");
        assert_eq!(agent_message.role, AiConversationRole::Model);
        assert_eq!(agent_message.model, "openai");
        assert_eq!(agent_message.markdown, "I will update the DTO.");
    }

    #[test]
    fn parse_codex_ignores_non_message_events() {
        let value = serde_json::json!({
            "timestamp": "2026-05-09T08:15:27.786Z",
            "type": "event_msg",
            "payload": {
                "type": "token_count",
                "message": "ignored"
            }
        });

        assert!(parse_codex_message_value(&value, 0, "openai").is_none());
    }

    #[test]
    fn parse_codex_item_completed_user_and_agent_messages() {
        let user = serde_json::json!({
            "timestamp": "2026-08-11T10:57:48.060Z",
            "type": "event_msg",
            "payload": {
                "type": "item_completed",
                "item": {
                    "type": "UserMessage",
                    "id": "019ff08a-fb9c-7b21-83c3-7d5f3b057eac",
                    "content": [{"type": "text", "text": "switch the viewing path", "text_elements": []}]
                }
            }
        });
        let agent = serde_json::json!({
            "timestamp": "2026-08-11T10:41:01.547Z",
            "type": "event_msg",
            "payload": {
                "type": "item_completed",
                "item": {
                    "type": "AgentMessage",
                    "id": "msg_0dbe395c759c8754016a7afcdf02d881989e44b91c368047bb",
                    "content": [{"type": "Text", "text": "Fixed the path comparison."}],
                    "phase": "commentary"
                }
            }
        });
        let reasoning = serde_json::json!({
            "timestamp": "2026-08-11T10:41:01.547Z",
            "type": "event_msg",
            "payload": {
                "type": "item_completed",
                "item": {"type": "Reasoning", "id": "rs_123", "summary_text": [], "raw_content": []}
            }
        });

        let user_message = parse_codex_message_value(&user, 0, "openai").unwrap();
        let agent_message = parse_codex_message_value(&agent, 1, "openai").unwrap();

        assert_eq!(user_message.role, AiConversationRole::User);
        assert_eq!(user_message.model, "");
        assert_eq!(user_message.markdown, "switch the viewing path");
        assert_eq!(agent_message.role, AiConversationRole::Model);
        assert_eq!(agent_message.model, "openai");
        assert_eq!(agent_message.markdown, "Fixed the path comparison.");
        assert!(parse_codex_message_value(&reasoning, 2, "openai").is_none());
    }

    #[test]
    fn parse_kimi_wire_keeps_only_complete_lines_from_latest_limit() {
        use std::io::{Seek, Write};

        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            file,
            "{}",
            serde_json::json!({
                "type": "turn.prompt",
                "input": [{"type": "text", "text": "old history outside the tail"}],
                "time": 1_769_000_000_000_u64
            })
        )
        .unwrap();
        file.as_file_mut().set_len(MAX_FILE_BYTES + 1).unwrap();
        file.as_file_mut().seek(SeekFrom::End(0)).unwrap();
        writeln!(file).unwrap();
        writeln!(
            file,
            "{}",
            serde_json::json!({
                "type": "turn.prompt",
                "input": [{"type": "text", "text": "hello from the recent Kimi tail"}],
                "time": 1_769_000_001_000_u64
            })
        )
        .unwrap();
        writeln!(
            file,
            "{}",
            serde_json::json!({
                "type": "context.append_loop_event",
                "event": {
                    "type": "content.part",
                    "uuid": "assistant-large-1",
                    "part": {"type": "text", "text": "history loaded"}
                },
                "time": 1_769_000_002_000_u64
            })
        )
        .unwrap();
        writeln!(
            file,
            "{}",
            serde_json::json!({
                "type": "usage.record",
                "model": "kimi-k2.5",
                "time": 1_769_000_002_100_u64
            })
        )
        .unwrap();

        let messages = parse_kimi_wire_file(file.path());

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, AiConversationRole::User);
        assert_eq!(messages[0].markdown, "hello from the recent Kimi tail");
        assert_eq!(messages[1].role, AiConversationRole::Model);
        assert_eq!(messages[1].markdown, "history loaded");
        assert_eq!(messages[1].model, "kimi-k2.5");
    }

    #[test]
    fn kimi_conversations_are_ordered_by_wire_modified_time() {
        use std::io::Write;

        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("project");
        let older_session = temp.path().join("older-session");
        let newer_session = temp.path().join("newer-session");

        for (session_dir, prompt, modified_secs) in [
            (&older_session, "older transcript", 1_769_000_000),
            (&newer_session, "newer transcript", 1_769_000_100),
        ] {
            let wire_dir = session_dir.join("agents").join("main");
            fs::create_dir_all(&wire_dir).unwrap();
            let wire_path = wire_dir.join("wire.jsonl");
            let mut wire = fs::File::create(&wire_path).unwrap();
            writeln!(
                wire,
                "{}",
                serde_json::json!({
                    "type": "turn.prompt",
                    "input": [{"type": "text", "text": prompt}]
                })
            )
            .unwrap();
            wire.set_modified(UNIX_EPOCH + std::time::Duration::from_secs(modified_secs))
                .unwrap();
        }

        // Put the older transcript later in the append-only index. The wire
        // mtimes, rather than this index order, must determine the result.
        let entries = vec![
            KimiSessionIndexEntry {
                session_id: "newer".to_string(),
                session_dir: newer_session,
                work_dir: project.display().to_string(),
            },
            KimiSessionIndexEntry {
                session_id: "older".to_string(),
                session_dir: older_session,
                work_dir: project.display().to_string(),
            },
        ];

        let items = kimi_conversation_infos(
            &entries,
            &normalize_path_text(&project.display().to_string()),
        );

        assert_eq!(
            items
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["newer", "older"]
        );
        assert_eq!(items[0].brief, "newer transcript");
        assert_eq!(items[1].brief, "older transcript");
    }
}
