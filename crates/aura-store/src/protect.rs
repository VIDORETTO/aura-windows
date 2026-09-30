//! Protection of the vault data key. On Windows the key is wrapped with DPAPI
//! (current-user scope), so only the same Windows user can unwrap it.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProtectError {
    #[error("could not protect data: {0}")]
    Protect(String),
    #[error("could not unprotect data (different user or corrupted profile): {0}")]
    Unprotect(String),
}

pub trait SecretProtector: Send + Sync {
    fn protect(&self, plaintext: &[u8]) -> Result<Vec<u8>, ProtectError>;
    fn unprotect(&self, protected: &[u8]) -> Result<Vec<u8>, ProtectError>;
}

/// Test/CI adapter: XORs with a fixed pad and a marker. Not secure; never used
/// in production builds of the desktop app.
#[derive(Debug, Clone)]
pub struct StaticKeyProtector {
    pad: [u8; 32],
}

impl StaticKeyProtector {
    pub fn new(pad: [u8; 32]) -> Self {
        Self { pad }
    }
}

impl Default for StaticKeyProtector {
    fn default() -> Self {
        Self::new([0x5a; 32])
    }
}

const MARKER: &[u8; 4] = b"ATP1";

impl SecretProtector for StaticKeyProtector {
    fn protect(&self, plaintext: &[u8]) -> Result<Vec<u8>, ProtectError> {
        let mut out = MARKER.to_vec();
        out.extend(
            plaintext
                .iter()
                .enumerate()
                .map(|(i, b)| b ^ self.pad[i % 32]),
        );
        Ok(out)
    }

    fn unprotect(&self, protected: &[u8]) -> Result<Vec<u8>, ProtectError> {
        let body = protected
            .strip_prefix(MARKER.as_slice())
            .ok_or_else(|| ProtectError::Unprotect("unknown format".into()))?;
        Ok(body
            .iter()
            .enumerate()
            .map(|(i, b)| b ^ self.pad[i % 32])
            .collect())
    }
}

#[cfg(windows)]
pub use dpapi::DpapiProtector;

#[cfg(windows)]
mod dpapi {
    use super::{ProtectError, SecretProtector};
    use windows::Win32::Foundation::{HLOCAL, LocalFree};
    use windows::Win32::Security::Cryptography::{
        CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
    };
    use windows::core::PCWSTR;

    /// DPAPI current-user protector.
    #[derive(Debug, Default, Clone, Copy)]
    pub struct DpapiProtector;

    fn blob(data: &[u8]) -> CRYPT_INTEGER_BLOB {
        CRYPT_INTEGER_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        }
    }

    unsafe fn take(out: CRYPT_INTEGER_BLOB) -> Vec<u8> {
        // SAFETY: DPAPI returned a LocalAlloc'ed buffer of `cbData` bytes.
        let v = unsafe { std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec() };
        unsafe {
            let _ = LocalFree(Some(HLOCAL(out.pbData as _)));
        }
        v
    }

    impl SecretProtector for DpapiProtector {
        fn protect(&self, plaintext: &[u8]) -> Result<Vec<u8>, ProtectError> {
            let input = blob(plaintext);
            let mut out = CRYPT_INTEGER_BLOB::default();
            unsafe {
                CryptProtectData(
                    &input,
                    PCWSTR::null(),
                    None,
                    None,
                    None,
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut out,
                )
                .map_err(|e| ProtectError::Protect(e.to_string()))?;
                Ok(take(out))
            }
        }

        fn unprotect(&self, protected: &[u8]) -> Result<Vec<u8>, ProtectError> {
            let input = blob(protected);
            let mut out = CRYPT_INTEGER_BLOB::default();
            unsafe {
                CryptUnprotectData(
                    &input,
                    None,
                    None,
                    None,
                    None,
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut out,
                )
                .map_err(|e| ProtectError::Unprotect(e.to_string()))?;
                Ok(take(out))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_protector_round_trips_and_rejects_foreign_data() {
        let p = StaticKeyProtector::default();
        let wrapped = p.protect(b"key").unwrap();
        assert_ne!(&wrapped[4..], b"key");
        assert_eq!(p.unprotect(&wrapped).unwrap(), b"key");
        assert!(p.unprotect(b"garbage").is_err());
    }
}
