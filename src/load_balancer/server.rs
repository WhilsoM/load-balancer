use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use axum::{
    Router,
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
};
use reqwest::header;

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
    backends: Vec<String>,
    next_backend: Arc<AtomicUsize>,
}

pub async fn run(addr: &str) -> std::io::Result<()> {
    let state = AppState {
        client: reqwest::Client::new(),
        backends: vec![
            "http://127.0.0.1:3001".to_string(),
            "http://127.0.0.1:3002".to_string(),
            "http://127.0.0.1:3003".to_string(),
        ],
        next_backend: Arc::new(AtomicUsize::new(0)),
    };

    let app = Router::new()
        .route("/", get(proxy_handler))
        .fallback(proxy)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("Load Balancer started on {addr}");

    axum::serve(listener, app).await
}

async fn proxy_handler(State(state): State<AppState>) -> (StatusCode, String) {
    (
        StatusCode::OK,
        format!(
            "Load Balancer is running! Backends: {}",
            state.backends.len()
        ),
    )
}

async fn proxy(
    State(state): State<AppState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if state.backends.is_empty() {
        return (StatusCode::SERVICE_UNAVAILABLE, "no available backends").into_response();
    }

    let path_query = uri
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or(uri.path());
    let index = state.next_backend.fetch_add(1, Ordering::Relaxed) % state.backends.len();
    let backend = &state.backends[index];
    let url = format!("{}{}", backend.trim_end_matches('/'), path_query);

    let upstream = match state
        .client
        .request(method, url)
        .headers(headers)
        .body(body)
        .send()
        .await
    {
        Ok(v) => v,
        Err(error) => {
            eprintln!("Backend request failed: {error}");

            return (StatusCode::BAD_GATEWAY, "Backend unavailable").into_response();
        }
    };

    let status =
        StatusCode::from_u16(upstream.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);

    let content_type = upstream.headers().get(header::CONTENT_TYPE).cloned();

    let response_body = match upstream.bytes().await {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("Failed to read backend body: {error}");

            return (StatusCode::BAD_GATEWAY, "Failed to read backend response").into_response();
        }
    };

    let mut response = Response::new(Body::from(response_body));
    *response.status_mut() = status;

    if let Some(value) = content_type {
        response.headers_mut().insert(header::CONTENT_TYPE, value);
    }

    response
}
