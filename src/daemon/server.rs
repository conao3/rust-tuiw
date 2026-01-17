use crate::config::{DEFAULT_HOST, DEFAULT_PORT};
use crate::daemon::session::SessionManager;
use crate::daemon::sse::screen_changes_handler;
use crate::graphql::{mutation::Mutation, query::Query, subscription::Subscriptions};
use crate::types::SessionId;
use anyhow::Result;
use async_graphql::Schema;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    Extension, Router,
    extract::{Path, State},
    routing::{get, post},
};

type AppSchema = Schema<Query, Mutation, Subscriptions>;

#[derive(Clone)]
struct AppState {
    schema: AppSchema,
    session_manager: SessionManager,
}

pub async fn run_daemon() -> Result<()> {
    tracing::info!("starting daemon on {}:{}", DEFAULT_HOST, DEFAULT_PORT);

    let session_manager = SessionManager::new();

    let schema = Schema::build(Query, Mutation, Subscriptions)
        .data(session_manager.clone())
        .finish();

    let state = AppState {
        schema,
        session_manager: session_manager.clone(),
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/graphql", post(graphql_handler))
        .route("/sse/{session_id}", get(sse_handler))
        .with_state(state);

    let addr = format!("{}:{}", DEFAULT_HOST, DEFAULT_PORT);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("daemon listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "ok"
}

async fn graphql_handler(State(state): State<AppState>, req: GraphQLRequest) -> GraphQLResponse {
    state.schema.execute(req.into_inner()).await.into()
}

async fn sse_handler(
    State(state): State<AppState>,
    Path(session_id_str): Path<String>,
) -> Result<impl axum::response::IntoResponse, String> {
    let uuid =
        uuid::Uuid::parse_str(&session_id_str).map_err(|_| "invalid session ID".to_string())?;
    let session_id = SessionId(uuid);

    Ok(screen_changes_handler(State(state.session_manager), Extension(session_id)).await)
}
