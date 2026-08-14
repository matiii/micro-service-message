use std::sync::Arc;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use rustls::{RootCertStore, ServerConfig};
use rustls::server::WebPkiClientVerifier;
use tokio::net::TcpListener;
use tokio::time::{timeout, Duration, interval};
use tokio_rustls::TlsAcceptor;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use tokio_util::sync::CancellationToken;
use common::certificates::{load_certificates, load_private_key};
use common::op_codes::OpCode;

pub struct Server {
    configuration: ServerConfiguration,
}

impl Server {
    pub fn new(configuration: ServerConfiguration) -> Self {
        Server { configuration }
    }

    pub async fn run(&self, cancellation_token: CancellationToken) -> anyhow::Result<()> {
        let server_certs = load_certificates(self.configuration().certificate_path())?;
        let keys = load_private_key(self.configuration().private_key_path())?;

        let mut roots = RootCertStore::empty();
        let certificates = load_certificates(self.configuration().ca_path())?;
        for cert in certificates {
            roots.add(cert)?;
        }

        let client_verifier = WebPkiClientVerifier::builder(Arc::new(roots))
            .build()?;

        let config = ServerConfig::builder()
            .with_client_cert_verifier(client_verifier)
            .with_single_cert(server_certs, keys)?;
        let acceptor = TlsAcceptor::from(Arc::new(config));
        let listener = TcpListener::bind(self.configuration.address()).await?;

        loop {
            if cancellation_token.is_cancelled() {
                break;
            }

            let (stream, addr) = listener.accept().await?;
            let acceptor = acceptor.clone();

            tokio::spawn(async move {
                match acceptor.accept(stream).await {
                    Ok(stream) => {
                        let test_connection_code: Bytes = OpCode::TestConnection.serialize().unwrap().into();
                        let (stream_read, stream_write) = tokio::io::split(stream);
                        let mut framed_read = FramedRead::new(stream_read, LengthDelimitedCodec::new());
                        let mut framed_write = FramedWrite::new(stream_write, LengthDelimitedCodec::new());
                        loop {
                            if let Ok(message_stream) = timeout(Duration::from_secs(120), framed_read.next()).await {
                                if let Some(Ok(x)) = message_stream {
                                    if let Ok(op_code) = OpCode::deserialize(&x) {
                                        match op_code {
                                            OpCode::KeepAlive(_) => { println!("KeepAlive"); continue; }
                                            OpCode::TestConnection => {
                                                println!("TestConnection");
                                                continue;
                                            }
                                            OpCode::Connect(connect_details) => {
                                                println!("Connect: {:?}", connect_details);
                                            }
                                            OpCode::Disconnect(_) => {}
                                            OpCode::Send(send_details) => {
                                                println!("Send: {:?}", send_details);
                                            }
                                            OpCode::Confirmed(_) => {}
                                            OpCode::Receive(_) => {}
                                            OpCode::Commit(_) => {}
                                            OpCode::SetState(_, _) => {}
                                            OpCode::GetState(_) => {}
                                        }
                                    }
                                }
                            }
                            else {
                                println!("Timeout from message read.");
                                if framed_write.send(test_connection_code.clone()).await.is_err() {
                                    println!("Client connection closed. Quit connection.");
                                    break;
                                }
                            }
                        }

                    },
                    Err(e) => {
                        println!("Failed to accept connection: {}", e);
                    }
                }
            });
        }

        Ok(())
    }

    pub fn configuration(&self) -> &ServerConfiguration {
        &self.configuration
    }
}

pub struct ServerConfiguration {
    host: String,
    port: u16,
    certificate_path: String,
    private_key_path: String,
    ca_path: String,
}

impl ServerConfiguration {
    pub fn new(host: String, port: u16, certificate_path: String, private_key_path: String, ca_path: String) -> Self {
        Self {
            host,
            port,
            certificate_path,
            private_key_path,
            ca_path,
        }
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn address(&self) -> String { format!("{}:{}",self.host(), self.port()) }

    pub fn certificate_path(&self) -> &str {
        &self.certificate_path
    }

    pub fn private_key_path(&self) -> &str {
        &self.private_key_path
    }

    pub fn ca_path(&self) -> &str {
        &self.ca_path
    }
}