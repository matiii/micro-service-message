use tokio_util::sync::CancellationToken;
use crate::server::{Server, ServerConfiguration};

mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world!");

    let configuration = ServerConfiguration::new(
        "0.0.0.0".to_string(),
        3033,
        "certificates/server/server-cert.pem".to_string(),
        "certificates/server/server-key.pem".to_string(),
        "certificates/ca/ca-cert.pem".to_string());
    let server = Server::new(configuration);
    let cancellation_token = CancellationToken::new();

    server.run(cancellation_token).await?;

    Ok(())
}


