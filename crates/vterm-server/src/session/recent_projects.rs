use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use rusqlite::{Connection, OpenFlags};
use serde_json::Value;
use tokio::sync::watch;
use tracing::{debug, info, warn};
use vterm_common::RecentProjectDirectory;

const MAX_RECENT_PROJECTS: usize = 200;
const MAX_DSH_HEADER_BYTES: u64 = 1024 * 1024;
const MAX_DSH_LOG_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug)]
pub enum RecentProjectScanState {
    Loading,
    Ready(Vec<RecentProjectDirectory>),
}

#[derive(Debug)]
pub(super) struct DshSessionArtifact {
    pub path: PathBuf,
    pub id: String,
    pub cwd: String,
    pub origin: Option<String>,
    pub modified_secs: u64,
}

pub(super) fn dsh_sessions_root(home: &Path) -> PathBuf {
    std::env::var_os("DSH_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".dsh"))
        .join("sessions")
}

fn dsh_log_version(path: &Path) -> Option<u32> {
    let name = path.file_name()?.to_str()?;
    let base = name
        .strip_suffix(".jsonl.zstd")
        .or_else(|| name.strip_suffix(".jsonl"))?;
    if base == "session" {
        return Some(0);
    }
    let version = base.strip_prefix("session.v")?;
    if version.is_empty() || version.starts_with('0') {
        return None;
    }
    version
        .parse::<u32>()
        .ok()
        .filter(|parsed| parsed.to_string() == version)
}

fn read_dsh_prefix(path: &Path, limit: u64) -> Option<Vec<u8>> {
    let file = fs::File::open(path).ok()?;
    let compressed = path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".jsonl.zstd"));
    let mut bytes = Vec::new();
    if compressed {
        let decoder = zstd::stream::read::Decoder::new(file).ok()?;
        decoder.take(limit + 1).read_to_end(&mut bytes).ok()?;
    } else {
        file.take(limit + 1).read_to_end(&mut bytes).ok()?;
    }
    (bytes.len() as u64 <= limit).then_some(bytes)
}

pub(super) fn read_dsh_log(path: &Path) -> Option<Vec<u8>> {
    read_dsh_prefix(path, MAX_DSH_LOG_BYTES)
}

fn read_dsh_header(path: &Path) -> Option<Value> {
    let file = fs::File::open(path).ok()?;
    let compressed = path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".jsonl.zstd"));
    let reader: Box<dyn Read> = if compressed {
        Box::new(zstd::stream::read::Decoder::new(file).ok()?)
    } else {
        Box::new(file)
    };
    let mut line = Vec::new();
    BufReader::new(reader.take(MAX_DSH_HEADER_BYTES + 1))
        .read_until(b'\n', &mut line)
        .ok()?;
    if line.len() as u64 > MAX_DSH_HEADER_BYTES {
        return None;
    }
    let value = serde_json::from_slice::<Value>(&line).ok()?;
    (value.get("type").and_then(Value::as_str) == Some("session")).then_some(value)
}

fn latest_dsh_log(session_dir: &Path) -> Option<PathBuf> {
    let mut logs = fs::read_dir(session_dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter_map(|path| dsh_log_version(&path).map(|version| (version, path)))
        .collect::<Vec<_>>();
    logs.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.file_name().cmp(&right.1.file_name()))
    });
    logs.pop().map(|(_, path)| path)
}

fn dsh_session_artifact(session_dir: &Path) -> Option<DshSessionArtifact> {
    let path = latest_dsh_log(session_dir)?;
    let header = read_dsh_header(&path)?;
    let id = header.get("id")?.as_str()?.to_string();
    let cwd = header.get("cwd")?.as_str()?.to_string();
    let origin = header
        .get("origin")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let modified_secs = fs::metadata(&path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    Some(DshSessionArtifact {
        path,
        id,
        cwd,
        origin,
        modified_secs,
    })
}

pub(super) fn dsh_session_artifacts(sessions_root: &Path) -> Vec<DshSessionArtifact> {
    let mut artifacts = Vec::new();
    for project in fs::read_dir(sessions_root)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
    {
        if !project.path().is_dir() {
            continue;
        }
        for session in fs::read_dir(project.path())
            .ok()
            .into_iter()
            .flat_map(|entries| entries.flatten())
        {
            if session.path().is_dir() {
                artifacts.extend(dsh_session_artifact(&session.path()));
            }
        }
    }
    artifacts.sort_by(|left, right| {
        right
            .modified_secs
            .cmp(&left.modified_secs)
            .then_with(|| left.id.cmp(&right.id))
    });
    artifacts
}

const MAX_QWEN_HEADER_BYTES: u64 = 1024 * 1024;

#[derive(Debug)]
pub(super) struct QwenSessionArtifact {
    pub path: PathBuf,
    pub id: String,
    pub cwd: String,
    pub modified_secs: u64,
}

pub(super) fn qwen_runtime_root(home: &Path) -> PathBuf {
    std::env::var_os("QWEN_RUNTIME_DIR")
        .filter(|value| !value.is_empty())
        .or_else(|| std::env::var_os("QWEN_HOME").filter(|value| !value.is_empty()))
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".qwen"))
}

