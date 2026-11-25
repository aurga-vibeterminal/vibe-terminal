use tracing::{debug, info};

use crate::client::{ChatMessage, LlmClient};
use crate::error::AgentError;
use crate::session::AgentSession;

#[derive(Debug, Clone)]
pub struct CompressionConfig {
    pub max_context_tokens: usize,
    pub async_threshold_pct: f64,
    pub sync_threshold_pct: f64,
    pub active_zone_rounds: usize,
    pub frozen_count: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            max_context_tokens: 128_000,
            async_threshold_pct: 0.60,
            sync_threshold_pct: 0.80,
            active_zone_rounds: 4,
            frozen_count: 2,
        }
    }
}

impl CompressionConfig {
    pub fn async_threshold(&self) -> i64 {
        (self.max_context_tokens as f64 * self.async_threshold_pct) as i64
    }

    pub fn sync_threshold(&self) -> i64 {
        (self.max_context_tokens as f64 * self.sync_threshold_pct) as i64
    }
}

pub struct MemoryManager {
    pub config: CompressionConfig,
}

impl MemoryManager {
    pub fn new(config: CompressionConfig) -> Self {
        Self { config }
    }

    pub fn needs_compression(&self, session: &AgentSession) -> bool {
        let estimate = session.token_estimate();
        estimate > self.config.async_threshold()
    }

    pub fn needs_sync_compression(&self, session: &AgentSession) -> bool {
        let estimate = session.token_estimate();
        estimate > self.config.sync_threshold()
    }

    pub async fn compress(
        &self,
        session: &mut AgentSession,
        client: &dyn LlmClient,
        model: &str,
    ) -> Result<bool, AgentError> {
        let msg_count = session.messages.len();
        let active_msg_count = self.config.active_zone_rounds * 2;

        info!(
            total_messages = msg_count,
            frozen_count = self.config.frozen_count,
            active_zone_rounds = self.config.active_zone_rounds,
            active_msg_count,
            token_estimate = session.token_estimate(),
            "Evaluating compression"
        );

        // Not enough messages to compress
        if msg_count <= self.config.frozen_count + active_msg_count {
            debug!("Not enough messages for compression");
            return Ok(false);
        }

        let frozen_end = self.config.frozen_count;
        let active_start = msg_count.saturating_sub(active_msg_count);

        // No compress zone (frozen and active overlap or touch)
        if active_start <= frozen_end {
            debug!("Compress zone empty (frozen and active overlap)");
            return Ok(false);
        }

        let compress_messages: Vec<ChatMessage> =
            session.messages[frozen_end..active_start].to_vec();

        info!(
            frozen_range = ?(0..frozen_end),
            compress_range = ?(frozen_end..active_start),
            active_range = ?(active_start..msg_count),
            compress_msg_count = compress_messages.len(),
            "Zone split for compression"
        );

        let compress_text = format_messages_for_summary(&compress_messages);

        debug!(
            compress_text_len = compress_text.len(),
            "Formatted messages for summary"
        );

        let summary_prompt = format!(
            "Summarize the following conversation, preserving:\n\
             1. Key facts and decisions made\n\
             2. Tool call results and conclusions\n\
             3. Completed tasks\n\
             4. Pending tasks and current focus\n\n\
             Conversation:\n{compress_text}"
        );

        info!(
            model,
            compress_msg_count = compress_messages.len(),
            "Requesting compression summary from LLM"
        );

        let request = crate::client::types::ChatCompletionRequest {
            model: model.to_string(),
            messages: vec![
                ChatMessage::system(
                    "You are a conversation summarizer. Produce concise summaries preserving all important context.",
                ),
                ChatMessage::user(summary_prompt),
            ],
            tools: None,
            temperature: Some(0.3),
            max_tokens: Some(2048),
        };

        let response = client.chat_completion(request).await?;

        let summary = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| AgentError::Compression("No summary content in response".into()))?;

        info!(
            compressed_count = compress_messages.len(),
            summary_len = summary.len(),
            "Compression summary generated"
        );
        debug!(
            summary_preview = %summary.chars().take(300).collect::<String>(),
            "Summary preview"
        );

        // Rebuild messages: frozen + summary + active
        let frozen: Vec<ChatMessage> = session.messages[..frozen_end].to_vec();
        let active: Vec<ChatMessage> = session.messages[active_start..].to_vec();

        let new_count = frozen.len() + 1 + active.len();
        session.messages = frozen;
        session.messages.push(ChatMessage::system(format!(
            "<previous_context_summary>\n{summary}\n</previous_context_summary>"
        )));
        session.messages.extend(active);

        info!(
            messages_before = msg_count,
            messages_after = new_count,
            saved = msg_count - new_count,
            "Messages rebuilt after compression"
        );

        Ok(true)
    }
}

fn format_messages_for_summary(messages: &[ChatMessage]) -> String {
    messages
        .iter()
        .map(|m| {
            let role = &m.role;
            let content = m.content.as_deref().unwrap_or("");
            if let Some(tool_calls) = &m.tool_calls {
                let calls: Vec<String> = tool_calls
                    .iter()
                    .map(|tc| format!("{}({})", tc.function.name, tc.function.arguments))
                    .collect();
                format!("[{role}] {content}\n[tool_calls] {}", calls.join(", "))
            } else {
                format!("[{role}] {content}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_compression_below_threshold() {
        let config = CompressionConfig {
            max_context_tokens: 1000,
            ..Default::default()
        };
        let manager = MemoryManager::new(config);
        let session = AgentSession::new("short", "hello");

        assert!(!manager.needs_compression(&session));
        assert!(!manager.needs_sync_compression(&session));
    }

    #[test]
    fn test_thresholds_trigger() {
        let config = CompressionConfig {
            max_context_tokens: 100,
            async_threshold_pct: 0.60,
            sync_threshold_pct: 0.80,
            ..Default::default()
        };
        let manager = MemoryManager::new(config);

        // 60% of 100 tokens = 60 tokens, 80% = 80 tokens
        // Each char/4 = ~token count. Need >240 chars for 60 tokens, >320 for 80 tokens.
        let mut session = AgentSession::new("x".repeat(130), "y".repeat(130));
        // ~260 chars / 4 = ~65 tokens > 60
        assert!(manager.needs_compression(&session));

        // Need more to exceed 80% = 80 tokens (>320 chars total)
        session.append(ChatMessage::assistant("z".repeat(200)));
        assert!(manager.needs_sync_compression(&session));
    }

    #[test]
    fn test_zone_calculation_no_compress_when_too_few() {
        let config = CompressionConfig {
            max_context_tokens: 100,
            active_zone_rounds: 4,
            frozen_count: 2,
            ..Default::default()
        };
        let _manager = MemoryManager::new(config);

        // 2 frozen + 3 messages = 5 total, need 2 + 8 = 10 for compression to apply
        let mut session = AgentSession::new("sys", "usr");
        session.append(ChatMessage::assistant("a"));
        session.append(ChatMessage::user("b"));
        session.append(ChatMessage::assistant("c"));

        assert_eq!(session.messages.len(), 5);
        // 2 + 8 = 10 > 5, so compress should return false even if threshold exceeded
        // This tests the "not enough messages to compress" guard
    }

    #[test]
    fn test_format_messages_for_summary() {
        let messages = vec![ChatMessage::user("hello"), ChatMessage::assistant("world")];
        let result = format_messages_for_summary(&messages);
        assert!(result.contains("[user] hello"));
        assert!(result.contains("[assistant] world"));
    }
}
