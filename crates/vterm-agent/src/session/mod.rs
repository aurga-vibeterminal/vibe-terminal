use std::collections::HashMap;

use crate::client::ChatMessage;

pub type SessionId = String;

#[derive(Debug, Clone)]
pub struct AgentSession {
    pub id: SessionId,
    pub messages: Vec<ChatMessage>,
    pub total_tokens_used: i64,
    pub iteration_count: usize,
}

impl AgentSession {
    pub fn new(system_prompt: impl Into<String>, user_message: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            messages: vec![
                ChatMessage::system(system_prompt),
                ChatMessage::user(user_message),
            ],
            total_tokens_used: 0,
            iteration_count: 0,
        }
    }

    pub fn token_estimate(&self) -> i64 {
        self.messages.iter().map(|m| m.token_estimate()).sum()
    }

    pub fn append(&mut self, message: ChatMessage) {
        self.messages.push(message);
    }

    pub fn append_many(&mut self, messages: Vec<ChatMessage>) {
        self.messages.extend(messages);
    }
}

pub struct SessionManager {
    sessions: HashMap<SessionId, AgentSession>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn create(
        &mut self,
        system_prompt: impl Into<String>,
        user_message: impl Into<String>,
    ) -> SessionId {
        let session = AgentSession::new(system_prompt, user_message);
        let id = session.id.clone();
        self.sessions.insert(id.clone(), session);
        id
    }

    pub fn get(&self, id: &str) -> Option<&AgentSession> {
        self.sessions.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut AgentSession> {
        self.sessions.get_mut(id)
    }

    pub fn remove(&mut self, id: &str) -> Option<AgentSession> {
        self.sessions.remove(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get() {
        let mut manager = SessionManager::new();
        let id = manager.create("You are a helper.", "Hello");
        assert!(manager.get(&id).is_some());
        assert!(manager.get("nonexistent").is_none());
    }

    #[test]
    fn test_session_messages() {
        let mut manager = SessionManager::new();
        let id = manager.create("system prompt", "user message");
        let session = manager.get(&id).unwrap();
        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.messages[0].role, "system");
        assert_eq!(session.messages[1].role, "user");
    }

    #[test]
    fn test_append_messages() {
        let mut manager = SessionManager::new();
        let id = manager.create("system", "user");
        {
            let session = manager.get_mut(&id).unwrap();
            session.append(ChatMessage::assistant("reply"));
        }
        let session = manager.get(&id).unwrap();
        assert_eq!(session.messages.len(), 3);
        assert_eq!(session.messages[2].role, "assistant");
    }

    #[test]
    fn test_token_estimate() {
        let session = AgentSession::new("short", "medium length message here");
        let estimate = session.token_estimate();
        assert!(estimate > 0);
    }

    #[test]
    fn test_remove() {
        let mut manager = SessionManager::new();
        let id = manager.create("s", "u");
        assert!(manager.remove(&id).is_some());
        assert!(manager.get(&id).is_none());
    }
}
