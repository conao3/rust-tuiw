use crate::config::{DEFAULT_HOST, DEFAULT_PORT};
use crate::daemon::session::SessionManager;
use crate::graphql::query::Query;
use anyhow::Result;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use axum::{routing::get, Router};
use std::sync::Arc;

pub async fn run_daemon() -> Result<()> {
    tracing::info!("starting daemon on {}:{}", DEFAULT_HOST, DEFAULT_PORT);

    let session_manager = Arc::new(SessionManager::new());

    let _schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(session_manager)
        .finish();

    let app = Router::new()
        .route("/health", get(health_check));

    let addr = format!("{}:{}", DEFAULT_HOST, DEFAULT_PORT);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("daemon listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "ok"
}