pub(super) fn qwen_projects_root(home: &Path) -> PathBuf {
    qwen_runtime_root(home).join("projects")
}

pub(super) fn qwen_session_artifact(path: PathBuf) -> Option<QwenSessionArtifact> {
    if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") || !path.is_file() {
        return None;
    }
    let file = fs::File::open(&path).ok()?;
    let mut line = Vec::new();
    BufReader::new(file.take(MAX_QWEN_HEADER_BYTES + 1))
        .read_until(b'\n', &mut line)
        .ok()?;
    if line.len() as u64 > MAX_QWEN_HEADER_BYTES {
        return None;
    }
    let header = serde_json::from_slice::<Value>(&line).ok()?;
    let id = header.get("sessionId")?.as_str()?.to_string();
    let cwd = header.get("cwd")?.as_str()?.to_string();
    let modified_secs = fs::metadata(&path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    Some(QwenSessionArtifact {
        path,
        id,
        cwd,
        modified_secs,
    })
}

pub(super) fn qwen_session_artifacts(chats_root: &Path) -> Vec<QwenSessionArtifact> {
    let mut artifacts = fs::read_dir(chats_root)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .filter_map(|entry| qwen_session_artifact(entry.path()))
        .collect::<Vec<_>>();
    artifacts.sort_by(|left, right| {
        right
            .modified_secs
            .cmp(&left.modified_secs)
            .then_with(|| left.id.cmp(&right.id))
    });
    artifacts
}

fn scan_qwen_projects(home: &Path) -> Vec<RecentProjectDirectory> {
    scan_qwen_projects_at_root(&qwen_projects_root(home))
}

fn scan_qwen_projects_at_root(projects_root: &Path) -> Vec<RecentProjectDirectory> {
    let mut projects = HashMap::new();
    for project in fs::read_dir(projects_root)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
    {
        for artifact in qwen_session_artifacts(&project.path().join("chats")) {
            add_project_path(
                &mut projects,
                PathBuf::from(artifact.cwd),
                artifact.modified_secs,
            );
        }
    }
    project_map_to_sorted_list(projects)
}

pub(super) fn scan_recent_project_directories() -> Vec<RecentProjectDirectory> {
    let Some(home) = user_home_dir() else {
        return Vec::new();
    };

    let mut projects: HashMap<String, u64> = HashMap::new();

    let claude_projects =
        scan_claude_code_project_history(&home.join(".claude").join("history.jsonl"));
    //log_recent_project_group("Claude Code", &claude_projects);
    merge_project_directories(&mut projects, claude_projects);

    let codex_projects = scan_codex_projects_from_db(&home);
    //log_recent_project_group("Codex", &codex_projects);
    merge_project_directories(&mut projects, codex_projects);

    let zcode_projects = scan_zcode_projects_from_db(&home);
    //log_recent_project_group("ZCode", &zcode_projects);
    merge_project_directories(&mut projects, zcode_projects);

    let kimi_projects =
        scan_kimi_code_session_index(&home.join(".kimi-code").join("session_index.jsonl"));
    //log_recent_project_group("Kimi Code", &kimi_projects);
    merge_project_directories(&mut projects, kimi_projects);

    let qwen_projects = scan_qwen_projects(&home);
    merge_project_directories(&mut projects, qwen_projects);

    let pi_agent_dir = home.join(".pi").join("agent");
    let pi_sessions_root = pi_agent_dir.join("sessions");
    let pi_projects = scan_pi_session_projects(&pi_sessions_root);
    merge_project_directories(&mut projects, pi_projects);
    // Trust entries are not a history store, but keep them as a fallback for
    // trusted projects whose sessions were deleted.
    let pi_trusted = scan_pi_trusted_projects(&pi_agent_dir.join("trust.json"), &pi_sessions_root);
    merge_project_directories(&mut projects, pi_trusted);

    let opencode_projects = scan_opencode_projects_from_db(&home);
    merge_project_directories(&mut projects, opencode_projects);

    for artifact in dsh_session_artifacts(&dsh_sessions_root(&home)) {
        add_project_path(
            &mut projects,
            PathBuf::from(artifact.cwd),
            artifact.modified_secs,
        );
    }

    let mut directories = project_map_to_sorted_list(projects);
    directories.truncate(MAX_RECENT_PROJECTS);
    //log_recent_project_group("Combined", &directories);
    directories
}

fn scan_claude_code_project_history(history_path: &Path) -> Vec<RecentProjectDirectory> {
    let mut projects = HashMap::new();
    let Ok(contents) = fs::read_to_string(history_path) else {
        debug!(
            "Claude Code project history not available at {:?}",
            history_path
        );
        return Vec::new();
    };

    for (line_index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
            debug!(
                "Failed to parse Claude Code project history line {}",
                line_index + 1
            );
            continue;
        };
        let Some(project) = value.get("project").and_then(Value::as_str) else {
            continue;
        };
        let timestamp_secs = value
            .get("timestamp")
            .map(parse_timestamp_value)
            .unwrap_or(0);
        add_project_path(&mut projects, PathBuf::from(project), timestamp_secs);
    }

    project_map_to_sorted_list(projects)
}

