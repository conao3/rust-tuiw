use async_graphql::{Enum, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct SessionId(pub Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

async_graphql::scalar!(SessionId);

impl From<SessionId> for async_graphql::Value {
    fn from(id: SessionId) -> Self {
        async_graphql::Value::String(id.0.to_string())
    }
}

impl TryFrom<async_graphql::Value> for SessionId {
    type Error = &'static str;

    fn try_from(value: async_graphql::Value) -> Result<Self, Self::Error> {
        match value {
            async_graphql::Value::String(s) => {
                let uuid = Uuid::parse_str(&s).map_err(|_| "invalid UUID")?;
                Ok(SessionId(uuid))
            }
            _ => Err("expected string"),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct Session {
    pub id: SessionId,
    pub command: String,
    pub cwd: String,
    pub tmux_session: String,
}

#[allow(dead_code)]
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
