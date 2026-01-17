use super::wrapper::TmuxWrapper;

#[tokio::test]
#[ignore]
async fn test_tmux_session_lifecycle() {
    let wrapper = TmuxWrapper::new();
    let session_name = "test-rust-tuiw";

    if wrapper.session_exists(session_name).await.unwrap_or(false) {
        wrapper.kill_session(session_name).await.ok();
    }

    wrapper
        .create_session(session_name, "bash", "/tmp")
        .await
        .expect("failed to create session");

    assert!(
        wrapper.session_exists(session_name).await.unwrap(),
        "session should exist"
    );

    wrapper
        .send_keys(session_name, "echo hello")
        .await
        .expect("failed to send keys");

    wrapper
        .send_keys(session_name, "Enter")
        .await
        .expect("failed to send Enter");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let output = wrapper
        .capture_pane(session_name)
        .await
        .expect("failed to capture pane");

    assert!(output.contains("hello"), "output should contain 'hello'");

    wrapper
        .kill_session(session_name)
        .await
        .expect("failed to kill session");

    assert!(
        !wrapper.session_exists(session_name).await.unwrap(),
        "session should not exist after kill"
    );
}
