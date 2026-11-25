//! AI agent that resolves rebase conflict hunks one at a time.
//!
//! The client computes the diff3 chunk list of each conflicted file itself
//! and hands this agent the still-unresolved conflict hunks (base/ours/theirs
//! line arrays plus an optional free-text instruction). Each hunk is resolved
//! by a small `vterm-agent` tool loop: the model may inspect the repository
//! through a restricted read-only tool set (the incoming commit's message
//! and diff via REBASE_HEAD, file reads, git grep) before answering. The run
//! streams each outcome back as a `GitRebaseAiResolveResult` event: `running`
//! when analysis of a hunk starts, `done` with the proposed replacement
//! `lines` plus a human-facing `explanation` and `confidence`, or `error`
//! when the hunk could not be resolved (the run then continues with the
//! next hunk).
//!
//! The agent never rewrites whole files, never writes or stages anything:
//! applying a resolution (write + `git add`) stays with the client-driven
//! `GitRebaseResolve` message, and `rebase --continue/--skip/--abort` remain
//! user-controlled. Cancellation is cooperative: the run finishes the hunk
//! currently being analyzed, then stops and sends the final `done` event;
//! resolutions already streamed stay valid, and the client resumes by
//! re-requesting the still-unresolved hunks.

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tracing::{info, warn};
use vterm_agent::agent::{Agent, AgentConfig, LoopEvent, LoopResult};
use vterm_agent::memory::CompressionConfig;
use vterm_common::{GitRebaseAiHunk, GitRebaseAiHunkEvent, GitRebaseAiTarget, SessionId};
use vterm_core::protocol::Message;

use crate::addon::{
    resolve_addon_config, resolve_llm_addon_id, run_chat_completion, AddonHost, LlmRequestContext,
};
use crate::config::TerminalServerConfig;
use crate::session::session_core::ConnectionTx;

use super::agent_client::CallbackLlmClient;
use super::repo_tools;

const DEFAULT_TIMEOUT_SECONDS: u64 = 60;
const DEFAULT_OUTPUT_MAX_TOKENS: usize = 4096;
/// Default model context window; the agent loop's pre-flight check reserves
/// input + output against `max_input_tokens + max_output_tokens`.
const DEFAULT_MAX_INPUT_TOKENS: usize = 128_000;
/// Sanity cap on the output budget; reasoning models legitimately want
/// tens of thousands of tokens, so the cap stays generous.
const MAX_OUTPUT_MAX_TOKENS: usize = 131_072;
/// A hunk with any side larger than this is refused up front: it would blow
/// up the prompt, and the model cannot reliably reproduce that many lines
/// verbatim anyway. The user resolves such hunks manually.
const MAX_HUNK_LINES_PER_SIDE: usize = 400;
/// Sanity cap on the resolution itself, against runaway model output.
const MAX_RESOLUTION_LINES: usize = 2000;
const MAX_ATTEMPTS_PER_HUNK: usize = 2;
/// Tool-loop turns one hunk resolution may take (each turn is one LLM call
/// plus its tool executions). Kept small: the hunk text is already in the
/// prompt, so the tools only fill in missing context.
const MAX_ITERATIONS_PER_HUNK: usize = 8;

const REBASE_AGENT_SYSTEM_PROMPT: &str = "\
You resolve one git conflict hunk at a time during an interactive rebase.\n\
During a rebase, \"ours\" is the branch being rebased ONTO (the target) and \
\"theirs\" is the incoming commit being replayed.\n\
\n\
Given the base/ours/theirs sections of one conflict hunk, produce the final \
replacement lines for that hunk.\n\
Rules:\n\
1. Preserve the intent of both sides whenever the changes can coexist; drop a \
side only when the changes fundamentally contradict each other or the user \
instruction says so.\n\
2. Never emit conflict markers (<<<<<<<, |||||||, =======, >>>>>>>) or diff \
headers.\n\
3. Keep the code syntactically valid: match the indentation, braces, and \
style of the hunk.\n\
4. Honor the user instruction when it conflicts with the defaults above.\n\
\n\
Respond with ONLY a JSON object (no markdown fences, no commentary):\n\
{\"lines\": [\"..replacement lines..\"], \"strategy\": \
\"ours|theirs|union|custom\", \"explanation\": \"1-3 sentences\", \
\"confidence\": 0.0}\n\
- lines: the replacement lines for the whole hunk, without line terminators \
(an empty array removes the hunk entirely).\n\
- strategy: \"ours\" = kept the target side, \"theirs\" = took the incoming \
side, \"union\" = kept both, \"custom\" = anything else.\n\
- explanation: why this resolution is correct, written for the user who \
reviews it.\n\
- confidence: 0.0-1.0, how certain you are the resolution preserves behavior \
(1.0 = certain).";

