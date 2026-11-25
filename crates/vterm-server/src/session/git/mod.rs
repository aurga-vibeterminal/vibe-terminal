use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Component, Path};
use std::process::Stdio;
use std::time::Duration;

use tracing::{info, trace, warn};
use vterm_common::{GitBranchInfo, GitChangedFile, GitCommitInfo, GitFileStatus};
use vterm_core::protocol::FileDiffRange;

use crate::session::SessionError;

mod agent_client;
mod analysis;
mod commit;
pub(super) mod conflict;
mod rebase;
mod rebase_agent;
pub(super) mod repo_tools;
mod stash;
mod submodule;
mod tracker;

pub(super) use analysis::git_analyze_hunks_streaming;
pub(super) use commit::git_generate_commit_message_with_agent;
pub(super) use commit::truncate_diff_preserve_hunks;
pub(super) use rebase::{
    git_rebase_abort, git_rebase_apply, git_rebase_conflict_content, git_rebase_continue,
    git_rebase_resolve, git_rebase_skip, git_rebase_todo,
};
pub(super) use rebase_agent::git_rebase_ai_resolve_streaming;
pub(super) use stash::{
    git_do_stash_apply, git_do_stash_drop, git_do_stash_pop, git_do_stash_push, git_read_stash_list,
};
pub(super) use submodule::git_read_submodule_list;

/// A `git` command pre-configured with the invocation-scoped settings every
/// git call in this module needs. `core.quotepath=off` makes non-ASCII paths
/// (e.g. `Doc/项目说明.md`) print as raw UTF-8 instead of quoted octal
/// escapes (`"\351\241\271..."`), which the client would otherwise display
/// verbatim. The `GIT_CONFIG_*` environment variables scope the setting to
/// the processes we spawn — the user's repository config is never modified.
/// `GIT_TERMINAL_PROMPT=0` / `GCM_INTERACTIVE=never` make credential prompts
/// fail fast instead of blocking forever (Git Credential Manager would
/// otherwise pop a GUI dialog and hang the spawned process indefinitely).
/// The returned [`tracker::GitCommand`] also registers every spawned process
/// in the in-flight git-op tracker (see `tracker.rs`).
pub(super) fn git_command() -> tracker::GitCommand {
    // The raw `Command::new("git")` here must not go through git_command()
    // itself — that would recurse forever.
    let mut command = std::process::Command::new("git");
    vterm_common::command_no_window(&mut command);
    command
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "core.quotepath")
        .env("GIT_CONFIG_VALUE_0", "off")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "never");
    tracker::GitCommand::new(command)
}

/// Cap on how long a network-bound git operation (fetch/pull/push) may run
/// before the child is killed and a timeout error is returned to the client.
const NETWORK_OP_TIMEOUT: Duration = Duration::from_secs(120);

pub(super) fn git_read_changed_files(
    working_path: &str,
    commit: Option<&str>,
) -> Result<Vec<GitChangedFile>, SessionError> {
    git_check_repo(working_path)?;

    if let Some(commit) = commit {
        if !commit.is_empty() {
            return git_read_commit_changed_files(working_path, commit);
        }
    }

    git_read_working_tree_changed_files(working_path)
}

pub(super) fn git_read_log(
    working_path: &str,
    path: Option<&str>,
    commit: Option<&str>,
    branch: Option<&str>,
) -> Result<Vec<GitCommitInfo>, SessionError> {
    git_check_repo(working_path)?;
    let repo_root = git_repo_root(working_path)?;

    let mut args = vec![
        "log".to_string(),
        "--max-count=100".to_string(),
        "--topo-order".to_string(),
        "--format=%H%x00%P%x00%B%x00%an%x00%cr%x00%ci%x1e".to_string(),
    ];
    let revision = commit
        .map(str::trim)
        .filter(|commit| !commit.is_empty())
        .or_else(|| branch.map(str::trim).filter(|branch| !branch.is_empty()))
        .unwrap_or("HEAD");
    args.push(revision.to_string());
    if let Some(path) = path.and_then(|path| git_log_pathspec(&repo_root, path)) {
        if !path.is_empty() {
            args.push("--".to_string());
            args.push(path);
        }
    }

    trace!("[git_log] cwd={} git {}", repo_root, args.join(" "));

    let output = crate::session::git::git_command()
        .args(&args)
        .current_dir(&repo_root)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.contains("ambiguous argument 'HEAD'")
            || stderr.contains("unknown revision or path")
        {
            return Ok(Vec::new());
        }
        return Err(SessionError::FileSystemError(format!(
            "git log failed: {stderr}"
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();

    for record in stdout.split('\x1e') {
        let record = record.trim_matches(['\r', '\n']);
        if record.is_empty() {
            continue;
        }
        let mut parts = record.splitn(6, '\0');
        let Some(hash) = parts.next() else { continue };
        let Some(parents) = parts.next() else {
            continue;
        };
        let Some(message) = parts.next() else {
            continue;
        };
        let Some(author) = parts.next() else { continue };
        let Some(relative_time) = parts.next() else {
            continue;
        };
        let Some(exact_time) = parts.next() else {
            continue;
        };
        if hash.is_empty() {
            continue;
        }
        commits.push(GitCommitInfo {
            hash: hash.to_string(),
            message: message.trim_end_matches(['\r', '\n']).to_string(),
            author: author.to_string(),
            date: relative_time.to_string(),
            exact_time: exact_time.trim_end_matches(['\r', '\n']).to_string(),
            parents: parents
                .split_whitespace()
                .filter(|parent| !parent.is_empty())
                .map(ToString::to_string)
                .collect(),
        });
    }

    Ok(commits)
}

fn git_log_pathspec(repo_root: &str, path: &str) -> Option<String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }

    let normalized = trimmed.replace('\\', "/");
    let path = Path::new(&normalized);
    let repo_root = Path::new(repo_root);

    if path.is_absolute() {
        let Ok(relative) = path.strip_prefix(repo_root) else {
            return Some(normalized);
        };
        let relative = path_to_git_path(relative);
        if relative.is_empty() {
            None
        } else {
            Some(relative)
        }
    } else {
        Some(normalized.trim_start_matches("./").to_string())
    }
}

fn path_to_git_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

pub(super) fn git_read_worktree_file_diff(
    working_path: &str,
    path: &str,
) -> Result<String, SessionError> {
    git_check_repo(working_path)?;
    let repo_root = git_repo_root(working_path)?;
    let Some(pathspec) = git_log_pathspec(&repo_root, path) else {
        return Ok(String::new());
    };
    let raw = git_read_diff(working_path, &pathspec, None, false, false)?;
    Ok(String::from_utf8_lossy(&raw).into_owned())
}

pub(super) fn git_read_worktree_file_diff_ranges(
    working_path: &str,
    path: &str,
) -> Result<Vec<FileDiffRange>, SessionError> {
    let repo_root = git_repo_root_cached(working_path)?;
    let Some(pathspec) = git_log_pathspec(&repo_root, path) else {
        return Ok(Vec::new());
    };

    // One `git diff HEAD` (worktree vs last commit) instead of separate
    // unstaged + staged invocations: the union of added-line ranges is
    // equivalent for gutter markers, and each git spawn costs ~25 ms on
    // Windows — this call sits on the file-open critical path.
    let unstaged_and_staged = run_diff_best_effort(
        &repo_root,
        &["diff", "--unified=0", "HEAD", "--", &pathspec],
    );

    // Only the ASCII `@@` headers are parsed here, so a lossy decode of
    // non-UTF-8 content cannot shift the line numbers.
    Ok(parse_current_file_added_ranges(&String::from_utf8_lossy(
        &unstaged_and_staged,
    )))
}

/// Repo-root lookups, keyed by working path. `git rev-parse --show-toplevel`
/// both verifies we're inside a work tree and yields its root, but is a
/// ~25 ms subprocess on Windows — and every file open used to pay it (plus
/// a second `--is-inside-work-tree` check). Roots are stable for the life
/// of a session, so resolve each distinct working path once. `None` caches
/// the not-a-repo answer too.
static REPO_ROOT_CACHE: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<String, Option<String>>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

fn git_repo_root_cached(working_path: &str) -> Result<String, SessionError> {
    if let Some(cached) = REPO_ROOT_CACHE.lock().unwrap().get(working_path) {
        return cached.clone().ok_or(SessionError::FileSystemError(
            "Not a git repository".to_string(),
        ));
    }
    let resolved = git_repo_root(working_path).ok();
    REPO_ROOT_CACHE
        .lock()
        .unwrap()
        .insert(working_path.to_string(), resolved.clone());
    resolved.ok_or(SessionError::FileSystemError(
        "Not a git repository".to_string(),
    ))
}

fn parse_current_file_added_ranges(diff: &str) -> Vec<FileDiffRange> {
    if diff.trim().is_empty() || diff.lines().any(|line| line == "--- /dev/null") {
        return Vec::new();
    }

    let mut ranges = Vec::new();
    let mut new_line: Option<u32> = None;
    let mut active_start: Option<u32> = None;
    let mut active_end: u32 = 0;

    let flush = |ranges: &mut Vec<FileDiffRange>, start: &mut Option<u32>, end: u32| {
        if let Some(start_line) = start.take() {
            if start_line < end {
                ranges.push(FileDiffRange {
                    start_line,
                    end_line: end,
                });
            }
        }
    };

    for line in diff.lines() {
        if let Some((start, _count)) = parse_unified_new_hunk(line) {
            flush(&mut ranges, &mut active_start, active_end);
            new_line = Some(start.saturating_sub(1));
            continue;
        }

        let Some(current) = new_line else {
            continue;
        };

        if line.starts_with("+++") || line.starts_with("---") || line.starts_with("diff --git ") {
            continue;
        }

        if line.starts_with('+') {
            if active_start.is_none() {
                active_start = Some(current);
            }
            active_end = current + 1;
            new_line = Some(current + 1);
        } else {
            flush(&mut ranges, &mut active_start, active_end);
            if line.starts_with(' ') || line.is_empty() || line.starts_with('\\') {
                new_line = Some(current + 1);
            }
        }
    }

    flush(&mut ranges, &mut active_start, active_end);
    ranges
}

fn parse_unified_new_hunk(line: &str) -> Option<(u32, u32)> {
    let rest = line.strip_prefix("@@ ")?;
    let plus = rest.find('+')?;
    let after_plus = &rest[plus + 1..];
    let end = after_plus.find(' ')?;
    let spec = &after_plus[..end];
    let mut parts = spec.splitn(2, ',');
    let start = parts.next()?.parse::<u32>().ok()?;
    let count = parts
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(1);
    Some((start, count))
}

pub(super) fn git_read_diff(
    working_path: &str,
    path: &str,
    commit: Option<&str>,
    cached: bool,
    compare_with_worktree: bool,
) -> Result<Vec<u8>, SessionError> {
    git_check_repo(working_path)?;
    let repo_root = git_repo_root(working_path)?;

    if path.is_empty() {
        return Ok(Vec::new());
    }

    // 1. Compare a specific revision directly against the working tree.
    if let Some(commit) = commit.filter(|c| !c.is_empty()) {
        if compare_with_worktree {
            let diff = run_diff_best_effort(&repo_root, &["diff", "-U999999", commit, "--", path]);
            //             tracing::info!(
            //                 "[git_diff] case=revision_vs_worktree path={} commit={} len={}",
            //                 path,
            //                 commit,
            //                 diff.len()
            //             );
            return Ok(diff);
        }

        //         tracing::info!("[git_diff] case=committed path={} commit={}", path, commit);
        // Try parent commit first
        let diff = run_diff_best_effort(
            &repo_root,
            &[
                "diff",
                "-U999999",
                &format!("{}^..{}", commit, commit),
                "--",
                path,
            ],
        );
        if !diff.is_empty() {
            return Ok(diff);
        }
        // Root commit (no parent) — diff against empty tree
        //         tracing::info!(
        //             "[git_diff] case=root_commit path={} commit={}",
        //             path,
        //             commit
        //         );
        let diff = run_diff_best_effort(
            &repo_root,
            &[
                "diff",
                "-U999999",
                &format!("4b825dc642cb6eb9a060e54bf899d69f2b7c0d41..{}", commit),
                "--",
                path,
            ],
        );
        if !diff.is_empty() {
            return Ok(diff);
        }
        //         tracing::warn!(
        //             "[git_diff] case=committed_empty path={} commit={}",
        //             path,
        //             commit
        //         );
        return Ok(Vec::new());
    }

    // 2. Staged file — index vs HEAD
    if cached {
        //         tracing::info!("[git_diff] case=staged path={}", path);
        return run_diff_required(&repo_root, &["diff", "-U999999", "--cached", "--", path]);
    }

    // 3. Unstaged tracked file — working tree vs index
    let unstaged = run_diff_best_effort(&repo_root, &["diff", "-U999999", "--", path]);
    if !unstaged.is_empty() {
        //         tracing::info!(
        //             "[git_diff] case=unstaged path={} len={}",
        //             path,
        //             unstaged.len()
        //         );
        return Ok(unstaged);
    }

    // 4. Untracked file — show entire file as added
    let full_path = std::path::Path::new(&repo_root).join(path);
    if full_path.exists() {
        let untracked =
            run_diff_best_effort(&repo_root, &["diff", "--no-index", "/dev/null", "--", path]);
        if !untracked.is_empty() {
            //             tracing::info!(
            //                 "[git_diff] case=untracked path={} len={}",
            //                 path,
            //                 untracked.len()
            //             );
            return Ok(untracked);
        }
    }

    //     tracing::warn!("[git_diff] case=empty path={}", path);
    Ok(Vec::new())
}

/// Run a git diff command that must succeed (committed / staged).
/// Returns the raw stdout bytes; the client decodes them with the
/// user-selected encoding.
fn run_diff_required(repo_root: &str, args: &[&str]) -> Result<Vec<u8>, SessionError> {
    let output = crate::session::git::git_command()
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SessionError::FileSystemError(format!(
            "git diff failed: {stderr}"
        )));
    }

    Ok(output.stdout)
}

