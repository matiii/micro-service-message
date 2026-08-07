use std::sync::Arc;
use anyhow::Context;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::pki_types::pem::PemObject;
use rustls::ServerConfig;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
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
        let certificates = load_certificates(self.configuration().certificate_path())?;
        let keys = load_private_key(self.configuration().private_key_path())?;

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certificates, keys)?;
        let acceptor = TlsAcceptor::from(Arc::new(config));
        let listener = TcpListener::bind(self.configuration.address()).await?;

        loop {
            if cancellation_token.is_cancelled() {
                break
            }

            let (stream, addr) = listener.accept().await?;
            let acceptor = acceptor.clone();

            tokio::spawn(async move {
                match acceptor.accept(stream).await {
                    Ok(mut stream) => {
                        let mut bufor = [0u8; 1024];
                        loop {
                            if let Ok(x) = stream.read(&mut bufor).await {
                                println!("Received message: '{}'", String::from_utf8_lossy(&bufor[..x]));

                                _ = stream.write_all("I send respond. Server :)".as_bytes()).await;
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
}

impl ServerConfiguration {
    pub fn new(host: String, port: u16, certificate_path: String, private_key_path: String) -> Self {
        Self {
            host,
            port,
            certificate_path,
            private_key_path,
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
}


// Since August 2025 `rustls-pemfile` has been archived/unmaintained — the recommended path now is to skip it entirely and use the PEM-parsing that's built into `rustls-pki-types` (v1.9+) directly via the `PemObject` trait. Here's the current setup with `tokio-rustls` (latest 0.26.x) and Tokio I/O.
//
// **Cargo.toml**
// ```toml
// [dependencies]
// tokio = { version = "1", features = ["full"] }
// tokio-rustls = "0.26"
// rustls-pki-types = { version = "1", features = ["std"] }
// rustls = "0.23"          # re-exported by tokio-rustls too, but explicit is fine
// webpki-roots = "0.26"     # only needed for client-side default trust roots
// ```
//
// **Loading certs/keys (server side)**
//
// ```rust
// use rustls_pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
// use std::path::Path;
//
// fn load_certs(path: impl AsRef<Path>) -> std::io::Result<Vec<CertificateDer<'static>>> {
//     CertificateDer::pem_file_iter(path)
//         .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
//         .collect::<Result<Vec<_>, _>>()
//         .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
// }
//
// fn load_key(path: impl AsRef<Path>) -> std::io::Result<PrivateKeyDer<'static>> {
//     PrivateKeyDer::from_pem_file(path)
//         .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
// }
// ```
//
// **Building a `TlsAcceptor` and wrapping a Tokio stream**
//
// ```rust
// use std::sync::Arc;
// use tokio::net::TcpListener;
// use tokio_rustls::TlsAcceptor;
// use tokio_rustls::rustls::ServerConfig;
//
// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     let certs = load_certs("certs/server.pem")?;
//     let key = load_key("certs/server.key.pem")?;
//
//     let config = ServerConfig::builder()
//         .with_no_client_auth()
//         .with_single_cert(certs, key)?;
//
//     let acceptor = TlsAcceptor::from(Arc::new(config));
//     let listener = TcpListener::bind("0.0.0.0:8443").await?;
//
//     loop {
//         let (tcp_stream, _peer) = listener.accept().await?;
//         let acceptor = acceptor.clone();
//
//         tokio::spawn(async move {
//             match acceptor.accept(tcp_stream).await {
//                 Ok(mut tls_stream) => {
//                     use tokio::io::{AsyncReadExt, AsyncWriteExt};
//                     let mut buf = [0u8; 1024];
//                     if let Ok(n) = tls_stream.read(&mut buf).await {
//                         let _ = tls_stream.write_all(&buf[..n]).await;
//                     }
//                 }
//                 Err(e) => eprintln!("TLS accept error: {e}"),
//             }
//         });
//     }
// }
// ```
//
// **Client side (loading a custom root CA + connecting)**
//
// ```rust
// use std::sync::Arc;
// use tokio::net::TcpStream;
// use tokio_rustls::TlsConnector;
// use tokio_rustls::rustls::{ClientConfig, RootCertStore};
// use rustls_pki_types::{CertificateDer, ServerName, pem::PemObject};
//
// async fn connect(host: &str, port: u16, ca_path: &str) -> anyhow::Result<()> {
//     let mut roots = RootCertStore::empty();
//     for cert in CertificateDer::pem_file_iter(ca_path)? {
//         roots.add(cert?)?;
//     }
//
//     let config = ClientConfig::builder()
//         .with_root_certificates(roots)
//         .with_no_client_auth();
//
//     let connector = TlsConnector::from(Arc::new(config));
//     let tcp_stream = TcpStream::connect((host, port)).await?;
//     let server_name = ServerName::try_from(host.to_string())?;
//
//     let mut tls_stream = connector.connect(server_name, tcp_stream).await?;
//
//     use tokio::io::AsyncWriteExt;
//     tls_stream.write_all(b"hello").await?;
//     Ok(())
// }
// ```
//
// Key points about the "newest" setup:
//
// - **No `rustls-pemfile` dependency needed** — `rustls-pki-types::pem::PemObject` (bring the trait into scope) gives you `from_pem_file`, `pem_file_iter`, `from_pem_slice`, `pem_slice_iter`, `from_pem_reader`, etc. directly on `CertificateDer`, `PrivateKeyDer`, `CertificateRevocationListDer`, etc.
// - `PrivateKeyDer::from_pem_file` auto-detects PKCS#8, PKCS#1 (RSA), or SEC1 (EC) key types — no need to pick a specific loader function anymore.
// - If you just want the standard Mozilla/webpki trust anchors instead of a custom CA file, use the `webpki-roots` crate: `RootCertStore { roots: webpki_roots::TLS_SERVER_ROOTS.into() }` instead of parsing a PEM.
// - `tokio-rustls` re-exports `rustls` (`tokio_rustls::rustls::...`), so you often don't need a separate `rustls` dependency at all unless you want types not re-exported.
// - Watch your `rustls` crypto provider — as of `rustls` 0.23 you need either the `ring` or `aws-lc-rs` feature enabled (one is default depending on how you pulled it in); if you get a runtime panic about "no process-level CryptoProvider," call `rustls::crypto::ring::default_provider().install_default()` (or the aws-lc-rs equivalent) once at startup.