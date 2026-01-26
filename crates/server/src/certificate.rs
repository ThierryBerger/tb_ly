use async_compat::Compat;
use bevy::tasks::IoTaskPool;
use lightyear::prelude::Identity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WebTransportCertificateSettings {
    /// Generate a self-signed certificate, with given SANs list to add to the certifictate
    /// eg: ["example.com", "*.gameserver.example.org", "10.1.2.3", "::1"]
    AutoSelfSigned(Vec<String>),
    /// Load certificate pem files from disk
    FromFile {
        /// Path to cert .pem file
        cert: String,
        /// Path to private key .pem file
        key: String,
    },
}

impl Default for WebTransportCertificateSettings {
    fn default() -> Self {
        let sans = vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            "::1".to_string(),
        ];
        WebTransportCertificateSettings::AutoSelfSigned(sans)
    }
}

impl From<&WebTransportCertificateSettings> for Identity {
    fn from(wt: &WebTransportCertificateSettings) -> Identity {
        match wt {
            WebTransportCertificateSettings::AutoSelfSigned(sans) => {
                // In addition to and Subject Alternate Names (SAN) added via the config,
                // we add the public ip and domain for edgegap, if detected, and also
                // any extra values specified via the SELF_SIGNED_SANS environment variable.
                let mut sans = sans.clone();
                // Are we running on edgegap?
                if let Ok(public_ip) = std::env::var("ARBITRIUM_PUBLIC_IP") {
                    println!("🔐 SAN += ARBITRIUM_PUBLIC_IP: {public_ip}");
                    sans.push(public_ip);
                    sans.push("*.pr.edgegap.net".to_string());
                }
                // generic env to add domains and ips to SAN list:
                // SELF_SIGNED_SANS="example.org,example.com,127.1.1.1"
                if let Ok(san) = std::env::var("SELF_SIGNED_SANS") {
                    println!("🔐 SAN += SELF_SIGNED_SANS: {san}");
                    sans.extend(san.split(',').map(|s| s.to_string()));
                }
                println!("🔐 Generating self-signed certificate with SANs: {sans:?}");
                let identity = Identity::self_signed(sans).unwrap();
                let digest = identity.certificate_chain().as_slice()[0].hash();
                println!("🔐 Certificate digest: {digest}");
                identity
            }
            WebTransportCertificateSettings::FromFile {
                cert: cert_pem_path,
                key: private_key_pem_path,
            } => {
                println!(
                    "Reading certificate PEM files:\n * cert: {cert_pem_path}\n * key: {private_key_pem_path}",
                );
                // this is async because we need to load the certificate from io
                // we need async_compat because wtransport expects a tokio reactor
                let identity = IoTaskPool::get()
                    .scope(|s| {
                        s.spawn(Compat::new(async {
                            Identity::load_pemfiles(cert_pem_path, private_key_pem_path)
                                .await
                                .unwrap()
                        }));
                    })
                    .pop()
                    .unwrap();
                let digest = identity.certificate_chain().as_slice()[0].hash();
                println!("🔐 Certificate digest: {digest}");
                identity
            }
        }
    }
}
