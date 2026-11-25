//! Compact, allocation-light request message summaries for diagnostics.

use vterm_core::protocol::Message;

#[allow(dead_code)]
fn request_log_summary(message: &Message) -> String {
    match message {
        Message::CreateSession { cwd, .. } => {
            format!("CreateSession cwd={}", cwd.as_deref().unwrap_or("<none>"))
        }
        Message::SessionCreated { session_id } => format!("SessionCreated session={session_id}"),
        Message::SplitTerminal {
            target_session_id,
            placement,
            ..
        } => format!("SplitTerminal target={target_session_id} placement={placement:?}"),
        Message::CloseSession { session_id } => format!("CloseSession session={session_id}"),
        Message::CloseTab { tab_id } => format!("CloseTab tab={tab_id}"),
        Message::ListSessions => "ListSessions".to_string(),
        Message::SessionList { sessions } => format!("SessionList count={}", sessions.len()),
        Message::ListTabGroups => "ListTabGroups".to_string(),
        Message::TabGroupList { groups } => {
            format!("TabGroupList count={}", groups.len())
        }
        Message::GetTabInfoList { server_id } => {
            format!("GetTabInfoList server={server_id:?}")
        }
        Message::TabInfoList { server_id, tabs } => {
            format!("TabInfoList server={server_id:?} count={}", tabs.len())
        }
        Message::UpdateTerminalLayout { tab_id, .. } => {
            format!("UpdateTerminalLayout tab={tab_id}")
        }
        Message::CreateTabGroup { name, .. } => format!("CreateTabGroup name={name}"),
        Message::DeleteTabGroup { group_id, .. } => {
            format!("DeleteTabGroup group={group_id}")
        }
        Message::RenameTabGroup { group_id, name, .. } => {
            format!("RenameTabGroup group={group_id} name={name}")
        }
        Message::MoveSessionToTab {
            session_id, tab_id, ..
        } => {
            format!("MoveSessionToTab session={session_id} tab={tab_id}")
        }
        Message::MoveTabToGroup {
            session_id,
            group_id,
            ..
        } => format!("MoveTabToGroup session={session_id} group={group_id}"),
        Message::ListTerminalProfiles => "ListTerminalProfiles".to_string(),
        Message::TerminalProfileList { profiles } => {
            format!("TerminalProfileList count={}", profiles.len())
        }
        Message::ListPathContent { session_id, path } => {
            format!("ListPathContent session={session_id} path={path}")
        }
        Message::PathContentList {
            session_id,
            path,
            items,
            ..
        } => {
            format!(
                "PathContentList session={session_id} path={path} count={}",
                items.len()
            )
        }
        Message::FindFiles {
            session_id,
            request_id,
            query,
            max_results,
            ..
        } => {
            format!(
                "FindFiles session={session_id} request_id={request_id} query_bytes={} max_results={max_results}",
                query.len(),
            )
        }
        Message::FindFilesResult {
            session_id,
            request_id,
            query,
            chunk_index,
            is_final,
            results,
            error,
            ..
        } => {
            format!(
                "FindFilesResult session={session_id} request_id={request_id} query_bytes={} chunk={} final={} files={} error={}",
                query.len(),
                chunk_index,
                is_final,
                results.len(),
                error.is_some()
            )
        }
        Message::SearchPathNamesResult {
            session_id,
            request_id,
            query,
            is_final,
            results,
            error,
            ..
        } => {
            // Keep this summary lightweight — the full `{:?}` of the results
            // list (the catch-all below) can stall the request-connection loop
            // on large result sets and trip client timeouts.
            format!(
                "SearchPathNamesResult session={session_id} request_id={request_id} query_bytes={} final={} matches={} error={}",
                query.len(),
                is_final,
                results.len(),
                error.is_some()
            )
        }
        Message::RemoveRecentProjectDirectory { path, .. } => {
            format!("RemoveRecentProjectDirectory path={path}")
        }
        Message::RecentProjectDirectoryList { directories, .. } => {
            format!("RecentProjectDirectoryList count={}", directories.len())
        }
        Message::KnownAppList { apps, .. } => format!("KnownAppList count={}", apps.len()),
        Message::ListAiConversations { session_id } => {
            format!("ListAiConversations session={session_id}")
        }
        Message::StopWatchingAiConversation {
            session_id,
            conversation_id,
        } => {
            format!(
                "StopWatchingAiConversation session={session_id} conversation={conversation_id}"
            )
        }
        Message::AiConversationList {
            session_id,
            conversations,
        } => {
            format!(
                "AiConversationList session={session_id} count={}",
                conversations.len()
            )
        }
        Message::ReadAiConversation {
            session_id,
            conversation_id,
        } => {
            format!("ReadAiConversation session={session_id} conversation={conversation_id}")
        }
        Message::AiConversationMessages {
            session_id,
            conversation_id,
            messages,
        } => {
            format!(
                "AiConversationMessages session={session_id} conversation={conversation_id} count={}",
                messages.len()
            )
        }
        Message::TranslateMessage {
            session_id,
            message_id,
            ..
        } => {
            format!("TranslateMessage session={session_id} message_id={message_id}")
        }
        Message::TranslateMessageResult {
            session_id,
            message_id,
            translation,
            error,
            ..
        } => {
            format!(
                "TranslateMessageResult session={session_id} message_id={message_id} ok={} err={:?}",
                translation.is_some(),
                error
            )
        }
        Message::GitChangedFiles { session_id, commit } => {
            format!(
                "GitChangedFiles session={session_id} commit={}",
                commit.as_deref().unwrap_or("<worktree>")
            )
        }
        Message::GitChangedFilesList {
            session_id,
            commit,
            items,
            ..
        } => {
            format!(
                "GitChangedFilesList session={session_id} commit={} count={}",
                commit.as_deref().unwrap_or("<worktree>"),
                items.len()
            )
        }
        Message::GitLog {
            session_id,
            path,
            commit,
            branch,
        } => {
            format!(
                "GitLog session={session_id} path={} commit={} branch={}",
                path.as_deref().unwrap_or("<all>"),
                commit.as_deref().unwrap_or("HEAD"),
                branch.as_deref().unwrap_or("<default>")
            )
        }
        Message::GitLogList {
            session_id,
            commit,
            branch,
            commits,
            ..
        } => {
            format!(
                "GitLogList session={session_id} commit={} branch={} count={}",
                commit.as_deref().unwrap_or("HEAD"),
                branch.as_deref().unwrap_or("<default>"),
                commits.len()
            )
        }
        Message::GitDiff {
            session_id,
            path,
            commit,
            cached,
            compare_with_worktree,
            ..
        } => {
            format!(
                "GitDiff session={session_id} path={path} commit={} cached={cached} compare_with_worktree={compare_with_worktree}",
                commit.as_deref().unwrap_or("<worktree>")
            )
        }
        Message::GitDiffResult {
            session_id,
            path,
            diff,
        } => {
            format!(
                "GitDiffResult session={session_id} path={path} bytes={}",
                diff.len()
            )
        }
        Message::GitStage {
            session_id, paths, ..
        } => {
            format!("GitStage session={session_id} paths={}", paths.len())
        }
        Message::GitUnstage {
            session_id, paths, ..
        } => {
            format!("GitUnstage session={session_id} paths={}", paths.len())
        }
        Message::GitDiscard {
            session_id, paths, ..
        } => {
            format!("GitDiscard session={session_id} paths={}", paths.len())
        }
        Message::GitCommit {
            session_id,
            message,
            ..
        } => {
            format!(
                "GitCommit session={session_id} message_bytes={}",
                message.len()
            )
        }
        Message::GitFetch { session_id } => format!("GitFetch session={session_id}"),
        Message::GitPull { session_id } => format!("GitPull session={session_id}"),
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
            format!(
                "GitPush session={session_id} all={push_all} force={force} lease={force_with_lease} upstream={set_upstream} tags={tags} dry_run={dry_run} remote={remote} branch={branch}"
            )
        }
        Message::GitAnalyzeDiff {
            session_id,
            path,
            commit,
            cached,
            compare_with_worktree,
        } => {
            format!(
                "GitAnalyzeDiff session={session_id} path={path} commit={} cached={cached} compare_with_worktree={compare_with_worktree}",
                commit.as_deref().unwrap_or("none")
            )
        }
        Message::GitStageResult { session_id, error } => {
            format!(
                "GitStageResult session={session_id} error={}",
                error.as_deref().unwrap_or("none")
            )
        }
        Message::GitUnstageResult { session_id, error } => {
            format!(
                "GitUnstageResult session={session_id} error={}",
                error.is_some()
            )
        }
        Message::GitDiscardResult { session_id, error } => {
            format!(
                "GitDiscardResult session={session_id} error={}",
                error.is_some()
            )
        }
        Message::GitCommitResult { session_id, error } => {
            format!(
                "GitCommitResult session={session_id} error={}",
                error.is_some()
            )
        }
        Message::GitFetchResult { session_id, error } => {
            format!(
                "GitFetchResult session={session_id} error={}",
                error.is_some()
            )
        }
        Message::GitPullResult { session_id, error } => {
            format!(
                "GitPullResult session={session_id} error={}",
                error.is_some()
            )
        }
        Message::GitPushResult { session_id, error } => {
            format!(
                "GitPushResult session={session_id} error={}",
                error.is_some()
            )
        }
        Message::GitGenerateCommitMessageResult {
            session_id,
            message,
            error,
        } => {
            format!(
                "GitGenerateCommitMessageResult session={session_id} message={} error={}",
                message.is_some(),
                error.is_some()
            )
        }
        Message::GitAnalyzeDiffResult {
            session_id,
            hunks,
            error,
            done,
        } => {
            format!(
                "GitAnalyzeDiffResult session={session_id} hunks={} error={} done={}",
                hunks.len(),
                error.is_some(),
                done
            )
        }
        Message::GitBranches { session_id } => format!("GitBranches session={session_id}"),
        Message::GitBranchesResult {
            session_id,
            branches,
            ..
        } => {
            format!(
                "GitBranchesResult session={session_id} count={}",
                branches.len()
            )
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
            format!(
                "GitCheckout session={session_id} ref={ref_} create_branch={create_branch} branch_name={branch_name} track={track} force={force} merge={merge}"
            )
        }
        Message::GitCheckoutResult { session_id, error } => {
            format!(
                "GitCheckoutResult session={session_id} error={}",
                error.is_some()
            )
        }
        Message::GitCreateWorktree {
            session_id,
            branch,
            name,
        } => {
            format!("GitCreateWorktree session={session_id} branch={branch} name={name}")
        }
        Message::GitCreateBranch {
            session_id,
            source_ref,
            branch,
        } => {
            format!("GitCreateBranch session={session_id} source_ref={source_ref} branch={branch}")
        }
        Message::GitCreateBranchResult { session_id, error } => {
            format!(
                "GitCreateBranchResult session={session_id} error={}",
                error.is_some()
            )
        }
        Message::GitCreateWorktreeResult {
            session_id,
            worktree_path,
            new_session_id,
            error,
        } => {
            format!(
                "GitCreateWorktreeResult session={session_id} path={} new_session={} error={}",
                worktree_path.as_deref().unwrap_or("<none>"),
                new_session_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "<none>".to_string()),
                error.is_some()
            )
        }
        Message::GitDeleteBranch {
            session_id,
            branch,
            is_worktree,
        } => {
            format!(
                "GitDeleteBranch session={session_id} branch={branch} is_worktree={is_worktree}"
            )
        }
        Message::GitDeleteBranchResult { session_id, error } => {
            format!(
                "GitDeleteBranchResult session={session_id} error={}",
                error.is_some()
            )
        }
        Message::GitMerge { session_id, branch } => {
            format!("GitMerge session={session_id} branch={branch}")
        }
        Message::GitMergeResult { session_id, error } => {
            format!(
                "GitMergeResult session={session_id} error={}",
                error.is_some()
            )
        }
        Message::GitRebaseStart {
            session_id,
            target_branch,
        } => {
            format!("GitRebaseStart session={session_id} target={target_branch}")
        }
        Message::GitRebaseStartResult {
            session_id, error, ..
        } => {
            format!(
                "GitRebaseStartResult session={session_id} error={}",
                error.is_some()
            )
        }
        Message::GitRebaseApply {
            session_id,
            target_branch,
            commits,
        } => {
            format!(
                "GitRebaseApply session={session_id} target={target_branch} commits={}",
                commits.len()
            )
        }
        Message::GitRebaseApplyResult {
            session_id, error, ..
        } => {
            format!(
                "GitRebaseApplyResult session={session_id} error={}",
                error.is_some()
            )
        }
        Message::GitRebaseConflictContent { session_id, path } => {
            format!("GitRebaseConflictContent session={session_id} path={path}")
        }
        Message::GitRebaseConflictContentResult {
            session_id,
            path,
            error,
            ..
        } => {
            format!(
                "GitRebaseConflictContentResult session={session_id} path={path} error={}",
                error.is_some()
            )
        }
        Message::GitRebaseResolve {
            session_id, path, ..
        } => {
            format!("GitRebaseResolve session={session_id} path={path}")
        }
        Message::GitRebaseResolveResult {
            session_id,
            path,
            error,
        } => {
            format!(
                "GitRebaseResolveResult session={session_id} path={path} error={}",
                error.is_some()
            )
        }
        Message::GitRebaseContinue { session_id } => {
            format!("GitRebaseContinue session={session_id}")
        }
        Message::GitRebaseContinueResult {
            session_id, error, ..
        } => {
            format!(
                "GitRebaseContinueResult session={session_id} error={}",
                error.is_some()
            )
        }
        Message::GitRebaseSkip { session_id } => {
            format!("GitRebaseSkip session={session_id}")
        }
        Message::GitRebaseSkipResult {
            session_id, error, ..
        } => {
            format!(
                "GitRebaseSkipResult session={session_id} error={}",
                error.is_some()
            )
        }
        Message::GitRebaseAbort { session_id } => {
            format!("GitRebaseAbort session={session_id}")
        }
        Message::GitRebaseAbortResult { session_id, error } => {
            format!(
                "GitRebaseAbortResult session={session_id} error={}",
                error.is_some()
            )
        }
        Message::GitRebaseAiResolve {
            session_id,
            targets,
            ..
        } => {
            let hunks: usize = targets.iter().map(|target| target.hunks.len()).sum();
            format!(
                "GitRebaseAiResolve session={session_id} files={} hunks={hunks}",
                targets.len()
            )
        }
        Message::GitRebaseAiResolveCancel { session_id } => {
            format!("GitRebaseAiResolveCancel session={session_id}")
        }
        Message::GitRebaseAiResolveResult {
            session_id,
            hunk,
            error,
            done,
            ..
        } => {
            format!(
                "GitRebaseAiResolveResult session={session_id} hunk={} error={} done={done}",
                hunk.is_some(),
                error.is_some()
            )
        }
        Message::FetchFile {
            session_id,
            path,
            max_size,
        } => {
            format!("FetchFile session={session_id} path={path} max_size={max_size}")
        }
        Message::FileContent {
            session_id,
            path,
            data,
            diff_ranges: _,
            error,
        } => {
            format!(
                "FileContent session={session_id} path={path} bytes={} error={}",
                data.len(),
                error.is_some()
            )
        }
        Message::SaveFile {
            session_id,
            path,
            data,
        } => {
            format!(
                "SaveFile session={session_id} path={path} bytes={}",
                data.len()
            )
        }
        Message::SaveFileResult {
            session_id,
            path,
            error,
        } => {
            format!(
                "SaveFileResult session={session_id} path={path} error={}",
                error.is_some()
            )
        }
        Message::CreatePath {
            session_id,
            path,
            is_dir,
        } => {
            format!("CreatePath session={session_id} path={path} is_dir={is_dir}")
        }
        Message::CreatePathResult {
            session_id,
            path,
            error,
        } => {
            format!(
                "CreatePathResult session={session_id} path={path} error={}",
                error.is_some()
            )
        }
        Message::DeletePaths { session_id, paths } => {
            format!("DeletePaths session={session_id} count={}", paths.len())
        }
        Message::DeletePathsResult { session_id, error } => {
            format!(
                "DeletePathsResult session={session_id} error={}",
                error.is_some()
            )
        }
        Message::PastePaths {
            session_id,
            src_paths,
            dst_dir,
            is_cut,
        } => {
            format!(
                "PastePaths session={session_id} count={} dst={dst_dir} is_cut={is_cut}",
                src_paths.len()
            )
        }
        Message::PastePathsResult { session_id, error } => {
            format!(
                "PastePathsResult session={session_id} error={}",
                error.is_some()
            )
        }
        Message::RenamePath {
            session_id,
            old_path,
            new_path,
        } => {
            format!("RenamePath session={session_id} old={old_path} new={new_path}")
        }
        Message::RenamePathResult { session_id, error } => {
            format!(
                "RenamePathResult session={session_id} error={}",
                error.is_some()
            )
        }
        Message::GenerateVtermConfig {
            session_id,
            vterm_path,
            config_type,
        } => format!(
            "GenerateVtermConfig session={session_id:?} path={vterm_path} type={config_type}"
        ),
        Message::GenerateVtermConfigResult {
            session_id,
            vterm_path,
            config_type,
            error,
        } => format!(
            "GenerateVtermConfigResult session={session_id:?} path={vterm_path} type={config_type} error={}",
            error.is_some()
        ),
        Message::StartStaticHttpServer {
            session_id,
            path,
            request_port_forward,
        } => format!(
            "StartStaticHttpServer session={session_id} path={path} request_port_forward={request_port_forward}"
        ),
        Message::StartStaticHttpServerResult {
            session_id,
            path,
            port,
            error,
            ..
        } => format!(
            "StartStaticHttpServerResult session={session_id} path={path} port={port:?} error={}",
            error.is_some()
        ),
        Message::DebugLaunchList { session_id } => {
            format!("DebugLaunchList session={session_id}")
        }
        Message::DebugLaunchListResult {
            session_id,
            launches,
            error,
            ..
        } => {
            format!(
                "DebugLaunchListResult session={session_id} count={} error={}",
                launches.len(),
                error.as_deref().unwrap_or("none")
            )
        }
        Message::DebugStart {
            session_id,
            launch_name,
        } => {
            format!("DebugStart session={session_id} launch={launch_name}")
        }
        Message::DebugStartResult {
            session_id,
            launch_name,
            success,
            error,
        } => {
            format!(
                "DebugStartResult session={session_id} launch={launch_name} success={success} error={}",
                error.as_deref().unwrap_or("none")
            )
        }
        Message::DebugOutput {
            session_id,
            launch_name,
            stream,
            data,
        } => {
            format!(
                "DebugOutput session={session_id} launch={} stream={stream} bytes={}",
                launch_name.as_deref().unwrap_or("<none>"),
                data.len()
            )
        }
        Message::DebugDap {
            session_id,
            launch_name,
            message,
        } => {
            format!(
                "DebugDap session={session_id} launch={} message={}",
                launch_name.as_deref().unwrap_or("<none>"),
                message
            )
        }
        Message::QuickCommandList { session_id } => {
            format!("QuickCommandList session={session_id}")
        }
        Message::QuickCommandListResult {
            session_id,
            commands,
            error,
            ..
        } => format!(
            "QuickCommandListResult session={session_id} count={} error={}",
            commands.len(),
            error.as_deref().unwrap_or("none")
        ),
        Message::RunQuickCommand { session_id, name } => {
            format!("RunQuickCommand session={session_id} name={name}")
        }
        Message::SetupMcpApp {
            session_id,
            executable,
        } => {
            format!("SetupMcpApp session={session_id} executable={executable}")
        }
        Message::RunQuickCommandResult {
            session_id,
            name,
            success,
            error,
        } => format!(
            "RunQuickCommandResult session={session_id} name={name} success={success} error={}",
            error.as_deref().unwrap_or("none")
        ),
        Message::FetchHistoryRaw {
            session_id,
            before_offset,
            max_bytes,
        } => {
            format!(
                "FetchHistoryRaw session={session_id} before_offset={before_offset} max_bytes={max_bytes}"
            )
        }
        Message::ExportSessionRawHistory { session_id } => {
            format!("ExportSessionRawHistory session={session_id}")
        }
        Message::HistoryRawChunk {
            session_id,
            start_offset,
            end_offset,
            data,
        } => {
            format!(
                "HistoryRawChunk session={session_id} start_offset={start_offset} end_offset={end_offset} bytes={}",
                data.len()
            )
        }
        Message::Error { code, message } => {
            format!("Error code={code:?} message_bytes={}", message.len())
        }
        other => format!("{other:?}"),
    }
}
