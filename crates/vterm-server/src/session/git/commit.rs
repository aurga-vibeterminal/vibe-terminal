use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tracing::{info, warn};
use vterm_agent::agent::{Agent, AgentConfig, LoopResult};
use vterm_agent::client::types::{ChatCompletionRequest, ChatCompletionResponse, Choice};
use vterm_agent::client::LlmClient;
use vterm_agent::client::{ChatMessage, Usage};
use vterm_agent::error::AgentError;
use vterm_agent::memory::CompressionConfig;
use vterm_agent::tool::builtin::ShellTool;
use vterm_agent::tool::ToolRegistry;
use vterm_common::{GitChangedFile, GitFileStatus};

use crate::addon::{
    resolve_addon_config, resolve_llm_addon_id, run_chat_completion, AddonHost, LlmRequestContext,
};
use crate::config::TerminalServerConfig;
use crate::session::SessionError;

use super::{git_check_repo, git_read_changed_files, git_read_log, git_repo_root};

const DEFAULT_RECENT_COMMITS: usize = 3;
const DEFAULT_MAX_FILES: usize = 50;
const DEFAULT_MAX_DIFF_LINES: usize = 200;
// One context line is enough to locate a change; the hunk header already
// carries the enclosing function signature. Fewer context lines means more
// files fit the context budget.
const DEFAULT_DIFF_CONTEXT_LINES: usize = 1;
const DEFAULT_LLM_TIMEOUT_SECONDS: u64 = 60;
const DEFAULT_MAX_INPUT_TOKENS: usize = 128_000;
const DEFAULT_MAX_OUTPUT_TOKENS: usize = 4_096;
/// Commit generation normally converges in 1-2 iterations. The whole
/// conversation (including the preloaded diff) is re-sent every iteration, so
/// a runaway loop multiplies token cost — cap it tightly.
const DEFAULT_MAX_ITERATIONS: usize = 8;
/// In overview mode, only this many files keep their full (truncated) diff;
/// the rest are reduced to hunk headers.
const DEFAULT_MAX_FULL_DIFF_FILES: usize = 10;
/// Hard cap on a single diff line's length. Minified/JSON blobs can be a
/// single multi-KB line, which would burn thousands of tokens for no signal.
const DEFAULT_MAX_DIFF_LINE_CHARS: usize = 400;
/// Conservative allowance for the system prompt, agent wrapper, message
/// framing, and tool definitions. The remaining configured context is available
/// to the preloaded diff.
const COMMIT_AGENT_PROMPT_OVERHEAD_TOKENS: usize = 2_048;
const UNTRACKED_READ_LIMIT_BYTES: u64 = 256 * 1024;
/// Hard cap on a single file's diff text fed to the LLM. Files larger than this
/// (e.g. a multi-megabyte binary asset) are summarized as "changed" instead of
/// having their content dumped — preventing token-limit blowups.
const DEFAULT_MAX_FILE_DIFF_BYTES: usize = 64 * 1024;
/// Chars-per-token heuristic used to convert the token budget into a char budget.
/// Matches the `chars / 4` estimator in `llm_stats.rs`.
const CHARS_PER_TOKEN: usize = 4;

const DEFAULT_GIT_SYSTEM_PROMPT: &str = "\
You are a git commit message generator.\n\
\n\
Output format: Conventional Commits — type(scope): subject + optional body.\n\
\n\
Types:\n\
- feat: new feature or significant enhancement\n\
- fix: bug fix\n\
- refactor: code restructuring without behavior change\n\
- docs: documentation changes\n\
- test: adding or updating tests\n\
- chore: build, config, tooling, or dependency changes\n\
- perf: performance improvement\n\
- style: formatting, whitespace (no logic change)\n\
\n\
Rules:\n\
1. First line: summarize the single primary purpose in imperative mood, ≤72 characters, with no period at the end.\n\
2. Add a body only when it provides useful context. Separate it from the subject with one blank line.\n\
3. Keep the body concise: at most 3 key points. Write each key point as one short physical line; use additional lines only for distinct changes. Do not hard-wrap a point across lines, use bullets, or add headings.\n\
4. Explain intent, motivation, or important behavior. Do not narrate the analysis process, list files or churn statistics, or speculate about changes you could not inspect.\n\
5. The final answer must start directly with the commit subject and contain only the commit message. Never output reasoning, analysis, evidence, labels such as \"Based on the analysis\" or \"Commit message\", Markdown, or commentary.\n\
\n\
Examples:\n\
\n\
Changes: rename UserService to AccountService across 5 files\n\
Output:\n\
refactor(user): rename UserService to AccountService\n\
\n\
Prepare for multi-account support by aligning naming across the codebase.\n\
\n\
Changes: add JWT validation middleware and update auth routes\n\
Output:\n\
feat(auth): add JWT validation middleware\n\
\n\
Secure API endpoints by validating JWT tokens in a dedicated middleware layer before reaching route handlers.\n\
\n\
Changes: update Cargo.toml dependencies and Cargo.lock\n\
Output:\n\
chore(deps): bump dependencies\n\
\n\
Update serde, tokio, and clap to latest patch versions.";

/// Variant used when recent commits are available as possible style samples.
/// The model must first decide whether they establish a coherent repository
/// style. A coherent style wins; mixed or insufficient samples fall back to
/// the built-in Conventional Commits template.
const GIT_SYSTEM_PROMPT_MIRROR_STYLE: &str = "\
You are a git commit message generator.\n\
\n\
Style selection (perform this before writing):\n\
1. Examine the recent commit messages as a set, not as independent examples.\n\
2. Consistency means a dominant recurring pattern, not identical wording. When multiple samples share the same broad conventions and outnumber alternatives, treat that majority pattern as the repository's style. Ignore isolated outliers such as release, merge, revert, or automated commits. Different change topics, verbs, scopes, or lengths do not by themselves make the style inconsistent. One sample alone is not enough.\n\
3. Determine the dominant subject structure, presence or absence of type/scope prefixes, language, tone, capitalization, punctuation, terminology, and use of a body. If most subjects omit a Conventional Commit type/scope prefix, you must also omit it. If most messages are subject-only, output only a subject and do not invent a body.\n\
4. If a consistent historical style exists, follow it closely, unless a later explicit Language instruction overrides the language. The historical style takes precedence over every formatting rule in the fallback template below, including Conventional Commits, imperative mood, the 72-character limit, and body layout. Do not copy historical commit content.\n\
5. Use the fallback template only when there are fewer than two usable samples or no pattern is repeated more strongly than the alternatives. Never choose the fallback merely because the history contains an isolated outlier, and never blend the historical and fallback formats.\n\
\n\
Fallback template (use only when no consistent historical style exists):\n\
- Format: Conventional Commits — type(scope): subject + optional body.\n\
- Types: feat, fix, refactor, docs, test, chore, perf, style.\n\
- First line: summarize the single primary purpose in imperative mood, ≤72 characters, with no period at the end.\n\
- Add a body only when it provides useful context, separated from the subject by one blank line.\n\
- Keep the body concise: at most 3 key points. Write each key point as one short physical line; use additional lines only for distinct changes. Do not hard-wrap a point across lines, use bullets, or add headings.\n\
\n\
Universal rules:\n\
1. Explain intent, motivation, or important behavior. Do not narrate the analysis process, list files or churn statistics, or speculate about changes you could not inspect.\n\
2. The final answer must start directly with the commit subject and contain only the commit message. Never output reasoning, analysis, evidence, labels such as \"Based on the analysis\" or \"Commit message\", Markdown, or commentary.\n";

/// Pick the system prompt for a commit generation run. When recent commits are
/// available, the model evaluates whether they establish a consistent style;
/// without usable samples, the built-in Conventional Commits default applies.
fn git_system_prompt(recent_commits: &[String]) -> String {
    if recent_commits.is_empty() {
        DEFAULT_GIT_SYSTEM_PROMPT.to_string()
    } else {
        GIT_SYSTEM_PROMPT_MIRROR_STYLE.to_string()
    }
}