/// Scan Codex project directories from the state_5.sqlite database.
/// Uses the `threads` table's `cwd` column and timestamps directly,
/// which is more reliable than scanning session files.
fn scan_codex_projects_from_db(home: &Path) -> Vec<RecentProjectDirectory> {
    let db_path = home.join(".codex").join("state_5.sqlite");
    let Ok(connection) = Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) else {
        debug!("Codex project database not available at {:?}", db_path);
        return Vec::new();
    };

    let Ok(mut statement) = connection.prepare(
        "SELECT cwd, \
            MAX(COALESCE(updated_at_ms, created_at_ms, updated_at * 1000, created_at * 1000)) \
         FROM threads \
         WHERE cwd IS NOT NULL AND cwd != '' AND archived = 0 \
         GROUP BY cwd \
         ORDER BY MAX(COALESCE(updated_at_ms, created_at_ms, updated_at * 1000, created_at * 1000)) DESC"
    ) else {
        warn!("Failed to prepare Codex project query from {:?}", db_path);
        return Vec::new();
    };

    let mut projects = Vec::new();
    let Ok(rows) = statement.query_map([], |row| {
        let cwd: String = row.get(0)?;
        let ts_ms: i64 = row.get(1)?;
        Ok((cwd, ts_ms))
    }) else {
        return projects;
    };

    for row in rows.flatten() {
        let (raw_path, ts_ms) = row;
        // Normalize the path: strip Windows extended-length prefix \\?\
        let normalized_path = strip_windows_extended_path_prefix(&raw_path);
        let path = PathBuf::from(&normalized_path);
        if !path.is_dir() {
            continue;
        }
        // ts_ms is in milliseconds (already the MAX of ms/s*1000 columns)
        let accessed_at_unix_secs = normalize_timestamp(ts_ms.max(0) as u64);
        projects.push(RecentProjectDirectory {
            path: display_path_string(&path),
            accessed_at_unix_secs,
        });
    }
    projects
}

/// Scan ZCode CLI project directories from `~/.zcode/cli/db/db.sqlite`.
///
/// Reads the `session` table's `directory` column (the project root) and
/// `time_updated` (Unix milliseconds), grouped by directory and filtered to
/// non-archived sessions. Only project directories are extracted — no AI
/// conversation content is read.
fn scan_zcode_projects_from_db(home: &Path) -> Vec<RecentProjectDirectory> {
    let db_path = home.join(".zcode").join("cli").join("db").join("db.sqlite");
    let Ok(connection) = Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) else {
        debug!("ZCode project database not available at {:?}", db_path);
        return Vec::new();
    };

    let Ok(mut statement) = connection.prepare(
        "SELECT directory, MAX(time_updated) \
         FROM session \
         WHERE directory IS NOT NULL AND directory != '' AND time_archived IS NULL \
         GROUP BY directory \
         ORDER BY MAX(time_updated) DESC",
    ) else {
        warn!("Failed to prepare ZCode project query from {:?}", db_path);
        return Vec::new();
    };

    let mut projects = Vec::new();
    let Ok(rows) = statement.query_map([], |row| {
        let cwd: String = row.get(0)?;
        let ts_ms: i64 = row.get(1)?;
        Ok((cwd, ts_ms))
    }) else {
        return projects;
    };

    for row in rows.flatten() {
        let (raw_path, ts_ms) = row;
        let normalized_path = strip_windows_extended_path_prefix(&raw_path);
        let path = PathBuf::from(&normalized_path);
        if !path.is_dir() {
            continue;
        }
        // time_updated is Unix milliseconds.
        let accessed_at_unix_secs = normalize_timestamp(ts_ms.max(0) as u64);
        projects.push(RecentProjectDirectory {
            path: display_path_string(&path),
            accessed_at_unix_secs,
        });
    }
    projects
}

/// OpenCode stores projects and conversations in its XDG data SQLite database.
/// Current versions keep projects in the `project` table, whose `worktree`
/// is the project root and `time_active` (Unix milliseconds) tracks recency.
/// The v1 `session` table remains as a migration leftover, so aggregate its
/// per-directory timestamps as a fallback for pre-v2 databases.
fn scan_opencode_projects_from_db(home: &Path) -> Vec<RecentProjectDirectory> {
    let mut projects = HashMap::new();
    for db_path in opencode_database_paths(home) {
        let Ok(connection) = Connection::open_with_flags(
            &db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        ) else {
            debug!("OpenCode project database not available at {:?}", db_path);
            continue;
        };
        add_opencode_db_projects(&connection, &mut projects);
    }
    project_map_to_sorted_list(projects)
}

