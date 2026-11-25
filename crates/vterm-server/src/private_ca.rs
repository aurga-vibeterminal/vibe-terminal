use std::collections::BTreeSet;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::sign::CertifiedKey;
use tokio_rustls::TlsAcceptor;
use tracing::warn;
use vterm_common::private_ca::{
    certificate_info, generate_root_ca, issue_leaf_certificate, leaf_certificate_needs_reissue,
    normalize_san, normalize_sans, CertificateInfo,
};

const TLS_RENEWAL_WINDOW_SECS: u64 = 30 * 24 * 60 * 60;

pub(crate) struct PrivateCaManager {
    root_cert_path: PathBuf,
    root_key_path: PathBuf,
    leaf_cert_path: PathBuf,
    leaf_key_path: PathBuf,
    certified_key: Arc<RwLock<Option<Arc<CertifiedKey>>>>,
    issue_lock: Mutex<()>,
    last_error: RwLock<Option<String>>,
    managed_identity_active: AtomicBool,
}

impl PrivateCaManager {
    pub(crate) fn new(directory: &Path) -> Self {
        Self {
            root_cert_path: directory.join("root-ca.pem"),
            root_key_path: directory.join("root-ca.key"),
            leaf_cert_path: directory.join("server.crt"),
            leaf_key_path: directory.join("server.key"),
            certified_key: Arc::new(RwLock::new(None)),
            issue_lock: Mutex::new(()),
            last_error: RwLock::new(None),
            managed_identity_active: AtomicBool::new(false),
        }
    }

    pub(crate) fn ensure_certificate(&self, extra_sans: &[String]) -> anyhow::Result<()> {
        let _guard = self.issue_lock.lock().unwrap();
        let result = self.ensure_certificate_locked(extra_sans);
        self.record_result(&result);
        result
    }

    pub(crate) fn reissue_certificate(&self, extra_sans: &[String]) -> anyhow::Result<()> {
        let _guard = self.issue_lock.lock().unwrap();
        let result = (|| {
            generate_root_ca(
                &self.root_cert_path,
                &self.root_key_path,
                "Vibe Terminal Root CA",
            )?;
            issue_leaf_certificate(
                &self.root_cert_path,
                &self.root_key_path,
                &self.leaf_cert_path,
                &self.leaf_key_path,
                "Vibe Terminal",
                &combined_sans(extra_sans)?,
            )?;
            self.reload_identity()
        })();
        self.record_result(&result);
        result
    }

    pub(crate) fn tls_acceptor(&self) -> anyhow::Result<TlsAcceptor> {
        let resolver = Arc::new(ReloadableCertificate {
            certified_key: Arc::clone(&self.certified_key),
        });
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_cert_resolver(resolver);
        self.managed_identity_active.store(true, Ordering::Release);
        Ok(TlsAcceptor::from(Arc::new(config)))
    }

    pub(crate) fn root_ca_pem(&self) -> anyhow::Result<String> {
        Ok(std::fs::read_to_string(&self.root_cert_path)?)
    }

    pub(crate) fn root_certificate_info(&self) -> Option<CertificateInfo> {
        certificate_info(&self.root_cert_path).ok()
    }

    pub(crate) fn leaf_certificate_info(&self) -> Option<CertificateInfo> {
        certificate_info(&self.leaf_cert_path).ok()
    }

    pub(crate) fn last_error(&self) -> Option<String> {
        self.last_error.read().unwrap().clone()
    }

    pub(crate) fn managed_identity_active(&self) -> bool {
        self.managed_identity_active.load(Ordering::Acquire)
    }

    fn ensure_certificate_locked(&self, extra_sans: &[String]) -> anyhow::Result<()> {
        let root_was_complete = self.root_cert_path.exists() && self.root_key_path.exists();
        generate_root_ca(
            &self.root_cert_path,
            &self.root_key_path,
            "Vibe Terminal Root CA",
        )?;
        let sans = combined_sans(extra_sans)?;
        if !root_was_complete
            || leaf_certificate_needs_reissue(&self.leaf_cert_path, &sans, TLS_RENEWAL_WINDOW_SECS)
        {
            issue_leaf_certificate(
                &self.root_cert_path,
                &self.root_key_path,
                &self.leaf_cert_path,
                &self.leaf_key_path,
                "Vibe Terminal",
                &sans,
            )?;
        }
        self.reload_identity()
    }

