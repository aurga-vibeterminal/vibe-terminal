//! Restricted tools for an agent that resolves the repository's current Git
//! conflicts.
//!
//! These tools are intentionally not part of `vterm-agent`'s builtin registry.
//! The private registry builder is called only by the conflict-agent runner
//! below. In particular, the registry does not contain the generic shell or
//! unrestricted file tools.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vterm_agent::agent::{Agent, AgentConfig, LoopResult};
use vterm_agent::client::types::{ChatCompletionRequest, ChatCompletionResponse};
use vterm_agent::client::LlmClient;
use vterm_agent::error::AgentError;
use vterm_agent::memory::CompressionConfig;
use vterm_agent::tool::{Tool, ToolDefinition, ToolRegistry};

use crate::addon::{resolve_addon_config, resolve_llm_addon_id, run_chat_completion, AddonHost};
use crate::config::TerminalServerConfig;
use crate::session::SessionError;

use super::repo_tools::{
    decode_text, function_tool, json_string, parse_args, run_git, validate_relative_repo_path,
    RepoReadTool, RepoSearchTool,
};
use super::{git_check_repo, git_repo_root};

const MAX_CONFLICT_BLOB_BYTES: usize = 512 * 1024;
const MAX_RESOLVED_CONTENT_BYTES: usize = 1024 * 1024;
const DEFAULT_AGENT_OUTPUT_TOKENS: usize = 8192;
const MAX_AGENT_OUTPUT_TOKENS: usize = 32768;
const DEFAULT_AGENT_TIMEOUT_SECONDS: u64 = 120;

const CONFLICT_AGENT_SYSTEM_PROMPT: &str = "\
You resolve the current Git conflict batch using only the provided restricted tools.\n\
\n\
Rules:\n\
1. Start with git_conflict_list. For every conflict, read base/ours/theirs with git_conflict_read before resolving it.\n\
2. Use git_repo_read and git_repo_search only when surrounding code is needed to preserve behavior.\n\
3. Resolve each conflict according to intent, not by mechanically deleting markers or always choosing one side.\n\
4. The meaning of ours/theirs is provided by git_conflict_list and differs between merge and rebase.\n\
5. Pass the exact version returned by git_conflict_read to git_conflict_resolve. If it is stale, reread the conflict.\n\
6. Never claim success based on your own reasoning. Call git_conflict_status after all resolutions; success requires remaining_count to be zero.\n\
7. Do not attempt to continue, skip, abort, commit, merge, or rebase. The user controls those operations.\n\
8. Keep the final response short and report resolved and remaining files. Do not include hidden reasoning.";

#[derive(Debug, Clone)]
pub(crate) struct GitConflictAgentResult {
    pub resolved: bool,
    pub remaining_files: Vec<String>,
    pub summary: Option<String>,
    pub error: Option<String>,
}

type LlmCallback = Box<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync>;

struct ConflictLlmClient {
    callback: Arc<LlmCallback>,
}

