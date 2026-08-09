use std::sync::Arc;
use rustls::{RootCertStore, ServerConfig};
use rustls::server::WebPkiClientVerifier;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::{timeout, Duration, interval};
use tokio_rustls::TlsAcceptor;
use tokio_util::sync::CancellationToken;
use common::certificates::{load_certificates, load_private_key};

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
                    Ok(mut stream) => {
                        let mut ping_interval = interval(Duration::from_secs(10));
                        let mut bufor = [0u8; 1024];
                        loop {
                            tokio::select! {
                                _ = ping_interval.tick() => {
                                    if let Err(e) = stream.write_all(b"PING\n").await {
                                        println!("Client dead (write failed): {e}");
                                        break;
                                    }
                                }
                                result = timeout(Duration::from_secs(15), stream.read(&mut bufor)) => {
                                    match result {
                                        Ok(Ok(0)) => { println!("Client closed connection."); break; }
                                        Ok(Ok(n)) => {
                                            let response = String::from_utf8_lossy(&bufor[..n]);
                                            if response.eq("PONG") {
                                                println!("Client responded with PONG.");
                                            } else {
                                                println!("Received message: '{}'", response);

                                                _ = stream.write_all("I send respond. Server :)".as_bytes()).await;
                                            }
                                        }
                                        Ok(Err(e)) => { println!("Read error: {e}"); break; }
                                        Err(_) => { println!("No response in 15s — client presumed dead."); break; }
                                    }
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