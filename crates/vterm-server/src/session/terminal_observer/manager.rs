use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, info, warn};

use vterm_common::{
    KnownAppInfo, SessionApp, SessionAppStatus, SessionId, TerminalAttention, TerminalAttentionKind,
};

use crate::addon::{resolve_llm_addon_id, run_chat_completion};
use crate::addon::{AddonHost, LlmRequestContext};
use crate::config::{ConfigManager, TerminalServerConfig};
use crate::session::{
    prompt_signature, run_detect_command, run_detect_output, run_get_program_name,
    run_get_supported_commands, should_auto_confirm, strip_ansi_sequences, trim_to_recent_limit,
    PromptMatch, SupportedCommand, RECENT_OUTPUT_LIMIT,
};

use super::types::TerminalObserverEvent;

/// `known_apps` shells out to the filesystem (PATH stat per candidate app)
/// synchronously; the result only changes when software is (un)installed,
/// so a one-minute cache absorbs the relay sync tick's repeated scans.
const KNOWN_APPS_CACHE_TTL: Duration = Duration::from_secs(60);

const LLM_SYSTEM_PROMPT: &str = "\
You are an experienced professional software engineer who now uses AI code agents \
(such as Claude Code, Codex, OpenCode, Cursor, etc.) for autonomous programming.\n\
Your task is to analyze the current terminal output from a code agent session and \
classify what kind of user interaction the agent is waiting for.\n\n\
Analyze the terminal text and respond with a JSON object containing two fields:\n\
- \"status\": one of \"idle\", \"confirm\", \"confirm_special\", \"selection\"\n\
- \"message\": A very brief summary (<100 words) of what the agent is waiting for or doing\n\n\
Status meanings:\n\
- \"idle\": The agent is actively working, running commands, writing code, or has completed a task. No user action needed.\n\
- \"confirm\": The agent is asking for a routine confirmation or simple choice that does NOT involve safety risks. \
This includes: proceeding with file edits, creating new files, installing packages, running tests, \
continuing to the next step, pressing Enter to continue, answering y/n to proceed, \
selecting from a list of harmless options, or any low-risk action that a developer would routinely approve.\n\
- \"confirm_special\": The agent is requesting permission for an action that could affect safety or cause significant changes. \
This includes: deleting important files (not temporary files or build artifacts), committing code to git, \
pushing code to remote, dropping or writing to databases, executing destructive shell commands (rm -rf on important dirs), \
overwriting critical configs, any irreversible operation, or actions involving credentials/secrets.\n\
- \"selection\": The agent is presenting a significant decision point where the user's choice substantially affects \
the direction or outcome of the task. This includes: choosing between different architectural approaches, \
selecting a technology stack or framework, deciding on a design pattern, choosing between alternative implementations \
that produce fundamentally different results, or any choice where the options are meaningfully different in scope/impact. \
Do NOT use this for simple y/n confirmations or routine tool approvals.\n\n\
Important guidelines:\n\
- When in doubt between \"confirm\" and \"confirm_special\", prefer \"confirm\" — most agent prompts are routine.\n\
- Reserve \"confirm_special\" only for genuinely dangerous or irreversible operations.\n\
- Reserve \"selection\" only for genuine decision points, not for simple approval flows.\n\
- If the agent appears to be actively working (showing progress, tool calls, streaming output), return \"idle\".\n\n\
Determine the state from agent output, not from arbitrary text nearest the bottom of the screen:\n\
- Terminal snapshots can include UI chrome, a command composer, a remembered user request, model/directory labels,\n\
  and other text that is not a prompt from the agent. Treat these as context only; they do not by themselves mean\n\
  the agent is waiting for the user. In particular, an instruction such as \"Improve documentation in @filename\"\n\
  shown after a Codex completion footer is often a composer value or prior user request, not an unanswered question.\n\
- Give priority to the latest substantive agent event: tool result, completion/result summary, explicit question,\n\
  or an actual interactive prompt with choices. A completion footer such as \"Worked for 2m 10s\", followed by\n\
  a summary of changed files, means the task completed unless the agent itself then asks a clear question.\n\
- Never infer a missing parameter, unfinished task, or request for clarification from an imperative user instruction\n\
  visible in the snapshot. Only classify confirm or selection when the AGENT explicitly requests input now, usually\n\
  with a question, an input field, or actionable options.\n\
- For \"idle\", summarize the real latest agent activity or result (for example, \"Updated prototype/index.html\"),\n\
  rather than paraphrasing a stale user instruction.\n\
\n\
Handling multiple affirmative options:\n\
- Agents often present several variants of \"yes\" that all mean the same thing, e.g. \
\"Yes\" / \"Yes, and don't ask again\" / \"Yes, always allow this action\" / \"Continue\" / \"Yes to all\". \
Treat ALL of these as a single routine \"confirm\" — they are different wordings of the same approval, \
NOT a meaningful choice between outcomes. Only classify as \"selection\" when the options lead to \
genuinely different results (e.g. \"Use TypeScript\" vs \"Use JavaScript\", \"Overwrite\" vs \"Skip\" vs \"Rename\").\n\
- Likewise, multiple negative variants (\"No\" / \"No, and don't ask again\" / \"Never\") are the same decline, not a selection.\n\n\
Respond ONLY with the JSON object, no other text. /no_think";

/// Build the language directive appended to the terminal-classifier system
/// prompt. Only the `"message"` summary follows the configured language — the
/// `"status"` enum values must stay in English so downstream pattern matching
/// keeps working.
///
/// - Empty or "auto": mirror the dominant natural language of the terminal
///   output (the agent's displayed text).
/// - Otherwise: write the summary in the user-specified language, used verbatim
///   so codes like "zh-CN" and names like "中文" both work.
fn terminal_language_instruction(output_language: &str) -> String {
    let normalized = output_language.trim();
    if normalized.is_empty() || normalized.eq_ignore_ascii_case("auto") {
        concat!(
            "\n\nLanguage: detect the dominant natural language of the terminal ",
            "output (the agent's displayed text) and write the \"message\" ",
            "summary in that same language — for example, English output yields ",
            "an English summary, Chinese output yields a Chinese summary. The ",
            "\"status\" field values (idle, confirm, confirm_special, selection) ",
            "must always stay in English."
        )
        .to_string()
    } else {
        format!(
            "\n\nLanguage: write the \"message\" summary in {}. Keep the \"status\" field values in English.",
            normalized
        )
    }
}

/// Snapshot of the rendered terminal state needed by the observer.
#[derive(Debug, Clone, Default)]
pub struct ObservedTerminalState {
    pub terminal_title: String,
    pub current_line_text: String,
}

impl ObservedTerminalState {
    /// Snapshot the title and current line from a terminal model.
    pub fn from_terminal(terminal: &vterm_core::emulator::Terminal) -> Self {
        Self {
            terminal_title: terminal.title().to_string(),
            current_line_text: terminal.current_line_text(),
        }
    }
}

/// Applications that can be launched directly from the file explorer. This
/// launcher list is independent from app addons: an addon adds output
/// detection and automation, while a CLI only needs to be on PATH.
const BUILTIN_KNOWN_APPS: &[(&str, &str)] = &[
    ("claude", "Claude Code"),
    ("codex", "Codex"),
    ("kimi", "Kimi Code"),
    ("pi", "Pi Agent"),
    ("opencode", "OpenCode"),
    ("qwen", "Qwen Code"),
    ("dsh", "DeepSeek Harness"),
];

// ---------------------------------------------------------------------------
// Shared factory — holds config, addon host, addon cache, event sender
// ---------------------------------------------------------------------------

pub struct TerminalObserverManager {
    config_manager: Arc<ConfigManager>,
    addon_host: Arc<AddonHost>,
    app_addons: Arc<RwLock<Vec<AppAddonEntry>>>,
    event_tx: mpsc::UnboundedSender<TerminalObserverEvent>,
    /// TTL cache for `known_apps`: the underlying PATH scan is a series of
    /// synchronous stat calls that must not run on every relay sync tick.
    known_apps_cache: Mutex<Option<(Instant, Vec<KnownAppInfo>)>>,
}

impl TerminalObserverManager {
    pub fn new(
        config_manager: Arc<ConfigManager>,
        addon_host: Arc<AddonHost>,
        event_tx: mpsc::UnboundedSender<TerminalObserverEvent>,
    ) -> Self {
        let app_addons = Arc::new(RwLock::new(Vec::new()));
        let manager = Self {
            config_manager,
            addon_host,
            app_addons,
            event_tx,
            known_apps_cache: Mutex::new(None),
        };
        manager.build_app_addon_cache();
        manager
    }

