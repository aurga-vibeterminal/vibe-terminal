//! Per-feature message handlers.
//!
//! [`crate::connection`] owns the wire protocol (WebSocket framing,
//! handshake, encryption, request multiplexing) and routes decoded
//! [`Message`]s here; each module implements one feature family end to end.

pub(crate) mod ai_conversations;
pub(crate) mod debug_tools;
pub(crate) mod files;
pub(crate) mod git;
pub(crate) mod port_forward;
pub(crate) mod relay_admin;
pub(crate) mod request_log;
pub(crate) mod screen;
pub(crate) mod sessions;
pub(crate) mod simulator;
pub(crate) mod translation;
pub(crate) mod voice;
