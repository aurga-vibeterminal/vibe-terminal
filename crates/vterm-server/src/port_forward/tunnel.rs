use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::pin::Pin;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use std::task::{Context, Poll};

use socket2::{Domain, Protocol, Socket, Type};
use tokio::io::{copy_bidirectional, AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinSet;
use tracing::{debug, warn};

use super::types::{PortForwardActivity, PortForwardError};
use crate::ServerTlsAcceptor;

const DEFAULT_PROXY_LISTEN_BACKLOG: i32 = 1024;

#[derive(Clone, Copy)]
pub(crate) enum ProxyBindStrategy {
    Any,
    #[cfg(test)]
    Loopback,
}

pub(crate) struct ProxyListeners {
    listeners: Vec<TcpListener>,
}

impl ProxyListeners {
    pub(crate) async fn accept(&self) -> std::io::Result<(TcpStream, SocketAddr)> {
        match self.listeners.as_slice() {
            [listener] => listener.accept().await,
            [first, second] => {
                tokio::select! {
                    result = first.accept() => result,
                    result = second.accept() => result,
                }
            }
            _ => unreachable!("proxy listeners must contain one or two listeners"),
        }
    }
}

pub(crate) fn bind_proxy_listeners(
    proxy_port: u16,
    bind_strategy: ProxyBindStrategy,
) -> Result<ProxyListeners, PortForwardError> {
    let bind_targets = bind_targets(proxy_port, bind_strategy);
    let mut listeners = Vec::new();
    let mut last_bind_error = None;

    for (addr, only_v6) in bind_targets {
        match bind_listener(addr, only_v6) {
            Ok(listener) => listeners.push(listener),
            Err(error) => {
                let message = error.to_string();
                last_bind_error = Some(error);
                debug!(
                    "Failed to bind port forward listener on {}: {}",
                    addr, message
                );
            }
        }
    }

    if listeners.is_empty() {
        let message = last_bind_error
            .map(|error| error.to_string())
            .unwrap_or_else(|| "unknown bind failure".to_string());
        return Err(PortForwardError::ListenerBind {
            port: proxy_port,
            message,
        });
    }

    Ok(ProxyListeners { listeners })
}

fn bind_targets(proxy_port: u16, bind_strategy: ProxyBindStrategy) -> [(SocketAddr, bool); 2] {
    let ipv6_addr = match bind_strategy {
        ProxyBindStrategy::Any => Ipv6Addr::UNSPECIFIED,
        #[cfg(test)]
        ProxyBindStrategy::Loopback => Ipv6Addr::LOCALHOST,
    };
    let ipv4_addr = match bind_strategy {
        ProxyBindStrategy::Any => Ipv4Addr::UNSPECIFIED,
        #[cfg(test)]
        ProxyBindStrategy::Loopback => Ipv4Addr::LOCALHOST,
    };

    [
        (
            SocketAddr::V6(SocketAddrV6::new(ipv6_addr, proxy_port, 0, 0)),
            true,
        ),
        (
            SocketAddr::V4(SocketAddrV4::new(ipv4_addr, proxy_port)),
            false,
        ),
    ]
}

pub(crate) async fn validate_target(target_port: u16) -> Result<SocketAddr, PortForwardError> {
    let candidate_addrs = [
        SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, target_port, 0, 0)),
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, target_port)),
    ];

    for candidate in candidate_addrs {
        if let Ok(stream) = TcpStream::connect(candidate).await {
            drop(stream);
            return Ok(candidate);
        }
    }

    Err(PortForwardError::TargetUnavailable(target_port))
}

pub(crate) async fn run_proxy_loop(
    listeners: ProxyListeners,
    upstream_addr: SocketAddr,
    active_connections: Arc<AtomicU32>,
    activity: Arc<PortForwardActivity>,
    tls_acceptor: ServerTlsAcceptor,
) {
    let mut connections = JoinSet::new();
    loop {
        tokio::select! {
            result = listeners.accept() => {
                match result {
                    Ok((stream, peer_addr)) => {
                        let active_connections = Arc::clone(&active_connections);
                        let activity = Arc::clone(&activity);
                        let tls_acceptor = tls_acceptor.clone();
                        connections.spawn(async move {
                            if let Err(error) = proxy_connection(
                                stream,
                                peer_addr,
                                upstream_addr,
                                active_connections,
                                activity,
                                tls_acceptor,
                            )
                            .await
                            {
                                warn!(
                                    "Port forward connection {} -> {} failed: {}",
                                    peer_addr, upstream_addr, error
                                );
                            }
                        });
                    }
                    Err(error) => {
                        warn!(
                            "Port forward listener for upstream {} failed to accept: {}",
                            upstream_addr, error
                        );
                    }
                }
            }
            Some(_) = connections.join_next(), if !connections.is_empty() => {}
        }
    }
}

pub(crate) async fn run_passthrough_proxy_loop(
    listeners: ProxyListeners,
    upstream_addr: SocketAddr,
    active_connections: Arc<AtomicU32>,
    activity: Arc<PortForwardActivity>,
) {
    let mut connections = JoinSet::new();
    loop {
        tokio::select! {
            result = listeners.accept() => {
                match result {
                    Ok((stream, peer_addr)) => {
                        let active_connections = Arc::clone(&active_connections);
                        let activity = Arc::clone(&activity);
                        connections.spawn(async move {
                            if let Err(error) = passthrough_connection(
                                stream,
                                peer_addr,
                                upstream_addr,
                                active_connections,
                                activity,
                            )
                            .await
                            {
                                warn!(
                                    "Relayed port forward connection {} -> {} failed: {}",
                                    peer_addr, upstream_addr, error
                                );
                            }
                        });
                    }
                    Err(error) => {
                        warn!(
                            "Relayed port forward listener for upstream {} failed to accept: {}",
                            upstream_addr, error
                        );
                    }
                }
            }
            Some(_) = connections.join_next(), if !connections.is_empty() => {}
        }
    }
}

