pub mod types;

pub use types::ChatCompletionRequest;
pub use types::ChatCompletionResponse;

use std::time::Duration;

use async_trait::async_trait;
use reqwest::StatusCode;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::error::AgentError;

pub use types::{ChatMessage, Choice, ToolCall, ToolCallFunction, Usage};

const DEFAULT_TIMEOUT_SECS: u64 = 120;
const MAX_RETRIES: u32 = 3;
const RETRY_BASE_DELAY_MS: u64 = 1000;
const RETRY_MAX_DELAY_MS: u64 = 60000;

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AgentError>;
}

pub struct OpenAiClient {
    /// Client timeout; a fresh reqwest client is built per request so the
    /// proxy config reflects the CURRENT system proxy state (see
    /// `vterm_common::proxy`) instead of the process-start env snapshot.
    timeout: Duration,
    base_url: String,
    api_key: String,
    default_model: String,
    max_retries: u32,
}

/// Build a client whose proxy config matches the current system proxy state:
/// desktop proxy on → route through it (loopback bypassed); desktop proxy
/// explicitly off → direct, overriding any stale proxy env this process was
/// started with; no desktop settings (headless) → reqwest's default env
/// behavior.
fn client_with_current_proxy(timeout: Duration) -> reqwest::Client {
    // Agent endpoints may be self-hosted with self-signed certificates.
    let builder = reqwest::Client::builder()
        .timeout(timeout)
        .danger_accept_invalid_certs(true);
    let builder = match vterm_common::proxy::resolve_system_proxy() {
        vterm_common::proxy::ResolvedProxy::Direct => builder.no_proxy(),
        vterm_common::proxy::ResolvedProxy::Environment => builder,
        vterm_common::proxy::ResolvedProxy::Url(url) => match reqwest::Proxy::all(&url) {
            Ok(proxy) => builder.proxy(
                proxy.no_proxy(reqwest::NoProxy::from_string("localhost,127.0.0.1,::1")),
            ),
            Err(_) => builder,
        },
    };
    builder
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

impl OpenAiClient {
    pub fn new(base_url: String, api_key: String, default_model: String) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();

        Self {
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            base_url,
            api_key,
            default_model,
            max_retries: MAX_RETRIES,
        }
    }

    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn chat_completion(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AgentError> {
        if request.model.is_empty() {
            request.model = self.default_model.clone();
        }

        let url = format!("{}/chat/completions", self.base_url);
        let mut builder = client_with_current_proxy(self.timeout)
            .post(&url)
            .header("Content-Type", "application/json");

        if !self.api_key.is_empty() {
            builder = builder.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let body = serde_json::to_string(&request)?;

        debug!(
            url = %url,
            model = %request.model,
            message_count = request.messages.len(),
            has_tools = request.tools.is_some(),
            "Sending chat completion request"
        );

        let request_start = std::time::Instant::now();
        let mut last_error = None;
        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                let delay_ms =
                    (RETRY_BASE_DELAY_MS * 2u64.pow(attempt - 1)).min(RETRY_MAX_DELAY_MS);
                let jitter = (delay_ms as f64 * 0.1 * (rand_jitter(attempt))) as u64;
                info!(
                    attempt,
                    delay_ms = delay_ms + jitter,
                    "Retrying LLM request"
                );
                sleep(Duration::from_millis(delay_ms + jitter)).await;
            }

            let result = builder
                .try_clone()
                .expect("request builder clone")
                .body(body.clone())
                .send()
                .await;

            match result {
                Ok(response) => {
                    let status = response.status();
                    let elapsed = request_start.elapsed();
                    if status.is_success() {
                        let response_text = response.text().await?;
                        debug!(
                            status = %status,
                            elapsed_ms = elapsed.as_millis() as u64,
                            response_len = response_text.len(),
                            "LLM response received"
                        );
                        let parsed: ChatCompletionResponse = serde_json::from_str(&response_text)
                            .map_err(|e| {
                            AgentError::LlmClient(format!(
                                "Failed to parse response: {e}\nResponse: {}",
                                &response_text[..response_text.len().min(500)]
                            ))
                        })?;
                        return Ok(parsed);
                    }

                    let error_body = response.text().await.unwrap_or_default();
                    let should_retry =
                        status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error();

                    if !should_retry || attempt == self.max_retries {
                        error!(status = %status, elapsed_ms = elapsed.as_millis() as u64, "LLM request failed permanently");
                        return Err(AgentError::LlmClient(format!(
                            "HTTP {status}: {error_body}"
                        )));
                    }

                    warn!(
                        status = %status,
                        attempt = attempt + 1,
                        max_retries = self.max_retries,
                        "LLM request failed, will retry"
                    );
                    last_error = Some(AgentError::LlmClient(format!(
                        "HTTP {status}: {error_body}"
                    )));
                }
                Err(e) => {
                    if attempt == self.max_retries {
                        error!(error = %e, "LLM request network error, all retries exhausted");
                        return Err(AgentError::Http(e));
                    }
                    warn!(
                        error = %e,
                        attempt = attempt + 1,
                        max_retries = self.max_retries,
                        "LLM request network error, will retry"
                    );
                    last_error = Some(AgentError::Http(e));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| AgentError::LlmClient("Unknown error".into())))
    }
}

fn rand_jitter(seed: u32) -> f64 {
    let x = seed.wrapping_mul(1103515245).wrapping_add(12345);
    ((x >> 16) & 0x7FFF) as f64 / 32767.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_client() -> OpenAiClient {
        OpenAiClient::new(
            "http://localhost:2000/v1".to_string(),
            "qwen3".to_string(),
            "qwen3".to_string(),
        )
    }

    #[tokio::test]
    #[ignore = "requires llama-server running at localhost:2000"]
    async fn test_basic_completion() {
        crate::init_test_tracing();
        let client = test_client();
        let request = ChatCompletionRequest {
            model: "qwen3".into(),
            messages: vec![ChatMessage::user("Say exactly: pong")],
            tools: None,
            temperature: Some(0.0),
            max_tokens: Some(256),
        };

        let response = client.chat_completion(request).await.unwrap();
        assert!(!response.choices.is_empty());
        let content = response.choices[0]
            .message
            .content
            .clone()
            .unwrap_or_default();
        println!("Raw response: {content:?}");

        // qwen3 may wrap thinking in <think/> tags; strip them for assertion
        let cleaned = content.replace("<think/>", "").replace("</think", "");
        let visible: String = cleaned.split(">").last().unwrap_or("").trim().to_string();
        assert!(!visible.is_empty(), "Response had no visible content");
    }

    #[tokio::test]
    #[ignore = "requires llama-server running at localhost:2000"]
    async fn test_tool_call_response() {
        crate::init_test_tracing();
        let client = test_client();
        let request = ChatCompletionRequest {
            model: "qwen3".into(),
            messages: vec![
                ChatMessage::system("You are a helpful assistant with access to tools."),
                ChatMessage::user(
                    "Please use the file_read tool to read the file /etc/hostname. \
                     If you cannot read it, just say 'cannot read'.",
                ),
            ],
            tools: Some(vec![serde_json::json!({
                "type": "function",
                "function": {
                    "name": "file_read",
                    "description": "Read file content",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string" }
                        },
                        "required": ["path"]
                    }
                }
            })]),
            temperature: Some(0.0),
            max_tokens: Some(256),
        };

        let response = client.chat_completion(request).await.unwrap();
        assert!(!response.choices.is_empty());
        println!("Response: {:?}", response.choices[0].message);
    }

    #[test]
    fn test_chat_message_constructors() {
        let sys = ChatMessage::system("hello");
        assert_eq!(sys.role, "system");
        assert_eq!(sys.content.as_deref(), Some("hello"));

        let user = ChatMessage::user("world");
        assert_eq!(user.role, "user");

        let asst = ChatMessage::assistant("reply");
        assert_eq!(asst.role, "assistant");

        let tool = ChatMessage::tool_result("call_123", "result text");
        assert_eq!(tool.role, "tool");
        assert_eq!(tool.tool_call_id.as_deref(), Some("call_123"));
    }
}
