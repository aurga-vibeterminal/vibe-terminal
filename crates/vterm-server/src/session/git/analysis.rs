use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tracing::{info, warn};
use vterm_common::HunkAnalysis;

use crate::addon::{resolve_llm_addon_id, run_chat_completion, AddonHost, LlmRequestContext};
use crate::config::TerminalServerConfig;
use crate::session::session_core::ConnectionTx;

use super::{git_check_repo, git_repo_root, truncate_diff_preserve_hunks};

use vterm_core::protocol::Message;

const DEFAULT_LLM_TIMEOUT_SECONDS: u64 = 120;
const DEFAULT_MAX_DIFF_LINES_PER_HUNK: usize = 150;

const SINGLE_HUNK_SYSTEM_PROMPT: &str = "\
You are a code analysis assistant embedded in a git diff viewer.\n\
Analyze the given diff hunk concisely (1-3 sentences) covering:\n\
1. What the change does (intent)\n\
2. Any potential issues, bugs, or risks\n\
3. Suggestions for improvement (if any)\n\
\n\
Return ONLY your analysis text. No JSON, no markdown headers, no code blocks.";

/// Analyze each hunk individually with a focused LLM call, streaming results
/// back via the connection sender as each hunk completes.
pub(crate) fn git_analyze_hunks_streaming(
    session_id: vterm_common::SessionId,
    working_path: &str,
    path: &str,
    commit: Option<&str>,
    cached: bool,
    compare_with_worktree: bool,
    addon_host: &AddonHost,
    config: &TerminalServerConfig,
    cancellation: &Arc<AtomicBool>,
    tx: &ConnectionTx,
) {
    let addon_id = match resolve_llm_addon_id(config, "code_analysis") {
        Some(id) => id,
        None => {
            warn!("[code-analysis] Code analysis LLM is not configured");
            send_analysis_result(
                tx,
                session_id,
                Vec::new(),
                Some("code analysis LLM not configured".to_string()),
                true,
            );
            return;
        }
    };

    // Run git diff with normal context (not -U999999) to get proper hunk splits
    let diff_text =
        match git_read_diff_for_analysis(working_path, path, commit, cached, compare_with_worktree)
        {
            Ok(diff) => diff,
            Err(e) => {
                send_analysis_result(
                    tx,
                    session_id,
                    Vec::new(),
                    Some(format!("Failed to read diff: {}", e)),
                    true,
                );
                return;
            }
        };

    if diff_text.trim().is_empty() {
        send_analysis_result(
            tx,
            session_id,
            Vec::new(),
            Some("No diff content".to_string()),
            true,
        );
        return;
    }

    // Parse hunk headers and extract per-hunk diff sections
    let hunk_sections = extract_hunk_diff_sections(&diff_text);
    if hunk_sections.is_empty() {
        send_analysis_result(
            tx,
            session_id,
            Vec::new(),
            Some("No hunks found in diff".to_string()),
            true,
        );
        return;
    }

    info!(
        "[code-analysis] Starting per-hunk analysis: {} hunks for file {}",
        hunk_sections.len(),
        path
    );

    let output_language = config.llm.code_analysis.output_language.trim();
    let language_instruction = if output_language != "en" {
        format!("\nWrite your response in {}.", output_language)
    } else {
        String::new()
    };

    let system_prompt = format!("{}{}", SINGLE_HUNK_SYSTEM_PROMPT, language_instruction);

    let stats_context = LlmRequestContext {
        category: "code analysis".to_string(),
    };

    let mut llm_config = config.llm.code_analysis.provider.user_config.clone();
    llm_config.insert(
        "system_prompt".to_string(),
        SINGLE_HUNK_SYSTEM_PROMPT.to_string(),
    );

    let mut accumulated: Vec<HunkAnalysis> = Vec::new();

    for (idx, (coords, hunk_diff)) in hunk_sections.iter().enumerate() {
        // Check cancellation before each hunk
        if cancellation.load(Ordering::SeqCst) {
            info!(
                "[code-analysis] Analysis cancelled at hunk {}/{}",
                idx,
                hunk_sections.len()
            );
            send_analysis_result(tx, session_id, accumulated.clone(), None, true);
            return;
        }

        let (os, oc, ns, nc) = *coords;

        // Truncate individual hunk diff if too large
        let (truncated_diff, _) =
            truncate_diff_preserve_hunks(hunk_diff, DEFAULT_MAX_DIFF_LINES_PER_HUNK);

        let user_prompt = format!(
            "Analyze this diff hunk from file `{}`:\n\n```\n{}\n```",
            path, truncated_diff
        );

        let messages_json = serde_json::json!([
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_prompt }
        ])
        .to_string();

        info!(
            "[code-analysis] Analyzing hunk {}/{}: @@ -{},{} +{},{} @@ ({} bytes)",
            idx + 1,
            hunk_sections.len(),
            os,
            oc,
            ns,
            nc,
            messages_json.len()
        );

        // Try up to 2 attempts per hunk
        let mut analysis_text = None;
        for attempt in 1..=2 {
            if cancellation.load(Ordering::SeqCst) {
                break;
            }

            let completion = run_chat_completion(
                addon_host,
                config,
                &addon_id,
                &messages_json,
                "{}",
                Some(&llm_config),
                None,
                Some(&stats_context),
            );

            if completion.success {
                if let Some(content) = &completion.content {
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        analysis_text = Some(trimmed.to_string());
                        break;
                    }
                }
            }

            warn!(
                "[code-analysis] Hunk {}/{} attempt {} failed: {}",
                idx + 1,
                hunk_sections.len(),
                attempt,
                completion.error.as_deref().unwrap_or("unknown")
            );
        }

        let analysis = analysis_text.unwrap_or_else(|| "Analysis unavailable.".to_string());
        accumulated.push(HunkAnalysis {
            old_start: os,
            old_count: oc,
            new_start: ns,
            new_count: nc,
            analysis,
        });

        let is_final = idx == hunk_sections.len() - 1;
        send_analysis_result(tx, session_id, accumulated.clone(), None, is_final);
    }
}

