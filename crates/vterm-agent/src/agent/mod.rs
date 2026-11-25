use std::sync::Arc;

use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::client::{ChatMessage, LlmClient};
use crate::error::AgentError;
use crate::memory::{CompressionConfig, MemoryManager};
use crate::session::SessionManager;
use crate::tool::{ToolRegistry, ToolResult};

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub max_iterations: usize,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub compression: CompressionConfig,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_iterations: 20,
            model: String::new(),
            temperature: Some(0.7),
            max_tokens: Some(4096),
            compression: CompressionConfig::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LoopEvent {
    IterationStart {
        iteration: usize,
    },
    LlmResponse {
        iteration: usize,
        content: Option<String>,
        tool_call_count: usize,
    },
    ToolExecuted {
        iteration: usize,
        tool_name: String,
        result: ToolResult,
    },
    Compressed {
        messages_before: usize,
        messages_after: usize,
    },
    Cancelled {
        iteration: usize,
        reason: CancelReason,
    },
}

#[derive(Debug, Clone)]
pub enum CancelReason {
    IterationCheck,
    LlmCallAborted,
    ToolAborted { tool_name: String },
    UserRequested,
}

#[derive(Debug)]
pub enum LoopResult {
    Completed {
        session_id: String,
        final_content: String,
        iterations: usize,
        total_tokens: i64,
    },
    MaxIterationsExceeded {
        session_id: String,
        last_content: String,
        iterations: usize,
    },
    Cancelled {
        session_id: String,
        iterations: usize,
        reason: CancelReason,
    },
    Error {
        session_id: String,
        error: AgentError,
        iterations: usize,
    },
}

/// Iterations still available when the budget warning is issued.
const BUDGET_WARNING_REMAINING: usize = 2;

/// Outcome of the forced wrap-up turn after the iteration budget runs out.
enum WrapUpOutcome {
    /// The model produced a final answer with tools disabled.
    Answered(String),
    /// The cancel token fired during the wrap-up call.
    Cancelled,
    /// No answer could be produced (context window, LLM error, empty).
    Unavailable,
}

pub struct Agent {
    client: Arc<dyn LlmClient>,
    tools: Arc<ToolRegistry>,
    memory: Arc<MemoryManager>,
    sessions: Arc<Mutex<SessionManager>>,
    config: AgentConfig,
}

impl Agent {
    pub fn new(client: Arc<dyn LlmClient>, tools: Arc<ToolRegistry>, config: AgentConfig) -> Self {
        let memory = Arc::new(MemoryManager::new(config.compression.clone()));
        Self {
            client,
            tools,
            memory,
            sessions: Arc::new(Mutex::new(SessionManager::new())),
            config,
        }
    }

    pub async fn create_session(
        &self,
        system_prompt: impl Into<String>,
        user_message: impl Into<String>,
    ) -> String {
        let mut sessions = self.sessions.lock().await;
        let id = sessions.create(system_prompt, user_message);
        info!(session_id = %id, "Session created");
        id
    }

    pub async fn get_session_messages(&self, session_id: &str) -> Option<Vec<ChatMessage>> {
        let sessions = self.sessions.lock().await;
        sessions.get(session_id).map(|s| s.messages.clone())
    }

    /// Run the agent loop without cancellation support.
    pub async fn run_loop<F>(&self, session_id: &str, event_callback: F) -> LoopResult
    where
        F: FnMut(LoopEvent),
    {
        self.run_loop_with_cancel(session_id, CancellationToken::new(), event_callback)
            .await
    }

