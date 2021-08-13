use rustls::{RootCertStore, Certificate, ServerCertVerified, TLSError};
use webpki::DNSNameRef;

pub struct CertificateValidator {}

impl rustls::ServerCertVerifier for CertificateValidator {
    fn verify_server_cert(&self, _roots: &RootCertStore,
                          _presented_certs: &[Certificate],
                          _dns_name: DNSNameRef,
                          _ocsp_response: &[u8]) -> Result<ServerCertVerified, TLSError> {
        Ok(rustls::ServerCertVerified::assertion())
    }
}