fn send_analysis_result(
    tx: &ConnectionTx,
    session_id: vterm_common::SessionId,
    hunks: Vec<HunkAnalysis>,
    error: Option<String>,
    done: bool,
) {
    info!(
        "[code-analysis] Sending result: {} hunks, done={}, error={}",
        hunks.len(),
        done,
        error.is_some()
    );
    let _ = tx.send(Message::GitAnalyzeDiffResult {
        session_id,
        hunks,
        error,
        done,
    });
}

/// Run `git diff` with standard context (default 3 lines) for analysis purposes.
/// This produces proper hunk splits, unlike the display diff which uses `-U999999`.
fn git_read_diff_for_analysis(
    working_path: &str,
    path: &str,
    commit: Option<&str>,
    cached: bool,
    compare_with_worktree: bool,
) -> Result<String, String> {
    git_check_repo(working_path).map_err(|e| e.to_string())?;
    let repo_root = git_repo_root(working_path).map_err(|e| e.to_string())?;

    if path.is_empty() {
        return Ok(String::new());
    }

    // Compare a specific revision directly against the working tree.
    if let Some(commit) = commit.filter(|c| !c.is_empty()) {
        if compare_with_worktree {
            let output = crate::session::git::git_command()
                .args(["diff", commit, "--", path])
                .current_dir(&repo_root)
                .output()
                .map_err(|e| format!("git diff failed: {}", e))?;
            return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
        }

        // Committed file — diff between commit's parent and commit
        let output = crate::session::git::git_command()
            .args(["diff", &format!("{}^..{}", commit, commit), "--", path])
            .current_dir(&repo_root)
            .output()
            .map_err(|e| format!("git diff failed: {}", e))?;
        let diff = String::from_utf8_lossy(&output.stdout).into_owned();
        if !diff.is_empty() {
            return Ok(diff);
        }
        // Root commit (no parent) — diff against empty tree
        let output = crate::session::git::git_command()
            .args([
                "diff",
                &format!("4b825dc642cb6eb9a060e54bf899d69f2b7c0d41..{}", commit),
                "--",
                path,
            ])
            .current_dir(&repo_root)
            .output()
            .map_err(|e| format!("git diff failed: {}", e))?;
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }

    // Staged file
    if cached {
        let output = crate::session::git::git_command()
            .args(["diff", "--cached", "--", path])
            .current_dir(&repo_root)
            .output()
            .map_err(|e| format!("git diff failed: {}", e))?;
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }

    // Unstaged tracked file
    let output = crate::session::git::git_command()
        .args(["diff", "--", path])
        .current_dir(&repo_root)
        .output()
        .map_err(|e| format!("git diff failed: {}", e))?;
    let diff = String::from_utf8_lossy(&output.stdout).into_owned();
    if !diff.is_empty() {
        return Ok(diff);
    }

    // Untracked file — show entire file as added
    let full_path = std::path::Path::new(&repo_root).join(path);
    if full_path.exists() {
        let output = match crate::session::git::git_command()
            .args(["diff", "--no-index", "/dev/null", "--", path])
            .current_dir(&repo_root)
            .output()
        {
            Ok(o) => o,
            Err(_) => return Ok(String::new()),
        };
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }

    Ok(String::new())
}

/// Extract per-hunk diff sections from unified diff text.
/// Returns a list of ((old_start, old_count, new_start, new_count), hunk_diff_text).
fn extract_hunk_diff_sections(diff: &str) -> Vec<((u32, u32, u32, u32), String)> {
    let mut sections = Vec::new();
    let mut hunk_start_line = None;
    let mut hunk_coords = (0u32, 0u32, 0u32, 0u32);

    for (line_idx, line) in diff.lines().enumerate() {
        if let Some(rest) = line.strip_prefix("@@ -") {
            // New hunk starts — flush previous section
            if hunk_start_line.is_some() {
                let section_text = diff
                    .lines()
                    .skip(hunk_start_line.unwrap())
                    .take(line_idx - hunk_start_line.unwrap())
                    .collect::<Vec<_>>()
                    .join("\n");
                sections.push((hunk_coords, section_text));
            }

            hunk_coords = parse_single_hunk_header(rest);
            hunk_start_line = Some(line_idx);
        }
    }

    // Flush last section
    if let Some(start) = hunk_start_line {
        let section_text = diff.lines().skip(start).collect::<Vec<_>>().join("\n");
        sections.push((hunk_coords, section_text));
    }

    sections
}

fn parse_single_hunk_header(rest: &str) -> (u32, u32, u32, u32) {
    if let Some(bracket) = rest.find(" @@") {
        let header_part = &rest[..bracket];
        if let Some((old_part, new_part)) = header_part.split_once(' ') {
            let old_start = old_part
                .split(',')
                .next()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);
            let old_count = old_part
                .split(',')
                .nth(1)
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(1);
            let new_part = new_part.strip_prefix('+').unwrap_or(new_part);
            let new_start = new_part
                .split(',')
                .next()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);
            let new_count = new_part
                .split(',')
                .nth(1)
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(1);
            return (old_start, old_count, new_start, new_count);
        }
    }
    (0, 0, 0, 0)
}
