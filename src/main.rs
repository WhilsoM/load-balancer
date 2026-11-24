use crate::backend::server::run as run_backend;
use crate::load_balancer::server::run as run_load_balancer;

mod backend;
mod load_balancer;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let _ = tokio::try_join!(
        run_backend("Backend 1", "127.0.0.1:3001"),
        run_backend("Backend 2", "127.0.0.1:3002"),
        run_backend("Backend 3", "127.0.0.1:3003"),
        run_load_balancer("127.0.0.1:8080"),
    );

    Ok(())
}
