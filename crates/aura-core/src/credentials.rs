//! Secret storage abstraction. Production uses the Windows Credential Manager
//! (`aura-win`); tests and non-Windows builds use [`MemoryCredentialStore`].
//! Targets are namespaced as `Aura/<area>/<id>`.

use crate::Secret;
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CredentialError {
    #[error("credential store unavailable: {0}")]
    Unavailable(String),
    #[error("credential store error: {0}")]
    Os(String),
}

pub trait CredentialStore: Send + Sync {
    fn put(&self, target: &str, secret: &Secret<String>) -> Result<(), CredentialError>;
    fn get(&self, target: &str) -> Result<Option<Secret<String>>, CredentialError>;
    fn delete(&self, target: &str) -> Result<(), CredentialError>;
    /// Targets starting with `prefix` (used by "apagar meus dados").
    fn list(&self, prefix: &str) -> Result<Vec<String>, CredentialError>;
}

pub fn target(area: &str, id: &str) -> String {
    format!("Aura/{area}/{id}")
}

#[derive(Default)]
pub struct MemoryCredentialStore {
    map: Mutex<HashMap<String, String>>,
}

impl MemoryCredentialStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CredentialStore for MemoryCredentialStore {
    fn put(&self, target: &str, secret: &Secret<String>) -> Result<(), CredentialError> {
        self.map
            .lock()
            .unwrap()
            .insert(target.to_string(), secret.expose().clone());
        Ok(())
    }
    fn get(&self, target: &str) -> Result<Option<Secret<String>>, CredentialError> {
        Ok(self
            .map
            .lock()
            .unwrap()
            .get(target)
            .cloned()
            .map(Secret::new))
    }
    fn delete(&self, target: &str) -> Result<(), CredentialError> {
        self.map.lock().unwrap().remove(target);
        Ok(())
    }
    fn list(&self, prefix: &str) -> Result<Vec<String>, CredentialError> {
        let mut v: Vec<String> = self
            .map
            .lock()
            .unwrap()
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        v.sort();
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_round_trip_and_prefix_listing() {
        let s = MemoryCredentialStore::new();
        s.put(&target("provider", "groq"), &Secret::from("k1"))
            .unwrap();
        s.put(&target("chatgpt", "oaiapp_1"), &Secret::from("k2"))
            .unwrap();
        assert_eq!(s.get("Aura/provider/groq").unwrap().unwrap().expose(), "k1");
        assert_eq!(s.list("Aura/").unwrap().len(), 2);
        s.delete("Aura/provider/groq").unwrap();
        assert!(s.get("Aura/provider/groq").unwrap().is_none());
    }
}
