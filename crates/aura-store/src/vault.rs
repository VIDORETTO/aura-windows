//! Encrypted secrets at rest (AC-016, 001).
//!
//! A random 256-bit data key (DEK) is generated once, wrapped by a
//! [`SecretProtector`] (DPAPI on Windows) and stored in `vault_keys`. Values
//! are encrypted with AES-256-GCM, a random 96-bit nonce per value and the
//! secret id as associated data. Other modules derive purpose-specific keys
//! with HKDF (e.g. `"capture-segment"`).

use crate::protect::{ProtectError, SecretProtector};
use crate::{Store, StoreError, now_secs};
use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use aura_core::Secret;
use hkdf::Hkdf;
use rusqlite::{OptionalExtension, params};
use sha2::Sha256;
use std::sync::Arc;
use thiserror::Error;

pub const NONCE_LEN: usize = 12;
const SEALED_MAGIC: &[u8; 4] = b"AVS1";

#[derive(Debug, Error)]
pub enum VaultError {
    #[error(transparent)]
    Protect(#[from] ProtectError),
    #[error("decryption failed")]
    Decrypt,
    #[error("storage: {0}")]
    Storage(#[from] StoreError),
    #[error("random generator failure")]
    Random,
}

impl From<rusqlite::Error> for VaultError {
    fn from(e: rusqlite::Error) -> Self {
        VaultError::Storage(StoreError::Sqlite(e))
    }
}

#[derive(Clone)]
pub struct Vault {
    store: Store,
    key_id: i64,
    dek: Arc<Secret<Vec<u8>>>,
}

fn random_bytes<const N: usize>() -> Result<[u8; N], VaultError> {
    let mut buf = [0u8; N];
    getrandom::fill(&mut buf).map_err(|_| VaultError::Random)?;
    Ok(buf)
}

impl Vault {
    /// Loads the data key or creates one on first use.
    pub fn open(store: &Store, protector: &dyn SecretProtector) -> Result<Vault, VaultError> {
        let existing: Option<(i64, Vec<u8>)> = store.with_conn(|c| {
            Ok(c.query_row(
                "SELECT id, protected_key FROM vault_keys ORDER BY id DESC LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?)
        })?;
        let (key_id, dek) = match existing {
            Some((id, wrapped)) => (id, protector.unprotect(&wrapped)?),
            None => {
                let dek = random_bytes::<32>()?.to_vec();
                let wrapped = protector.protect(&dek)?;
                let id = store.with_conn(|c| {
                    c.execute(
                        "INSERT INTO vault_keys(protected_key, created_at) VALUES (?1, ?2)",
                        params![wrapped, now_secs()],
                    )?;
                    Ok(c.last_insert_rowid())
                })?;
                (id, dek)
            }
        };
        Ok(Vault {
            store: store.clone(),
            key_id,
            dek: Arc::new(Secret::new(dek)),
        })
    }

    fn cipher_for(key: &[u8]) -> Aes256Gcm {
        Aes256Gcm::new_from_slice(key).expect("32-byte key")
    }

    /// Encrypts and stores `plaintext` under `id`.
    pub fn seal(&self, id: &str, plaintext: &[u8]) -> Result<(), VaultError> {
        let nonce = random_bytes::<NONCE_LEN>()?;
        let ct = Self::cipher_for(self.dek.expose())
            .encrypt(
                &Nonce::from(nonce),
                Payload {
                    msg: plaintext,
                    aad: id.as_bytes(),
                },
            )
            .map_err(|_| VaultError::Decrypt)?;
        self.store.with_conn(|c| {
            c.execute(
                "INSERT INTO secrets(id, key_id, nonce, ciphertext, updated_at) VALUES (?1,?2,?3,?4,?5)
                 ON CONFLICT(id) DO UPDATE SET key_id=excluded.key_id, nonce=excluded.nonce,
                   ciphertext=excluded.ciphertext, updated_at=excluded.updated_at",
                params![id, self.key_id, nonce.to_vec(), ct, now_secs()],
            )?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn open_secret(&self, id: &str) -> Result<Option<Secret<Vec<u8>>>, VaultError> {
        let row: Option<(Vec<u8>, Vec<u8>)> = self.store.with_conn(|c| {
            Ok(c.query_row(
                "SELECT nonce, ciphertext FROM secrets WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?)
        })?;
        let Some((nonce, ct)) = row else {
            return Ok(None);
        };
        let nonce: [u8; NONCE_LEN] = nonce.try_into().map_err(|_| VaultError::Decrypt)?;
        let pt = Self::cipher_for(self.dek.expose())
            .decrypt(
                &Nonce::from(nonce),
                Payload {
                    msg: &ct,
                    aad: id.as_bytes(),
                },
            )
            .map_err(|_| VaultError::Decrypt)?;
        Ok(Some(Secret::new(pt)))
    }

    pub fn delete(&self, id: &str) -> Result<(), VaultError> {
        self.store.with_conn(|c| {
            c.execute("DELETE FROM secrets WHERE id = ?1", [id])?;
            Ok(())
        })?;
        Ok(())
    }

    /// Derives an independent 32-byte key for another purpose.
    pub fn derive_key(&self, context: &str) -> Secret<Vec<u8>> {
        let hk = Hkdf::<Sha256>::new(Some(b"aura-vault-v1"), self.dek.expose());
        let mut okm = vec![0u8; 32];
        hk.expand(context.as_bytes(), &mut okm)
            .expect("32 bytes is a valid HKDF length");
        Secret::new(okm)
    }

    /// Encrypts a standalone blob (e.g. a capture segment) with a derived key.
    /// Output layout: `AVS1 | nonce(12) | ciphertext+tag`.
    pub fn seal_bytes(&self, context: &str, plaintext: &[u8]) -> Result<Vec<u8>, VaultError> {
        seal_with_key(self.derive_key(context).expose(), plaintext)
    }

    pub fn open_bytes(&self, context: &str, sealed: &[u8]) -> Result<Vec<u8>, VaultError> {
        open_with_key(self.derive_key(context).expose(), sealed)
    }
}

/// Seals with an explicit key (used by the worker, which receives the derived
/// key over its private pipe).
pub fn seal_with_key(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, VaultError> {
    let nonce = random_bytes::<NONCE_LEN>()?;
    let ct = Vault::cipher_for(key)
        .encrypt(
            &Nonce::from(nonce),
            Payload {
                msg: plaintext,
                aad: SEALED_MAGIC,
            },
        )
        .map_err(|_| VaultError::Decrypt)?;
    let mut out = Vec::with_capacity(4 + NONCE_LEN + ct.len());
    out.extend_from_slice(SEALED_MAGIC);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ct);
    Ok(out)
}

pub fn open_with_key(key: &[u8], sealed: &[u8]) -> Result<Vec<u8>, VaultError> {
    let body = sealed
        .strip_prefix(SEALED_MAGIC.as_slice())
        .ok_or(VaultError::Decrypt)?;
    if body.len() < NONCE_LEN {
        return Err(VaultError::Decrypt);
    }
    let (nonce, ct) = body.split_at(NONCE_LEN);
    let nonce: [u8; NONCE_LEN] = nonce.try_into().map_err(|_| VaultError::Decrypt)?;
    Vault::cipher_for(key)
        .decrypt(
            &Nonce::from(nonce),
            Payload {
                msg: ct,
                aad: SEALED_MAGIC,
            },
        )
        .map_err(|_| VaultError::Decrypt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protect::StaticKeyProtector;

    #[test]
    fn sealed_value_is_not_in_the_database_file_and_survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("aura.db");
        let protector = StaticKeyProtector::default();
        {
            let store = Store::open(&path).unwrap();
            let vault = Vault::open(&store, &protector).unwrap();
            vault.seal("teste", b"sk-test-SEGREDO-123").unwrap();
        }
        let mut all = Vec::new();
        for entry in std::fs::read_dir(dir.path()).unwrap() {
            all.extend(std::fs::read(entry.unwrap().path()).unwrap());
        }
        let needle = b"sk-test-SEGREDO-123";
        assert!(!all.windows(needle.len()).any(|w| w == needle));
        let store = Store::open(&path).unwrap();
        let vault = Vault::open(&store, &protector).unwrap();
        assert_eq!(
            vault.open_secret("teste").unwrap().unwrap().expose(),
            needle
        );
    }

    #[test]
    fn a_different_protector_cannot_read_existing_secrets() {
        let store = Store::open_in_memory().unwrap();
        Vault::open(&store, &StaticKeyProtector::default())
            .unwrap()
            .seal("x", b"segredo")
            .unwrap();
        let other = StaticKeyProtector::new([1; 32]);
        match Vault::open(&store, &other) {
            Err(_) => {}
            Ok(v) => assert!(matches!(v.open_secret("x"), Err(VaultError::Decrypt))),
        }
    }

    #[test]
    fn tampered_ciphertext_is_rejected_and_id_is_bound() {
        let store = Store::open_in_memory().unwrap();
        let vault = Vault::open(&store, &StaticKeyProtector::default()).unwrap();
        vault.seal("a", b"one").unwrap();
        store
            .with_conn(|c| {
                c.execute("UPDATE secrets SET id = 'b' WHERE id = 'a'", [])?;
                Ok(())
            })
            .unwrap();
        assert!(matches!(vault.open_secret("b"), Err(VaultError::Decrypt)));
    }

    #[test]
    fn bytes_round_trip_with_derived_keys() {
        let store = Store::open_in_memory().unwrap();
        let vault = Vault::open(&store, &StaticKeyProtector::default()).unwrap();
        let sealed = vault.seal_bytes("capture-segment", b"ftyp....").unwrap();
        assert!(!sealed.windows(4).any(|w| w == b"ftyp"));
        assert_eq!(
            vault.open_bytes("capture-segment", &sealed).unwrap(),
            b"ftyp...."
        );
        assert!(vault.open_bytes("other-context", &sealed).is_err());
    }

    #[test]
    fn delete_removes_secret() {
        let store = Store::open_in_memory().unwrap();
        let vault = Vault::open(&store, &StaticKeyProtector::default()).unwrap();
        vault.seal("x", b"1").unwrap();
        vault.delete("x").unwrap();
        assert!(vault.open_secret("x").unwrap().is_none());
    }
}
