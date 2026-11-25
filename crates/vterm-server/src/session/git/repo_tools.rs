//! Read-only repository tools shared by the git agents.
//!
//! Every tool here is strictly read-only and sandboxed to the repository
//! root: no shell, no writes, no staging. The rebase agent uses them to
//! gather context (the incoming commit's message/diff, surrounding code)
//! before proposing a hunk resolution; the conflict agent reuses the file
//! read and search tools.

use std::path::{Component, Path, PathBuf};
use std::process::Output;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use vterm_agent::tool::{FunctionDefinition, Tool, ToolDefinition, ToolRegistry};

const MAX_REPOSITORY_READ_BYTES: usize = 256 * 1024;
const MAX_SEARCH_OUTPUT_BYTES: usize = 64 * 1024;
const MAX_COMMIT_INFO_BYTES: usize = 16 * 1024;
const MAX_COMMIT_DIFF_BYTES: usize = 32 * 1024;
/// Upper bound for model-supplied revision names and pathspecs.
const MAX_REV_LEN: usize = 128;
const MAX_PATHSPEC_LEN: usize = 256;

/// Registry with the four read-only tools, rooted at a canonicalized
/// repository root. No shell, no write, no staging tool is registered here.
pub(crate) fn read_only_repo_tools(repo_root: PathBuf) -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(RepoReadTool::new(repo_root.clone())));
    registry.register(Box::new(RepoSearchTool::new(repo_root.clone())));
    registry.register(Box::new(GitCommitInfoTool::new(repo_root.clone())));
    registry.register(Box::new(GitCommitDiffTool::new(repo_root)));
    registry
}

pub(crate) fn run_git(repo_root: &Path, args: &[&str]) -> Result<Output, String> {
    let output = crate::session::git::git_command()
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|error| format!("failed to run git {}: {error}", args.join(" ")))?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(format!(
            "git {} failed with {:?}: {}",
            args.join(" "),
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

pub(crate) fn validate_relative_repo_path(path: &str) -> Result<PathBuf, String> {
    if path.is_empty() {
        return Err("path must not be empty".to_string());
    }
    let path = Path::new(path);
    if path.is_absolute() {
        return Err("absolute paths are not allowed".to_string());
    }

    let mut relative = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => {
                if value.to_string_lossy().eq_ignore_ascii_case(".git") {
                    return Err("access to .git is not allowed".to_string());
                }
                relative.push(value);
            }
            Component::CurDir => {}
            Component::ParentDir => {
                return Err("parent-directory components are not allowed".to_string())
            }
            Component::Prefix(_) | Component::RootDir => {
                return Err("rooted paths are not allowed".to_string())
            }
        }
    }
    if relative.as_os_str().is_empty() {
        return Err("path must identify a file".to_string());
    }
    Ok(relative)
}

pub(crate) fn safe_existing_file(repo_root: &Path, path: &str) -> Result<PathBuf, String> {
    let relative = validate_relative_repo_path(path)?;
    let joined = repo_root.join(relative);
    let metadata = std::fs::symlink_metadata(&joined)
        .map_err(|error| format!("failed to inspect `{path}`: {error}"))?;
    if metadata.file_type().is_symlink() {
        return Err(format!("symbolic links are not supported: `{path}`"));
    }
    if !metadata.is_file() {
        return Err(format!("path is not a regular file: `{path}`"));
    }
    let canonical = joined
        .canonicalize()
        .map_err(|error| format!("failed to canonicalize `{path}`: {error}"))?;
    if !canonical.starts_with(repo_root) {
        return Err(format!("path escapes the repository: `{path}`"));
    }
    Ok(canonical)
}

pub(crate) fn decode_text(bytes: &[u8]) -> Result<String, String> {
    if bytes.iter().any(|byte| *byte == 0) {
        return Err("binary or UTF-16 files are not supported by the agent".to_string());
    }
    String::from_utf8(bytes.to_vec()).map_err(|_| "content is not valid UTF-8".to_string())
}

pub(crate) fn function_tool(
    name: &str,
    description: &str,
    parameters: serde_json::Value,
) -> ToolDefinition {
    ToolDefinition {
        r#type: "function".to_string(),
        function: FunctionDefinition {
            name: name.to_string(),
            description: description.to_string(),
            parameters,
        },
    }
}

pub(crate) fn parse_args<T: for<'de> Deserialize<'de>>(
    args: serde_json::Value,
) -> Result<T, String> {
    serde_json::from_value(args).map_err(|error| format!("invalid arguments: {error}"))
}

pub(crate) fn json_string(value: &impl Serialize) -> Result<String, String> {
    serde_json::to_string(value)
        .map_err(|error| format!("failed to serialize tool result: {error}"))
}

pub(crate) fn floor_char_boundary(value: &str, mut index: usize) -> usize {
    index = index.min(value.len());
    while index > 0 && !value.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn truncate_output(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.trim().to_string();
    }
    let kept = floor_char_boundary(text, max);
    format!(
        "{}\n[output truncated: {} bytes total, kept first {}]",
        text[..kept].trim_end(),
        text.len(),
        kept
    )
}