    fn build_app_addon_cache(&self) {
        let mut cache = Vec::new();
        for addon in self.addon_host.app_addons() {
            let addon_id = addon.manifest.id.clone();
            let program_name = match run_get_program_name(&self.addon_host, &addon_id) {
                Some(name) => name,
                None => continue,
            };
            let supported_commands = run_get_supported_commands(&self.addon_host, &addon_id);
            debug!(
                "App addon cached: addon_id={}, program_name={}, commands={}",
                addon_id,
                program_name,
                supported_commands.len()
            );
            cache.push(AppAddonEntry {
                addon_id,
                program_name,
                supported_commands,
            });
        }
        if let Ok(mut guard) = self.app_addons.try_write() {
            *guard = cache;
        }
    }

    pub async fn refresh_app_cache(&self) {
        let mut cache = Vec::new();
        for addon in self.addon_host.app_addons() {
            let addon_id = addon.manifest.id.clone();
            let program_name = match run_get_program_name(&self.addon_host, &addon_id) {
                Some(name) => name,
                None => continue,
            };
            let supported_commands = run_get_supported_commands(&self.addon_host, &addon_id);
            debug!(
                "App addon refreshed: addon_id={}, program_name={}, commands={}",
                addon_id,
                program_name,
                supported_commands.len()
            );
            cache.push(AppAddonEntry {
                addon_id,
                program_name,
                supported_commands,
            });
        }
        *self.app_addons.write().await = cache;
    }

    /// Return built-in and addon-provided terminal applications found on PATH.
    /// The executable name is deliberately kept as the shell command, rather
    /// than its resolved path, so it can be inserted directly into a terminal.
    /// Each entry is annotated with where its MCP setup line must be typed
    /// (see `KnownAppInfo::mcp_command_target`) so clients can offer apps in
    /// the right context: shell-command apps in a plain terminal, chat-prompt
    /// apps (Kimi) only inside the running app.
    pub async fn known_apps(&self) -> Vec<KnownAppInfo> {
        {
            let cache = self.known_apps_cache.lock().await;
            if let Some((cached_at, apps)) = cache.as_ref() {
                if cached_at.elapsed() < KNOWN_APPS_CACHE_TTL {
                    return apps.clone();
                }
            }
        }
        let entries = self.app_addons.read().await;
        // One config snapshot classifies every app through its MCP template.
        let mcp_config = self.config_manager.snapshot().await.mcp;
        let path = terminal_command_path();
        let mut apps = BUILTIN_KNOWN_APPS
            .iter()
            .filter(|(executable, _)| command_is_available(executable, path.as_deref()))
            .map(|(executable, display_name)| KnownAppInfo {
                executable: (*executable).to_string(),
                display_name: (*display_name).to_string(),
                mcp_command_target: crate::mcp::mcp_command_target_for_app(executable, &mcp_config),
            })
            .collect::<Vec<_>>();
        // Preserve custom app addons as extra launchers, but do not duplicate
        // built-in commands such as claude/codex/kimi.
        let addon_apps = entries
            .iter()
            .filter(|entry| command_is_available(&entry.program_name, path.as_deref()))
            .filter(|entry| !apps.iter().any(|app| app.executable == entry.program_name))
            .filter_map(|entry| {
                self.addon_host
                    .get_instance(&entry.addon_id)
                    .map(|addon| KnownAppInfo {
                        executable: entry.program_name.clone(),
                        display_name: addon.manifest.name.clone(),
                        mcp_command_target: crate::mcp::mcp_command_target_for_app(
                            &entry.program_name,
                            &mcp_config,
                        ),
                    })
            })
            .collect::<Vec<_>>();
        apps.extend(addon_apps);
        apps.sort_by(|left, right| left.display_name.cmp(&right.display_name));
        *self.known_apps_cache.lock().await = Some((Instant::now(), apps.clone()));
        apps
    }

    fn build_observer(
        &self,
        session_id: SessionId,
        title: String,
        session_name: Option<String>,
        detection_mode: AppDetectionMode,
        terminal_os: Option<vterm_common::SessionTerminalOs>,
    ) -> SessionObserver {
        SessionObserver {
            session_id,
            title,
            session_name,
            app: SessionApp::none(),
            app_status: SessionAppStatus::None,
            published_state: PublishedSessionState::default(),
            active_addon: None,
            terminal_title: String::new(),
            current_line_text: String::new(),
            recent_output: String::new(),
            output_revision: 0,
            analyzed_revision: 0,
            output_heartbeat: None,
            output_heartbeat_generation: 0,
            analysis_generation: 0,
            prompt_generation: 0,
            input_revision: 0,
            active_prompt: None,
            notification_enabled: true,
            auto_confirm_enabled: true,
            llm_failure_count: 0,
            llm_disabled: false,
            llm_disabled_at: None,
            retry_cooldown_secs: 300,
            detection_mode,
            app_launch_source: None,
            terminal_os,
            config_manager: self.config_manager.clone(),
            addon_host: self.addon_host.clone(),
            app_addons: self.app_addons.clone(),
            event_tx: self.event_tx.clone(),
        }
    }

    /// Spawn the per-session observer actor and return its handle.
    ///
    /// The actor task is the sole owner of the `SessionObserver` state: every
    /// mutation arrives as an ordered `ObserverCommand`, so the input/output
    /// hot paths never wait on a lock — and never on synchronous QuickJS
    /// detection running under one. Delayed work (analysis, prompt grace)
    /// re-enters through the same channel, which keeps the generation-based
    /// staleness checks intact without any `Arc<Mutex<SessionObserver>>`.
    pub fn spawn_observer_actor(
        &self,
        session: &Arc<tokio::sync::Mutex<crate::session::session_core::Session>>,
        session_id: SessionId,
        title: String,
        session_name: Option<String>,
        detection_mode: AppDetectionMode,
        terminal_os: Option<vterm_common::SessionTerminalOs>,
        notification_enabled: bool,
        auto_confirm_enabled: bool,
    ) -> ObserverHandle {
        let mut observer =
            self.build_observer(session_id, title, session_name, detection_mode, terminal_os);
        observer.set_notification_enabled(notification_enabled);
        observer.set_auto_confirm(auto_confirm_enabled);
        let (tx, rx) = mpsc::unbounded_channel();
        let handle = ObserverHandle { tx: tx.clone() };
        let scheduler = ObserverScheduler {
            tx,
            session: Arc::downgrade(session),
        };
        tokio::spawn(run_observer_actor(observer, scheduler, rx));
        // Install the emulator sequence callback if the observer starts out
        // with watched sequence types (e.g. shell-integration sessions).
        handle.sync_sequence_callback();
        handle
    }

    /// Test-facing constructor: tests drive the observer directly, without an
    /// actor, so they keep deterministic control over ordering.
    #[cfg(test)]
    pub fn create_observer(
        &self,
        session_id: SessionId,
        title: String,
        session_name: Option<String>,
        detection_mode: AppDetectionMode,
    ) -> Arc<Mutex<SessionObserver>> {
        Arc::new(Mutex::new(self.build_observer(
            session_id,
            title,
            session_name,
            detection_mode,
            None,
        )))
    }

    #[cfg(test)]
    pub async fn insert_test_app_addon(
        &self,
        addon_id: &str,
        program_name: &str,
        supported_commands: Vec<SupportedCommand>,
    ) {
        self.app_addons.write().await.push(AppAddonEntry {
            addon_id: addon_id.to_string(),
            program_name: program_name.to_string(),
            supported_commands,
        });
    }
}

fn terminal_command_path() -> Option<std::ffi::OsString> {
    #[cfg(unix)]
    {
        let login_env = crate::session::shell_integration::bash_login_environment(
            "bash",
            &std::collections::HashMap::new(),
        );
        if let Some(path) = login_env.get("PATH") {
            return Some(path.into());
        }
    }
    std::env::var_os("PATH")
}

