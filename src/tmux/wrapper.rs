use anyhow::{Context, Result};
use tokio::process::Command;

pub async fn create_session(name: &str, command: &str, cwd: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(["new-session", "-d", "-s", name, "-c", cwd, command])
        .output()
        .await
        .context("failed to execute tmux new-session")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("tmux new-session failed: {}", stderr);
    }

    Ok(())
}

pub async fn send_keys(session: &str, keys: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(["send-keys", "-t", session, keys])
        .output()
        .await
        .context("failed to execute tmux send-keys")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("tmux send-keys failed: {}", stderr);
    }

    Ok(())
}

pub async fn capture_pane(session: &str) -> Result<String> {
    let output = Command::new("tmux")
        .args(["capture-pane", "-t", session, "-p"])
        .output()
        .await
        .context("failed to execute tmux capture-pane")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("tmux capture-pane failed: {}", stderr);
    }

    let stdout = String::from_utf8(output.stdout)
        .context("failed to parse tmux capture-pane output as UTF-8")?;

    Ok(stdout)
}

pub async fn kill_session(session: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(["kill-session", "-t", session])
        .output()
        .await
        .context("failed to execute tmux kill-session")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("tmux kill-session failed: {}", stderr);
    }

    Ok(())
}

pub async fn session_exists(session: &str) -> Result<bool> {
    let output = Command::new("tmux")
        .args(["has-session", "-t", session])
        .output()
        .await
        .context("failed to execute tmux has-session")?;

    Ok(output.status.success())
}
