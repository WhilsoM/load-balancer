use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use serde_json::{Value, json};

struct AppState {
    name: String,
}

pub async fn run(name: &str, addr: &str) -> std::io::Result<()> {
    let shared_state = Arc::new(AppState {
        name: name.to_string(),
    });

    let app = Router::new()
        .route("/name", get(get_name))
        .route("/health", get(health))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("{name} listening on {addr}");

    axum::serve(listener, app).await
}

async fn get_name(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({ "name": state.name }))
}

async fn health() -> Json<Value> {
    Json(json!({ "message": "OK" }))
}
