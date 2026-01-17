use anyhow::Result;

pub struct TmuxWrapper;

impl TmuxWrapper {
    pub fn new() -> Self {
        Self
    }

    pub async fn create_session(&self, name: &str, command: &str, cwd: &str) -> Result<()> {
        Ok(())
    }

    pub async fn send_keys(&self, session: &str, keys: &str) -> Result<()> {
        Ok(())
    }

    pub async fn capture_pane(&self, session: &str) -> Result<String> {
        Ok(String::new())
    }

    pub async fn kill_session(&self, session: &str) -> Result<()> {
        Ok(())
    }
}

impl Default for TmuxWrapper {
    fn default() -> Self {
        Self::new()
    }
}