fn command_is_available(command: &str, path: Option<&std::ffi::OsStr>) -> bool {
    let Some(path) = path else {
        return false;
    };

    std::env::split_paths(path).any(|directory| {
        let candidate = directory.join(command);
        if candidate.is_file() {
            return true;
        }

        #[cfg(target_os = "windows")]
        {
            ["exe", "cmd", "bat"]
                .into_iter()
                .any(|extension| directory.join(format!("{command}.{extension}")).is_file())
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    })
}

// ---------------------------------------------------------------------------
// Per-session observer actor
// ---------------------------------------------------------------------------

/// Commands sent to the per-session observer actor. The actor is the only
/// mutator of `SessionObserver` state; ordering is the channel's FIFO order.
enum ObserverCommand {
    RecordOutput {
        data: Arc<Vec<u8>>,
        state: ObservedTerminalState,
    },
    RecordInput {
        is_newline_only: bool,
        current_line_text: Option<String>,
    },
    Sequence {
        seq_type: String,
        code: String,
        data: Vec<u8>,
    },
    SetNotificationEnabled(bool),
    SetAutoConfirm(bool),
    UpdateMetadata {
        title: String,
        session_name: Option<String>,
    },
    ResolveAttention,
    ResetAppTracking,
    SyncSequenceCallback,
    /// A delayed analysis sub-task finished sleeping and captured the screen;
    /// the actor decides (via `prepare_analysis`) whether it is still current.
    RunAnalysisPrepared {
        generation: u64,
        snapshot: String,
    },
    /// The LLM job of a prepared analysis finished; apply with staleness checks.
    ApplyAnalysisResult {
        result: AnalysisResult,
    },
    PromptGraceExpired {
        generation: u64,
    },
    OutputHeartbeatExpired {
        generation: u64,
    },
    Shutdown,
}

/// How observer-owned sub-tasks reach back into the actor: the command
/// channel plus a weak session link for emulator-callback and screen
/// snapshot access. Weak so a closed session is never kept alive by its
/// observer machinery.
#[derive(Clone)]
struct ObserverScheduler {
    tx: mpsc::UnboundedSender<ObserverCommand>,
    session: std::sync::Weak<tokio::sync::Mutex<crate::session::session_core::Session>>,
}

/// Handle to the per-session observer actor. Every method is a plain
/// non-blocking channel send: callers on the keystroke/echo paths never wait
/// for observer work (prompt detection, QuickJS handlers, analysis).
#[derive(Clone)]
pub struct ObserverHandle {
    tx: mpsc::UnboundedSender<ObserverCommand>,
}

impl ObserverHandle {
    fn send(&self, command: ObserverCommand) {
        // A closed channel means the actor stopped (session closed). The
        // observer is best-effort sideband tooling, so dropping is fine.
        let _ = self.tx.send(command);
    }

    pub fn record_output(&self, data: Arc<Vec<u8>>, state: ObservedTerminalState) {
        self.send(ObserverCommand::RecordOutput { data, state });
    }

    pub fn record_input(&self, is_newline_only: bool, current_line_text: Option<String>) {
        self.send(ObserverCommand::RecordInput {
            is_newline_only,
            current_line_text,
        });
    }

    pub fn set_notification_enabled(&self, enabled: bool) {
        self.send(ObserverCommand::SetNotificationEnabled(enabled));
    }

    pub fn set_auto_confirm(&self, enabled: bool) {
        self.send(ObserverCommand::SetAutoConfirm(enabled));
    }

    pub fn update_metadata(&self, title: String, session_name: Option<String>) {
        self.send(ObserverCommand::UpdateMetadata {
            title,
            session_name,
        });
    }

    pub fn resolve_attention(&self) {
        self.send(ObserverCommand::ResolveAttention);
    }

    pub fn reset_app_tracking(&self) {
        self.send(ObserverCommand::ResetAppTracking);
    }

    pub fn sync_sequence_callback(&self) {
        self.send(ObserverCommand::SyncSequenceCallback);
    }

    /// Ask the actor to stop. Dropping all handles would also stop it once
    /// delayed sub-tasks drain; session close uses this for prompt teardown.
    pub fn shutdown(&self) {
        self.send(ObserverCommand::Shutdown);
    }
}

/// Without messages for this long, the actor checks whether its session is
/// gone and exits if so — a safety net for handles dropped without Shutdown.
const OBSERVER_IDLE_WATCHDOG: Duration = Duration::from_secs(60);

async fn run_observer_actor(
    mut observer: SessionObserver,
    scheduler: ObserverScheduler,
    mut rx: mpsc::UnboundedReceiver<ObserverCommand>,
) {
    loop {
        let command = tokio::select! {
            command = rx.recv() => match command {
                Some(command) => command,
                None => break,
            },
            _ = tokio::time::sleep(OBSERVER_IDLE_WATCHDOG) => {
                if scheduler.session.upgrade().is_none() {
                    break;
                }
                continue;
            }
        };
        match command {
            ObserverCommand::Shutdown => break,
            ObserverCommand::RecordOutput { data, state } => {
                observer.record_output(&data, state, &scheduler).await;
            }
            ObserverCommand::RecordInput {
                is_newline_only,
                current_line_text,
            } => {
                let detected = observer
                    .record_input(is_newline_only, current_line_text)
                    .await;
                if detected {
                    sync_sequence_callback_now(&observer, &scheduler).await;
                }
            }
            ObserverCommand::Sequence {
                seq_type,
                code,
                data,
            } => {
                let stopped = observer
                    .handle_sequence(seq_type, code, data, &scheduler)
                    .await;
                if stopped {
                    if let Some(session) = scheduler.session.upgrade() {
                        session.lock().await.terminal.clear_sequence_callback();
                    }
                }
            }
            ObserverCommand::SyncSequenceCallback => {
                sync_sequence_callback_now(&observer, &scheduler).await;
            }
            ObserverCommand::SetNotificationEnabled(enabled) => {
                observer.set_notification_enabled(enabled);
            }
            ObserverCommand::SetAutoConfirm(enabled) => {
                observer.set_auto_confirm(enabled);
            }
            ObserverCommand::UpdateMetadata {
                title,
                session_name,
            } => {
                observer.update_metadata(title, session_name);
            }
            ObserverCommand::ResolveAttention => observer.resolve_attention(),
            ObserverCommand::ResetAppTracking => observer.reset_app_tracking(),
            ObserverCommand::RunAnalysisPrepared {
                generation,
                snapshot,
            } => {
                // Refresh the LLM retry cooldown from the live config so the
                // synchronous prepare_analysis can use it for cooldown checks.
                observer.retry_cooldown_secs = {
                    let config = observer.config_manager.snapshot().await;
                    config.llm.terminal.provider.retry_cooldown_secs
                };
                if let Some(job) = observer.prepare_analysis(generation) {
                    let tx = scheduler.tx.clone();
                    tokio::spawn(async move {
                        let result = run_analysis_job(job, snapshot).await;
                        let _ = tx.send(ObserverCommand::ApplyAnalysisResult { result });
                    });
                }
            }
            ObserverCommand::ApplyAnalysisResult { result } => {
                let should_auto_confirm = observer.apply_analysis_result(result, &scheduler);
                if should_auto_confirm {
                    // `apply_analysis_result` already emitted
                    // `AutoConfirmRequested`. The observer event loop routes
                    // that through `forward_input`, the same path a physical
                    // Enter from the client takes.
                    info!(
                        session_id = %observer.session_id,
                        "Auto-confirm triggered: Enter forwarded via observer event loop"
                    );
                }
            }
            ObserverCommand::PromptGraceExpired { generation } => {
                observer.emit_prompt_attention_if_pending(generation);
            }
            ObserverCommand::OutputHeartbeatExpired { generation } => {
                observer.handle_output_heartbeat_expired(generation, &scheduler);
            }
        }
    }
}

/// Install or clear the terminal sequence callback based on the observer's
/// current needs. Runs inside the actor; the emulator is reached through the
/// session lock, which serializes against the drain task's parsing.
async fn sync_sequence_callback_now(observer: &SessionObserver, scheduler: &ObserverScheduler) {
    let types = observer.callback_types();
    let Some(session) = scheduler.session.upgrade() else {
        return;
    };
    let mut session_guard = session.lock().await;
    if types.is_empty() {
        session_guard.terminal.clear_sequence_callback();
        return;
    }

    let callback = create_sequence_callback(scheduler.tx.clone());
    session_guard
        .terminal
        .set_sequence_callback(types, callback);
}

/// Create a `TerminalSequenceCallback` that forwards sequences to the actor.
/// Called by the emulator while parsing (on a blocking thread), so this is a
/// plain channel send — no runtime context and no locking required.
fn create_sequence_callback(
    tx: mpsc::UnboundedSender<ObserverCommand>,
) -> vterm_core::emulator::TerminalSequenceCallback {
    Box::new(move |seq_type, code, data| {
        let _ = tx.send(ObserverCommand::Sequence {
            seq_type,
            code,
            data,
        });
    })
}

// ---------------------------------------------------------------------------
// Per-session observer — owns all session-specific state
// ---------------------------------------------------------------------------

struct AppAddonEntry {
    addon_id: String,
    program_name: String,
    supported_commands: Vec<SupportedCommand>,
}

struct ActiveAddon {
    addon_id: String,
    supported_commands: Vec<SupportedCommand>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppDetectionMode {
    LegacyPrompt,
    ShellIntegration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppLaunchSource {
    LegacyPrompt,
    ShellIntegration,
}

struct PromptState {
    prompt: PromptMatch,
    signature: String,
    notified: bool,
    auto_confirmed: bool,
    auto_confirmed_revision: Option<u64>,
}

struct AnalysisJob {
    generation: u64,
    output_revision: u64,
    prompt_generation: u64,
    input_revision: u64,
    session_id: SessionId,
    auto_confirm_enabled: bool,
    llm_disabled: bool,
    /// True when this job is a cooldown-expired retry of a previously-disabled
    /// LLM. The LLM is called despite `llm_disabled` so the caller can decide
    /// whether to re-enable it permanently based on the outcome.
    llm_retry: bool,
    app: SessionApp,
    addon_id: Option<String>,
    addon_host: Arc<AddonHost>,
    config_manager: Arc<ConfigManager>,
}

struct AnalysisResult {
    generation: u64,
    output_revision: u64,
    prompt_generation: u64,
    input_revision: u64,
    auto_confirm_enabled: bool,
    app_prompt: Option<PromptMatch>,
    llm_succeeded: bool,
    llm_failure: Option<LlmFailure>,
    /// Mirrors the job's `llm_retry` so the result handler knows this was a
    /// cooldown retry and can re-enable or restart the cooldown accordingly.
    llm_retry: bool,
}

struct LlmFailure {
    error: String,
    max_failures: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct PublishedSessionState {
    app: SessionApp,
    app_status: SessionAppStatus,
    app_status_summary: Option<String>,
}

#[derive(Clone, Copy)]
struct OutputHeartbeat {
    generation: u64,
    last_output: Instant,
    idle_after: Duration,
}

pub struct SessionObserver {
    session_id: SessionId,
    title: String,
    session_name: Option<String>,
    app: SessionApp,
    app_status: SessionAppStatus,
    published_state: PublishedSessionState,
    active_addon: Option<ActiveAddon>,
    terminal_title: String,
    current_line_text: String,
    recent_output: String,
    output_revision: u64,
    analyzed_revision: u64,
    output_heartbeat: Option<OutputHeartbeat>,
    output_heartbeat_generation: u64,
    analysis_generation: u64,
    prompt_generation: u64,
    input_revision: u64,
    active_prompt: Option<PromptState>,
    notification_enabled: bool,
    auto_confirm_enabled: bool,
    llm_failure_count: u32,
    llm_disabled: bool,
    /// When the LLM was disabled (for cooldown-based retry). `None` when the
    /// LLM is not disabled or has never been disabled.
    llm_disabled_at: Option<Instant>,
    /// Configured LLM retry cooldown in seconds (refreshed from config before
    /// each analysis). Kept on the observer so the synchronous `prepare_analysis`
    /// can read it without an async config snapshot.
    retry_cooldown_secs: u32,
    detection_mode: AppDetectionMode,
    app_launch_source: Option<AppLaunchSource>,
    /// Cached at creation: the sequence handler reads this on every matched
    /// sequence, and it never changes over a session's lifetime.
    terminal_os: Option<vterm_common::SessionTerminalOs>,
    // Shared resources (cloned from TerminalObserverManager)
    config_manager: Arc<ConfigManager>,
    addon_host: Arc<AddonHost>,
    app_addons: Arc<RwLock<Vec<AppAddonEntry>>>,
    event_tx: mpsc::UnboundedSender<TerminalObserverEvent>,
}

impl SessionObserver {
    fn display_title(&self) -> String {
        self.session_name
            .as_deref()
            .filter(|value| !value.is_empty())
            .unwrap_or(&self.title)
            .to_string()
    }

    pub fn update_metadata(&mut self, title: String, session_name: Option<String>) {
        self.title = title;
        self.session_name = session_name;
    }

    /// Clear any active prompt. Session removal itself is broadcast through the
    /// authoritative SessionList snapshot, so no separate resolved event is needed.
    pub fn resolve_attention(&mut self) {
        self.active_prompt = None;
    }

    /// Reset app tracking after the session's shell was respawned: the old
    /// shell died while an app was running, so no exit marker was ever
    /// observed and the observer still believes that app is active. Shell
    /// integration only detects a new app while none is active, so without
    /// this reset every later app launch goes undetected.
    pub fn reset_app_tracking(&mut self) {
        self.app_status = SessionAppStatus::None;
        self.output_heartbeat = None;
        self.active_prompt = None;
        self.active_addon = None;
        self.app_launch_source = None;
        update_app(self, SessionApp::none());
        update_app_status(self, SessionAppStatus::None);
    }

    pub fn set_notification_enabled(&mut self, enabled: bool) {
        self.notification_enabled = enabled;
    }

    pub fn set_auto_confirm(&mut self, enabled: bool) {
        info!(
            session_id = %self.session_id,
            enabled,
            "set_auto_confirm: auto-confirm setting changed"
        );
        self.auto_confirm_enabled = enabled;
    }

    /// Get the set of sequence types to watch based on active_addon's supported_commands.
    pub fn callback_types(&self) -> std::collections::HashSet<String> {
        let mut types: std::collections::HashSet<String> = match &self.active_addon {
            Some(addon) => addon
                .supported_commands
                .iter()
                .map(|cmd| cmd.seq_type.clone())
                .collect(),
            None => std::collections::HashSet::new(),
        };
        if self.detection_mode == AppDetectionMode::ShellIntegration {
            types.insert("osc".to_string());
        }
        types
    }

    /// Record PTY output using the latest terminal snapshot.
    pub async fn record_output(
        &mut self,
        data: &[u8],
        terminal_state: ObservedTerminalState,
        scheduler: &ObserverScheduler,
    ) {
        if !self.notification_enabled {
            return;
        }

        self.terminal_title = terminal_state.terminal_title;
        self.current_line_text = terminal_state.current_line_text;

        self.output_revision = self.output_revision.saturating_add(1);
        self.prompt_generation = self.prompt_generation.saturating_add(1);
        let output = strip_ansi_sequences(&String::from_utf8_lossy(data));
        self.recent_output.push_str(&output);
        trim_to_recent_limit(&mut self.recent_output, RECENT_OUTPUT_LIMIT);

        let check_output = self.app_status == SessionAppStatus::Idle
            || (self.app_status == SessionAppStatus::Busy && self.output_heartbeat.is_some());
        if !check_output {
            return;
        }
        let Some(result) = self
            .active_addon
            .as_ref()
            .map(|addon| run_detect_output(&self.addon_host, &addon.addon_id, &output))
        else {
            return;
        };
        if result.status != SessionAppStatus::Busy {
            return;
        }

        if self.app_status == SessionAppStatus::Idle {
            if result.refresh_only {
                return;
            }
            self.app_status = SessionAppStatus::Busy;
            self.analysis_generation = self.analysis_generation.saturating_add(1);
            update_app_status(self, SessionAppStatus::Busy);
            if let Some(idle_after_ms) = result.idle_after_ms {
                self.output_heartbeat_generation =
                    self.output_heartbeat_generation.saturating_add(1);
                let generation = self.output_heartbeat_generation;
                let idle_after = Duration::from_millis(idle_after_ms);
                self.output_heartbeat = Some(OutputHeartbeat {
                    generation,
                    last_output: Instant::now(),
                    idle_after,
                });
                self.schedule_output_heartbeat(generation, idle_after, scheduler);
            }
        } else if let (Some(heartbeat), Some(idle_after_ms)) =
            (self.output_heartbeat.as_mut(), result.idle_after_ms)
        {
            heartbeat.last_output = Instant::now();
            heartbeat.idle_after = Duration::from_millis(idle_after_ms);
        }
    }

    /// Record user input and detect apps.
    /// Returns `true` if a new app was detected via command matching.
    pub async fn record_input(
        &mut self,
        is_newline_only: bool,
        current_line_text: Option<String>,
    ) -> bool {
        if let Some(current_line_text) = current_line_text {
            self.current_line_text = current_line_text;
        }

        let mut detected_app = false;

        if is_newline_only
            && self.notification_enabled
            && self.detection_mode == AppDetectionMode::LegacyPrompt
        {
            let trimmed = self.current_line_text.trim_end();
            if self.app.is_none() && !trimmed.is_empty() && trimmed.len() < 512 {
                if let Some(cmd) = strip_prompt(trimmed) {
                    if let Some((program_name, addon_id, supported_commands)) =
                        self.match_app_addon_for_command_line(&cmd).await
                    {
                        debug!(
                            "App detected via command: program_name={}, addon_id={}",
                            program_name, addon_id
                        );
                        self.app_status = SessionAppStatus::Idle;
                        // Publish the detected app together with its initial
                        // status. `update_app` snapshots both fields into the
                        // SessionStateChanged event.
                        update_app(self, SessionApp(program_name));
                        self.active_addon = Some(ActiveAddon {
                            addon_id,
                            supported_commands,
                        });
                        self.app_launch_source = Some(AppLaunchSource::LegacyPrompt);
                        detected_app = true;
                    }
                }
            }
        }

        self.prompt_generation = self.prompt_generation.saturating_add(1);
        self.input_revision = self.input_revision.saturating_add(1);
        if self.active_prompt.take().is_some() {
            update_app_status(self, SessionAppStatus::Idle);
        }

        detected_app
    }

    /// Handle a terminal control sequence forwarded from the terminal emulator callback.
    /// Returns `true` if the app was stopped (exited).
    pub async fn handle_sequence(
        &mut self,
        seq_type: String,
        code: String,
        data: Vec<u8>,
        scheduler: &ObserverScheduler,
    ) -> bool {
        if !self.notification_enabled {
            return false;
        }

        let mut refresh_sequence_callback = false;
        if self.detection_mode == AppDetectionMode::ShellIntegration
            && self
                .handle_shell_integration_sequence(&seq_type, &code, &data)
                .await
        {
            refresh_sequence_callback = true;
        }

        if refresh_sequence_callback {
            let _ = scheduler.tx.send(ObserverCommand::SyncSequenceCallback);
        }

        if self.app.is_none() {
            return false;
        }

        let Some(addon) = &self.active_addon else {
            return false;
        };

        // Check if this sequence matches a supported command
        let matches = addon
            .supported_commands
            .iter()
            .any(|cmd| cmd.seq_type == seq_type && cmd.code == code);
        if !matches {
            return false;
        }

        // Call detect_command with the session OS, seq_type, code, and data.
        let result = run_detect_command(
            &self.addon_host,
            &addon.addon_id,
            &seq_type,
            &code,
            &data,
            self.terminal_os,
        );

        if result.stopped
            || result.status == SessionAppStatus::Idle
            || result.status == SessionAppStatus::Busy
        {
            self.output_heartbeat = None;
        }
        if result.status == self.app_status {
            return false;
        }

        if result.stopped && self.app_launch_source == Some(AppLaunchSource::ShellIntegration) {
            if self.app_status != SessionAppStatus::Idle {
                self.app_status = SessionAppStatus::Idle;
                self.analysis_generation = self.analysis_generation.saturating_add(1);
                let generation = self.analysis_generation;
                update_app_status(self, SessionAppStatus::Idle);
                self.schedule_analysis(generation, scheduler);
            }
            return false;
        }

        if result.stopped {
            // 'stop' — app exited the terminal
            info!(
                "App '{}' exited in session {} ({})",
                self.app.0,
                self.session_id,
                self.display_title()
            );
            self.app_status = SessionAppStatus::None;
            if self.active_prompt.take().is_some() {
                update_app_status(self, SessionAppStatus::None);
            }
            self.active_addon = None;
            self.app_launch_source = None;
            update_app(self, SessionApp::none());
            update_app_status(self, SessionAppStatus::None);
            return true;
        }

        match result.status {
            SessionAppStatus::None => {
                // 'none' — do not process (e.g. title === 'claude')
            }
            SessionAppStatus::Idle => {
                self.app_status = SessionAppStatus::Idle;
                self.analysis_generation = self.analysis_generation.saturating_add(1);
                let generation = self.analysis_generation;
                update_app_status(self, SessionAppStatus::Idle);
                self.schedule_analysis(generation, scheduler);
            }
            SessionAppStatus::Busy => {
                self.app_status = SessionAppStatus::Busy;
                // Invalidate analysis queued while the app was idle. Some agents
                // briefly enter Idle before automatically starting the next task.
                self.analysis_generation = self.analysis_generation.saturating_add(1);
                update_app_status(self, SessionAppStatus::Busy);
            }
            _ => {}
        }
        false
    }

    async fn match_app_addon_for_command_line(
        &self,
        command_line: &str,
    ) -> Option<(String, String, Vec<SupportedCommand>)> {
        let app_addons = self.app_addons.read().await;
        app_addons
            .iter()
            .find(|entry| command_matches_program(command_line, &entry.program_name))
            .map(|entry| {
                (
                    entry.program_name.clone(),
                    entry.addon_id.clone(),
                    entry.supported_commands.clone(),
                )
            })
    }

    async fn handle_shell_integration_sequence(
        &mut self,
        seq_type: &str,
        code: &str,
        data: &[u8],
    ) -> bool {
        if seq_type != "osc" || code != "133" {
            return false;
        }

        let Some(marker) = parse_shell_integration_marker(data) else {
            return false;
        };

        match marker.kind.as_str() {
            "C" => {
                if self.app.is_none() {
                    if let Some(command_line) = marker.command_line {
                        if let Some(matched) =
                            self.match_app_addon_for_command_line(&command_line).await
                        {
                            let (program_name, addon_id, supported_commands) = matched;
                            info!(
                                "App '{}' started via shell integration in session {} ({})",
                                program_name,
                                self.session_id,
                                self.display_title()
                            );
                            self.app_status = SessionAppStatus::Idle;
                            // Publish the detected app together with its initial
                            // status. `update_app` snapshots both fields into the
                            // SessionStateChanged event.
                            update_app(self, SessionApp(program_name));
                            self.active_addon = Some(ActiveAddon {
                                addon_id,
                                supported_commands,
                            });
                            self.app_launch_source = Some(AppLaunchSource::ShellIntegration);
                            return true;
                        }

                        let Some(program_name) = extract_command_executable(&command_line)
                            .map(|executable| normalize_program_name(&executable))
                            .filter(|program_name| {
                                is_shell_integration_app_candidate(program_name)
                            })
                        else {
                            return false;
                        };
                        info!(
                            session_id = %self.session_id,
                            program_name,
                            "App without addon started via shell integration"
                        );
                        // The shell proves that a foreground app is running,
                        // but without an addon its busy/idle state is unknown.
                        self.app_status = SessionAppStatus::None;
                        self.active_addon = None;
                        self.app_launch_source = Some(AppLaunchSource::ShellIntegration);
                        update_app(self, SessionApp(program_name));
                        return true;
                    }
                }
            }
            "D" => {
                if self.app_launch_source == Some(AppLaunchSource::ShellIntegration)
                    && !self.app.is_none()
                {
                    info!(
                        "App '{}' exited via shell integration in session {} ({})",
                        self.app.0,
                        self.session_id,
                        self.display_title()
                    );
                    self.app_status = SessionAppStatus::None;
                    if self.active_prompt.take().is_some() {
                        update_app_status(self, SessionAppStatus::None);
                    }
                    self.active_addon = None;
                    self.app_launch_source = None;
                    update_app(self, SessionApp::none());
                    update_app_status(self, SessionAppStatus::None);
                    return true;
                }
            }
            _ => {}
        }

        false
    }

    fn schedule_output_heartbeat(
        &self,
        generation: u64,
        delay: Duration,
        scheduler: &ObserverScheduler,
    ) {
        let tx = scheduler.tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            let _ = tx.send(ObserverCommand::OutputHeartbeatExpired { generation });
        });
    }

    fn handle_output_heartbeat_expired(&mut self, generation: u64, scheduler: &ObserverScheduler) {
        let Some(heartbeat) = self.output_heartbeat else {
            return;
        };
        if heartbeat.generation != generation || self.app_status != SessionAppStatus::Busy {
            return;
        }

        let remaining = heartbeat
            .idle_after
            .saturating_sub(heartbeat.last_output.elapsed());
        if !remaining.is_zero() {
            self.schedule_output_heartbeat(generation, remaining, scheduler);
            return;
        }

        self.output_heartbeat = None;
        self.app_status = SessionAppStatus::Idle;
        self.analysis_generation = self.analysis_generation.saturating_add(1);
        let analysis_generation = self.analysis_generation;
        update_app_status(self, SessionAppStatus::Idle);
        self.schedule_analysis(analysis_generation, scheduler);
    }

    fn schedule_analysis(&self, generation: u64, scheduler: &ObserverScheduler) {
        let notification_enabled = self.notification_enabled;
        let config_manager = self.config_manager.clone();
        let scheduler = scheduler.clone();
        tokio::spawn(async move {
            if !notification_enabled {
                return;
            }
            let config = config_manager.snapshot().await;
            let delay_secs = config.terminal.analysis_delay_seconds as u64;
            tokio::time::sleep(Duration::from_secs(delay_secs)).await;
            let Some(session) = scheduler.session.upgrade() else {
                return;
            };
            let snapshot = trim_snapshot_lines(&session.lock().await.terminal.grid_to_string());
            // The actor revalidates the generation before running the job.
            let _ = scheduler.tx.send(ObserverCommand::RunAnalysisPrepared {
                generation,
                snapshot,
            });
        });
    }

    fn schedule_prompt_grace(&self, generation: u64, scheduler: &ObserverScheduler) {
        let notification_enabled = self.notification_enabled;
        let config_manager = self.config_manager.clone();
        let scheduler = scheduler.clone();
        tokio::spawn(async move {
            if !notification_enabled {
                return;
            }
            let config = config_manager.snapshot().await;
            let grace_secs = config.terminal.confirmation_grace_seconds as u64;
            tokio::time::sleep(Duration::from_secs(grace_secs)).await;
            let _ = scheduler
                .tx
                .send(ObserverCommand::PromptGraceExpired { generation });
        });
    }

    fn prepare_analysis(&mut self, generation: u64) -> Option<AnalysisJob> {
        if !self.notification_enabled {
            return None;
        }

        if self.analysis_generation != generation || self.output_revision == self.analyzed_revision
        {
            //             debug!(
            //                 session_id = %self.session_id,
            //                 generation,
            //                 analysis_generation = self.analysis_generation,
            //                 output_revision = self.output_revision,
            //                 analyzed_revision = self.analyzed_revision,
            //                 auto_confirm_enabled = self.auto_confirm_enabled,
            //                 "prepare_analysis: skipped (generation mismatch or no new output)"
            //             );
            return None;
        }
        info!(
            session_id = %self.session_id,
            generation,
            output_revision = self.output_revision,
            prompt_generation = self.prompt_generation,
            auto_confirm_enabled = self.auto_confirm_enabled,
            active_prompt = self.active_prompt.is_some(),
            "prepare_analysis: starting analysis job"
        );
        self.analyzed_revision = self.output_revision;

        // Cooldown-based LLM retry: if the LLM was disabled after consecutive
        // failures but the cooldown has elapsed, retry it once for this job.
        // The result handler re-enables it on success or restarts the cooldown
        // on failure. `retry_cooldown_secs` is refreshed from the config by the
        // caller (schedule_analysis) before this runs.
        let mut llm_disabled = self.llm_disabled;
        let mut llm_retry = false;
        if llm_disabled {
            if let Some(disabled_at) = self.llm_disabled_at {
                let cooldown = Duration::from_secs(self.retry_cooldown_secs as u64);
                if disabled_at.elapsed() >= cooldown {
                    info!(
                        session_id = %self.session_id,
                        cooldown_secs = self.retry_cooldown_secs,
                        elapsed_secs = disabled_at.elapsed().as_secs(),
                        "LLM cooldown elapsed, retrying LLM for this analysis"
                    );
                    llm_disabled = false;
                    llm_retry = true;
                }
            }
        }

        Some(AnalysisJob {
            generation,
            output_revision: self.output_revision,
            prompt_generation: self.prompt_generation,
            input_revision: self.input_revision,
            session_id: self.session_id,
            auto_confirm_enabled: self.auto_confirm_enabled,
            llm_disabled,
            llm_retry,
            app: self.app.clone(),
            addon_id: self
                .active_addon
                .as_ref()
                .map(|addon| addon.addon_id.clone()),
            addon_host: self.addon_host.clone(),
            config_manager: self.config_manager.clone(),
        })
    }

    /// Returns `true` if auto-confirm was triggered and a carriage return should be
    /// forwarded to the session PTY.
    fn apply_analysis_result(
        &mut self,
        result: AnalysisResult,
        scheduler: &ObserverScheduler,
    ) -> bool {
        let stale = !self.notification_enabled
            || self.analysis_generation != result.generation
            || self.input_revision != result.input_revision;

        if stale {
            info!(
                session_id = %self.session_id,
                result_generation = result.generation,
                current_generation = self.analysis_generation,
                result_input_revision = result.input_revision,
                current_input_revision = self.input_revision,
                notification_enabled = self.notification_enabled,
                auto_confirm_enabled = self.auto_confirm_enabled,
                "apply_analysis_result: result STALE, discarding"
            );
            return false;
        }

        info!(
            session_id = %self.session_id,
            llm_succeeded = result.llm_succeeded,
            has_app_prompt = result.app_prompt.is_some(),
            auto_confirm_enabled = result.auto_confirm_enabled,
            active_prompt = self.active_prompt.is_some(),
            "apply_analysis_result: processing result"
        );

        if result.llm_succeeded {
            self.llm_failure_count = 0;
            // A cooldown retry that succeeds re-enables the LLM permanently.
            if result.llm_retry && self.llm_disabled {
                info!(
                    session_id = %self.session_id,
                    "LLM re-enabled after successful cooldown retry"
                );
                self.llm_disabled = false;
                self.llm_disabled_at = None;
            }
        } else if let Some(failure) = result.llm_failure {
            // A cooldown retry that fails keeps the LLM disabled and restarts
            // the cooldown so it will be retried again later.
            if result.llm_retry {
                info!(
                    session_id = %self.session_id,
                    error = %failure.error,
                    "LLM cooldown retry failed, restarting cooldown"
                );
                self.llm_disabled_at = Some(Instant::now());
            } else {
                self.llm_failure_count = self.llm_failure_count.saturating_add(1);
                info!(
                    session_id = %self.session_id,
                    llm_failure_count = self.llm_failure_count,
                    max_failures = failure.max_failures,
                    error = %failure.error,
                    "LLM call failed, incremented failure count"
                );
                if self.llm_failure_count >= failure.max_failures {
                    info!(
                        session_id = %self.session_id,
                        "LLM disabled for session after {} consecutive failures (threshold: {})",
                        self.llm_failure_count, failure.max_failures
                    );
                    self.llm_disabled = true;
                    self.llm_disabled_at = Some(Instant::now());
                }
            }
        }

        match result.app_prompt {
            Some(prompt) => {
                let signature = prompt_signature(&prompt);
                let session_id = self.session_id;
                let should_auto_confirm_now =
                    should_auto_confirm(result.auto_confirm_enabled, &prompt);
                let next_app_status = prompt.app_status;
                info!(
                    session_id = %session_id,
                    prompt_status = ?prompt.app_status,
                    prompt_summary = %prompt.summary,
                    auto_confirm_enabled = result.auto_confirm_enabled,
                    should_auto_confirm_now,
                    active_prompt_state = match self.active_prompt {
                        Some(ref p) => format!(
                            "Some(sig={}, auto_confirmed={}, auto_confirmed_revision={:?})",
                            p.signature, p.auto_confirmed, p.auto_confirmed_revision
                        ),
                        None => "None".to_string(),
                    },
                    "apply_analysis_result: prompt detected, deciding auto-confirm"
                );
                let mut send_auto_confirm = false;
                match self.active_prompt.as_mut() {
                    Some(active) if active.signature == signature => {
                        info!(
                            session_id = %session_id,
                            "apply_analysis_result: same prompt signature, re-checking auto-confirm"
                        );
                        let already_auto_confirmed_this_revision =
                            active.auto_confirmed_revision == Some(result.output_revision);
                        if should_auto_confirm(result.auto_confirm_enabled, &active.prompt)
                            && !already_auto_confirmed_this_revision
                        {
                            active.auto_confirmed = true;
                            active.auto_confirmed_revision = Some(result.output_revision);
                            log_auto_confirm_executed(&self.session_id, &active.prompt);
                            let _ = self
                                .event_tx
                                .send(TerminalObserverEvent::AutoConfirmRequested { session_id });
                            return true;
                        }
                    }
                    Some(_) => {
                        if self.active_prompt.take().is_some() {
                            update_app_status(self, SessionAppStatus::Idle);
                        }
                        self.active_prompt = Some(PromptState {
                            prompt,
                            signature,
                            notified: false,
                            auto_confirmed: should_auto_confirm_now,
                            auto_confirmed_revision: should_auto_confirm_now
                                .then_some(result.output_revision),
                        });
                        self.prompt_generation = self.prompt_generation.saturating_add(1);
                        let prompt_generation = self.prompt_generation;
                        log_app_waiting_approval(self, self.active_prompt.as_ref());
                        update_app_status(self, next_app_status);
                        self.schedule_prompt_grace(prompt_generation, scheduler);
                        send_auto_confirm = should_auto_confirm_now;
                    }
                    None => {
                        self.active_prompt = Some(PromptState {
                            prompt,
                            signature,
                            notified: false,
                            auto_confirmed: should_auto_confirm_now,
                            auto_confirmed_revision: should_auto_confirm_now
                                .then_some(result.output_revision),
                        });
                        self.prompt_generation = self.prompt_generation.saturating_add(1);
                        let prompt_generation = self.prompt_generation;
                        log_app_waiting_approval(self, self.active_prompt.as_ref());
                        update_app_status(self, next_app_status);
                        self.schedule_prompt_grace(prompt_generation, scheduler);
                        send_auto_confirm = should_auto_confirm_now;
                    }
                }

                update_app_status(self, next_app_status);
                if send_auto_confirm {
                    if let Some(active) = self.active_prompt.as_ref() {
                        log_auto_confirm_executed(&self.session_id, &active.prompt);
                    }
                    let _ = self
                        .event_tx
                        .send(TerminalObserverEvent::AutoConfirmRequested { session_id });
                    return true;
                }
            }
            None => {
                if self.active_prompt.take().is_some() {
                    update_app_status(self, SessionAppStatus::Idle);
                }
                update_app_status(self, SessionAppStatus::Idle);
            }
        }
        false
    }

    fn emit_prompt_attention_if_pending(&mut self, generation: u64) {
        if self.prompt_generation != generation {
            return;
        }

        if !self.notification_enabled {
            return;
        }

        let title = self.display_title();
        let Some(prompt) = self.active_prompt.as_mut() else {
            return;
        };
        if prompt.notified {
            return;
        }

        let prompt_match = prompt.prompt.clone();
        let attention = TerminalAttention {
            session_id: self.session_id,
            title,
            kind: match prompt_match.app_status {
                SessionAppStatus::Selection => TerminalAttentionKind::Selection,
                _ => TerminalAttentionKind::Confirmation,
            },
            summary: prompt_match.summary.clone(),
            matched_line: prompt_match.matched_line.clone(),
        };
        prompt.notified = true;
        let _ = self
            .event_tx
            .send(TerminalObserverEvent::AttentionRequired(attention));
    }
}

async fn run_analysis_job(job: AnalysisJob, snapshot: String) -> AnalysisResult {
    let mut llm_succeeded = false;
    let mut app_prompt: Option<PromptMatch> = None;
    let mut llm_failure = None;
    let llm_retry = job.llm_retry;

    // Call the LLM unless it is disabled — except for a cooldown-expired retry,
    // which proceeds so the caller can decide to re-enable it.
    if !job.llm_disabled || job.llm_retry {
        if job.llm_retry {
            info!(
                session_id = %job.session_id,
                "LLM cooldown retry: attempting LLM analysis"
            );
        }
        let config = job.config_manager.snapshot().await;
        let max_failures = config.llm.terminal.provider.max_failures as u32;
        let session_id = job.session_id;
        let addon_host = job.addon_host.clone();
        let llm_snapshot = snapshot.clone();
        let app = job.app.clone();
        let llm_result = tokio::task::spawn_blocking(move || {
            analyze_with_llm(session_id, &addon_host, &llm_snapshot, &config, &app)
        })
        .await
        .unwrap_or_else(|error| Some(Err(format!("LLM worker failed: {error}"))));

        match llm_result {
            Some(Ok((status, message))) => {
                llm_succeeded = true;
                info!(
                    session_id = %job.session_id,
                    status = ?status,
                    message = message.as_deref().unwrap_or(""),
                    "LLM analysis succeeded"
                );
                if status != SessionAppStatus::None
                    && status != SessionAppStatus::Idle
                    && status != SessionAppStatus::Busy
                {
                    app_prompt = Some(PromptMatch {
                        app_status: status,
                        summary: message.unwrap_or_default(),
                        matched_line: String::new(),
                    });
                }
            }
            Some(Err(error)) => {
                llm_failure = Some(LlmFailure {
                    error,
                    max_failures,
                });
            }
            None => {}
        }
    } else {
        debug!(
            session_id = %job.session_id,
            "LLM is disabled for this session, skipping"
        );
    }

    if !llm_succeeded || app_prompt.is_none() {
        debug!(
            session_id = %job.session_id,
            llm_succeeded = llm_succeeded,
            has_app_prompt = app_prompt.is_some(),
            "Falling back to addon-based detection"
        );
        if let Some(addon_id) = &job.addon_id {
            let result = run_detect_output(&job.addon_host, addon_id, &snapshot);
            if result.status != SessionAppStatus::None
                && result.status != SessionAppStatus::Idle
                && result.status != SessionAppStatus::Busy
            {
                app_prompt = Some(PromptMatch {
                    app_status: result.status,
                    summary: result.message.clone().unwrap_or_default(),
                    matched_line: String::new(),
                });
            }
        }
    }

    // Log analysis result to file
    if job.auto_confirm_enabled {
        log_auto_confirm_analysis(&job.session_id, llm_succeeded, &app_prompt, &snapshot);
    }

    AnalysisResult {
        generation: job.generation,
        output_revision: job.output_revision,
        prompt_generation: job.prompt_generation,
        input_revision: job.input_revision,
        auto_confirm_enabled: job.auto_confirm_enabled,
        app_prompt,
        llm_succeeded,
        llm_failure,
        llm_retry,
    }
}

/// Attempt to classify terminal state using the configured LLM.
///
/// Returns:
/// - `Some(Ok((status, message)))` on successful LLM call
/// - `Some(Err(description))` when LLM is configured but the call failed
/// - `None` when LLM is not configured (no addon resolved)
fn analyze_with_llm(
    session_id: SessionId,
    addon_host: &AddonHost,
    snapshot: &str,
    config: &TerminalServerConfig,
    app: &SessionApp,
) -> Option<Result<(SessionAppStatus, Option<String>), String>> {
    let addon_id = match resolve_llm_addon_id(config, "terminal") {
        Some(id) => id,
        None => {
            info!(
                session_id = %session_id,
                enabled = config.llm.terminal.provider.enabled,
                provider = ?config.llm.terminal.provider.provider,
                "LLM not configured for terminal — resolve_llm_addon_id returned None"
            );
            return None;
        }
    };

    debug!(session_id = %session_id, addon_id = %addon_id, "Starting LLM analysis");

    let mut system_prompt = config
        .llm
        .terminal
        .provider
        .user_config
        .get("system_prompt")
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| LLM_SYSTEM_PROMPT.to_string());
    system_prompt.push_str(&terminal_language_instruction(
        &config.llm.terminal.output_language,
    ));

    let messages = serde_json::json!([
        { "role": "system", "content": system_prompt },
        { "role": "user", "content": snapshot }
    ])
    .to_string();

    let options = serde_json::json!({
        "temperature": 0.1,
        "max_tokens": 32768
    })
    .to_string();

    info!(
        session_id = %session_id,
        addon_id = %addon_id,
        snapshot_len = snapshot.len(),
        "Sending LLM chat completion request"
    );

    let stats_context = LlmRequestContext {
        category: if app.0.trim().is_empty() || app.0 == "none" {
            "app analysis".to_string()
        } else {
            format!("app {} analysis", app.0)
        },
    };
    let result = run_chat_completion(
        addon_host,
        config,
        &addon_id,
        &messages,
        &options,
        Some(&config.llm.terminal.provider.user_config),
        None,
        Some(&stats_context),
    );

    if !result.success {
        let error = result.error.unwrap_or_else(|| "unknown error".to_string());
        info!(
            session_id = %session_id,
            "LLM analysis failed: {}", error
        );
        return Some(Err(error));
    }

    let content = match result.content {
        Some(c) => c,
        None => {
            debug!(session_id = %session_id, "LLM returned success but no content");
            return Some(Err("no content in response".to_string()));
        }
    };

    debug!(
        session_id = %session_id,
        raw_content = %content,
        "LLM returned content"
    );

    let parsed: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => return Some(Err(format!("JSON parse error: {}", e))),
    };
    let status_str = match parsed.get("status").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Some(Err("missing 'status' field in LLM response".to_string())),
    };
    let message = parsed
        .get("message")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let status = match status_str {
        "idle" => SessionAppStatus::Idle,
        "confirm" => SessionAppStatus::ConfirmingNormal,
        "confirm_special" => SessionAppStatus::ConfirmingSpecial,
        "selection" => SessionAppStatus::Selection,
        _ => {
            return Some(Err(format!("unknown status: {}", status_str)));
        }
    };

    debug!(
        session_id = %session_id,
        status = ?status,
        message = message.as_deref().unwrap_or(""),
        "LLM analysis completed"
    );

    Some(Ok((status, message)))
}