/// Merge one OpenCode database's project rows into `projects`: the v2
/// `project` table first (authoritative, `time_active` tracks recency), then
/// the v1 `session` table's per-directory aggregation as a fallback for
/// pre-v2 databases. Missing tables simply contribute nothing.
fn add_opencode_db_projects(connection: &Connection, projects: &mut HashMap<String, u64>) {
    if let Ok(mut statement) = connection.prepare(
        "SELECT worktree, time_active \
         FROM project \
         WHERE worktree IS NOT NULL AND worktree != '' AND time_active > 0 \
         ORDER BY time_active DESC",
    ) {
        if let Ok(rows) = statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        }) {
            for row in rows.flatten() {
                add_project_path(
                    projects,
                    PathBuf::from(strip_windows_extended_path_prefix(&row.0)),
                    normalize_timestamp(row.1.max(0) as u64),
                );
            }
        }
    }
    if let Ok(mut statement) = connection.prepare(
        "SELECT directory, MAX(time_updated) \
         FROM session \
         WHERE directory IS NOT NULL AND directory != '' \
           AND parent_id IS NULL AND time_archived IS NULL \
         GROUP BY directory \
         ORDER BY MAX(time_updated) DESC",
    ) {
        if let Ok(rows) = statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        }) {
            for row in rows.flatten() {
                add_project_path(
                    projects,
                    PathBuf::from(strip_windows_extended_path_prefix(&row.0)),
                    normalize_timestamp(row.1.max(0) as u64),
                );
            }
        }
    }
}

pub(super) fn opencode_data_dir(home: &Path) -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local").join("share"))
        .join("opencode")
}

/// Return OpenCode's configured database plus any channel-specific databases
/// (`opencode-beta.db`, development channel names, and so on).
pub(super) fn opencode_database_paths(home: &Path) -> Vec<PathBuf> {
    let data_dir = opencode_data_dir(home);
    let mut paths = Vec::new();
    if let Some(configured) = std::env::var_os("OPENCODE_DB").filter(|value| !value.is_empty()) {
        let configured = PathBuf::from(configured);
        if configured.as_os_str() != ":memory:" {
            paths.push(if configured.is_absolute() {
                configured
            } else {
                data_dir.join(configured)
            });
        }
    }
    paths.push(data_dir.join("opencode.db"));
    if let Ok(entries) = fs::read_dir(&data_dir) {
        paths.extend(entries.flatten().filter_map(|entry| {
            let path = entry.path();
            let name = path.file_name()?.to_str()?;
            (path.is_file()
                && name.starts_with("opencode-")
                && path.extension().and_then(|ext| ext.to_str()) == Some("db"))
            .then_some(path)
        }));
    }
    paths.sort();
    paths.dedup();
    paths.retain(|path| path.is_file());
    paths
}

/// Scan Kimi Code project directories from `~/.kimi-code/session_index.jsonl`.
///
/// Each line is `{"sessionId":..., "sessionDir":..., "workDir":...}`; the
/// project directory is the `workDir` field. The index has no timestamp, so
/// the access time is taken from the session's wire.jsonl mtime (falling back
/// to the session directory mtime, then 0).
fn scan_kimi_code_session_index(index_path: &Path) -> Vec<RecentProjectDirectory> {
    let mut projects = HashMap::new();
    let Ok(contents) = fs::read_to_string(index_path) else {
        debug!("Kimi Code session index not available at {:?}", index_path);
        return Vec::new();
    };

    for (line_index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
            debug!(
                "Failed to parse Kimi Code session index line {}",
                line_index + 1
            );
            continue;
        };
        let Some(work_dir) = value.get("workDir").and_then(Value::as_str) else {
            continue;
        };
        let timestamp_secs = value
            .get("sessionDir")
            .and_then(Value::as_str)
            .map(kimi_session_modified_secs)
            .unwrap_or(0);
        add_project_path(&mut projects, PathBuf::from(work_dir), timestamp_secs);
    }

    project_map_to_sorted_list(projects)
}

/// Best-effort modification time for a Kimi Code session: the wire.jsonl
/// transcript if present, otherwise the session directory itself.
fn kimi_session_modified_secs(session_dir: &str) -> u64 {
    let session_dir = Path::new(session_dir);
    for path in [
        session_dir.join("agents").join("main").join("wire.jsonl"),
        session_dir.to_path_buf(),
    ] {
        if let Some(secs) = fs::metadata(&path)
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
        {
            return secs;
        }
    }
    0
}

/// Scan Pi Agent projects from `~/.pi/agent/sessions`.
///
/// Pi stores one session directory per project, named by escaping the cwd
/// (see `pi_session_directory_name`). The encoding is lossy (`-` is
/// ambiguous), so the project path is read from the `cwd` field of a
/// transcript's header line instead. The newest transcript mtime is the
/// project's access time. Empty session directories are skipped.
fn scan_pi_session_projects(sessions_root: &Path) -> Vec<RecentProjectDirectory> {
    let mut projects = HashMap::new();
    let Ok(entries) = fs::read_dir(sessions_root) else {
        debug!("Pi Agent sessions not available at {:?}", sessions_root);
        return Vec::new();
    };

    for entry in entries.flatten() {
        let session_dir = entry.path();
        if !session_dir.is_dir() {
            continue;
        }
        let Some((cwd, timestamp_secs)) = pi_session_dir_project(&session_dir) else {
            continue;
        };
        add_project_path(&mut projects, PathBuf::from(cwd), timestamp_secs);
    }

    project_map_to_sorted_list(projects)
}

