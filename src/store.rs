use crate::tmux::wrapper;
use crate::types::{Session, SessionStore};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_store_path() -> Result<PathBuf> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
    let tuiw_dir = config_dir.join("tuiw");
    std::fs::create_dir_all(&tuiw_dir)?;
    Ok(tuiw_dir.join("session.json"))
}

pub fn load_store() -> Result<SessionStore> {
    let path = get_store_path()?;
    if !path.exists() {
        return Ok(SessionStore::default());
    }

    let content = std::fs::read_to_string(&path)?;
    let store: SessionStore = serde_json::from_str(&content)?;
    Ok(store)
}

pub fn save_store(store: &SessionStore) -> Result<()> {
    let path = get_store_path()?;
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_session(id: String, command: String, cwd: String) -> Result<()> {
    let mut store = load_store()?;
    let session = Session {
        id: id.clone(),
        command,
        cwd,
    };
    store.sessions.insert(id, session);
    save_store(&store)?;
    Ok(())
}

pub async fn cleanup_stale_sessions() -> Result<()> {
    let active_sessions = wrapper::list_sessions().await?;
    let mut store = load_store()?;

    store.sessions.retain(|id, _| active_sessions.contains(id));

    save_store(&store)?;
    Ok(())
}