    fn reload_identity(&self) -> anyhow::Result<()> {
        let cert_file = std::fs::File::open(&self.leaf_cert_path)?;
        let certs =
            rustls_pemfile::certs(&mut BufReader::new(cert_file)).collect::<Result<Vec<_>, _>>()?;
        let key_file = std::fs::File::open(&self.leaf_key_path)?;
        let key = rustls_pemfile::private_key(&mut BufReader::new(key_file))?
            .ok_or_else(|| anyhow::anyhow!("Managed TLS private key is missing"))?;
        let signing_key = rustls::crypto::ring::sign::any_supported_type(&key)?;
        *self.certified_key.write().unwrap() =
            Some(Arc::new(CertifiedKey::new(certs, signing_key)));
        Ok(())
    }

    fn record_result(&self, result: &anyhow::Result<()>) {
        *self.last_error.write().unwrap() = result.as_ref().err().map(|error| error.to_string());
        if let Err(error) = result {
            warn!("Private CA certificate update failed: {}", error);
        }
    }
}

#[derive(Debug)]
struct ReloadableCertificate {
    certified_key: Arc<RwLock<Option<Arc<CertifiedKey>>>>,
}

impl ResolvesServerCert for ReloadableCertificate {
    fn resolve(&self, _client_hello: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
        self.certified_key.read().unwrap().clone()
    }
}

pub(crate) fn automatic_sans() -> Vec<String> {
    let mut values = BTreeSet::from([
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ]);
    if let Ok(hostname) = normalize_san(&gethostname::gethostname().to_string_lossy()) {
        values.insert(hostname);
    }
    if let Ok(interfaces) = if_addrs::get_if_addrs() {
        for interface in interfaces {
            let address = interface.ip();
            if interface.is_loopback() || address.is_unspecified() || address.is_multicast() {
                continue;
            }
            values.insert(address.to_string());
        }
    }
    values.into_iter().collect()
}

pub(crate) fn combined_sans(extra_sans: &[String]) -> anyhow::Result<Vec<String>> {
    Ok(normalize_sans(
        automatic_sans()
            .into_iter()
            .chain(extra_sans.iter().cloned())
            .collect::<Vec<_>>(),
    )?)
}

pub(crate) fn normalized_extra_sans(extra_sans: &[String]) -> anyhow::Result<Vec<String>> {
    normalize_sans(extra_sans)
}

pub(crate) fn managed_tls_directory() -> anyhow::Result<PathBuf> {
    let directory = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("could not determine home directory for TLS certificate"))?
        .join(".vibeterminal")
        .join("certs")
        .join("vterm-server");
    std::fs::create_dir_all(&directory)?;
    Ok(directory)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reissue_preserves_root_and_replaces_the_live_leaf() {
        let directory = tempfile::tempdir().unwrap();
        let manager = PrivateCaManager::new(directory.path());
        manager.ensure_certificate(&[]).unwrap();
        let root = manager.root_certificate_info().unwrap();
        let first_leaf = manager.certified_key.read().unwrap().clone().unwrap();

        manager
            .reissue_certificate(&["server.home.arpa".to_string()])
            .unwrap();

        assert_eq!(
            manager.root_certificate_info().unwrap().fingerprint_sha256,
            root.fingerprint_sha256
        );
        assert!(manager
            .leaf_certificate_info()
            .unwrap()
            .sans
            .contains(&"server.home.arpa".to_string()));
        let second_leaf = manager.certified_key.read().unwrap().clone().unwrap();
        assert!(!Arc::ptr_eq(&first_leaf, &second_leaf));
        assert!(manager.last_error().is_none());
    }

    #[test]
    fn incomplete_root_forces_a_new_leaf_chain() {
        let directory = tempfile::tempdir().unwrap();
        let manager = PrivateCaManager::new(directory.path());
        manager.ensure_certificate(&[]).unwrap();
        let old_root = manager.root_certificate_info().unwrap().fingerprint_sha256;
        std::fs::remove_file(&manager.root_key_path).unwrap();

        manager.ensure_certificate(&[]).unwrap();

        assert_ne!(
            manager.root_certificate_info().unwrap().fingerprint_sha256,
            old_root
        );
        assert!(manager.certified_key.read().unwrap().is_some());
    }
}
