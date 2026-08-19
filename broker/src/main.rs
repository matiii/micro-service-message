use tokio_util::sync::CancellationToken;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::server::{Server, ServerConfiguration};

mod server;
mod client_connection;
mod router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Micro Service Message broker started!");

    let _logging = init_tracing();
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

fn init_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    // Rolling daily log file: logs/myapp.2026-08-17.log
    let file_appender = tracing_appender::rolling::daily("logs", "msm.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let stdout_layer = fmt::layer()
        .with_target(true)
        .with_writer(std::io::stdout);

    let file_layer = fmt::layer()
        .json()
        .with_target(true)
        .with_ansi(false) //
        .with_writer(non_blocking);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("trace")))
        .with(stdout_layer)
        .with(file_layer)
        .init();

    guard // must be kept alive for the life of the program
}


