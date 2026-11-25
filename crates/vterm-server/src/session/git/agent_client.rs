//! Bridge between `vterm-agent`'s async `LlmClient` and the blocking addon
//! executor used by the git agents.
//!
//! The commit and conflict agents carry their own copies of this adapter;
//! this shared one exists for the rebase agent. Folding the other two copies
//! into it is a deliberate follow-up, not part of the current change.

use std::sync::Arc;

use async_trait::async_trait;
use vterm_agent::client::types::{ChatCompletionRequest, ChatCompletionResponse};
use vterm_agent::client::LlmClient;
use vterm_agent::error::AgentError;

/// Synchronous addon call: (messages_json, options_json) -> raw response body.
pub(crate) type LlmCallback = Box<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync>;

/// `LlmClient` adapter that forwards chat completions (with optional tool
/// definitions) to the addon executor and parses the raw response body.
/// Think blocks are stripped so reasoning text never re-enters the context.
pub(crate) struct CallbackLlmClient {
    callback: Arc<LlmCallback>,
}

impl CallbackLlmClient {
    pub(crate) fn new(callback: LlmCallback) -> Self {
        Self {
            callback: Arc::new(callback),
        }
    }
}

#[async_trait]
impl LlmClient for CallbackLlmClient {
    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AgentError> {
        let messages_json = serde_json::to_string(&request.messages)
            .map_err(|error| AgentError::LlmClient(error.to_string()))?;

        let mut options = serde_json::Map::new();
        if !request.model.is_empty() {
            options.insert("model".into(), serde_json::json!(request.model));
        }
        if let Some(temperature) = request.temperature {
            options.insert("temperature".into(), serde_json::json!(temperature));
        }
        if let Some(max_tokens) = request.max_tokens {
            options.insert("max_tokens".into(), serde_json::json!(max_tokens));
        }
        if let Some(tools) = &request.tools {
            options.insert("tools".into(), serde_json::json!(tools));
        }
        let options_json = serde_json::to_string(&options)
            .map_err(|error| AgentError::LlmClient(error.to_string()))?;

        let callback = self.callback.clone();
        let raw_body = tokio::task::spawn_blocking(move || callback(&messages_json, &options_json))
            .await
            .map_err(|error| AgentError::LlmClient(error.to_string()))?
            .map_err(AgentError::LlmClient)?;

        let mut response: ChatCompletionResponse = serde_json::from_str(&raw_body)
            .map_err(|error| AgentError::LlmClient(format!("Failed to parse response: {error}")))?;

        if let Some(content) = response
            .choices
            .first_mut()
            .and_then(|choice| choice.message.content.as_mut())
        {
            *content = strip_think_tags(content);
        }

        Ok(response)
    }
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