const REBASE_AGENT_TOOLS_PROMPT: &str = "\
\n\
Context tools (read-only — they never modify the repository):\n\
- git_commit_info / git_commit_diff: the commit being replayed (default rev \
is REBASE_HEAD during a rebase) — its message and patch explain the intent \
of the theirs side.\n\
- git_repo_read / git_repo_search: surrounding code and related definitions \
or usages.\n\
Use them when the hunk text alone is not enough to resolve confidently. When \
you are ready, answer with the JSON object and make no further tool calls.";

/// Resolve every target hunk with the git-conflict LLM, streaming one
/// `GitRebaseAiResolveResult` event per hunk state change and a final
/// `done = true` message when the run ends (completed, cancelled, or a
/// run-level fatal error). Each hunk is resolved by a small agent loop with
/// read-only repository tools; spawn this on the tokio runtime (the blocking
/// addon calls are wrapped in `spawn_blocking` by the client adapter).
pub(crate) async fn git_rebase_ai_resolve_streaming(
    session_id: SessionId,
    run_id: u32,
    instruction: &str,
    ours_label: Option<&str>,
    theirs_label: Option<&str>,
    targets: &[GitRebaseAiTarget],
    repo_root: &str,
    addon_host: Arc<AddonHost>,
    config: &TerminalServerConfig,
    cancellation: &Arc<AtomicBool>,
    tx: &ConnectionTx,
) {
    let total_hunks: usize = targets.iter().map(|target| target.hunks.len()).sum();

    let addon_id = match resolve_llm_addon_id(config, "git_conflict") {
        Some(addon_id) => addon_id,
        None => {
            warn!("[rebase-ai] Run {run_id}: git conflict LLM is not configured");
            send_run_result(
                tx,
                session_id,
                run_id,
                Some("git conflict LLM is not configured".to_string()),
            );
            return;
        }
    };

    // The tool path sandbox compares canonicalized paths against the root,
    // so the root itself must be canonical (Windows: canonicalize adds the
    // \\?\ prefix that plain absolute paths lack).
    let repo_root = match Path::new(repo_root).canonicalize() {
        Ok(root) => root,
        Err(error) => {
            warn!("[rebase-ai] Run {run_id}: cannot canonicalize repo root `{repo_root}`: {error}");
            send_run_result(
                tx,
                session_id,
                run_id,
                Some(format!("repository path unavailable: {error}")),
            );
            return;
        }
    };

    let config_overrides = config.llm.git_conflict.provider.user_config.clone();
    // Feature-level Git settings override the addon's global settings, but
    // omitted fields still inherit from the addon — mirrors the commit
    // agent's merged_llm_config, so values configured on either the addon
    // page or the Git feature page apply.
    let mut config_values = resolve_addon_config(config, &addon_id, None);
    config_values.extend(resolve_addon_config(
        config,
        &addon_id,
        Some(&config_overrides),
    ));
    let model = config_values.get("model").cloned().unwrap_or_default();
    // Verbose switch: log the prompts sent to the model, its per-iteration
    // content, and every tool call at info level.
    let debug_logging = config_flag(&config_values, "rebase_debug_logging");
    // Feature-specific keys win; fall back to the shared git-LLM keys the
    // commit agent already exposes in the config UI.
    let max_tokens = config_value_usize_any(
        &config_values,
        &["rebase_output_max_tokens", "max_output_tokens"],
        DEFAULT_OUTPUT_MAX_TOKENS,
    )
    .min(MAX_OUTPUT_MAX_TOKENS) as u32;
    // The agent loop hard-checks `estimated input + reserved output` against
    // the context window before every call, so the window must cover both
    // sides — mirrors the commit agent's max_input_tokens + max_output_tokens.
    let max_input_tokens = config_value_usize_any(
        &config_values,
        &["max_input_tokens"],
        DEFAULT_MAX_INPUT_TOKENS,
    );
    // Resolve the effective request timeout with the same precedence the
    // executor applies: the addon's own `timeout` setting (Git feature
    // page, then addon page), then the addon manifest default, and only
    // then the shared git-LLM fallback key.
    let timeout = Duration::from_secs(
        config_values
            .get("timeout")
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|secs| *secs > 0)
            .or_else(|| {
                addon_host
                    .config_field_default(&addon_id, "timeout")
                    .and_then(|value| value.trim().parse::<u64>().ok())
                    .filter(|secs| *secs > 0)
            })
            .unwrap_or_else(|| {
                config_value_usize_any(
                    &config_values,
                    &["llm_timeout_seconds"],
                    DEFAULT_TIMEOUT_SECONDS as usize,
                ) as u64
            }),
    );

    info!(
        "[rebase-ai] Starting AI resolve run {run_id} (session={session_id}): {} files, {} hunks (addon={addon_id}, model={}, timeout={}s, max_tokens={max_tokens})",
        targets.len(),
        total_hunks,
        if model.is_empty() { "<addon default>" } else { model.as_str() },
        timeout.as_secs(),
    );

    let system_prompt = build_system_prompt(config.llm.git_conflict.output_language.trim());
    if debug_logging {
        info!("[rebase-ai] Run {run_id} system prompt:\n{system_prompt}");
    }

    let callback_host = addon_host.clone();
    let callback_config = config.clone();
    let callback_addon_id = addon_id.clone();
    let callback_values = config_values.clone();
    let callback = move |messages_json: &str, options_json: &str| -> Result<String, String> {
        let stats_context = LlmRequestContext {
            category: "rebase ai resolve".to_string(),
        };
        let result = run_chat_completion(
            &callback_host,
            &callback_config,
            &callback_addon_id,
            messages_json,
            options_json,
            Some(&callback_values),
            Some(timeout),
            Some(&stats_context),
        );
        if !result.success {
            return Err(result
                .error
                .unwrap_or_else(|| "unknown LLM error".to_string()));
        }
        result
            .raw_body
            .ok_or_else(|| "no response body".to_string())
    };
    let agent = Agent::new(
        Arc::new(CallbackLlmClient::new(Box::new(callback))),
        Arc::new(repo_tools::read_only_repo_tools(repo_root)),
        AgentConfig {
            max_iterations: MAX_ITERATIONS_PER_HUNK,
            model: model.clone(),
            temperature: Some(0.1),
            max_tokens: Some(max_tokens),
            compression: CompressionConfig {
                max_context_tokens: max_input_tokens.saturating_add(max_tokens as usize),
                ..CompressionConfig::default()
            },
        },
    );

    let mut resolved_count = 0usize;
    let mut failed_count = 0usize;
    let mut handled = 0usize;

    for target in targets {
        for hunk in &target.hunks {
            handled += 1;
            if cancellation.load(Ordering::SeqCst) {
                info!(
                    "[rebase-ai] Run cancelled at hunk {}/{} ({} resolved, {} failed)",
                    handled, total_hunks, resolved_count, failed_count
                );
                send_run_result(tx, session_id, run_id, None);
                return;
            }

            let path = target.path.as_str();
            send_hunk_event(
                tx,
                session_id,
                run_id,
                GitRebaseAiHunkEvent {
                    path: path.to_string(),
                    idx: hunk.idx,
                    status: "running".to_string(),
                    lines: None,
                    strategy: None,
                    explanation: None,
                    confidence: None,
                    error: None,
                },
            );

            if let Some(error) = hunk_size_error(hunk) {
                warn!(
                    "[rebase-ai] Skipping hunk {} of {}: {}",
                    hunk.idx, path, error
                );
                failed_count += 1;
                send_hunk_event(
                    tx,
                    session_id,
                    run_id,
                    hunk_error_event(path, hunk.idx, error),
                );
                continue;
            }

            info!(
                "[rebase-ai] Resolving hunk {} of {} (base {} / ours {} / theirs {} lines)",
                hunk.idx,
                path,
                hunk.base.len(),
                hunk.ours.len(),
                hunk.theirs.len()
            );
            let user_prompt =
                build_hunk_user_prompt(path, hunk, instruction, ours_label, theirs_label);

            let mut resolution = None;
            let mut last_error = "no response from LLM".to_string();
            let mut last_preview: Option<String> = None;
            let mut cancelled = false;
            for attempt in 1..=MAX_ATTEMPTS_PER_HUNK {
                if cancellation.load(Ordering::SeqCst) {
                    cancelled = true;
                    break;
                }
                // Retries get an explicit JSON-only nudge for chatty models.
                let attempt_prompt = if attempt > 1 {
                    format!(
                        "{user_prompt}\n\nReminder: respond with ONLY the JSON object — no prose, no markdown fences, no tool calls."
                    )
                } else {
                    user_prompt.clone()
                };
                if debug_logging {
                    info!(
                        "[rebase-ai] Run {run_id} hunk {} of {} attempt {}/{} user prompt:\n{}",
                        hunk.idx, path, attempt, MAX_ATTEMPTS_PER_HUNK, attempt_prompt
                    );
                }
                let started = Instant::now();
                let agent_session = agent
                    .create_session(system_prompt.as_str(), attempt_prompt)
                    .await;
                let loop_result = agent
                    .run_loop(&agent_session, |event| {
                        log_loop_event(run_id, path, hunk.idx, debug_logging, event);
                    })
                    .await;
                let elapsed_ms = started.elapsed().as_millis();
                match loop_result {
                    LoopResult::Completed {
                        final_content,
                        iterations,
                        ..
                    } => {
                        info!(
                            "[rebase-ai] Run {run_id} hunk {} of {} attempt {}/{}: agent loop completed ({iterations} iterations, {elapsed_ms}ms)",
                            hunk.idx,
                            path,
                            attempt,
                            MAX_ATTEMPTS_PER_HUNK
                        );
                        match parse_hunk_resolution(&final_content) {
                            Ok(parsed) => {
                                resolution = Some(parsed);
                                break;
                            }
                            Err(error) => {
                                last_error = error;
                                // Keep the raw answer visible in the log —
                                // "no JSON object" alone is undiagnosable.
                                last_preview = Some(response_preview(&final_content, 400));
                            }
                        }
                    }
                    LoopResult::MaxIterationsExceeded {
                        last_content,
                        iterations,
                        ..
                    } => {
                        last_error = format!(
                            "agent exceeded {iterations} iterations without a final answer"
                        );
                        last_preview = Some(response_preview(&last_content, 400));
                    }
                    LoopResult::Cancelled { .. } => {
                        cancelled = true;
                        break;
                    }
                    LoopResult::Error { error, .. } => {
                        last_error = format!("agent error: {error}");
                        last_preview = None;
                    }
                }
                warn!(
                    "[rebase-ai] Hunk {} of {} attempt {}/{} failed: {}{}",
                    hunk.idx,
                    path,
                    attempt,
                    MAX_ATTEMPTS_PER_HUNK,
                    last_error,
                    last_preview
                        .as_deref()
                        .map(|preview| format!("; response preview: {preview}"))
                        .unwrap_or_default(),
                );
            }

            if cancelled {
                // Stop without reporting the interrupted hunk; the final
                // done event tells the client the run is over.
                info!(
                    "[rebase-ai] Run cancelled during hunk {}/{} ({} resolved, {} failed)",
                    handled, total_hunks, resolved_count, failed_count
                );
                send_run_result(tx, session_id, run_id, None);
                return;
            }

            match resolution {
                Some(parsed) => {
                    resolved_count += 1;
                    info!(
                        "[rebase-ai] Hunk {} of {} resolved: strategy={} confidence={:.2} lines={} explanation={:?}",
                        hunk.idx,
                        path,
                        parsed.strategy,
                        parsed.confidence,
                        parsed.lines.len(),
                        parsed.explanation,
                    );
                    send_hunk_event(
                        tx,
                        session_id,
                        run_id,
                        GitRebaseAiHunkEvent {
                            path: path.to_string(),
                            idx: hunk.idx,
                            status: "done".to_string(),
                            lines: Some(parsed.lines),
                            strategy: Some(parsed.strategy),
                            explanation: Some(parsed.explanation),
                            confidence: Some(parsed.confidence),
                            error: None,
                        },
                    );
                }
                None => {
                    failed_count += 1;
                    send_hunk_event(
                        tx,
                        session_id,
                        run_id,
                        hunk_error_event(
                            path,
                            hunk.idx,
                            format!("AI resolution failed: {last_error}"),
                        ),
                    );
                }
            }
        }
    }

    info!(
        "[rebase-ai] Run finished: {} resolved, {} failed out of {} hunks",
        resolved_count, failed_count, total_hunks
    );
    send_run_result(tx, session_id, run_id, None);
}

