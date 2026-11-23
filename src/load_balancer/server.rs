use axum::Router;

pub async fn run(name: &str, addr: &str) -> std::io::Result<()> {
    let app = Router::new();

    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("{name} listening on {addr}");

    axum::serve(listener, app).await
}
