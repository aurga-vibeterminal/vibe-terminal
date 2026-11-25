//! Git integration: changed files, diff, log, branches, stash, commit,
//! fetch/pull/push, checkout/merge/rebase (including AI-assisted conflict
//! resolution), and worktree creation.

use tracing::{info, warn};
use vterm_common::ErrorCode;
use vterm_core::protocol::Message;

use crate::connection::Connection;
use crate::session::SessionError;

impl Connection {
    pub(crate) async fn handle_git_message(&mut self, message: Message) -> anyhow::Result<()> {
        match message {
            Message::GitChangedFiles { session_id, commit } => {
                info!(
                    %session_id,
                    ?commit,
                    "[git] server received changed-files request"
                );
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .git_changed_files(session_id, commit)
                    .await
                {
                    Ok(message) => self.send_message(message).await?,
                    Err(SessionError::FileSystemError(msg)) if msg == "Not a git repository" => {
                        self.send_error(ErrorCode::InvalidMessage, &msg).await;
                    }
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitLog {
                session_id,
                path,
                commit,
                branch,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .git_log(session_id, path, commit, branch)
                    .await
                {
                    Ok(message) => self.send_message(message).await?,
                    Err(SessionError::FileSystemError(msg)) if msg == "Not a git repository" => {
                        self.send_error(ErrorCode::InvalidMessage, &msg).await;
                    }
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitDiff {
                session_id,
                path,
                commit,
                cached,
                compare_with_worktree,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .git_diff(session_id, path, commit, cached, compare_with_worktree)
                    .await
                {
                    Ok(message) => self.send_message(message).await?,
                    Err(SessionError::FileSystemError(msg)) if msg == "Not a git repository" => {
                        self.send_error(ErrorCode::InvalidMessage, &msg).await;
                    }
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitStage {
                session_id,
                paths,
                repo_root,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .git_stage(session_id, paths, repo_root)
                    .await
                {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitUnstage {
                session_id,
                paths,
                repo_root,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .git_unstage(session_id, paths, repo_root)
                    .await
                {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitDiscard {
                session_id,
                paths,
                repo_root,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .git_discard(session_id, paths, repo_root)
                    .await
                {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitRevert {
                session_id,
                path,
                commit,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .git_revert(session_id, path, commit)
                    .await
                {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitCommit {
                session_id,
                message,
                expected_detached_head,
                target_branch,
                create_branch,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .git_commit(
                        session_id,
                        message,
                        expected_detached_head,
                        target_branch,
                        create_branch,
                    )
                    .await
                {
                    Ok(msg) => self.send_message(msg).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitFetch { session_id } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self.session_manager.git_fetch(session_id).await {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitPull { session_id } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self.session_manager.git_pull(session_id).await {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitPush {
                session_id,
                push_all,
                force,
                force_with_lease,
                set_upstream,
                tags,
                dry_run,
                remote,
                branch,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .git_push(
                        session_id,
                        push_all,
                        force,
                        force_with_lease,
                        set_upstream,
                        tags,
                        dry_run,
                        remote,
                        branch,
                    )
                    .await
                {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitGenerateCommitMessageAgent { session_id } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                let session_manager = self.session_manager.clone();
                let addon_host = self.addon_host.clone();
                let tx = self.tx.clone();
                tokio::spawn(async move {
                    let result = session_manager
                        .git_generate_commit_message_agent(session_id, addon_host)
                        .await;
                    // Correlated response: send it back over the request
                    // connection — for a relayed shared server that is the
                    // only route back to the requesting client. If that
                    // connection is already gone (e.g. the client
                    // disconnected and reconnected while the agent was
                    // running), fall back to the session's local viewers so
                    // a reconnected local client still receives the result
                    // instead of losing it silently.
                    if tx.send(result.clone()).is_err() {
                        crate::fs_watcher::send_message_to_session_viewers(
                            &session_manager,
                            session_id,
                            result,
                        )
                        .await;
                    }
                });
                self.send_message(Message::GitGenerateCommitMessageAccepted { session_id })
                    .await?;
            }
            Message::GitAnalyzeDiff {
                session_id,
                path,
                commit,
                cached,
                compare_with_worktree,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                self.session_manager
                    .spawn_git_analyze_diff(
                        session_id,
                        path,
                        commit,
                        cached,
                        compare_with_worktree,
                        self.addon_host.clone(),
                        self.tx.clone(),
                    )
                    .await?;
            }
            Message::GitBranches { session_id } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self.session_manager.git_branches(session_id).await {
                    Ok(message) => self.send_message(message).await?,
                    Err(SessionError::FileSystemError(msg)) if msg == "Not a git repository" => {
                        self.send_error(ErrorCode::InvalidMessage, &msg).await;
                    }
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitStashList { session_id } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self.session_manager.git_stash_list(session_id).await {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitStashPush {
                session_id,
                message,
                paths,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .git_stash_push(session_id, message, paths)
                    .await
                {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitStashApply { session_id, index } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .git_stash_apply(session_id, index)
                    .await
                {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitStashPop { session_id, index } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self.session_manager.git_stash_pop(session_id, index).await {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitStashDrop { session_id, index } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self.session_manager.git_stash_drop(session_id, index).await {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitSubmoduleList { session_id } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self.session_manager.git_submodule_list(session_id).await {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitCheckout {
                session_id,
                ref_,
                create_branch,
                branch_name,
                track,
                force,
                merge,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .git_checkout(
                        session_id,
                        ref_,
                        create_branch,
                        branch_name,
                        track,
                        force,
                        merge,
                    )
                    .await
                {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitCreateBranch {
                session_id,
                source_ref,
                branch,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .git_create_branch(session_id, source_ref, branch)
                    .await
                {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitDeleteBranch {
                session_id,
                branch,
                is_worktree,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .git_delete_branch(session_id, branch, is_worktree)
                    .await
                {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitMerge { session_id, branch } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self.session_manager.git_merge(session_id, branch).await {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitRebaseStart {
                session_id,
                target_branch,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .git_rebase_start(session_id, target_branch)
                    .await
                {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitRebaseApply {
                session_id,
                target_branch,
                commits,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .git_rebase_apply(session_id, target_branch, commits)
                    .await
                {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitRebaseConflictContent { session_id, path } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .git_rebase_conflict_content(session_id, path)
                    .await
                {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitRebaseResolve {
                session_id,
                path,
                content,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .git_rebase_resolve(session_id, path, content)
                    .await
                {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitRebaseContinue { session_id } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self.session_manager.git_rebase_continue(session_id).await {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitRebaseSkip { session_id } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self.session_manager.git_rebase_skip(session_id).await {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitRebaseAbort { session_id } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self.session_manager.git_rebase_abort(session_id).await {
                    Ok(message) => self.send_message(message).await?,
                    Err(error) => return Err(error.into()),
                }
            }
            Message::GitRebaseAiResolve {
                session_id,
                run_id,
                instruction,
                ours_label,
                theirs_label,
                targets,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                self.session_manager
                    .spawn_git_rebase_ai_resolve(
                        session_id,
                        run_id,
                        instruction,
                        ours_label,
                        theirs_label,
                        targets,
                        self.addon_host.clone(),
                        self.tx.clone(),
                    )
                    .await?;
            }
            Message::GitRebaseAiResolveCancel { session_id } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                self.session_manager
                    .cancel_git_rebase_ai_resolve(session_id)
                    .await;
            }
            Message::GitCreateWorktree {
                session_id,
                branch,
                name,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                match self
                    .session_manager
                    .git_create_worktree(session_id, branch, name)
                    .await
                {
                    Ok(message) => {
                        // Send the result to the requesting connection.
                        self.send_message(message.clone()).await?;
                        // If a new session was created, also broadcast SessionCreated
                        // so other connections (e.g. primary) can track it.
                        if let Message::GitCreateWorktreeResult {
                            new_session_id: Some(new_sid),
                            ..
                        } = &message
                        {
                            info!("Worktree session created: {} by {}", new_sid, self.addr);
                        }
                    }
                    Err(error) => {
                        warn!("GitCreateWorktree failed: {:#}", error);
                        self.send_message(Message::GitCreateWorktreeResult {
                            session_id,
                            worktree_path: None,
                            new_session_id: None,
                            error: Some(error.to_string()),
                        })
                        .await?;
                    }
                }
            }
            other => {
                warn!("Unhandled git message: {}", other.msg_name());
            }
        }

        Ok(())
    }
}
