use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use distrust::{AppState, data::Database, routes};
use std::env;
use std::io;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let db_path = env::current_dir()?.join("data/resources.db");

    let db = Database::connect(&db_path)
        .await
        .map_err(|e| io::Error::other(e.to_string()))?;


    let state = AppState { db: Arc::new(db) };

    let app = Router::new()
        .route("/", get(routes::serve_homepage))
        .route("/static/style.css", get(routes::serve_css))
        .route("/paste", post(routes::create_paste))
        .route("/paste/{id}", get(routes::get_paste))
        .route("/raw/{id}", get(routes::get_paste_raw))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .with_state(state);

    let host = "0.0.0.0:6969";
    let listener = TcpListener::bind(host).await?;
    println!("[INFO] Server listening on http://{}", host);

    axum::serve(listener, app)
        .await
        .map_err(io::Error::other)
}
