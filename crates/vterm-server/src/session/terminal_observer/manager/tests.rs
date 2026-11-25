use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::sync::mpsc;

use vterm_common::{
    SessionAppStatus, SessionId, TerminalAttention, TerminalAttentionKind, TerminalSize,
};
use vterm_core::emulator::Terminal;

use crate::addon::AddonHost;
use crate::config::{ConfigManager, ConfigPatchRequest, TerminalConfigPatch};
use crate::session::SupportedCommand;

use super::*;

#[test]
fn terminal_language_instruction_auto_when_empty_or_auto() {
    // Empty or "auto" (any case, any surrounding whitespace) asks the model to
    // mirror the dominant language of the terminal output.
    for value in ["", "auto", "AUTO", "  auto  "] {
        let instruction = terminal_language_instruction(value);
        assert!(
            instruction.starts_with("\n\nLanguage: detect the dominant"),
            "expected auto directive for {value:?}, got: {instruction:?}"
        );
        // The status enum must stay English even in auto mode.
        assert!(instruction.contains("\"status\" field values"));
    }
}

#[test]
fn terminal_language_instruction_uses_configured_language() {
    // The configured value is free-form text (a code or a language name) and is
    // used verbatim.
    assert_eq!(
        terminal_language_instruction("en"),
        "\n\nLanguage: write the \"message\" summary in en. Keep the \"status\" field values in English."
    );
    assert_eq!(
        terminal_language_instruction("中文"),
        "\n\nLanguage: write the \"message\" summary in 中文. Keep the \"status\" field values in English."
    );
}

/// Verify prompt parsing keeps the command content for conda-wrapped Unix prompts.
#[test]
fn strip_prompt_handles_conda_linux_prompt() {
    assert_eq!(
        strip_prompt("(base) user@devbox:~/Documents/projects/vibe-terminal/vibe-terminal$ code ."),
        Some("code .".to_string())
    );
}

/// Verify prompt parsing works when only the wrapped tail of a Unix prompt is visible.
#[test]
fn strip_prompt_handles_wrapped_linux_prompt_tail() {
    assert_eq!(
        strip_prompt("vibe-terminal$ code ."),
        Some("code .".to_string())
    );
}

/// Verify prompt parsing keeps the command for cmd-style prompts, where the
/// command follows `>` without a separating space.
#[test]
fn strip_prompt_handles_cmd_prompt_without_space() {
    assert_eq!(strip_prompt(r"C:\Users\user>pi"), Some("pi".to_string()));
    assert_eq!(
        strip_prompt(r"C:\Users\user>pi --resume"),
        Some("pi --resume".to_string())
    );
    assert_eq!(
        strip_prompt(r"PS C:\Users\user>pi"),
        Some("pi".to_string())
    );
    assert_eq!(
        strip_prompt(r"PS C:\Users\user> pi"),
        Some("pi".to_string())
    );
}

/// Verify OSC 133 command-start markers decode the integrated command line.
#[test]
fn parse_shell_integration_marker_decodes_cmdline_url() {
    let marker = parse_shell_integration_marker(b"C;aid=42;cmdline_url=claude%20--print")
        .expect("expected OSC 133 marker");
    assert_eq!(marker.kind, "C");
    assert_eq!(marker.command_line.as_deref(), Some("claude --print"));
}

#[cfg(unix)]
#[test]
fn command_is_available_uses_login_shell_path() {
    let home = tempfile::tempdir().expect("test home should be created");
    let bin = home.path().join("bin");
    std::fs::create_dir(&bin).expect("test bin should be created");
    std::fs::write(bin.join("login-only-app"), "").expect("test app should be created");
    std::fs::write(
        home.path().join(".bash_profile"),
        "export PATH=\"$HOME/bin:/usr/bin:/bin\"\n",
    )
    .expect("test profile should be written");
    let env = std::collections::HashMap::from([(
        "HOME".to_string(),
        home.path().to_string_lossy().into_owned(),
    )]);

    let login_env = crate::session::shell_integration::bash_login_environment("bash", &env);
    let path = login_env.get("PATH").map(std::ffi::OsString::from);
    assert!(command_is_available("login-only-app", path.as_deref()));
    assert!(!command_is_available("missing-app", path.as_deref()));
}