/// Build the language directive appended to the git commit system prompt.
///
/// - Empty or "auto": mirror the language only when the recent commits
///   establish a consistent style; otherwise use English with the fallback.
/// - Otherwise: write the commit message in the user-specified language.
fn git_language_instruction(output_language: &str) -> String {
    let normalized = output_language.trim();
    if normalized.is_empty() || normalized.eq_ignore_ascii_case("auto") {
        concat!(
            "\n\nLanguage: when the recent commits establish a consistent historical ",
            "style, use that style's language. If they do not establish a ",
            "consistent style, or no recent commits are available, use English ",
            "with the fallback template."
        )
        .to_string()
    } else {
        // The value is user-entered free text — it may be a code ("en",
        // "zh-CN") or a natural-language name ("English", "中文"). Use it
        // verbatim so whatever the user typed is what the model sees.
        format!(
            "\n\nLanguage: write the entire commit message (subject and body) in {}.",
            normalized
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GitCommitMessageResult {
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
struct CommitAgentConfig {
    recent_commits_count: usize,
    max_files: usize,
    max_diff_lines: usize,
    diff_context_lines: usize,
    /// Maximum input tokens accepted while preparing prompts and agent context.
    max_input_tokens: usize,
    /// Maximum completion tokens requested from the model.
    max_output_tokens: usize,
    input_budget_tokens: usize,
    /// Input-side diff budget after reserving prompt and tool overhead.
    context_budget_chars: usize,
    /// Per-file diff byte cap; larger diffs are summarized rather than dumped.
    max_file_diff_bytes: usize,
    /// Per-line char cap inside a diff; longer lines are truncated.
    max_diff_line_chars: usize,
    /// In overview mode, how many files keep their full diff.
    max_full_diff_files: usize,
    /// Hard cap on agent loop iterations.
    max_iterations: usize,
    llm_timeout: Duration,
}

#[derive(Debug, Clone)]
struct FileChangeSummary {
    file: GitChangedFile,
    diff_bytes: usize,
    diff: Option<String>,
    note: Option<String>,
}

type LlmCallback = Box<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync>;

struct CallbackLlmClient {
    callback: Arc<LlmCallback>,
}

#[async_trait]
impl LlmClient for CallbackLlmClient {
    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AgentError> {
        let messages_json = serde_json::to_string(&request.messages)
            .map_err(|e| AgentError::LlmClient(e.to_string()))?;

        let mut options = serde_json::Map::new();
        if !request.model.is_empty() {
            options.insert("model".into(), serde_json::json!(request.model));
        }
        if let Some(temp) = request.temperature {
            options.insert("temperature".into(), serde_json::json!(temp));
        }
        if let Some(max_tokens) = request.max_tokens {
            options.insert("max_tokens".into(), serde_json::json!(max_tokens));
        }
        if let Some(tools) = &request.tools {
            options.insert("tools".into(), serde_json::json!(tools));
        }
        let options_json =
            serde_json::to_string(&options).map_err(|e| AgentError::LlmClient(e.to_string()))?;

        let callback = self.callback.clone();
        let raw_body = tokio::task::spawn_blocking(move || callback(&messages_json, &options_json))
            .await
            .map_err(|e| AgentError::LlmClient(e.to_string()))?
            .map_err(AgentError::LlmClient)?;

        let mut response: ChatCompletionResponse = serde_json::from_str(&raw_body)
            .map_err(|e| AgentError::LlmClient(format!("Failed to parse response: {}", e)))?;

        if let Some(msg) = response
            .choices
            .first_mut()
            .and_then(|c| c.message.content.as_mut())
        {
            let stripped = strip_think_tags(msg);
            *msg = stripped;
        }

        Ok(response)
    }
}

pub(crate) async fn git_generate_commit_message_with_agent(
    working_path: &str,
    addon_host: Arc<AddonHost>,
    config: &TerminalServerConfig,
) -> GitCommitMessageResult {
    let addon_id = match resolve_llm_addon_id(config, "git") {
        Some(id) => id,
        None => {
            warn!("[git-commit-agent] Git LLM is not configured");
            return GitCommitMessageResult {
                message: None,
                error: Some("git LLM not configured".to_string()),
            };
        }
    };

    let config_values = merged_llm_config(config, &addon_id);
    let commit_config = CommitAgentConfig::from_values(&config_values);
    if let Err(error) = commit_config.validate_context_window() {
        return GitCommitMessageResult {
            message: None,
            error: Some(error),
        };
    }
    let llm_timeout = commit_config.llm_timeout;
    let output_max_tokens = commit_config.max_output_tokens as u32;
    let temperature = config_f32(&config_values, "temperature", 0.7);

    let model = config_values
        .get("model")
        .cloned()
        .unwrap_or_else(|| "gpt-4".to_string());

    let os = std::env::consts::OS;

    let repo_root = match git_repo_root(working_path) {
        Ok(root) => root,
        Err(error) => {
            warn!("[git-commit-agent] Failed to get repo root: {}", error);
            return GitCommitMessageResult {
                message: None,
                error: Some(error.to_string()),
            };
        }
    };

    // Preload the same bounded, binary-safe diff context used by the direct LLM
    // path. Tool-capable models must not be relied on to inspect hunks: some
    // models stop after --stat/--numstat and produce a generic churn summary.
    let prepared = match build_commit_context(working_path, &commit_config) {
        Ok(Some(prepared)) => prepared,
        Ok(None) => {
            return GitCommitMessageResult {
                message: None,
                error: Some("no changes to commit".to_string()),
            };
        }
        Err(error) => {
            warn!("[git-commit-agent] Failed to build diff context: {}", error);
            return GitCommitMessageResult {
                message: None,
                error: Some(error.to_string()),
            };
        }
    };
    let diff_context = prepared.context;
    let recent_commits = prepared.recent_commits;

    // When there are no style samples or the samples are mostly
    // Conventional Commits, the normalizer may additionally use the
    // Conventional shape to locate a subject that was appended below leaked
    // analysis. It is never a hard requirement: a coherent subject in the
    // repository's own historical style is always accepted, whatever that
    // style is.
    let require_conventional =
        recent_commits.is_empty() || recent_style_is_conventional(&recent_commits);

    let mut system_prompt = git_system_prompt(&recent_commits);
    system_prompt.push_str(&git_language_instruction(&config.llm.git.output_language));
    system_prompt.push_str("\n\nA bounded staged diff is already provided in the user message. Analyze its actual +/- hunks before writing the commit message. A stat-only or line-count summary is not an analysis. Use the shell tool only when the supplied context explicitly says a relevant diff was omitted, truncated, or reduced to hunk headers. When you do need more detail, inspect several files in ONE shell call (e.g. `git diff -- path/a.rs path/b.rs`) rather than one call per file. Always set `cwd` to the repository path when running commands.\n");
    system_prompt.push_str("Current OS: ");
    system_prompt.push_str(os);
    system_prompt.push('\n');
    system_prompt.push_str("Repository path: ");
    system_prompt.push_str(&repo_root);
    system_prompt.push('\n');

    let user_prompt = build_agent_user_prompt(&repo_root, os, &diff_context);

    info!(
        "[git-commit-agent] Starting agent with model={} repo={} max_input_tokens={} input_budget_tokens={} max_output_tokens={} temperature={} input_budget_chars={} diff_context_bytes={}",
        model,
        repo_root,
        commit_config.max_input_tokens,
        commit_config.input_budget_tokens,
        output_max_tokens,
        temperature,
        commit_config.context_budget_chars,
        diff_context.len()
    );

    let addon_id_cb = addon_id.clone();
    let config_cb = config.clone();
    let config_overrides_cb = config_values.clone();
    let timeout_cb = llm_timeout;

    let callback: LlmCallback = Box::new(move |messages_json: &str, options_json: &str| {
        let result = run_chat_completion(
            &addon_host,
            &config_cb,
            &addon_id_cb,
            messages_json,
            options_json,
            Some(&config_overrides_cb),
            Some(timeout_cb),
            None,
        );

        if !result.success {
            return Err(result
                .error
                .unwrap_or_else(|| "LLM call failed".to_string()));
        }

        result
            .raw_body
            .ok_or_else(|| "no response body".to_string())
    });

    let client = CallbackLlmClient {
        callback: Arc::new(callback),
    };

    let mut registry = ToolRegistry::new();
    registry.register(Box::new(ShellTool));

    let compression = CompressionConfig {
        // The generic agent performs a hard input + output boundary check
        // against this total window before every API call.
        max_context_tokens: commit_config
            .max_input_tokens
            .saturating_add(commit_config.max_output_tokens),
        ..CompressionConfig::default()
    };
    let agent_config = AgentConfig {
        max_iterations: commit_config.max_iterations,
        model,
        temperature: Some(temperature),
        max_tokens: Some(output_max_tokens),
        compression,
    };

    let agent = Agent::new(Arc::new(client), Arc::new(registry), agent_config);

    let session_id = agent.create_session(&system_prompt, &user_prompt).await;

    let result = agent
        .run_loop(&session_id, |event| {
            //info!("[git-commit-agent] {:?}", event);
        })
        .await;

    match result {
        LoopResult::Completed {
            final_content,
            iterations,
            ..
        } => {
            info!(
                "[git-commit-agent] Completed in {} iterations, {} bytes",
                iterations,
                final_content.len()
            );
            match normalize_commit_message_detailed(Some(&final_content), require_conventional) {
                Ok(message) => GitCommitMessageResult {
                    message: Some(message),
                    error: None,
                },
                Err(reason) => {
                    warn!(
                        "[git-commit-agent] Discarding agent response ({} bytes) and falling back: {}. First line: {:?}",
                        final_content.len(),
                        reason,
                        final_content.lines().next().unwrap_or("")
                    );
                    GitCommitMessageResult {
                        message: Some(fallback_commit_message(working_path)),
                        error: None,
                    }
                }
            }
        }
        LoopResult::MaxIterationsExceeded {
            last_content,
            iterations,
            ..
        } => {
            warn!(
                "[git-commit-agent] Max iterations exceeded ({})",
                iterations
            );
            match normalize_commit_message_detailed(Some(&last_content), require_conventional) {
                Ok(message) => GitCommitMessageResult {
                    message: Some(message),
                    error: None,
                },
                Err(reason) => {
                    warn!(
                        "[git-commit-agent] Discarding partial agent response ({} bytes) and falling back: {}. First line: {:?}",
                        last_content.len(),
                        reason,
                        last_content.lines().next().unwrap_or("")
                    );
                    GitCommitMessageResult {
                        message: Some(fallback_commit_message(working_path)),
                        error: None,
                    }
                }
            }
        }
        LoopResult::Cancelled { iterations, .. } => {
            warn!(
                "[git-commit-agent] Cancelled after {} iterations",
                iterations
            );
            GitCommitMessageResult {
                message: None,
                error: Some("agent cancelled".to_string()),
            }
        }
        LoopResult::Error {
            error, iterations, ..
        } => {
            warn!(
                "[git-commit-agent] Error after {} iterations: {}",
                iterations, error
            );
            GitCommitMessageResult {
                message: None,
                error: Some(error.to_string()),
            }
        }
    }
}

impl CommitAgentConfig {
    fn from_values(values: &HashMap<String, String>) -> Self {
        let max_input_tokens = config_usize_with_legacy(
            values,
            "max_input_tokens",
            "max_tokens",
            DEFAULT_MAX_INPUT_TOKENS,
        );
        let max_output_tokens =
            config_usize(values, "max_output_tokens", DEFAULT_MAX_OUTPUT_TOKENS);
        let input_budget_tokens =
            max_input_tokens.saturating_sub(COMMIT_AGENT_PROMPT_OVERHEAD_TOKENS);
        let context_budget_chars = input_budget_tokens.saturating_mul(CHARS_PER_TOKEN).max(1);

        Self {
            recent_commits_count: config_usize(
                values,
                "recent_commits_count",
                DEFAULT_RECENT_COMMITS,
            ),
            max_files: config_usize(values, "max_files", DEFAULT_MAX_FILES),
            max_diff_lines: config_usize(values, "max_diff_lines", DEFAULT_MAX_DIFF_LINES),
            diff_context_lines: config_usize(
                values,
                "diff_context_lines",
                DEFAULT_DIFF_CONTEXT_LINES,
            ),
            max_input_tokens,
            max_output_tokens,
            input_budget_tokens,
            context_budget_chars,
            max_file_diff_bytes: config_usize(
                values,
                "max_file_diff_bytes",
                DEFAULT_MAX_FILE_DIFF_BYTES,
            ),
            max_diff_line_chars: config_usize(
                values,
                "max_diff_line_chars",
                DEFAULT_MAX_DIFF_LINE_CHARS,
            ),
            max_full_diff_files: config_usize(
                values,
                "max_full_diff_files",
                DEFAULT_MAX_FULL_DIFF_FILES,
            ),
            max_iterations: config_usize(values, "max_iterations", DEFAULT_MAX_ITERATIONS),
            llm_timeout: Duration::from_secs(config_usize(
                values,
                "llm_timeout_seconds",
                DEFAULT_LLM_TIMEOUT_SECONDS as usize,
            ) as u64),
        }
    }

    fn validate_context_window(&self) -> Result<(), String> {
        if self.input_budget_tokens == 0 {
            return Err(format!(
                "configured max_input_tokens ({}) must exceed prompt overhead ({})",
                self.max_input_tokens, COMMIT_AGENT_PROMPT_OVERHEAD_TOKENS
            ));
        }
        if self.max_output_tokens > u32::MAX as usize {
            return Err(format!(
                "configured max_output_tokens ({}) exceeds the supported maximum ({})",
                self.max_output_tokens,
                u32::MAX
            ));
        }
        Ok(())
    }
}

/// Feature-level Git settings override the addon's legacy/global settings, but
/// omitted fields still inherit from the addon. This keeps existing server
/// configurations (for example a globally configured 256K max_input_tokens value)
/// effective for the commit agent.
fn merged_llm_config(config: &TerminalServerConfig, addon_id: &str) -> HashMap<String, String> {
    let mut values = resolve_addon_config(config, addon_id, None);
    values.extend(resolve_addon_config(
        config,
        addon_id,
        Some(&config.llm.git.provider.user_config),
    ));
    values
}

fn config_usize(
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

fn config_usize_with_legacy(
    values: &HashMap<String, String>,
    key: &str,
    legacy_key: &str,
    default_value: usize,
) -> usize {
    values
        .get(key)
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
        .or_else(|| {
            values
                .get(legacy_key)
                .and_then(|value| value.trim().parse::<usize>().ok())
                .filter(|value| *value > 0)
        })
        .unwrap_or(default_value)
}

fn config_f32(values: &HashMap<String, String>, key: &str, default_value: f32) -> f32 {
    values
        .get(key)
        .and_then(|value| value.trim().parse::<f32>().ok())
        .unwrap_or(default_value)
}

/// Inputs assembled for one commit-generation run.
struct PreparedCommitContext {
    /// The user-message context: change hints, style reference, stat, diffs.
    context: String,
    /// The recent commit messages embedded as the style reference, verbatim.
    /// Mirrored back to the caller to pick the system prompt and output
    /// validation mode.
    recent_commits: Vec<String>,
}

fn build_commit_context(
    working_path: &str,
    config: &CommitAgentConfig,
) -> Result<Option<PreparedCommitContext>, SessionError> {
    git_check_repo(working_path)?;

    let repo_root = git_repo_root(working_path)?;
    let changes = select_commit_changes(working_path)?;
    if changes.is_empty() {
        return Ok(None);
    }

    let recent_commits = read_recent_commit_messages(working_path, config.recent_commits_count)?;

    let staged = changes.iter().any(|change| change.staged);

    let mut context = String::new();

    // User prompt instructions
    context.push_str("Analyze the changes below and generate a precise commit message.\n");
    context.push_str("Identify the primary purpose of the change.\n");
    context.push_str("Focus on the intent and motivation, not just listing what changed.\n");
    context.push('\n');

    // Branch context
    if let Some(branch) = read_current_branch(&repo_root) {
        context.push_str("Current branch: ");
        context.push_str(&branch);
        context.push('\n');
    }

    context.push_str(if staged {
        "Scope: staged files only.\n"
    } else {
        "Scope: unstaged and untracked files because nothing is staged.\n"
    });

    // Change hints
    let hints = detect_change_hints(&changes);
    if !hints.is_empty() {
        context.push('\n');
        for hint in &hints {
            context.push_str("Hint: ");
            context.push_str(hint);
            context.push('\n');
        }
    }
    context.push('\n');

    // Recent commits are candidate style samples and are included verbatim.
    // They win only when they collectively establish a consistent pattern.
    // Merge commits and messages that look like leaked agent analysis are
    // skipped before this point.
    context.push_str("Recent commits (derive the dominant style; ignore isolated outliers):\n");
    if recent_commits.is_empty() {
        context.push_str("- No recent commits available.\n");
    } else {
        for message in &recent_commits {
            context.push_str("--- commit ---\n");
            context.push_str(message.trim());
            context.push_str("\n--- end commit ---\n");
        }
    }
    context.push('\n');

    // Diff stat overview
    if let Some(stat) = read_diff_stat(&repo_root, staged) {
        context.push_str("Diff stat:\n");
        context.push_str(&stat);
        context.push_str("\n\n");
    }

    // Sort changes by file priority, then status rank
    let mut selected_changes = changes.clone();
    selected_changes.sort_by(|left, right| {
        file_priority(&left.file)
            .cmp(&file_priority(&right.file))
            .then_with(|| change_rank(&left.status).cmp(&change_rank(&right.status)))
            .then_with(|| left.file.cmp(&right.file))
            .then_with(|| left.staged.cmp(&right.staged))
    });

    let omitted_file_count = selected_changes.len().saturating_sub(config.max_files);
    if omitted_file_count > 0 {
        warn!(
            "[git-commit] Summarizing {} files and omitting detailed diffs for {} because max_files={}",
            changes.len(),
            omitted_file_count,
            config.max_files
        );
        context
            .push_str("Large change set; all files are listed, but detailed diffs are limited.\n");
        context.push_str(&directory_summary(&changes));
        context.push('\n');
        selected_changes.truncate(config.max_files);
    }

    let summaries = summarize_files(&repo_root, &selected_changes, config)?;

    // Overview mode: when the total diff volume would eat most of the context
    // budget, only the top files keep their full (already truncated) diff; the
    // rest are reduced to hunk headers so every file stays visible to the
    // model at a fraction of the token cost.
    let total_diff_bytes: usize = summaries
        .iter()
        .filter_map(|summary| summary.diff.as_ref())
        .map(String::len)
        .sum();
    let overview_mode = total_diff_bytes > config.context_budget_chars * 4 / 5;
    let mut full_diff_count = 0usize;
    let full_diff_flags: Vec<bool> = summaries
        .iter()
        .map(|summary| {
            if summary.diff.is_none() {
                return false;
            }
            if !overview_mode || full_diff_count < config.max_full_diff_files {
                full_diff_count += 1;
                true
            } else {
                false
            }
        })
        .collect();

    // Merged "Changed files" section (status + byte size + notes in one pass)
    context.push_str("Changed files:\n");
    let mut manifest_omitted = 0usize;
    for (index, summary) in summaries.iter().enumerate() {
        let mut line = String::new();
        line.push_str("- ");
        line.push_str(status_label(&summary.file.status));
        line.push(' ');
        line.push_str(&summary.file.file);
        if summary.file.staged {
            line.push_str(" (staged)");
        }
        line.push_str(" [");
        line.push_str(&summary.diff_bytes.to_string());
        line.push_str(" bytes");
        if let Some(note) = &summary.note {
            line.push_str(", ");
            line.push_str(note);
        }
        if summary.diff.is_some() && !full_diff_flags[index] {
            line.push_str(", hunk headers only");
        }
        line.push(']');
        line.push('\n');
        if context.len().saturating_add(line.len()).saturating_add(256)
            > config.context_budget_chars
        {
            manifest_omitted += 1;
            continue;
        }
        context.push_str(&line);
    }
    if manifest_omitted > 0 {
        context.push_str("- Omitted ");
        context.push_str(&manifest_omitted.to_string());
        context.push_str(" additional files due to context budget.\n");
    }
    if omitted_file_count > 0 {
        context.push_str("- Detailed diffs omitted for ");
        context.push_str(&omitted_file_count.to_string());
        context.push_str(" lower-priority files.\n");
    }

    context.push_str("\nDiffs:\n");
    if overview_mode {
        context.push_str(
            "Large change set: full diffs are limited to the top files; other files show hunk headers only.\n",
        );
    }
    for (index, summary) in summaries.into_iter().enumerate() {
        let Some(diff) = summary.diff else {
            continue;
        };
        let body = if full_diff_flags[index] {
            diff
        } else {
            hunk_headers_only(&diff)
        };
        if context.len().saturating_add(body.len()).saturating_add(128)
            > config.context_budget_chars
        {
            warn!(
                "[git-commit] Context budget reached before adding {}",
                summary.file.file
            );
            context.push_str("\n[Additional diffs omitted due to context budget]\n");
            break;
        }
        context.push_str("\n--- ");
        context.push_str(&summary.file.file);
        context.push_str(" ---\n");
        context.push_str(&body);
        if !body.ends_with('\n') {
            context.push('\n');
        }
    }

    if context.len() > config.context_budget_chars {
        const MARKER: &str = "\n[Context truncated to configured input budget]\n";
        let content_budget = config.context_budget_chars.saturating_sub(MARKER.len());
        let mut boundary = content_budget.min(context.len());
        while boundary > 0 && !context.is_char_boundary(boundary) {
            boundary -= 1;
        }
        context.truncate(boundary);
        if MARKER.len() <= config.context_budget_chars {
            context.push_str(MARKER);
        }
    }

    Ok(Some(PreparedCommitContext {
        context,
        recent_commits,
    }))
}

fn select_commit_changes(working_path: &str) -> Result<Vec<GitChangedFile>, SessionError> {
    let changes = git_read_changed_files(working_path, None)?;
    let staged = changes
        .iter()
        .filter(|change| change.staged)
        .cloned()
        .collect::<Vec<_>>();
    if !staged.is_empty() {
        return Ok(staged);
    }
    Ok(changes
        .into_iter()
        .filter(|change| !change.staged)
        .collect::<Vec<_>>())
}

fn summarize_files(
    repo_root: &str,
    changes: &[GitChangedFile],
    config: &CommitAgentConfig,
) -> Result<Vec<FileChangeSummary>, SessionError> {
    let mut summaries = Vec::with_capacity(changes.len());
    for change in changes {
        let (diff, note) = match change.status {
            GitFileStatus::Untracked => untracked_file_diff(repo_root, &change.file, config)?,
            _ => tracked_file_diff(repo_root, change, config)?,
        };
        let diff_bytes = diff.as_ref().map(|value| value.len()).unwrap_or(0);
        summaries.push(FileChangeSummary {
            file: change.clone(),
            diff_bytes,
            diff,
            note,
        });
    }

    summaries.sort_by(|left, right| {
        file_priority(&left.file.file)
            .cmp(&file_priority(&right.file.file))
            .then_with(|| {
                let left_rank = change_rank(&left.file.status);
                let right_rank = change_rank(&right.file.status);
                left_rank.cmp(&right_rank)
            })
            // Within the same class, larger diffs first: the biggest change is
            // usually the commit's primary purpose and should win the budget.
            .then_with(|| right.diff_bytes.cmp(&left.diff_bytes))
            .then_with(|| left.file.file.cmp(&right.file.file))
            .then_with(|| left.file.staged.cmp(&right.file.staged))
    });
    Ok(summaries)
}

fn tracked_file_diff(
    repo_root: &str,
    change: &GitChangedFile,
    config: &CommitAgentConfig,
) -> Result<(Option<String>, Option<String>), SessionError> {
    let diff = read_commit_diff(
        repo_root,
        &change.file,
        change.staged,
        config.diff_context_lines,
    )?;
    if diff.trim().is_empty() {
        return Ok((None, Some("empty or binary diff omitted".to_string())));
    }
    // git collapses true binaries to a "Binary files ... differ" line; never
    // forward that as content to the LLM.
    if looks_like_binary_diff(&diff) {
        return Ok((None, Some("binary file, content omitted".to_string())));
    }
    // Reformat-only churn carries no intent; labeling it keeps the model from
    // over-weighting whitespace noise when deciding the primary purpose.
    if is_whitespace_only_change(repo_root, &change.file, change.staged) {
        return Ok((
            None,
            Some("whitespace-only change, diff omitted".to_string()),
        ));
    }
    // Collapse single-line blobs (minified JS, packed JSON) to a short snippet
    // before the byte cap, so they contribute signal instead of being dropped.
    let diff = truncate_long_lines(&diff, config.max_diff_line_chars);
    // Large text blobs (e.g. a multi-MB pretty-printed glTF/JSON) would
    // dominate the context budget. Summarize them instead of dumping content.
    if diff.len() > config.max_file_diff_bytes {
        return Ok((
            None,
            Some(format!("large file ({} bytes), diff omitted", diff.len())),
        ));
    }
    let (diff, truncated) = truncate_diff_preserve_hunks(&diff, config.max_diff_lines);
    Ok((
        Some(diff),
        truncated.then(|| format!("diff truncated to {} lines", config.max_diff_lines)),
    ))
}

fn untracked_file_diff(
    repo_root: &str,
    path: &str,
    config: &CommitAgentConfig,
) -> Result<(Option<String>, Option<String>), SessionError> {
    let full_path = Path::new(repo_root).join(path);
    let metadata = std::fs::metadata(&full_path)
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    if !metadata.is_file() {
        return Ok((None, Some("untracked non-file omitted".to_string())));
    }

    let mut file =
        File::open(&full_path).map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(UNTRACKED_READ_LIMIT_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    if bytes.contains(&0) {
        return Ok((None, Some("binary file omitted".to_string())));
    }

    let content = String::from_utf8_lossy(&bytes);
    let content = truncate_long_lines(&content, config.max_diff_line_chars);
    // Multi-MB pretty-printed blobs still exceed the byte cap after line
    // truncation; summarize them rather than dumping the content. Single-line
    // blobs were already collapsed to a snippet by truncate_long_lines.
    if content.len() > config.max_file_diff_bytes {
        return Ok((
            None,
            Some(format!(
                "large untracked file ({} bytes), content omitted",
                metadata.len()
            )),
        ));
    }

    let mut diff = String::new();
    diff.push_str("new file mode 100644\n");
    diff.push_str("--- /dev/null\n");
    diff.push_str("+++ b/");
    diff.push_str(path);
    diff.push('\n');
    for line in content.lines() {
        diff.push('+');
        diff.push_str(line);
        diff.push('\n');
    }
    let (diff, truncated) = truncate_diff_preserve_hunks(&diff, config.max_diff_lines);
    let note = if metadata.len() > UNTRACKED_READ_LIMIT_BYTES {
        Some("untracked file read limit reached".to_string())
    } else {
        truncated.then(|| format!("diff truncated to {} lines", config.max_diff_lines))
    };
    Ok((Some(diff), note))
}

fn directory_summary(changes: &[GitChangedFile]) -> String {
    let mut dirs: BTreeMap<String, [usize; 5]> = BTreeMap::new();
    for change in changes {
        let dir = Path::new(&change.file)
            .parent()
            .and_then(|path| path.to_str())
            .filter(|path| !path.is_empty())
            .unwrap_or(".")
            .to_string();
        let counts = dirs.entry(dir).or_insert([0, 0, 0, 0, 0]);
        counts[change_rank(&change.status)] += 1;
    }

    let mut output = String::from("Directory summary:\n");
    for (dir, counts) in dirs {
        output.push_str("- ");
        output.push_str(&dir);
        output.push_str(": ");
        output.push_str(&format!(
            "{} modified, {} added, {} deleted, {} untracked, {} conflict\n",
            counts[0], counts[1], counts[2], counts[3], counts[4]
        ));
    }
    output
}

pub fn truncate_diff_preserve_hunks(input: &str, max_lines: usize) -> (String, bool) {
    let lines: Vec<&str> = input.lines().collect();
    if lines.len() <= max_lines {
        return (input.to_string(), false);
    }
    if max_lines <= 3 {
        return (
            lines
                .into_iter()
                .take(max_lines)
                .chain(std::iter::once("[...]"))
                .collect::<Vec<_>>()
                .join("\n"),
            true,
        );
    }

    // Classify each line: is it a changed line (+/-), header, or context?
    #[derive(PartialEq, Eq, Clone, Copy)]
    enum LineKind {
        Header,   // diff --git, index, ---, +++
        HunkHead, // @@ ... @@
        Changed,  // + or - lines
        Context,  // unchanged lines
    }

    let kinds: Vec<LineKind> = lines
        .iter()
        .map(|line| {
            if line.starts_with("diff --git ")
                || line.starts_with("index ")
                || line.starts_with("--- ")
                || line.starts_with("+++ ")
            {
                LineKind::Header
            } else if line.starts_with("@@ ") {
                LineKind::HunkHead
            } else if line.starts_with('+') || line.starts_with('-') {
                LineKind::Changed
            } else {
                LineKind::Context
            }
        })
        .collect();

    // Reserve budget for context lines around each important line
    // Reserve 2 slots for [...] and omission count line
    let effective_max = max_lines.saturating_sub(2).max(1);
    let context_budget = (effective_max / 5).max(2);
    let mut keep = vec![false; lines.len()];
    let mut kept_count = 0usize;

    // Keep file headers first, then spread the remaining budget across hunks.
    // The old implementation kept important lines from the beginning until the
    // budget was exhausted, which could entirely hide later behavior changes.
    for (index, kind) in kinds.iter().enumerate() {
        if *kind == LineKind::Header && kept_count < effective_max {
            keep[index] = true;
            kept_count += 1;
        }
    }

    let hunk_heads: Vec<usize> = kinds
        .iter()
        .enumerate()
        .filter_map(|(index, kind)| (*kind == LineKind::HunkHead).then_some(index))
        .collect();
    let hunk_head_budget = effective_max.saturating_sub(kept_count);
    for index in evenly_spaced_indices(&hunk_heads, hunk_head_budget) {
        if !keep[index] {
            keep[index] = true;
            kept_count += 1;
        }
    }

    // Group changed lines by hunk, allocate quotas round-robin so small/later
    // hunks are represented, then sample evenly inside each hunk.
    let mut changed_groups: Vec<Vec<usize>> = Vec::new();
    let mut current_group: Vec<usize> = Vec::new();
    for (index, kind) in kinds.iter().enumerate() {
        if *kind == LineKind::HunkHead && !current_group.is_empty() {
            changed_groups.push(std::mem::take(&mut current_group));
        } else if *kind == LineKind::Changed {
            current_group.push(index);
        }
    }
    if !current_group.is_empty() {
        changed_groups.push(current_group);
    }

    let mut quotas = vec![0usize; changed_groups.len()];
    let mut remaining = effective_max.saturating_sub(kept_count);
    while remaining > 0 {
        let mut allocated = false;
        for (quota, group) in quotas.iter_mut().zip(&changed_groups) {
            if remaining == 0 {
                break;
            }
            if *quota < group.len() {
                *quota += 1;
                remaining -= 1;
                allocated = true;
            }
        }
        if !allocated {
            break;
        }
    }
    for (group, quota) in changed_groups.iter().zip(quotas) {
        for index in evenly_spaced_indices(group, quota) {
            if !keep[index] {
                keep[index] = true;
                kept_count += 1;
            }
        }
    }

    let important: Vec<usize> = keep
        .iter()
        .enumerate()
        .filter_map(|(index, kept)| kept.then_some(index))
        .collect();

    // Distribute context budget around important lines
    let mut remaining_context = context_budget;
    for &i in &important {
        if remaining_context == 0 || kept_count >= effective_max {
            break;
        }
        // Try to add 1 context line before and after
        for &offset in &[1usize, 2] {
            if remaining_context == 0 || kept_count >= effective_max {
                break;
            }
            if i >= offset && !keep[i - offset] && kinds[i - offset] == LineKind::Context {
                keep[i - offset] = true;
                remaining_context -= 1;
                kept_count += 1;
            }
        }
        for offset in [1, 2] {
            if remaining_context == 0 || kept_count >= effective_max {
                break;
            }
            let after = i + offset;
            if after < lines.len() && !keep[after] && kinds[after] == LineKind::Context {
                keep[after] = true;
                remaining_context -= 1;
                kept_count += 1;
            }
        }
    }

    // Assemble the kept lines, inserting "[...]" at gaps. Gap markers and the
    // omission-count line are not part of the line budget, so the result can
    // still exceed max_lines. Shrink by unkeeping the least valuable lines —
    // context lines first, then changed lines that are not their hunk's only
    // kept sample — instead of blindly chopping the tail, which used to drop
    // entire later hunks.
    fn assemble<'a>(lines: &[&'a str], keep: &[bool]) -> (Vec<&'a str>, usize) {
        let mut output: Vec<&'a str> = Vec::new();
        let mut omitted = 0usize;
        let mut prev_was_kept = false;
        for (i, &line) in lines.iter().enumerate() {
            if keep[i] {
                if !prev_was_kept && !output.is_empty() {
                    output.push("[...]");
                }
                output.push(line);
                prev_was_kept = true;
            } else {
                omitted += 1;
                prev_was_kept = false;
            }
        }
        (output, omitted)
    }

    // True when another changed line in the same hunk is also kept, so
    // unkeeping `index` does not erase the hunk's only sample.
    let has_kept_changed_sibling = |keep: &[bool], index: usize| -> bool {
        let mut start = index;
        while start > 0 && kinds[start - 1] != LineKind::HunkHead {
            start -= 1;
        }
        let mut end = index;
        while end + 1 < lines.len() && kinds[end + 1] != LineKind::HunkHead {
            end += 1;
        }
        (start..=end).any(|i| i != index && keep[i] && kinds[i] == LineKind::Changed)
    };

    let (mut output, mut omitted) = assemble(&lines, &keep);
    while output.len() > max_lines {
        let victim = (0..lines.len())
            .rev()
            .find(|&i| keep[i] && kinds[i] == LineKind::Context)
            .or_else(|| {
                (0..lines.len()).rev().find(|&i| {
                    keep[i] && kinds[i] == LineKind::Changed && has_kept_changed_sibling(&keep, i)
                })
            });
        let Some(victim) = victim else {
            break;
        };
        keep[victim] = false;
        (output, omitted) = assemble(&lines, &keep);
    }

    // Fallback for pathological layouts (headers/hunk heads alone exceeding
    // the budget): keep the old tail truncation rather than exceed max_lines.
    if output.len() > max_lines {
        output.truncate(max_lines);
        if let Some(last) = output.last_mut() {
            *last = "[...]";
        }
    }

    let omitted_line = format!("[{} context lines omitted]", omitted);
    output.push(&omitted_line);
    (output.join("\n"), true)
}

fn evenly_spaced_indices(indices: &[usize], count: usize) -> Vec<usize> {
    let count = count.min(indices.len());
    match count {
        0 => Vec::new(),
        1 => vec![indices[indices.len() / 2]],
        _ => (0..count)
            .map(|slot| indices[slot * (indices.len() - 1) / (count - 1)])
            .collect(),
    }
}

fn read_commit_diff(
    repo_root: &str,
    path: &str,
    cached: bool,
    context_lines: usize,
) -> Result<String, SessionError> {
    let mut args = vec!["diff".to_string(), format!("--unified={context_lines}")];
    if cached {
        args.push("--cached".to_string());
    }
    args.push("--".to_string());
    args.push(path.to_string());

    let output = crate::session::git::git_command()
        .args(&args)
        .current_dir(repo_root)
        .output()
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(SessionError::FileSystemError(format!(
            "git {} failed: {stderr}",
            args.join(" ")
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// git collapses binary files into a `Binary files a/x and b/x differ` line.
/// Detect that so the raw diff is never forwarded to the LLM as content.
fn looks_like_binary_diff(diff: &str) -> bool {
    diff.lines().any(|line| line.starts_with("Binary files"))
}

/// True when the file's only changes are whitespace (reformat/indent churn).
/// `git diff -w --quiet` exits 0 when ignoring whitespace leaves no delta.
fn is_whitespace_only_change(repo_root: &str, path: &str, cached: bool) -> bool {
    let mut args = vec!["diff".to_string(), "-w".to_string(), "--quiet".to_string()];
    if cached {
        args.push("--cached".to_string());
    }
    args.push("--".to_string());
    args.push(path.to_string());

    crate::session::git::git_command()
        .args(&args)
        .current_dir(repo_root)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Truncate any diff line longer than `max_chars`, appending a marker. Keeps
/// single-line blobs (minified JS, packed JSON) from burning the budget.
fn truncate_long_lines(diff: &str, max_chars: usize) -> String {
    const MARKER: &str = " [... line truncated]";
    let mut out = String::with_capacity(diff.len());
    for segment in diff.split_inclusive('\n') {
        let line = segment.strip_suffix('\n').unwrap_or(segment);
        if line.chars().count() > max_chars {
            out.extend(line.chars().take(max_chars));
            out.push_str(MARKER);
        } else {
            out.push_str(line);
        }
        if segment.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

/// Reduce a diff to its structural skeleton: file headers and hunk headers
/// (which carry the enclosing function signature). Used in overview mode so
/// every file stays visible at a fraction of the token cost.
fn hunk_headers_only(diff: &str) -> String {
    diff.lines()
        .filter(|line| {
            line.starts_with("diff --git ")
                || line.starts_with("--- ")
                || line.starts_with("+++ ")
                || line.starts_with("@@ ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Accept the repository's Conventional Commit-shaped subjects without
/// restricting it to a hard-coded type list. Repositories often use additional
/// types such as `build`, `ci`, or `revert`.
fn is_conventional_subject(line: &str) -> bool {
    let subject = line.trim();
    let Some((header, description)) = subject.split_once(": ") else {
        return false;
    };
    if description.trim().is_empty() {
        return false;
    }

    let header = header.strip_suffix('!').unwrap_or(header);
    let (commit_type, scope) = if let Some(scope_start) = header.find('(') {
        if !header.ends_with(')') {
            return false;
        }
        (
            &header[..scope_start],
            Some(&header[scope_start + 1..header.len() - 1]),
        )
    } else {
        (header, None)
    };

    !commit_type.is_empty()
        && commit_type.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
        && scope.is_none_or(|scope| {
            !scope.is_empty()
                && !scope
                    .chars()
                    .any(|character| character.is_control() || matches!(character, '(' | ')'))
        })
}

/// The most recent commit messages, verbatim. These are the primary style
/// reference: whatever format, language, and tone the repository actually
/// uses. Only merge commits (branch bookkeeping, not style) and messages
/// whose first line looks like leaked agent analysis are skipped.
fn read_recent_commit_messages(
    working_path: &str,
    count: usize,
) -> Result<Vec<String>, SessionError> {
    Ok(git_read_log(working_path, None, None, None)?
        .into_iter()
        .filter(|commit| is_style_reference_candidate(&commit.message, commit.parents.len()))
        .map(|commit| commit.message.trim().to_string())
        .take(count)
        .collect())
}

/// True when `message` (with `parent_count` parents) can serve as a style
/// reference. Merge commits and analysis-polluted messages are excluded.
fn is_style_reference_candidate(message: &str, parent_count: usize) -> bool {
    if parent_count > 1 {
        return false;
    }
    let Some(first_line) = message.lines().find(|line| !line.trim().is_empty()) else {
        return false;
    };
    !looks_like_leaked_analysis(first_line)
}

/// First-line markers of leaked agent analysis. A commit message starting
/// with one of these (case-insensitive) was produced by a failed generation
/// run and must not become a style reference.
const LEAKED_ANALYSIS_PREFIXES: &[&str] = &[
    "based on the",
    "here is",
    "here's",
    "commit message:",
    "suggested commit message:",
    "proposed commit message:",
    "generated commit message:",
    "the commit message",
];

fn looks_like_leaked_analysis(line: &str) -> bool {
    let lower = line.trim().to_lowercase();
    LEAKED_ANALYSIS_PREFIXES
        .iter()
        .any(|prefix| lower.starts_with(prefix))
}

/// True when the majority of the recent commit subjects follow the
/// Conventional Commits shape. When true the normalizer also uses the shape
/// as a cue to locate a subject buried under leaked analysis; it never
/// rejects output on its own, because the repository's own style — in
/// whatever format the history uses — always wins.
fn recent_style_is_conventional(recent: &[String]) -> bool {
    let mut total = 0usize;
    let mut conventional = 0usize;
    for message in recent {
        if let Some(first_line) = message.lines().find(|line| !line.trim().is_empty()) {
            total += 1;
            conventional += is_conventional_subject(first_line) as usize;
        }
    }
    total > 0 && conventional * 2 > total
}

fn normalize_commit_message(content: Option<&str>, require_conventional: bool) -> Option<String> {
    normalize_commit_message_detailed(content, require_conventional).ok()
}

/// [`normalize_commit_message`] with a human-readable rejection reason, so
/// callers can log *why* a completed agent response was discarded and the
/// generic fallback message shown to the user instead.
///
/// Rejection only happens when the response is empty after cleanup or its
/// first line is too long to be a subject. Format is never a rejection
/// reason: the message mirrors the repository's own commit history, whose
/// style can be anything (Conventional Commits, plain sentences, another
/// language, ...).
fn normalize_commit_message_detailed(
    content: Option<&str>,
    require_conventional: bool,
) -> Result<String, String> {
    let content = content
        .ok_or_else(|| "response is missing".to_string())?
        .trim();
    if content.is_empty() {
        return Err("response is empty".to_string());
    }

    // Strip <think/> tags from reasoning models
    let content = strip_think_tags(content);

    // Strip surrounding quotes
    let content = if content.starts_with('"') && content.ends_with('"') && content.len() >= 2 {
        content[1..content.len() - 1].to_string()
    } else if content.starts_with('\'') && content.ends_with('\'') && content.len() >= 2 {
        content[1..content.len() - 1].to_string()
    } else {
        content
    };

    // Strip surrounding tilde fences that some LLMs use instead of backticks.
    let content = strip_surrounding_tildes(&content);

    // Strip Markdown code blocks
    let content = content.strip_prefix("```").unwrap_or(&content).to_string();
    let content = content.strip_suffix("```").unwrap_or(&content).to_string();
    let content = content.trim().to_string();

    // Strip LLM preamble patterns
    let content = strip_preamble_owned(&content);

    let mut lines: Vec<&str> = content.lines().map(str::trim_end).collect();

    // Remove trailing blank lines
    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }

    // Remove leading blank lines
    while lines.first().is_some_and(|line| line.trim().is_empty()) {
        lines.remove(0);
    }

    if require_conventional {
        // Reasoning models sometimes emit analysis and then append the actual
        // commit message. When the first line is not itself a Conventional
        // Commit subject, search from the end for one buried below the
        // leaked analysis and drop everything above it. When none exists,
        // the first line is kept as-is: the repository's own history style
        // wins, so format is never a reason to discard the response.
        if !lines
            .first()
            .is_some_and(|line| is_conventional_subject(line))
        {
            if let Some(subject_index) =
                lines.iter().rposition(|line| is_conventional_subject(line))
            {
                lines.drain(..subject_index);
            }
        }
    }

    // Basic subject hygiene is the only remaining gate. The subject may be
    // in any format the repository history uses — it is never rejected for
    // not matching one particular shape.
    let subject = lines
        .first()
        .ok_or_else(|| "no non-empty lines after cleanup".to_string())?
        .trim();
    if subject.is_empty() {
        return Err("subject line is empty after cleanup".to_string());
    }
    // Subject line max 120 chars (lenient), prefer ≤72
    if subject.len() > 120 {
        return Err(format!(
            "subject line is {} chars (limit 120)",
            subject.len()
        ));
    }

    Ok(lines.join("\n"))
}

/// Remove matching leading/trailing tilde fences, then trim surrounding
/// whitespace so Markdown-style `~~~ ... ~~~` wrappers do not leak into the
/// final commit message.
fn strip_surrounding_tildes(content: &str) -> String {
    let trimmed = content.trim();
    if trimmed.starts_with('`') && trimmed.ends_with('`') {
        return trimmed.trim_matches('`').trim().to_string();
    }
    trimmed.to_string()
}

fn strip_think_tags(content: &str) -> String {
    let mut result = content.to_string();
    // Remove <think...>...</think...> tags
    // Also handles <think\n...\n</think\n (without closing >)
    loop {
        let before = result.len();
        if let Some(start) = result.find("<think") {
            // Find end of opening tag (either > or whitespace/</think)
            let tag_content = &result[start..];
            let open_end = tag_content.find('>').map(|i| start + i + 1).or_else(|| {
                // No > found — opening tag runs into content; find </think
                tag_content.find("\n").map(|i| start + i + 1)
            });

            let Some(open_end) = open_end else {
                break;
            };

            let after_open = &result[open_end..];
            if let Some(close_offset) = after_open.find("</think") {
                let close_start = open_end + close_offset;
                let close_end = result[close_start..]
                    .find('>')
                    .map(|i| close_start + i + 1)
                    .or_else(|| result[close_start..].find('\n').map(|i| close_start + i))
                    .unwrap_or(result.len());
                result.replace_range(start..close_end, "");
            } else {
                // Unclosed think tag - remove from <think to end
                result.replace_range(start.., "");
            }
        }
        if result.len() == before {
            break;
        }
    }
    result
}

/// True when `haystack` starts with `prefix`, ignoring ASCII case. The
/// prefix is always pure ASCII, so a case-insensitive match is guaranteed to
/// consume exactly `prefix.len()` bytes of the haystack.
fn has_ci_prefix(haystack: &str, prefix: &str) -> bool {
    haystack.len() >= prefix.len()
        && haystack
            .chars()
            .zip(prefix.chars())
            .all(|(h, p)| h.to_ascii_lowercase() == p)
}

/// Strip LLM preamble labels that appear at the *start* of the response
/// (e.g. "Here is the commit message:"). Matching is anchored to the
/// beginning on purpose: a pattern occurring inside a legitimate subject or
/// body — for example a body sentence about "the commit message" — must
/// never truncate the message, which the previous substring search did.
fn strip_preamble_owned(content: &str) -> String {
    let mut result = content.trim().to_string();
    loop {
        let mut found = false;
        for pattern in PREAMBLE_PATTERNS {
            if has_ci_prefix(&result, pattern) {
                let after = &result[pattern.len()..];
                let cleaned = after
                    .trim_start()
                    // Labels such as "commit message:" leave a dangling colon.
                    .trim_start_matches(|c| matches!(c, ':' | '：'))
                    .trim_start();
                if !cleaned.is_empty() && cleaned.len() < result.len() {
                    result = cleaned.to_string();
                    found = true;
                    break;
                }
            }
        }
        if !found {
            break;
        }
    }
    result
}

fn fallback_commit_message(working_path: &str) -> String {
    match select_commit_changes(working_path) {
        Ok(changes) if changes.len() == 1 => {
            let change = &changes[0];
            format!("Update {}", change.file)
        }
        Ok(changes) if !changes.is_empty() => format!("Update {} files", changes.len()),
        _ => "Update repository changes".to_string(),
    }
}

fn change_rank(status: &GitFileStatus) -> usize {
    match status {
        GitFileStatus::Modified => 0,
        GitFileStatus::Added => 1,
        GitFileStatus::Deleted => 2,
        GitFileStatus::Untracked => 3,
        GitFileStatus::Conflict => 4,
    }
}

fn status_label(status: &GitFileStatus) -> &'static str {
    match status {
        GitFileStatus::Modified => "modified",
        GitFileStatus::Added => "added",
        GitFileStatus::Deleted => "deleted",
        GitFileStatus::Untracked => "untracked",
        GitFileStatus::Conflict => "conflict",
    }
}

fn read_current_branch(repo_root: &str) -> Option<String> {
    let output = crate::session::git::git_command()
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo_root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() || branch == "HEAD" {
        return None;
    }
    Some(branch)
}

/// Build an agent prompt around a preloaded, bounded diff. Providing the hunks
/// up front is important for models that otherwise stop after reading only
/// --stat/--numstat and return a generic line-count summary.
fn build_agent_user_prompt(repo_root: &str, os: &str, diff_context: &str) -> String {
    format!(
        "Analyze the supplied git changes and generate a precise commit message.\n\
         Repository path: {repo_root}\n\
         Current OS: {os}\n\
         \n\
         Requirements:\n\
         1. Read the actual added and removed lines in the Diffs section before deciding the purpose.\n\
         2. Describe the concrete behavior change; do not merely repeat filenames, line counts, or \"update logic\".\n\
         3. The supplied context is authoritative and already limited for safety. Do not call shell when it contains enough detail.\n\
         4. If a relevant diff is explicitly marked omitted, truncated, or shown as hunk headers only, inspect only that file with `git diff --cached -- <path>`; never run an unscoped diff.\n\
         5. Return only the commit message. No Markdown or explanation.\n\
         \n\
         BEGIN SUPPLIED CHANGE CONTEXT\n\
         {diff_context}\n\
         END SUPPLIED CHANGE CONTEXT"
    )
}

fn read_diff_stat(repo_root: &str, cached: bool) -> Option<String> {
    let mut args = vec!["diff".to_string(), "--stat".to_string()];
    if cached {
        args.push("--cached".to_string());
    }
    let output = crate::session::git::git_command()
        .args(&args)
        .current_dir(repo_root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stat = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stat.is_empty() {
        None
    } else {
        Some(stat)
    }
}

fn detect_change_hints(changes: &[GitChangedFile]) -> Vec<String> {
    let mut hints = Vec::new();
    let total = changes.len();
    if total == 0 {
        return hints;
    }

    let all_new = changes
        .iter()
        .all(|c| matches!(c.status, GitFileStatus::Added | GitFileStatus::Untracked));
    if all_new {
        hints.push("Only new files added — likely a new feature or module".to_string());
    }

    let all_tests = changes.iter().all(|c| {
        let f = c.file.to_lowercase();
        f.contains("test") || f.contains("spec") || f.contains("__tests__")
    });
    if all_tests && total > 0 {
        hints.push("Only test files changed".to_string());
    }

    let all_config = changes.iter().all(|c| {
        let f = c.file.to_lowercase();
        f.starts_with(".")
            || f.contains("config")
            || f.contains("Makefile")
            || f.contains("Dockerfile")
            || f.contains("docker-compose")
            || f.contains("CI")
            || f.contains(".yml")
            || f.contains(".yaml")
            || f.contains(".toml")
            || f.contains(".json")
    });
    if all_config && total > 0 && !all_tests {
        hints.push("Only configuration/build files changed".to_string());
    }

    let has_renames = changes.iter().any(|c| {
        let f = &c.file;
        f.contains(" => ") || f.contains("{") || f.contains("}")
    });
    if has_renames {
        hints.push("File renames detected — likely a refactoring".to_string());
    }

    let deleted_count = changes
        .iter()
        .filter(|c| matches!(c.status, GitFileStatus::Deleted))
        .count();
    if deleted_count == total {
        hints.push("All files deleted — likely removing a feature or cleanup".to_string());
    }

    hints
}

fn file_priority(path: &str) -> u8 {
    let lower = path.to_lowercase();
    let lock_files = [
        "cargo.lock",
        "package-lock.json",
        "yarn.lock",
        "pnpm-lock.yaml",
        "go.sum",
        "composer.lock",
        "gemfile.lock",
        "poetry.lock",
    ];
    for lock in lock_files {
        if lower.ends_with(lock) {
            return 2; // low priority
        }
    }
    if lower.ends_with(".min.js") || lower.ends_with(".min.css") {
        return 2;
    }
    if lower.ends_with(".generated.")
        || lower.contains(".generated.")
        || lower.contains("autogenerated")
    {
        return 1;
    }
    0 // normal priority
}

const PREAMBLE_PATTERNS: &[&str] = &[
    "here is",
    "here's",
    "commit message:",
    "suggested commit message:",
    "proposed commit message:",
    "generated commit message:",
    "the commit message",
    "commit:\n",
    "```\n",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_language_instruction_auto_when_empty_or_auto() {
        // Empty or "auto" (any case, any surrounding whitespace) asks the model
        // to mirror language only as part of a consistent historical style.
        for value in ["", "auto", "AUTO", "  auto  "] {
            let instruction = git_language_instruction(value);
            assert!(
                instruction.starts_with(
                    "\n\nLanguage: when the recent commits establish a consistent historical style"
                ),
                "expected auto directive for {value:?}, got: {instruction:?}"
            );
            assert!(instruction.contains("use English with the fallback template"));
        }
    }

    #[test]
    fn git_language_instruction_uses_configured_language() {
        // The configured value is free-form text the user typed (a code or a
        // language name), so it is used verbatim — no mapping is applied.
        assert_eq!(
            git_language_instruction("en"),
            "\n\nLanguage: write the entire commit message (subject and body) in en."
        );
        assert_eq!(
            git_language_instruction("zh-CN"),
            "\n\nLanguage: write the entire commit message (subject and body) in zh-CN."
        );
        assert_eq!(
            git_language_instruction("English"),
            "\n\nLanguage: write the entire commit message (subject and body) in English."
        );
        assert_eq!(
            git_language_instruction("中文"),
            "\n\nLanguage: write the entire commit message (subject and body) in 中文."
        );
    }

    #[test]
    fn truncate_diff_preserves_changed_lines_and_context() {
        let input = [
            "diff --git a/file.rs b/file.rs",
            "index 111..222 100644",
            "--- a/file.rs",
            "+++ b/file.rs",
            "@@ -1,6 +1,6 @@",
            " unchanged 1",
            " unchanged 2",
            "-old",
            "+new",
            " unchanged 3",
            " unchanged 4",
        ]
        .join("\n");

        let (output, truncated) = truncate_diff_preserve_hunks(&input, 10);

        assert!(truncated);
        assert!(output.contains("@@ -1,6 +1,6 @@"));
        assert!(output.contains("-old"));
        assert!(output.contains("+new"));
        assert!(output.contains("context lines omitted"));
    }

    #[test]
    fn truncate_diff_keeps_context_around_changes() {
        let input = [
            "unchanged 1",
            "unchanged 2",
            "unchanged 3",
            "-removed",
            "+added",
            "unchanged 4",
            "unchanged 5",
            "unchanged 6",
        ]
        .join("\n");

        let (output, truncated) = truncate_diff_preserve_hunks(&input, 6);

        assert!(truncated);
        assert!(output.contains("-removed"));
        assert!(output.contains("+added"));
        // Should have some context nearby
        assert!(output.contains("unchanged 3") || output.contains("unchanged 4"));
    }

    #[test]
    fn truncate_diff_samples_later_hunks_in_large_files() {
        let input = [
            "diff --git a/file.rs b/file.rs",
            "index 111..222 100644",
            "--- a/file.rs",
            "+++ b/file.rs",
            "@@ -10,4 +10,4 @@ fn first()",
            "-first old 1",
            "+first new 1",
            "-first old 2",
            "+first new 2",
            "@@ -100,4 +100,4 @@ fn middle()",
            "-middle old 1",
            "+middle new 1",
            "-middle old 2",
            "+middle new 2",
            "@@ -1000,4 +1000,4 @@ fn last()",
            "-last old 1",
            "+last new 1",
            "-last old 2",
            "+last new 2",
        ]
        .join("\n");

        let (output, truncated) = truncate_diff_preserve_hunks(&input, 14);

        assert!(truncated);
        assert!(output.contains("fn first()"));
        assert!(output.contains("fn middle()"));
        assert!(output.contains("fn last()"));
        assert!(output.contains("first "));
        assert!(output.contains("middle "));
        assert!(output.contains("last "));
    }

    #[test]
    fn agent_prompt_includes_preloaded_diff_hunks() {
        let context = "Diffs:\n@@ -1 +1 @@\n-old behavior\n+new behavior";
        let prompt = build_agent_user_prompt("D:/repo", "windows", context);

        assert!(prompt.contains(context));
        assert!(prompt.contains("actual added and removed lines"));
        assert!(prompt.contains("do not merely repeat filenames, line counts"));
    }

    #[test]
    fn normalize_rejects_empty_and_overlong_subjects() {
        assert!(normalize_commit_message(Some(""), true).is_none());
        assert!(normalize_commit_message(Some(&"x".repeat(121)), true).is_none());
        assert_eq!(
            normalize_commit_message(Some("fix: update parser\n\nExplain why"), true).unwrap(),
            "fix: update parser\n\nExplain why"
        );
    }

    #[test]
    fn normalize_strips_think_tags() {
        let input = "<think\nreasoning here\n</think\nfix: update parser";
        assert_eq!(
            normalize_commit_message(Some(input), true).unwrap(),
            "fix: update parser"
        );
    }

    #[test]
    fn normalize_strips_preamble() {
        assert_eq!(
            normalize_commit_message(
                Some("Here is the commit message:\nfix: update parser"),
                true
            )
            .unwrap(),
            "fix: update parser"
        );
        assert_eq!(
            normalize_commit_message(Some("Commit message:\nfix: update parser"), true).unwrap(),
            "fix: update parser"
        );
    }

    #[test]
    fn normalize_strips_code_blocks() {
        assert_eq!(
            normalize_commit_message(Some("```\nfix: update parser\n```"), true).unwrap(),
            "fix: update parser"
        );
    }

    #[test]
    fn normalize_strips_tilde_fences() {
        assert_eq!(
            normalize_commit_message(Some("  ```\nfix: update parser\n```  "), true).unwrap(),
            "fix: update parser"
        );
    }

    #[test]
    fn normalize_strips_surrounding_quotes() {
        assert_eq!(
            normalize_commit_message(Some("\"fix: update parser\""), true).unwrap(),
            "fix: update parser"
        );
    }

    #[test]
    fn conventional_subject_accepts_repository_defined_types_and_scopes() {
        assert!(is_conventional_subject("feat(terminal): add idle delay"));
        assert!(is_conventional_subject("ci!: replace release workflow"));
        assert!(is_conventional_subject("build(android-app): update SDK"));
        assert!(!is_conventional_subject("Based on the analysis:"));
        assert!(!is_conventional_subject("commit message: fix parser"));
        assert!(!is_conventional_subject("feat:"));
    }

    #[test]
    fn normalize_extracts_commit_after_leaked_analysis() {
        let input = "\
Based on the analysis:

- **File changed:** `src/editor.rs`
- **Churn:** 355 insertions, 121 deletions

refactor(editor): overhaul code editor state management

Improve state handling and rendering stability.";

        assert_eq!(
            normalize_commit_message(Some(input), true).unwrap(),
            "refactor(editor): overhaul code editor state management\n\nImprove state handling and rendering stability."
        );
    }

    #[test]
    fn invalid_analysis_commit_is_not_a_style_reference() {
        // A commit message that starts with leaked analysis must not become a
        // style reference, even though its tail contains a valid subject.
        let input = "\
based on the staged changes:

feat(terminal): add idle analysis delay";
        assert!(!is_style_reference_candidate(input, 1));
        assert!(looks_like_leaked_analysis(
            "Based on the analysis of the diff:"
        ));
        assert!(!looks_like_leaked_analysis(
            "fix(editor): repair wrap movement"
        ));
        // Merge commits are branch bookkeeping, not style.
        assert!(!is_style_reference_candidate("Merge branch 'bugfixing'", 2));
        // Plain (non-Conventional) repo styles ARE valid references now.
        assert!(is_style_reference_candidate(
            "修复光标在折行时的移动问题",
            1
        ));
        assert!(is_style_reference_candidate(
            "Update parser to handle CRLF",
            1
        ));
        // Empty messages are skipped.
        assert!(!is_style_reference_candidate("   \n  ", 1));
    }

    #[test]
    fn recent_style_detection_uses_majority() {
        let conventional = vec![
            "fix(client): repair cursor".to_string(),
            "feat(server): add endpoint".to_string(),
        ];
        let plain = vec!["修复登录问题".to_string(), "更新依赖".to_string()];
        assert!(recent_style_is_conventional(&conventional));
        assert!(!recent_style_is_conventional(&plain));
        assert!(!recent_style_is_conventional(&[]));
        // Mixed: 3 conventional out of 5 → majority wins.
        let mixed = vec![
            "fix(a): one".to_string(),
            "修复问题".to_string(),
            "feat(b): two".to_string(),
            "chore(c): three".to_string(),
            "更新文档".to_string(),
        ];
        assert!(recent_style_is_conventional(&mixed));
        // 2 out of 5 is not a majority.
        let mixed = vec![
            "fix(a): one".to_string(),
            "修复问题".to_string(),
            "feat(b): two".to_string(),
            "更新文档".to_string(),
            "重构模块".to_string(),
        ];
        assert!(!recent_style_is_conventional(&mixed));
    }

    #[test]
    fn system_prompt_requires_consistent_recent_style_before_mirroring() {
        let recent = vec!["修复登录问题".to_string()];
        let mirror = git_system_prompt(&recent);
        assert!(mirror.contains("Examine the recent commit messages as a set"));
        assert!(mirror.contains("One sample alone is not enough"));
        assert!(mirror.contains("If a consistent historical style exists"));
        assert!(mirror.contains("you must also omit it"));
        assert!(mirror.contains("output only a subject"));
        assert!(mirror.contains("Fallback template"));
        assert!(!mirror.contains("Examples:"));

        // No reference → built-in Conventional Commits default with examples.
        let fallback = git_system_prompt(&[]);
        assert!(fallback.contains("Output format: Conventional Commits"));
        assert!(fallback.contains("Examples:"));
    }

    #[test]
    fn normalize_accepts_plain_subjects_in_mirror_mode() {
        // A repo whose commits are plain Chinese subjects: the mirrored
        // output must survive validation instead of being discarded for not
        // being Conventional Commits.
        assert_eq!(
            normalize_commit_message(Some("修复编辑器折行时的光标移动"), false).unwrap(),
            "修复编辑器折行时的光标移动"
        );
        assert_eq!(
            normalize_commit_message(Some("修复光标移动\n\n同时调整了测试"), false).unwrap(),
            "修复光标移动\n\n同时调整了测试"
        );
        // Hygiene still applies: empty and overlong subjects are rejected.
        assert!(normalize_commit_message(Some(""), false).is_none());
        assert!(normalize_commit_message(Some(&"x".repeat(121)), false).is_none());
        // Conventional mode no longer rejects non-Conventional subjects
        // either: the repository history style wins, and the Conventional
        // shape is only used to locate a subject buried under leaked
        // analysis.
        assert_eq!(
            normalize_commit_message(Some("修复光标移动"), true).unwrap(),
            "修复光标移动"
        );
    }

    #[test]
    fn normalize_preserves_subject_when_body_mentions_preamble_words() {
        // Regression: a valid Conventional subject whose body contains a
        // preamble pattern ("the commit message") used to be truncated and
        // the whole response discarded. Preamble matching is now anchored to
        // the start of the response only.
        let input = "\
feat: match draft language to the conversation and keep drafts concise

Draft replies and the commit message now follow the active conversation
language, keeping both drafts and messages concise.";

        assert_eq!(normalize_commit_message(Some(input), true).unwrap(), input);
    }

    #[test]
    fn normalize_strips_leading_preamble_before_code_fence() {
        // A preamble label followed by a fenced message is fully stripped,
        // including the dangling colon and the fence exposed by the strip.
        assert_eq!(
            normalize_commit_message(
                Some("Here is the commit message:\n```\nfix: update parser\n```"),
                true
            )
            .unwrap(),
            "fix: update parser"
        );
    }

    #[test]
    fn strip_preamble_is_safe_on_short_or_exact_labels() {
        // A response shorter than a pattern must not panic and must be
        // left untouched; an exact label with nothing after is kept as-is.
        assert_eq!(strip_preamble_owned("Here"), "Here");
        assert_eq!(strip_preamble_owned("commit message:"), "commit message:");
        // A short response that merely *starts like* a longer label is not
        // a label match.
        assert_eq!(
            strip_preamble_owned("Here comes the fix"),
            "Here comes the fix"
        );
    }

    #[test]
    fn normalize_never_discards_on_format_mismatch() {
        // Whatever format the repository history uses, a coherent response
        // must survive: no rejection for missing Conventional shape.
        assert_eq!(
            normalize_commit_message(Some("Update parser to handle CRLF"), true).unwrap(),
            "Update parser to handle CRLF"
        );
        assert_eq!(
            normalize_commit_message(Some("修复光标移动"), true).unwrap(),
            "修复光标移动"
        );
        // Leaked analysis is still drained when a Conventional subject is
        // appended after it.
        let input = "\
Analysis of the diff:\n- 12 files changed

fix(editor): repair wrap movement";
        assert_eq!(
            normalize_commit_message(Some(input), true).unwrap(),
            "fix(editor): repair wrap movement"
        );
    }

    #[test]
    fn default_recent_commits_is_five() {
        let values = std::collections::HashMap::new();
        let config = CommitAgentConfig::from_values(&values);
        assert_eq!(config.recent_commits_count, 5);
    }

    #[test]
    fn detect_change_hints_all_new_files() {
        let changes = vec![
            GitChangedFile {
                file: "a.rs".into(),
                status: GitFileStatus::Added,
                staged: false,
                is_submodule: false,
                submodule_dirty: false,
            },
            GitChangedFile {
                file: "b.rs".into(),
                status: GitFileStatus::Untracked,
                staged: false,
                is_submodule: false,
                submodule_dirty: false,
            },
        ];
        let hints = detect_change_hints(&changes);
        assert!(hints.iter().any(|h| h.contains("new feature")));
    }

    #[test]
    fn detect_change_hints_all_tests() {
        let changes = vec![GitChangedFile {
            file: "test_a.rs".into(),
            status: GitFileStatus::Modified,
            staged: false,
            is_submodule: false,
            submodule_dirty: false,
        }];
        let hints = detect_change_hints(&changes);
        assert!(hints.iter().any(|h| h.contains("test files")));
    }

    #[test]
    fn file_priority_lock_files() {
        assert_eq!(file_priority("Cargo.lock"), 2);
        assert_eq!(file_priority("package-lock.json"), 2);
        assert_eq!(file_priority("yarn.lock"), 2);
        assert_eq!(file_priority("src/main.rs"), 0);
    }

    #[test]
    fn file_priority_minified() {
        assert_eq!(file_priority("bundle.min.js"), 2);
        assert_eq!(file_priority("style.min.css"), 2);
    }

    #[test]
    fn looks_like_binary_diff_detects_binary_marker() {
        assert!(looks_like_binary_diff(
            "diff --git a/x.bin b/x.bin\nindex 1..2 100644\nBinary files a/x.bin and b/x.bin differ\n"
        ));
        assert!(!looks_like_binary_diff(
            "diff --git a/x.rs b/x.rs\n--- a/x.rs\n+++ b/x.rs\n@@ -1 +1 @@\n-old\n+new\n"
        ));
    }

    #[test]
    fn config_budget_derives_from_input_and_output_limits() {
        let mut values = std::collections::HashMap::new();
        values.insert("max_input_tokens".to_string(), "262144".to_string());
        values.insert("max_output_tokens".to_string(), "8192".to_string());
        values.insert("temperature".to_string(), "0.35".to_string());
        let config = CommitAgentConfig::from_values(&values);
        assert_eq!(config.max_input_tokens, 262_144);
        assert_eq!(config.max_output_tokens, 8_192);
        assert_eq!(config.input_budget_tokens, 260_096);
        assert_eq!(config.context_budget_chars, 1_040_384);
        assert!(config.validate_context_window().is_ok());
        assert_eq!(config_f32(&values, "temperature", 0.7), 0.35);
    }

    #[test]
    fn config_budget_uses_default_input_and_output_limits() {
        let values = std::collections::HashMap::new();
        let config = CommitAgentConfig::from_values(&values);
        assert_eq!(config.max_input_tokens, DEFAULT_MAX_INPUT_TOKENS);
        assert_eq!(config.max_output_tokens, DEFAULT_MAX_OUTPUT_TOKENS);
        assert_eq!(config.input_budget_tokens, 125_952);
        assert_eq!(config.context_budget_chars, 503_808);
        assert!(config.validate_context_window().is_ok());
        assert_eq!(config_f32(&values, "temperature", 0.7), 0.7);
    }

    #[test]
    fn config_rejects_input_limit_no_larger_than_prompt_reserve() {
        let mut values = std::collections::HashMap::new();
        values.insert("max_input_tokens".to_string(), "2048".to_string());
        let config = CommitAgentConfig::from_values(&values);

        assert_eq!(config.input_budget_tokens, 0);
        assert!(config.validate_context_window().is_err());
    }

    #[test]
    fn legacy_max_tokens_is_only_an_input_limit_fallback() {
        let mut values = std::collections::HashMap::new();
        values.insert("max_tokens".to_string(), "262144".to_string());
        let config = CommitAgentConfig::from_values(&values);

        assert_eq!(config.max_input_tokens, 262_144);
        assert_eq!(config.max_output_tokens, DEFAULT_MAX_OUTPUT_TOKENS);

        values.insert("max_input_tokens".to_string(), "65536".to_string());
        let config = CommitAgentConfig::from_values(&values);
        assert_eq!(config.max_input_tokens, 65_536);
    }

    #[test]
    fn config_rejects_output_limit_larger_than_protocol_field() {
        let mut values = std::collections::HashMap::new();
        values.insert(
            "max_output_tokens".to_string(),
            ((u32::MAX as u64) + 1).to_string(),
        );
        let config = CommitAgentConfig::from_values(&values);

        assert!(config.validate_context_window().is_err());
    }

    #[test]
    fn truncate_long_lines_caps_long_lines() {
        let long = "x".repeat(1000);
        let input = format!("short line\n{long}\ntrail no newline");
        let output = truncate_long_lines(&input, 400);
        let mut lines = output.lines();
        assert_eq!(lines.next(), Some("short line"));
        let capped = lines.next().unwrap();
        assert!(capped.ends_with("[... line truncated]"));
        assert_eq!(capped.len(), 400 + " [... line truncated]".len());
        assert_eq!(lines.next(), Some("trail no newline"));
    }

    #[test]
    fn truncate_long_lines_is_multibyte_safe() {
        // 500 CJK chars = 1500 bytes; a byte-based cut would split a char.
        let long = "汉".repeat(500);
        let output = truncate_long_lines(&long, 400);
        let capped = output.trim_end();
        assert!(capped.starts_with(&"汉".repeat(400)));
        assert!(capped.ends_with("[... line truncated]"));
    }

    #[test]
    fn truncate_long_lines_leaves_short_lines_untouched() {
        let input = "a\nb\n";
        assert_eq!(truncate_long_lines(input, 400), input);
    }

    #[test]
    fn hunk_headers_only_keeps_skeleton() {
        let input = [
            "diff --git a/file.rs b/file.rs",
            "index 111..222 100644",
            "--- a/file.rs",
            "+++ b/file.rs",
            "@@ -10,4 +10,4 @@ fn first()",
            "-old body line",
            "+new body line",
            " context line",
            "@@ -100,2 +100,2 @@ fn last()",
            "-old2",
            "+new2",
        ]
        .join("\n");
        let output = hunk_headers_only(&input);
        assert!(output.contains("diff --git a/file.rs b/file.rs"));
        assert!(output.contains("@@ -10,4 +10,4 @@ fn first()"));
        assert!(output.contains("@@ -100,2 +100,2 @@ fn last()"));
        assert!(!output.contains("-old body line"));
        assert!(!output.contains(" context line"));
    }

    #[test]
    fn hunk_headers_only_handles_empty_input() {
        assert_eq!(hunk_headers_only(""), "");
    }

    #[test]
    fn config_parses_new_knobs_with_defaults() {
        let values = std::collections::HashMap::new();
        let config = CommitAgentConfig::from_values(&values);
        assert_eq!(config.diff_context_lines, DEFAULT_DIFF_CONTEXT_LINES);
        assert_eq!(config.max_diff_line_chars, DEFAULT_MAX_DIFF_LINE_CHARS);
        assert_eq!(config.max_full_diff_files, DEFAULT_MAX_FULL_DIFF_FILES);
        assert_eq!(config.max_iterations, DEFAULT_MAX_ITERATIONS);

        let mut values = std::collections::HashMap::new();
        values.insert("max_diff_line_chars".to_string(), "120".to_string());
        values.insert("max_full_diff_files".to_string(), "5".to_string());
        values.insert("max_iterations".to_string(), "3".to_string());
        let config = CommitAgentConfig::from_values(&values);
        assert_eq!(config.max_diff_line_chars, 120);
        assert_eq!(config.max_full_diff_files, 5);
        assert_eq!(config.max_iterations, 3);

        // Invalid or zero values fall back to defaults.
        let mut values = std::collections::HashMap::new();
        values.insert("max_diff_line_chars".to_string(), "abc".to_string());
        values.insert("max_full_diff_files".to_string(), "0".to_string());
        let config = CommitAgentConfig::from_values(&values);
        assert_eq!(config.max_diff_line_chars, DEFAULT_MAX_DIFF_LINE_CHARS);
        assert_eq!(config.max_full_diff_files, DEFAULT_MAX_FULL_DIFF_FILES);
    }
}
