use anyhow::Result;

pub struct TmuxWrapper;

impl TmuxWrapper {
    pub fn new() -> Self {
        Self
    }

    pub async fn create_session(&self, _name: &str, _command: &str, _cwd: &str) -> Result<()> {
        Ok(())
    }

    pub async fn send_keys(&self, _session: &str, _keys: &str) -> Result<()> {
        Ok(())
    }

    pub async fn capture_pane(&self, _session: &str) -> Result<String> {
        Ok(String::new())
    }

    pub async fn kill_session(&self, _session: &str) -> Result<()> {
        Ok(())
    }
}

impl Default for TmuxWrapper {
    fn default() -> Self {
        Self::new()
    }
}
