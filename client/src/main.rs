use std::io;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::TlsConnector;
use common::certificates::{load_certificates, load_private_key};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello from client side!");

    let mut roots = RootCertStore::empty();
    let certificates = load_certificates("certificates/ca/ca-cert.pem")?;
    for cert in certificates {
        roots.add(cert)?;
    }

    let client_cert = load_certificates("certificates/client/client-cert.pem")?;
    let client_key = load_private_key("certificates/client/client-key.pem")?;
    let config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_client_auth_cert(client_cert, client_key)?;
    let connector = TlsConnector::from(Arc::new(config));
    let tcp_stream = TcpStream::connect("127.0.0.1:3033").await?;
    let server_name = ServerName::try_from("localhost")?;
    let mut tls_stream = connector.connect(server_name, tcp_stream).await?;

    loop {
        print!("Type a message: ");

        let mut user_input = String::new();
        if let Ok(x) = io::stdin().read_line(&mut user_input) {
            let input = user_input.trim();

            println!("Send input from client: '{}'", input);

            tls_stream.write_all(input.as_bytes()).await?;
        }
    }

    Ok(())
}
