use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use distrust::{AppState, MAX_SIZE, data::Database, routes};
use std::{env, io};
use tokio::fs;
use std::sync::Arc;
use tokio::{io::Result, net::TcpListener};

#[tokio::main]
async fn main() -> Result<()> {
    let db_path = env::current_dir()?.join("data/resources.db");
    let cache_dir = env::current_dir()?.join("cache");
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir).await?;
        println!("[INFO] Created cache directory at {}", cache_dir.display());
    }

    let db = Database::connect(&db_path)
        .await
        .map_err(|e| io::Error::other(e.to_string()))?;

    let state = AppState {
        db: Arc::new(db),
        cache_dir,
    };

    let app = Router::new()
        .route("/", get(routes::serve_homepage))
        .route("/static/style.css", get(routes::serve_css))
        .route("/paste", post(routes::create_paste))
        .route("/paste/{id}", get(routes::get_paste))
        .route("/raw/{id}", get(routes::get_paste_raw))
        .layer(DefaultBodyLimit::max(MAX_SIZE))
        .with_state(state);

    let host = "0.0.0.0:6969";
    let listener = TcpListener::bind(host).await?;
    println!("[INFO] Server listening on http://{}", host);

    axum::serve(listener, app).await.map_err(io::Error::other)
}