    /// Run the agent loop with cancellation support.
    ///
    /// Call `cancel.cancel()` from any thread/task to abort the loop.
    /// The loop checks the token at every iteration boundary, and uses
    /// `tokio::select!` to abort in-flight LLM calls and tool executions.
    pub async fn run_loop_with_cancel<F>(
        &self,
        session_id: &str,
        cancel: CancellationToken,
        mut event_callback: F,
    ) -> LoopResult
    where
        F: FnMut(LoopEvent),
    {
        let model = self.config.model.clone();
        let max_iterations = self.config.max_iterations;

        info!(
            session_id = session_id,
            max_iterations,
            model = %model,
            "Agent loop starting"
        );

        for iteration in 0..max_iterations {
            // -- Cancel check: iteration boundary --
            if cancel.is_cancelled() {
                info!(session_id, iteration, "Cancelled at iteration boundary");
                event_callback(LoopEvent::Cancelled {
                    iteration,
                    reason: CancelReason::IterationCheck,
                });
                return LoopResult::Cancelled {
                    session_id: session_id.to_string(),
                    iterations: iteration,
                    reason: CancelReason::IterationCheck,
                };
            }

            info!(session_id, iteration, "=== Iteration start ===");
            event_callback(LoopEvent::IterationStart { iteration });

            // 1. Check memory compression
            {
                let mut sessions = self.sessions.lock().await;
                let session = match sessions.get_mut(session_id) {
                    Some(s) => s,
                    None => {
                        error!(session_id, iteration, "Session not found");
                        return LoopResult::Error {
                            session_id: session_id.to_string(),
                            error: AgentError::SessionNotFound(session_id.to_string()),
                            iterations: iteration,
                        };
                    }
                };

                session.iteration_count = iteration + 1;

                let token_estimate = session.token_estimate();
                debug!(
                    session_id,
                    iteration,
                    token_estimate,
                    message_count = session.messages.len(),
                    "Memory check"
                );

                if self.memory.needs_sync_compression(session) {
                    info!(session_id, iteration, "Sync compression triggered");
                    let before = session.messages.len();
                    match self
                        .memory
                        .compress(session, self.client.as_ref(), &model)
                        .await
                    {
                        Ok(true) => {
                            let after = session.messages.len();
                            info!(session_id, iteration, before, after, "Memory compressed");
                            event_callback(LoopEvent::Compressed {
                                messages_before: before,
                                messages_after: after,
                            });
                        }
                        Ok(false) => {
                            debug!(
                                session_id,
                                iteration, "Compression skipped (not enough messages)"
                            );
                        }
                        Err(e) => {
                            warn!(session_id, iteration, error = %e, "Memory compression failed");
                        }
                    }
                }
            }

            // -- Budget warning: two iterations before the cap, nudge the
            // model to stop exploring and wrap up (appended exactly once).
            if max_iterations > BUDGET_WARNING_REMAINING
                && iteration == max_iterations - BUDGET_WARNING_REMAINING
            {
                let mut sessions = self.sessions.lock().await;
                if let Some(session) = sessions.get_mut(session_id) {
                    session.append(ChatMessage::user(format!(
                        "Budget notice: you have {BUDGET_WARNING_REMAINING} iterations left \
                         before the tool-use limit. Avoid further exploratory tool calls — \
                         gather only what is strictly necessary, then provide your final answer."
                    )));
                }
            }

            // 2. Build request
            let request = {
                let sessions = self.sessions.lock().await;
                let session = sessions.get(session_id).expect("session checked above");
                let message_tokens = session.token_estimate().max(0) as usize;
                let tool_tokens = if self.tools.is_empty() {
                    0
                } else {
                    serde_json::to_string(&self.tools.get_definitions())
                        .map(|json| json.chars().count().div_ceil(4))
                        .unwrap_or(0)
                };
                let estimated_input_tokens = message_tokens.saturating_add(tool_tokens);
                let reserved_output_tokens = self.config.max_tokens.unwrap_or(0) as usize;
                let max_context_tokens = self.memory.config.max_context_tokens;
                if estimated_input_tokens.saturating_add(reserved_output_tokens)
                    > max_context_tokens
                {
                    error!(
                        session_id,
                        iteration,
                        estimated_input_tokens,
                        reserved_output_tokens,
                        max_context_tokens,
                        "Agent request exceeds configured context window"
                    );
                    return LoopResult::Error {
                        session_id: session_id.to_string(),
                        error: AgentError::ContextWindowExceeded {
                            input_tokens: estimated_input_tokens,
                            output_tokens: reserved_output_tokens,
                            max_context_tokens,
                        },
                        iterations: iteration,
                    };
                }

                debug!(
                    session_id,
                    iteration,
                    message_count = session.messages.len(),
                    estimated_input_tokens,
                    reserved_output_tokens,
                    max_context_tokens,
                    "Building LLM request"
                );

                let mut req = crate::client::types::ChatCompletionRequest {
                    model: model.clone(),
                    messages: session.messages.clone(),
                    tools: None,
                    temperature: self.config.temperature,
                    max_tokens: self.config.max_tokens,
                };

                if !self.tools.is_empty() {
                    req.tools = Some(self.tools.get_definitions());
                }

                req
            };

            // 3. Call LLM (cancellable)
            info!(session_id, iteration, "Calling LLM...");
            let response = tokio::select! {
                result = self.client.chat_completion(request) => {
                    match result {
                        Ok(r) => r,
                        Err(e) => {
                            error!(session_id, iteration, error = %e, "LLM call failed");
                            return LoopResult::Error {
                                session_id: session_id.to_string(),
                                error: e,
                                iterations: iteration + 1,
                            };
                        }
                    }
                }
                _ = cancel.cancelled() => {
                    warn!(session_id, iteration, "LLM call aborted by cancellation");
                    event_callback(LoopEvent::Cancelled {
                        iteration,
                        reason: CancelReason::LlmCallAborted,
                    });
                    return LoopResult::Cancelled {
                        session_id: session_id.to_string(),
                        iterations: iteration,
                        reason: CancelReason::LlmCallAborted,
                    };
                }
            };

            // 4. Track usage
            let tokens_used = response.usage.as_ref().map(|u| u.total()).unwrap_or(0);
            let prompt_tokens = response
                .usage
                .as_ref()
                .and_then(|u| u.prompt_tokens)
                .unwrap_or(0);
            let completion_tokens = response
                .usage
                .as_ref()
                .and_then(|u| u.completion_tokens)
                .unwrap_or(0);
            {
                let mut sessions = self.sessions.lock().await;
                if let Some(session) = sessions.get_mut(session_id) {
                    session.total_tokens_used += tokens_used;
                }
            }

            // 5. Extract response
            let assistant_message = match response.choices.first() {
                Some(choice) => choice.message.clone(),
                None => {
                    error!(session_id, iteration, "LLM returned empty choices");
                    return LoopResult::Error {
                        session_id: session_id.to_string(),
                        error: AgentError::EmptyResponse,
                        iterations: iteration + 1,
                    };
                }
            };

            let has_content = assistant_message
                .content
                .as_deref()
                .map(|c| !c.is_empty())
                .unwrap_or(false);
            let tool_calls = assistant_message.tool_calls.clone();
            let has_tool_calls = tool_calls
                .as_deref()
                .map(|tc| !tc.is_empty())
                .unwrap_or(false);

            let content_preview: String = assistant_message
                .content
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(200)
                .collect();
            let tool_call_count = tool_calls.as_deref().map(|tc| tc.len()).unwrap_or(0);

            // info!(
            //     session_id,
            //     iteration,
            //     prompt_tokens,
            //     completion_tokens,
            //     total_tokens = tokens_used,
            //     content_len = assistant_message.content.as_deref().map(|c| c.len()).unwrap_or(0),
            //     tool_call_count,
            //     finish_reason = ?response.choices.first().and_then(|c| c.finish_reason.clone()),
            //     "LLM response received"
            // );

            if has_content {
                debug!(
                    session_id,
                    iteration,
                    content_preview = %content_preview,
                    "Response content preview"
                );
            }

            // 6. Append assistant message
            {
                let mut sessions = self.sessions.lock().await;
                if let Some(session) = sessions.get_mut(session_id) {
                    session.append(assistant_message);
                }
            }

            // 7. If no tool calls, check if we have meaningful content
            if !has_tool_calls {
                let final_content = {
                    let sessions = self.sessions.lock().await;
                    sessions
                        .get(session_id)
                        .and_then(|s| s.messages.last())
                        .and_then(|m| m.content.clone())
                        .unwrap_or_default()
                };

                let content_is_empty = final_content.trim().is_empty();

                if content_is_empty {
                    // LLM returned empty content with no tool calls — nudge it to respond
                    warn!(
                        session_id,
                        iteration,
                        "LLM returned empty content with no tool calls, sending follow-up"
                    );

                    let nudge = ChatMessage::user(
                        "You returned an empty response. Please provide your answer now.",
                    );
                    {
                        let mut sessions = self.sessions.lock().await;
                        if let Some(session) = sessions.get_mut(session_id) {
                            session.append(nudge);
                        }
                    }

                    event_callback(LoopEvent::LlmResponse {
                        iteration,
                        content: Some(String::new()),
                        tool_call_count: 0,
                    });

                    continue;
                }

                let total_tokens = {
                    let sessions = self.sessions.lock().await;
                    sessions
                        .get(session_id)
                        .map(|s| s.total_tokens_used)
                        .unwrap_or(0)
                };

                info!(
                    session_id,
                    iterations = iteration + 1,
                    total_tokens,
                    content_len = final_content.len(),
                    "Agent loop completed"
                );

                event_callback(LoopEvent::LlmResponse {
                    iteration,
                    content: Some(final_content.clone()),
                    tool_call_count: 0,
                });

                return LoopResult::Completed {
                    session_id: session_id.to_string(),
                    final_content,
                    iterations: iteration + 1,
                    total_tokens,
                };
            }

            // 8. Execute tool calls (each tool call is individually cancellable)
            let tool_calls = tool_calls.expect("checked above");
            info!(
                session_id,
                iteration,
                tool_call_count = tool_calls.len(),
                tool_names = ?tool_calls.iter().map(|tc| tc.function.name.as_str()).collect::<Vec<_>>(),
                "Executing tool calls"
            );

            event_callback(LoopEvent::LlmResponse {
                iteration,
                content: if has_content {
                    Some(content_preview)
                } else {
                    None
                },
                tool_call_count: tool_calls.len(),
            });

            let mut tool_results = Vec::new();
            for tool_call in &tool_calls {
                // -- Cancel check: before each tool --
                if cancel.is_cancelled() {
                    warn!(session_id, iteration, "Cancelled before tool execution");
                    event_callback(LoopEvent::Cancelled {
                        iteration,
                        reason: CancelReason::ToolAborted {
                            tool_name: tool_call.function.name.clone(),
                        },
                    });
                    return LoopResult::Cancelled {
                        session_id: session_id.to_string(),
                        iterations: iteration,
                        reason: CancelReason::ToolAborted {
                            tool_name: tool_call.function.name.clone(),
                        },
                    };
                }

                let tool_name = &tool_call.function.name;
                debug!(
                    session_id,
                    iteration,
                    tool_name = %tool_name,
                    tool_call_id = %tool_call.id,
                    arguments = %tool_call.function.arguments,
                    "Executing tool"
                );

                let args = match serde_json::from_str(&tool_call.function.arguments) {
                    Ok(v) => v,
                    Err(e) => {
                        error!(
                            session_id,
                            iteration,
                            tool_name = %tool_name,
                            error = %e,
                            "Failed to parse tool arguments"
                        );
                        let result = ToolResult::error(
                            &tool_call.id,
                            format!("Invalid JSON arguments: {e}"),
                        );
                        event_callback(LoopEvent::ToolExecuted {
                            iteration,
                            tool_name: tool_call.function.name.clone(),
                            result: result.clone(),
                        });
                        tool_results.push(result);
                        continue;
                    }
                };

                let tool_name_owned = tool_call.function.name.clone();
                let tool_call_id = tool_call.id.clone();

                // -- Tool execution with cancellation --
                let start = std::time::Instant::now();
                let tool_result = tokio::select! {
                    result = self.tools.execute(&tool_call.function.name, args) => {
                        match result {
                            Ok(content) => {
                                let elapsed = start.elapsed();
                                info!(
                                    session_id,
                                    iteration,
                                    tool_name = %tool_name_owned,
                                    elapsed_ms = elapsed.as_millis() as u64,
                                    result_len = content.len(),
                                    "Tool executed successfully"
                                );
                                debug!(
                                    session_id,
                                    iteration,
                                    tool_name = %tool_name_owned,
                                    result_preview = %content.chars().take(300).collect::<String>(),
                                    "Tool result preview"
                                );
                                ToolResult::success(&tool_call_id, content)
                            }
                            Err(e) => {
                                let elapsed = start.elapsed();
                                error!(
                                    session_id,
                                    iteration,
                                    tool_name = %tool_name_owned,
                                    elapsed_ms = elapsed.as_millis() as u64,
                                    error = %e,
                                    "Tool execution failed"
                                );
                                ToolResult::error(&tool_call_id, e)
                            }
                        }
                    }
                    _ = cancel.cancelled() => {
                        warn!(
                            session_id,
                            iteration,
                            tool_name = %tool_name_owned,
                            "Tool execution aborted by cancellation"
                        );
                        let reason = CancelReason::ToolAborted {
                            tool_name: tool_name_owned.clone(),
                        };
                        event_callback(LoopEvent::Cancelled {
                            iteration,
                            reason: reason.clone(),
                        });
                        return LoopResult::Cancelled {
                            session_id: session_id.to_string(),
                            iterations: iteration,
                            reason,
                        };
                    }
                };

                event_callback(LoopEvent::ToolExecuted {
                    iteration,
                    tool_name: tool_name_owned,
                    result: tool_result.clone(),
                });
                tool_results.push(tool_result);
            }

            // 9. Append tool result messages
            {
                let mut sessions = self.sessions.lock().await;
                if let Some(session) = sessions.get_mut(session_id) {
                    for result in &tool_results {
                        debug!(
                            session_id,
                            iteration,
                            tool_call_id = %result.tool_call_id,
                            is_error = result.is_error,
                            result_len = result.content.len(),
                            "Appending tool result"
                        );
                    }
                    for result in tool_results {
                        session.append(ChatMessage::tool_result(
                            &result.tool_call_id,
                            &result.content,
                        ));
                    }
                }
            }
        }

        // Max iterations reached: force one wrap-up turn with tools disabled
        // so the model must answer from what it has already gathered, instead
        // of the caller falling back to a tool-result message.
        error!(
            session_id,
            max_iterations, "Agent loop exceeded max iterations"
        );

        match self.wrap_up_turn(session_id, &cancel, &model).await {
            WrapUpOutcome::Answered(content) => {
                event_callback(LoopEvent::LlmResponse {
                    iteration: max_iterations,
                    content: Some(content.clone()),
                    tool_call_count: 0,
                });
                LoopResult::MaxIterationsExceeded {
                    session_id: session_id.to_string(),
                    last_content: content,
                    iterations: max_iterations,
                }
            }
            WrapUpOutcome::Cancelled => {
                event_callback(LoopEvent::Cancelled {
                    iteration: max_iterations,
                    reason: CancelReason::LlmCallAborted,
                });
                LoopResult::Cancelled {
                    session_id: session_id.to_string(),
                    iterations: max_iterations,
                    reason: CancelReason::LlmCallAborted,
                }
            }
            WrapUpOutcome::Unavailable => LoopResult::MaxIterationsExceeded {
                session_id: session_id.to_string(),
                last_content: self.last_assistant_content(session_id).await,
                iterations: max_iterations,
            },
        }
    }