/// Verify command matching tolerates quoted executable paths emitted by shell integration.
#[test]
fn command_matches_program_handles_quoted_paths() {
    assert!(command_matches_program(
        r#""C:\Users\user\AppData\Local\Programs\Claude\claude.exe" --dangerously-skip-permissions"#,
        "claude"
    ));
}

/// Build an isolated config manager backed by a unique temp file for tests.
async fn test_config_manager() -> Arc<ConfigManager> {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("observer-tests-{unique_suffix}.json"));
    Arc::new(
        ConfigManager::load_from_path(PathBuf::from(&path))
            .await
            .unwrap(),
    )
}

fn test_scheduler() -> (ObserverScheduler, mpsc::UnboundedReceiver<ObserverCommand>) {
    let (tx, rx) = mpsc::unbounded_channel();
    (
        ObserverScheduler {
            tx,
            session: std::sync::Weak::new(),
        },
        rx,
    )
}

#[tokio::test]
async fn legacy_detection_publishes_app_with_idle_status() {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let manager = TerminalObserverManager::new(
        test_config_manager().await,
        Arc::new(AddonHost::new()),
        event_tx,
    );
    manager
        .insert_test_app_addon("claude-code", "claude", Vec::new())
        .await;

    let observer = manager.create_observer(
        SessionId::new(),
        "Observer Test".to_string(),
        None,
        AppDetectionMode::LegacyPrompt,
    );
    let detected = observer
        .lock()
        .await
        .record_input(true, Some("/home/user$ claude".to_string()))
        .await;

    assert!(detected);
    let event = event_rx.try_recv().expect("expected session state event");
    assert!(matches!(
        event,
        TerminalObserverEvent::SessionStateChanged {
            app,
            app_status: SessionAppStatus::Idle,
            ..
        } if app.0 == "claude"
    ));
}

async fn spinner_test_observer(
    addon_id: &str,
    program_name: &str,
    event_tx: mpsc::UnboundedSender<TerminalObserverEvent>,
) -> Arc<Mutex<SessionObserver>> {
    let manager = TerminalObserverManager::new(
        test_config_manager().await,
        Arc::new(AddonHost::new()),
        event_tx,
    );
    manager
        .insert_test_app_addon(addon_id, program_name, Vec::new())
        .await;
    let observer = manager.create_observer(
        SessionId::new(),
        "Observer Test".to_string(),
        None,
        AppDetectionMode::ShellIntegration,
    );
    observer
        .lock()
        .await
        .handle_shell_integration_sequence(
            "osc",
            "133",
            format!("C;cmdline={program_name}").as_bytes(),
        )
        .await;
    observer
}

#[tokio::test]
async fn qwen_spinner_output_publishes_busy_status() {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let observer = spinner_test_observer("qwen-code", "qwen", event_tx).await;
    let (scheduler, _rx) = test_scheduler();
    let _ = event_rx.try_recv();

    observer
        .lock()
        .await
        .record_output(
            "\x1b[38;2;128;128;128m  ⠸ Thinking…\x1b[0m\r\n────────\r\n> prompt\r\n➜ qwen · deepseek-flash".as_bytes(),
            ObservedTerminalState::default(),
            &scheduler,
        )
        .await;

    let event = event_rx.try_recv().expect("expected busy state event");
    assert!(matches!(
        event,
        TerminalObserverEvent::SessionStateChanged {
            app_status: SessionAppStatus::Busy,
            ..
        }
    ));
}

#[tokio::test]
async fn qwen_stopped_spinner_returns_to_idle_regardless_of_text() {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let observer = spinner_test_observer("qwen-code", "qwen", event_tx).await;
    let (scheduler, mut rx) = test_scheduler();
    let _ = event_rx.try_recv();

    observer
        .lock()
        .await
        .record_output(
            "  ⠏ 等待用户确认...".as_bytes(),
            ObservedTerminalState::default(),
            &scheduler,
        )
        .await;
    assert!(matches!(
        event_rx.try_recv(),
        Ok(TerminalObserverEvent::SessionStateChanged {
            app_status: SessionAppStatus::Busy,
            ..
        })
    ));

    let command = tokio::time::timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("heartbeat timer must fire")
        .expect("scheduler channel must stay open");
    let ObserverCommand::OutputHeartbeatExpired { generation } = command else {
        panic!("expected heartbeat expiration");
    };
    observer
        .lock()
        .await
        .handle_output_heartbeat_expired(generation, &scheduler);

    assert_eq!(observer.lock().await.app_status, SessionAppStatus::Idle);
    assert!(matches!(
        event_rx.try_recv(),
        Ok(TerminalObserverEvent::SessionStateChanged {
            app_status: SessionAppStatus::Idle,
            ..
        })
    ));
}

