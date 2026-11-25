mod ai_conversation;
mod app_event;
mod conversation_history;
mod environment_script;
mod file_ops;
mod find_files;
pub mod git;
mod path_listing;
mod prompt_detection;
mod quick_command_script;
mod raw_history;
mod recent_commands;
mod recent_projects;
mod screen_log;
mod search_path_names;
mod session_core;
mod session_error;
mod session_io;
mod session_manager;
mod shell_integration;
mod terminal_observer;
mod terminal_profiles;
mod util;

#[cfg(target_os = "windows")]
mod session_win;

pub use app_event::{
    run_detect_command, run_detect_output, run_get_program_name, run_get_supported_commands,
    SupportedCommand,
};
pub use prompt_detection::{
    prompt_signature, should_auto_confirm, strip_ansi_sequences, trim_to_recent_limit, PromptMatch,
    RECENT_OUTPUT_LIMIT,
};
pub use recent_projects::RecentProjectScanState;
pub use session_core::ConnectionTx;
pub use session_error::SessionError;
pub(crate) fn session_log_dir(session_id: vterm_common::SessionId) -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".vibeterminal")
        .join("sessions")
        .join(session_id.to_string())
}

pub use session_manager::{FolderUploadSink, SessionManager};
pub use terminal_observer::{
    ObservedTerminalState, TerminalObserverEvent, TerminalObserverManager,
};
