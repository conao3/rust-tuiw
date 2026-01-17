use crate::daemon::session::SessionManager;
use crate::types::{CreateSessionInput, SendKeysInput, SessionId};
use async_graphql::{Context, Object, Result};

#[allow(dead_code)]
pub struct Mutation;

#[Object]
impl Mutation {
    async fn create_session(
        &self,
        ctx: &Context<'_>,
        input: CreateSessionInput,
    ) -> Result<SessionId> {
        let session_manager = ctx.data::<SessionManager>()?;
        let session_id = session_manager
            .create_session(input.command, input.cwd)
            .await?;
        Ok(session_id)
    }

    async fn send_keys(&self, ctx: &Context<'_>, input: SendKeysInput) -> Result<bool> {
        let session_manager = ctx.data::<SessionManager>()?;
        session_manager
            .send_keys(&input.session_id, input.keys)
            .await?;
        Ok(true)
    }

    async fn close_session(&self, ctx: &Context<'_>, session_id: SessionId) -> Result<bool> {
        let session_manager = ctx.data::<SessionManager>()?;
        let removed = session_manager.remove_session(&session_id).await?;
        Ok(removed.is_some())
    }
}
