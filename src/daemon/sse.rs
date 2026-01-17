use crate::daemon::session::SessionManager;
use crate::types::SessionId;
use axum::{
    Extension,
    extract::State,
    response::sse::{Event, Sse},
};
use futures::stream::Stream;
use std::convert::Infallible;
use std::time::Duration;

pub async fn screen_changes_handler(
    State(session_manager): State<SessionManager>,
    Extension(session_id): Extension<SessionId>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        let mut last_output = String::new();

        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;

            if let Ok(output) = session_manager.get_output(&session_id).await
                && output != last_output
            {
                last_output = output.clone();
                yield Ok(Event::default().data(output));
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive"),
    )
}
