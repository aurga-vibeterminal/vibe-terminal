use tracing::{info, trace};
use vterm_common::{GitChangedFile, GitFileStatus, GitStashEntry};

use crate::session::SessionError;

use super::{git_check_repo, git_repo_root};

pub(crate) fn git_read_stash_list(
    working_path: &str,
) -> Result<(String, Vec<GitStashEntry>), SessionError> {
    git_check_repo(working_path)?;
    let repo_root = git_repo_root(working_path)?;

    trace!("[git_stash_list] git stash list (repo_root={repo_root})");

    let output = crate::session::git::git_command()
        .args(["stash", "list", "--format=%gd%x00%gs%x00%cr%x1e"])
        .current_dir(&repo_root)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SessionError::FileSystemError(format!(
            "git stash list failed: {stderr}"
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut stashes = Vec::new();

    for record in stdout.split('\x1e') {
        let record = record.trim_matches(['\r', '\n']);
        if record.is_empty() {
            continue;
        }
        let mut parts = record.splitn(3, '\0');
        let Some(ref_name) = parts.next() else {
            continue;
        };
        let Some(subject) = parts.next() else {
            continue;
        };
        let Some(relative_date) = parts.next() else {
            continue;
        };
        let Some(index) = parse_stash_index(ref_name) else {
            continue;
        };

        let (branch, message) = parse_stash_subject(subject);
        let files = git_stash_show_files(&repo_root, index)?;

        stashes.push(GitStashEntry {
            index,
            message,
            branch,
            date: relative_date.to_string(),
            files,
        });
    }

    Ok((repo_root, stashes))
}

/// Extracts the index out of a reflog name like `stash@{2}`.
fn parse_stash_index(ref_name: &str) -> Option<u32> {
    let start = ref_name.find('{')? + 1;
    let end = ref_name.rfind('}')?;
    ref_name.get(start..end)?.parse().ok()
}

/// Splits a stash subject into (branch, message). Default stashes look like
/// `WIP on main: abc1234 subject` (the whole subject is kept as the message);
/// custom-message stashes look like `On main: my message`.
fn parse_stash_subject(subject: &str) -> (String, String) {
    if let Some(rest) = subject.strip_prefix("WIP on ") {
        let branch = rest
            .split_once(": ")
            .map(|(branch, _)| branch)
            .unwrap_or(rest);
        return (branch.to_string(), subject.to_string());
    }
    if let Some(rest) = subject.strip_prefix("On ") {
        if let Some((branch, message)) = rest.split_once(": ") {
            return (branch.to_string(), message.to_string());
        }
        return (rest.to_string(), subject.to_string());
    }
    (String::new(), subject.to_string())
}

fn git_stash_show_files(repo_root: &str, index: u32) -> Result<Vec<GitChangedFile>, SessionError> {
    let stash_ref = format!("stash@{{{index}}}");
    let output = crate::session::git::git_command()
        .args([
            "stash",
            "show",
            "--name-status",
            "--include-untracked",
            &stash_ref,
        ])
        .current_dir(repo_root)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SessionError::FileSystemError(format!(
            "git stash show failed: {stderr}"
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut files = Vec::new();

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
            Some('M') | Some('R') | Some('C') => GitFileStatus::Modified,
            Some('A') => GitFileStatus::Added,
            Some('D') => GitFileStatus::Deleted,
            Some('?') => GitFileStatus::Untracked,
            _ => GitFileStatus::Untracked,
        };
        files.push(GitChangedFile {
            file,
            status,
            staged: false,
            is_submodule: false,
            submodule_dirty: false,
        });
    }

    Ok(files)
}

pub(crate) fn git_do_stash_push(
    working_path: &str,
    message: Option<&str>,
    paths: &[String],
) -> Result<(), SessionError> {
    info!(
        "[git_stash_push] working_path={} message={:?} paths={:?}",
        working_path, message, paths
    );
    git_check_repo(working_path)?;
    let repo_root = git_repo_root(working_path)?;

    let mut args = vec![
        "stash".to_string(),
        "push".to_string(),
        "--include-untracked".to_string(),
        "--keep-index".to_string(),
    ];
    if let Some(message) = message.map(str::trim).filter(|message| !message.is_empty()) {
        args.push("-m".to_string());
        args.push(message.to_string());
    }
    if !paths.is_empty() {
        args.push("--".to_string());
        args.extend(paths.iter().cloned());
    }

    let output = crate::session::git::git_command()
        .args(&args)
        .current_dir(&repo_root)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SessionError::FileSystemError(format!(
            "git stash push failed: {stderr}"
        )));
    }

    Ok(())
}

pub(crate) fn git_do_stash_apply(working_path: &str, index: u32) -> Result<(), SessionError> {
    git_do_stash_ref_op(working_path, "apply", index)
}

pub(crate) fn git_do_stash_pop(working_path: &str, index: u32) -> Result<(), SessionError> {
    git_do_stash_ref_op(working_path, "pop", index)
}

pub(crate) fn git_do_stash_drop(working_path: &str, index: u32) -> Result<(), SessionError> {
    git_do_stash_ref_op(working_path, "drop", index)
}

fn git_do_stash_ref_op(
    working_path: &str,
    operation: &str,
    index: u32,
) -> Result<(), SessionError> {
    info!("[git_stash_{operation}] working_path={working_path} index={index}");
    git_check_repo(working_path)?;
    let repo_root = git_repo_root(working_path)?;

    let stash_ref = format!("stash@{{{index}}}");
    let output = crate::session::git::git_command()
        .args(["stash", operation, &stash_ref])
        .current_dir(&repo_root)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SessionError::FileSystemError(format!(
            "git stash {operation} failed: {stderr}"
        )));
    }

    Ok(())
}