#[async_trait]
impl LlmClient for ConflictLlmClient {
    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AgentError> {
        let messages_json = serde_json::to_string(&request.messages)
            .map_err(|error| AgentError::LlmClient(error.to_string()))?;
        let mut options = serde_json::Map::new();
        if !request.model.is_empty() {
            options.insert("model".to_string(), serde_json::json!(request.model));
        }
        if let Some(temperature) = request.temperature {
            options.insert("temperature".to_string(), serde_json::json!(temperature));
        }
        if let Some(max_tokens) = request.max_tokens {
            options.insert("max_tokens".to_string(), serde_json::json!(max_tokens));
        }
        if let Some(tools) = request.tools {
            options.insert("tools".to_string(), serde_json::json!(tools));
        }
        let options_json = serde_json::to_string(&options)
            .map_err(|error| AgentError::LlmClient(error.to_string()))?;
        let callback = self.callback.clone();
        let raw_body = tokio::task::spawn_blocking(move || callback(&messages_json, &options_json))
            .await
            .map_err(|error| AgentError::LlmClient(error.to_string()))?
            .map_err(AgentError::LlmClient)?;
        serde_json::from_str(&raw_body).map_err(|error| {
            AgentError::LlmClient(format!("failed to parse conflict LLM response: {error}"))
        })
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ConflictStage {
    mode: String,
    oid: String,
}

#[derive(Debug, Clone, Serialize)]
struct ConflictEntry {
    conflict_id: String,
    path: String,
    status: String,
    version: String,
    stages: BTreeMap<u8, ConflictStage>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum ConflictOperation {
    Merge,
    Rebase,
    Unknown,
}

impl ConflictOperation {
    fn labels(self) -> (&'static str, &'static str) {
        match self {
            Self::Merge => ("current branch (ours)", "branch being merged (theirs)"),
            Self::Rebase => (
                "branch being rebased onto (ours)",
                "commit currently being replayed (theirs)",
            ),
            Self::Unknown => ("index stage 2 (ours)", "index stage 3 (theirs)"),
        }
    }
}

#[derive(Debug)]
struct ConflictToolState {
    repo_root: PathBuf,
    operation: ConflictOperation,
    mutation_lock: Mutex<()>,
}

impl ConflictToolState {
    fn new(working_path: &str) -> Result<Self, SessionError> {
        git_check_repo(working_path)?;
        let root = PathBuf::from(git_repo_root(working_path)?);
        let repo_root = root.canonicalize().map_err(|error| {
            SessionError::FileSystemError(format!(
                "failed to canonicalize repository root `{}`: {error}",
                root.display()
            ))
        })?;
        let operation = detect_operation(&repo_root);
        Ok(Self {
            repo_root,
            operation,
            mutation_lock: Mutex::new(()),
        })
    }

    fn current_conflicts(&self) -> Result<Vec<ConflictEntry>, String> {
        read_unmerged_entries(&self.repo_root)
    }

    fn conflict_by_id(&self, conflict_id: &str) -> Result<ConflictEntry, String> {
        self.current_conflicts()?
            .into_iter()
            .find(|entry| entry.conflict_id == conflict_id)
            .ok_or_else(|| {
                format!(
                    "unknown or already resolved conflict_id `{conflict_id}`; call git_conflict_list again"
                )
            })
    }
}

fn git_path_exists(repo_root: &Path, name: &str) -> bool {
    let Ok(output) = run_git(repo_root, &["rev-parse", "--git-path", name]) else {
        return false;
    };
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        return false;
    }
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path.exists()
    } else {
        repo_root.join(path).exists()
    }
}

fn detect_operation(repo_root: &Path) -> ConflictOperation {
    if git_path_exists(repo_root, "rebase-merge") || git_path_exists(repo_root, "rebase-apply") {
        ConflictOperation::Rebase
    } else if git_path_exists(repo_root, "MERGE_HEAD") {
        ConflictOperation::Merge
    } else {
        ConflictOperation::Unknown
    }
}

fn conflict_id(path: &str) -> String {
    let digest = Sha256::digest(path.as_bytes());
    format!("conflict-{}", hex_prefix(&digest, 8))
}

fn conflict_version(path: &str, stages: &BTreeMap<u8, ConflictStage>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    for (stage, value) in stages {
        hasher.update([*stage]);
        hasher.update(value.mode.as_bytes());
        hasher.update([0]);
        hasher.update(value.oid.as_bytes());
        hasher.update([0]);
    }
    let digest = hasher.finalize();
    hex_prefix(&digest, 16)
}

fn hex_prefix(bytes: &[u8], count: usize) -> String {
    let mut output = String::with_capacity(count * 2);
    for byte in bytes.iter().take(count) {
        use std::fmt::Write;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn conflict_status(stages: &BTreeMap<u8, ConflictStage>) -> &'static str {
    match (
        stages.contains_key(&1),
        stages.contains_key(&2),
        stages.contains_key(&3),
    ) {
        (true, true, true) => "both_modified",
        (false, true, true) => "both_added",
        (true, false, false) => "both_deleted",
        (true, true, false) => "deleted_by_theirs",
        (true, false, true) => "deleted_by_ours",
        _ => "unmerged",
    }
}

fn read_unmerged_entries(repo_root: &Path) -> Result<Vec<ConflictEntry>, String> {
    let output = run_git(repo_root, &["ls-files", "-u", "-z"])?;
    let stdout = String::from_utf8(output.stdout)
        .map_err(|_| "conflict paths or index output are not valid UTF-8".to_string())?;
    let mut grouped: BTreeMap<String, BTreeMap<u8, ConflictStage>> = BTreeMap::new();

    for record in stdout.split('\0').filter(|record| !record.is_empty()) {
        let (metadata, path) = record
            .split_once('\t')
            .ok_or_else(|| format!("invalid git ls-files record: {record:?}"))?;
        let mut fields = metadata.split_whitespace();
        let mode = fields.next().unwrap_or_default();
        let oid = fields.next().unwrap_or_default();
        let stage = fields
            .next()
            .and_then(|value| value.parse::<u8>().ok())
            .unwrap_or_default();
        if mode.is_empty() || oid.is_empty() || !(1..=3).contains(&stage) {
            return Err(format!("invalid unmerged index metadata: {metadata:?}"));
        }
        grouped.entry(path.to_string()).or_default().insert(
            stage,
            ConflictStage {
                mode: mode.to_string(),
                oid: oid.to_string(),
            },
        );
    }

    Ok(grouped
        .into_iter()
        .map(|(path, stages)| ConflictEntry {
            conflict_id: conflict_id(&path),
            status: conflict_status(&stages).to_string(),
            version: conflict_version(&path, &stages),
            path,
            stages,
        })
        .collect())
}

fn safe_conflict_target(repo_root: &Path, path: &str) -> Result<PathBuf, String> {
    let relative = validate_relative_repo_path(path)?;
    let joined = repo_root.join(relative);
    let parent = joined
        .parent()
        .ok_or_else(|| format!("conflict path has no parent: `{path}`"))?;
    let canonical_parent = parent
        .canonicalize()
        .map_err(|error| format!("failed to canonicalize parent of `{path}`: {error}"))?;
    if !canonical_parent.starts_with(repo_root) {
        return Err(format!("parent directory escapes the repository: `{path}`"));
    }

    if let Ok(metadata) = std::fs::symlink_metadata(&joined) {
        if metadata.file_type().is_symlink() {
            return Err(format!("refusing to overwrite symbolic link `{path}`"));
        }
        if !metadata.is_file() {
            return Err(format!("conflict target is not a regular file: `{path}`"));
        }
        let canonical = joined
            .canonicalize()
            .map_err(|error| format!("failed to canonicalize `{path}`: {error}"))?;
        if !canonical.starts_with(repo_root) {
            return Err(format!("path escapes the repository: `{path}`"));
        }
    }
    Ok(joined)
}

fn read_blob(repo_root: &Path, stage: &ConflictStage) -> Result<Vec<u8>, String> {
    if stage.mode == "120000" {
        return Err("symbolic-link conflict stages are not supported".to_string());
    }
    if stage.mode == "160000" {
        return Err("submodule conflict stages are not supported".to_string());
    }
    let output = run_git(repo_root, &["cat-file", "blob", &stage.oid])?;
    if output.stdout.len() > MAX_CONFLICT_BLOB_BYTES {
        return Err(format!(
            "conflict blob is too large: {} bytes (limit {})",
            output.stdout.len(),
            MAX_CONFLICT_BLOB_BYTES
        ));
    }
    Ok(output.stdout)
}

fn contains_conflict_markers(content: &str) -> bool {
    content.lines().any(|line| {
        let line = line.trim_start();
        line.starts_with("<<<<<<< ")
            || line == "======="
            || line.starts_with(">>>>>>> ")
            || line.starts_with("||||||| ")
    })
}

fn stage_path(repo_root: &Path, path: &str) -> Result<(), String> {
    const ATTEMPTS: usize = 8;
    for attempt in 1..=ATTEMPTS {
        match run_git(repo_root, &["add", "-A", "--", path]) {
            Ok(_) => return Ok(()),
            Err(error) if error.contains(".lock") && attempt < ATTEMPTS => {
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!("stage retry loop always returns")
}

/// Safe content-only bridge for the existing client conflict editor.
///
/// Unlike the old implementation, `path` must exactly match a path currently
/// present in the unmerged index and must pass the same repository-boundary
/// checks as the conflict agent.
pub(crate) fn git_resolve_conflict_content(
    working_path: &str,
    path: &str,
    content: &str,
) -> Result<(), SessionError> {
    let state = ConflictToolState::new(working_path)?;
    let entry = state
        .current_conflicts()
        .map_err(SessionError::FileSystemError)?
        .into_iter()
        .find(|entry| entry.path == path)
        .ok_or_else(|| {
            SessionError::FileSystemError(format!(
                "`{path}` is not a current unmerged conflict path"
            ))
        })?;
    if content.len() > MAX_RESOLVED_CONTENT_BYTES {
        return Err(SessionError::FileSystemError(format!(
            "resolved content is too large: {} bytes (limit {})",
            content.len(),
            MAX_RESOLVED_CONTENT_BYTES
        )));
    }
    if contains_conflict_markers(content) {
        return Err(SessionError::FileSystemError(
            "resolved content still contains Git conflict markers".to_string(),
        ));
    }
    let target = safe_conflict_target(&state.repo_root, &entry.path)
        .map_err(SessionError::FileSystemError)?;
    std::fs::write(&target, content.as_bytes()).map_err(|error| {
        SessionError::FileSystemError(format!("failed to write `{}`: {error}", entry.path))
    })?;
    stage_path(&state.repo_root, &entry.path).map_err(SessionError::FileSystemError)
}

#[derive(Clone)]
struct ConflictListTool {
    state: Arc<ConflictToolState>,
}

#[async_trait]
impl Tool for ConflictListTool {
    fn definition(&self) -> ToolDefinition {
        function_tool(
            "git_conflict_list",
            "List the current unmerged files, conflict IDs, versions, and ours/theirs meaning.",
            serde_json::json!({"type": "object", "properties": {}}),
        )
    }

    async fn execute(&self, _args: serde_json::Value) -> Result<String, String> {
        let entries = self.state.current_conflicts()?;
        let (ours, theirs) = self.state.operation.labels();
        json_string(&serde_json::json!({
            "operation": self.state.operation,
            "ours": ours,
            "theirs": theirs,
            "conflict_count": entries.len(),
            "conflicts": entries,
        }))
    }
}

#[derive(Deserialize)]
struct ConflictIdArgs {
    conflict_id: String,
}

#[derive(Clone)]
struct ConflictReadTool {
    state: Arc<ConflictToolState>,
}

#[async_trait]
impl Tool for ConflictReadTool {
    fn definition(&self) -> ToolDefinition {
        function_tool(
            "git_conflict_read",
            "Read base, ours, and theirs for one current conflict ID.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "conflict_id": {"type": "string"}
                },
                "required": ["conflict_id"]
            }),
        )
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, String> {
        let args: ConflictIdArgs = parse_args(args)?;
        let entry = self.state.conflict_by_id(&args.conflict_id)?;
        let mut content: BTreeMap<&str, Option<String>> = BTreeMap::new();
        for (name, stage) in [("base", 1), ("ours", 2), ("theirs", 3)] {
            let value = entry
                .stages
                .get(&stage)
                .map(|stage| {
                    read_blob(&self.state.repo_root, stage).and_then(|bytes| decode_text(&bytes))
                })
                .transpose()?;
            content.insert(name, value);
        }
        json_string(&serde_json::json!({
            "conflict_id": entry.conflict_id,
            "path": entry.path,
            "status": entry.status,
            "version": entry.version,
            "operation": self.state.operation,
            "content": content,
        }))
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ResolveAction {
    Content,
    UseOurs,
    UseTheirs,
    Delete,
}

#[derive(Deserialize)]
struct ResolveArgs {
    conflict_id: String,
    version: String,
    action: ResolveAction,
    #[serde(default)]
    content: Option<String>,
}

#[derive(Clone)]
struct ConflictResolveTool {
    state: Arc<ConflictToolState>,
}

#[async_trait]
impl Tool for ConflictResolveTool {
    fn definition(&self) -> ToolDefinition {
        function_tool(
            "git_conflict_resolve",
            "Resolve and stage exactly one current conflict. Only conflict IDs returned by git_conflict_list are accepted.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "conflict_id": {"type": "string"},
                    "version": {"type": "string", "description": "Exact version returned by git_conflict_read"},
                    "action": {"type": "string", "enum": ["content", "use_ours", "use_theirs", "delete"]},
                    "content": {"type": "string", "description": "Required only when action is content"}
                },
                "required": ["conflict_id", "version", "action"]
            }),
        )
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, String> {
        let args: ResolveArgs = parse_args(args)?;
        let _guard = self
            .state
            .mutation_lock
            .lock()
            .map_err(|_| "conflict mutation lock is poisoned".to_string())?;
        let entry = self.state.conflict_by_id(&args.conflict_id)?;
        if entry.version != args.version {
            return Err(format!(
                "stale conflict version for `{}`; call git_conflict_read again",
                entry.path
            ));
        }

        match args.action {
            ResolveAction::Content => {
                let content = args
                    .content
                    .ok_or_else(|| "content is required when action is content".to_string())?;
                if content.len() > MAX_RESOLVED_CONTENT_BYTES {
                    return Err(format!(
                        "resolved content is too large: {} bytes (limit {})",
                        content.len(),
                        MAX_RESOLVED_CONTENT_BYTES
                    ));
                }
                if contains_conflict_markers(&content) {
                    return Err("resolved content still contains Git conflict markers".to_string());
                }
                let target = safe_conflict_target(&self.state.repo_root, &entry.path)?;
                std::fs::write(&target, content.as_bytes())
                    .map_err(|error| format!("failed to write `{}`: {error}", entry.path))?;
            }
            ResolveAction::UseOurs | ResolveAction::UseTheirs => {
                let stage_number = if matches!(args.action, ResolveAction::UseOurs) {
                    2
                } else {
                    3
                };
                let stage = entry.stages.get(&stage_number).ok_or_else(|| {
                    format!(
                        "{} does not exist for `{}`; choose content or delete",
                        if stage_number == 2 { "ours" } else { "theirs" },
                        entry.path
                    )
                })?;
                let bytes = read_blob(&self.state.repo_root, stage)?;
                let target = safe_conflict_target(&self.state.repo_root, &entry.path)?;
                std::fs::write(&target, bytes)
                    .map_err(|error| format!("failed to write `{}`: {error}", entry.path))?;
            }
            ResolveAction::Delete => {
                let relative = validate_relative_repo_path(&entry.path)?;
                let target = self.state.repo_root.join(relative);
                let parent = target
                    .parent()
                    .ok_or_else(|| format!("conflict path has no parent: `{}`", entry.path))?
                    .canonicalize()
                    .map_err(|error| {
                        format!("failed to canonicalize parent of `{}`: {error}", entry.path)
                    })?;
                if !parent.starts_with(&self.state.repo_root) {
                    return Err(format!("path escapes the repository: `{}`", entry.path));
                }
                match std::fs::symlink_metadata(&target) {
                    Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
                        return Err(format!("refusing to delete directory `{}`", entry.path))
                    }
                    Ok(_) => std::fs::remove_file(&target)
                        .map_err(|error| format!("failed to delete `{}`: {error}", entry.path))?,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => {
                        return Err(format!("failed to inspect `{}`: {error}", entry.path))
                    }
                }
            }
        }

        stage_path(&self.state.repo_root, &entry.path)?;
        let remaining = self.state.current_conflicts()?;
        json_string(&serde_json::json!({
            "resolved": entry.path,
            "remaining_count": remaining.len(),
            "remaining": remaining.iter().map(|item| &item.path).collect::<Vec<_>>(),
        }))
    }
}

#[derive(Clone)]
struct ConflictStatusTool {
    state: Arc<ConflictToolState>,
}

#[async_trait]
impl Tool for ConflictStatusTool {
    fn definition(&self) -> ToolDefinition {
        function_tool(
            "git_conflict_status",
            "Check the authoritative Git index state after resolving conflicts.",
            serde_json::json!({"type": "object", "properties": {}}),
        )
    }

    async fn execute(&self, _args: serde_json::Value) -> Result<String, String> {
        let entries = self.state.current_conflicts()?;
        json_string(&serde_json::json!({
            "resolved": entries.is_empty(),
            "remaining_count": entries.len(),
            "remaining": entries.iter().map(|entry| &entry.path).collect::<Vec<_>>(),
        }))
    }
}

/// Build the only registry that should be supplied to a conflict-resolution
/// agent. No generic shell, unrestricted read, or generic write tool is
/// registered here.
fn build_conflict_agent_tool_registry(working_path: &str) -> Result<ToolRegistry, SessionError> {
    let state = Arc::new(ConflictToolState::new(working_path)?);
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(ConflictListTool {
        state: state.clone(),
    }));
    registry.register(Box::new(ConflictReadTool {
        state: state.clone(),
    }));
    registry.register(Box::new(RepoReadTool::new(state.repo_root.clone())));
    registry.register(Box::new(RepoSearchTool::new(state.repo_root.clone())));
    registry.register(Box::new(ConflictResolveTool {
        state: state.clone(),
    }));
    registry.register(Box::new(ConflictStatusTool { state }));
    Ok(registry)
}

/// Run a conflict-only agent for the repository's current unmerged batch.
///
/// This entry point deliberately constructs the restricted registry above
/// instead of calling `vterm_agent::tool::builtin::register_all`. Completion is
/// verified against the Git index; a natural-language success claim is never
/// treated as authoritative.
pub(crate) async fn git_resolve_conflicts_with_agent(
    working_path: &str,
    addon_host: Arc<AddonHost>,
    config: &TerminalServerConfig,
) -> GitConflictAgentResult {
    let initial_state = match ConflictToolState::new(working_path) {
        Ok(state) => state,
        Err(error) => return conflict_agent_error(error.to_string()),
    };
    let initial_conflicts = match initial_state.current_conflicts() {
        Ok(conflicts) => conflicts,
        Err(error) => return conflict_agent_error(error),
    };
    if initial_conflicts.is_empty() {
        return GitConflictAgentResult {
            resolved: true,
            remaining_files: Vec::new(),
            summary: Some("No unresolved Git conflicts were found.".to_string()),
            error: None,
        };
    }

    let addon_id = match resolve_llm_addon_id(config, "git") {
        Some(addon_id) => addon_id,
        None => return conflict_agent_error("git LLM is not configured".to_string()),
    };
    let config_overrides = config.llm.git.provider.user_config.clone();
    let config_values = resolve_addon_config(config, &addon_id, Some(&config_overrides));
    let model = config_values
        .get("model")
        .cloned()
        .unwrap_or_else(|| "gpt-4".to_string());
    let max_tokens = config_value_usize(
        &config_overrides,
        "conflict_output_max_tokens",
        DEFAULT_AGENT_OUTPUT_TOKENS,
    )
    .min(MAX_AGENT_OUTPUT_TOKENS) as u32;
    let max_iterations =
        config_value_usize(&config_overrides, "conflict_max_iterations", 50).min(200);
    let timeout = Duration::from_secs(config_value_usize(
        &config_overrides,
        "conflict_timeout_seconds",
        DEFAULT_AGENT_TIMEOUT_SECONDS as usize,
    ) as u64);

    let registry = match build_conflict_agent_tool_registry(working_path) {
        Ok(registry) => registry,
        Err(error) => return conflict_agent_error(error.to_string()),
    };
    let repo_root = initial_state.repo_root.display().to_string();
    let conflict_paths = initial_conflicts
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<Vec<_>>()
        .join("\n- ");
    let user_prompt = format!(
        "Resolve the current Git conflict batch.\n\nRepository: {repo_root}\nOperation: {:?}\nConflicted files:\n- {conflict_paths}\n\nUse the restricted tools to inspect and resolve every file. Do not continue the Git operation.",
        initial_state.operation
    );

    let addon_id_callback = addon_id.clone();
    let config_callback = config.clone();
    let overrides_callback = config_overrides.clone();
    let callback: LlmCallback = Box::new(move |messages_json, options_json| {
        let result = run_chat_completion(
            &addon_host,
            &config_callback,
            &addon_id_callback,
            messages_json,
            options_json,
            Some(&overrides_callback),
            Some(timeout),
            None,
        );
        if !result.success {
            return Err(result
                .error
                .unwrap_or_else(|| "conflict LLM call failed".to_string()));
        }
        result
            .raw_body
            .ok_or_else(|| "conflict LLM returned no response body".to_string())
    });
    let client = ConflictLlmClient {
        callback: Arc::new(callback),
    };
    let agent = Agent::new(
        Arc::new(client),
        Arc::new(registry),
        AgentConfig {
            max_iterations,
            model,
            temperature: Some(0.1),
            max_tokens: Some(max_tokens),
            compression: CompressionConfig::default(),
        },
    );
    let session_id = agent
        .create_session(CONFLICT_AGENT_SYSTEM_PROMPT, user_prompt)
        .await;
    let loop_result = agent.run_loop(&session_id, |_| {}).await;
    let (summary, loop_error) = match loop_result {
        LoopResult::Completed { final_content, .. } => (Some(final_content), None),
        LoopResult::MaxIterationsExceeded {
            last_content,
            iterations,
            ..
        } => (
            Some(last_content),
            Some(format!("conflict agent exceeded {iterations} iterations")),
        ),
        LoopResult::Cancelled { iterations, .. } => (
            None,
            Some(format!(
                "conflict agent was cancelled after {iterations} iterations"
            )),
        ),
        LoopResult::Error {
            error, iterations, ..
        } => (
            None,
            Some(format!(
                "conflict agent failed after {iterations} iterations: {error}"
            )),
        ),
    };

    let remaining = match initial_state.current_conflicts() {
        Ok(conflicts) => conflicts,
        Err(error) => {
            return GitConflictAgentResult {
                resolved: false,
                remaining_files: initial_conflicts
                    .into_iter()
                    .map(|entry| entry.path)
                    .collect(),
                summary,
                error: Some(loop_error.unwrap_or(error)),
            }
        }
    };
    let resolved = remaining.is_empty();
    let error = if resolved {
        loop_error
    } else {
        Some(loop_error.unwrap_or_else(|| {
            format!(
                "conflict agent stopped with {} unresolved file(s)",
                remaining.len()
            )
        }))
    };
    GitConflictAgentResult {
        resolved,
        remaining_files: remaining.into_iter().map(|entry| entry.path).collect(),
        summary,
        error,
    }
}

fn conflict_agent_error(error: String) -> GitConflictAgentResult {
    GitConflictAgentResult {
        resolved: false,
        remaining_files: Vec::new(),
        summary: None,
        error: Some(error),
    }
}

fn config_value_usize(
    values: &std::collections::HashMap<String, String>,
    key: &str,
    default_value: usize,
) -> usize {
    values
        .get(key)
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default_value)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use vterm_agent::tool::ToolRegistry;

