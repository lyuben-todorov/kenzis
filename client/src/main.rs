use quinn::{Endpoint, NewConnection, ClientConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use rustls::ServerCertVerified;

static SERVER_NAME: &str = "localhost";

fn client_addr() -> SocketAddr {
    "127.0.0.1:5000".parse::<SocketAddr>().unwrap()
}

fn server_addr() -> SocketAddr {
    "127.0.0.1:4433".parse::<SocketAddr>().unwrap()
}

struct SkipCertificationVerification;


impl rustls::ServerCertVerifier for SkipCertificationVerification {
    fn verify_server_cert(
        &self,
        _roots: &rustls::RootCertStore,
        _presented_certs: &[rustls::Certificate],
        _dns_name: webpki::DNSNameRef,
        _ocsp_response: &[u8],
    ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
        Ok(ServerCertVerified::assertion())
    }
}

pub fn insecure() -> ClientConfig {
    let mut cfg = quinn::ClientConfigBuilder::default().build();

    // Get a mutable reference to the 'crypto' config in the 'client config'..
    let tls_cfg: &mut rustls::ClientConfig =
        std::sync::Arc::get_mut(&mut cfg.crypto).unwrap();

    // Change the certification verifier.
    // This is only available when compiled with 'dangerous_configuration' feature.
    tls_cfg
        .dangerous()
        .set_certificate_verifier(Arc::new(SkipCertificationVerification));
    cfg
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut endpoint_builder = Endpoint::builder();
    endpoint_builder.default_client_config(insecure());
    // Bind this endpoint to a UDP socket on the given client address.
    let (endpoint, _) = endpoint_builder.bind(&client_addr())?;

    // Connect to the server passing in the server name which is supposed to be in the server certificate.
    let connection: NewConnection = endpoint
        .connect(&server_addr(), SERVER_NAME)?
        .await?;

    // Start transferring, receiving data, see data transfer page.
    Ok(())
}