#[derive(Debug)]
struct ParsedResolution {
    lines: Vec<String>,
    strategy: String,
    explanation: String,
    confidence: f32,
}

fn send_hunk_event(
    tx: &ConnectionTx,
    session_id: SessionId,
    run_id: u32,
    event: GitRebaseAiHunkEvent,
) {
    let summary = format!("{}#{} status={}", event.path, event.idx, event.status);
    let result = tx.send(Message::GitRebaseAiResolveResult {
        session_id,
        run_id,
        hunk: Some(event),
        error: None,
        done: false,
    });
    match result {
        Ok(()) => info!("[rebase-ai] Run {run_id}: sent hunk event to client ({summary})"),
        // A closed channel means every event of this run is silently lost —
        // the client never updates. Make that loud.
        Err(error) => warn!(
            "[rebase-ai] Run {run_id}: failed to deliver hunk event ({summary}) — client channel closed: {error}"
        ),
    }
}

fn send_run_result(tx: &ConnectionTx, session_id: SessionId, run_id: u32, error: Option<String>) {
    let result = tx.send(Message::GitRebaseAiResolveResult {
        session_id,
        run_id,
        hunk: None,
        error: error.clone(),
        done: true,
    });
    match result {
        Ok(()) => info!(
            "[rebase-ai] Run {run_id}: sent run-finished event to client (error={})",
            error.as_deref().unwrap_or("<none>")
        ),
        Err(send_error) => warn!(
            "[rebase-ai] Run {run_id}: failed to deliver run-finished event (error={error:?}) — client channel closed: {send_error}"
        ),
    }
}