// ---------------------------------------------------------------------------
// Free functions for updating observer state
// ---------------------------------------------------------------------------

fn update_app(observer: &mut SessionObserver, next_app: SessionApp) {
    if observer.app != next_app {
        let prev_empty = observer.app.is_none();
        let next_empty = next_app.is_none();

        if prev_empty && !next_empty {
            info!(
                "App '{}' started in session {} ({})",
                next_app.0,
                observer.session_id,
                observer.display_title()
            );
        } else if !prev_empty && next_empty {
            info!(
                "App '{}' exited in session {} ({})",
                observer.app.0,
                observer.session_id,
                observer.display_title()
            );
        }

        observer.app = next_app;
    }

    publish_session_state(observer);
}

fn update_app_status(observer: &mut SessionObserver, next_app_status: SessionAppStatus) {
    observer.app_status = next_app_status;
    publish_session_state(observer);
}

/// Publish the complete app observation as one authoritative snapshot. Keeping
/// the last published tuple lets summary-only changes propagate even when the
/// app and status themselves stay unchanged.
fn publish_session_state(observer: &mut SessionObserver) {
    let next_state = PublishedSessionState {
        app: observer.app.clone(),
        app_status: observer.app_status,
        app_status_summary: status_summary_for(observer),
    };
    if observer.published_state == next_state {
        return;
    }

    observer.published_state = next_state.clone();
    let _ = observer
        .event_tx
        .send(TerminalObserverEvent::SessionStateChanged {
            session_id: observer.session_id,
            app: next_state.app,
            app_status: next_state.app_status,
            app_status_summary: next_state.app_status_summary,
        });
}

