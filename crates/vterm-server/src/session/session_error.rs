use vterm_common::SessionId;

#[derive(thiserror::Error, Debug)]
pub enum SessionError {
    #[error("Session not found: {0}")]
    SessionNotFound(SessionId),
    #[error("Resize not allowed for session: {0}")]
    ResizeNotAllowed(SessionId),
    #[error("Failed to manage PTY: {0}")]
    PtyError(String),
    #[error("Failed to access file system: {0}")]
    FileSystemError(String),
    #[error("Invalid session state: {0}")]
    InvalidState(String),
    #[error("Operation not supported: {0}")]
    Unsupported(String),
}