fn hunk_error_event(path: &str, idx: u32, error: String) -> GitRebaseAiHunkEvent {
    GitRebaseAiHunkEvent {
        path: path.to_string(),
        idx,
        status: "error".to_string(),
        lines: None,
        strategy: None,
        explanation: None,
        confidence: None,
        error: Some(error),
    }
}

/// Verbose per-iteration tracing for `rebase_debug_logging`: the model's
/// step-by-step content and every tool call with its (truncated) result.
fn log_loop_event(run_id: u32, path: &str, idx: u32, enabled: bool, event: LoopEvent) {
    if !enabled {
        return;
    }
    match event {
        LoopEvent::IterationStart { iteration } => {
            info!("[rebase-ai] Run {run_id} hunk {idx} of {path} iteration {iteration}: start");
        }
        LoopEvent::LlmResponse {
            iteration,
            content,
            tool_call_count,
        } => {
            info!(
                "[rebase-ai] Run {run_id} hunk {idx} of {path} iteration {iteration}: LLM response ({tool_call_count} tool calls):\n{}",
                content.unwrap_or_else(|| "<no content>".to_string())
            );
        }
        LoopEvent::ToolExecuted {
            iteration,
            tool_name,
            result,
        } => {
            info!(
                "[rebase-ai] Run {run_id} hunk {idx} of {path} iteration {iteration}: tool {tool_name} (error={}):\n{}",
                result.is_error,
                response_preview(&result.content, 2000)
            );
        }
        LoopEvent::Compressed {
            messages_before,
            messages_after,
        } => {
            info!(
                "[rebase-ai] Run {run_id} hunk {idx} of {path}: context compressed {messages_before} -> {messages_after} messages"
            );
        }
        LoopEvent::Cancelled { iteration, .. } => {
            info!("[rebase-ai] Run {run_id} hunk {idx} of {path} iteration {iteration}: loop cancelled");
        }
    }
}

