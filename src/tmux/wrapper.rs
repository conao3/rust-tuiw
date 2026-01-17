use anyhow::Result;

#[allow(dead_code)]
pub struct TmuxWrapper;

impl TmuxWrapper {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    pub async fn create_session(&self, _name: &str, _command: &str, _cwd: &str) -> Result<()> {
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn send_keys(&self, _session: &str, _keys: &str) -> Result<()> {
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn capture_pane(&self, _session: &str) -> Result<String> {
        Ok(String::new())
    }

    #[allow(dead_code)]
    pub async fn kill_session(&self, _session: &str) -> Result<()> {
        Ok(())
    }
}

impl Default for TmuxWrapper {
    fn default() -> Self {
        Self::new()
    }
}
