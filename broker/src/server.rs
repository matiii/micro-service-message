use std::fs::File;
use std::io::BufReader;
use anyhow::Context;
use rustls::pki_types::CertificateDer;
use rustls_pemfile::certs;
use tokio_rustls::rustls::{  }

pub struct Server {}

impl Server {
    pub fn new(configuration: ServerConfiguration) -> Self {

    }

    pub async fn run(self) -> anyhow::Result<()> {

    }

    fn load_certificates(path: &str) -> anyhow::Result<Vec<CertificateDer>> {
        let file = File::open(path).context(format!("Cannot open certificate file in: '{}' path", path))?; // todo: async version
        let mut reader = BufReader::new(file);
        let certs = certs(&mut reader)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .context(format!("Failed to read certificate file in: '{}'", path))?;

        Ok(certs)
    }


}

pub struct ServerConfiguration {
    host: String,
    port: u16,
    certificate_path: String,
    certificate_password: String,
}

impl ServerConfiguration {
    pub fn new(host: String, port: u16, certificate_path: String, certificate_password: String) -> Self {
        Self {
            host,
            port,
            certificate_path,
            certificate_password,
        }
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn certificate_path(&self) -> &str {
        &self.certificate_path
    }

    pub fn certificate_password(&self) -> &str {
        &self.certificate_password
    }
}


// use rustls_pemfile::{certs, pkcs8_private_keys};
// use std::fs::File;
// use std::io::BufReader;
// use std::sync::Arc;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use tokio::net::TcpListener;
// use tokio_rustls::rustls::{self, Certificate, PrivateKey};
// use tokio_rustls::TlsAcceptor;
//
// fn load_certs(path: &str) -> Vec<Certificate> {
//     let file = File::open(path).unwrap_or_else(|e| panic!("open {path}: {e}"));
//     let mut reader = BufReader::new(file);
//     certs(&mut reader)
//         .expect("failed to parse cert.pem")
//         .into_iter()
//         .map(Certificate)
//         .collect()
// }
//
// fn load_key(path: &str) -> PrivateKey {
//     let file = File::open(path).unwrap_or_else(|e| panic!("open {path}: {e}"));
//     let mut reader = BufReader::new(file);
//     let mut keys = pkcs8_private_keys(&mut reader).expect("failed to parse key.pem");
//     assert!(!keys.is_empty(), "no private key found in {path}");
//     PrivateKey(keys.remove(0))
// }
//
// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     let certs = load_certs("cert.pem");
//     let key = load_key("key.pem");
//
//     let tls_config = rustls::ServerConfig::builder()
//         .with_safe_defaults()
//         .with_no_client_auth()
//         .with_single_cert(certs, key)?;
//     let acceptor = TlsAcceptor::from(Arc::new(tls_config));
//
//     let addr = "127.0.0.1:7878";
//     let listener = TcpListener::bind(addr).await?;
//     println!("[server] listening on {addr} (TLS)");
//
//     loop {
//         let (tcp_stream, peer) = listener.accept().await?;
//         let acceptor = acceptor.clone();
//
//         tokio::spawn(async move {
//             println!("[server] connection from {peer}");
//
//             let mut tls_stream = match acceptor.accept(tcp_stream).await {
//                 Ok(s) => s,
//                 Err(e) => {
//                     eprintln!("[server] TLS handshake failed with {peer}: {e}");
//                     return;
//                 }
//             };
//
//             let mut buf = vec![0u8; 4096];
//             loop {
//                 let n = match tls_stream.read(&mut buf).await {
//                     Ok(0) => {
//                         println!("[server] {peer} disconnected");
//                         return;
//                     }
//                     Ok(n) => n,
//                     Err(e) => {
//                         eprintln!("[server] read error from {peer}: {e}");
//                         return;
//                     }
//                 };
//
//                 let msg = String::from_utf8_lossy(&buf[..n]);
//                 println!("[server] received from {peer}: {msg}");
//
//                 let reply = format!("echo: {msg}");
//                 if let Err(e) = tls_stream.write_all(reply.as_bytes()).await {
//                     eprintln!("[server] write error to {peer}: {e}");
//                     return;
//                 }
//             }
//         });
//     }
// }