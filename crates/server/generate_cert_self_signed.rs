use aeronet_webtransport::wtransport::tls::Sha256DigestFmt;
use lightyear::prelude::Identity;

#[path = "./src/certificate.rs"]
mod certificate;

#[tokio::main]
async fn main() {
    let settings = certificate::WebTransportCertificateSettings::default();
    let identity = Identity::from(&settings);
    let cert = identity.certificate_chain();
    let digest = cert.as_slice()[0].hash();
    let digest = digest.fmt(Sha256DigestFmt::DottedHex).replace(":", "");

    std::fs::write("digest.txt", digest).expect("could not write digest.");
    cert.store_pemfile("cert.pem")
        .await
        .expect("failed to write certificate.");
    identity
        .private_key()
        .store_secret_pemfile("key.pem")
        .await
        .expect("failed to write private key.");
}
