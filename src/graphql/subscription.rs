use crate::daemon::session::SessionManager;
use crate::types::SessionId;
use async_graphql::futures_util::stream::Stream;
use async_graphql::{Context, Result, Subscription};
use std::time::Duration;

#[allow(dead_code)]
pub struct Subscriptions;

#[Subscription]
impl Subscriptions {
    async fn screen_changes(
        &self,
        ctx: &Context<'_>,
        session_id: SessionId,
    ) -> Result<impl Stream<Item = String>> {
        let session_manager = ctx.data::<SessionManager>()?.clone();

        let stream = async_stream::stream! {
            let mut last_output = String::new();

            loop {
                tokio::time::sleep(Duration::from_millis(500)).await;

                if let Ok(output) = session_manager.get_output(&session_id).await {
                    if output != last_output {
                        last_output = output.clone();
                        yield output;
                    }
                }
            }
        };

        Ok(stream)
    }
}
