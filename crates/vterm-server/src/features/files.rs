//! File-system features for sessions: fs watchers, directory listings,
//! file search, file read/write/create/delete/paste/rename, static HTTP
//! hosting of a directory, and the body-streaming fast paths for the
//! request transport (file fetch, download, upload, save-with-body).

use anyhow::anyhow;
use tokio::sync::mpsc;
use tracing::{info, warn};
use vterm_common::{ErrorCode, SessionId};
use vterm_core::protocol::{Message, TempFileUsage};
use vterm_transport::{OutboundCommand as RequestOutboundCommand, ReceivedBody, RequestId};

use crate::connection::{send_correlated_response_message, Connection};
use crate::fs_watcher::FsWatcherError;

/// Short label for a received transport body, for transfer diagnostics.
fn body_kind_label(body: &ReceivedBody) -> String {
    match body {
        ReceivedBody::Empty => "empty".to_string(),
        ReceivedBody::Memory(bytes) => format!("memory({} bytes)", bytes.len()),
        ReceivedBody::TempFile(spool) => format!("tempfile({} bytes)", spool.len()),
    }
}

async fn ensure_remote_static_http_forward(
    manager: &crate::port_forward::PortForwardManager,
    static_port: u16,
) -> Result<u16, crate::port_forward::PortForwardError> {
    let lease = manager.ensure_forward(static_port).await?;
    info!(
        "Remote static HTTP port forward ready: static_port={} proxy_port={} reused={}",
        static_port, lease.proxy_port, lease.reused_existing
    );
    Ok(lease.proxy_port)
}