    /// One final LLM call with `tools: None` after the iteration budget is
    /// exhausted: the model cannot call tools and must produce a final
    /// answer from the context it has. Returns `Answered` with the content,
    /// `Cancelled` if the cancel token fired during the call, and
    /// `Unavailable` when the call could not produce content (context
    /// window, LLM error, empty response).
    async fn wrap_up_turn(
        &self,
        session_id: &str,
        cancel: &CancellationToken,
        model: &str,
    ) -> WrapUpOutcome {
        {
            let mut sessions = self.sessions.lock().await;
            let Some(session) = sessions.get_mut(session_id) else {
                return WrapUpOutcome::Unavailable;
            };
            session.append(ChatMessage::user(
                "The tool-use budget is exhausted. Do NOT call any more tools. Based on \
                 everything you have gathered so far, provide your final answer now.",
            ));
        }

        // Compress first if the wrap-up request would exceed the window.
        {
            let mut sessions = self.sessions.lock().await;
            if let Some(session) = sessions.get_mut(session_id) {
                if self.memory.needs_sync_compression(session) {
                    if let Err(e) = self
                        .memory
                        .compress(session, self.client.as_ref(), model)
                        .await
                    {
                        warn!(session_id, error = %e, "Wrap-up compression failed");
                    }
                }
            }
        }

        let request = {
            let sessions = self.sessions.lock().await;
            let Some(session) = sessions.get(session_id) else {
                return WrapUpOutcome::Unavailable;
            };
            let message_tokens = session.token_estimate().max(0) as usize;
            let reserved_output_tokens = self.config.max_tokens.unwrap_or(0) as usize;
            if message_tokens.saturating_add(reserved_output_tokens)
                > self.memory.config.max_context_tokens
            {
                warn!(session_id, "Wrap-up request exceeds context window, skipping");
                return WrapUpOutcome::Unavailable;
            }
            crate::client::types::ChatCompletionRequest {
                model: model.to_string(),
                messages: session.messages.clone(),
                tools: None,
                temperature: self.config.temperature,
                max_tokens: self.config.max_tokens,
            }
        };

        let response = tokio::select! {
            result = self.client.chat_completion(request) => result,
            _ = cancel.cancelled() => return WrapUpOutcome::Cancelled,
        };
        let Ok(response) = response else {
            return WrapUpOutcome::Unavailable;
        };
        let Some(message) = response.choices.first().map(|choice| choice.message.clone()) else {
            return WrapUpOutcome::Unavailable;
        };

        {
            let mut sessions = self.sessions.lock().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.append(message.clone());
            }
        }
        match message.content {
            Some(content) if !content.trim().is_empty() => WrapUpOutcome::Answered(content),
            _ => WrapUpOutcome::Unavailable,
        }
    }

    /// The most recent non-empty assistant content — the loop tail used to
    /// read `messages.last()`, which after a tool-call iteration is a tool
    /// result message, not an answer.
    async fn last_assistant_content(&self, session_id: &str) -> String {
        let sessions = self.sessions.lock().await;
        sessions
            .get(session_id)
            .map(|session| {
                session
                    .messages
                    .iter()
                    .rev()
                    .filter(|message| message.role == "assistant")
                    .filter_map(|message| message.content.clone())
                    .find(|content| !content.trim().is_empty())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::OpenAiClient;
    use crate::tool::builtin;

    fn test_agent(max_iterations: usize) -> Agent {
        let client = OpenAiClient::new(
            "http://localhost:2000/v1".to_string(),
            "qwen3".to_string(),
            "qwen3".to_string(),
        );
        let mut registry = ToolRegistry::new();
        builtin::register_all(&mut registry);

        let config = AgentConfig {
            max_iterations,
            model: "qwen3".to_string(),
            temperature: Some(0.0),
            max_tokens: Some(512),
            compression: CompressionConfig {
                max_context_tokens: 128_000,
                ..Default::default()
            },
        };

        Agent::new(Arc::new(client), Arc::new(registry), config)
    }

    #[tokio::test]
    #[ignore = "requires llama-server running at localhost:2000"]
    async fn test_simple_completion() {
        crate::init_test_tracing();
        let agent = test_agent(5);
        let session_id = agent
            .create_session(
                "You are a helpful assistant. Reply with exactly one word.",
                "What is 1+1? Reply with just the number.",
            )
            .await;

        let result = agent.run_loop(&session_id, |_| {}).await;

        match result {
            LoopResult::Completed {
                final_content,
                iterations,
                ..
            } => {
                println!("Completed in {iterations} iterations: {final_content}");
                assert!(!final_content.is_empty());
                assert!(iterations <= 5);
            }
            LoopResult::Error { error, .. } => {
                panic!("Agent error: {error}");
            }
            _ => {
                panic!("Unexpected result type");
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires llama-server running at localhost:2000"]
    async fn test_tool_call_loop() {
        crate::init_test_tracing();
        let agent = test_agent(10);

        // Create a temp file for the agent to read
        let dir = std::env::temp_dir().join("vterm_agent_test_tool");
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("secret.txt");
        std::fs::write(&file_path, "The answer is 42.").unwrap();

        let session_id = agent
            .create_session(
                "You are a helpful assistant. Use the file_read tool when asked about file contents. \
                 After reading, provide the answer directly without any tool calls.",
                &format!(
                    "Read the file at {} and tell me what the answer is.",
                    file_path.display()
                ),
            )
            .await;

        let mut events = Vec::new();
        let result = agent
            .run_loop(&session_id, |event| {
                events.push(format!("{event:?}"));
            })
            .await;

        match result {
            LoopResult::Completed { final_content, .. } => {
                println!("Final: {final_content}");
                assert!(
                    final_content.contains("42"),
                    "Expected answer to contain '42', got: {final_content}"
                );
            }
            LoopResult::Error { error, .. } => {
                panic!("Agent error: {error}");
            }
            _ => {}
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    #[ignore = "requires llama-server running at localhost:2000"]
    async fn test_commit_message_generation() {
        crate::init_test_tracing();
        let agent = test_agent(5);

        let session_id = agent
            .create_session(
                "You are a git commit message generator. \
                 Analyze the given diff and generate a concise, conventional commit message. \
                 Output ONLY the commit message, nothing else.",
                "Generate a commit message for these changes:\n\
                 --- a/src/main.rs\n\
                 +++ b/src/main.rs\n\
                 @@ -10,3 +10,5 @@\n\
                 +    println!(\"Hello, world!\");\n\
                 +    println!(\"Version 2.0\");\n\
                 ",
            )
            .await;

        let result = agent.run_loop(&session_id, |_| {}).await;

        match result {
            LoopResult::Completed { final_content, .. } => {
                println!("Commit message: {final_content}");
                assert!(!final_content.is_empty());
            }
            LoopResult::Error { error, .. } => {
                panic!("Agent error: {error}");
            }
            _ => {}
        }
    }

    #[tokio::test]
    #[ignore = "requires llama-server running at localhost:2000"]
    async fn test_codebase_analysis() {
        crate::init_test_tracing();

        let client = OpenAiClient::new(
            "http://localhost:2000/v1".to_string(),
            "qwen3".to_string(),
            "qwen3".to_string(),
        );
        let mut registry = ToolRegistry::new();
        builtin::register_all(&mut registry);

        let config = AgentConfig {
            max_iterations: 300,
            model: "qwen3".to_string(),
            temperature: Some(0.3),
            max_tokens: Some(8192),
            compression: CompressionConfig {
                max_context_tokens: 128_000,
                ..Default::default()
            },
        };

        let agent = Agent::new(Arc::new(client), Arc::new(registry), config);

        let target_dir = r"D:\Documents\Projects\codebase";

        let session_id = agent
            .create_session(
                "You are an expert code analyst. You have access to tools: file_find to discover files, \
                 file_read to read source code, code_search to search for patterns, and shell to run commands. \
                 \n\nIMPORTANT: When providing your final analysis, output the analysis DIRECTLY as plain text. \
                 Do NOT wrap your response in any special tags. Do NOT leave the response empty.\n\n\
                 Use tools systematically to explore the codebase. When you have gathered enough \
                 information, provide a comprehensive analysis WITHOUT using any more tools. \
                 Your analysis should cover:\n\
                 1. Project purpose and overall functionality\n\
                 2. Technology stack and frameworks used\n\
                 3. Architecture and module structure\n\
                 4. Key design patterns",
                format!(
                    "Analyze the codebase at: {target_dir}\n\n\
                     Start by finding the main entry points (package.json, Cargo.toml, go.mod, etc.) \
                     and exploring the directory structure. Then read key source files to understand \
                     the architecture. Provide a detailed analysis of what this project does, its \
                     technical framework, and how it's structured.\n\n\
                     IMPORTANT: Your final response must contain the analysis text directly."
                ),
            )
            .await;

        let mut tool_call_count = 0usize;
        let result = agent.run_loop(&session_id, |event| {
            match &event {
                LoopEvent::ToolExecuted { tool_name, result, .. } => {
                    tool_call_count += 1;
                    if result.is_error {
                        tracing::warn!(tool = %tool_name, "Tool error: {}", &result.content[..result.content.len().min(100)]);
                    }
                }
                _ => {}
            }
        }).await;

        match result {
            LoopResult::Completed {
                final_content,
                iterations,
                total_tokens,
                ..
            } => {
                println!("\n{}", "=".repeat(60));
                println!("Codebase Analysis (iterations={iterations}, tool_calls={tool_call_count}, tokens={total_tokens}):");
                println!("{}", "=".repeat(60));
                println!("{final_content}");
                assert!(
                    iterations > 1,
                    "Expected multiple iterations for analysis, got {iterations}"
                );
                assert!(
                    tool_call_count >= 2,
                    "Expected at least 2 tool calls, got {tool_call_count}"
                );
            }
            LoopResult::MaxIterationsExceeded {
                last_content,
                iterations,
                ..
            } => {
                println!("\nMax iterations ({iterations}) reached. Partial analysis:");
                println!("{last_content}");
                // Still count it as a pass if tool calls were made
                assert!(
                    tool_call_count >= 2,
                    "Expected at least 2 tool calls, got {tool_call_count}"
                );
            }
            LoopResult::Error { error, .. } => {
                panic!("Agent error: {error}");
            }
            _ => {}
        }
    }

    /// Test that cancellation during an LLM call works correctly.
    /// Uses a short timeout + cancel to simulate aborting a slow LLM response.
    #[tokio::test]
    #[ignore = "requires llama-server running at localhost:2000"]
    async fn test_cancel_during_llm_call() {
        crate::init_test_tracing();

        let client = OpenAiClient::new(
            "http://localhost:2000/v1".to_string(),
            "qwen3".to_string(),
            "qwen3".to_string(),
        );
        let registry = Arc::new(ToolRegistry::new());

        let config = AgentConfig {
            max_iterations: 10,
            model: "qwen3".to_string(),
            temperature: Some(0.3),
            max_tokens: Some(4096),
            compression: CompressionConfig::default(),
        };

        let agent = Agent::new(Arc::new(client), registry, config);
        let cancel = CancellationToken::new();

        let session_id = agent
            .create_session(
                "You are a helpful assistant.",
                "Count from 1 to 100, one number per line.",
            )
            .await;

        // Cancel after a short delay (while the LLM call is in-flight)
        let cancel_clone = cancel.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            cancel_clone.cancel();
        });

        let result = agent
            .run_loop_with_cancel(&session_id, cancel, |_| {})
            .await;

        match &result {
            LoopResult::Cancelled {
                iterations, reason, ..
            } => {
                println!("Cancelled at iteration {iterations}, reason: {reason:?}");
                // Should have cancelled during iteration 0's LLM call
                assert!(matches!(
                    reason,
                    CancelReason::LlmCallAborted | CancelReason::IterationCheck
                ));
            }
            other => {
                panic!("Expected Cancelled, got: {other:?}");
            }
        }
    }

    /// Test cancellation during tool execution.
    #[tokio::test]
    async fn test_cancel_during_tool_execution() {
        crate::init_test_tracing();

        // Use a mock client that returns a tool call, so we can cancel during tool exec
        let client = Arc::new(MockLlmClient);
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(SlowTool));
        let registry = Arc::new(registry);

        let config = AgentConfig {
            max_iterations: 10,
            model: "mock".to_string(),
            temperature: None,
            max_tokens: None,
            compression: CompressionConfig::default(),
        };

        let agent = Agent::new(client, registry, config);
        let cancel = CancellationToken::new();

        let session_id = agent.create_session("system", "user").await;

        // Cancel after the tool starts executing
        let cancel_clone = cancel.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            cancel_clone.cancel();
        });

        let result = agent
            .run_loop_with_cancel(&session_id, cancel, |event| {
                println!("Event: {event:?}");
            })
            .await;

        match &result {
            LoopResult::Cancelled {
                iterations, reason, ..
            } => {
                println!("Cancelled at iteration {iterations}, reason: {reason:?}");
                assert!(matches!(reason, CancelReason::ToolAborted { .. }));
            }
            other => {
                panic!("Expected Cancelled, got: {other:?}");
            }
        }
    }

    /// Mock LLM client that returns a single tool call.
    struct MockLlmClient;

    #[async_trait::async_trait]
    impl crate::client::LlmClient for MockLlmClient {
        async fn chat_completion(
            &self,
            _request: crate::client::types::ChatCompletionRequest,
        ) -> Result<crate::client::types::ChatCompletionResponse, AgentError> {
            Ok(crate::client::types::ChatCompletionResponse {
                id: Some("mock".into()),
                choices: vec![crate::client::types::Choice {
                    index: Some(0),
                    message: crate::client::ChatMessage {
                        role: "assistant".into(),
                        content: None,
                        tool_calls: Some(vec![crate::client::ToolCall {
                            id: "call_1".into(),
                            call_type: "function".into(),
                            function: crate::client::ToolCallFunction {
                                name: "slow_tool".into(),
                                arguments: "{}".into(),
                            },
                        }]),
                        tool_call_id: None,
                        name: None,
                    },
                    finish_reason: Some("tool_calls".into()),
                }],
                usage: Some(crate::client::types::Usage {
                    prompt_tokens: Some(10),
                    completion_tokens: Some(5),
                    total_tokens: Some(15),
                }),
            })
        }
    }

    /// A tool that sleeps for 10 seconds (simulating a slow tool call).
    struct SlowTool;

    #[async_trait::async_trait]
    impl crate::tool::Tool for SlowTool {
        fn definition(&self) -> crate::tool::ToolDefinition {
            crate::tool::ToolDefinition {
                r#type: "function".into(),
                function: crate::tool::FunctionDefinition {
                    name: "slow_tool".into(),
                    description: "A slow tool for testing cancellation".into(),
                    parameters: serde_json::json!({"type": "object", "properties": {}}),
                },
            }
        }

        async fn execute(&self, _args: serde_json::Value) -> Result<String, String> {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            Ok("done".into())
        }
    }

    /// Mock client that keeps calling a tool while tools are offered and
    /// answers only once they are withdrawn. Records, per request, whether
    /// tools were enabled and whether the budget notice was present.
    #[derive(Default)]
    struct WrapUpMockClient {
        calls: std::sync::Mutex<Vec<(bool, bool)>>,
    }

    #[async_trait::async_trait]
    impl crate::client::LlmClient for WrapUpMockClient {
        async fn chat_completion(
            &self,
            request: crate::client::types::ChatCompletionRequest,
        ) -> Result<crate::client::types::ChatCompletionResponse, AgentError> {
            let saw_budget_notice = request.messages.iter().any(|message| {
                message.role == "user"
                    && message
                        .content
                        .as_deref()
                        .map(|content| content.contains("Budget notice"))
                        .unwrap_or(false)
            });
            self.calls
                .lock()
                .unwrap()
                .push((request.tools.is_some(), saw_budget_notice));
            let message = if request.tools.is_some() {
                crate::client::ChatMessage {
                    role: "assistant".into(),
                    content: None,
                    tool_calls: Some(vec![crate::client::ToolCall {
                        id: "call_1".into(),
                        call_type: "function".into(),
                        function: crate::client::ToolCallFunction {
                            name: "unknown_tool".into(),
                            arguments: "{}".into(),
                        },
                    }]),
                    tool_call_id: None,
                    name: None,
                }
            } else {
                crate::client::ChatMessage::assistant("wrap-up final answer")
            };
            Ok(crate::client::types::ChatCompletionResponse {
                id: Some("mock".into()),
                choices: vec![crate::client::types::Choice {
                    index: Some(0),
                    message,
                    finish_reason: None,
                }],
                usage: None,
            })
        }
    }

    /// Exhausting the iteration budget must not bail out with a tool-result
    /// message: the loop makes one final wrap-up call with tools disabled
    /// and returns its content as `last_content`.
    #[tokio::test]
    async fn test_max_iterations_forces_wrap_up_turn() {
        let client = Arc::new(WrapUpMockClient::default());
        // A non-empty registry so the loop actually offers tools (the mock
        // calls "unknown_tool", which errors and keeps the loop going).
        let mut tool_registry = ToolRegistry::new();
        tool_registry.register(Box::new(SlowTool));
        let registry = Arc::new(tool_registry);
        let config = AgentConfig {
            max_iterations: 3,
            model: "mock".to_string(),
            temperature: None,
            max_tokens: Some(512),
            compression: CompressionConfig {
                max_context_tokens: 128_000,
                ..Default::default()
            },
        };
        let agent = Agent::new(client.clone(), registry, config);
        let session_id = agent.create_session("system", "task").await;

        let result = agent.run_loop(&session_id, |_| {}).await;

        match result {
            LoopResult::MaxIterationsExceeded {
                last_content,
                iterations,
                ..
            } => {
                assert_eq!(iterations, 3);
                assert_eq!(last_content, "wrap-up final answer");
            }
            other => panic!("Expected MaxIterationsExceeded, got: {other:?}"),
        }

        let calls = client.calls.lock().unwrap();
        assert_eq!(calls.len(), 4, "3 budgeted iterations + 1 wrap-up");
        assert!(
            calls[..3].iter().all(|(tools, _)| *tools),
            "budgeted iterations must offer tools"
        );
        assert!(!calls[3].0, "wrap-up request must disable tools");
        assert!(
            calls[1].1,
            "budget warning must be issued two iterations before the cap"
        );
    }
}
