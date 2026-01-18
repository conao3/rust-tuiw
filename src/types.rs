use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct SessionId(pub String);

impl SessionId {
    pub fn new() -> Self {
        let uuid = Uuid::new_v4();
        let short_id = uuid.to_string().chars().take(8).collect();
        Self(short_id)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub command: String,
    pub cwd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionStore {
    pub sessions: HashMap<String, Session>,
}
