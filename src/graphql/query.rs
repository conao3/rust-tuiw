use crate::daemon::session::SessionManager;
use crate::types::{Session, SessionId, SessionStatus};
use async_graphql::{Context, Object, Result};

pub struct Query;

#[Object]
impl Query {
    async fn sessions(&self, ctx: &Context<'_>) -> Result<Vec<Session>> {
        let session_manager = ctx.data::<SessionManager>()?;
        Ok(session_manager.list_sessions().await)
    }

    async fn session_capture(&self, ctx: &Context<'_>, session_id: SessionId) -> Result<String> {
        let session_manager = ctx.data::<SessionManager>()?;
        let output = session_manager.get_output(&session_id).await?;
        Ok(output)
    }

    async fn session_status(
        &self,
        ctx: &Context<'_>,
        session_id: SessionId,
    ) -> Result<SessionStatus> {
        let session_manager = ctx.data::<SessionManager>()?;
        let status = session_manager.get_session_status(&session_id).await?;
        Ok(status)
    }
}
