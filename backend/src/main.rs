use anyhow::Result;
use axum::{
    extract::DefaultBodyLimit,
    routing::get,
    Router,
};
use dotenvy::dotenv;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::env;
use std::str::FromStr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod models;
mod services;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Setup logging (RUST_LOG)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting paper-gloss backend");

    // Setup database connection pool
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://paper-gloss.db".to_string());

    let connect_options = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;

    tracing::info!("Database connection pool established");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    tracing::info!("Database migrations completed");

    // Build application routes
    let app = Router::new()
        .route("/", get(|| async { "Paper Gloss API" }))
        .route("/health", get(|| async { "OK" }))
        // Papers API
        .route("/api/papers/import", axum::routing::post(api::papers::import_paper))
        .route("/api/papers", get(api::papers::list_papers))
        .route("/api/papers/:id", get(api::papers::get_paper).delete(api::papers::delete_paper))
        .route("/api/papers/:id/translation", get(api::papers::get_translation))
        .route("/api/papers/:id/process", axum::routing::post(api::papers::process_paper))
        .route("/api/papers/:id/status", get(api::papers::get_paper_status))
        .route("/api/papers/:id/file", get(api::papers::get_paper_file))
        // Pipeline endpoints (JP-first architecture)
        .route("/api/papers/:id/translate", axum::routing::post(api::papers::translate_paper))
        .route("/api/papers/:id/extract-terms-jp", axum::routing::post(api::papers::extract_terms_jp))
        .route("/api/papers/:id/scan-jp", axum::routing::post(api::papers::scan_jp))
        .route("/api/papers/:id/generate-definitions", axum::routing::post(api::papers::generate_definitions))
        // Chunks API
        .route("/api/chunks/:id/retry", axum::routing::post(api::chunks::retry_chunk))
        // Terms API
        .route("/api/terms", get(api::terms::list_terms).post(api::terms::create_term))
        .route("/api/terms/duplicates", get(api::terms::find_duplicates))
        .route("/api/terms/merge", axum::routing::post(api::terms::merge_terms))
        .route("/api/terms/:id", get(api::terms::get_term_detail).patch(api::terms::update_term).delete(api::terms::delete_term))
        .route("/api/terms/:id/define", axum::routing::post(api::terms::generate_definition))
        // Occurrences API
        .route("/api/occurrences", get(api::occurrences::list_occurrences))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(150 * 1024 * 1024)) // 150 MB limit for PDF uploads
        .with_state(pool);

    // Get server configuration
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);

    tracing::info!("Server listening on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