#[tokio::test]
async fn pi_spinner_stops_publishing_and_returns_to_idle() {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let observer = spinner_test_observer("pi", "pi", event_tx).await;
    let (scheduler, mut rx) = test_scheduler();
    let _ = event_rx.try_recv();

    observer
        .lock()
        .await
        .record_output(
            "\x1b[38;2;128;128;128m── ⠸ Working ─────\x1b[0m".as_bytes(),
            ObservedTerminalState::default(),
            &scheduler,
        )
        .await;
    assert!(matches!(
        event_rx.try_recv(),
        Ok(TerminalObserverEvent::SessionStateChanged {
            app_status: SessionAppStatus::Busy,
            ..
        })
    ));

    let command = tokio::time::timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("heartbeat timer must fire")
        .expect("scheduler channel must stay open");
    let ObserverCommand::OutputHeartbeatExpired { generation } = command else {
        panic!("expected heartbeat expiration");
    };
    observer
        .lock()
        .await
        .handle_output_heartbeat_expired(generation, &scheduler);

    assert_eq!(observer.lock().await.app_status, SessionAppStatus::Idle);
    assert!(matches!(
        event_rx.try_recv(),
        Ok(TerminalObserverEvent::SessionStateChanged {
            app_status: SessionAppStatus::Idle,
            ..
        })
    ));
}