/// A model-supplied revision is inserted as a plain argv element (never a
/// shell), but a leading dash would still let it masquerade as a git
/// option — reject those and cap the length.
fn validate_rev(rev: &str) -> Result<&str, String> {
    let rev = rev.trim();
    if rev.is_empty() || rev.len() > MAX_REV_LEN {
        return Err(format!("rev must contain 1 to {MAX_REV_LEN} bytes"));
    }
    if rev.starts_with('-') {
        return Err("rev must not start with '-'".to_string());
    }
    Ok(rev)
}

/// The commit whose context the agent usually wants: during a rebase stop
/// `REBASE_HEAD` names the commit being replayed; otherwise fall back to
/// HEAD.
fn default_context_rev(repo_root: &Path) -> String {
    for candidate in ["REBASE_HEAD", "HEAD"] {
        if run_git(repo_root, &["rev-parse", "--verify", "--quiet", candidate]).is_ok() {
            return candidate.to_string();
        }
    }
    "HEAD".to_string()
}

fn resolve_rev(repo_root: &Path, rev: Option<&str>) -> Result<String, String> {
    match rev {
        Some(rev) => Ok(validate_rev(rev)?.to_string()),
        None => Ok(default_context_rev(repo_root)),
    }
}

/* ── git_repo_read ─────────────────────────────────────── */

#[derive(Deserialize)]
struct RepoReadArgs {
    path: String,
    #[serde(default)]
    offset: Option<usize>,
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Clone)]
pub(crate) struct RepoReadTool {
    repo_root: PathBuf,
}

impl RepoReadTool {
    pub(crate) fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }
}

#[async_trait]
impl Tool for RepoReadTool {
    fn definition(&self) -> ToolDefinition {
        function_tool(
            "git_repo_read",
            "Read a UTF-8 regular file inside the repository. Paths outside the repository and .git are rejected.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "offset": {"type": "integer", "minimum": 1},
                    "limit": {"type": "integer", "minimum": 1, "maximum": 2000}
                },
                "required": ["path"]
            }),
        )
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, String> {
        let args: RepoReadArgs = parse_args(args)?;
        let path = safe_existing_file(&self.repo_root, &args.path)?;
        let metadata = path
            .metadata()
            .map_err(|error| format!("failed to inspect `{}`: {error}", args.path))?;
        if metadata.len() as usize > MAX_REPOSITORY_READ_BYTES {
            return Err(format!(
                "file is too large: {} bytes (limit {})",
                metadata.len(),
                MAX_REPOSITORY_READ_BYTES
            ));
        }
        let content = std::fs::read(&path)
            .map_err(|error| format!("failed to read `{}`: {error}", args.path))
            .and_then(|bytes| decode_text(&bytes))?;
        let offset = args.offset.unwrap_or(1).saturating_sub(1);
        let limit = args.limit.unwrap_or(500).min(2000);
        let selected = content
            .lines()
            .skip(offset)
            .take(limit)
            .enumerate()
            .map(|(index, line)| format!("{}: {}", offset + index + 1, line))
            .collect::<Vec<_>>()
            .join("\n");
        Ok(selected)
    }
}

/* ── git_repo_search ───────────────────────────────────── */

#[derive(Deserialize)]
struct RepoSearchArgs {
    pattern: String,
    #[serde(default)]
    pathspec: Option<String>,
}

#[derive(Clone)]
pub(crate) struct RepoSearchTool {
    repo_root: PathBuf,
}

impl RepoSearchTool {
    pub(crate) fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }
}

#[async_trait]
impl Tool for RepoSearchTool {
    fn definition(&self) -> ToolDefinition {
        function_tool(
            "git_repo_search",
            "Search tracked repository text with git grep. This never invokes a shell.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {"type": "string", "maxLength": 256},
                    "pathspec": {"type": "string", "description": "Optional Git pathspec such as *.rs"}
                },
                "required": ["pattern"]
            }),
        )
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, String> {
        let args: RepoSearchArgs = parse_args(args)?;
        if args.pattern.is_empty() || args.pattern.len() > 256 {
            return Err("pattern must contain 1 to 256 bytes".to_string());
        }
        let mut command = crate::session::git::git_command();
        command
            .args(["grep", "-n", "-I", "--full-name", "-e", &args.pattern, "--"])
            .current_dir(&self.repo_root);
        if let Some(pathspec) = args.pathspec.as_deref() {
            if pathspec.len() > MAX_PATHSPEC_LEN {
                return Err("pathspec is too long".to_string());
            }
            command.arg(pathspec);
        }
        let output = command
            .output()
            .map_err(|error| format!("failed to run git grep: {error}"))?;
        // git grep returns 1 when no lines match.
        if !output.status.success() && output.status.code() != Some(1) {
            return Err(format!(
                "git grep failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(if stdout.trim().is_empty() {
            "No matches found.".to_string()
        } else {
            truncate_output(&stdout, MAX_SEARCH_OUTPUT_BYTES)
        })
    }
}

/* ── git_commit_info ───────────────────────────────────── */

#[derive(Deserialize)]
struct CommitInfoArgs {
    #[serde(default)]
    rev: Option<String>,
}

