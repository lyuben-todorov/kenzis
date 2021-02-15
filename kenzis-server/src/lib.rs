use std::path::PathBuf;
use std::net::SocketAddr;
use structopt::{self, StructOpt};
use std::{fs, io};
use log::{info};
use anyhow::{Result, Context};
use quinn::ServerConfigBuilder;
use rustls::ServerCertVerified;

#[derive(StructOpt, Debug)]
#[structopt(name = "server")]
pub struct Opt {
    /// TLS private key in PEM format
    #[structopt(parse(from_os_str), short = "k", long = "key", requires = "cert")]
    pub key: Option<PathBuf>,
    /// TLS certificate in PEM format
    #[structopt(parse(from_os_str), short = "c", long = "cert", requires = "key")]
    pub cert: Option<PathBuf>,
    /// Enable stateless retries
    #[structopt(long = "stateless-retry")]
    pub stateless_retry: bool,
    /// Address to listen on
    #[structopt(long = "listen", default_value = "[::1]:4433")]
    pub listen: SocketAddr,
}

pub fn fix_certs(options: &Opt, server_config: &mut ServerConfigBuilder) -> Result<()> {
    if let (Some(key_path), Some(cert_path)) = (&options.key, &options.cert) {
        log::info!("b");
        let key = fs::read(key_path).context("failed to read private key")?;
        let key = if key_path.extension().map_or(false, |x| x == "der") {
            quinn::PrivateKey::from_der(&key)?
        } else {
            quinn::PrivateKey::from_pem(&key)?
        };
        let cert_chain = fs::read(cert_path).context("failed to read certificate chain")?;
        let cert_chain = if cert_path.extension().map_or(false, |x| x == "der") {
            quinn::CertificateChain::from_certs(quinn::Certificate::from_der(&cert_chain))
        } else {
            quinn::CertificateChain::from_pem(&cert_chain)?
        };
        server_config.certificate(cert_chain, key)?;
        info!("parsed certificates");
    } else {
        let dirs = directories_next::ProjectDirs::from("org", "quinn", "quinn-examples").unwrap();
        let path = dirs.data_local_dir();
        let cert_path = path.join("cert.der");
        let key_path = path.join("key.der");
        let (cert, key) = match fs::read(&cert_path).and_then(|x| Ok((x, fs::read(&key_path)?))) {
            Ok(x) => {
                info!("read certs from disk");
                x
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                info!("generating self-signed certificate");
                let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
                let key = cert.serialize_private_key_der();
                let cert = cert.serialize_der().unwrap();
                fs::create_dir_all(&path).context("failed to create certificate directory")?;
                fs::write(&cert_path, &cert).context("failed to write certificate")?;
                fs::write(&key_path, &key).context("failed to write private key")?;
                (cert, key)
            }
            Err(e) => {
                panic!("failed to read certificate: {}", e);
            }
        };
        let key = quinn::PrivateKey::from_der(&key)?;
        let cert = quinn::Certificate::from_der(&cert)?;
        server_config.certificate(quinn::CertificateChain::from_certs(vec![cert]), key)?;
    }
    Ok(())
}

