use std::sync::{atomic::AtomicU32, Arc};
use std::time::Duration;

use tokio::sync::{watch, Mutex, Notify};
use tracing::{info, warn};
use vterm_common::PortForwardInfo;

use crate::ServerTlsAcceptor;

use super::registry::PortForwardRegistry;
use super::tunnel::{
    bind_proxy_listeners, run_passthrough_proxy_loop, run_proxy_loop, validate_target,
    ProxyBindStrategy,
};
use super::types::{
    PortForwardActivity, PortForwardError, PortForwardLease, PortForwardPolicy, PortForwardRuntime,
    PortForwardTarget,
};

pub struct PortForwardManager {
    port_range_start: u16,
    port_range_end: u16,
    bind_strategy: ProxyBindStrategy,
    registry: Arc<Mutex<PortForwardRegistry>>,
    tls_acceptor: ServerTlsAcceptor,
    policy: watch::Sender<PortForwardPolicy>,
}

impl PortForwardManager {
    pub(crate) fn with_port_range(
        port_range_start: u16,
        port_range_end: u16,
        tls_acceptor: ServerTlsAcceptor,
    ) -> Self {
        Self::with_port_range_and_strategy(
            port_range_start,
            port_range_end,
            ProxyBindStrategy::Any,
            tls_acceptor,
        )
    }

    pub(crate) fn with_port_range_and_strategy(
        port_range_start: u16,
        port_range_end: u16,
        bind_strategy: ProxyBindStrategy,
        tls_acceptor: ServerTlsAcceptor,
    ) -> Self {
        let (port_range_start, port_range_end) =
            normalized_port_range(port_range_start, port_range_end);
        let (policy, _) = watch::channel(PortForwardPolicy::default());
        Self {
            port_range_start,
            port_range_end,
            bind_strategy,
            registry: Arc::new(Mutex::new(PortForwardRegistry::default())),
            tls_acceptor,
            policy,
        }
    }

    pub fn update_settings(&self, enabled: bool, idle_timeout_minutes: u32) {
        let idle_timeout = (idle_timeout_minutes > 0)
            .then(|| Duration::from_secs(u64::from(idle_timeout_minutes) * 60));
        self.policy.send_replace(PortForwardPolicy {
            enabled,
            idle_timeout,
        });
    }

    #[cfg(test)]
    fn set_idle_timeout_for_tests(&self, idle_timeout: Duration) {
        let mut policy = *self.policy.borrow();
        policy.idle_timeout = Some(idle_timeout);
        self.policy.send_replace(policy);
    }

    fn ensure_enabled(&self) -> Result<(), PortForwardError> {
        if self.policy.borrow().enabled {
            Ok(())
        } else {
            Err(PortForwardError::Disabled)
        }
    }

    pub async fn ensure_forward(
        &self,
        target_port: u16,
    ) -> Result<PortForwardLease, PortForwardError> {
        self.ensure_enabled()?;
        if target_port == 0 {
            warn!("Rejected port forwarding request for invalid target port 0");
            return Err(PortForwardError::InvalidTargetPort);
        }

        let target = PortForwardTarget::Local(target_port);
        if let Some(existing) = self.existing_forward(target).await {
            info!(
                "Reusing existing port forward mapping: target port {} -> proxy port {}",
                existing.target_port, existing.proxy_port
            );
            return Ok(existing);
        }

        let upstream_addr = match validate_target(target_port).await {
            Ok(addr) => addr,
            Err(error) => {
                warn!(
                    "Forward target port {} is not reachable on localhost: {}",
                    target_port, error
                );
                return Err(error);
            }
        };
        self.ensure_forward_target(target, upstream_addr).await
    }

    pub async fn ensure_relayed_forward(
        &self,
        upstream_addr: std::net::SocketAddr,
    ) -> Result<PortForwardLease, PortForwardError> {
        self.ensure_enabled()?;
        if upstream_addr.port() == 0 {
            warn!("Rejected relayed port forwarding request for invalid target port 0");
            return Err(PortForwardError::InvalidTargetPort);
        }

        self.ensure_forward_target(PortForwardTarget::Remote(upstream_addr), upstream_addr)
            .await
    }