async fn passthrough_connection(
    mut downstream: TcpStream,
    peer_addr: SocketAddr,
    upstream_addr: SocketAddr,
    active_connections: Arc<AtomicU32>,
    activity: Arc<PortForwardActivity>,
) -> Result<(), PortForwardError> {
    let _counter = ActiveConnectionCounter::new(active_connections);
    if peer_addr.ip().is_loopback() {
        let mut first = [0u8; 1];
        if downstream.peek(&mut first).await? == 0 {
            return Ok(());
        }
        if first[0] != 0x16 {
            let upstream = TcpStream::connect(upstream_addr).await?;
            let connector = tokio_rustls::TlsConnector::from(
                vterm_common::tls::insecure_client_tls_config()
                    .map_err(|error| PortForwardError::Tls(error.to_string()))?,
            );
            let server_name = rustls::pki_types::ServerName::try_from("localhost".to_string())
                .map_err(|error| PortForwardError::Tls(error.to_string()))?;
            let mut upstream = connector
                .connect(server_name, upstream)
                .await
                .map_err(|error| PortForwardError::Tls(error.to_string()))?;
            let mut downstream = ActivityStream::new(downstream, Arc::clone(&activity));
            let (bytes_to_upstream, bytes_to_downstream) =
                copy_bidirectional(&mut downstream, &mut upstream).await?;
//             debug!(
//                 "Relayed port forward forwarded {} bytes to upstream and {} bytes to downstream for local HTTP {} -> {}",
//                 bytes_to_upstream, bytes_to_downstream, peer_addr, upstream_addr
//             );
            return Ok(());
        }
    }

    let mut downstream = ActivityStream::new(downstream, activity);
    let mut upstream = TcpStream::connect(upstream_addr).await?;
    let (bytes_to_upstream, bytes_to_downstream) =
        copy_bidirectional(&mut downstream, &mut upstream).await?;

//     debug!(
//         "Relayed port forward forwarded {} bytes to upstream and {} bytes to downstream for {} -> {}",
//         bytes_to_upstream, bytes_to_downstream, peer_addr, upstream_addr
//     );

    Ok(())
}

async fn proxy_connection(
    downstream: TcpStream,
    peer_addr: SocketAddr,
    upstream_addr: SocketAddr,
    active_connections: Arc<AtomicU32>,
    activity: Arc<PortForwardActivity>,
    tls_acceptor: ServerTlsAcceptor,
) -> Result<(), PortForwardError> {
    let _counter = ActiveConnectionCounter::new(active_connections);
    let downstream = tls_acceptor
        .accept(downstream)
        .await
        .map_err(|error| PortForwardError::Tls(error.to_string()))?;
    let mut downstream = ActivityStream::new(downstream, activity);
    let mut upstream = TcpStream::connect(upstream_addr).await?;
    let (bytes_to_upstream, bytes_to_downstream) =
        copy_bidirectional(&mut downstream, &mut upstream).await?;

//     debug!(
//         "Port forward forwarded {} bytes to upstream and {} bytes to downstream for {} -> {}",
//         bytes_to_upstream, bytes_to_downstream, peer_addr, upstream_addr
//     );

    Ok(())
}

struct ActivityStream<S> {
    inner: S,
    activity: Arc<PortForwardActivity>,
}

impl<S> ActivityStream<S> {
    fn new(inner: S, activity: Arc<PortForwardActivity>) -> Self {
        Self { inner, activity }
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for ActivityStream<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let remaining_before = buf.remaining();
        let this = self.as_mut().get_mut();
        let result = Pin::new(&mut this.inner).poll_read(cx, buf);
        if matches!(&result, Poll::Ready(Ok(()))) && buf.remaining() < remaining_before {
            this.activity.mark_activity();
        }
        result
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for ActivityStream<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let this = self.as_mut().get_mut();
        let result = Pin::new(&mut this.inner).poll_write(cx, buf);
        if let Poll::Ready(Ok(written)) = &result {
            if *written > 0 {
                this.activity.mark_activity();
            }
        }
        result
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.as_mut().get_mut().inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.as_mut().get_mut().inner).poll_shutdown(cx)
    }
}

fn bind_listener(addr: SocketAddr, only_v6: bool) -> std::io::Result<TcpListener> {
    let domain = match addr {
        SocketAddr::V4(_) => Domain::IPV4,
        SocketAddr::V6(_) => Domain::IPV6,
    };
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;

    if matches!(addr, SocketAddr::V6(_)) {
        socket.set_only_v6(only_v6)?;
    }

    socket.set_nonblocking(true)?;
    socket.bind(&addr.into())?;
    socket.listen(DEFAULT_PROXY_LISTEN_BACKLOG)?;

    let listener = std::net::TcpListener::from(socket);
    TcpListener::from_std(listener)
}

struct ActiveConnectionCounter {
    active_connections: Arc<AtomicU32>,
}

impl ActiveConnectionCounter {
    fn new(active_connections: Arc<AtomicU32>) -> Self {
        active_connections.fetch_add(1, Ordering::Relaxed);
        Self { active_connections }
    }
}

impl Drop for ActiveConnectionCounter {
    fn drop(&mut self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }
}
