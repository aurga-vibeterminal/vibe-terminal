//! Browsing and watching AI chat conversations discovered in a session's
//! working directory.

use tracing::warn;
use vterm_core::protocol::Message;

use crate::connection::Connection;

impl Connection {
    pub(crate) async fn handle_ai_conversation_message(
        &mut self,
        message: Message,
    ) -> anyhow::Result<()> {
        match message {
            Message::ListAiConversations { session_id } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                let conversations = self.session_manager.list_ai_conversations(session_id).await;
                self.send_message(Message::AiConversationList {
                    session_id,
                    conversations,
                })
                .await?;
            }
            Message::StopWatchingAiConversation {
                session_id,
                conversation_id,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                self.session_manager
                    .stop_watching_ai_conversation(session_id, &conversation_id)
                    .await;
            }
            Message::ReadAiConversation {
                session_id,
                conversation_id,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                let messages = self
                    .session_manager
                    .read_ai_conversation(session_id, conversation_id.clone())
                    .await;
                self.send_message(Message::AiConversationMessages {
                    session_id,
                    conversation_id,
                    messages,
                })
                .await?;
            }
            other => {
                warn!("Unhandled AI conversation message: {}", other.msg_name());
            }
        }

        Ok(())
    }
}
