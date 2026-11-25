use std::fmt;

/// Exit codes for vterm
pub mod exit_code {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 1;
    pub const CONNECTION_FAILED: i32 = 2;
    pub const AUTHENTICATION_FAILED: i32 = 3;
    pub const SESSION_CREATION_FAILED: i32 = 4;
    pub const PROFILE_NOT_FOUND: i32 = 5;
    pub const INTERRUPTED: i32 = 130;
}

/// VTerm specific errors
#[derive(Debug)]
pub enum VTermError {
    ConnectionFailed { url: String, cause: String },
    AuthenticationFailed { message: String },
    ProfileNotFound { id: String, available: Vec<String> },
    SessionCreationFailed { message: String },
    ServerError { code: String, message: String },
    TerminalError { message: String },
}

impl fmt::Display for VTermError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VTermError::ConnectionFailed { url, cause } => {
                write!(
                    f,
                    "Cannot connect to vterm-server at {}\n  Cause: {}\n  Hint: Is the vterm-server running?",
                    url, cause
                )
            }
            VTermError::AuthenticationFailed { message } => {
                write!(
                    f,
                    "Authentication failed: {}\n  Hint: Check TERMINAL_GUI_SECRET_KEY environment variable.",
                    message
                )
            }
            VTermError::ProfileNotFound { id, available } => {
                writeln!(f, "Profile '{}' not found", id)?;
                if !available.is_empty() {
                    writeln!(f, "\n  Available profiles:")?;
                    for (i, profile) in available.iter().enumerate() {
                        writeln!(f, "    {}. {}", i + 1, profile)?;
                    }
                }
                write!(f, "\n  Use 'vterm --profile' to select interactively.")
            }
            VTermError::SessionCreationFailed { message } => {
                write!(f, "Failed to create terminal session: {}", message)
            }
            VTermError::ServerError { code, message } => {
                write!(f, "Server error ({}): {}", code, message)
            }
            VTermError::TerminalError { message } => {
                write!(f, "Terminal error: {}", message)
            }
        }
    }
}

impl std::error::Error for VTermError {}

impl VTermError {
    /// Get the exit code for this error
    pub fn exit_code(&self) -> i32 {
        match self {
            VTermError::ConnectionFailed { .. } => exit_code::CONNECTION_FAILED,
            VTermError::AuthenticationFailed { .. } => exit_code::AUTHENTICATION_FAILED,
            VTermError::ProfileNotFound { .. } => exit_code::PROFILE_NOT_FOUND,
            VTermError::SessionCreationFailed { .. } => exit_code::SESSION_CREATION_FAILED,
            _ => exit_code::GENERAL_ERROR,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_codes() {
        let err = VTermError::ConnectionFailed {
            url: "ws://test".to_string(),
            cause: "refused".to_string(),
        };
        assert_eq!(err.exit_code(), exit_code::CONNECTION_FAILED);
    }
}