/// Return the pending-prompt summary when the observer is in a confirm/select
/// state, otherwise `None`. Used to populate the snapshotted
/// `app_status_summary` so newly-connecting clients can see what is pending.
fn status_summary_for(observer: &SessionObserver) -> Option<String> {
    match observer.app_status {
        SessionAppStatus::ConfirmingNormal
        | SessionAppStatus::ConfirmingSpecial
        | SessionAppStatus::Selection => observer
            .active_prompt
            .as_ref()
            .map(|p| p.prompt.summary.clone()),
        _ => None,
    }
}

fn log_app_waiting_approval(session: &SessionObserver, prompt: Option<&PromptState>) {
    let Some(prompt) = prompt else {
        return;
    };

    debug!(
        "App '{}' is waiting for approval in session {} ({}): {}",
        session.app.0,
        session.session_id,
        session.display_title(),
        prompt.prompt.matched_line
    );
}

/// Strip a shell prompt prefix from a line, returning only the command portion.
///
/// Recognized patterns:
/// - Windows: `X:\path>` or `PS X:\path>` — a drive-letter path ending with `> `
/// - Unix: `/path$ `, `/path# `, or `/path% ` — a path ending with `$`, `#`, or `%`
fn strip_prompt(line: &str) -> Option<String> {
    // Windows-style: look for pattern like `X:\...>` where the segment between
    // the last `\` and `>` contains only path-safe characters (no spaces longer than 1 word).
    if let Some(gt_pos) = line.rfind('>') {
        let before_gt = &line[..gt_pos];
        let after_gt = &line[gt_pos + 1..];
        // There must be a `\` before the `>` (a drive-letter path). The
        // command follows `>` directly — cmd and PowerShell prompts do not
        // insert a space (`C:\path>pi`), so none is required here.
        if before_gt.contains('\\') {
            // Check the segment between last `\` and `>` is a short path fragment
            // (not something like a sentence)
            let after_backslash = &before_gt[before_gt.rfind('\\').unwrap() + 1..];
            // Path fragment should be short and not contain typical sentence chars
            if after_backslash.len() < 64
                && !after_backslash.contains('.')
                && !after_backslash.contains(';')
            {
                let cmd = after_gt.trim();
                if !cmd.is_empty() {
                    return Some(cmd.to_lowercase());
                }
            }
        }
    }

    // Unix-style: look for `/path$ `, `/path# `, or `/path% `. Long prompts
    // can wrap, so the current visual line may only contain the tail, e.g.
    // `repo-name$ code .`.
    for marker in &['$', '#', '%'] {
        if let Some(pos) = line.rfind(*marker) {
            let before = &line[..pos];
            let after = &line[pos + 1..];
            // Marker must be followed by space or end-of-line, and the before
            // part must look like a prompt. Accept `/` (absolute path),
            // `:` (user@host:path pattern like `user@host:~$`), or
            // `~` (home directory shorthand), or `@` (user@hostname pattern
            // like macOS zsh: `user@host xxx %`).
            let prompt_like = before.contains('/')
                || before.contains(':')
                || before.contains('~')
                || before.contains('@')
                || looks_like_wrapped_unix_prompt_tail(before);
            if (after.is_empty() || after.starts_with(' ')) && prompt_like {
                let cmd = after.trim();
                if !cmd.is_empty() {
                    return Some(cmd.to_lowercase());
                }
            }
        }
    }

    None
}

