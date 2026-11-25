//! Screen-streaming coordination (spec
//! `docs/superpowers/specs/2026-08-06-screen-streaming-coordination.md`):
//! vterm-server's handle on the local vterm-screen-server process.
//!
//! - [`ipc`] — the ctl pipe/UDS client (async façade over a blocking pump).
//! - [`supervisor`] — spawn/attach, config push, token minting.

pub mod ctl;
pub mod ipc;
pub mod supervisor;

pub use supervisor::{
    EncoderCatalog, ScreenServerSupervisor, StreamStart, DEFAULT_MAX_RESOLUTION, DEFAULT_STREAM_FPS,
};

/// Convert a ctl catalog entry to the WSS protocol shape (spec §5.1).
pub fn source_entry_to_proto(
    entry: &crate::screen::ctl::SourceEntry,
) -> vterm_core::protocol::ScreenSourceInfo {
    use vterm_core::protocol::{ScreenSourceInfo, ScreenSourceKind, ScreenSourceStatus};
    ScreenSourceInfo {
        spec: entry.spec.clone(),
        kind: match entry.kind {
            crate::screen::ctl::SourceKind::Screen => ScreenSourceKind::Screen,
            crate::screen::ctl::SourceKind::Window => ScreenSourceKind::Window,
            crate::screen::ctl::SourceKind::IosSimulator => ScreenSourceKind::IosSimulator,
            crate::screen::ctl::SourceKind::AndroidDevice => ScreenSourceKind::AndroidDevice,
        },
        title: entry.title.clone(),
        status: match &entry.status {
            crate::screen::ctl::SourceStatus::Available => ScreenSourceStatus::Available,
            crate::screen::ctl::SourceStatus::Busy => ScreenSourceStatus::Busy,
            crate::screen::ctl::SourceStatus::Streaming => ScreenSourceStatus::Streaming,
            crate::screen::ctl::SourceStatus::Unavailable { reason } => {
                ScreenSourceStatus::Unavailable {
                    reason: reason.clone(),
                }
            }
        },
        width: entry.width,
        height: entry.height,
        primary: entry.primary,
    }
}
