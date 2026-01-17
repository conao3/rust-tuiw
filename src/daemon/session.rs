use crate::tmux::wrapper::TmuxWrapper;
use crate::types::{Session, SessionId, SessionStatus};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[allow(dead_code)]
#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    tmux: Arc<TmuxWrapper>,
}

impl SessionManager {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            tmux: Arc::new(TmuxWrapper::new()),
        }
    }

    #[allow(dead_code)]
    pub async fn create_session(&self, command: String, cwd: String) -> Result<SessionId> {
        let session_id = SessionId::new();
        let tmux_session = format!("tuiw-{}", session_id.0);

        self.tmux
            .create_session(&tmux_session, &command, &cwd)
            .await?;

        let session = Session {
            id: session_id.clone(),
            command,
            cwd,
            tmux_session,
        };

        self.sessions
            .write()
            .await
            .insert(session_id.clone(), session);
        Ok(session_id)
    }

    #[allow(dead_code)]
    pub async fn get_session(&self, id: &SessionId) -> Option<Session> {
        self.sessions.read().await.get(id).cloned()
    }

    #[allow(dead_code)]
    pub async fn list_sessions(&self) -> Vec<Session> {
        self.sessions.read().await.values().cloned().collect()
    }

    #[allow(dead_code)]
    pub async fn remove_session(&self, id: &SessionId) -> Result<Option<Session>> {
        let session = self.sessions.write().await.remove(id);

        if let Some(ref sess) = session {
            if self.tmux.session_exists(&sess.tmux_session).await? {
                self.tmux.kill_session(&sess.tmux_session).await?;
            }
        }

        Ok(session)
    }

    #[allow(dead_code)]
    pub async fn send_keys(&self, id: &SessionId, keys: String) -> Result<()> {
        let session = self
            .sessions
            .read()
            .await
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("session not found"))?;

        self.tmux.send_keys(&session.tmux_session, &keys).await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_output(&self, id: &SessionId) -> Result<String> {
        let session = self
            .sessions
            .read()
            .await
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("session not found"))?;

        let output = self.tmux.capture_pane(&session.tmux_session).await?;
        Ok(output)
    }

    #[allow(dead_code)]
    pub async fn get_session_status(&self, id: &SessionId) -> Result<SessionStatus> {
        let session = self
            .sessions
            .read()
            .await
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("session not found"))?;

        let exists = self.tmux.session_exists(&session.tmux_session).await?;

        if exists {
            Ok(SessionStatus::Running)
        } else {
            Ok(SessionStatus::Stopped)
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