#[derive(Clone)]
pub(crate) struct GitCommitInfoTool {
    repo_root: PathBuf,
}

impl GitCommitInfoTool {
    pub(crate) fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }
}

#[async_trait]
impl Tool for GitCommitInfoTool {
    fn definition(&self) -> ToolDefinition {
        function_tool(
            "git_commit_info",
            "Show a commit's message, author, and changed-file summary. Without `rev`, uses REBASE_HEAD (the commit being replayed) during a rebase, else HEAD.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "rev": {"type": "string", "description": "Revision such as HEAD, REBASE_HEAD, or a commit hash"}
                }
            }),
        )
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, String> {
        let args: CommitInfoArgs = parse_args(args)?;
        let rev = resolve_rev(&self.repo_root, args.rev.as_deref())?;
        let log = run_git(&self.repo_root, &["log", "-1", "--format=fuller", &rev])?;
        let stat = run_git(&self.repo_root, &["show", "--stat", "--format=", &rev])?;
        let mut output = String::from_utf8_lossy(&log.stdout).into_owned();
        output.push_str("\nChanged files:\n");
        output.push_str(String::from_utf8_lossy(&stat.stdout).trim());
        Ok(truncate_output(&output, MAX_COMMIT_INFO_BYTES))
    }
}

/* ── git_commit_diff ───────────────────────────────────── */

#[derive(Deserialize)]
struct CommitDiffArgs {
    #[serde(default)]
    rev: Option<String>,
    #[serde(default)]
    path: Option<String>,
}

#[derive(Clone)]
pub(crate) struct GitCommitDiffTool {
    repo_root: PathBuf,
}

impl GitCommitDiffTool {
    pub(crate) fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }
}

#[async_trait]
impl Tool for GitCommitDiffTool {
    fn definition(&self) -> ToolDefinition {
        function_tool(
            "git_commit_diff",
            "Show the patch a commit introduced, optionally limited to one file. Without `rev`, uses REBASE_HEAD (the commit being replayed) during a rebase, else HEAD.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "rev": {"type": "string", "description": "Revision such as HEAD, REBASE_HEAD, or a commit hash"},
                    "path": {"type": "string", "description": "Optional repository-relative file path to limit the diff to"}
                }
            }),
        )
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, String> {
        let args: CommitDiffArgs = parse_args(args)?;
        let rev = resolve_rev(&self.repo_root, args.rev.as_deref())?;
        // `git show --format=` prints just the patch; for a root commit it
        // diffs against the empty tree, so no `<rev>^` dance is needed.
        let mut argv = vec!["show", "--format=", rev.as_str()];
        let path_storage;
        if let Some(path) = args.path.as_deref() {
            let path = path.trim();
            if path.is_empty() || path.len() > MAX_PATHSPEC_LEN || path.starts_with('-') {
                return Err("invalid path filter".to_string());
            }
            path_storage = path.to_string();
            argv.push("--");
            argv.push(&path_storage);
        }
        let output = run_git(&self.repo_root, &argv)?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(if stdout.trim().is_empty() {
            "No diff (empty patch for this revision/path).".to_string()
        } else {
            truncate_output(&stdout, MAX_COMMIT_DIFF_BYTES)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_validation_rejects_escape_and_git_metadata() {
        assert!(validate_relative_repo_path("../outside").is_err());
        assert!(validate_relative_repo_path("C:\\outside").is_err());
        assert!(validate_relative_repo_path(".git/config").is_err());
        assert!(validate_relative_repo_path("src/lib.rs").is_ok());
    }

    #[test]
    fn rev_validation_rejects_option_injection() {
        assert!(validate_rev("HEAD").is_ok());
        assert!(validate_rev("REBASE_HEAD").is_ok());
        assert!(validate_rev("abc123").is_ok());
        assert!(validate_rev("--help").is_err());
        assert!(validate_rev("-HEAD").is_err());
        assert!(validate_rev("").is_err());
        assert!(validate_rev(&"a".repeat(MAX_REV_LEN + 1)).is_err());
    }

    #[test]
    fn truncate_output_marks_truncation_on_char_boundary() {
        assert_eq!(truncate_output("short", 10), "short");
        let text = "无法访问文件".repeat(100);
        let truncated = truncate_output(&text, 30);
        assert!(truncated.contains("[output truncated:"));
        assert!(truncated.starts_with("无法访问文件"));
    }

    #[tokio::test]
    async fn read_only_registry_contains_no_shell_or_write_tools() {
        let registry = read_only_repo_tools(PathBuf::from("."));
        let definitions = registry.get_definitions();
        let names = definitions
            .iter()
            .filter_map(|definition| definition.get("function")?.get("name")?.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names.len(), 4);
        for expected in [
            "git_repo_read",
            "git_repo_search",
            "git_commit_info",
            "git_commit_diff",
        ] {
            assert!(names.contains(&expected), "missing tool {expected}");
        }
        for banned in ["shell", "file_read", "file_write", "git_conflict_resolve"] {
            assert!(!names.contains(&banned), "unexpected tool {banned}");
        }
    }
}
