use async_graphql::{Enum, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct SessionId(pub String);

impl SessionId {
    pub fn new() -> Self {
        let uuid = Uuid::new_v4();
        let short_id = uuid.to_string().chars().take(8).collect();
        Self(short_id)
    }

    pub fn from_str(s: &str) -> Result<Self, &'static str> {
        if s.len() != 8 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("session ID must be 8 hexadecimal characters");
        }
        Ok(Self(s.to_string()))
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

async_graphql::scalar!(SessionId);

impl From<SessionId> for async_graphql::Value {
    fn from(id: SessionId) -> Self {
        async_graphql::Value::String(id.0)
    }
}

impl TryFrom<async_graphql::Value> for SessionId {
    type Error = &'static str;

    fn try_from(value: async_graphql::Value) -> Result<Self, Self::Error> {
        match value {
            async_graphql::Value::String(s) => SessionId::from_str(&s),
            _ => Err("expected string"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct Session {
    pub id: SessionId,
    pub command: String,
    pub cwd: String,
    pub tmux_session: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Enum, Copy, Eq, PartialEq)]
pub enum SessionStatus {
    Running,
    Stopped,
    Error,
}

#[derive(Debug, InputObject)]
pub struct CreateSessionInput {
    pub command: String,
    pub cwd: String,
}

#[derive(Debug, InputObject)]
pub struct SendKeysInput {
    pub session_id: SessionId,
    pub keys: String,
}
