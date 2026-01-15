//! The main entry point for the rust-ai-auditor web service.
//!
//! This module sets up the database connection, initializes the web server (Axum),
//! configures logging (tracing), and defines the application's routes.

// Import necessary crates and modules.
use anyhow::Context;
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Declare application modules.
mod auditor;
mod error;
mod models;
mod schema;
mod services;

// Import items from our modules.
use crate::error::AppError;
use models::{AiAudit, AuditStats, CreateAuditRequest};
use schema::{AppSchema, MutationRoot, QueryRoot};

/// Represents the shared state that is accessible from all route handlers.
#[derive(Clone)]
struct AppState {
    /// The database connection pool.
    db: PgPool,
    /// The GraphQL schema.
    schema: AppSchema,
}

/// Handles REST requests to create a new AI code audit.
///
/// # Arguments
///
/// * `state` - The shared application state.
/// * `payload` - The JSON payload containing the audit request data.
///
/// # Returns
///
/// * `Ok((StatusCode, Json<AiAudit>))` - On success, returns a `201 CREATED` status
///   and the newly created audit record.
/// * `Err(AppError)` - On failure, returns an application-specific error.
async fn create_audit_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateAuditRequest>,
) -> Result<(StatusCode, Json<AiAudit>), AppError> {
    let audit = services::create_audit(&state.db, &payload).await?;
    Ok((StatusCode::CREATED, Json(audit)))
}

/// Handles REST requests to get audit statistics.
///
/// # Arguments
///
/// * `state` - The shared application state.
///
/// # Returns
///
/// * `Ok(Json<AuditStats>)` - On success, returns the audit statistics.
/// * `Err(AppError)` - On failure, returns an application-specific error.
async fn stats_handler(State(state): State<AppState>) -> Result<Json<AuditStats>, AppError> {
    let stats = services::get_audit_stats(&state.db).await?;
    Ok(Json(stats))
}

/// The main handler for all GraphQL requests.
///
/// It executes the incoming GraphQL query against the schema.
///
/// # Arguments
///
/// * `state` - The shared application state.
/// * `req` - The incoming GraphQL request.
///
/// # Returns
///
/// * `GraphQLResponse` - The result of the query execution.
async fn graphql_handler(State(state): State<AppState>, req: GraphQLRequest) -> GraphQLResponse {
    state.schema.execute(req.into_inner()).await.into()
}

/// Serves the GraphiQL user interface.
///
/// This provides a web-based IDE for exploring and testing the GraphQL API.
///
/// # Returns
///
/// * `impl IntoResponse` - An HTML response containing the GraphiQL page.
async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

/// The main entry point of the application.
///
/// It initializes the logger, loads environment variables, connects to the database,
/// runs migrations, builds the application state and router, and starts the web server.
///
/// # Returns
///
/// * `anyhow::Result<()>` - Returns `Ok(())` on successful server shutdown,
///   or an error if any part of the setup or server execution fails.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber for logging.
    // It reads the log level from the `RUST_LOG` environment variable,
    // defaulting to "rust_ai_auditor=info".
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_ai_auditor=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables from a .env file if it exists.
    dotenvy::dotenv().ok();

    // Get the database URL from the environment.
    let database_url = std::env::var("DATABASE_URL")
        .context("DATABASE_URL must be set in the environment or .env file")?;

    // Create a database connection pool.
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("Failed to connect to Postgres")?;

    // Verify the database connection with a test query.
    let version: (String,) = sqlx::query_as("SELECT version()").fetch_one(&db).await?;
    tracing::info!(db_version = %version.0, "Successfully connected to Postgres");

    // Run database migrations.
    sqlx::migrate!()
        .run(&db)
        .await
        .context("Failed to run database migrations")?;
    tracing::info!("Database migrations ran successfully");

    // Check if the Rust compiler is available.
    match auditor::check_rustc_available() {
        Ok(version) => tracing::info!(version = %version, "Rust compiler is available"),
        Err(e) => tracing::warn!(
            "Could not execute rustc: {}. Audit functionality will be impaired.",
            e
        ),
    }

    // Create the GraphQL schema.
    let schema =
        async_graphql::Schema::build(QueryRoot, MutationRoot, async_graphql::EmptySubscription)
            .data(db.clone())
            .finish();

    // Create the application state.
    let state = AppState { db, schema };

    // Build the Axum router.
    let app = Router::new()
        .route("/", get(graphiql))
        .route("/graphql", post(graphql_handler))
        .route("/audit", post(create_audit_handler))
        .route("/stats", get(stats_handler))
        .with_state(state);

    // Start the web server.
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Server listening on http://0.0.0.0:3000");
    tracing::info!("GraphiQL IDE available at http://localhost:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
