use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use sqlx::{postgres::PgPoolOptions, PgPool};

mod models;
use models::{AiAudit, CreateAuditRequest};

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

async fn create_audit(
    State(state): State<AppState>,
    Json(payload): Json<CreateAuditRequest>,
) -> Result<(StatusCode, Json<AiAudit>), StatusCode> {
    let audit = sqlx::query_as::<_, AiAudit>(
        r#"
        INSERT INTO ai_audits (prompt, codigo_generado, es_valido, error_compilacion)
        VALUES ($1, $2, $3, $4)
        RETURNING id, prompt, codigo_generado, es_valido, error_compilacion, created_at
        "#,
    )
    .bind(&payload.prompt)
    .bind(&payload.codigo_generado)
    .bind(payload.es_valido)
    .bind(&payload.error_compilacion)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(audit)))
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

    // Create app state
    let state = AppState { db };

    // Create router with state
    let app = Router::new()
        .route("/", get(|| async { "Hola Mundo" }))
        .route("/audit", post(create_audit))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Servidor escuchando en http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
