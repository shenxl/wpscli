use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD, Engine};
use keyring::Entry;

use crate::error::WpsError;

fn set_secure_parent_permissions(path: &Path) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700));
        }
    }
}

fn write_secure_file(path: &Path, data: &[u8]) -> Result<(), WpsError> {
    set_secure_parent_permissions(path);
    let tmp_path = path.with_extension("tmp");
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true).mode(0o600);
        let mut file = options
            .open(&tmp_path)
            .map_err(|e| WpsError::Auth(format!("failed to open tmp secure file: {e}")))?;
        file.write_all(data)
            .map_err(|e| WpsError::Auth(format!("failed to write tmp secure file: {e}")))?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(&tmp_path, data)
            .map_err(|e| WpsError::Auth(format!("failed to write tmp secure file: {e}")))?;
    }
    std::fs::rename(&tmp_path, path)
        .map_err(|e| WpsError::Auth(format!("failed to replace secure file: {e}")))?;
    Ok(())
}

fn key_file_path(config_dir: &Path) -> PathBuf {
    config_dir.join(".encryption_key")
}

fn decode_key(b64: &str) -> Option<[u8; 32]> {
    let decoded = STANDARD.decode(b64.trim()).ok()?;
    if decoded.len() != 32 {
        return None;
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&decoded);
    Some(key)
}

fn store_key_to_local_file(path: &Path, key: &[u8; 32]) -> Result<(), WpsError> {
    let b64 = STANDARD.encode(key);
    write_secure_file(path, b64.as_bytes())
}

fn get_or_create_key(config_dir: &Path) -> Result<[u8; 32], WpsError> {
    static KEY: OnceLock<[u8; 32]> = OnceLock::new();
    if let Some(k) = KEY.get() {
        return Ok(*k);
    }

    let key_file = key_file_path(config_dir);
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown-user".to_string());
    let entry = Entry::new("wpscli", &username).ok();

    if let Some(ent) = entry.as_ref() {
        if let Ok(b64) = ent.get_password() {
            if let Some(key) = decode_key(&b64) {
                let _ = KEY.set(key);
                return Ok(key);
            }
        }
    }

    if key_file.exists() {
        if let Ok(raw) = std::fs::read_to_string(&key_file) {
            if let Some(key) = decode_key(&raw) {
                if let Some(ent) = entry.as_ref() {
                    let _ = ent.set_password(raw.trim());
                }
                let _ = KEY.set(key);
                return Ok(key);
            }
        }
    }

    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    let b64 = STANDARD.encode(key);

    if let Some(ent) = entry.as_ref() {
        let _ = ent.set_password(&b64);
    }
    let _ = store_key_to_local_file(&key_file, &key);
    let _ = KEY.set(key);
    Ok(key)
}

pub fn encrypt_json(config_dir: &Path, plaintext_json: &str) -> Result<Vec<u8>, WpsError> {
    let key = get_or_create_key(config_dir)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| WpsError::Auth(format!("failed to init cipher: {e}")))?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext_json.as_bytes())
        .map_err(|e| WpsError::Auth(format!("failed to encrypt data: {e}")))?;
    let mut out = nonce.to_vec();
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

pub fn decrypt_json(config_dir: &Path, encrypted: &[u8]) -> Result<String, WpsError> {
    if encrypted.len() < 12 {
        return Err(WpsError::Auth("encrypted data is too short".to_string()));
    }
    let key = get_or_create_key(config_dir)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| WpsError::Auth(format!("failed to init cipher: {e}")))?;
    let nonce = Nonce::from_slice(&encrypted[..12]);
    let plaintext = cipher
        .decrypt(nonce, &encrypted[12..])
        .map_err(|_| {
            WpsError::Auth(
                "failed to decrypt secure file (possibly created on another machine)".to_string(),
            )
        })?;
    String::from_utf8(plaintext)
        .map_err(|e| WpsError::Auth(format!("secure file is not valid UTF-8: {e}")))
}

pub fn save_encrypted_json(
    config_dir: &Path,
    target_file: &Path,
    plaintext_json: &str,
) -> Result<(), WpsError> {
    let encrypted = encrypt_json(config_dir, plaintext_json)?;
    write_secure_file(target_file, &encrypted)
}

pub fn load_encrypted_json(config_dir: &Path, target_file: &Path) -> Result<String, WpsError> {
    let raw = std::fs::read(target_file)
        .map_err(|e| WpsError::Auth(format!("failed to read secure file: {e}")))?;
    decrypt_json(config_dir, &raw)
}