    async fn ensure_forward_target(
        &self,
        target: PortForwardTarget,
        upstream_addr: std::net::SocketAddr,
    ) -> Result<PortForwardLease, PortForwardError> {
        self.ensure_enabled()?;
        let target_port = target.port();
        if let Some(existing) = self.existing_forward(target).await {
            info!(
                "Reusing port forward mapping: target {} -> proxy port {}",
                upstream_addr, existing.proxy_port
            );
            return Ok(existing);
        }

        let mut registry = self.registry.lock().await;
        if let Some(existing) = registry.get_by_target(target) {
            info!(
                "Reusing port forward mapping after registry lookup: target {} -> proxy port {}",
                upstream_addr, existing.proxy_port
            );
            return Ok(PortForwardLease {
                target_port,
                proxy_port: existing.proxy_port,
                reused_existing: true,
            });
        }

        for proxy_port in
            candidate_proxy_ports(target_port, self.port_range_start, self.port_range_end)
        {
            if registry.contains_proxy_port(proxy_port) {
                continue;
            }

            let listeners = match bind_proxy_listeners(proxy_port, self.bind_strategy) {
                Ok(listeners) => listeners,
                Err(PortForwardError::ListenerBind { .. }) => continue,
                Err(error) => return Err(error),
            };

            let active_connections = Arc::new(AtomicU32::new(0));
            let activity = Arc::new(PortForwardActivity::new());
            let close_notify = Arc::new(Notify::new());
            let listener_task = match target {
                PortForwardTarget::Local(_) => tokio::spawn(run_proxy_loop(
                    listeners,
                    upstream_addr,
                    Arc::clone(&active_connections),
                    Arc::clone(&activity),
                    self.tls_acceptor.clone(),
                )),
                PortForwardTarget::Remote(_) => tokio::spawn(run_passthrough_proxy_loop(
                    listeners,
                    upstream_addr,
                    Arc::clone(&active_connections),
                    Arc::clone(&activity),
                )),
            };

            registry.insert(PortForwardRuntime::new(
                target,
                proxy_port,
                active_connections,
                Arc::clone(&activity),
                Arc::clone(&close_notify),
                listener_task,
            ));
            drop(registry);

            tokio::spawn(run_idle_timeout_monitor(
                Arc::clone(&self.registry),
                proxy_port,
                activity,
                close_notify,
                self.policy.subscribe(),
            ));

            info!(
                "Created port forward mapping: target {} -> proxy port {}",
                upstream_addr, proxy_port
            );
            return Ok(PortForwardLease {
                target_port,
                proxy_port,
                reused_existing: false,
            });
        }

        warn!(
            "Failed to allocate port forwarding port for target {} in range {}-{}",
            upstream_addr, self.port_range_start, self.port_range_end
        );
        Err(PortForwardError::PortRangeExhausted {
            start: self.port_range_start,
            end: self.port_range_end,
        })
    }

    pub async fn list_forwards(&self) -> Vec<PortForwardInfo> {
        let registry = self.registry.lock().await;
        registry.list()
    }

    /// Stops the listener and active connections bound to `proxy_port`.
    /// The mapping is removed from the registry, so a later
    /// [`ensure_forward`](Self::ensure_forward) call allocates a fresh mapping.
    pub async fn stop_forward(&self, proxy_port: u16) -> Result<PortForwardInfo, PortForwardError> {
        let runtime = {
            let mut registry = self.registry.lock().await;
            registry.remove_by_proxy_port(proxy_port)
        };
        let Some(runtime) = runtime else {
            return Err(PortForwardError::PortNotFound(proxy_port));
        };
        info!(
            target_port = runtime.target.port(),
            proxy_port = runtime.proxy_port,
            "Stopping port-forwarding listener on manual request"
        );
        let info = stop_runtime(runtime).await;
        info!(
            target_port = info.target_port,
            proxy_port = info.proxy_port,
            "Port-forward listener stopped on manual request"
        );
        Ok(info)
    }

    pub async fn shutdown_all(&self) {
        let runtimes = self.registry.lock().await.take_all();
        info!(
            count = runtimes.len(),
            "Stopping redirect/port-forwarding listeners"
        );
        for runtime in runtimes {
            let info = runtime.info();
            info!(
                target_port = info.target_port,
                proxy_port = info.proxy_port,
                "Stopping redirect/port-forwarding listener"
            );
            stop_runtime(runtime).await;
            info!(
                target_port = info.target_port,
                proxy_port = info.proxy_port,
                "Redirect/port-forwarding listener stopped"
            );
        }
        info!("All redirect/port-forwarding listeners stopped");
    }