/// Extract the project cwd and latest activity time from one Pi session
/// directory: the `cwd` field of the newest transcript's header line, and
/// that file's mtime.
fn pi_session_dir_project(session_dir: &Path) -> Option<(String, u64)> {
    let mut newest: Option<(u64, PathBuf)> = None;
    for entry in fs::read_dir(session_dir).ok()?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
            continue;
        }
        let mtime = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        if newest.as_ref().is_none_or(|(latest, _)| mtime >= *latest) {
            newest = Some((mtime, path));
        }
    }
    let (mtime, transcript) = newest?;
    let cwd = pi_session_header_cwd(&transcript)?;
    Some((cwd, mtime))
}

/// Read the `cwd` field from a Pi transcript's header line
/// (`{"type":"session",...,"cwd":"..."}`).
fn pi_session_header_cwd(transcript: &Path) -> Option<String> {
    let file = fs::File::open(transcript).ok()?;
    let mut line = String::new();
    std::io::BufRead::read_line(&mut std::io::BufReader::new(file), &mut line).ok()?;
    let value = serde_json::from_str::<Value>(line.trim()).ok()?;
    if value.get("type").and_then(Value::as_str) != Some("session") {
        return None;
    }
    let cwd = value.get("cwd").and_then(Value::as_str)?;
    Some(strip_windows_extended_path_prefix(cwd))
}

/// Scan Pi Agent projects from `~/.pi/agent/trust.json`.
///
/// Pi's trust store is an object keyed by canonical project path. Its boolean
/// value is a trust decision, not a recency marker, so both trusted and
/// explicitly-untrusted projects count as previously visited. Pi stores JSONL
/// sessions below a cwd-encoded directory; the newest transcript mtime is the
/// project's access time.
fn scan_pi_trusted_projects(
    trust_path: &Path,
    sessions_root: &Path,
) -> Vec<RecentProjectDirectory> {
    let mut projects = HashMap::new();
    let Ok(contents) = fs::read_to_string(trust_path) else {
        debug!("Pi Agent trust store not available at {:?}", trust_path);
        return Vec::new();
    };
    let Ok(Value::Object(entries)) = serde_json::from_str::<Value>(&contents) else {
        warn!("Failed to parse Pi Agent trust store at {:?}", trust_path);
        return Vec::new();
    };

    for (project, decision) in entries {
        if !decision.is_boolean() {
            continue;
        }
        let timestamp_secs = pi_project_latest_session_modified_secs(sessions_root, &project);
        add_project_path(&mut projects, PathBuf::from(project), timestamp_secs);
    }

    project_map_to_sorted_list(projects)
}

/// Match Pi's `getDefaultSessionDirPath` / `jsonlSessionDirectoryName` cwd
/// escaping: remove one leading slash, replace slash/backslash/colon with `-`,
/// then wrap the result in `--`.
pub(super) fn pi_session_directory_name(cwd: &str) -> String {
    let cwd = strip_windows_extended_path_prefix(cwd);
    let cwd = cwd
        .strip_prefix('/')
        .or_else(|| cwd.strip_prefix('\\'))
        .unwrap_or(&cwd);
    let safe = cwd
        .chars()
        .map(|ch| {
            if matches!(ch, '/' | '\\' | ':') {
                '-'
            } else {
                ch
            }
        })
        .collect::<String>();
    format!("--{safe}--")
}

fn pi_project_latest_session_modified_secs(sessions_root: &Path, cwd: &str) -> u64 {
    let session_dir = sessions_root.join(pi_session_directory_name(cwd));
    fs::read_dir(session_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
                return None;
            }
            entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs())
        })
        .max()
        .unwrap_or(0)
}

fn add_project_path(projects: &mut HashMap<String, u64>, path: PathBuf, accessed_at: u64) {
    let Ok(path) = normalize_existing_dir(path) else {
        return;
    };
    let path = display_path_string(&path);
    projects
        .entry(path)
        .and_modify(|existing| *existing = (*existing).max(accessed_at))
        .or_insert(accessed_at);
}

fn normalize_existing_dir(path: PathBuf) -> std::io::Result<PathBuf> {
    if path.is_dir() {
        fs::canonicalize(path)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "project path is not an existing directory",
        ))
    }
}

fn display_path_string(path: &Path) -> String {
    strip_windows_extended_path_prefix(path.to_string_lossy().as_ref())
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

fn is_project_metadata_file(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
        return true;
    };
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "json" | "jsonl" | "txt" | "log"
    )
}

fn user_home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

