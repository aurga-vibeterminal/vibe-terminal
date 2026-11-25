mod host;
pub(crate) mod http;
mod llm_executor;
mod llm_stats;
mod mcp_executor;
mod types;
mod voice_executor;

pub use host::AddonHost;
pub use llm_executor::{
    resolve_addon_config, resolve_llm_addon_id, run_chat_completion, ChatCompletionResult,
};
pub use llm_stats::LlmRequestContext;
pub use llm_stats::{query_llm_stats, LlmStatsResponse};
pub use mcp_executor::{run_call_tool, run_get_instructions, run_list_tools};
pub use types::{
    AddonConfigField, AddonFieldType, AddonInstance, AddonManifest, AddonRunResult, AppDetectResult,
};
pub use voice_executor::{run_transcription, TranscriptionResult};
