pub mod agent;
pub mod client;
pub mod error;
pub mod memory;
pub mod session;
pub mod tool;

pub use error::AgentError;

#[cfg(test)]
pub fn init_test_tracing() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("vterm_agent=debug")),
            )
            .with_test_writer()
            .init();
    });
}