/// Parse a timestamp value from JSON into Unix epoch seconds.
/// Supports:
/// - Numeric seconds (< 10_000_000_000)
/// - Numeric milliseconds (>= 10_000_000_000, divided by 1000)
/// - ISO 8601 strings: "2024-06-03T10:30:00Z", "2024-06-03T10:30:00+00:00",
///   "2024-06-03 10:30:00", "2024-06-03" (treated as start of day UTC)
fn parse_timestamp_value(value: &Value) -> u64 {
    match value {
        Value::Number(n) => n.as_u64().map(normalize_timestamp).unwrap_or(0),
        Value::String(s) => parse_iso8601_timestamp(s).unwrap_or(0),
        _ => 0,
    }
}

/// Normalize a numeric timestamp to Unix epoch seconds.
fn normalize_timestamp(timestamp: u64) -> u64 {
    if timestamp > 10_000_000_000 {
        timestamp / 1_000 // milliseconds → seconds
    } else {
        timestamp
    }
}

/// Parse common ISO 8601 timestamp strings into Unix epoch seconds.
fn parse_iso8601_timestamp(s: &str) -> Option<u64> {
    let s = s.trim();
    // Strip trailing 'Z' for UTC
    let s = s.strip_suffix('Z').unwrap_or(s);
    // Strip timezone offset like +05:30 or -05:00 (6 chars at end: sign + HH:MM)
    let s = if s.len() > 6 {
        let (body, tz) = s.split_at(s.len() - 6);
        let is_offset = (tz.starts_with('+') || tz.starts_with('-'))
            && tz.as_bytes().get(3) == Some(&b':')
            && tz[1..3].bytes().all(|b| b.is_ascii_digit())
            && tz[4..6].bytes().all(|b| b.is_ascii_digit());
        if is_offset {
            body
        } else {
            s
        }
    } else {
        s
    };

    // Accept both 'T' and space as date-time separator
    let (date_part, time_part) = s.split_once(|c| c == 'T' || c == ' ').unwrap_or((s, ""));

    // Parse date components: YYYY-MM-DD
    let mut parts = date_part.splitn(3, '-');
    let year: u32 = parts.next()?.parse().ok()?;
    let month: u32 = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;

    // Parse time components: HH:MM:SS or HH:MM
    let (hour, minute, second) = if time_part.is_empty() {
        (0, 0, 0)
    } else {
        let mut time_parts = time_part.splitn(3, ':');
        let h: u32 = time_parts.next()?.parse().ok()?;
        let m: u32 = time_parts.next()?.parse().ok()?;
        let s: u32 = time_parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
        (h, m, s)
    };

    // Use a simple algorithm for days since epoch to avoid external dependencies.
    let days = days_from_civil(year as i64, month, day)?;
    let secs = days as u64 * 86400 + hour as u64 * 3600 + minute as u64 * 60 + second as u64;
    Some(secs)
}

/// Convert a civil date to days since Unix epoch (1970-01-01).
/// Uses the algorithm from Howard Hinnant's chrono-compatible的低层次日期算法.
fn days_from_civil(year: i64, month: u32, day: u32) -> Option<i64> {
    if month < 1 || month > 12 || day < 1 || day > 31 {
        return None;
    }
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 };
    let era = era / 400;
    let yoe = year - era * 400;
    let doy =
        (153 * (if month > 2 { month - 3 } else { month + 9 } as i64) + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let epoch_era = 400i64 * 365 + 97;
    Some(era * epoch_era + doe - 719468)
}

fn merge_project_directories(
    projects: &mut HashMap<String, u64>,
    directories: Vec<RecentProjectDirectory>,
) {
    for directory in directories {
        if !Path::new(&directory.path).is_dir() {
            debug!(
                path = %directory.path,
                "skipping missing recent project directory"
            );
            continue;
        }
        projects
            .entry(directory.path)
            .and_modify(|existing| *existing = (*existing).max(directory.accessed_at_unix_secs))
            .or_insert(directory.accessed_at_unix_secs);
    }
}

fn project_map_to_sorted_list(projects: HashMap<String, u64>) -> Vec<RecentProjectDirectory> {
    let mut directories = projects
        .into_iter()
        .map(|(path, accessed_at_unix_secs)| RecentProjectDirectory {
            path,
            accessed_at_unix_secs,
        })
        .collect::<Vec<_>>();
    directories.sort_by(|left, right| {
        right
            .accessed_at_unix_secs
            .cmp(&left.accessed_at_unix_secs)
            .then_with(|| left.path.cmp(&right.path))
    });
    directories
}

fn log_recent_project_group(source: &str, directories: &[RecentProjectDirectory]) {
    info!(
        "{} recent project scan found {} project(s)",
        source,
        directories.len()
    );
    for directory in directories {
        info!(
            "{} recent project: timestamp={} path={}",
            source, directory.accessed_at_unix_secs, directory.path
        );
    }
}

