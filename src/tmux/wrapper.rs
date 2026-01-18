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

pub async fn capture_pane_with_color(session: &str, with_color: bool) -> Result<String> {
    let mut args = vec!["capture-pane", "-t", session, "-p"];
    if with_color {
        args.push("-e");
    }

    let output = Command::new("tmux")
        .args(&args)
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

pub async fn list_sessions() -> Result<Vec<String>> {
    let output = Command::new("tmux")
        .args(["list-sessions", "-F", "#{session_name}"])
        .output()
        .await
        .context("failed to execute tmux list-sessions")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("no server running") {
            return Ok(Vec::new());
        }
        anyhow::bail!("tmux list-sessions failed: {}", stderr);
    }

    let stdout = String::from_utf8(output.stdout)
        .context("failed to parse tmux list-sessions output as UTF-8")?;

    Ok(stdout.lines().map(|s| s.to_string()).collect())
}
