use crate::types::{Session, SessionId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_session(&self, command: String, cwd: String) -> SessionId {
        let session_id = SessionId::new();
        let tmux_session = format!("tuiw-{}", session_id.0);

        let session = Session {
            id: session_id.clone(),
            command,
            cwd,
            tmux_session,
        };

        self.sessions.write().await.insert(session_id.clone(), session);
        session_id
    }

    pub async fn get_session(&self, id: &SessionId) -> Option<Session> {
        self.sessions.read().await.get(id).cloned()
    }

    pub async fn list_sessions(&self) -> Vec<Session> {
        self.sessions.read().await.values().cloned().collect()
    }

    pub async fn remove_session(&self, id: &SessionId) -> Option<Session> {
        self.sessions.write().await.remove(id)
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