impl Connection {
    pub(crate) async fn handle_file_message(&mut self, message: Message) -> anyhow::Result<()> {
        match message {
            Message::SetFsWatchers {
                session_id,
                watches,
            } => {
                //                 info!(
                //                     "Client {} set fs watchers for session {} file_content={:?} git={} directories={:?}",
                //                     self.addr,
                //                     session_id,
                //                     watches.file_content.as_ref().map(|watch| watch.path.as_str()),
                //                     watches.git.is_some(),
                //                     watches.directories,
                //                 );
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                self.fs_watched_session_ids.insert(session_id);
                // Applied inline (not spawned) so updates on this terminal
                // connection are applied in message order: the last
                // SetFsWatchers the client sent wins.
                let result = self
                    .fs_watcher_manager
                    .update_session_watches(
                        session_id,
                        watches,
                        &self.session_manager,
                        self.tx.clone(),
                    )
                    .await;
                match result {
                    Ok(()) => {
                        //                         info!(
                        //                             "Applied fs watchers for session {} on connection {}",
                        //                             session_id, self.addr
                        //                         );
                    }
                    Err(error) => {
                        warn!(
                            "Failed to apply fs watchers for session {} on connection {}: {}",
                            session_id, self.addr, error
                        );
                        // A missing path is not a protocol violation. Most
                        // commonly a WSL / remote-shell tab reports a
                        // Linux-style cwd ("~/work/...") that never resolves
                        // on the Windows host, and the client would flash
                        // "path does not exist" on every tab switch. Log it
                        // server-side and drop the client-facing error;
                        // other failures still surface.
                        if !matches!(error, FsWatcherError::PathNotFound(_)) {
                            self.send_message(Message::Error {
                                code: ErrorCode::InvalidMessage,
                                message: error.to_string(),
                            })
                            .await?;
                        }
                    }
                }
            }
            Message::ListPathContent { session_id, path } => {
                info!(
                    "Client {} listed path content for session {} path={}",
                    self.addr, session_id, path
                );
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                let (message, pending_annotation) = self
                    .session_manager
                    .list_path_content(session_id, path)
                    .await?;
                self.send_message(message).await?;
                if let Some(pending_annotation) = pending_annotation {
                    let session_manager = self.session_manager.clone();
                    let response_tx = self.tx.clone();
                    tokio::spawn(async move {
                        match session_manager
                            .finish_path_content_annotation(pending_annotation)
                            .await
                        {
                            Ok(Some(message)) => {
                                let _ = response_tx.send(message);
                            }
                            Ok(None) => {}
                            Err(error) => warn!(
                                session_id = %session_id,
                                error = %error,
                                "failed to finish path-listing Git annotation"
                            ),
                        }
                    });
                }
                // The listing just moved the session's viewing path; a git
                // watch the client asked for may have been resolved against
                // the pre-listing (stale) path and dropped, so re-resolve it
                // now that the path is authoritative. Spawned to keep the
                // resolve's latency off the listing response.
                let watcher = self.fs_watcher_manager.clone();
                let session_manager = self.session_manager.clone();
                tokio::spawn(async move {
                    if let Err(error) = watcher
                        .refresh_session_git_watch(session_id, &session_manager)
                        .await
                    {
                        warn!(
                            session_id = %session_id,
                            error = %error,
                            "failed to refresh git watch after path listing"
                        );
                    }
                });
            }
            Message::FindFiles {
                session_id,
                request_id,
                root_path,
                query,
                include,
                exclude,
                case_sensitive,
                whole_word,
                use_regex,
                max_results,
            } => {
                info!(
                    "Client {} requested FindFiles session={} request_id={} query_bytes={} include='{}' exclude='{}' case_sensitive={} whole_word={} regex={} max_results={}",
                    self.addr,
                    session_id,
                    request_id,
                    query.len(),
                    include,
                    exclude,
                    case_sensitive,
                    whole_word,
                    use_regex,
                    max_results
                );
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .spawn_find_files(
                        session_id,
                        request_id.clone(),
                        root_path,
                        query.clone(),
                        include,
                        exclude,
                        case_sensitive,
                        whole_word,
                        use_regex,
                        max_results,
                        self.tx.clone(),
                    )
                    .await
                {
                    Ok(()) => {}
                    Err(error) => {
                        self.send_message(Message::FindFilesResult {
                            session_id,
                            request_id,
                            query,
                            chunk_index: 0,
                            is_final: true,
                            results: Vec::new(),
                            error: Some(error.to_string()),
                        })
                        .await?;
                    }
                }
            }
            Message::SearchPathNames {
                session_id,
                request_id,
                root_path,
                query,
                max_results,
            } => {
                info!(
                    "Client {} requested SearchPathNames session={} request_id={} query_bytes={} max_results={}",
                    self.addr,
                    session_id,
                    request_id,
                    query.len(),
                    max_results
                );
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .spawn_search_path_names(
                        session_id,
                        request_id.clone(),
                        root_path.clone(),
                        query.clone(),
                        max_results,
                        self.tx.clone(),
                    )
                    .await
                {
                    Ok(()) => {}
                    Err(error) => {
                        self.send_message(Message::SearchPathNamesResult {
                            session_id,
                            request_id,
                            root_path,
                            query,
                            is_final: true,
                            results: Vec::new(),
                            error: Some(error.to_string()),
                        })
                        .await?;
                    }
                }
            }
            Message::FetchFile {
                session_id,
                path,
                max_size,
            } => {
                info!(
                    "Client {} requested file for session {} path={} max_size={}",
                    self.addr, session_id, path, max_size
                );
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                let message = self
                    .session_manager
                    .read_file(session_id, path.clone(), max_size)
                    .await?;
                self.send_message(message).await?;
                // Recent-file bookkeeping + state broadcast happen after the
                // response is on the wire — they don't gate the open latency.
                let _ = self
                    .session_manager
                    .record_recent_file_and_broadcast(session_id, path)
                    .await;
            }
            Message::SaveFile {
                session_id,
                path,
                data,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self.session_manager.save_file(session_id, path, data).await {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::CreatePath {
                session_id,
                path,
                is_dir,
            } => {
                info!(
                    "Client {} requested create_path session={} path={} is_dir={}",
                    self.addr, session_id, path, is_dir
                );
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                let message = self
                    .session_manager
                    .create_path(session_id, path, is_dir)
                    .await?;
                self.send_message(message).await?;
            }
            Message::DeletePaths { session_id, paths } => {
                info!(
                    "Client {} requested delete_paths session={} count={}",
                    self.addr,
                    session_id,
                    paths.len()
                );
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                let message = self.session_manager.delete_paths(session_id, paths).await?;
                self.send_message(message).await?;
            }
            Message::PastePaths {
                session_id,
                src_paths,
                dst_dir,
                is_cut,
            } => {
                info!(
                    "Client {} requested paste_paths session={} count={} dst={} is_cut={}",
                    self.addr,
                    session_id,
                    src_paths.len(),
                    dst_dir,
                    is_cut
                );
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                let message = self
                    .session_manager
                    .paste_paths(session_id, src_paths, dst_dir, is_cut)
                    .await?;
                self.send_message(message).await?;
            }
            Message::RenamePath {
                session_id,
                old_path,
                new_path,
            } => {
                info!(
                    "Client {} requested rename_path session={} old={} new={}",
                    self.addr, session_id, old_path, new_path
                );
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                let message = self
                    .session_manager
                    .rename_path(session_id, old_path, new_path)
                    .await?;
                self.send_message(message).await?;
            }
            Message::StartStaticHttpServer {
                session_id,
                path,
                request_port_forward,
            } => {
                info!(
                    "StartStaticHttpServer received: session_id={:?} path={} via_relay={} request_port_forward={}",
                    session_id, path, self.via_relay, request_port_forward
                );

                // If this is a remote session (on a shared server), forward the request
                // to that shared server instead of handling it locally.
                if self.relay_manager.is_remote_session(session_id).await {
                    info!(
                        "StartStaticHttpServer for remote session {:?}, forwarding to shared server",
                        session_id
                    );
                    if let Some(server_id) = self
                        .relay_manager
                        .remote_server_for_session(session_id)
                        .await
                    {
                        // Forward the request to the shared server via the primary connection
                        if let Err(e) = self
                            .relay_manager
                            .send_to_shared_server(
                                &server_id,
                                Message::StartStaticHttpServer {
                                    session_id,
                                    path: path.clone(),
                                    request_port_forward,
                                },
                            )
                            .await
                        {
                            warn!(
                                "Failed to forward StartStaticHttpServer to shared server {}: {}",
                                server_id, e
                            );
                            self.send_message(Message::StartStaticHttpServerResult {
                                session_id,
                                path,
                                port: None,
                                url: None,
                                share_token: None,
                                port_forwarded: false,
                                error: Some(format!("Failed to forward to shared server: {}", e)),
                            })
                            .await?;
                        }
                        // The shared server will process the request and send the result
                        // back through its own channel. We need to wait for that.
                        // For now, just return - the response will be handled asynchronously.
                        return Ok(());
                    }
                    // If we can't forward, fall through to error handling
                    self.send_message(Message::StartStaticHttpServerResult {
                        session_id,
                        path,
                        port: None,
                        url: None,
                        share_token: None,
                        port_forwarded: false,
                        error: Some("Shared server not available".to_string()),
                    })
                    .await?;
                    return Ok(());
                }

                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                let (share_token, result) =
                    match self.static_http_manager.host_directory(&path).await {
                        Ok(hosted) => (Some(hosted.share_token), Ok(hosted.port)),
                        Err(error) => (None, Err(error)),
                    };
                info!(
                    "host_directory result: Ok={} via_relay={}",
                    result.is_ok(),
                    self.via_relay
                );

                // Check if this is a shared server by checking if we have an upstream relay connection.
                // Shared servers have an upstream relay connection to coordinate with the relay.
                let is_shared_server = self.relay_manager.is_shared_server().await;

                let (final_port, final_url, final_error, port_forwarded) = match result {
                    Ok(port) if is_shared_server => {
                        info!(
                            "Static HTTP server started on port {} for shared server session {:?}, requesting relay port forwarding",
                            port, session_id
                        );
                        match self
                            .relay_manager
                            .request_shared_port_forward_and_wait(session_id, port)
                            .await
                        {
                            Ok(relay_port) => {
                                info!(
                                    "Relay port forward ready for shared server: static_port={} relay_port={}",
                                    port, relay_port
                                );
                                (Some(relay_port), None, None, true)
                            }
                            Err(error) => {
                                warn!("Failed to establish shared port forwarding: {}", error);
                                (None, None, Some(format!("{error:#}")), false)
                            }
                        }
                    }
                    Ok(port) if request_port_forward => {
                        info!(
                            "Static HTTP server started on port {} for remote client, creating port forward",
                            port
                        );
                        match ensure_remote_static_http_forward(&self.port_forward_manager, port)
                            .await
                        {
                            Ok(proxy_port) => (Some(proxy_port), None, None, true),
                            Err(error) => {
                                warn!(
                                    "Failed to establish remote static HTTP port forwarding: {}",
                                    error
                                );
                                (None, None, Some(format!("{error:#}")), false)
                            }
                        }
                    }
                    Ok(port) => (
                        Some(port),
                        Some(format!("https://127.0.0.1:{port}/")),
                        None,
                        false,
                    ),
                    Err(error) => (None, None, Some(format!("{error:#}")), false),
                };

                self.send_message(Message::StartStaticHttpServerResult {
                    session_id,
                    path,
                    port: final_port,
                    url: final_url,
                    share_token,
                    port_forwarded,
                    error: final_error,
                })
                .await?;
            }
            other => {
                warn!("Unhandled file message: {}", other.msg_name());
            }
        }

        Ok(())
    }
}

/// Transport fast path for [`Message::FetchFile`]: answers with a body
/// streamed by the transport producer instead of one assembled Message.
/// Returns `true` when the request was handled here; `false` defers to the
/// generic message path (remote sessions).
pub(crate) async fn handle_fetch_file_request(
    connection: &mut Connection,
    outbound_tx: &mpsc::Sender<RequestOutboundCommand>,
    request_id: RequestId,
    session_id: SessionId,
    path: String,
    max_size: u64,
) -> anyhow::Result<bool> {
    if connection.relay_manager.is_remote_session(session_id).await {
        return Ok(false);
    }
    if connection.ensure_session_access(session_id).await? {
        let response = connection
            .session_manager
            .read_file_transport(session_id, path, max_size)
            .await?;
        outbound_tx
            .send(RequestOutboundCommand::Message {
                request_id,
                message_sequence: 0,
                message: response,
            })
            .await
            .map_err(|_| anyhow!("Persistent request writer stopped"))?;
    } else {
        let mut rx = connection.rx.take().expect("request child rx must exist");
        let response = rx.recv().await.unwrap_or_else(|| Message::Error {
            code: ErrorCode::AuthenticationFailed,
            message: "Session access denied".to_string(),
        });
        send_correlated_response_message(outbound_tx, request_id, 0, response).await?;
    }
    outbound_tx
        .send(RequestOutboundCommand::Finish(request_id))
        .await
        .map_err(|_| anyhow!("Persistent request writer stopped"))?;
    Ok(true)
}

/// Transport fast path for [`Message::DownloadStart`]: the response body is
/// a framed entry stream produced by a lazy server-side walk.
/// Returns `true` when the request was handled here; `false` defers to the
/// generic message path (remote sessions).
pub(crate) async fn handle_download_request(
    connection: &mut Connection,
    outbound_tx: &mpsc::Sender<RequestOutboundCommand>,
    request_id: RequestId,
    task_id: String,
    session_id: SessionId,
    paths: Vec<String>,
) -> anyhow::Result<bool> {
    if connection.relay_manager.is_remote_session(session_id).await {
        return Ok(false);
    }
    info!(
        %task_id,
        session_id = ?session_id,
        ?paths,
        "download request received"
    );
    if connection.ensure_session_access(session_id).await? {
        let response = connection
            .session_manager
            .download_transport(session_id, task_id, paths)
            .await?;
        outbound_tx
            .send(RequestOutboundCommand::Message {
                request_id,
                message_sequence: 0,
                message: response,
            })
            .await
            .map_err(|_| anyhow!("Persistent request writer stopped"))?;
    } else {
        let mut rx = connection.rx.take().expect("request child rx must exist");
        let response = rx.recv().await.unwrap_or_else(|| Message::Error {
            code: ErrorCode::AuthenticationFailed,
            message: "Session access denied".to_string(),
        });
        send_correlated_response_message(outbound_tx, request_id, 0, response).await?;
    }
    outbound_tx
        .send(RequestOutboundCommand::Finish(request_id))
        .await
        .map_err(|_| anyhow!("Persistent request writer stopped"))?;
    Ok(true)
}

/// Upload fast path: consumes the streamed request body (files land as
/// bytes arrive) once the request has finished assembling.
pub(crate) async fn handle_upload_body_request(
    connection: &mut Connection,
    session_id: SessionId,
    task_id: String,
    dst_dir: String,
    body: ReceivedBody,
) -> anyhow::Result<()> {
    info!(
        %task_id,
        session_id = ?session_id,
        %dst_dir,
        body = %body_kind_label(&body),
        "upload request received"
    );
    if connection.ensure_session_access(session_id).await? {
        let response = connection
            .session_manager
            .upload_body(session_id, task_id, dst_dir, body)
            .await?;
        connection.send_message(response).await
    } else {
        Ok(())
    }
}

pub(crate) async fn handle_temp_file_upload_body_request(
    connection: &mut Connection,
    session_id: SessionId,
    task_id: String,
    file_name: String,
    content_type: String,
    usage: TempFileUsage,
    body: ReceivedBody,
) -> anyhow::Result<()> {
    if connection.ensure_session_access(session_id).await? {
        let response = connection
            .session_manager
            .upload_temp_file(session_id, task_id, file_name, content_type, usage, body)
            .await;
        connection.send_message(response).await
    } else {
        Ok(())
    }
}

/// Save-with-body fast path: a [`Message::SaveFile`] whose inline `data` is
/// empty rides its content in the request body instead of the message.
pub(crate) async fn handle_save_file_body_request(
    connection: &mut Connection,
    session_id: SessionId,
    path: String,
    body: ReceivedBody,
) -> anyhow::Result<()> {
    if connection.ensure_session_access(session_id).await? {
        let response = connection
            .session_manager
            .save_file_body(session_id, path, body)
            .await?;
        connection.send_message(response).await
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;
    use crate::config::{ConfigManager, StaticHttpBindAddress};
    use crate::port_forward::PortForwardManager;
    use crate::static_http::StaticHttpManager;

    fn test_tls_acceptor() -> crate::ServerTlsAcceptor {
        let certified_key = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let key = rustls::pki_types::PrivateKeyDer::Pkcs8(
            certified_key.signing_key.serialize_der().into(),
        );
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![certified_key.cert.der().clone()], key)
            .unwrap();
        crate::ServerTlsAcceptor::from(Arc::new(config))
    }

    fn available_test_port(excluded: Option<u16>) -> u16 {
        for _ in 0..32 {
            let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
            let port = listener.local_addr().unwrap().port();
            drop(listener);
            if Some(port) != excluded {
                return port;
            }
        }
        panic!("failed to allocate a distinct test port");
    }

    #[tokio::test]
    async fn remote_static_http_forward_exposes_loopback_service_over_https() {
        let static_port = available_test_port(None);
        let proxy_port = available_test_port(Some(static_port));
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tag = format!("remote-static-forward-{}-{unique}", std::process::id());
        let config_path = std::env::temp_dir().join(format!("{tag}.json"));
        let root_dir = std::env::temp_dir().join(tag);
        let _ = tokio::fs::remove_file(&config_path).await;
        let _ = tokio::fs::remove_dir_all(&root_dir).await;
        tokio::fs::create_dir_all(&root_dir).await.unwrap();
        tokio::fs::write(root_dir.join("hello.txt"), "hello via remote forward")
            .await
            .unwrap();

        let config_manager = Arc::new(
            ConfigManager::load_from_path(config_path.clone())
                .await
                .unwrap(),
        );
        let mut config = config_manager.snapshot().await;
        config.server.static_http_bind_address = StaticHttpBindAddress::Local;
        config.server.static_http_port_start = static_port;
        config.server.static_http_port_end = static_port;
        config_manager.replace(config).await.unwrap();

        let static_http = StaticHttpManager::new(config_manager, test_tls_acceptor());
        let root = root_dir.canonicalize().unwrap().display().to_string();
        let hosted = static_http.host_directory(&root).await.unwrap();
        let hosted_port = hosted.port;
        assert_eq!(hosted_port, static_port);

        let mut bootstrap = tokio::net::TcpStream::connect(("127.0.0.1", hosted_port))
            .await
            .unwrap();
        bootstrap
            .write_all(
                format!(
                    "GET /hello.txt?share_token={} HTTP/1.1\r\nHost: localhost\r\n\r\n",
                    hosted.share_token
                )
                .as_bytes(),
            )
            .await
            .unwrap();
        let mut response = Vec::new();
        bootstrap.read_to_end(&mut response).await.unwrap();
        let response = String::from_utf8_lossy(&response);
        let session_cookie = response
            .lines()
            .find_map(|line| line.strip_prefix("Set-Cookie: "))
            .and_then(|value| value.split(';').next())
            .unwrap()
            .to_string();

        let forward_manager =
            PortForwardManager::with_port_range(proxy_port, proxy_port, test_tls_acceptor());
        // The policy default is off in headless builds; this test exercises
        // the forwarding path, not the default.
        forward_manager.update_settings(true, 0);
        let forwarded_port = ensure_remote_static_http_forward(&forward_manager, hosted_port)
            .await
            .unwrap();
        assert_eq!(forwarded_port, proxy_port);
        let forwards = forward_manager.list_forwards().await;
        assert_eq!(forwards.len(), 1);
        assert_eq!(forwards[0].target_port, hosted_port);
        assert_eq!(forwards[0].proxy_port, forwarded_port);

        let connector = tokio_rustls::TlsConnector::from(
            vterm_common::tls::insecure_client_tls_config().unwrap(),
        );
        let server_name = rustls::pki_types::ServerName::try_from("localhost".to_string()).unwrap();
        let tcp = tokio::net::TcpStream::connect(("127.0.0.1", forwarded_port))
            .await
            .unwrap();
        let mut tls = connector.connect(server_name, tcp).await.unwrap();
        tls.write_all(
            format!(
                "GET /hello.txt HTTP/1.1\r\nHost: localhost\r\nCookie: {session_cookie}\r\n\r\n"
            )
            .as_bytes(),
        )
        .await
        .unwrap();
        let response = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let mut response = Vec::new();
            while !response.ends_with(b"hello via remote forward") {
                let mut chunk = [0u8; 1024];
                let count = tls.read(&mut chunk).await.unwrap();
                assert!(count > 0, "forward closed before sending the response body");
                response.extend_from_slice(&chunk[..count]);
                assert!(response.len() <= 16 * 1024, "unexpectedly large response");
            }
            response
        })
        .await
        .unwrap();
        let response = String::from_utf8_lossy(&response);
        assert!(
            response.starts_with("HTTP/1.1 200 OK"),
            "response: {response}"
        );

        forward_manager.stop_forward(forwarded_port).await.unwrap();
        assert!(static_http.stop_directory(&root).await);
        let _ = tokio::fs::remove_file(config_path).await;
        let _ = tokio::fs::remove_dir_all(root_dir).await;
    }
}