    async fn existing_forward(&self, target: PortForwardTarget) -> Option<PortForwardLease> {
        let registry = self.registry.lock().await;
        registry
            .get_by_target(target)
            .map(|existing| PortForwardLease {
                target_port: target.port(),
                proxy_port: existing.proxy_port,
                reused_existing: true,
            })
    }
}

async fn run_idle_timeout_monitor(
    registry: Arc<Mutex<PortForwardRegistry>>,
    proxy_port: u16,
    activity: Arc<PortForwardActivity>,
    close_notify: Arc<Notify>,
    mut policy: watch::Receiver<PortForwardPolicy>,
) {
    loop {
        let idle_timeout = policy.borrow_and_update().idle_timeout;
        let Some(idle_timeout) = idle_timeout else {
            tokio::select! {
                _ = close_notify.notified() => return,
                changed = policy.changed() => {
                    if changed.is_err() {
                        return;
                    }
                }
            }
            continue;
        };

        let idle_for = activity.idle_for();
        if idle_for >= idle_timeout {
            let runtime = {
                let mut registry = registry.lock().await;
                let is_same_forward = registry
                    .get_by_proxy_port(proxy_port)
                    .is_some_and(|runtime| Arc::ptr_eq(&runtime.activity, &activity));
                if is_same_forward {
                    registry.remove_by_proxy_port(proxy_port)
                } else {
                    None
                }
            };
            let Some(runtime) = runtime else {
                return;
            };
            let info = runtime.info();
            info!(
                target_port = info.target_port,
                proxy_port = info.proxy_port,
                idle_timeout_seconds = idle_timeout.as_secs(),
                "Stopping port-forwarding listener after inactivity"
            );
            stop_runtime(runtime).await;
            info!(
                target_port = info.target_port,
                proxy_port = info.proxy_port,
                "Idle port-forward listener stopped"
            );
            return;
        }

        let remaining = idle_timeout.saturating_sub(idle_for);
        tokio::select! {
            _ = tokio::time::sleep(remaining) => {}
            _ = close_notify.notified() => return,
            changed = policy.changed() => {
                if changed.is_err() {
                    return;
                }
            }
        }
    }
}

async fn stop_runtime(runtime: PortForwardRuntime) -> PortForwardInfo {
    runtime.close_notify.notify_one();
    let info = runtime.info();
    runtime.listener_task.abort();
    let _ = runtime.listener_task.await;
    info
}

fn normalized_port_range(start: u16, end: u16) -> (u16, u16) {
    if start <= end {
        (start, end)
    } else {
        (end, start)
    }
}

fn candidate_proxy_ports(target_port: u16, start: u16, end: u16) -> Vec<u16> {
    let width = end - start + 1;
    let offset = target_port % width;
    (0..width)
        .map(|step| start + ((offset + step) % width))
        .collect()
}

