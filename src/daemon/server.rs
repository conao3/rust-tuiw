use crate::config::{DEFAULT_HOST, DEFAULT_PORT};
use crate::daemon::session::SessionManager;
use crate::graphql::{mutation::Mutation, query::Query, subscription::Subscriptions};
use anyhow::Result;
use async_graphql::Schema;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::State, routing::{get, post}, Router};

type AppSchema = Schema<Query, Mutation, Subscriptions>;

#[derive(Clone)]
struct AppState {
    schema: AppSchema,
}

pub async fn run_daemon() -> Result<()> {
    tracing::info!("starting daemon on {}:{}", DEFAULT_HOST, DEFAULT_PORT);

    let session_manager = SessionManager::new();

    let schema = Schema::build(Query, Mutation, Subscriptions)
        .data(session_manager)
        .finish();

    let state = AppState { schema };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/graphql", post(graphql_handler))
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

async fn graphql_handler(
    State(state): State<AppState>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    state.schema.execute(req.into_inner()).await.into()
}
