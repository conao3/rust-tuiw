use crate::daemon::session::SessionManager;
use crate::types::{Session, SessionId, SessionStatus};
use async_graphql::{Context, Object, Result};

#[allow(dead_code)]
pub struct Query;

#[Object]
impl Query {
    async fn list_sessions(&self, ctx: &Context<'_>) -> Result<Vec<Session>> {
        let session_manager = ctx.data::<SessionManager>()?;
        Ok(session_manager.list_sessions().await)
    }

    async fn get_output(&self, ctx: &Context<'_>, session_id: SessionId) -> Result<String> {
        let session_manager = ctx.data::<SessionManager>()?;
        let output = session_manager.get_output(&session_id).await?;
        Ok(output)
    }

    async fn get_session_status(
        &self,
        ctx: &Context<'_>,
        session_id: SessionId,
    ) -> Result<SessionStatus> {
        let session_manager = ctx.data::<SessionManager>()?;
        let status = session_manager.get_session_status(&session_id).await?;
        Ok(status)
    }
}