pub(super) fn spawn_recent_project_scan(
    store: std::sync::Arc<std::sync::RwLock<Vec<RecentProjectDirectory>>>,
    state_tx: watch::Sender<RecentProjectScanState>,
) {
    if let Err(error) = std::thread::Builder::new()
        .name("recent-project-scan".to_string())
        .spawn(move || {
            let directories = scan_recent_project_directories();
            match store.write() {
                Ok(mut guard) => *guard = directories.clone(),
                Err(error) => warn!("Failed to store recent project directories: {}", error),
            }
            let _ = state_tx.send(RecentProjectScanState::Ready(directories));
        })
    {
        warn!("Failed to spawn recent project scan thread: {}", error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qwen_projects_use_active_session_cwd() {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("project");
        let other = temp.path().join("other");
        fs::create_dir(&project).unwrap();
        fs::create_dir(&other).unwrap();
        let projects_root = temp.path().join("runtime").join("projects");
        let chats = projects_root.join("project-hash").join("chats");
        fs::create_dir_all(&chats).unwrap();
        let session_id = "550e8400-e29b-41d4-a716-446655440000";
        let record = serde_json::json!({
            "sessionId": session_id,
            "cwd": project.to_string_lossy(),
            "type": "user",
            "message": {"role": "user", "parts": [{"text": "hello qwen"}]}
        });
        fs::write(
            chats.join(format!("{session_id}.jsonl")),
            format!("{record}\n"),
        )
        .unwrap();
        let archive = chats.join("archive");
        fs::create_dir(&archive).unwrap();
        let archived = serde_json::json!({
            "sessionId": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
            "cwd": other.to_string_lossy()
        });
        fs::write(
            archive.join("6ba7b810-9dad-11d1-80b4-00c04fd430c8.jsonl"),
            format!("{archived}\n"),
        )
        .unwrap();
        fs::write(chats.join("invalid.jsonl"), "not json\n").unwrap();

        let projects = scan_qwen_projects_at_root(&projects_root);

        assert_eq!(projects.len(), 1);
        assert_eq!(
            projects[0].path,
            display_path_string(&fs::canonicalize(project).unwrap())
        );
        assert!(projects[0].accessed_at_unix_secs > 0);
    }

    #[test]
    fn dsh_artifacts_select_latest_generation_and_decode_concatenated_zstd_frames() {
        use std::io::Write;

        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("project");
        fs::create_dir(&project).unwrap();
        let session_dir = temp
            .path()
            .join("sessions")
            .join("--project--")
            .join("dsh-id");
        fs::create_dir_all(&session_dir).unwrap();
        fs::write(
            session_dir.join("session.v3.jsonl"),
            format!(
                "{{\"type\":\"session\",\"id\":\"old\",\"cwd\":{:?}}}\n",
                project.to_string_lossy()
            ),
        )
        .unwrap();
        let header = format!(
            "{{\"type\":\"session\",\"id\":\"dsh-id\",\"cwd\":{:?}}}\n",
            project.to_string_lossy()
        );
        let event = format!(
            "{{\"type\":\"session/title\",\"data\":{{\"title\":\"{}\"}}}}\n",
            "x".repeat(MAX_DSH_HEADER_BYTES as usize)
        );
        let compress = |text: &str| {
            let mut encoder = zstd::stream::write::Encoder::new(Vec::new(), 0).unwrap();
            encoder.write_all(text.as_bytes()).unwrap();
            encoder.finish().unwrap()
        };
        let expected = format!("{header}{event}");
        let mut compressed = compress(&header);
        compressed.extend(compress(&event));
        fs::write(session_dir.join("session.v4.jsonl.zstd"), compressed).unwrap();
        fs::write(session_dir.join("session.v04.jsonl"), "invalid").unwrap();

        let artifacts = dsh_session_artifacts(&temp.path().join("sessions"));

        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].id, "dsh-id");
        assert_eq!(artifacts[0].cwd, project.to_string_lossy());
        assert!(artifacts[0].path.ends_with("session.v4.jsonl.zstd"));
        assert_eq!(
            read_dsh_log(&artifacts[0].path).unwrap(),
            expected.as_bytes()
        );
    }

    #[test]
    fn dsh_log_version_accepts_only_canonical_generation_names() {
        assert_eq!(dsh_log_version(Path::new("session.jsonl")), Some(0));
        assert_eq!(dsh_log_version(Path::new("session.jsonl.zstd")), Some(0));
        assert_eq!(dsh_log_version(Path::new("session.v4.jsonl.zstd")), Some(4));
        assert_eq!(dsh_log_version(Path::new("session.v04.jsonl")), None);
        assert_eq!(dsh_log_version(Path::new("session.v4.JSONL")), None);
        assert_eq!(dsh_log_version(Path::new("other.v4.jsonl")), None);
    }

    /// v2 databases keep projects in the `project` table (`time_active`
    /// tracks recency); the v1 `session` table is a migration leftover. Both
    /// sources merge, keeping the newest timestamp per directory.
    #[test]
    fn opencode_projects_merge_v2_project_table_and_v1_sessions() {
        let temp = tempfile::tempdir().unwrap();
        let project_v2 = temp.path().join("project-v2");
        let project_v1 = temp.path().join("project-v1");
        fs::create_dir_all(&project_v2).unwrap();
        fs::create_dir_all(&project_v1).unwrap();

        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute(
                "CREATE TABLE project (worktree TEXT, time_active INTEGER)",
                [],
            )
            .unwrap();
        connection
            .execute(
                "CREATE TABLE session (directory TEXT, time_updated INTEGER, \
                 parent_id TEXT, time_archived INTEGER)",
                [],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO project VALUES (?1, 2000)",
                [project_v2.to_string_lossy().as_ref()],
            )
            .unwrap();
        // Empty worktrees and inactive rows are ignored.
        connection
            .execute("INSERT INTO project VALUES ('', 3000)", [])
            .unwrap();
        connection
            .execute(
                "INSERT INTO project VALUES (?1, 0)",
                [project_v1.to_string_lossy().as_ref()],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO session VALUES (?1, 1000, NULL, NULL)",
                [project_v1.to_string_lossy().as_ref()],
            )
            .unwrap();
        // Archived v1 sessions do not count.
        connection
            .execute(
                "INSERT INTO session VALUES (?1, 5000, NULL, 1)",
                [project_v2.to_string_lossy().as_ref()],
            )
            .unwrap();

        let mut projects = HashMap::new();
        add_opencode_db_projects(&connection, &mut projects);

        assert_eq!(projects.len(), 2);
        let v2_path = display_path_string(&fs::canonicalize(&project_v2).unwrap());
        let v1_path = display_path_string(&fs::canonicalize(&project_v1).unwrap());
        assert!(projects[&v2_path] > 0);
        assert!(projects[&v1_path] > 0);
        // The archived v1 row must not override the v2 timestamp (2000 is
        // below normalize_timestamp's ms threshold and passes through).
        assert_eq!(projects[&v2_path], 2000);
    }

    /// A v1-only database has no `project` table; the scan must fall back to
    /// aggregating `session` directories.
    #[test]
    fn opencode_projects_fall_back_to_v1_sessions() {
        let temp = tempfile::tempdir().unwrap();
        let project_v1 = temp.path().join("project-v1");
        fs::create_dir_all(&project_v1).unwrap();

        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute(
                "CREATE TABLE session (directory TEXT, time_updated INTEGER, \
                 parent_id TEXT, time_archived INTEGER)",
                [],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO session VALUES (?1, 1000, NULL, NULL)",
                [project_v1.to_string_lossy().as_ref()],
            )
            .unwrap();

        let mut projects = HashMap::new();
        add_opencode_db_projects(&connection, &mut projects);

        assert_eq!(projects.len(), 1);
    }

    #[test]
    fn pi_session_directory_name_matches_pi_cwd_escaping() {
        assert_eq!(
            pi_session_directory_name("/home/dev/projects/vibe"),
            "--home-dev-projects-vibe--"
        );
        assert_eq!(
            pi_session_directory_name(r"C:\Users\dev\projects\vibe"),
            "--C--Users-dev-projects-vibe--"
        );
        assert_eq!(
            pi_session_directory_name(r"\\?\C:\Users\dev\projects\vibe"),
            "--C--Users-dev-projects-vibe--"
        );
    }

    #[test]
    fn pi_trust_projects_use_session_file_timestamp() {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("project");
        fs::create_dir_all(&project).unwrap();
        let sessions_root = temp.path().join("sessions");
        let session_dir = sessions_root.join(pi_session_directory_name(
            project.to_string_lossy().as_ref(),
        ));
        fs::create_dir_all(&session_dir).unwrap();
        fs::write(session_dir.join("session.jsonl"), "{}\n").unwrap();
        let trust_path = temp.path().join("trust.json");
        fs::write(
            &trust_path,
            serde_json::to_string(&HashMap::from([(
                project.to_string_lossy().into_owned(),
                false,
            )]))
            .unwrap(),
        )
        .unwrap();

        let projects = scan_pi_trusted_projects(&trust_path, &sessions_root);

        assert_eq!(projects.len(), 1);
        assert_eq!(
            projects[0].path,
            display_path_string(&fs::canonicalize(project).unwrap())
        );
        assert!(projects[0].accessed_at_unix_secs > 0);
    }

    #[test]
    fn pi_session_projects_use_transcript_header_cwd() {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("project");
        fs::create_dir_all(&project).unwrap();
        let sessions_root = temp.path().join("sessions");
        let session_dir = sessions_root.join(pi_session_directory_name(
            project.to_string_lossy().as_ref(),
        ));
        fs::create_dir_all(&session_dir).unwrap();
        let header = serde_json::json!({
            "type": "session",
            "version": 3,
            "cwd": project.to_string_lossy(),
        });
        fs::write(session_dir.join("session.jsonl"), format!("{header}\n")).unwrap();
        // Empty session directories (sessions deleted) are skipped.
        fs::create_dir_all(sessions_root.join("--empty--")).unwrap();

        let projects = scan_pi_session_projects(&sessions_root);

        assert_eq!(projects.len(), 1);
        assert_eq!(
            projects[0].path,
            display_path_string(&fs::canonicalize(project).unwrap())
        );
        assert!(projects[0].accessed_at_unix_secs > 0);
    }
}
