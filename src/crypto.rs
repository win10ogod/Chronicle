use std::fs;
use std::path::{Path, PathBuf};

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use anyhow::{Context, Result, anyhow};
use argon2::Argon2;
use base64::Engine as _;
use rand::TryRng as _;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const ENCRYPTION_MAGIC: &[u8] = b"CHRON1";
const NONCE_LEN: usize = 12;

#[derive(Debug, Clone)]
pub enum Crypto {
    Plain,
    Aes256Gcm { key: [u8; 32] },
}

#[derive(Debug, Clone)]
pub struct CryptoInit {
    pub crypto: Crypto,
    pub created_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CryptoConfig {
    version: u8,
    salt_b64: String,
}

impl Crypto {
    pub fn from_env(chronicle_dir: &Path) -> Result<CryptoInit> {
        let key_raw = match std::env::var("CHRONICLE_KEY") {
            Ok(v) if !v.trim().is_empty() => v,
            _ => {
                return Ok(CryptoInit {
                    crypto: Crypto::Plain,
                    created_paths: Vec::new(),
                });
            }
        };

        let mut created_paths = Vec::new();
        let key = derive_key(&key_raw, chronicle_dir, &mut created_paths)?;
        Ok(CryptoInit {
            crypto: Crypto::Aes256Gcm { key },
            created_paths,
        })
    }

    pub fn is_enabled(&self) -> bool {
        matches!(self, Crypto::Aes256Gcm { .. })
    }

    pub fn encrypt_bytes(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let Crypto::Aes256Gcm { key } = self else {
            return Ok(plaintext.to_vec());
        };

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|_| anyhow!("invalid AES-256-GCM key length"))?;
        let mut nonce_bytes = [0u8; NONCE_LEN];
        let mut rng = rand::rngs::SysRng;
        rng.try_fill_bytes(&mut nonce_bytes)
            .map_err(|_| anyhow!("system RNG failed"))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| anyhow!("AES-GCM encryption failed"))?;

        let mut out = Vec::with_capacity(ENCRYPTION_MAGIC.len() + NONCE_LEN + ciphertext.len());
        out.extend_from_slice(ENCRYPTION_MAGIC);
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    pub fn decrypt_bytes(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.starts_with(ENCRYPTION_MAGIC) {
            let Crypto::Aes256Gcm { key } = self else {
                return Err(anyhow!(
                    "encrypted memory detected but `CHRONICLE_KEY` is not set"
                ));
            };

            if data.len() < ENCRYPTION_MAGIC.len() + NONCE_LEN {
                return Err(anyhow!("invalid encrypted payload (too short)"));
            }

            let nonce_start = ENCRYPTION_MAGIC.len();
            let nonce_end = nonce_start + NONCE_LEN;
            let nonce = Nonce::from_slice(&data[nonce_start..nonce_end]);
            let ciphertext = &data[nonce_end..];

            let cipher = Aes256Gcm::new_from_slice(key)
                .map_err(|_| anyhow!("invalid AES-256-GCM key length"))?;
            let plaintext = cipher
                .decrypt(nonce, ciphertext)
                .map_err(|_| anyhow!("AES-GCM decryption failed (bad key or corrupted data)"))?;
            Ok(plaintext)
        } else {
            Ok(data.to_vec())
        }
    }
}

fn derive_key(
    key_raw: &str,
    chronicle_dir: &Path,
    created_paths: &mut Vec<PathBuf>,
) -> Result<[u8; 32]> {
    let key_raw = key_raw.trim();
    if let Some(rest) = key_raw.strip_prefix("base64:") {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(rest.trim())
            .context("decode CHRONICLE_KEY base64")?;
        return bytes_to_key(&bytes);
    }

    if let Some(rest) = key_raw.strip_prefix("hex:") {
        let bytes = hex::decode(rest.trim()).context("decode CHRONICLE_KEY hex")?;
        return bytes_to_key(&bytes);
    }

    // Treat as passphrase and derive a per-repo key via Argon2id + stable salt.
    let config_path = chronicle_dir.join("crypto.yaml");
    let config = load_or_create_crypto_config(&config_path, created_paths)?;
    let salt = base64::engine::general_purpose::STANDARD
        .decode(config.salt_b64)
        .context("decode crypto salt base64")?;

    let mut out = [0u8; 32];
    let argon2 = Argon2::default();
    if argon2
        .hash_password_into(key_raw.as_bytes(), &salt, &mut out)
        .is_ok()
    {
        return Ok(out);
    }

    // Fallback: SHA-256(passphrase || salt). (Shouldn't happen unless Argon2 fails.)
    let mut hasher = Sha256::new();
    hasher.update(key_raw.as_bytes());
    hasher.update(&salt);
    let digest = hasher.finalize();
    bytes_to_key(&digest)
}

fn bytes_to_key(bytes: &[u8]) -> Result<[u8; 32]> {
    if bytes.len() != 32 {
        return Err(anyhow!(
            "CHRONICLE_KEY must decode to 32 bytes (got {})",
            bytes.len()
        ));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(bytes);
    Ok(key)
}

fn load_or_create_crypto_config(
    path: &Path,
    created_paths: &mut Vec<PathBuf>,
) -> Result<CryptoConfig> {
    if let Ok(bytes) = fs::read(path) {
        let cfg: CryptoConfig =
            serde_yaml::from_slice(&bytes).context("parse .chronicle/crypto.yaml")?;
        if cfg.version != 1 {
            return Err(anyhow!("unsupported crypto config version {}", cfg.version));
        }
        return Ok(cfg);
    }

    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| anyhow!("crypto config path has no parent"))?,
    )
    .with_context(|| format!("create chronicle dir for {}", path.display()))?;

    let mut salt = [0u8; 16];
    let mut rng = rand::rngs::SysRng;
    rng.try_fill_bytes(&mut salt)
        .map_err(|_| anyhow!("system RNG failed"))?;
    let salt_b64 = base64::engine::general_purpose::STANDARD.encode(salt);
    let cfg = CryptoConfig {
        version: 1,
        salt_b64,
    };

    let yaml = serde_yaml::to_string(&cfg).context("serialize crypto config")?;
    fs::write(path, yaml).with_context(|| format!("write {}", path.display()))?;
    created_paths.push(path.to_path_buf());
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes_gcm_roundtrip_and_magic() {
        let crypto = Crypto::Aes256Gcm { key: [7u8; 32] };
        let pt = b"hello\n---\nworld";
        let ct = crypto.encrypt_bytes(pt).expect("encrypt");
        assert!(ct.starts_with(ENCRYPTION_MAGIC));
        let rt = crypto.decrypt_bytes(&ct).expect("decrypt");
        assert_eq!(rt, pt);
    }

    #[test]
    fn decrypt_requires_key() {
        let crypto = Crypto::Aes256Gcm { key: [9u8; 32] };
        let ct = crypto.encrypt_bytes(b"secret").expect("encrypt");
        let plain = Crypto::Plain;
        let err = plain.decrypt_bytes(&ct).expect_err("should fail");
        assert!(err.to_string().contains("encrypted memory detected"));
    }
}
