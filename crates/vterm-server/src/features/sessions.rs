//! Session and tab lifecycle: create/split/close/detach/rename sessions,
//! tab groups and layouts, terminal input and resize, history fetch, and
//! terminal stream (viewer) start/end.

use tracing::{info, trace, warn};
use vterm_common::{ErrorCode, SessionId, TerminalSize};
use vterm_core::protocol::{ConnectionKind, Message};

use crate::connection::Connection;

impl Connection {
    pub(crate) async fn handle_session_message(&mut self, message: Message) -> anyhow::Result<()> {
        match message {
            Message::CreateSession {
                shell,
                profile_id,
                size,
                env,
                cwd,
                title,
                sibling_session_id,
                server_id,
                tab_group_id,
                duplicate_session_id,
                app,
                background,
                foreground,
            } => {
                let local_server_id = self.config_manager.snapshot().await.server.server_id;
                let session_id = if let Some(server_id) = server_id {
                    if server_id == local_server_id {
                        self.session_manager
                            .create_session_with_theme_colors(
                                size,
                                shell,
                                profile_id,
                                env,
                                cwd,
                                sibling_session_id,
                                duplicate_session_id,
                                tab_group_id,
                                title.clone(),
                                app,
                                background,
                                foreground,
                            )
                            .await?
                    } else if !self
                        .relay_manager
                        .create_remote_session(
                            &server_id,
                            shell,
                            profile_id,
                            size,
                            env,
                            cwd,
                            sibling_session_id,
                            duplicate_session_id,
                            tab_group_id,
                            title,
                            app,
                            background,
                            foreground,
                            self.tx.clone(),
                        )
                        .await?
                    {
                        self.send_error(ErrorCode::UnknownSession, "Shared server not found")
                            .await;
                        return Ok(());
                    } else {
                        return Ok(());
                    }
                } else {
                    self.session_manager
                        .create_session_with_theme_colors(
                            size,
                            shell,
                            profile_id,
                            env,
                            cwd,
                            sibling_session_id,
                            duplicate_session_id,
                            tab_group_id,
                            title.clone(),
                            app,
                            background,
                            foreground,
                        )
                        .await?
                };
                self.send_message(Message::SessionCreated { session_id })
                    .await?;
                info!("Session created: {} by {}", session_id, self.addr);
            }
            Message::SplitTerminal {
                target_session_id,
                placement,
                size,
                profile_id,
            } => {
                if self
                    .relay_manager
                    .is_remote_session(target_session_id)
                    .await
                {
                    // Session belongs to a shared (relay) server: forward the
                    // split there. The shared server creates the new session and
                    // replies with SessionCreated, which the relay routes back
                    // to this client.
                    if !self
                        .relay_manager
                        .split_remote_terminal(
                            target_session_id,
                            placement,
                            size,
                            profile_id,
                            self.tx.clone(),
                        )
                        .await?
                    {
                        self.send_error(ErrorCode::UnknownSession, "Session not found")
                            .await;
                    }
                } else {
                    let session_id = self
                        .session_manager
                        .split_terminal(target_session_id, placement, size, profile_id)
                        .await?;
                    self.send_message(Message::SessionCreated { session_id })
                        .await?;
                    info!(
                        "Terminal split: target={} new={} placement={:?} by {}",
                        target_session_id, session_id, placement, self.addr
                    );
                }
            }
            Message::FetchHistoryRaw {
                session_id,
                before_offset,
                max_bytes,
            } => {
                let fetch_started = std::time::Instant::now();
                trace!(
                    "{:?} connection {} requested history for session {} before_offset={} max_bytes={}",
                    self.connection_kind, self.addr, session_id, before_offset, max_bytes
                );
                if !self.ensure_session_access(session_id).await? {
                    warn!(
                        "FetchHistoryRaw session={} denied access after {:.3}s",
                        session_id,
                        fetch_started.elapsed().as_secs_f64()
                    );
                    return Ok(());
                }
                let chunk = match self
                    .session_manager
                    .fetch_history_chunk(session_id, before_offset, max_bytes as usize)
                    .await
                {
                    Ok(chunk) => chunk,
                    Err(error) => {
                        warn!(
                            "FetchHistoryRaw session={} fetch_history_chunk failed after {:.3}s: {}",
                            session_id,
                            fetch_started.elapsed().as_secs_f64(),
                            error
                        );
                        return Err(anyhow::anyhow!(error.to_string()));
                    }
                };
                if let Message::HistoryRawChunk {
                    start_offset,
                    end_offset,
                    ref data,
                    ..
                } = chunk
                {
                    trace!(
                        "FetchHistoryRaw session={} local fetch ok start={} end={} bytes={} after {:.3}s, sending",
                        session_id,
                        start_offset,
                        end_offset,
                        data.len(),
                        fetch_started.elapsed().as_secs_f64()
                    );
                }
                self.send_message(chunk).await?;
            }
            Message::ExportSessionRawHistory { session_id } => {
                info!(
                    "{:?} connection {} requested raw history export for session {}",
                    self.connection_kind, self.addr, session_id
                );
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self.session_manager.export_raw_history(session_id).await {
                    Ok(path) => {
                        info!(
                            "Raw history export complete session={} path={}",
                            session_id,
                            path.display()
                        );
                        self.send_message(Message::ExportSessionRawHistoryResult {
                            session_id,
                            path: Some(path.display().to_string()),
                            error: None,
                        })
                        .await?;
                    }
                    Err(error) => {
                        self.send_message(Message::ExportSessionRawHistoryResult {
                            session_id,
                            path: None,
                            error: Some(error.to_string()),
                        })
                        .await?;
                    }
                }
            }
            Message::CloseSession { session_id } => {
                if self.relay_manager.is_remote_session(session_id).await {
                    self.relay_manager.close_remote_session(session_id).await?;
                } else {
                    self.session_manager.close_session(session_id).await?;
                }
                self.streamed_session_ids.remove(&session_id);
                self.fs_watcher_manager.remove_session(session_id);
                self.fs_watched_session_ids.remove(&session_id);
                info!("Session closed: {} by {}", session_id, self.addr);
            }
            Message::CloseTab { tab_id } => {
                // A tab's id is the root session id. When the tab belongs to a
                // shared (relay) server, forward the close there; the shared
                // server broadcasts the updated session/tab state back.
                if self.relay_manager.is_remote_session(tab_id).await {
                    if !self.relay_manager.close_remote_tab(tab_id).await? {
                        self.send_error(ErrorCode::UnknownSession, "Session not found")
                            .await;
                    }
                } else {
                    let closed_ids = self.session_manager.close_tab(tab_id).await?;
                    for session_id in &closed_ids {
                        self.fs_watcher_manager.remove_session(*session_id);
                        self.fs_watched_session_ids.remove(session_id);
                        self.streamed_session_ids.remove(session_id);
                    }
                    info!(
                        "Tab closed: {} ({} sessions) by {}",
                        tab_id,
                        closed_ids.len(),
                        self.addr
                    );
                }
            }
            Message::DetachSession { session_id } => {
                if self.relay_manager.is_remote_session(session_id).await {
                    self.relay_manager.detach_remote_session(session_id).await?;
                } else {
                    self.session_manager.detach_session(session_id).await?;
                }
                info!("Session detached: {} by {}", session_id, self.addr);
            }
            Message::RenameSession {
                session_id,
                session_name,
            } => {
                if self.relay_manager.is_remote_session(session_id).await {
                    self.relay_manager
                        .rename_remote_session(session_id, session_name)
                        .await?;
                } else {
                    self.session_manager
                        .rename_session(session_id, session_name)
                        .await?;
                }
            }
            Message::ApplyEnvironment {
                session_id,
                environment_id,
                enabled,
            } => {
                // Routed over the request connection (possibly a relay request
                // tunnel); respond on the same connection so the requester gets
                // a definitive result instead of relying on later state pushes.
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                let error = match self
                    .session_manager
                    .apply_environment(session_id, &environment_id, enabled)
                    .await
                {
                    Ok(()) => None,
                    Err(error) => Some(error.to_string()),
                };
                self.send_message(Message::ApplyEnvironmentResult {
                    session_id,
                    environment_id,
                    error,
                })
                .await?;
            }
            Message::SetNotificationEnabled {
                session_id,
                enabled,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                let error = match self
                    .session_manager
                    .set_notification_enabled(session_id, enabled)
                    .await
                {
                    Ok(()) => None,
                    Err(error) => Some(error.to_string()),
                };
                self.send_message(Message::SetNotificationEnabledResult { session_id, error })
                    .await?;
            }
            Message::SetAutoConfirmEnabled {
                session_id,
                enabled,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                let error = match self
                    .session_manager
                    .set_auto_confirm(session_id, enabled)
                    .await
                {
                    Ok(()) => None,
                    Err(error) => Some(error.to_string()),
                };
                self.send_message(Message::SetAutoConfirmEnabledResult { session_id, error })
                    .await?;
            }
            Message::SetSessionEncoding {
                session_id,
                encoding,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                let error = self
                    .session_manager
                    .set_session_encoding(session_id, encoding)
                    .await
                    .err()
                    .map(|error| error.to_string());
                self.send_message(Message::SetSessionEncodingResult { session_id, error })
                    .await?;
            }
            Message::SetTerminalThemeColors {
                session_id,
                background,
                foreground,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                // Fire-and-forget: a stale message for a session that raced
                // ahead of close is as harmless as a late watcher update.
                let _ = self
                    .session_manager
                    .apply_terminal_theme_colors(session_id, background, foreground)
                    .await;
            }
            Message::ClearSessionScreen { session_id } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                let error = self
                    .session_manager
                    .clear_session_screen(session_id)
                    .await
                    .err()
                    .map(|error| error.to_string());
                self.send_message(Message::ClearSessionScreenResult { session_id, error })
                    .await?;
            }
            Message::Input {
                session_id, data, ..
            } => {
                if !self.ensure_session_access(session_id).await? {
                    warn!(
                        "{:?} connection {} rejected terminal input for inactive stream session {}",
                        self.connection_kind, self.addr, session_id
                    );
                    return Ok(());
                }
                self.session_manager
                    .forward_input(session_id, Some(&self.tx), data)
                    .await?;
            }
            Message::ConfirmSession { session_id } => {
                // User-initiated confirm: write `\r` to the session PTY. Routed
                // over the request connection (possibly a relay request tunnel)
                // so it works even when the session is not the active tab (no
                // live terminal stream). forward_input records the input via the
                // observer; the resulting SessionList snapshot clears any
                // pending attention prompt on every client.
                if !self.ensure_session_access(session_id).await? {
                    warn!(
                        "{:?} connection {} rejected confirm for inaccessible session {}",
                        self.connection_kind, self.addr, session_id
                    );
                    return Ok(());
                }
                info!(
                    "Client {} confirmed session {} (writing \\r to PTY)",
                    self.addr, session_id
                );
                let error = match self
                    .session_manager
                    .forward_input(session_id, None, b"\r".to_vec())
                    .await
                {
                    Ok(()) => None,
                    Err(error) => Some(error.to_string()),
                };
                self.send_message(Message::ConfirmSessionResult { session_id, error })
                    .await?;
            }
            Message::Resize { session_id, size } => {
                // info!(
                //     "Client {} resized session {} to {}x{}",
                //     self.addr, session_id, size.rows, size.cols
                // );
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                self.session_manager
                    .resize_session(session_id, &self.tx, size)
                    .await?;
            }
            Message::ListSessions => {
                let sessions = self.relay_manager.list_sessions().await?;
                self.send_message(Message::SessionList { sessions }).await?;
            }
            Message::ListTabGroups => {
                let groups = self.relay_manager.list_tab_groups().await?;
                self.send_message(Message::TabGroupList { groups }).await?;
            }
            Message::GetTabInfoList { server_id } => {
                let local_server_id = self.config_manager.snapshot().await.server.server_id;
                if let Some(server_id) = server_id {
                    let tabs = if server_id == local_server_id {
                        self.session_manager.list_tab_infos().await
                    } else {
                        self.relay_manager
                            .list_all_tab_infos()
                            .await
                            .into_iter()
                            .filter(|tab| tab.server_id.as_deref() == Some(server_id.as_str()))
                            .collect()
                    };
                    self.send_message(Message::TabInfoList {
                        server_id: Some(server_id),
                        tabs,
                    })
                    .await?;
                } else {
                    let tabs = self.relay_manager.list_all_tab_infos().await;
                    self.send_message(Message::TabInfoList {
                        server_id: None,
                        tabs,
                    })
                    .await?;
                }
            }
            Message::UpdateTerminalLayout { tab_id, layout } => {
                if self.relay_manager.is_remote_session(tab_id).await {
                    if !self
                        .relay_manager
                        .update_remote_tab_layout(tab_id, layout)
                        .await?
                    {
                        self.send_error(ErrorCode::UnknownSession, "Session not found")
                            .await;
                    }
                } else {
                    self.session_manager
                        .update_tab_layout(tab_id, layout)
                        .await?;
                    let server_id = self.config_manager.snapshot().await.server.server_id;
                    let tabs = self.session_manager.list_tab_infos().await;
                    self.send_message(Message::TabInfoList {
                        server_id: Some(server_id),
                        tabs,
                    })
                    .await?;
                }
            }
            Message::CreateTabGroup {
                server_id,
                name,
                initial_session_id,
            } => {
                let local_server_id = self.config_manager.snapshot().await.server.server_id;
                let initial_session_is_remote = match initial_session_id {
                    Some(session_id) => self.relay_manager.is_remote_session(session_id).await,
                    None => false,
                };
                if let Some(server_id) = server_id.as_ref().filter(|id| *id != &local_server_id) {
                    if !self
                        .relay_manager
                        .create_remote_tab_group(server_id, name, initial_session_id)
                        .await?
                    {
                        self.send_error(
                            ErrorCode::InvalidMessage,
                            "Cannot create a terminal group on an unknown server",
                        )
                        .await;
                    }
                } else if initial_session_is_remote {
                    self.send_error(
                        ErrorCode::InvalidMessage,
                        "Cannot create a terminal group on a different server",
                    )
                    .await;
                } else {
                    self.session_manager
                        .create_tab_group(name, initial_session_id)
                        .await?;
                }
            }
            Message::DeleteTabGroup {
                server_id,
                group_id,
            } => {
                let local_server_id = self.config_manager.snapshot().await.server.server_id;
                if let Some(server_id) = server_id.as_ref().filter(|id| *id != &local_server_id) {
                    if !self
                        .relay_manager
                        .delete_remote_tab_group(server_id, group_id)
                        .await?
                    {
                        self.send_error(
                            ErrorCode::InvalidMessage,
                            "Cannot delete a terminal group on an unknown server",
                        )
                        .await;
                    }
                } else {
                    self.session_manager.delete_tab_group(group_id).await?;
                }
            }
            Message::RenameTabGroup {
                server_id,
                group_id,
                name,
            } => {
                let local_server_id = self.config_manager.snapshot().await.server.server_id;
                if let Some(server_id) = server_id.as_ref().filter(|id| *id != &local_server_id) {
                    if !self
                        .relay_manager
                        .rename_remote_tab_group(server_id, group_id, name)
                        .await?
                    {
                        self.send_error(
                            ErrorCode::InvalidMessage,
                            "Cannot rename a terminal group on an unknown server",
                        )
                        .await;
                    }
                } else {
                    self.session_manager
                        .rename_tab_group(group_id, name)
                        .await?;
                }
            }
            Message::MoveTabToGroup {
                session_id,
                server_id,
                group_id,
            } => {
                let local_server_id = self.config_manager.snapshot().await.server.server_id;
                if let Some(server_id) = server_id.as_ref().filter(|id| *id != &local_server_id) {
                    if !self
                        .relay_manager
                        .move_remote_terminal_to_group(session_id, server_id, group_id)
                        .await?
                    {
                        self.send_error(
                            ErrorCode::InvalidMessage,
                            "Cannot move a terminal to a group on a different server",
                        )
                        .await;
                    }
                } else if self.relay_manager.is_remote_session(session_id).await {
                    self.send_error(
                        ErrorCode::InvalidMessage,
                        "Cannot move a terminal to a group on a different server",
                    )
                    .await;
                } else {
                    self.session_manager
                        .move_terminal_to_group(session_id, group_id)
                        .await?;
                }
            }
            Message::MoveSessionToTab {
                session_id,
                server_id,
                tab_id,
            } => {
                let local_server_id = self.config_manager.snapshot().await.server.server_id;
                // Resolve the intended server for a cross-server move: either an
                // explicit server_id, or the single shared server that owns both
                // the moving session and the target tab (same-server moves are
                // forwarded upstream — see requirement: tabs of the same shared
                // server can be merged into one tab).
                let explicit = server_id
                    .as_ref()
                    .filter(|id| *id != &local_server_id)
                    .cloned();
                let session_server = self
                    .relay_manager
                    .remote_server_for_session(session_id)
                    .await;
                let tab_server = self.relay_manager.remote_server_for_session(tab_id).await;
                let inferred = match (session_server, tab_server) {
                    (Some(session_owner), Some(tab_owner)) if session_owner == tab_owner => {
                        Some(session_owner)
                    }
                    _ => None,
                };

                if let Some(target_server) = explicit.or(inferred) {
                    if !self
                        .relay_manager
                        .move_remote_session_to_tab(&target_server, session_id, tab_id)
                        .await?
                    {
                        self.send_error(
                            ErrorCode::InvalidMessage,
                            "Cannot move a session to a tab on a different server",
                        )
                        .await;
                    }
                } else if self.relay_manager.is_remote_session(session_id).await
                    || self.relay_manager.is_remote_session(tab_id).await
                {
                    self.send_error(
                        ErrorCode::InvalidMessage,
                        "Cannot move a session to a tab on a different server",
                    )
                    .await;
                } else {
                    self.session_manager
                        .move_session_to_tab(session_id, tab_id)
                        .await?;
                }
            }
            Message::ListTerminalProfiles => {
                let profiles = self.relay_manager.list_terminal_profiles().await;
                self.send_message(Message::TerminalProfileList { profiles })
                    .await?;
            }
            Message::RemoveRecentProjectDirectory { path, server_id: _ } => {
                let local_server_id = self.config_manager.snapshot().await.server.server_id;
                let directories = self.session_manager.remove_recent_project_directory(&path);
                self.send_message(Message::RecentProjectDirectoryList {
                    directories,
                    server_id: Some(local_server_id),
                })
                .await?;
            }
            Message::StreamStart {
                session_id,
                initial_size,
                background,
                foreground,
            } => {
                info!(
                    "{:?} connection {} started stream for session {}",
                    self.connection_kind, self.addr, session_id
                );
                if self.connection_kind == ConnectionKind::Debug {
                    self.streamed_session_ids.insert(session_id);
                    return Ok(());
                }
                if background.is_some() || foreground.is_some() {
                    self.session_manager
                        .set_terminal_theme_colors(session_id, background, foreground)
                        .await?;
                }
                self.start_session_stream(session_id, initial_size).await?;
            }
            Message::StreamEnd { session_id } => {
                info!(
                    "{:?} connection {} ended stream for session {}",
                    self.connection_kind, self.addr, session_id
                );
                if self.connection_kind == ConnectionKind::Debug {
                    self.streamed_session_ids.remove(&session_id);
                    return Ok(());
                }
                if self.streamed_session_ids.remove(&session_id) {
                    self.session_manager
                        .remove_viewer(session_id, &self.tx)
                        .await?;
                    // The watcher owner for this session is this terminal
                    // connection (SetFsWatchers now rides on it), so ending
                    // the stream also ends the watches — unless a newer
                    // connection has already taken over ownership.
                    self.fs_watcher_manager
                        .remove_session_if_owner(session_id, &self.tx);
                    self.fs_watched_session_ids.remove(&session_id);
                    info!(
                        "{:?} connection {} removed terminal viewer for session {}",
                        self.connection_kind, self.addr, session_id
                    );
                } else {
                    info!(
                        "{:?} connection {} stream end ignored for inactive stream session {}",
                        self.connection_kind, self.addr, session_id
                    );
                }
            }
            other => {
                warn!("Unhandled session message: {}", other.msg_name());
            }
        }