/// Run a git diff command where failure is acceptable (best-effort).
/// Returns raw stdout bytes, or empty on any failure.
fn run_diff_best_effort(repo_root: &str, args: &[&str]) -> Vec<u8> {
    match crate::session::git::git_command()
        .args(args)
        .current_dir(repo_root)
        .output()
    {
        Ok(o) => o.stdout,
        Err(_) => Vec::new(),
    }
}

pub(super) fn git_check_repo(working_path: &str) -> Result<(), SessionError> {
    let output = crate::session::git::git_command()
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(working_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    if !output.status.success() {
        return Err(SessionError::FileSystemError(
            "Not a git repository".to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout != "true" {
        return Err(SessionError::FileSystemError(
            "Not a git repository".to_string(),
        ));
    }

    Ok(())
}

pub(super) fn git_repo_root(working_path: &str) -> Result<String, SessionError> {
    let output = crate::session::git::git_command()
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(working_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    if !output.status.success() {
        return Err(SessionError::FileSystemError(
            "Not a git repository".to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn git_read_commit_changed_files(
    working_path: &str,
    commit: &str,
) -> Result<Vec<GitChangedFile>, SessionError> {
    info!(
        "[git_changed_files] git diff-tree --no-commit-id --name-status -r -m --root {}",
        commit
    );

    let output = crate::session::git::git_command()
        .args([
            "diff-tree",
            "--no-commit-id",
            "--name-status",
            "-r",
            "-m",
            "--root",
            commit,
        ])
        .current_dir(working_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SessionError::FileSystemError(format!(
            "git diff-tree failed: {stderr}"
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut items = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        let status_char = parts.first().copied().unwrap_or("").trim();
        let file = parts.last().copied().unwrap_or("").trim().to_string();
        if file.is_empty() {
            continue;
        }
        let status = match status_char.chars().next() {
            Some('M') | Some('R') => GitFileStatus::Modified,
            Some('A') | Some('C') => GitFileStatus::Added,
            Some('D') => GitFileStatus::Deleted,
            _ => continue,
        };
        if items.iter().any(|item: &GitChangedFile| item.file == file) {
            continue;
        }
        items.push(GitChangedFile {
            file,
            status,
            staged: false,
            is_submodule: false,
            submodule_dirty: false,
        });
    }

    Ok(items)
}

fn git_read_working_tree_changed_files(
    working_path: &str,
) -> Result<Vec<GitChangedFile>, SessionError> {
    let mut items = Vec::new();
    let mut seen = HashMap::<(String, bool), usize>::new();
    let submodules = git_submodule_states(working_path)?;

    trace!("[git_changed_files] git status --porcelain=v1 -z -uall --ignore-submodules=none");

    let output = crate::session::git::git_command()
        .args([
            "status",
            "--porcelain=v1",
            "-z",
            "-uall",
            "--ignore-submodules=none",
        ])
        // Read-only refresh must not take `.git/index.lock` — the optional
        // lock stalls a concurrently running rebase/cherry-pick on Windows.
        .env("GIT_OPTIONAL_LOCKS", "0")
        .current_dir(working_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    if !output.status.success() {
        // Tolerate status failures (broken remote/submodule state, etc.) the
        // same way git_submodule_states does: log and report no changes
        // instead of failing the whole GitChangedFiles request.
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        warn!("[git_changed_files] git status failed: {stderr}");
        return Ok(Vec::new());
    }

    let mut entries = output.stdout.split(|byte| *byte == 0);

    while let Some(entry) = entries.next() {
        if entry.is_empty() || entry.len() < 4 {
            continue;
        }
        let index_status = entry[0] as char;
        let worktree_status = entry[1] as char;
        let file = String::from_utf8_lossy(&entry[3..]).to_string();
        if file.is_empty() {
            continue;
        }

        if index_status == '?' && worktree_status == '?' {
            push_changed_file(
                &mut items,
                &mut seen,
                file,
                GitFileStatus::Untracked,
                false,
                false,
                false,
            );
            continue;
        }

        if is_unmerged_porcelain_status(index_status, worktree_status) {
            let submodule_dirty = submodules.get(&file).copied().unwrap_or(false);
            let is_submodule = submodules.contains_key(&file);
            push_changed_file(
                &mut items,
                &mut seen,
                file,
                GitFileStatus::Conflict,
                false,
                is_submodule,
                submodule_dirty,
            );
            continue;
        }

        if matches!(index_status, 'R' | 'C') || matches!(worktree_status, 'R' | 'C') {
            let _old_path = entries.next();
        }

        if let Some(status) = git_status_from_porcelain_code(index_status) {
            let submodule_dirty = submodules.get(&file).copied().unwrap_or(false);
            let is_submodule = submodules.contains_key(&file);
            push_changed_file(
                &mut items,
                &mut seen,
                file.clone(),
                status,
                true,
                is_submodule,
                submodule_dirty,
            );
        }

        if let Some(status) = git_status_from_porcelain_code(worktree_status) {
            let submodule_dirty = submodules.get(&file).copied().unwrap_or(false);
            let is_submodule = submodules.contains_key(&file);
            push_changed_file(
                &mut items,
                &mut seen,
                file,
                status,
                false,
                is_submodule,
                submodule_dirty,
            );
        }
    }

    Ok(items)
}

fn push_changed_file(
    items: &mut Vec<GitChangedFile>,
    seen: &mut HashMap<(String, bool), usize>,
    file: String,
    status: GitFileStatus,
    staged: bool,
    is_submodule: bool,
    submodule_dirty: bool,
) {
    let key = (file.clone(), staged);
    if let Some(&idx) = seen.get(&key) {
        items[idx].status = status;
        items[idx].is_submodule |= is_submodule;
        items[idx].submodule_dirty |= submodule_dirty;
        return;
    }

    let idx = items.len();
    seen.insert(key, idx);
    items.push(GitChangedFile {
        file,
        status,
        staged,
        is_submodule,
        submodule_dirty,
    });
}

fn git_submodule_states(working_path: &str) -> Result<HashMap<String, bool>, SessionError> {
    let output = crate::session::git::git_command()
        .args([
            "status",
            "--porcelain=v2",
            "-z",
            "-uall",
            "--ignore-submodules=none",
        ])
        .env("GIT_OPTIONAL_LOCKS", "0")
        .current_dir(working_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    if !output.status.success() {
        return Ok(HashMap::new());
    }
    let mut result = HashMap::new();
    for entry in output.stdout.split(|byte| *byte == 0) {
        let text = String::from_utf8_lossy(entry);
        let fields = match entry.first().copied() {
            Some(b'1') => text.splitn(9, ' ').collect::<Vec<_>>(),
            Some(b'2') => text.splitn(10, ' ').collect::<Vec<_>>(),
            Some(b'u') => text.splitn(11, ' ').collect::<Vec<_>>(),
            _ => continue,
        };
        let Some(sub) = fields.get(2) else {
            continue;
        };
        if !sub.starts_with('S') {
            continue;
        }
        let Some(path) = fields.last() else { continue };
        let bytes = sub.as_bytes();
        let dirty = bytes.get(2) == Some(&b'M') || bytes.get(3) == Some(&b'U');
        result.insert((*path).to_string(), dirty);
    }
    Ok(result)
}

fn git_status_from_porcelain_code(code: char) -> Option<GitFileStatus> {
    match code {
        'M' | 'R' => Some(GitFileStatus::Modified),
        'A' | 'C' => Some(GitFileStatus::Added),
        'D' => Some(GitFileStatus::Deleted),
        _ => None,
    }
}

fn is_unmerged_porcelain_status(index_status: char, worktree_status: char) -> bool {
    matches!(
        (index_status, worktree_status),
        ('D', 'D') | ('A', 'U') | ('U', 'D') | ('U', 'A') | ('D', 'U') | ('A', 'A') | ('U', 'U')
    )
}

/// Resolve the repository root for a path operation: when the client pins
/// the root (the repo its file list came from), trust it after validating
/// it is a work tree — the session's current working path may have moved
/// into a nested repo since the list was generated, and re-resolving from
/// it would redirect the repo-relative pathspecs (git add then fails with
/// "pathspec ... did not match any files").
fn resolve_op_repo_root(
    working_path: &str,
    pinned_root: Option<&str>,
) -> Result<String, SessionError> {
    if let Some(root) = pinned_root {
        git_check_repo(root)?;
        return Ok(root.to_string());
    }
    git_check_repo(working_path)?;
    git_repo_root(working_path)
}

pub(super) fn git_do_stage(
    working_path: &str,
    paths: &[String],
    pinned_root: Option<&str>,
) -> Result<(), SessionError> {
    info!(
        "[git_stage] working_path={} pinned_root={:?} paths={:?}",
        working_path, pinned_root, paths
    );
    let repo_root = resolve_op_repo_root(working_path, pinned_root)?;
    run_git_with_pathspec_stdin(
        &repo_root,
        &["add", "--pathspec-from-file=-", "--pathspec-file-nul"],
        paths,
    )
}

pub(super) fn git_do_unstage(
    working_path: &str,
    paths: &[String],
    pinned_root: Option<&str>,
) -> Result<(), SessionError> {
    let repo_root = resolve_op_repo_root(working_path, pinned_root)?;
    run_git_with_pathspec_stdin(
        &repo_root,
        &["reset", "--pathspec-from-file=-", "--pathspec-file-nul"],
        paths,
    )
}

pub(super) fn git_do_revert_file(
    working_path: &str,
    path: &str,
    commit: &str,
) -> Result<(), SessionError> {
    git_check_repo(working_path)?;
    let repo_root = git_repo_root(working_path)?;
    let status = crate::session::git::git_command()
        .args(["restore", "--source", commit, "--", path])
        .current_dir(&repo_root)
        .status()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    if !status.success() {
        return Err(SessionError::FileSystemError(format!(
            "git restore failed for {} at {}",
            path, commit
        )));
    }
    Ok(())
}

pub(super) fn git_do_discard(
    working_path: &str,
    paths: &[String],
    pinned_root: Option<&str>,
) -> Result<(), SessionError> {
    let repo_root = resolve_op_repo_root(working_path, pinned_root)?;
    if paths.is_empty() {
        return Ok(());
    }
    warn!(
        repo_root = %repo_root,
        ?paths,
        "file operation: git discard"
    );

    let changed_files = git_read_working_tree_changed_files(&repo_root)?;
    let requested_paths: HashSet<&str> = paths.iter().map(String::as_str).collect();
    let untracked_paths: HashSet<String> = changed_files
        .into_iter()
        .filter(|item| {
            !item.staged
                && item.status == GitFileStatus::Untracked
                && requested_paths.contains(item.file.as_str())
        })
        .map(|item| item.file)
        .collect();
    let tracked_paths = paths
        .iter()
        .filter(|path| !untracked_paths.contains(path.as_str()))
        .cloned()
        .collect::<Vec<_>>();

    if !tracked_paths.is_empty() {
        run_git_with_pathspec_stdin(
            &repo_root,
            &["checkout", "--pathspec-from-file=-", "--pathspec-file-nul"],
            &tracked_paths,
        )?;
    }

    for path in untracked_paths {
        remove_untracked_path(&repo_root, &path)?;
    }

    Ok(())
}

fn remove_untracked_path(repo_root: &str, path: &str) -> Result<(), SessionError> {
    let rel_path = Path::new(path);
    if rel_path.is_absolute()
        || rel_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err(SessionError::FileSystemError(format!(
            "refusing to discard unsafe untracked path: {path}"
        )));
    }

    let full_path = Path::new(repo_root).join(rel_path);
    if !full_path.exists() {
        return Ok(());
    }

    let repo_root = std::fs::canonicalize(repo_root)
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    let canonical_path = std::fs::canonicalize(&full_path)
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    if !canonical_path.starts_with(&repo_root) {
        return Err(SessionError::FileSystemError(format!(
            "refusing to discard untracked path outside repository: {path}"
        )));
    }

    if canonical_path.is_dir() {
        std::fs::remove_dir_all(&canonical_path)
    } else {
        std::fs::remove_file(&canonical_path)
    }
    .map_err(|error| SessionError::FileSystemError(error.to_string()))
}

fn run_git_with_pathspec_stdin(
    working_path: &str,
    args: &[&str],
    paths: &[String],
) -> Result<(), SessionError> {
    let mut child = crate::session::git::git_command()
        .args(args)
        .current_dir(working_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| SessionError::FileSystemError("failed to open git stdin".to_string()))?;
        for path in paths {
            stdin
                .write_all(path.as_bytes())
                .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
            stdin
                .write_all(&[0])
                .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        info!(
            "[git] command failed in {}: {:?} stderr={} stdout={}",
            working_path, args, stderr, stdout
        );
        return Err(SessionError::FileSystemError(stderr));
    }
    Ok(())
}
pub(super) fn git_do_commit(working_path: &str, message: &str) -> Result<(), SessionError> {
    git_check_repo(working_path)?;
    let output = crate::session::git::git_command()
        .args(["commit", "-m", message])
        .current_dir(working_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SessionError::FileSystemError(stderr));
    }
    Ok(())
}

/// Commit while recovering from a detached HEAD. The expected hash prevents a
/// stale dialog from acting on a repository whose HEAD changed in the
/// meantime. Branch selection/creation and commit run in one server task, so
/// the client never has to chain write-operation responses.
pub(super) fn git_do_detached_commit(
    working_path: &str,
    message: &str,
    expected_head: &str,
    target_branch: Option<&str>,
    create_branch: bool,
) -> Result<(), SessionError> {
    git_check_repo(working_path)?;

    let symbolic = crate::session::git::git_command()
        .args(["symbolic-ref", "-q", "--short", "HEAD"])
        .current_dir(working_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    if symbolic.status.success() {
        let branch = String::from_utf8_lossy(&symbolic.stdout).trim().to_string();
        return Err(SessionError::FileSystemError(format!(
            "HEAD is no longer detached{}",
            if branch.is_empty() {
                String::new()
            } else {
                format!(" (currently on {branch})")
            }
        )));
    }

    let head_output = crate::session::git::git_command()
        .args(["rev-parse", "HEAD"])
        .current_dir(working_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    if !head_output.status.success() {
        return Err(SessionError::FileSystemError(
            String::from_utf8_lossy(&head_output.stderr)
                .trim()
                .to_string(),
        ));
    }
    let actual_head = String::from_utf8_lossy(&head_output.stdout)
        .trim()
        .to_string();
    let expected_head = expected_head.trim();
    if expected_head.is_empty() || !actual_head.starts_with(expected_head) {
        return Err(SessionError::FileSystemError(format!(
            "detached HEAD changed (expected {expected_head}, found {actual_head})"
        )));
    }

    let target_branch = target_branch.map(str::trim).filter(|name| !name.is_empty());
    if create_branch && target_branch.is_none() {
        return Err(SessionError::FileSystemError(
            "target_branch is required when create_branch is set".to_string(),
        ));
    }

    if let Some(branch) = target_branch {
        if create_branch {
            // `-b`, not `-B`: never reset an existing local branch as a side
            // effect of detached-HEAD recovery. Creating a branch at the
            // current commit never touches the working tree, so staged
            // changes cannot conflict here.
            let output = crate::session::git::git_command()
                .args(["checkout", "-b", branch])
                .current_dir(working_path)
                .output()
                .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
            if !output.status.success() {
                return Err(SessionError::FileSystemError(
                    String::from_utf8_lossy(&output.stderr).trim().to_string(),
                ));
            }
            // If commit fails (for example, a hook rejects it), keep the new
            // branch and staged changes. This is safer than a destructive
            // rollback and lets the user fix the cause and retry.
            return git_do_commit(working_path, message);
        }

        // Move an existing branch onto the new commit. Commit FIRST on the
        // detached HEAD, then advance the branch ref and attach HEAD via
        // symbolic-ref. Checking the branch out first would move the working
        // tree to the branch's (typically older) commit and abort with "Your
        // local changes would be overwritten by checkout" whenever the staged
        // changes intersect the branch diff — and even when it succeeds, the
        // commit would land on the branch's base instead of the detached
        // commit the user was looking at.
        //
        // Refuse branches that have commits of their own not contained in
        // HEAD: advancing the ref would orphan those commits.
        let ancestor = crate::session::git::git_command()
            .args(["merge-base", "--is-ancestor", branch, "HEAD"])
            .current_dir(working_path)
            .output()
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        if !ancestor.status.success() {
            return Err(SessionError::FileSystemError(format!(
                "branch {branch} is missing or has commits not contained in the detached HEAD; refusing to move it"
            )));
        }

        // If the commit fails (for example, a hook rejects it), nothing has
        // changed yet: HEAD is still detached, the branch is unmoved and the
        // staged changes are intact, so the user can fix the cause and retry.
        git_do_commit(working_path, message)?;

        // Advance the branch ref to the new commit (a fast-forward: the
        // ancestry check above guarantees no commits are orphaned) and attach
        // HEAD without touching the working tree or index — no checkout means
        // no content conflict is possible.
        let moved = crate::session::git::git_command()
            .args(["branch", "-f", branch, "HEAD"])
            .current_dir(working_path)
            .output()
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        if !moved.status.success() {
            return Err(SessionError::FileSystemError(
                String::from_utf8_lossy(&moved.stderr).trim().to_string(),
            ));
        }
        let attached = crate::session::git::git_command()
            .args(["symbolic-ref", "HEAD", &format!("refs/heads/{branch}")])
            .current_dir(working_path)
            .output()
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        if !attached.status.success() {
            return Err(SessionError::FileSystemError(
                String::from_utf8_lossy(&attached.stderr).trim().to_string(),
            ));
        }
        return Ok(());
    }

    git_do_commit(working_path, message)
}

pub(super) fn git_do_fetch(working_path: &str) -> Result<(), SessionError> {
    git_check_repo(working_path)?;
    let output = crate::session::git::git_command()
        .args(["fetch", "--all"])
        .current_dir(working_path)
        .output_with_timeout(NETWORK_OP_TIMEOUT)
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SessionError::FileSystemError(stderr));
    }
    Ok(())
}

pub(super) fn git_do_pull(working_path: &str) -> Result<(), SessionError> {
    git_check_repo(working_path)?;
    let output = crate::session::git::git_command()
        .args(["pull"])
        .current_dir(working_path)
        .output_with_timeout(NETWORK_OP_TIMEOUT)
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SessionError::FileSystemError(stderr));
    }
    Ok(())
}

pub(super) fn git_do_push(
    working_path: &str,
    push_all: bool,
    force: bool,
    force_with_lease: bool,
    set_upstream: bool,
    tags: bool,
    dry_run: bool,
    remote: &str,
    branch: &str,
) -> Result<(), SessionError> {
    git_check_repo(working_path)?;

    // Build the arg vector in the prototype's flag order:
    //   push [--dry-run] [-u] [--tags] [--force | --force-with-lease] [--all]
    //        [<remote> <branch>]
    // `--force` takes precedence over `--force-with-lease` when both are set.
    // No refspec is appended when `--all` is set (it covers every branch).
    let mut args: Vec<String> = vec!["push".to_string()];
    if dry_run {
        args.push("--dry-run".to_string());
    }
    if set_upstream {
        args.push("-u".to_string());
    }
    if tags {
        args.push("--tags".to_string());
    }
    if force {
        args.push("--force".to_string());
    } else if force_with_lease {
        args.push("--force-with-lease".to_string());
    }
    if push_all {
        args.push("--all".to_string());
    }
    if !push_all && !remote.is_empty() {
        args.push(remote.to_string());
        if !branch.is_empty() {
            args.push(branch.to_string());
        }
    }

    let cmd_str = args.join(" ");
    info!("[git_push] git {} (cwd={})", cmd_str, working_path);

    let output = crate::session::git::git_command()
        .args(&args)
        .current_dir(working_path)
        .output_with_timeout(NETWORK_OP_TIMEOUT)
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SessionError::FileSystemError(stderr));
    }
    Ok(())
}

/// Get the commit hash of the remote tracking branch for the current branch.
/// Returns `None` if no upstream is set (e.g. detached HEAD or no remote).
pub(super) fn git_read_remote_tracking_hash(
    working_path: &str,
    current_branch: &Option<String>,
) -> Result<Option<String>, SessionError> {
    git_check_repo(working_path)?;

    // Primary: @{upstream} — respects the user's configured tracking.
    let output = crate::session::git::git_command()
        .args(["rev-parse", "@{upstream}"])
        .current_dir(working_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    if output.status.success() {
        let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !hash.is_empty() {
            tracing::trace!("[git_remote_hash] @{{upstream}} = {}", hash);
            return Ok(Some(hash));
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        tracing::trace!(
            "[git_remote_hash] @{{upstream}} failed: {}",
            if stderr.is_empty() {
                format!("exit code {}", output.status)
            } else {
                stderr
            }
        );
    }

    // Fallback 1: origin/<current_branch> — works when @{upstream} is not
    // configured (worktree branches, detached HEAD, etc.).
    if let Some(branch) = current_branch {
        let name = format!("origin/{}", branch);
        tracing::trace!("[git_remote_hash] trying fallback: git rev-parse {}", name);
        let fb_output = crate::session::git::git_command()
            .args(["rev-parse", &name])
            .current_dir(working_path)
            .output()
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        if fb_output.status.success() {
            let hash = String::from_utf8_lossy(&fb_output.stdout)
                .trim()
                .to_string();
            if !hash.is_empty() {
                tracing::trace!("[git_remote_hash] fallback {} = {}", name, hash);
                return Ok(Some(hash));
            }
        } else {
            let stderr = String::from_utf8_lossy(&fb_output.stderr)
                .trim()
                .to_string();
            tracing::trace!(
                "[git_remote_hash] fallback {} failed: {}",
                name,
                if stderr.is_empty() {
                    format!("exit code {}", fb_output.status)
                } else {
                    stderr
                }
            );
        }
    }

    // Fallback 2: merge-base with the remote's default HEAD branch.
    // For branches that have no upstream and no remote counterpart (e.g.
    // worktree branches), use the divergence point from origin/HEAD as a
    // best-effort remote-tracking hash.  This dynamically resolves the
    // remote's default branch (main/master/...) instead of hardcoding it.
    let default_branch = crate::session::git::git_command()
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .current_dir(working_path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let ref_str = String::from_utf8_lossy(&o.stdout).trim().to_string();
            // Strips "refs/remotes/origin/" prefix to get just "main"/"master"
            ref_str
                .strip_prefix("refs/remotes/origin/")
                .map(|s| s.to_string())
        });
    if let Some(branch) = default_branch {
        let remote_head = format!("origin/{}", branch);
        tracing::trace!("[git_remote_hash] trying merge-base with {}", remote_head);
        let mb_output = crate::session::git::git_command()
            .args(["merge-base", "HEAD", &remote_head])
            .current_dir(working_path)
            .output()
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        if mb_output.status.success() {
            let hash = String::from_utf8_lossy(&mb_output.stdout)
                .trim()
                .to_string();
            if !hash.is_empty() {
                tracing::trace!(
                    "[git_remote_hash] merge-base HEAD {} = {}",
                    remote_head,
                    hash
                );
                return Ok(Some(hash));
            }
        }
    }

    tracing::trace!("[git_remote_hash] no hash found");
    Ok(None)
}

pub(super) fn git_read_branches(
    working_path: &str,
) -> Result<(Option<String>, Option<String>, Vec<GitBranchInfo>), SessionError> {
    git_check_repo(working_path)?;

    // Get current branch name
    let current_output = crate::session::git::git_command()
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(working_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    let current = if current_output.status.success() {
        let name = String::from_utf8_lossy(&current_output.stdout)
            .trim()
            .to_string();
        if name.is_empty() || name == "HEAD" {
            None
        } else {
            Some(name)
        }
    } else {
        None
    };

    let detached_head = if current.is_none() {
        let output = crate::session::git::git_command()
            .args(["rev-parse", "HEAD"])
            .current_dir(working_path)
            .output()
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        if output.status.success() {
            let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            (!hash.is_empty()).then_some(hash)
        } else {
            None
        }
    } else {
        None
    };

    // Get all branches
    let output = crate::session::git::git_command()
        .args(["branch", "-a", "--no-color"])
        .current_dir(working_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SessionError::FileSystemError(format!(
            "git branch failed: {stderr}"
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut branches = Vec::new();

    // Map of "refs/heads/<branch>" -> worktree path, so each local branch can
    // report which linked worktree (if any) it is checked out in.
    let worktree_paths = parse_worktree_porcelain(working_path).unwrap_or_default();

    for line in stdout.lines() {
        let line = line.trim_start();
        if line.is_empty() {
            continue;
        }
        // git branch prefixes: "* " = current branch, "+ " = checked out in a
        // (linked) worktree. Strip whichever is present so the name is clean.
        let (is_current_marker, name) = if let Some(rest) = line.strip_prefix("* ") {
            (true, rest)
        } else if let Some(rest) = line.strip_prefix("+ ") {
            (false, rest)
        } else {
            (false, line)
        };
        let name = name.trim().to_string();
        if name.is_empty() {
            continue;
        }
        // Skip "HEAD detached" entries
        if name.starts_with('(') {
            continue;
        }
        let is_remote = name.starts_with("remotes/");
        let is_current = if is_remote {
            false
        } else {
            is_current_marker || current.as_ref().is_some_and(|c| c == &name)
        };
        let worktree_path = if is_remote {
            None
        } else {
            worktree_paths.get(&format!("refs/heads/{name}")).cloned()
        };
        branches.push(GitBranchInfo {
            name,
            is_current,
            is_remote,
            worktree_path,
        });
    }

    Ok((current, detached_head, branches))
}

pub(super) fn git_read_tags(working_path: &str) -> Result<Vec<String>, SessionError> {
    git_check_repo(working_path)?;

    let output = crate::session::git::git_command()
        .args(["tag", "--list"])
        .current_dir(working_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SessionError::FileSystemError(format!(
            "git tag failed: {stderr}"
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let tags = stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|name| !name.is_empty())
        .collect();
    Ok(tags)
}

/// Whether `ref_` names a local or remote-tracking branch — the only start
/// points `git checkout --track` accepts. Probes fully-qualified refs (never
/// ambiguous, unlike a DWIM lookup) for both client ref spellings: `origin/x`
/// and the branch list's `remotes/origin/x` form, whose `remotes/` prefix is
/// not part of the ref path.
fn ref_is_trackable(working_path: &str, ref_: &str) -> bool {
    let mut candidates = vec![format!("refs/heads/{ref_}"), format!("refs/remotes/{ref_}")];
    if let Some(without_prefix) = ref_.strip_prefix("remotes/") {
        candidates.push(format!("refs/remotes/{without_prefix}"));
    }
    candidates.iter().any(|full_ref| {
        crate::session::git::git_command()
            .args(["rev-parse", "--verify", "--quiet", full_ref])
            .current_dir(working_path)
            .output()
            .is_ok_and(|output| output.status.success())
    })
}

/// Check out a ref with the full set of TortoiseGit-style options.
///
/// Assembles and runs:
/// ```text
/// git checkout [-f] [-B <branch_name>] [--track] [--merge] <ref_>
/// ```
///
/// The caller (client CheckoutDialog) decides which flags to set; every option
/// maps 1:1 to a git argument so the assembled command matches exactly what
/// the user picked. The one exception is `--track`: git rejects it outright
/// when the start point is not a branch ("cannot set up tracking information;
/// starting point 'X' is not a branch"), so for tags and raw hashes the flag
/// is dropped rather than failing the whole checkout.
///
/// No `--` disambiguator is emitted: `git checkout` resolves a trailing ref
/// argument as a branch/tree-ish before falling back to pathspecs, so both
/// local branch names (`main`) and remote refs (`remotes/origin/feature/x`)
/// work. Adding `--` would force pathspec interpretation and break plain
/// local-branch checkouts.
pub(super) fn git_do_checkout(
    working_path: &str,
    ref_: &str,
    create_branch: bool,
    branch_name: &str,
    track: bool,
    force: bool,
    merge: bool,
) -> Result<(), SessionError> {
    git_check_repo(working_path)?;

    let mut args: Vec<String> = vec!["checkout".to_string()];
    if force {
        args.push("-f".to_string());
    }
    if create_branch {
        if branch_name.trim().is_empty() {
            return Err(SessionError::FileSystemError(
                "branch_name is required when create_branch is set".to_string(),
            ));
        }
        args.push("-B".to_string());
        args.push(branch_name.to_string());
    }
    if track {
        if ref_is_trackable(working_path, ref_) {
            args.push("--track".to_string());
        } else {
            info!("[git_checkout] dropping --track: '{ref_}' is not a branch (cwd={working_path})");
        }
    }
    if merge {
        args.push("--merge".to_string());
    }
    args.push(ref_.to_string());

    let cmd_str = args.join(" ");
    info!("[git_checkout] git {} (cwd={})", cmd_str, working_path);

    let output = crate::session::git::git_command()
        .args(&args)
        .current_dir(working_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SessionError::FileSystemError(stderr));
    }
    Ok(())
}

pub(super) fn git_do_create_branch(
    working_path: &str,
    source_ref: &str,
    branch: &str,
) -> Result<(), SessionError> {
    git_check_repo(working_path)?;

    let output = crate::session::git::git_command()
        .args(["branch", branch, source_ref])
        .current_dir(working_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SessionError::FileSystemError(stderr));
    }
    Ok(())
}

/// Create a git worktree for the given branch in `<parent>/<repo_name>-worktrees/<name>`.
/// Returns the absolute path of the created worktree directory.
pub(super) fn git_create_worktree(
    working_path: &str,
    branch: &str,
    name: &str,
) -> Result<String, SessionError> {
    git_check_repo(working_path)?;

    let working = Path::new(working_path);
    let repo_root = git_repo_root(working_path)?;
    let repo_root_path = Path::new(&repo_root);

    let parent = repo_root_path
        .parent()
        .ok_or_else(|| SessionError::FileSystemError("Cannot determine repo parent dir".into()))?;
    let repo_name = repo_root_path
        .file_name()
        .ok_or_else(|| SessionError::FileSystemError("Cannot determine repo folder name".into()))?
        .to_string_lossy();

    let worktree_base = parent.join(format!("{}-worktrees", repo_name));
    std::fs::create_dir_all(&worktree_base).map_err(|e| {
        SessionError::FileSystemError(format!("Failed to create worktree base dir: {e}"))
    })?;

    let worktree_path = worktree_base.join(name);
    let worktree_path_str = worktree_path.to_string_lossy().to_string();

    // We intentionally never use `git worktree add -b <name>` because the
    // bare branch name can trigger "refname 'X' is ambiguous" in repos
    // where the same shorthand resolves via git's DWIM ref lookup (this
    // happens even with a single refs/heads/<name> entry, and is
    // nondeterministic across repos/versions). Instead:
    //
    //   1. Create the branch separately (if it doesn't exist yet).
    //   2. Add the worktree in detached-HEAD mode using the fully-qualified
    //      `refs/heads/<name>` ref (never ambiguous).
    //   3. Point the worktree's HEAD at the branch with `symbolic-ref`.
    let head_ref = format!("refs/heads/{name}");

    let branch_exists = crate::session::git::git_command()
        .args(["rev-parse", "--verify", "--quiet", &head_ref])
        .current_dir(working)
        .output()
        .is_ok_and(|o| o.status.success());

    if !branch_exists {
        info!(
            "[git_worktree] git branch {name} {branch} (cwd={})",
            working.display()
        );
        let output = crate::session::git::git_command()
            .args(["branch", name, branch])
            .current_dir(working)
            .output()
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(SessionError::FileSystemError(stderr));
        }
    }

    // Create the worktree pointing at the branch's commit (detached).
    info!(
        "[git_worktree] git worktree add --detach {worktree_path_str} {head_ref} (cwd={})",
        working.display()
    );
    let output = crate::session::git::git_command()
        .args(["worktree", "add", "--detach", &worktree_path_str, &head_ref])
        .current_dir(working)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SessionError::FileSystemError(stderr));
    }

    // Point the worktree's HEAD at the branch so it tracks it.
    info!("[git_worktree] git symbolic-ref HEAD {head_ref} (cwd={worktree_path_str})");
    let output = crate::session::git::git_command()
        .args(["symbolic-ref", "HEAD", &head_ref])
        .current_dir(&worktree_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        // Clean up the detached worktree so the next attempt starts fresh.
        let _ = crate::session::git::git_command()
            .args(["worktree", "remove", "--force", &worktree_path_str])
            .current_dir(working)
            .output();
        return Err(SessionError::FileSystemError(stderr));
    }

    let canonical = worktree_path.canonicalize().map_err(|e| {
        SessionError::FileSystemError(format!("Failed to canonicalize worktree path: {e}"))
    })?;
    let path_str = canonical.to_string_lossy().to_string();
    // On Windows, canonicalize returns UNC prefix \\?\ — strip it for cwd use.
    let clean = path_str.strip_prefix(r"\\?\").unwrap_or(&path_str);
    Ok(clean.to_string())
}

/// Delete a git worktree by its path.
pub(super) fn git_delete_worktree(worktree_path: &str) -> Result<(), SessionError> {
    let output = crate::session::git::git_command()
        .args(["worktree", "remove", worktree_path])
        .current_dir(worktree_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SessionError::FileSystemError(stderr));
    }

    Ok(())
}

/// Delete a branch. If `is_worktree` is true, first remove the linked worktree
/// (located by looking up which worktree has the branch checked out), then
/// delete the branch itself.
pub(super) fn git_do_delete_branch(
    working_path: &str,
    branch: &str,
    is_worktree: bool,
) -> Result<(), SessionError> {
    git_check_repo(working_path)?;

    if is_worktree {
        // Find the worktree path that has `branch` checked out.
        let wt_path = find_worktree_path_for_branch(working_path, branch)?;
        let wt_path = wt_path.ok_or_else(|| {
            SessionError::FileSystemError(format!("No linked worktree found for branch '{branch}'"))
        })?;

        // Remove the worktree directory (-f forces removal even with
        // untracked/modified files, matching the user-confirmed intent).
        let remove_output = crate::session::git::git_command()
            .args(["worktree", "remove", "-f", &wt_path])
            .current_dir(working_path)
            .output()
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        if !remove_output.status.success() {
            let stderr = String::from_utf8_lossy(&remove_output.stderr)
                .trim()
                .to_string();
            return Err(SessionError::FileSystemError(format!(
                "git worktree remove failed: {stderr}"
            )));
        }
    }

    // Delete the branch (-d refuses to delete unmerged branches; the user has
    // already acknowledged data loss via the confirm dialog).
    let delete_output = crate::session::git::git_command()
        .args(["branch", "-D", branch])
        .current_dir(working_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    if !delete_output.status.success() {
        let stderr = String::from_utf8_lossy(&delete_output.stderr)
            .trim()
            .to_string();
        return Err(SessionError::FileSystemError(format!(
            "git branch -d failed: {stderr}"
        )));
    }

    Ok(())
}

/// Merge `branch` into the currently checked-out branch. `--no-ff` keeps a
/// merge commit even when a fast-forward is possible, matching the explicit
/// "merge" intent from the UI. rerere is force-enabled for this command so
/// repeated merges of long-lived branches replay previously recorded conflict
/// resolutions instead of surfacing the same conflicts again.
pub(super) fn git_do_merge(working_path: &str, branch: &str) -> Result<(), SessionError> {
    git_check_repo(working_path)?;

    let output = crate::session::git::git_command()
        .args(["-c", "rerere.enabled=true", "merge", "--no-ff", branch])
        .current_dir(working_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SessionError::FileSystemError(stderr));
    }

    Ok(())
}

/// Compare two filesystem paths case- and separator-insensitively (git on
/// Windows reports paths with forward slashes; canonicalize returns a UNC
/// `\\?\`-prefixed backslash path). Strips the `\\?\` prefix and normalizes
/// separators before comparing.
fn same_path(a: &str, b: &str) -> bool {
    let norm = |s: &str| {
        s.strip_prefix(r"\\?\")
            .unwrap_or(s)
            .replace('\\', "/")
            .to_lowercase()
    };
    norm(a) == norm(b)
}

/// Parse `git worktree list --porcelain` into a map of
/// `"refs/heads/<branch>"` -> worktree path, **excluding the primary (main)
/// worktree**. The primary worktree is the repo's main checkout — its branch
/// should not be badged as a "worktree" since it lives in the repo root, not a
/// linked worktree. Returns an empty map on failure.
fn parse_worktree_porcelain(
    working_path: &str,
) -> Result<std::collections::HashMap<String, String>, SessionError> {
    let output = crate::session::git::git_command()
        .args(["worktree", "list", "--porcelain"])
        .current_dir(working_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SessionError::FileSystemError(format!(
            "git worktree list failed: {stderr}"
        )));
    }

    // Resolve the primary worktree path so it can be excluded. The common git
    // dir (e.g. ".../vibe-terminal/.git") lives inside the primary worktree, so
    // its parent directory is the primary worktree itself. Failures here just
    // mean we can't distinguish primary from linked, so we bail out (empty map
    // → no worktree badges), which is safer than badging the main branch.
    let primary_path = crate::session::git::git_command()
        .args(["rev-parse", "--git-common-dir"])
        .current_dir(working_path)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let dir = String::from_utf8_lossy(&o.stdout).trim().to_string();
            // Resolve to absolute (rev-parse may return a relative path).
            let abs = std::path::Path::new(working_path)
                .join(&dir)
                .canonicalize()
                .ok()?;
            // ".git" → its parent is the worktree root.
            abs.parent().map(|p| p.to_string_lossy().into_owned())
        });

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut map = std::collections::HashMap::new();
    let mut current_path: Option<String> = None;
    for line in stdout.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            current_path = None;
            continue;
        }
        if let Some(rest) = line.strip_prefix("worktree ") {
            current_path = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("branch ") {
            let branch_ref = rest.trim();
            if let Some(path) = &current_path {
                // Skip the primary worktree — its branch is the main checkout,
                // not a linked worktree.
                let is_primary = primary_path
                    .as_deref()
                    .map(|p| same_path(p, path))
                    .unwrap_or(false);
                if !is_primary {
                    map.insert(branch_ref.to_string(), path.clone());
                }
            }
        }
    }
    Ok(map)
}

/// Parse `git worktree list --porcelain` to find the worktree path whose
/// checked-out branch matches `branch`. Returns Ok(None) if no worktree matches.
fn find_worktree_path_for_branch(
    working_path: &str,
    branch: &str,
) -> Result<Option<String>, SessionError> {
    let map = parse_worktree_porcelain(working_path)?;
    Ok(map.get(&format!("refs/heads/{branch}")).cloned())
}

#[cfg(test)]
mod tests {
    use super::{
        git_do_checkout, git_do_detached_commit, git_do_discard, git_do_stage, git_do_stash_apply,
        git_do_stash_drop, git_do_stash_pop, git_do_stash_push, git_read_commit_changed_files,
        git_read_stash_list, git_read_submodule_list, git_read_working_tree_changed_files,
        push_changed_file,
    };
    use crate::session::git::GitFileStatus;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};
    use vterm_common::GitChangedFile;

    struct TestRepo {
        path: PathBuf,
    }

    impl TestRepo {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("vterm-git-discard-{unique}"));
            std::fs::create_dir_all(&path).unwrap();
            run_git(&path, &["init"]);
            run_git(&path, &["config", "user.email", "test@example.com"]);
            run_git(&path, &["config", "user.name", "Test User"]);
            Self { path }
        }
    }

    impl Drop for TestRepo {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    fn run_git(repo: &Path, args: &[&str]) {
        let output = crate::session::git::git_command()
            .args(args)
            .current_dir(repo)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn git_output(repo: &Path, args: &[&str]) -> String {
        let output = crate::session::git::git_command()
            .args(args)
            .current_dir(repo)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn commit_file(repo: &Path, path: &str, content: &str) {
        std::fs::write(repo.join(path), content).unwrap();
        run_git(repo, &["add", path]);
        run_git(repo, &["commit", "-m", "initial"]);
    }

    /// `diff-tree --name-status` quotes non-ASCII paths as `"\351\241\271..."`
    /// (octal-escaped UTF-8) unless `core.quotepath` is off — which
    /// `git_command()` injects — and the client would display that escape
    /// soup verbatim. Guard the whole path pipeline with a real CJK filename.
    #[test]
    fn non_ascii_paths_are_not_octal_escaped() {
        let repo = TestRepo::new();
        std::fs::create_dir_all(repo.path.join("Doc")).unwrap();
        std::fs::write(repo.path.join("Doc").join("项目说明.md"), "hello\n").unwrap();
        run_git(&repo.path, &["add", "."]);
        run_git(&repo.path, &["commit", "-m", "add chinese doc"]);

        let working_path = repo.path.to_str().unwrap();
        let files = git_read_commit_changed_files(working_path, "HEAD").unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file, "Doc/项目说明.md");
        assert_eq!(files[0].status, GitFileStatus::Added);
    }

    #[test]
    fn detached_commit_checks_out_existing_branch_and_commits_in_one_operation() {
        let repo = TestRepo::new();
        commit_file(&repo.path, "tracked.txt", "base\n");
        let branch = git_output(&repo.path, &["symbolic-ref", "--short", "HEAD"]);
        let head = git_output(&repo.path, &["rev-parse", "--short", "HEAD"]);
        run_git(&repo.path, &["checkout", "--detach", "HEAD"]);
        std::fs::write(repo.path.join("tracked.txt"), "changed\n").unwrap();
        run_git(&repo.path, &["add", "tracked.txt"]);

        git_do_detached_commit(
            repo.path.to_str().unwrap(),
            "detached recovery",
            &head,
            Some(&branch),
            false,
        )
        .unwrap();

        assert_eq!(
            git_output(&repo.path, &["symbolic-ref", "--short", "HEAD"]),
            branch
        );
        assert_eq!(
            git_output(&repo.path, &["log", "-1", "--format=%s"]),
            "detached recovery"
        );
    }

    #[test]
    fn detached_commit_can_create_branch_or_explicitly_skip() {
        let create_repo = TestRepo::new();
        commit_file(&create_repo.path, "tracked.txt", "base\n");
        let head = git_output(&create_repo.path, &["rev-parse", "--short", "HEAD"]);
        run_git(&create_repo.path, &["checkout", "--detach", "HEAD"]);
        std::fs::write(create_repo.path.join("tracked.txt"), "branch\n").unwrap();
        run_git(&create_repo.path, &["add", "tracked.txt"]);
        git_do_detached_commit(
            create_repo.path.to_str().unwrap(),
            "new branch commit",
            &head,
            Some("detached-work"),
            true,
        )
        .unwrap();
        assert_eq!(
            git_output(&create_repo.path, &["symbolic-ref", "--short", "HEAD"]),
            "detached-work"
        );

        let skip_repo = TestRepo::new();
        commit_file(&skip_repo.path, "tracked.txt", "base\n");
        let head = git_output(&skip_repo.path, &["rev-parse", "--short", "HEAD"]);
        run_git(&skip_repo.path, &["checkout", "--detach", "HEAD"]);
        std::fs::write(skip_repo.path.join("tracked.txt"), "skip\n").unwrap();
        run_git(&skip_repo.path, &["add", "tracked.txt"]);
        git_do_detached_commit(
            skip_repo.path.to_str().unwrap(),
            "detached commit",
            &head,
            None,
            false,
        )
        .unwrap();
        let symbolic = crate::session::git::git_command()
            .args(["symbolic-ref", "-q", "HEAD"])
            .current_dir(&skip_repo.path)
            .output()
            .unwrap();
        assert!(!symbolic.status.success(), "Skip must leave HEAD detached");
        assert_eq!(
            git_output(&skip_repo.path, &["log", "-1", "--format=%s"]),
            "detached commit"
        );
    }

    #[test]
    fn detached_commit_moves_lagging_branch_forward_without_checkout_conflicts() {
        let repo = TestRepo::new();
        commit_file(&repo.path, "tracked.txt", "base\n");
        commit_file(&repo.path, "tracked.txt", "one\n");
        commit_file(&repo.path, "tracked.txt", "two\n");
        let branch = git_output(&repo.path, &["symbolic-ref", "--short", "HEAD"]);
        let base = git_output(&repo.path, &["rev-parse", "HEAD~2"]);
        let detached_head = git_output(&repo.path, &["rev-parse", "HEAD"]);
        run_git(&repo.path, &["checkout", "--detach", "HEAD"]);
        // The branch lags the detached HEAD by two commits — the checkout
        // done by the old implementation would abort with "local changes
        // would be overwritten" once a conflicting change is staged.
        run_git(&repo.path, &["branch", "-f", &branch, &base]);
        std::fs::write(repo.path.join("tracked.txt"), "changed\n").unwrap();
        run_git(&repo.path, &["add", "tracked.txt"]);

        git_do_detached_commit(
            repo.path.to_str().unwrap(),
            "detached recovery",
            &detached_head,
            Some(&branch),
            false,
        )
        .unwrap();

        assert_eq!(
            git_output(&repo.path, &["symbolic-ref", "--short", "HEAD"]),
            branch
        );
        assert_eq!(
            git_output(&repo.path, &["log", "-1", "--format=%s"]),
            "detached recovery"
        );
        // The commit lands on top of the detached HEAD, and the branch now
        // points there (a fast-forward move: the two intermediate commits
        // stay in its history).
        assert_eq!(
            git_output(&repo.path, &["rev-parse", "HEAD~1"]),
            detached_head
        );
        assert_eq!(
            git_output(&repo.path, &["rev-parse", &branch]),
            git_output(&repo.path, &["rev-parse", "HEAD"])
        );
    }

    #[test]
    fn detached_commit_refuses_to_move_diverged_branch() {
        let repo = TestRepo::new();
        commit_file(&repo.path, "tracked.txt", "base\n");
        let base = git_output(&repo.path, &["rev-parse", "HEAD"]);
        // A detached commit the branch does not contain…
        run_git(&repo.path, &["checkout", "--detach", "HEAD"]);
        std::fs::write(repo.path.join("detached.txt"), "detached\n").unwrap();
        run_git(&repo.path, &["add", "detached.txt"]);
        run_git(&repo.path, &["commit", "-m", "detached work"]);
        let detached_head = git_output(&repo.path, &["rev-parse", "HEAD"]);
        // …and a branch with a commit of its own (not contained in HEAD).
        run_git(&repo.path, &["checkout", "-b", "feature", &base]);
        std::fs::write(repo.path.join("feature.txt"), "feature\n").unwrap();
        run_git(&repo.path, &["add", "feature.txt"]);
        run_git(&repo.path, &["commit", "-m", "feature work"]);
        let feature_head = git_output(&repo.path, &["rev-parse", "feature"]);
        run_git(&repo.path, &["checkout", "--detach", &detached_head]);
        std::fs::write(repo.path.join("tracked.txt"), "pending\n").unwrap();
        run_git(&repo.path, &["add", "tracked.txt"]);

        let result = git_do_detached_commit(
            repo.path.to_str().unwrap(),
            "should not happen",
            &detached_head,
            Some("feature"),
            false,
        );

        assert!(result.is_err(), "diverged branch must be refused");
        // Nothing changed: HEAD is still detached at the same commit, the
        // branch is unmoved and the staged change is intact for a retry.
        assert_eq!(
            git_output(&repo.path, &["rev-parse", "HEAD"]),
            detached_head
        );
        let symbolic = crate::session::git::git_command()
            .args(["symbolic-ref", "-q", "HEAD"])
            .current_dir(&repo.path)
            .output()
            .unwrap();
        assert!(!symbolic.status.success(), "HEAD must stay detached");
        assert_eq!(
            git_output(&repo.path, &["rev-parse", "feature"]),
            feature_head
        );
        assert_eq!(
            git_output(&repo.path, &["diff", "--cached", "--name-only"]),
            "tracked.txt"
        );
    }

    #[test]
    fn git_discard_removes_untracked_file() {
        let repo = TestRepo::new();
        let untracked = repo.path.join("untracked.txt");
        std::fs::write(&untracked, "scratch").unwrap();

        git_do_discard(
            repo.path.to_str().unwrap(),
            &[String::from("untracked.txt")],
            None,
        )
        .unwrap();

        assert!(!untracked.exists());
    }

    #[test]
    fn git_discard_handles_tracked_and_untracked_paths_together() {
        let repo = TestRepo::new();
        commit_file(&repo.path, "tracked.txt", "committed");
        std::fs::write(repo.path.join("tracked.txt"), "modified").unwrap();
        std::fs::write(repo.path.join("untracked.txt"), "scratch").unwrap();

        git_do_discard(
            repo.path.to_str().unwrap(),
            &[String::from("tracked.txt"), String::from("untracked.txt")],
            None,
        )
        .unwrap();

        assert_eq!(
            std::fs::read_to_string(repo.path.join("tracked.txt")).unwrap(),
            "committed"
        );
        assert!(!repo.path.join("untracked.txt").exists());
    }

    /// Regression: the session working_path may sit inside a nested
    /// independent git repo (e.g. `outer/xray/`). Without a pinned repo_root
    /// the server resolves the nested repo as the toplevel and outer-relative
    /// paths fail with "pathspec did not match any files".
    #[test]
    fn git_stage_uses_pinned_root_when_working_path_is_nested_repo() {
        let repo = TestRepo::new();
        std::fs::create_dir_all(repo.path.join("backend")).unwrap();
        commit_file(&repo.path, "backend/build.rs", "committed");
        let nested = repo.path.join("xray");
        std::fs::create_dir_all(&nested).unwrap();
        run_git(&nested, &["init"]);
        std::fs::write(repo.path.join("backend/build.rs"), "modified").unwrap();

        // Without a pinned root the nested repo wins and the op fails.
        assert!(git_do_stage(
            nested.to_str().unwrap(),
            &[String::from("backend/build.rs")],
            None,
        )
        .is_err());

        git_do_stage(
            nested.to_str().unwrap(),
            &[String::from("backend/build.rs")],
            Some(repo.path.to_str().unwrap()),
        )
        .unwrap();

        assert_eq!(
            git_output(&repo.path, &["diff", "--cached", "--name-only"]),
            "backend/build.rs"
        );
    }

    #[test]
    fn git_read_working_tree_keeps_same_file_staged_and_unstaged() {
        let repo = TestRepo::new();
        commit_file(&repo.path, "tracked.txt", "base\n");

        std::fs::write(repo.path.join("tracked.txt"), "base\nstaged\n").unwrap();
        run_git(&repo.path, &["add", "tracked.txt"]);
        std::fs::write(repo.path.join("tracked.txt"), "base\nstaged\nunstaged\n").unwrap();

        let items = git_read_working_tree_changed_files(repo.path.to_str().unwrap()).unwrap();
        let tracked: Vec<_> = items
            .iter()
            .filter(|item| item.file == "tracked.txt")
            .collect();

        assert_eq!(tracked.len(), 2);
        assert!(tracked.iter().any(|item| item.staged));
        assert!(tracked.iter().any(|item| !item.staged));
    }

    #[test]
    fn git_read_working_tree_distinguishes_dirty_and_committed_submodule() {
        let child = TestRepo::new();
        commit_file(&child.path, "child.txt", "base\n");
        let parent = TestRepo::new();
        let output = crate::session::git::git_command()
            .args(["-c", "protocol.file.allow=always", "submodule", "add"])
            .arg(&child.path)
            .arg("modules/child")
            .current_dir(&parent.path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "submodule add failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        run_git(&parent.path, &["commit", "-am", "add submodule"]);
        std::fs::write(parent.path.join("modules/child/child.txt"), "dirty\n").unwrap();

        let items = git_read_working_tree_changed_files(parent.path.to_str().unwrap()).unwrap();
        assert!(
            items.iter().any(|item| {
                item.file == "modules/child"
                    && item.is_submodule
                    && item.submodule_dirty
                    && !item.staged
            }),
            "expected dirty submodule in unstaged list, got {items:?}"
        );
        assert!(!items
            .iter()
            .any(|item| item.file == "modules/child" && item.staged));

        let checkout = parent.path.join("modules/child");
        run_git(&checkout, &["config", "user.email", "test@example.com"]);
        run_git(&checkout, &["config", "user.name", "Test User"]);
        run_git(&checkout, &["add", "child.txt"]);
        run_git(&checkout, &["commit", "-m", "update child"]);

        let items = git_read_working_tree_changed_files(parent.path.to_str().unwrap()).unwrap();
        assert!(
            items.iter().any(|item| {
                item.file == "modules/child"
                    && item.is_submodule
                    && !item.submodule_dirty
                    && !item.staged
            }),
            "expected committed submodule HEAD to be stageable, got {items:?}"
        );

        run_git(&parent.path, &["add", "modules/child"]);
        let items = git_read_working_tree_changed_files(parent.path.to_str().unwrap()).unwrap();
        assert!(
            items.iter().any(|item| {
                item.file == "modules/child"
                    && item.is_submodule
                    && !item.submodule_dirty
                    && item.staged
            }),
            "expected committed submodule gitlink in staged list, got {items:?}"
        );
    }

    #[test]
    fn git_read_working_tree_reports_conflicted_files() {
        let repo = TestRepo::new();
        commit_file(&repo.path, "conflict.txt", "base\n");

        run_git(&repo.path, &["checkout", "-b", "side"]);
        std::fs::write(repo.path.join("conflict.txt"), "side\n").unwrap();
        run_git(&repo.path, &["add", "conflict.txt"]);
        run_git(&repo.path, &["commit", "-m", "side"]);

        run_git(&repo.path, &["checkout", "-b", "ours", "HEAD~1"]);
        std::fs::write(repo.path.join("conflict.txt"), "ours\n").unwrap();
        run_git(&repo.path, &["add", "conflict.txt"]);
        run_git(&repo.path, &["commit", "-m", "ours"]);

        let merge = crate::session::git::git_command()
            .args(["merge", "side"])
            .current_dir(&repo.path)
            .output()
            .unwrap();
        assert!(
            !merge.status.success(),
            "merge should conflict, stdout={}, stderr={}",
            String::from_utf8_lossy(&merge.stdout),
            String::from_utf8_lossy(&merge.stderr)
        );

        let items = git_read_working_tree_changed_files(repo.path.to_str().unwrap()).unwrap();
        assert!(
            items.iter().any(|item| {
                item.file == "conflict.txt"
                    && item.status == GitFileStatus::Conflict
                    && !item.staged
            }),
            "expected conflict.txt as unstaged conflict, got {:?}",
            items
        );
    }
    #[test]
    fn push_changed_file_keeps_staged_and_unstaged_for_same_path() {
        let mut items = Vec::<GitChangedFile>::new();
        let mut seen = std::collections::HashMap::new();

        push_changed_file(
            &mut items,
            &mut seen,
            "Svg.Skia".to_string(),
            GitFileStatus::Modified,
            false,
            false,
            false,
        );
        push_changed_file(
            &mut items,
            &mut seen,
            "Svg.Skia".to_string(),
            GitFileStatus::Added,
            true,
            false,
            false,
        );

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].file, "Svg.Skia");
        assert!(!items[0].staged);
        assert_eq!(items[0].status, GitFileStatus::Modified);
        assert_eq!(items[1].file, "Svg.Skia");
        assert!(items[1].staged);
        assert_eq!(items[1].status, GitFileStatus::Added);
    }

    #[test]
    fn parse_worktree_porcelain_excludes_primary_and_maps_linked() {
        // Build a repo with a committed file on main, then add a linked worktree
        // on a separate branch. The primary worktree's branch (main) must NOT
        // appear in the map; the linked worktree's branch must map to its path.
        let repo = TestRepo::new();
        commit_file(&repo.path, "README.md", "init\n");

        // Create a branch + linked worktree for it.
        let wt_path = repo.path.with_extension("wt-linked");
        run_git(
            &repo.path,
            &[
                "worktree",
                "add",
                wt_path.to_str().unwrap(),
                "-b",
                "feature",
            ],
        );

        let map = super::parse_worktree_porcelain(repo.path.to_str().unwrap()).unwrap();

        // Primary worktree's branch must be excluded.
        assert!(
            !map.contains_key("refs/heads/main"),
            "primary worktree branch should not be badged as a worktree: {:?}",
            map
        );

        // Linked worktree's branch must resolve to the linked path.
        let resolved = map.get("refs/heads/feature").cloned();
        let resolved_norm = resolved.as_deref().map(|p| p.replace('\\', "/"));
        assert_eq!(
            resolved_norm.as_deref(),
            Some(wt_path.to_string_lossy().replace('\\', "/").as_str())
        );

        let _ = std::fs::remove_dir_all(&wt_path);
    }

    #[test]
    fn parse_worktree_porcelain_empty_without_worktrees() {
        // A plain repo with no linked worktrees yields an empty map.
        let repo = TestRepo::new();
        commit_file(&repo.path, "README.md", "init\n");
        let map = super::parse_worktree_porcelain(repo.path.to_str().unwrap()).unwrap();
        assert!(
            map.is_empty(),
            "no linked worktrees → empty map, got {:?}",
            map
        );
    }

    #[test]
    fn git_read_branches_strips_worktree_marker_and_attaches_path() {
        // Regression: `git branch -a` prefixes worktree-checked-out branches
        // with "+ ". The parser must strip that marker so the branch name is
        // clean and its worktree_path resolves via the porcelain map.
        let repo = TestRepo::new();
        commit_file(&repo.path, "README.md", "init\n");

        // main lives in the primary worktree; feature lives in a linked one.
        let wt_path = repo.path.with_extension("wt-feature");
        run_git(
            &repo.path,
            &[
                "worktree",
                "add",
                wt_path.to_str().unwrap(),
                "-b",
                "feature",
            ],
        );

        let (current, detached_head, branches) =
            super::git_read_branches(repo.path.to_str().unwrap()).unwrap();
        assert_eq!(detached_head, None);

        // Current branch is the primary worktree's branch (main or master).
        assert!(
            matches!(current.as_deref(), Some("main") | Some("master")),
            "expected current to be main/master, got {:?}",
            current
        );

        let by_name: std::collections::HashMap<String, &super::GitBranchInfo> =
            branches.iter().map(|b| (b.name.clone(), b)).collect();

        // main must be present with a clean name (no "+ " prefix) and no
        // worktree path (primary worktree is excluded).
        let main_key = if by_name.contains_key("main") {
            "main"
        } else {
            "master"
        };
        let main_branch = by_name.get(main_key).expect("main/master branch present");
        assert!(
            !main_branch.name.starts_with('+'),
            "name should not retain '+ ' marker: {:?}",
            main_branch.name
        );
        assert_eq!(
            main_branch.worktree_path, None,
            "primary worktree must not be badged"
        );

        // feature must resolve to the linked worktree's path.
        let feature_branch = by_name.get("feature").expect("feature branch present");
        assert!(
            !feature_branch.name.starts_with('+'),
            "name should not retain '+ ' marker: {:?}",
            feature_branch.name
        );
        let resolved = feature_branch
            .worktree_path
            .as_deref()
            .map(|p| p.replace('\\', "/"));
        assert_eq!(
            resolved.as_deref(),
            Some(wt_path.to_string_lossy().replace('\\', "/").as_str())
        );

        let _ = std::fs::remove_dir_all(&wt_path);
    }

    #[test]
    fn git_stash_push_all_then_list_apply_drop() {
        let repo = TestRepo::new();
        commit_file(&repo.path, "tracked.txt", "base\n");
        let branch = git_output(&repo.path, &["symbolic-ref", "--short", "HEAD"]);
        std::fs::write(repo.path.join("tracked.txt"), "changed\n").unwrap();
        std::fs::write(repo.path.join("untracked.txt"), "scratch\n").unwrap();

        git_do_stash_push(repo.path.to_str().unwrap(), Some("custom message"), &[]).unwrap();

        // Working tree is clean again: the change and the untracked file left.
        assert_eq!(
            std::fs::read_to_string(repo.path.join("tracked.txt")).unwrap(),
            "base\n"
        );
        assert!(!repo.path.join("untracked.txt").exists());

        let (_root, stashes) = git_read_stash_list(repo.path.to_str().unwrap()).unwrap();
        assert_eq!(stashes.len(), 1, "expected one stash, got {stashes:?}");
        let entry = &stashes[0];
        assert_eq!(entry.index, 0);
        assert_eq!(entry.message, "custom message");
        assert_eq!(entry.branch, branch);
        assert!(!entry.date.is_empty());
        assert!(
            entry
                .files
                .iter()
                .any(|f| f.file == "tracked.txt" && f.status == GitFileStatus::Modified),
            "expected tracked.txt as modified in stash files, got {:?}",
            entry.files
        );
        assert!(
            entry.files.iter().any(|f| f.file == "untracked.txt"),
            "expected untracked.txt in stash files, got {:?}",
            entry.files
        );

        // Apply restores the changes but keeps the stash entry.
        git_do_stash_apply(repo.path.to_str().unwrap(), 0).unwrap();
        assert_eq!(
            std::fs::read_to_string(repo.path.join("tracked.txt")).unwrap(),
            "changed\n"
        );
        assert!(repo.path.join("untracked.txt").exists());
        let (_root, stashes) = git_read_stash_list(repo.path.to_str().unwrap()).unwrap();
        assert_eq!(stashes.len(), 1, "apply must keep the stash entry");

        // Applying the same stash again on top of a dirty tree conflicts, so
        // clean up before dropping.
        run_git(&repo.path, &["checkout", "--", "tracked.txt"]);
        let _ = std::fs::remove_file(repo.path.join("untracked.txt"));
        git_do_stash_drop(repo.path.to_str().unwrap(), 0).unwrap();
        let (_root, stashes) = git_read_stash_list(repo.path.to_str().unwrap()).unwrap();
        assert!(stashes.is_empty(), "drop must remove the stash entry");
    }

    #[test]
    fn git_stash_push_specific_path_then_pop() {
        let repo = TestRepo::new();
        commit_file(&repo.path, "a.txt", "a\n");
        commit_file(&repo.path, "b.txt", "b\n");
        std::fs::write(repo.path.join("a.txt"), "a changed\n").unwrap();
        std::fs::write(repo.path.join("b.txt"), "b changed\n").unwrap();

        git_do_stash_push(
            repo.path.to_str().unwrap(),
            Some("only a"),
            &[String::from("a.txt")],
        )
        .unwrap();

        // Only a.txt was stashed; b.txt keeps its working-tree change.
        assert_eq!(
            std::fs::read_to_string(repo.path.join("a.txt")).unwrap(),
            "a\n"
        );
        assert_eq!(
            std::fs::read_to_string(repo.path.join("b.txt")).unwrap(),
            "b changed\n"
        );

        let (_root, stashes) = git_read_stash_list(repo.path.to_str().unwrap()).unwrap();
        assert_eq!(stashes.len(), 1);
        assert_eq!(stashes[0].message, "only a");
        assert!(
            stashes[0].files.iter().any(|f| f.file == "a.txt"),
            "expected a.txt in stash files, got {:?}",
            stashes[0].files
        );
        assert!(
            !stashes[0].files.iter().any(|f| f.file == "b.txt"),
            "b.txt must not be in the stash, got {:?}",
            stashes[0].files
        );

        // Pop restores the change and removes the stash entry.
        git_do_stash_pop(repo.path.to_str().unwrap(), 0).unwrap();
        assert_eq!(
            std::fs::read_to_string(repo.path.join("a.txt")).unwrap(),
            "a changed\n"
        );
        let (_root, stashes) = git_read_stash_list(repo.path.to_str().unwrap()).unwrap();
        assert!(stashes.is_empty(), "pop must remove the stash entry");
    }

    #[test]
    fn git_stash_list_empty_without_stashes() {
        let repo = TestRepo::new();
        commit_file(&repo.path, "tracked.txt", "base\n");
        let (_root, stashes) = git_read_stash_list(repo.path.to_str().unwrap()).unwrap();
        assert!(stashes.is_empty(), "no stashes → empty list");
    }

    #[test]
    fn git_submodule_list_reports_head_and_detached() {
        let child = TestRepo::new();
        commit_file(&child.path, "child.txt", "base\n");
        let child_branch = git_output(&child.path, &["symbolic-ref", "--short", "HEAD"]);
        let parent = TestRepo::new();
        let output = crate::session::git::git_command()
            .args(["-c", "protocol.file.allow=always", "submodule", "add"])
            .arg(&child.path)
            .arg("modules/child")
            .current_dir(&parent.path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "submodule add failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        run_git(&parent.path, &["commit", "-am", "add submodule"]);

        let (_root, submodules) = git_read_submodule_list(parent.path.to_str().unwrap()).unwrap();
        assert_eq!(submodules.len(), 1, "expected one submodule");
        assert_eq!(submodules[0].path, "modules/child");
        assert_eq!(submodules[0].head.as_deref(), Some(child_branch.as_str()));
        assert_eq!(submodules[0].head_detached, None);

        // Detached HEAD inside the submodule → head_detached short hash.
        let checkout = parent.path.join("modules/child");
        run_git(&checkout, &["checkout", "--detach", "HEAD"]);
        let expected = git_output(&checkout, &["rev-parse", "--short", "HEAD"]);
        let (_root, submodules) = git_read_submodule_list(parent.path.to_str().unwrap()).unwrap();
        assert_eq!(submodules.len(), 1);
        assert_eq!(submodules[0].head, None);
        assert_eq!(
            submodules[0].head_detached.as_deref(),
            Some(expected.as_str())
        );

        // De-initialized submodule → recorded gitlink hash from the status line.
        run_git(
            &parent.path,
            &["submodule", "deinit", "-f", "modules/child"],
        );
        let recorded = git_output(&parent.path, &["rev-parse", "--short", ":modules/child"]);
        let (_root, submodules) = git_read_submodule_list(parent.path.to_str().unwrap()).unwrap();
        assert_eq!(submodules.len(), 1);
        assert_eq!(submodules[0].path, "modules/child");
        assert_eq!(submodules[0].head, None);
        assert_eq!(
            submodules[0].head_detached.as_deref(),
            Some(recorded.as_str()),
            "not initialized → recorded gitlink hash"
        );
    }

    #[test]
    fn git_submodule_list_empty_without_submodules() {
        let repo = TestRepo::new();
        commit_file(&repo.path, "tracked.txt", "base\n");
        let (_root, submodules) = git_read_submodule_list(repo.path.to_str().unwrap()).unwrap();
        assert!(submodules.is_empty(), "no submodules → empty list");
    }

    /// `git checkout --track <tag>` fails the whole command with "cannot set
    /// up tracking information; starting point 'X' is not a branch". Checking
    /// out a tag with `-B` (the CheckoutDialog's tag flow) must instead drop
    /// the flag and still create the local branch.
    #[test]
    fn checkout_tag_drops_track_and_creates_branch() {
        let repo = TestRepo::new();
        commit_file(&repo.path, "tracked.txt", "base\n");
        run_git(&repo.path, &["tag", "v0.1.0-alpha.6"]);

        git_do_checkout(
            repo.path.to_str().unwrap(),
            "v0.1.0-alpha.6",
            true,
            "v0.1.0-alpha.6",
            true,
            true,
            false,
        )
        .unwrap();

        assert_eq!(
            git_output(&repo.path, &["symbolic-ref", "HEAD"]),
            // Full ref: `--short` would yield "heads/v0.1.0-alpha.6" because
            // the same-named tag makes the short form ambiguous.
            "refs/heads/v0.1.0-alpha.6"
        );
        let upstream = crate::session::git::git_command()
            .args(["rev-parse", "-q", "--verify", "@{upstream}"])
            .current_dir(&repo.path)
            .output()
            .unwrap();
        assert!(
            !upstream.status.success(),
            "a tag start point must not configure an upstream"
        );
    }

    /// Remote branches keep `--track`: the branch list sends them as
    /// `remotes/<remote>/<name>`, so trackability must see through the
    /// `remotes/` prefix and the upstream must end up set.
    #[test]
    fn checkout_remote_branch_keeps_track_and_sets_upstream() {
        let repo = TestRepo::new();
        commit_file(&repo.path, "tracked.txt", "base\n");
        let head = git_output(&repo.path, &["rev-parse", "HEAD"]);
        // Synthesize the remote-tracking ref a fetch would create. The remote
        // itself must exist in the config — `git checkout --track` refuses the
        // `remotes/<remote>/<name>` spelling when it doesn't.
        run_git(
            &repo.path,
            &["remote", "add", "origin", "https://example.com/repo.git"],
        );
        run_git(
            &repo.path,
            &["update-ref", "refs/remotes/origin/feature/x", &head],
        );

        git_do_checkout(
            repo.path.to_str().unwrap(),
            "remotes/origin/feature/x",
            true,
            "feature/x",
            true,
            false,
            false,
        )
        .unwrap();

        assert_eq!(
            git_output(&repo.path, &["symbolic-ref", "HEAD"]),
            "refs/heads/feature/x"
        );
        assert_eq!(
            git_output(&repo.path, &["rev-parse", "--abbrev-ref", "@{upstream}"]),
            "origin/feature/x"
        );
    }
}