fn looks_like_wrapped_unix_prompt_tail(before_marker: &str) -> bool {
    let tail = before_marker.trim();
    !tail.is_empty()
        && tail.len() <= 80
        && !tail.contains(char::is_whitespace)
        && !tail.contains(';')
        && !tail.contains('|')
        && !tail.contains('>')
        && !tail.contains('<')
}

struct ShellIntegrationMarker {
    kind: String,
    command_line: Option<String>,
}

fn parse_shell_integration_marker(data: &[u8]) -> Option<ShellIntegrationMarker> {
    let data = std::str::from_utf8(data).ok()?;
    let mut parts = data.split(';');
    let kind = parts.next()?.trim().to_string();
    let mut command_line = None;
    for part in parts {
        if let Some(encoded) = part.strip_prefix("cmdline_url=") {
            command_line = percent_decode(encoded);
        } else if let Some(raw) = part.strip_prefix("cmdline=") {
            command_line = Some(raw.to_string());
        }
    }
    Some(ShellIntegrationMarker { kind, command_line })
}

fn percent_decode(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hi = from_hex(bytes[i + 1])?;
                let lo = from_hex(bytes[i + 2])?;
                output.push((hi << 4) | lo);
                i += 3;
            }
            b'+' => {
                output.push(b' ');
                i += 1;
            }
            byte => {
                output.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8(output).ok()
}