fn build_system_prompt(output_language: &str) -> String {
    let mut prompt = REBASE_AGENT_SYSTEM_PROMPT.to_string();
    prompt.push_str(REBASE_AGENT_TOOLS_PROMPT);
    let normalized = output_language.trim();
    if normalized.is_empty() || normalized.eq_ignore_ascii_case("auto") {
        prompt.push_str(
            "\nLanguage: write the explanation in the same language as the user instruction; with no instruction, use English.",
        );
    } else {
        // The value is user-entered free text — it may be a code ("en",
        // "zh-CN") or a natural-language name ("English", "中文"). Use it
        // verbatim so whatever the user typed is what the model sees.
        prompt.push_str(&format!(
            "\nLanguage: write the explanation in {normalized}."
        ));
    }
    prompt.push_str(
        " Never translate the resolution lines themselves — they are file content and must stay byte-exact.",
    );
    prompt
}

fn build_hunk_user_prompt(
    path: &str,
    hunk: &GitRebaseAiHunk,
    instruction: &str,
    ours_label: Option<&str>,
    theirs_label: Option<&str>,
) -> String {
    let mut prompt = format!(
        "Resolve this conflict hunk in file `{path}` (hunk {}).\n",
        hunk.idx
    );
    match (ours_label, theirs_label) {
        (Some(ours), Some(theirs)) => {
            prompt.push_str(&format!("Side labels: ours = {ours}, theirs = {theirs}.\n"));
        }
        (Some(ours), None) => {
            prompt.push_str(&format!("Side label: ours = {ours}.\n"));
        }
        (None, Some(theirs)) => {
            prompt.push_str(&format!("Side label: theirs = {theirs}.\n"));
        }
        (None, None) => {}
    }
    let instruction = instruction.trim();
    if !instruction.is_empty() {
        prompt.push_str(&format!("User instruction: {instruction}\n"));
    }
    prompt.push_str("\nBase (common ancestor):\n```text\n");
    push_side(&mut prompt, &hunk.base);
    prompt.push_str("```\n\nOurs (target branch side):\n```text\n");
    push_side(&mut prompt, &hunk.ours);
    prompt.push_str("```\n\nTheirs (incoming commit side):\n```text\n");
    push_side(&mut prompt, &hunk.theirs);
    prompt.push_str("```\n");
    prompt
}

