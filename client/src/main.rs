use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::TlsConnector;
use tokio_util::codec::{ FramedRead, FramedWrite, LengthDelimitedCodec};
use common::certificates::{load_certificates, load_private_key};
use common::op_codes::{ConnectDetails, OpCode, SendDetails};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello from client side!");

    let client_name = "some_client";
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
    let tls_stream = connector.connect(server_name, tcp_stream).await?;
    let (stream_read, stream_write) = tokio::io::split(tls_stream);
    let mut framed_read = FramedRead::new(stream_read, LengthDelimitedCodec::new());
    let mut framed_write = FramedWrite::new(stream_write, LengthDelimitedCodec::new());
    let (tx, mut rx) = tokio::sync::mpsc::channel::<OpCode>(32);
    let keep_alive_tx= tx.clone();

    tokio::spawn(async move {
       while let Some(x) = rx.recv().await {
            if let Ok(message) = x.serialize() {
                if framed_write.send(message.into()).await.is_err() {
                    println!("Failed to write message to broker: {:?}", x);
                }
            } else {
                println!("Failed to serialize message {:?}", x);
            }
        }
    });

    tokio::spawn(async move {
        while let Some(Ok(x)) =framed_read.next().await {
            println!("Received message: '{:?}'", x);
        }
    });

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;

            _ = keep_alive_tx.send(OpCode::KeepAlive(client_name.to_string())).await;
        }
    });

    tx.send(OpCode::Connect(ConnectDetails::new(client_name.to_string(), vec!["*".to_string()], 1000))).await?;

    loop {
        print!("Type a message: ");
        std::io::stdout().flush()?;

        let mut user_input = String::new();
        if let Ok(x) = std::io::stdin().read_line(&mut user_input) {
            let input = user_input.trim();

            println!("Send input from client: '{}'", input);
            
            tx.send(OpCode::Send(SendDetails::new(
                "some-queue".to_string(),
                input.to_string(),
                1,
                1,
                Vec::new(),
                Vec::new(),
                None,
            ))).await?;
        }
    }
}