fn from_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn command_matches_program(command_line: &str, program_name: &str) -> bool {
    let Some(executable) = extract_command_executable(command_line) else {
        return false;
    };
    let executable = normalize_program_name(&executable);
    let program_name = normalize_program_name(program_name);
    executable == program_name
}

fn extract_command_executable(command_line: &str) -> Option<String> {
    let trimmed = command_line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let trimmed = trimmed
        .strip_prefix("& ")
        .or_else(|| trimmed.strip_prefix(". "))
        .unwrap_or(trimmed)
        .trim_start();
    if trimmed.is_empty() {
        return None;
    }

    let token = if let Some(rest) = trimmed.strip_prefix('"') {
        rest.split('"').next()?
    } else if let Some(rest) = trimmed.strip_prefix('\'') {
        rest.split('\'').next()?
    } else {
        trimmed.split_whitespace().next()?
    };
    Some(token.to_string())
}

fn normalize_program_name(program: &str) -> String {
    let file_name = program
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(program)
        .trim_matches('"')
        .trim_matches('\'');
    let path = Path::new(file_name);
    path.file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(file_name)
        .to_ascii_lowercase()
}

/// Shell DEBUG traps can observe commands executed by prompt hooks as well as
/// user-entered foreground commands. Do not expose those implementation
/// details (or shell grammar) as running applications.
fn is_shell_integration_app_candidate(program_name: &str) -> bool {
    if program_name.is_empty()
        || program_name.starts_with("__vterm_")
        || program_name.starts_with("__vte_")
    {
        return false;
    }

    !matches!(
        program_name,
        "case"
            | "esac"
            | "if"
            | "then"
            | "elif"
            | "else"
            | "fi"
            | "for"
            | "select"
            | "while"
            | "until"
            | "do"
            | "done"
            | "in"
            | "function"
            | "time"
            | "coproc"
    )
}