fn push_side(prompt: &mut String, lines: &[String]) {
    if lines.is_empty() {
        prompt.push_str("(empty)\n");
    } else {
        for line in lines {
            prompt.push_str(line);
            prompt.push('\n');
        }
    }
}

fn hunk_size_error(hunk: &GitRebaseAiHunk) -> Option<String> {
    for (side, lines) in [
        ("base", &hunk.base),
        ("ours", &hunk.ours),
        ("theirs", &hunk.theirs),
    ] {
        if lines.len() > MAX_HUNK_LINES_PER_SIDE {
            return Some(format!(
                "hunk too large for AI resolution ({} side has {} lines, max {})",
                side,
                lines.len(),
                MAX_HUNK_LINES_PER_SIDE
            ));
        }
    }
    None
}

/// Parse the model's JSON response into a validated resolution.
fn parse_hunk_resolution(content: &str) -> Result<ParsedResolution, String> {
    let text = strip_think_tags(content);
    if text.trim().is_empty() {
        // Typical cause: a reasoning model spent the whole output budget
        // on its think block, leaving no room for the actual answer.
        return Err("model returned an empty response (reasoning may have exhausted the output budget — raise max_output_tokens)".to_string());
    }
    let start = text.find('{');
    let end = text.rfind('}');
    let (start, end) =
        match (start, end) {
            (Some(start), Some(end)) if end >= start => (start, end),
            // An opening brace without a closing one means the model ran out
            // of output tokens mid-JSON.
            (Some(_), _) => return Err(
                "response JSON is truncated (no closing brace — raise rebase_output_max_tokens?)"
                    .to_string(),
            ),
            _ => return Err("response contains no JSON object".to_string()),
        };
    let value: serde_json::Value = serde_json::from_str(&text[start..=end])
        .map_err(|error| format!("invalid resolution JSON: {error}"))?;

    let lines_value = value
        .get("lines")
        .ok_or_else(|| "resolution JSON has no \"lines\"".to_string())?;
    let lines_array = lines_value
        .as_array()
        .ok_or_else(|| "resolution \"lines\" is not an array".to_string())?;
    if lines_array.len() > MAX_RESOLUTION_LINES {
        return Err(format!(
            "resolution has {} lines (max {})",
            lines_array.len(),
            MAX_RESOLUTION_LINES
        ));
    }
    let mut lines = Vec::with_capacity(lines_array.len());
    for line in lines_array {
        let line = line
            .as_str()
            .ok_or_else(|| "resolution \"lines\" must contain only strings".to_string())?;
        if is_conflict_marker_line(line) {
            return Err("resolution still contains conflict markers".to_string());
        }
        lines.push(line.to_string());
    }

    let strategy_raw = value
        .get("strategy")
        .and_then(|value| value.as_str())
        .unwrap_or("custom")
        .to_ascii_lowercase();
    let strategy = match strategy_raw.as_str() {
        "ours" | "theirs" | "union" | "custom" => strategy_raw,
        _ => "custom".to_string(),
    };

    let explanation = value
        .get("explanation")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    let confidence = value
        .get("confidence")
        .and_then(|value| value.as_f64())
        .map(|value| (value as f32).clamp(0.0, 1.0))
        .unwrap_or(0.5);

    Ok(ParsedResolution {
        lines,
        strategy,
        explanation,
        confidence,
    })
}

fn is_conflict_marker_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("<<<<<<<")
        || trimmed.starts_with("|||||||")
        || trimmed.starts_with(">>>>>>>")
        || trimmed == "======="
}