        Ok(())
    }

    pub(crate) async fn start_session_stream(
        &mut self,
        session_id: SessionId,
        initial_size: Option<TerminalSize>,
    ) -> anyhow::Result<()> {
        if self.streamed_session_ids.contains(&session_id) {
            for buffer_info in self
                .session_manager
                .session_initial_buffer_infos(session_id)
                .await?
            {
                self.send_message(buffer_info).await?;
            }
            return Ok(());
        }

        match self
            .session_manager
            .add_viewer(session_id, self.tx.clone(), initial_size)
            .await
        {
            Ok(messages) => {
                self.streamed_session_ids.insert(session_id);
                info!(
                    "{:?} connection {} added local terminal viewer for session {} initial_messages={}",
                    self.connection_kind,
                    self.addr,
                    session_id,
                    messages.len()
                );
                for message in messages {
                    self.send_message(message).await?;
                }
                // Quick commands are state pushed by the server.  Send the
                // current list once this terminal connection is a viewer;
                // subsequent CWD and commands.json changes are broadcast by
                // FsWatcherManager to every viewer of the session.
                self.send_message(
                    crate::debug::command_list(&self.session_manager, session_id).await,
                )
                .await?;
                Ok(())
            }
            Err(_) => {
                self.send_error(ErrorCode::UnknownSession, "Session not found")
                    .await;
                Ok(())
            }
        }
    }
}
