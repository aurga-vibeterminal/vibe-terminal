mod polish;
mod supervisor;
pub(crate) mod wav;

pub use polish::{polish_voice_input_text, VoicePolishContext};
pub use supervisor::{asr_server_binary_path, AsrSupervisor, VoiceServiceState};

pub(crate) const VOICE_DISABLED_ERROR: &str = "Voice recognition is not enabled";
