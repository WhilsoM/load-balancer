use crate::backend::server::run;

mod backend;
mod load_balancer;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let _ = tokio::try_join!(
        run("Backend 1", "127.0.0.1:3001"),
        run("Backend 2", "127.0.0.1:3002"),
        run("Backend 3", "127.0.0.1:3003"),
    );

    Ok(())
}
