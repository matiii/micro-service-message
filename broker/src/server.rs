use std::sync::Arc;
use rustls::{RootCertStore, ServerConfig};
use rustls::server::WebPkiClientVerifier;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace};
use common::certificates::{load_certificates, load_private_key};
use crate::client_connection::ClientConnection;

pub struct Server {
    configuration: ServerConfiguration,
    storage_configuration: StorageConfiguration,
}

impl Server {
    pub fn new(configuration: ServerConfiguration, storage_configuration: StorageConfiguration) -> Self {
        Server { configuration, storage_configuration }
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

        info!("Server listening on {}", listener.local_addr()?);

        loop {
            if cancellation_token.is_cancelled() {
                break;
            }

            let client_connection = ClientConnection::new(&listener, acceptor.clone());

            match client_connection.handle().await {
                Err(error) => {
                    error!(%error, "Error handling client connection");
                }
                Ok(_) => {
                    trace!("Client connection established");
                }
            }
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

pub struct StorageConfiguration {
    storage_path: String,
    snapshot_path: String,
}

impl StorageConfiguration {
    pub fn new(storage_path: String, snapshot_path: String) -> Self {
        Self { storage_path, snapshot_path }
    }

    pub fn storage_path(&self) -> &str {
        &self.storage_path
    }

    pub fn snapshot_path(&self) -> &str {
        &self.snapshot_path
    }
}