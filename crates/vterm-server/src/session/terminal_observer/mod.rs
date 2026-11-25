mod manager;
mod types;

#[cfg(test)]
pub use manager::SessionObserver;
pub use manager::{AppDetectionMode, ObservedTerminalState};
pub use manager::{ObserverHandle, TerminalObserverManager};
pub use types::TerminalObserverEvent;
