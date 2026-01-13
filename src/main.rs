use axum::{routing::get, Router};
use sqlx::{postgres::PgPoolOptions, PgPool};

#[derive(Clone)]
struct AppState {
    db: PgPool,
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

    // Create app state
    let state = AppState { db };

    // Create router with state
    let app = Router::new()
        .route("/", get(|| async { "Hola Mundo" }))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Servidor escuchando en http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
