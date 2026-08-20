use anyhow::Context;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::pki_types::pem::PemObject;
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;
use x509_cert::der::Decode;

pub fn load_certificates(path: &str) -> anyhow::Result<Vec<CertificateDer<'static>>> {
    CertificateDer::pem_file_iter(path)
        .context(format!("Loading certificate pem file from '{}' path", path))?
        .collect::<Result<Vec<CertificateDer>, _>>()
        .context(format!("Collect results for certificate pem file from '{}' path'", path))
}

pub fn load_private_key(path: &str) -> anyhow::Result<PrivateKeyDer<'static>> {
    let key = PrivateKeyDer::from_pem_file(path)
        .context(format!("Loading private keys pem file from '{}' path'", path))?;

    Ok(key)
}

pub fn get_certificate_subject(stream: &TlsStream<TcpStream>) -> anyhow::Result<String> {
    let (_, connection) = stream.get_ref();
    let certs = connection.peer_certificates()
        .context("Could not get peer certificates")?;
    let leaf = certs.first()
        .context("Could not get first certificate")?;
    let parsed = x509_cert::Certificate::from_der(leaf.as_ref())?;
    let subject = parsed.tbs_certificate().subject().to_string();

    Ok(subject)
}