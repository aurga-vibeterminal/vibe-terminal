use std::sync::{
    atomic::{AtomicU32, AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;

use tokio::sync::Notify;
use tokio::time::Instant;
use vterm_common::PortForwardInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PortForwardPolicy {
    pub(crate) enabled: bool,
    pub(crate) idle_timeout: Option<Duration>,
}

impl Default for PortForwardPolicy {
    fn default() -> Self {
        Self {
            enabled: crate::config::default_proxy_enabled(),
            // Matches `ServerConfig`'s default (both are replaced by the
            // persisted policy once startup applies the config).
            idle_timeout: Some(Duration::from_secs(
                u64::from(crate::config::DEFAULT_PROXY_IDLE_TIMEOUT_MINUTES) * 60,
            )),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PortForwardError {
    #[error("Port forwarding is disabled")]
    Disabled,
    #[error("Target port must be non-zero")]
    InvalidTargetPort,
    #[error("No local service is reachable on target port {0}")]
    TargetUnavailable(u16),
    #[error("No free port forwarding port available in range {start}-{end}")]
    PortRangeExhausted { start: u16, end: u16 },
    #[error("Failed to bind port forward listener on port {port}: {message}")]
    ListenerBind { port: u16, message: String },
    #[error("No active port forward listener on port {0}")]
    PortNotFound(u16),
    #[error("TLS error: {0}")]
    Tls(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortForwardLease {
    pub target_port: u16,
    pub proxy_port: u16,
    pub reused_existing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PortForwardTarget {
    Local(u16),
    Remote(std::net::SocketAddr),
}

impl PortForwardTarget {
    pub(crate) fn port(self) -> u16 {
        match self {
            Self::Local(port) => port,
            Self::Remote(addr) => addr.port(),
        }
    }
}

pub(crate) struct PortForwardActivity {
    started_at: Instant,
    last_activity_ms: AtomicU64,
}

impl PortForwardActivity {
    pub(crate) fn new() -> Self {
        Self {
            started_at: Instant::now(),
            last_activity_ms: AtomicU64::new(0),
        }
    }

    pub(crate) fn mark_activity(&self) {
        let elapsed_ms = u64::try_from(self.started_at.elapsed().as_millis()).unwrap_or(u64::MAX);
        self.last_activity_ms
            .fetch_max(elapsed_ms, Ordering::Relaxed);
    }

    pub(crate) fn idle_for(&self) -> Duration {
        let last_activity = Duration::from_millis(self.last_activity_ms.load(Ordering::Relaxed));
        self.started_at.elapsed().saturating_sub(last_activity)
    }
}

pub(crate) struct PortForwardRuntime {
    pub(crate) target: PortForwardTarget,
    pub(crate) proxy_port: u16,
    pub(crate) active_connections: Arc<AtomicU32>,
    pub(crate) activity: Arc<PortForwardActivity>,
    pub(crate) close_notify: Arc<Notify>,
    pub(crate) listener_task: tokio::task::JoinHandle<()>,
}

impl PortForwardRuntime {
    pub(crate) fn new(
        target: PortForwardTarget,
        proxy_port: u16,
        active_connections: Arc<AtomicU32>,
        activity: Arc<PortForwardActivity>,
        close_notify: Arc<Notify>,
        listener_task: tokio::task::JoinHandle<()>,
    ) -> Self {
        Self {
            target,
            proxy_port,
            active_connections,
            activity,
            close_notify,
            listener_task,
        }
    }

    pub(crate) fn info(&self) -> PortForwardInfo {
        PortForwardInfo {
            target_port: self.target.port(),
            proxy_port: self.proxy_port,
            active_connection_count: self.active_connections.load(Ordering::Relaxed),
        }
    }
}