    use super::*;

    fn git(repo: &Path, args: &[&str]) {
        let output = crate::session::git::git_command()
            .args(args)
            .current_dir(repo)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn write_commit(repo: &Path, path: &str, content: &str, message: &str) {
        fs::write(repo.join(path), content).unwrap();
        git(repo, &["add", "--", path]);
        git(repo, &["commit", "-m", message]);
    }

    fn conflicted_repo() -> tempfile::TempDir {
        let repo = tempfile::tempdir().unwrap();
        git(repo.path(), &["init"]);
        git(repo.path(), &["config", "user.email", "test@example.com"]);
        git(repo.path(), &["config", "user.name", "Test"]);
        write_commit(repo.path(), "conflict.txt", "base\n", "base");
        git(repo.path(), &["checkout", "-b", "side"]);
        write_commit(repo.path(), "conflict.txt", "side\n", "side");
        git(repo.path(), &["checkout", "-"]);
        write_commit(repo.path(), "conflict.txt", "main\n", "main");
        let output = crate::session::git::git_command()
            .args(["merge", "side"])
            .current_dir(repo.path())
            .output()
            .unwrap();
        assert!(!output.status.success());
        repo
    }

    fn multi_conflicted_repo() -> tempfile::TempDir {
        let repo = tempfile::tempdir().unwrap();
        git(repo.path(), &["init"]);
        git(repo.path(), &["config", "user.email", "test@example.com"]);
        git(repo.path(), &["config", "user.name", "Test"]);
        fs::write(repo.path().join("first.txt"), "base first\n").unwrap();
        fs::write(repo.path().join("second.txt"), "base second\n").unwrap();
        git(repo.path(), &["add", "--", "first.txt", "second.txt"]);
        git(repo.path(), &["commit", "-m", "base"]);
        git(repo.path(), &["checkout", "-b", "side"]);
        fs::write(repo.path().join("first.txt"), "side first\n").unwrap();
        fs::write(repo.path().join("second.txt"), "side second\n").unwrap();
        git(repo.path(), &["add", "--", "first.txt", "second.txt"]);
        git(repo.path(), &["commit", "-m", "side"]);
        git(repo.path(), &["checkout", "-"]);
        fs::write(repo.path().join("first.txt"), "main first\n").unwrap();
        fs::write(repo.path().join("second.txt"), "main second\n").unwrap();
        git(repo.path(), &["add", "--", "first.txt", "second.txt"]);
        git(repo.path(), &["commit", "-m", "main"]);
        let output = crate::session::git::git_command()
            .args(["merge", "side"])
            .current_dir(repo.path())
            .output()
            .unwrap();
        assert!(!output.status.success());
        repo
    }

    async fn execute(
        registry: &ToolRegistry,
        name: &str,
        args: serde_json::Value,
    ) -> serde_json::Value {
        let output = registry.execute(name, args).await.unwrap();
        serde_json::from_str(&output).unwrap()
    }

    #[tokio::test]
    async fn conflict_registry_is_explicit_and_contains_no_shell_or_generic_write() {
        let repo = conflicted_repo();
        let registry = build_conflict_agent_tool_registry(repo.path().to_str().unwrap()).unwrap();
        let definitions = registry.get_definitions();
        let names = definitions
            .iter()
            .filter_map(|definition| definition.get("function")?.get("name")?.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names.len(), 6);
        assert!(!names.contains(&"shell"));
        assert!(!names.contains(&"file_read"));
        assert!(!names.contains(&"file_write"));
        assert!(names.contains(&"git_conflict_resolve"));
    }

    #[tokio::test]
    async fn resolves_one_conflict_by_id_and_checks_authoritative_status() {
        let repo = conflicted_repo();
        let registry = build_conflict_agent_tool_registry(repo.path().to_str().unwrap()).unwrap();
        let list = execute(&registry, "git_conflict_list", serde_json::json!({})).await;
        assert_eq!(list["operation"], "merge");
        assert_eq!(list["conflict_count"], 1);
        let id = list["conflicts"][0]["conflict_id"]
            .as_str()
            .unwrap()
            .to_string();
        let read = execute(
            &registry,
            "git_conflict_read",
            serde_json::json!({"conflict_id": id}),
        )
        .await;
        assert_eq!(read["content"]["base"], "base\n");
        assert_eq!(read["content"]["ours"], "main\n");
        assert_eq!(read["content"]["theirs"], "side\n");

        let resolved = execute(
            &registry,
            "git_conflict_resolve",
            serde_json::json!({
                "conflict_id": read["conflict_id"],
                "version": read["version"],
                "action": "content",
                "content": "main and side\n"
            }),
        )
        .await;
        assert_eq!(resolved["remaining_count"], 0);
        assert_eq!(
            fs::read_to_string(repo.path().join("conflict.txt")).unwrap(),
            "main and side\n"
        );

        let status = execute(&registry, "git_conflict_status", serde_json::json!({})).await;
        assert_eq!(status["resolved"], true);
    }

    #[tokio::test]
    async fn rejects_stale_versions_without_writing() {
        let repo = conflicted_repo();
        let registry = build_conflict_agent_tool_registry(repo.path().to_str().unwrap()).unwrap();
        let list = execute(&registry, "git_conflict_list", serde_json::json!({})).await;
        let id = list["conflicts"][0]["conflict_id"].as_str().unwrap();
        let result = registry
            .execute(
                "git_conflict_resolve",
                serde_json::json!({
                    "conflict_id": id,
                    "version": "stale",
                    "action": "content",
                    "content": "should not be written\n"
                }),
            )
            .await;
        assert!(result.unwrap_err().contains("stale conflict version"));
        assert_ne!(
            fs::read_to_string(repo.path().join("conflict.txt")).unwrap(),
            "should not be written\n"
        );
    }

    #[test]
    fn existing_client_bridge_rejects_non_conflict_escape_paths() {
        let repo = conflicted_repo();
        let outside_name = format!(
            "outside-{}.txt",
            repo.path().file_name().unwrap().to_string_lossy()
        );
        let outside = repo.path().parent().unwrap().join(&outside_name);
        let result = git_resolve_conflict_content(
            repo.path().to_str().unwrap(),
            &format!("../{outside_name}"),
            "escaped\n",
        );
        assert!(result.is_err());
        assert!(!outside.exists());
    }

    #[test]
    fn existing_client_bridge_rejects_unresolved_markers() {
        let repo = conflicted_repo();
        let before = fs::read_to_string(repo.path().join("conflict.txt")).unwrap();
        let result = git_resolve_conflict_content(
            repo.path().to_str().unwrap(),
            "conflict.txt",
            "<<<<<<< ours\nmain\n=======\nside\n>>>>>>> theirs\n",
        );
        assert!(result.unwrap_err().to_string().contains("conflict markers"));
        assert_eq!(
            fs::read_to_string(repo.path().join("conflict.txt")).unwrap(),
            before
        );
    }

    #[tokio::test]
    async fn resolves_multiple_conflicts_independently() {
        let repo = multi_conflicted_repo();
        let registry = build_conflict_agent_tool_registry(repo.path().to_str().unwrap()).unwrap();
        let list = execute(&registry, "git_conflict_list", serde_json::json!({})).await;
        assert_eq!(list["conflict_count"], 2);

        for (index, conflict) in list["conflicts"].as_array().unwrap().iter().enumerate() {
            let read = execute(
                &registry,
                "git_conflict_read",
                serde_json::json!({"conflict_id": conflict["conflict_id"]}),
            )
            .await;
            let action = if index == 0 { "use_ours" } else { "use_theirs" };
            execute(
                &registry,
                "git_conflict_resolve",
                serde_json::json!({
                    "conflict_id": read["conflict_id"],
                    "version": read["version"],
                    "action": action
                }),
            )
            .await;
        }

        let status = execute(&registry, "git_conflict_status", serde_json::json!({})).await;
        assert_eq!(status["resolved"], true);
        assert_eq!(status["remaining_count"], 0);
    }
}
