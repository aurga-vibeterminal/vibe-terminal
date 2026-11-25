use tracing::trace;
use vterm_common::GitSubmoduleInfo;

use crate::session::SessionError;

use super::{git_check_repo, git_repo_root};

pub(crate) fn git_read_submodule_list(
    working_path: &str,
) -> Result<(String, Vec<GitSubmoduleInfo>), SessionError> {
    git_check_repo(working_path)?;
    let repo_root = git_repo_root(working_path)?;

    trace!("[git_submodule_list] git submodule status (repo_root={repo_root})");

    let output = crate::session::git::git_command()
        .args(["submodule", "status"])
        .current_dir(&repo_root)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    if !output.status.success() {
        // A broken/incomplete submodule setup (e.g. a gitlink recorded in
        // the index without a matching `.gitmodules` entry makes git fail
        // with "no submodule mapping found in .gitmodules") must not take
        // down the whole Git panel — degrade to an empty submodule list.
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        tracing::warn!(
            "[git_submodule_list] git submodule status failed in {repo_root}, \
             returning no submodules: {stderr}"
        );
        return Ok((repo_root, Vec::new()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut submodules = Vec::new();

    for line in stdout.lines() {
        let line = line.trim_matches('\r');
        if line.len() < 2 {
            continue;
        }
        // `<status-char><sha> <path> [(describe)]`; status char is one of
        // ' ' (clean), '+' (pointer moved), '-' (not initialized),
        // 'U' (conflict).
        let (status_char, rest) = line.split_at(1);
        let mut fields = rest.split_whitespace();
        let Some(sha) = fields.next() else { continue };
        let Some(path) = fields.next() else { continue };
        let path = path.replace('\\', "/");

        // '-' = not initialized: the checkout directory is empty/absent, and
        // running git inside it would walk up to the parent repo — report the
        // recorded gitlink hash instead of probing.
        let (head, head_detached) = if status_char == "-" {
            (None, Some(short_hash(sha)))
        } else {
            let checkout = format!("{repo_root}/{path}");
            match git_output(&checkout, &["symbolic-ref", "--short", "-q", "HEAD"]) {
                Some(branch) if !branch.is_empty() => (Some(branch), None),
                _ => match git_output(&checkout, &["rev-parse", "--short", "HEAD"]) {
                    Some(hash) if !hash.is_empty() => (None, Some(hash)),
                    _ => (None, Some(short_hash(sha))),
                },
            }
        };

        submodules.push(GitSubmoduleInfo {
            path,
            head,
            head_detached,
        });
    }

    Ok((repo_root, submodules))
}

fn git_output(working_path: &str, args: &[&str]) -> Option<String> {
    let output = crate::session::git::git_command()
        .args(args)
        .current_dir(working_path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn short_hash(sha: &str) -> String {
    sha.chars().take(7).collect()
}