#[tokio::test]
async fn pi_spinner_frames_extend_the_idle_timeout() {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let observer = spinner_test_observer("pi", "pi", event_tx).await;
    let (scheduler, mut rx) = test_scheduler();
    let _ = event_rx.try_recv();

    for frame in ["── ⠋ Working ─────", "── ⠙ Working ─────"] {
        observer
            .lock()
            .await
            .record_output(
                frame.as_bytes(),
                ObservedTerminalState::default(),
                &scheduler,
            )
            .await;
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    let _ = event_rx.try_recv();

    let command = tokio::time::timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("heartbeat timer must fire")
        .expect("scheduler channel must stay open");
    let ObserverCommand::OutputHeartbeatExpired { generation } = command else {
        panic!("expected heartbeat expiration");
    };
    observer
        .lock()
        .await
        .handle_output_heartbeat_expired(generation, &scheduler);
    assert_eq!(observer.lock().await.app_status, SessionAppStatus::Busy);
    assert!(event_rx.try_recv().is_err());

    let command = tokio::time::timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("rescheduled heartbeat timer must fire")
        .expect("scheduler channel must stay open");
    let ObserverCommand::OutputHeartbeatExpired { generation } = command else {
        panic!("expected rescheduled heartbeat expiration");
    };
    observer
        .lock()
        .await
        .handle_output_heartbeat_expired(generation, &scheduler);
    assert_eq!(observer.lock().await.app_status, SessionAppStatus::Idle);
}

#[tokio::test]
async fn pi_osc_busy_disables_spinner_idle_timeout() {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let observer = spinner_test_observer("pi", "pi", event_tx).await;
    let (scheduler, mut rx) = test_scheduler();
    let _ = event_rx.try_recv();

    observer
        .lock()
        .await
        .record_output(
            "── ⠋ Working ─────".as_bytes(),
            ObservedTerminalState::default(),
            &scheduler,
        )
        .await;
    let _ = event_rx.try_recv();
    observer
        .lock()
        .await
        .handle_sequence("osc".into(), "9".into(), b"4;3".to_vec(), &scheduler)
        .await;

    let command = tokio::time::timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("heartbeat timer must fire")
        .expect("scheduler channel must stay open");
    let ObserverCommand::OutputHeartbeatExpired { generation } = command else {
        panic!("expected heartbeat expiration");
    };
    observer
        .lock()
        .await
        .handle_output_heartbeat_expired(generation, &scheduler);
    assert_eq!(observer.lock().await.app_status, SessionAppStatus::Busy);
    assert!(event_rx.try_recv().is_err());
}

#[tokio::test]
async fn opencode_mini_spinner_heartbeats_return_to_idle() {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let observer = spinner_test_observer("opencode", "opencode", event_tx).await;
    let (scheduler, mut rx) = test_scheduler();
    let _ = event_rx.try_recv();

    observer
        .lock()
        .await
        .record_output("🯦".as_bytes(), ObservedTerminalState::default(), &scheduler)
        .await;
    assert_eq!(observer.lock().await.app_status, SessionAppStatus::Idle);
    assert!(event_rx.try_recv().is_err());
    assert!(rx.try_recv().is_err());

    observer
        .lock()
        .await
        .record_output(
            "\x1b[38;2;128;128;128m🯦 esc stop · Build · GPT-5\x1b[0m".as_bytes(),
            ObservedTerminalState::default(),
            &scheduler,
        )
        .await;
    assert!(matches!(
        event_rx.try_recv(),
        Ok(TerminalObserverEvent::SessionStateChanged {
            app_status: SessionAppStatus::Busy,
            ..
        })
    ));

    tokio::time::sleep(Duration::from_millis(600)).await;
    observer
        .lock()
        .await
        .record_output("🯧".as_bytes(), ObservedTerminalState::default(), &scheduler)
        .await;

    let command = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("heartbeat timer must fire")
        .expect("scheduler channel must stay open");
    let ObserverCommand::OutputHeartbeatExpired { generation } = command else {
        panic!("expected heartbeat expiration");
    };
    observer
        .lock()
        .await
        .handle_output_heartbeat_expired(generation, &scheduler);
    assert_eq!(observer.lock().await.app_status, SessionAppStatus::Busy);

    let command = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("rescheduled heartbeat timer must fire")
        .expect("scheduler channel must stay open");
    let ObserverCommand::OutputHeartbeatExpired { generation } = command else {
        panic!("expected rescheduled heartbeat expiration");
    };
    observer
        .lock()
        .await
        .handle_output_heartbeat_expired(generation, &scheduler);
    assert_eq!(observer.lock().await.app_status, SessionAppStatus::Idle);
    assert!(matches!(
        event_rx.try_recv(),
        Ok(TerminalObserverEvent::SessionStateChanged {
            app_status: SessionAppStatus::Idle,
            ..
        })
    ));
}

/// A shell respawn (the previous shell died while an app was running, so no
/// exit marker was observed) must reset the observer's app tracking —
/// otherwise the `app.is_none()` gate blocks every later shell-integration
/// detection for the rest of the session.
#[tokio::test]
async fn reset_app_tracking_allows_detection_after_shell_respawn() {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let manager = TerminalObserverManager::new(
        test_config_manager().await,
        Arc::new(AddonHost::new()),
        event_tx,
    );
    manager.insert_test_app_addon("pi", "pi", Vec::new()).await;
    let observer = manager.create_observer(
        SessionId::new(),
        "Observer Test".to_string(),
        None,
        AppDetectionMode::ShellIntegration,
    );

    // An app without an addon was running when the parent shell died.
    assert!(
        observer
            .lock()
            .await
            .handle_shell_integration_sequence("osc", "133", b"C;cmdline=opencode")
            .await
    );
    let _ = event_rx.try_recv();

    // Without the reset, the stale app blocks the next detection.
    assert!(
        !observer
            .lock()
            .await
            .handle_shell_integration_sequence("osc", "133", b"C;cmdline=pi")
            .await,
        "stale app must block detection (the respawn must reset first)"
    );

    // The respawn resets the observer (respawn_session_shell).
    observer.lock().await.reset_app_tracking();

    assert!(
        observer
            .lock()
            .await
            .handle_shell_integration_sequence("osc", "133", b"C;cmdline=pi")
            .await,
        "pi must be detected after the respawn reset"
    );
    let saw_pi = std::iter::from_fn(|| event_rx.try_recv().ok()).any(|event| {
        matches!(
            event,
            TerminalObserverEvent::SessionStateChanged { ref app, .. } if app.0 == "pi"
        )
    });
    assert!(saw_pi, "expected a SessionStateChanged event for pi");
}

#[tokio::test]
async fn shell_integration_detection_publishes_app_with_idle_status() {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let manager = TerminalObserverManager::new(
        test_config_manager().await,
        Arc::new(AddonHost::new()),
        event_tx,
    );
    manager
        .insert_test_app_addon("claude-code", "claude", Vec::new())
        .await;

    let observer = manager.create_observer(
        SessionId::new(),
        "Observer Test".to_string(),
        None,
        AppDetectionMode::ShellIntegration,
    );
    let detected = observer
        .lock()
        .await
        .handle_shell_integration_sequence("osc", "133", b"C;cmdline=claude")
        .await;

    assert!(detected);
    let event = event_rx.try_recv().expect("expected session state event");
    assert!(matches!(
        event,
        TerminalObserverEvent::SessionStateChanged {
            app,
            app_status: SessionAppStatus::Idle,
            ..
        } if app.0 == "claude"
    ));
}

#[tokio::test]
async fn shell_integration_tracks_app_without_addon_by_program_name() {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let manager = TerminalObserverManager::new(
        test_config_manager().await,
        Arc::new(AddonHost::new()),
        event_tx,
    );
    let observer = manager.create_observer(
        SessionId::new(),
        "Observer Test".to_string(),
        None,
        AppDetectionMode::ShellIntegration,
    );

    let started = observer
        .lock()
        .await
        .handle_shell_integration_sequence("osc", "133", b"C;cmdline=cargo run --example demo")
        .await;

    assert!(started);
    let start_event = event_rx.try_recv().expect("expected app start event");
    assert!(matches!(
        start_event,
        TerminalObserverEvent::SessionStateChanged {
            app,
            app_status: SessionAppStatus::None,
            ..
        } if app.0 == "cargo"
    ));

    let stopped = observer
        .lock()
        .await
        .handle_shell_integration_sequence("osc", "133", b"D;0;aid=42")
        .await;

    assert!(stopped);
    let stop_event = event_rx.try_recv().expect("expected app exit event");
    assert!(matches!(
        stop_event,
        TerminalObserverEvent::SessionStateChanged {
            app,
            app_status: SessionAppStatus::None,
            ..
        } if app.is_none()
    ));
}

#[tokio::test]
async fn shell_integration_ignores_prompt_hooks_and_shell_grammar() {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let manager = TerminalObserverManager::new(
        test_config_manager().await,
        Arc::new(AddonHost::new()),
        event_tx,
    );
    let observer = manager.create_observer(
        SessionId::new(),
        "Observer Test".to_string(),
        None,
        AppDetectionMode::ShellIntegration,
    );

    for marker in [
        b"C;cmdline=case %22%24cmdline%22 in".as_slice(),
        b"C;cmdline=__vte_prompt_command".as_slice(),
        b"C;cmdline=__vterm_prompt_command".as_slice(),
    ] {
        let detected = observer
            .lock()
            .await
            .handle_shell_integration_sequence("osc", "133", marker)
            .await;
        assert!(!detected);
    }

    assert!(event_rx.try_recv().is_err());
    assert!(observer.lock().await.app.is_none());
}

#[tokio::test]
async fn publishes_summary_change_when_app_and_status_are_unchanged() {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let manager = TerminalObserverManager::new(
        test_config_manager().await,
        Arc::new(AddonHost::new()),
        event_tx,
    );
    let observer = manager.create_observer(
        SessionId::new(),
        "Observer Test".to_string(),
        None,
        AppDetectionMode::ShellIntegration,
    );
    let mut obs = observer.lock().await;
    obs.app = SessionApp("test-app".to_string());
    obs.active_prompt = Some(PromptState {
        prompt: PromptMatch {
            app_status: SessionAppStatus::ConfirmingNormal,
            summary: "First summary".to_string(),
            matched_line: String::new(),
        },
        signature: "first".to_string(),
        notified: false,
        auto_confirmed: false,
        auto_confirmed_revision: None,
    });
    update_app_status(&mut obs, SessionAppStatus::ConfirmingNormal);

    let first = event_rx.try_recv().expect("expected initial state event");
    assert!(matches!(
        first,
        TerminalObserverEvent::SessionStateChanged {
            app_status_summary: Some(summary),
            ..
        } if summary == "First summary"
    ));

    obs.active_prompt.as_mut().unwrap().prompt.summary = "Updated summary".to_string();
    update_app_status(&mut obs, SessionAppStatus::ConfirmingNormal);

    let updated = event_rx.try_recv().expect("expected summary update event");
    assert!(matches!(
        updated,
        TerminalObserverEvent::SessionStateChanged {
            app_status_summary: Some(summary),
            ..
        } if summary == "Updated summary"
    ));
}

/// Directly set app status for testing and optionally run the idle analysis path.
async fn set_test_app_status(
    observer: &Arc<Mutex<SessionObserver>>,
    status: SessionAppStatus,
    emit_attention: bool,
    terminal: &Terminal,
) {
    let mut obs = observer.lock().await;
    let should_schedule =
        status == SessionAppStatus::Idle && obs.app_status == SessionAppStatus::Busy;
    obs.app_status = status.clone();
    if status == SessionAppStatus::Idle {
        obs.analysis_generation = obs.analysis_generation.saturating_add(1);
    }
    let generation = obs.analysis_generation;
    update_app_status(&mut obs, status);
    if should_schedule {
        let snapshot = trim_snapshot_lines(&terminal.unwrapped_screen_text());
        let addon_id = obs.active_addon.as_ref().map(|a| a.addon_id.clone());
        let app = obs.app.clone();
        let event_tx = obs.event_tx.clone();
        let addon_host = obs.addon_host.clone();
        let auto_confirm = obs.auto_confirm_enabled;
        drop(obs);

        perform_test_analysis(
            observer,
            generation,
            &snapshot,
            addon_id.as_deref(),
            &app,
            auto_confirm,
            &addon_host,
            &event_tx,
            emit_attention,
        )
        .await;
    }
}

/// Run the test-only analysis flow without spawning the full production task graph.
async fn perform_test_analysis(
    observer: &Arc<Mutex<SessionObserver>>,
    generation: u64,
    snapshot: &str,
    addon_id: Option<&str>,
    _app: &SessionApp,
    auto_confirm: bool,
    addon_host: &Arc<AddonHost>,
    event_tx: &mpsc::UnboundedSender<TerminalObserverEvent>,
    emit_attention: bool,
) {
    let app_prompt = if let Some(addon_id) = addon_id {
        let result = run_detect_output(addon_host, addon_id, snapshot);
        if result.status != SessionAppStatus::None
            && result.status != SessionAppStatus::Idle
            && result.status != SessionAppStatus::Busy
        {
            Some(PromptMatch {
                app_status: result.status,
                summary: result.message.clone().unwrap_or_default(),
                matched_line: String::new(),
            })
        } else {
            None
        }
    } else {
        None
    };

    let mut obs = observer.lock().await;
    if obs.analysis_generation != generation {
        return;
    }
    obs.analyzed_revision = obs.output_revision;

    match app_prompt {
        Some(prompt) => {
            let signature = prompt_signature(&prompt);
            let session_id = obs.session_id;
            let should_auto_confirm_now = should_auto_confirm(auto_confirm, &prompt);
            let next_app_status = prompt.app_status;
            let mut send_auto_confirm = false;
            let output_revision = obs.output_revision;
            match obs.active_prompt.as_mut() {
                Some(active) if active.signature == signature => {
                    let already_auto_confirmed_this_revision =
                        active.auto_confirmed_revision == Some(output_revision);
                    if should_auto_confirm(auto_confirm, &active.prompt)
                        && !already_auto_confirmed_this_revision
                    {
                        active.auto_confirmed = true;
                        active.auto_confirmed_revision = Some(output_revision);
                        let _ = event_tx
                            .send(TerminalObserverEvent::AutoConfirmRequested { session_id });
                        return;
                    }
                }
                Some(_) => {
                    if obs.active_prompt.take().is_some() {
                        update_app_status(&mut obs, SessionAppStatus::Idle);
                    }
                    obs.active_prompt = Some(PromptState {
                        prompt,
                        signature,
                        notified: false,
                        auto_confirmed: should_auto_confirm_now,
                        auto_confirmed_revision: should_auto_confirm_now.then_some(output_revision),
                    });
                    obs.prompt_generation = obs.prompt_generation.saturating_add(1);
                    log_app_waiting_approval(&obs, obs.active_prompt.as_ref());
                    update_app_status(&mut obs, next_app_status);
                    send_auto_confirm = should_auto_confirm_now;
                }
                None => {
                    obs.active_prompt = Some(PromptState {
                        prompt,
                        signature,
                        notified: false,
                        auto_confirmed: should_auto_confirm_now,
                        auto_confirmed_revision: should_auto_confirm_now.then_some(output_revision),
                    });
                    obs.prompt_generation = obs.prompt_generation.saturating_add(1);
                    log_app_waiting_approval(&obs, obs.active_prompt.as_ref());
                    update_app_status(&mut obs, next_app_status);
                    send_auto_confirm = should_auto_confirm_now;
                }
            }

            update_app_status(&mut obs, next_app_status);
            if send_auto_confirm {
                let _ = event_tx.send(TerminalObserverEvent::AutoConfirmRequested { session_id });
            } else if emit_attention {
                let title = obs.display_title();
                if let Some(active) = obs.active_prompt.as_mut() {
                    if !active.notified {
                        let attention = TerminalAttention {
                            session_id,
                            title,
                            kind: match active.prompt.app_status {
                                SessionAppStatus::Selection => TerminalAttentionKind::Selection,
                                _ => TerminalAttentionKind::Confirmation,
                            },
                            summary: active.prompt.summary.clone(),
                            matched_line: active.prompt.matched_line.clone(),
                        };
                        active.notified = true;
                        let _ = event_tx.send(TerminalObserverEvent::AttentionRequired(attention));
                    }
                }
            }
        }
        None => {
            if obs.active_prompt.take().is_some() {
                update_app_status(&mut obs, SessionAppStatus::Idle);
            }
            update_app_status(&mut obs, SessionAppStatus::Idle);
        }
    }
}

/// Verify a stable confirmation prompt produces an attention event after the grace period.
#[tokio::test]
async fn emits_attention_after_stable_prompt_delay() {
    let config_manager = test_config_manager().await;
    config_manager
        .apply_patch(ConfigPatchRequest {
            terminal: Some(TerminalConfigPatch {
                enabled: Some(true),
                confirmation_grace_seconds: Some(1),
                ..TerminalConfigPatch::default()
            }),
            ..ConfigPatchRequest::default()
        })
        .await
        .unwrap();

    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let addon_host = Arc::new(AddonHost::new());
    let manager = Arc::new(TerminalObserverManager::new(
        config_manager,
        addon_host,
        event_tx,
    ));

    let session_id = SessionId::new();
    let observer = manager.create_observer(
        session_id,
        "Observer Test".to_string(),
        None,
        AppDetectionMode::LegacyPrompt,
    );
    observer.lock().await.set_auto_confirm(false);

    manager
        .insert_test_app_addon(
            "claude-code",
            "claude",
            vec![SupportedCommand {
                seq_type: "osc".to_string(),
                code: "0".to_string(),
            }],
        )
        .await;

    observer
        .lock()
        .await
        .record_input(true, Some("/home/user$ claude".to_string()))
        .await;

    set_test_app_status(
        &observer,
        SessionAppStatus::Busy,
        false,
        &Terminal::new_with_type(TerminalSize::new(24, 80), Some("xterm")),
    )
    .await;

    let mut terminal = Terminal::new_with_type(TerminalSize::new(24, 80), Some("xterm"));
    terminal.process(b"\x1b]0;\xe2\x9c\xb3 Claude Code\x07Do you want to proceed? [y/n]");
    let (scheduler, _rx) = test_scheduler();
    observer
        .lock()
        .await
        .record_output(
            b"\x1b]0;\xe2\x9c\xb3 Claude Code\x07Do you want to proceed? [y/n]",
            ObservedTerminalState::from_terminal(&terminal),
            &scheduler,
        )
        .await;
    set_test_app_status(&observer, SessionAppStatus::Idle, true, &terminal).await;

    tokio::time::sleep(Duration::from_secs(4)).await;

    let mut received = false;
    while let Ok(event) = event_rx.try_recv() {
        if matches!(event, TerminalObserverEvent::AttentionRequired(_)) {
            received = true;
            break;
        }
    }

    assert!(
        received,
        "expected attention event after stable prompt delay"
    );
}

/// Verify new user input clears a pending attention before the grace period completes.
#[tokio::test]
async fn input_cancels_pending_attention() {
    let config_manager = test_config_manager().await;
    config_manager
        .apply_patch(ConfigPatchRequest {
            terminal: Some(TerminalConfigPatch {
                enabled: Some(true),
                confirmation_grace_seconds: Some(2),
                ..TerminalConfigPatch::default()
            }),
            ..ConfigPatchRequest::default()
        })
        .await
        .unwrap();

    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let addon_host = Arc::new(AddonHost::new());
    let manager = Arc::new(TerminalObserverManager::new(
        config_manager,
        addon_host,
        event_tx,
    ));

    let session_id = SessionId::new();
    let observer = manager.create_observer(
        session_id,
        "Observer Test".to_string(),
        None,
        AppDetectionMode::LegacyPrompt,
    );
    observer.lock().await.set_auto_confirm(false);

    manager
        .insert_test_app_addon(
            "claude-code",
            "claude",
            vec![SupportedCommand {
                seq_type: "osc".to_string(),
                code: "0".to_string(),
            }],
        )
        .await;
    observer
        .lock()
        .await
        .record_input(true, Some("/home/user$ claude".to_string()))
        .await;

    let mut terminal = Terminal::new_with_type(TerminalSize::new(24, 80), Some("xterm"));
    set_test_app_status(&observer, SessionAppStatus::Busy, false, &terminal).await;
    terminal.process(b"\x1b]0;\xe2\x9c\xb3 Claude Code\x07Do you want to proceed? [y/n]");
    let (scheduler, _rx) = test_scheduler();
    observer
        .lock()
        .await
        .record_output(
            b"\x1b]0;\xe2\x9c\xb3 Claude Code\x07Do you want to proceed? [y/n]",
            ObservedTerminalState::from_terminal(&terminal),
            &scheduler,
        )
        .await;
    set_test_app_status(&observer, SessionAppStatus::Idle, false, &terminal).await;

    tokio::time::sleep(Duration::from_secs(1)).await;
    observer.lock().await.record_input(false, None).await;
    tokio::time::sleep(Duration::from_secs(3)).await;

    let mut saw_required = false;
    while let Ok(event) = event_rx.try_recv() {
        if matches!(event, TerminalObserverEvent::AttentionRequired(_)) {
            saw_required = true;
        }
    }

    assert!(
        !saw_required,
        "did not expect attention event after user input"
    );
}

/// Verify normal confirmations trigger auto-confirm when the feature is enabled.
#[tokio::test]
async fn auto_confirm_requests_enter_for_normal_confirmation() {
    let config_manager = test_config_manager().await;

    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let addon_host = Arc::new(AddonHost::new());
    let manager = Arc::new(TerminalObserverManager::new(
        config_manager,
        addon_host,
        event_tx,
    ));

    let session_id = SessionId::new();
    let observer = manager.create_observer(
        session_id,
        "Observer Test".to_string(),
        None,
        AppDetectionMode::LegacyPrompt,
    );
    observer.lock().await.set_auto_confirm(true);

    manager
        .insert_test_app_addon(
            "claude-code",
            "claude",
            vec![SupportedCommand {
                seq_type: "osc".to_string(),
                code: "0".to_string(),
            }],
        )
        .await;

    observer
        .lock()
        .await
        .record_input(true, Some("/home/user$ claude".to_string()))
        .await;

    let mut terminal = Terminal::new_with_type(TerminalSize::new(24, 80), Some("xterm"));
    set_test_app_status(&observer, SessionAppStatus::Busy, false, &terminal).await;

    terminal.process(b"\x1b]0;\xe2\x9c\xb3 Claude Code\x07Do you want to proceed? [y/n]");
    let (scheduler, _rx) = test_scheduler();
    observer
        .lock()
        .await
        .record_output(
            b"\x1b]0;\xe2\x9c\xb3 Claude Code\x07Do you want to proceed? [y/n]",
            ObservedTerminalState::from_terminal(&terminal),
            &scheduler,
        )
        .await;
    set_test_app_status(&observer, SessionAppStatus::Idle, false, &terminal).await;

    tokio::time::sleep(Duration::from_secs(2)).await;

    let mut saw_auto_confirm = false;
    let mut all_events = Vec::new();
    while let Ok(event) = event_rx.try_recv() {
        all_events.push(format!("{:?}", event));
        if matches!(event, TerminalObserverEvent::AutoConfirmRequested { .. }) {
            saw_auto_confirm = true;
            break;
        }
    }

    assert!(
        saw_auto_confirm,
        "expected auto-confirm event for normal confirmation, got events: {:?}",
        all_events
    );
}
