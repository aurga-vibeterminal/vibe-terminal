#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("LLM client error: {0}")]
    LlmClient(String),
    #[error("LLM returned no content and no tool calls")]
    EmptyResponse,
    #[error("Tool execution error: {tool} - {message}")]
    ToolExecution { tool: String, message: String },
    #[error("Max iterations ({0}) exceeded")]
    MaxIterationsExceeded(usize),
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Compression error: {0}")]
    Compression(String),
    #[error(
        "Context window exceeded: estimated input {input_tokens} + reserved output {output_tokens} > limit {max_context_tokens}"
    )]
    ContextWindowExceeded {
        input_tokens: usize,
        output_tokens: usize,
        max_context_tokens: usize,
    },
    #[error("Cancelled")]
    Cancelled,
}