// ---------------------------------------------------------------------------
// Auto-confirm log file
// ---------------------------------------------------------------------------

fn auto_confirm_log_path(session_id: &SessionId) -> Option<PathBuf> {
    let log_dir = crate::session::session_log_dir(*session_id);
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        warn!("Failed to create log dir {:?}: {}", log_dir, e);
        return None;
    }
    Some(log_dir.join("auto_confirm.log"))
}

fn append_auto_confirm_log(path: &PathBuf, entry: &str) {
    if let Err(e) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| f.write_all(entry.as_bytes()))
    {
        warn!("Failed to write auto-confirm log {:?}: {}", path, e);
    }
}

/// Maximum number of terminal lines sent to the LLM for analysis. The most
/// recent (bottom) lines are kept because agent prompts and confirmations
/// appear at the end of the visible screen.
const SNAPSHOT_MAX_LINES: usize = 40;

fn trim_snapshot_lines(snapshot: &str) -> String {
    let mut lines: Vec<&str> = snapshot.lines().map(|line| line.trim_end()).collect();
    // Keep only the last SNAPSHOT_MAX_LINES so the LLM focuses on the most
    // recent output (where confirmations live) without exceeding the budget.
    if lines.len() > SNAPSHOT_MAX_LINES {
        let drop = lines.len() - SNAPSHOT_MAX_LINES;
        lines.drain(..drop);
    }
    lines.join("\n")
}

fn log_auto_confirm_analysis(
    session_id: &SessionId,
    llm_succeeded: bool,
    prompt: &Option<PromptMatch>,
    snapshot: &str,
) {
    let Some(log_path) = auto_confirm_log_path(session_id) else {
        return;
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    let status_str = prompt
        .as_ref()
        .map(|p| match p.app_status {
            SessionAppStatus::ConfirmingNormal => "confirm",
            SessionAppStatus::ConfirmingSpecial => "confirm_special",
            SessionAppStatus::Selection => "selection",
            _ => "other",
        })
        .unwrap_or("idle");

    let summary = prompt.as_ref().map(|p| p.summary.as_str()).unwrap_or("");

    let snapshot_tail = if snapshot.len() > 500 {
        let mut start = snapshot.len() - 500;
        while !snapshot.is_char_boundary(start) {
            start -= 1;
        }
        &snapshot[start..]
    } else {
        snapshot
    };

    let entry = format!(
        "[{timestamp}] ANALYSIS llm={llm_succeeded} status={status_str} summary={summary}\n  snapshot: {snapshot_tail}\n\n"
    );
    append_auto_confirm_log(&log_path, &entry);
}

fn log_auto_confirm_executed(session_id: &SessionId, prompt: &PromptMatch) {
    let Some(log_path) = auto_confirm_log_path(session_id) else {
        return;
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    let status_str = match prompt.app_status {
        SessionAppStatus::ConfirmingNormal => "confirm",
        SessionAppStatus::ConfirmingSpecial => "confirm_special",
        SessionAppStatus::Selection => "selection",
        _ => "other",
    };

    let entry = format!(
        "[{timestamp}] EXECUTED auto_confirm=true status={status_str} summary={}\n\n",
        prompt.summary
    );
    append_auto_confirm_log(&log_path, &entry);
}

#[cfg(test)]
mod tests;
