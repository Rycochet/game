//! Reading, generating and writing certificate files, privately.

use bevy::prelude::*;
use rcgen::{CertificateParams, DnType, KeyPair, PublicKeyData};
use rustls::crypto::ring::default_provider;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::sign::CertifiedKey;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{fs, io};
use time::{Duration, OffsetDateTime};

use super::{
    Current, LASTS, LOCAL_NAMES, RENEW_WITHIN, certificate_facts, key_fingerprint, modified,
};

/// Reads a certificate and its key, checking that they belong together.
pub(crate) fn read_pair(certificate: &Path, key: &Path) -> Result<Current, String> {
    let changed = (modified(certificate), modified(key));
    let chain: Vec<CertificateDer> = CertificateDer::pem_file_iter(certificate)
        .and_then(|items| items.collect())
        .map_err(|error| format!("Can't read {}: {error}", certificate.display()))?;
    let first = chain
        .first()
        .ok_or_else(|| format!("{} has no certificate in it", certificate.display()))?;
    let facts = certificate_facts(first)
        .ok_or_else(|| format!("{} is not a certificate", certificate.display()))?;
    let private_key = PrivateKeyDer::from_pem_file(key)
        .map_err(|error| format!("Can't read {}: {error}", key.display()))?;
    let signing_key = default_provider()
        .key_provider
        .load_private_key(private_key)
        .map_err(|error| format!("Can't use {}: {error}", key.display()))?;
    let pair = CertifiedKey::new(chain, signing_key);
    pair.keys_match().map_err(|_| {
        format!(
            "{} is not the key of {}",
            key.display(),
            certificate.display()
        )
    })?;
    let now = OffsetDateTime::now_utc().unix_timestamp();
    if now > facts.not_after {
        error!(
            "{} has expired: clients will refuse it until it is replaced",
            certificate.display()
        );
    }
    let text = fs::read_to_string(certificate)
        .map_err(|error| format!("Can't read {}: {error}", certificate.display()))?;
    Ok(Current {
        key: Arc::new(pair),
        not_before: facts.not_before,
        not_after: facts.not_after,
        public_pem: only_certificates(&text),
        fingerprint: facts.key_fingerprint,
        changed,
    })
}

/// Just the certificates of a PEM file, which may hold its key as well.
pub(super) fn only_certificates(pem: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for line in pem.lines() {
        let line = line.trim();
        inside |= line == "-----BEGIN CERTIFICATE-----";
        if inside {
            out.push_str(line);
            out.push('\n');
        }
        inside &= line != "-----END CERTIFICATE-----";
    }
    out
}

/// Makes the key once, and the certificate whenever there is none that covers
/// `names` with long enough left. Returns the paths of both.
pub(super) fn generated(
    folder: &Path,
    names: &[String],
    now: OffsetDateTime,
) -> Result<(PathBuf, PathBuf), String> {
    let failed = |error: &dyn std::fmt::Display| {
        format!(
            "Can't set up a certificate in {}: {error}",
            folder.display()
        )
    };
    let certificate_path = folder.join("certificate.pem");
    let key_path = folder.join("key.pem");
    fs::create_dir_all(folder).map_err(|error| failed(&error))?;

    let key = match fs::read_to_string(&key_path) {
        Ok(pem) => KeyPair::from_pem(&pem)
            .map_err(|error| format!("Can't use {}: {error}", key_path.display()))?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let key = KeyPair::generate().map_err(|error| failed(&error))?;
            write_private(&key_path, &key.serialize_pem()).map_err(|error| failed(&error))?;
            key
        }
        Err(error) => return Err(failed(&error)),
    };

    let mut wanted: Vec<String> = LOCAL_NAMES.iter().map(|&name| name.to_owned()).collect();
    wanted.extend(names.iter().cloned());
    wanted.sort();
    wanted.dedup();

    if !certificate_will_do(&certificate_path, &key, &wanted, now) {
        let mut params = CertificateParams::new(wanted).map_err(|error| failed(&error))?;
        params
            .distinguished_name
            .push(DnType::CommonName, "Hall of Worlds server");
        params.not_before = now - Duration::days(1);
        params.not_after = now + LASTS;
        let certificate = params.self_signed(&key).map_err(|error| failed(&error))?;
        fs::write(&certificate_path, certificate.pem()).map_err(|error| failed(&error))?;
        info!("Made a new certificate in {}", folder.display());
    }
    Ok((certificate_path, key_path))
}

/// Whether the certificate on disk can go on being used.
pub(super) fn certificate_will_do(
    path: &Path,
    key: &KeyPair,
    wanted: &[String],
    now: OffsetDateTime,
) -> bool {
    let Ok(der) = CertificateDer::from_pem_file(path) else {
        return false;
    };
    let Some(facts) = certificate_facts(&der) else {
        return false;
    };
    let mut names = facts.names;
    names.sort();
    facts.key_fingerprint == key_fingerprint(&key.subject_public_key_info())
        && names == wanted
        && facts.not_after > (now + RENEW_WITHIN).unix_timestamp()
}

/// Makes a folder that only this user can use (on Unix; on Windows the user's
/// data folder is already private).
pub(crate) fn create_private_folder(folder: &Path) -> io::Result<()> {
    fs::create_dir_all(folder)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(folder, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

/// Writes a private key that only this user can read (on Unix; on Windows the
/// user's data folder is already private).
pub(crate) fn write_private(path: &Path, contents: &str) -> io::Result<()> {
    use std::io::Write;

    let new = path.with_extension("pem.new");
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(&new)?.write_all(contents.as_bytes())?;
    fs::rename(new, path)
}
