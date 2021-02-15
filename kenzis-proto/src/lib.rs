mod proto_generated;

pub mod proto {
    pub struct ClientMeta {

    }

}



pub mod security {
    use std::fs;
    use rustls::ServerCertVerified;
    // Implementation of `ServerCertVerifier` that verifies everything as trustworthy.
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

    #[allow(unused)]
    pub const ALPN_QUIC_HTTP: &[&[u8]] = &[b"hq-29"];

}