/// Remove `<think>...</think>` reasoning blocks some models emit before the
/// actual answer, so JSON extraction does not trip over their content.
fn strip_think_tags(content: &str) -> String {
    let mut result = content.to_string();
    while let Some(start) = result.find("<think") {
        let Some(open_end) = result[start..].find('>').map(|i| start + i + 1) else {
            break;
        };
        match result[open_end..].find("</think") {
            Some(close_offset) => {
                let close_start = open_end + close_offset;
                let close_end = result[close_start..]
                    .find('>')
                    .map(|i| close_start + i + 1)
                    .unwrap_or(result.len());
                result.replace_range(start..close_end, "");
            }
            None => result.truncate(start),
        }
    }
    result
}

/// First `max_chars` of an LLM response (char-boundary safe), for failure
/// logs.
fn response_preview(content: &str, max_chars: usize) -> String {
    let preview: String = content.chars().take(max_chars).collect();
    if preview.len() < content.len() {
        format!("{preview}… ({} bytes total)", content.len())
    } else {
        preview
    }
}

/// First `keys` present in `values` with a positive integer wins.
fn config_value_usize_any(
    values: &HashMap<String, String>,
    keys: &[&str],
    default_value: usize,
) -> usize {
    keys.iter()
        .find_map(|key| {
            values
                .get(*key)
                .and_then(|value| value.trim().parse::<usize>().ok())
                .filter(|value| *value > 0)
        })
        .unwrap_or(default_value)
}

