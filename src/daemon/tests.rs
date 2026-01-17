use super::session::SessionManager;
use crate::types::SessionStatus;

#[tokio::test]
#[ignore]
async fn test_session_manager_lifecycle() {
    let manager = SessionManager::new();

    let session_id = manager
        .create_session("bash".to_string(), "/tmp".to_string())
        .await
        .expect("failed to create session");

    let sessions = manager.list_sessions().await;
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, session_id);

    let session = manager.get_session(&session_id).await;
    assert!(session.is_some());
    assert_eq!(session.unwrap().command, "bash");

    let status = manager
        .get_session_status(&session_id)
        .await
        .expect("failed to get status");
    assert!(matches!(status, SessionStatus::Running));

    manager
        .send_keys(&session_id, "echo test".to_string())
        .await
        .expect("failed to send keys");

    manager
        .send_keys(&session_id, "Enter".to_string())
        .await
        .expect("failed to send Enter");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let output = manager
        .get_output(&session_id)
        .await
        .expect("failed to get output");
    assert!(output.contains("test"), "output should contain 'test'");

    let removed = manager
        .remove_session(&session_id)
        .await
        .expect("failed to remove session");
    assert!(removed.is_some());

    let sessions = manager.list_sessions().await;
    assert_eq!(sessions.len(), 0);
}
