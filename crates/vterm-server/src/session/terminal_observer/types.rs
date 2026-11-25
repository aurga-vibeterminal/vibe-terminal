use vterm_common::{
    SessionApp, SessionAppStatus, SessionId, TerminalAttention, TerminalCompletion,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalObserverEvent {
    SessionStateChanged {
        session_id: SessionId,
        app: SessionApp,
        app_status: SessionAppStatus,
        /// Summary to snapshot for newly-connecting clients. Set when entering
        /// a confirm/select state, cleared otherwise.
        app_status_summary: Option<String>,
    },
    AttentionRequired(TerminalAttention),
    TaskCompleted(TerminalCompletion),
    AutoConfirmRequested {
        session_id: SessionId,
    },
}