/// Truthy config flag: "1", "true", "yes", "on" (case-insensitive).
fn config_flag(values: &HashMap<String, String>, key: &str) -> bool {
    values
        .get(key)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hunk(base: &[&str], ours: &[&str], theirs: &[&str]) -> GitRebaseAiHunk {
        GitRebaseAiHunk {
            idx: 3,
            base: base.iter().map(|line| line.to_string()).collect(),
            ours: ours.iter().map(|line| line.to_string()).collect(),
            theirs: theirs.iter().map(|line| line.to_string()).collect(),
        }
    }

    #[test]
    fn parses_clean_resolution_json() {
        let parsed = parse_hunk_resolution(
            r#"{"lines": ["a", "b"], "strategy": "union", "explanation": "kept both", "confidence": 0.9}"#,
        )
        .expect("valid resolution");
        assert_eq!(parsed.lines, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(parsed.strategy, "union");
        assert_eq!(parsed.explanation, "kept both");
        assert!((parsed.confidence - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn parses_resolution_inside_prose_and_fences() {
        let content = "Here is the resolution:\n```json\n{\"lines\": [], \"strategy\": \"theirs\"}\n```\nDone.";
        let parsed = parse_hunk_resolution(content).expect("valid resolution");
        assert!(parsed.lines.is_empty());
        assert_eq!(parsed.strategy, "theirs");
        // Defaults when fields are absent.
        assert_eq!(parsed.explanation, "");
        assert!((parsed.confidence - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn ignores_think_blocks() {
        let content = "<think>let me think about { braces } here</think>{\"lines\": [\"x\"], \"strategy\": \"ours\", \"confidence\": 2.5}";
        let parsed = parse_hunk_resolution(content).expect("valid resolution");
        assert_eq!(parsed.lines, vec!["x".to_string()]);
        // Confidence is clamped into 0.0..=1.0.
        assert!((parsed.confidence - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn normalizes_unknown_strategy_to_custom() {
        let parsed = parse_hunk_resolution(r#"{"lines": ["x"], "strategy": "Weird"}"#)
            .expect("valid resolution");
        assert_eq!(parsed.strategy, "custom");
    }

    #[test]
    fn rejects_response_without_json() {
        assert!(parse_hunk_resolution("no json here").is_err());
        assert!(parse_hunk_resolution("{\"lines\": [1]}").is_err());
        assert!(parse_hunk_resolution("{\"other\": []}").is_err());
    }

    #[test]
    fn truncated_json_gets_a_distinct_error() {
        let error = parse_hunk_resolution("{\"lines\": [\"a\"").unwrap_err();
        assert!(error.contains("truncated"));
    }

    #[test]
    fn empty_and_think_only_responses_get_a_distinct_error() {
        let error = parse_hunk_resolution("").unwrap_err();
        assert!(error.contains("empty response"));
        let error = parse_hunk_resolution("<think>reasoning only, no answer</think>").unwrap_err();
        assert!(error.contains("empty response"));
    }

    #[test]
    fn response_preview_is_char_boundary_safe() {
        assert_eq!(response_preview("short", 10), "short");
        assert_eq!(
            response_preview("无法访问文件", 3),
            "无法访… (18 bytes total)"
        );
    }

    #[test]
    fn rejects_leftover_conflict_markers() {
        let content = r#"{"lines": ["<<<<<<< HEAD", "x"], "strategy": "ours"}"#;
        let error = parse_hunk_resolution(content).unwrap_err();
        assert!(error.contains("conflict markers"));

        let content = r#"{"lines": ["======="], "strategy": "ours"}"#;
        assert!(parse_hunk_resolution(content).is_err());
    }

    #[test]
    fn rejects_oversized_resolution() {
        let lines: Vec<String> = (0..MAX_RESOLUTION_LINES + 1)
            .map(|i| i.to_string())
            .collect();
        let content = serde_json::json!({ "lines": lines }).to_string();
        assert!(parse_hunk_resolution(&content).is_err());
    }

    #[test]
    fn marker_detection_matches_conflict_marker_shapes() {
        assert!(is_conflict_marker_line("<<<<<<< HEAD"));
        assert!(is_conflict_marker_line("  >>>>>>> branch"));
        assert!(is_conflict_marker_line("||||||| base"));
        assert!(is_conflict_marker_line("======="));
        assert!(!is_conflict_marker_line("========"));
        assert!(!is_conflict_marker_line("if (a == b) {"));
    }

    #[test]
    fn config_flag_accepts_common_truthy_values() {
        let values: HashMap<String, String> = [("k".to_string(), "true".to_string())]
            .into_iter()
            .collect();
        assert!(config_flag(&values, "k"));
        assert!(!config_flag(&values, "missing"));
        for truthy in ["1", "true", "TRUE", " yes ", "on"] {
            let values: HashMap<String, String> = [("k".to_string(), truthy.to_string())]
                .into_iter()
                .collect();
            assert!(config_flag(&values, "k"), "{truthy} should be truthy");
        }
        for falsy in ["0", "false", "off", "no", ""] {
            let values: HashMap<String, String> =
                [("k".to_string(), falsy.to_string())].into_iter().collect();
            assert!(!config_flag(&values, "k"), "{falsy} should be falsy");
        }
    }

    #[test]
    fn size_guard_flags_oversized_hunks() {
        let big: Vec<&str> = std::iter::repeat("x")
            .take(MAX_HUNK_LINES_PER_SIDE + 1)
            .collect();
        assert!(hunk_size_error(&hunk(&big, &[], &[])).is_some());
        assert!(hunk_size_error(&hunk(&[], &big, &[])).is_some());
        assert!(hunk_size_error(&hunk(&[], &[], &big)).is_some());
        assert!(hunk_size_error(&hunk(&["a"], &["b"], &["c"])).is_none());
    }

    #[test]
    fn user_prompt_carries_path_instruction_labels_and_sides() {
        let hunk = hunk(&["base"], &["ours"], &[]);
        let prompt = build_hunk_user_prompt(
            "src/a.rs",
            &hunk,
            "Prefer ours",
            Some("main (target)"),
            Some("abc123 (incoming)"),
        );
        assert!(prompt.contains("src/a.rs"));
        assert!(prompt.contains("hunk 3"));
        assert!(prompt.contains("User instruction: Prefer ours"));
        assert!(prompt.contains("ours = main (target)"));
        assert!(prompt.contains("theirs = abc123 (incoming)"));
        assert!(prompt.contains("Base (common ancestor):\n```text\nbase\n```"));
        assert!(prompt.contains("Ours (target branch side):\n```text\nours\n```"));
        // Empty side renders an explicit placeholder.
        assert!(prompt.contains("Theirs (incoming commit side):\n```text\n(empty)\n```"));
    }

    #[test]
    fn user_prompt_omits_empty_instruction_and_missing_labels() {
        let hunk = hunk(&[], &["o"], &["t"]);
        let prompt = build_hunk_user_prompt("f.rs", &hunk, "  ", None, None);
        assert!(!prompt.contains("User instruction"));
        assert!(!prompt.contains("Side label"));
    }

    #[test]
    fn system_prompt_language_directive() {
        // auto/empty: mirror the user instruction's language.
        for auto in ["", "auto", "AUTO"] {
            assert!(
                build_system_prompt(auto).contains("same language as the user instruction"),
                "{auto:?} should get the mirror directive"
            );
        }
        // Any explicit language — including "en" — gets an explicit directive.
        assert!(build_system_prompt("en").contains("explanation in en."));
        assert!(build_system_prompt("Chinese").contains("explanation in Chinese."));
        // Resolution lines are file content and are never translated.
        assert!(build_system_prompt("Chinese").contains("Never translate the resolution lines"));
    }
}