// Exercises the full rustls path (rcgen identity + insecure client
// connector) and therefore runs on every platform — Windows included since
// the native-tls/SChannel listener was removed.
#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio_rustls::TlsAcceptor as RustlsTlsAcceptor;

    use super::*;
    use crate::port_forward::tunnel::bind_proxy_listeners;

    /// Manager with forwarding explicitly enabled. The policy default
    /// follows the build flavor (`PortForwardPolicy::default()` is off in
    /// headless builds), and these tests exercise the forwarding machinery,
    /// not the default.
    fn enabled_test_manager(start: u16, end: u16) -> PortForwardManager {
        let manager = PortForwardManager::with_port_range_and_strategy(
            start,
            end,
            ProxyBindStrategy::Loopback,
            test_tls_acceptor(),
        );
        manager.update_settings(true, 0);
        manager
    }

    #[tokio::test]
    async fn ensure_forward_reuses_existing_mapping() {
        let target_listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let target_port = target_listener.local_addr().unwrap().port();

        let accept_task = tokio::spawn(async move {
            let _ = target_listener.accept().await.unwrap();
        });

        let proxy_port = available_test_proxy_port();
        let manager = enabled_test_manager(proxy_port, proxy_port);
        let first = manager.ensure_forward(target_port).await.unwrap();
        let second = manager.ensure_forward(target_port).await.unwrap();

        assert_eq!(first.proxy_port, second.proxy_port);
        assert!(!first.reused_existing);
        assert!(second.reused_existing);

        accept_task.abort();
    }

    #[tokio::test]
    async fn proxy_forwards_tcp_payloads() {
        let target_listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let target_port = target_listener.local_addr().unwrap().port();

        let target_task = tokio::spawn(async move {
            let (validation_stream, _) = target_listener.accept().await.unwrap();
            drop(validation_stream);

            let (mut target_stream, _) = target_listener.accept().await.unwrap();
            let mut request = [0u8; 4];
            target_stream.read_exact(&mut request).await.unwrap();
            assert_eq!(&request, b"ping");
            target_stream.write_all(b"pong").await.unwrap();
        });

        let proxy_port = available_test_proxy_port();
        let manager = enabled_test_manager(proxy_port, proxy_port);
        let lease = manager.ensure_forward(target_port).await.unwrap();
        let connector = tokio_rustls::TlsConnector::from(
            vterm_common::tls::insecure_client_tls_config().unwrap(),
        );
        let proxy_stream = TcpStream::connect(("127.0.0.1", lease.proxy_port))
            .await
            .unwrap();
        let server_name = rustls::pki_types::ServerName::try_from("localhost".to_string()).unwrap();
        let mut proxy_stream = connector.connect(server_name, proxy_stream).await.unwrap();

        proxy_stream.write_all(b"ping").await.unwrap();
        let mut response = [0u8; 4];
        proxy_stream.read_exact(&mut response).await.unwrap();

        assert_eq!(&response, b"pong");

        target_task.await.unwrap();
    }

    #[tokio::test]
    async fn relayed_proxy_supports_local_http_and_tls_and_lists_both_stages() {
        let target_listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let target_port = target_listener.local_addr().unwrap().port();
        let target_task = tokio::spawn(async move {
            let (validation_stream, _) = target_listener.accept().await.unwrap();
            drop(validation_stream);

            for _ in 0..2 {
                let (mut target_stream, _) = target_listener.accept().await.unwrap();
                let mut request = [0u8; 4];
                target_stream.read_exact(&mut request).await.unwrap();
                assert_eq!(&request, b"ping");
                target_stream.write_all(b"pong").await.unwrap();
            }
        });

        let shared_proxy_port = available_test_proxy_port();
        let relay_proxy_port = loop {
            let candidate = available_test_proxy_port();
            if candidate != shared_proxy_port {
                break candidate;
            }
        };
        let shared_manager = enabled_test_manager(shared_proxy_port, shared_proxy_port);
        let relay_manager = enabled_test_manager(relay_proxy_port, relay_proxy_port);

        let shared_lease = shared_manager.ensure_forward(target_port).await.unwrap();
        let shared_upstream = std::net::SocketAddr::new(
            std::net::Ipv4Addr::LOCALHOST.into(),
            shared_lease.proxy_port,
        );
        let relay_lease = relay_manager
            .ensure_relayed_forward(shared_upstream)
            .await
            .unwrap();
        let reused = relay_manager
            .ensure_relayed_forward(shared_upstream)
            .await
            .unwrap();
        assert_eq!(relay_lease.proxy_port, reused.proxy_port);
        assert!(reused.reused_existing);

        let shared_forwards = shared_manager.list_forwards().await;
        assert_eq!(shared_forwards.len(), 1);
        assert_eq!(shared_forwards[0].target_port, target_port);
        assert_eq!(shared_forwards[0].proxy_port, shared_lease.proxy_port);
        let relay_forwards = relay_manager.list_forwards().await;
        assert_eq!(relay_forwards.len(), 1);
        assert_eq!(relay_forwards[0].target_port, shared_lease.proxy_port);
        assert_eq!(relay_forwards[0].proxy_port, relay_lease.proxy_port);

        let connector = tokio_rustls::TlsConnector::from(
            vterm_common::tls::insecure_client_tls_config().unwrap(),
        );
        let relay_stream = TcpStream::connect(("127.0.0.1", relay_lease.proxy_port))
            .await
            .unwrap();
        let server_name = rustls::pki_types::ServerName::try_from("localhost".to_string()).unwrap();
        let mut relay_stream = connector.connect(server_name, relay_stream).await.unwrap();
        relay_stream.write_all(b"ping").await.unwrap();
        let mut response = [0u8; 4];
        relay_stream.read_exact(&mut response).await.unwrap();
        assert_eq!(&response, b"pong");
        drop(relay_stream);

        let mut local_http_stream = TcpStream::connect(("127.0.0.1", relay_lease.proxy_port))
            .await
            .unwrap();
        local_http_stream.write_all(b"ping").await.unwrap();
        let mut response = [0u8; 4];
        local_http_stream.read_exact(&mut response).await.unwrap();
        assert_eq!(&response, b"pong");
        target_task.await.unwrap();

        let stopped = relay_manager
            .stop_forward(relay_lease.proxy_port)
            .await
            .unwrap();
        assert_eq!(stopped.target_port, shared_lease.proxy_port);
        assert!(relay_manager.list_forwards().await.is_empty());
        shared_manager
            .stop_forward(shared_lease.proxy_port)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn stop_forward_removes_listener() {
        let target_listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let target_port = target_listener.local_addr().unwrap().port();

        let accept_task = tokio::spawn(async move {
            let _ = target_listener.accept().await.unwrap();
        });

        let proxy_port = available_test_proxy_port();
        let manager = enabled_test_manager(proxy_port, proxy_port);
        manager.ensure_forward(target_port).await.unwrap();
        assert_eq!(manager.list_forwards().await.len(), 1);

        let stopped = manager.stop_forward(proxy_port).await.unwrap();
        assert_eq!(stopped.proxy_port, proxy_port);
        assert_eq!(stopped.target_port, target_port);
        assert!(manager.list_forwards().await.is_empty());

        // The port must be free again, and stopping an unknown port must fail.
        assert!(bind_proxy_listeners(proxy_port, ProxyBindStrategy::Loopback).is_ok());
        assert!(matches!(
            manager.stop_forward(proxy_port).await,
            Err(PortForwardError::PortNotFound(port)) if port == proxy_port
        ));

        accept_task.abort();
    }

    #[tokio::test]
    async fn disabled_forwarding_rejects_create_and_reuse_until_reenabled() {
        let target_listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let target_port = target_listener.local_addr().unwrap().port();
        let accept_task = tokio::spawn(async move {
            let _ = target_listener.accept().await.unwrap();
        });

        let proxy_port = available_test_proxy_port();
        let manager = enabled_test_manager(proxy_port, proxy_port);
        manager.ensure_forward(target_port).await.unwrap();
        manager.update_settings(false, 0);

        assert!(matches!(
            manager.ensure_forward(target_port).await,
            Err(PortForwardError::Disabled)
        ));
        assert!(matches!(
            manager
                .ensure_relayed_forward(std::net::SocketAddr::new(
                    std::net::Ipv4Addr::LOCALHOST.into(),
                    target_port,
                ))
                .await,
            Err(PortForwardError::Disabled)
        ));

        manager.update_settings(true, 0);
        let reused = manager.ensure_forward(target_port).await.unwrap();
        assert!(reused.reused_existing);
        manager.stop_forward(proxy_port).await.unwrap();
        accept_task.abort();
    }

    #[tokio::test]
    async fn traffic_resets_idle_timeout_and_idle_connections_are_closed() {
        let target_listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let target_port = target_listener.local_addr().unwrap().port();
        let target_task = tokio::spawn(async move {
            let (validation_stream, _) = target_listener.accept().await.unwrap();
            drop(validation_stream);

            let (mut target_stream, _) = target_listener.accept().await.unwrap();
            for _ in 0..2 {
                let mut request = [0u8; 4];
                target_stream.read_exact(&mut request).await.unwrap();
                assert_eq!(&request, b"ping");
                target_stream.write_all(b"pong").await.unwrap();
            }
            std::future::pending::<()>().await;
        });

        let proxy_port = available_test_proxy_port();
        let manager = enabled_test_manager(proxy_port, proxy_port);
        manager.set_idle_timeout_for_tests(std::time::Duration::from_millis(1500));
        manager.ensure_forward(target_port).await.unwrap();
        let mut proxy_stream = connect_test_proxy(proxy_port).await;

        proxy_stream.write_all(b"ping").await.unwrap();
        let mut response = [0u8; 4];
        proxy_stream.read_exact(&mut response).await.unwrap();
        assert_eq!(&response, b"pong");

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        proxy_stream.write_all(b"ping").await.unwrap();
        proxy_stream.read_exact(&mut response).await.unwrap();
        assert_eq!(&response, b"pong");

        tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
        assert_eq!(manager.list_forwards().await.len(), 1);

        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            while !manager.list_forwards().await.is_empty() {
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            }
        })
        .await
        .expect("port forward did not expire after the last traffic");
        let mut byte = [0u8; 1];
        let read = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            proxy_stream.read(&mut byte),
        )
        .await
        .expect("idle proxy connection was not closed");
        assert!(matches!(read, Ok(0) | Err(_)));
        target_task.abort();
    }

    #[tokio::test(start_paused = true)]
    async fn old_idle_monitor_does_not_remove_recreated_forward() {
        let target_listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let target_port = target_listener.local_addr().unwrap().port();
        let target_task = tokio::spawn(async move {
            for _ in 0..2 {
                let (validation_stream, _) = target_listener.accept().await.unwrap();
                drop(validation_stream);
            }
        });

        let proxy_port = available_test_proxy_port();
        let manager = PortForwardManager::with_port_range_and_strategy(
            proxy_port,
            proxy_port,
            ProxyBindStrategy::Loopback,
            test_tls_acceptor(),
        );
        manager.update_settings(true, 1);
        manager.ensure_forward(target_port).await.unwrap();

        tokio::time::advance(std::time::Duration::from_secs(30)).await;
        manager.stop_forward(proxy_port).await.unwrap();
        let recreated = manager.ensure_forward(target_port).await.unwrap();
        assert!(!recreated.reused_existing);

        tokio::time::advance(std::time::Duration::from_secs(30)).await;
        yield_to_idle_monitors().await;
        assert_eq!(manager.list_forwards().await.len(), 1);

        tokio::time::advance(std::time::Duration::from_secs(30)).await;
        yield_to_idle_monitors().await;
        assert!(manager.list_forwards().await.is_empty());
        target_task.await.unwrap();
    }

    #[tokio::test(start_paused = true)]
    async fn updated_idle_timeout_applies_to_existing_forward() {
        let target_listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let target_port = target_listener.local_addr().unwrap().port();
        let accept_task = tokio::spawn(async move {
            let _ = target_listener.accept().await.unwrap();
        });

        let proxy_port = available_test_proxy_port();
        let manager = enabled_test_manager(proxy_port, proxy_port);
        manager.ensure_forward(target_port).await.unwrap();
        tokio::time::advance(std::time::Duration::from_secs(120)).await;
        yield_to_idle_monitors().await;
        assert_eq!(manager.list_forwards().await.len(), 1);

        manager.update_settings(true, 1);
        tokio::task::yield_now().await;
        tokio::time::advance(std::time::Duration::from_secs(60)).await;
        yield_to_idle_monitors().await;
        assert!(manager.list_forwards().await.is_empty());
        accept_task.abort();
    }

    async fn connect_test_proxy(proxy_port: u16) -> tokio_rustls::client::TlsStream<TcpStream> {
        let connector = tokio_rustls::TlsConnector::from(
            vterm_common::tls::insecure_client_tls_config().unwrap(),
        );
        let proxy_stream = TcpStream::connect(("127.0.0.1", proxy_port)).await.unwrap();
        let server_name = rustls::pki_types::ServerName::try_from("localhost".to_string()).unwrap();
        connector.connect(server_name, proxy_stream).await.unwrap()
    }

    async fn yield_to_idle_monitors() {
        for _ in 0..8 {
            tokio::task::yield_now().await;
        }
    }

    fn test_tls_acceptor() -> RustlsTlsAcceptor {
        // rustls cannot read the repo's server.pfx, so mint an in-memory
        // self-signed identity instead; the test client disables verification.
        let certified_key = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let key = rustls::pki_types::PrivateKeyDer::Pkcs8(
            certified_key.signing_key.serialize_der().into(),
        );
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![certified_key.cert.der().clone()], key)
            .unwrap();
        RustlsTlsAcceptor::from(Arc::new(config))
    }

    fn available_test_proxy_port() -> u16 {
        for _ in 0..32 {
            let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
            let port = listener.local_addr().unwrap().port();
            drop(listener);

            if let Ok(listeners) = bind_proxy_listeners(port, ProxyBindStrategy::Loopback) {
                drop(listeners);
                return port;
            }
        }

        panic!("failed to find an available proxy port for tests");
    }
}
