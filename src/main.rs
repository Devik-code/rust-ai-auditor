use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use sqlx::{postgres::PgPoolOptions, PgPool};

mod models;
mod schema;
mod services;

use models::CreateAuditRequest;
use schema::{AppSchema, MutationRoot, QueryRoot};

#[derive(Clone)]
struct AppState {
    db: PgPool,
    schema: AppSchema,
}

async fn create_audit_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateAuditRequest>,
) -> Result<(StatusCode, Json<models::AiAudit>), StatusCode> {
    let audit = services::create_audit(&state.db, &payload)
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::CREATED, Json(audit)))
}

async fn graphql_handler(
    State(state): State<AppState>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    state.schema.execute(req.into_inner()).await.into()
}

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

#[tokio::main]
async fn main() {
    // Load .env file
    dotenvy::dotenv().ok();

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    // Create database connection pool
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap_or_else(|e| {
            panic!("Failed to connect to Postgres: {}", e);
        });

    // Verify connection with a test query
    let version: (String,) = sqlx::query_as("SELECT version()")
        .fetch_one(&db)
        .await
        .unwrap_or_else(|e| {
            panic!("Failed to execute test query: {}", e);
        });

    println!("🚀 Conexión exitosa a Postgres");
    println!("📊 Database version: {}", version.0);

    // Run database migrations
    sqlx::migrate!()
        .run(&db)
        .await
        .unwrap_or_else(|e| {
            panic!("Failed to run migrations: {}", e);
        });

    println!("✅ Migraciones ejecutadas correctamente");

    // Create GraphQL schema
    let schema = async_graphql::Schema::build(QueryRoot, MutationRoot, async_graphql::EmptySubscription)
        .data(db.clone())
        .finish();

    // Create app state
    let state = AppState { db, schema };

    // Create router with state
    let app = Router::new()
        .route("/", get(graphiql))
        .route("/graphql", post(graphql_handler))
        .route("/audit", post(create_audit_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Servidor escuchando en http://localhost:3000");
    println!("🔮 GraphiQL IDE disponible en http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
