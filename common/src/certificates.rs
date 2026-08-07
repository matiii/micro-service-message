use anyhow::Context;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::pki_types::pem::PemObject;

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