//! `CredentialStore` over Windows Credential Manager (generic credentials,
//! per-user, `CRED_PERSIST_LOCAL_MACHINE` so they do not roam). Values larger
//! than a credential blob are split (see [`crate::chunks`]).

use crate::chunks::{is_part, part_target, parts_of, plan_write};
use crate::{from_wide, to_wide};
use aura_core::credentials::{CredentialError, CredentialStore};
use aura_core::secret::Secret;
use windows::Win32::Foundation::{ERROR_NOT_FOUND, WIN32_ERROR};
use windows::Win32::Security::Credentials::{
    CRED_ENUMERATE_FLAGS, CRED_FLAGS, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC, CREDENTIALW,
    CredDeleteW, CredEnumerateW, CredFree, CredReadW, CredWriteW,
};
use windows::core::{PCWSTR, PWSTR};

#[derive(Default)]
pub struct WinCredentialStore;

fn err(e: windows::core::Error) -> CredentialError {
    CredentialError::Os(e.message().to_string())
}

fn is_not_found(e: &windows::core::Error) -> bool {
    e.code() == WIN32_ERROR(ERROR_NOT_FOUND.0).to_hresult()
}

fn write_raw(target: &str, blob: &[u8]) -> Result<(), CredentialError> {
    let mut name = to_wide(target);
    let mut user = to_wide("Aura");
    let mut blob = blob.to_vec();
    let cred = CREDENTIALW {
        Flags: CRED_FLAGS(0),
        Type: CRED_TYPE_GENERIC,
        TargetName: PWSTR(name.as_mut_ptr()),
        CredentialBlobSize: blob.len() as u32,
        CredentialBlob: blob.as_mut_ptr(),
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        UserName: PWSTR(user.as_mut_ptr()),
        ..Default::default()
    };
    unsafe { CredWriteW(&cred, 0) }.map_err(err)
}

fn read_raw(target: &str) -> Result<Option<Vec<u8>>, CredentialError> {
    let name = to_wide(target);
    let mut out: *mut CREDENTIALW = std::ptr::null_mut();
    match unsafe { CredReadW(PCWSTR(name.as_ptr()), CRED_TYPE_GENERIC, None, &mut out) } {
        Ok(()) => unsafe {
            let c = &*out;
            let bytes = std::slice::from_raw_parts(c.CredentialBlob, c.CredentialBlobSize as usize)
                .to_vec();
            CredFree(out as *const _);
            Ok(Some(bytes))
        },
        Err(e) if is_not_found(&e) => Ok(None),
        Err(e) => Err(err(e)),
    }
}

fn delete_raw(target: &str) -> Result<(), CredentialError> {
    let name = to_wide(target);
    match unsafe { CredDeleteW(PCWSTR(name.as_ptr()), CRED_TYPE_GENERIC, None) } {
        Ok(()) => Ok(()),
        Err(e) if is_not_found(&e) => Ok(()),
        Err(e) => Err(err(e)),
    }
}

fn delete_parts(target: &str, from: usize) -> Result<(), CredentialError> {
    // Parts are contiguous; stop at the first missing one.
    let mut i = from;
    while read_raw(&part_target(target, i))?.is_some() {
        delete_raw(&part_target(target, i))?;
        i += 1;
    }
    Ok(())
}

impl CredentialStore for WinCredentialStore {
    fn put(&self, target: &str, secret: &Secret<String>) -> Result<(), CredentialError> {
        let plan = plan_write(target, secret.expose().as_bytes());
        // Parts first, header last: a crash leaves the old value readable.
        for (t, b) in plan.iter().skip(1) {
            write_raw(t, b)?;
        }
        write_raw(&plan[0].0, &plan[0].1)?;
        delete_parts(target, plan.len().saturating_sub(1))
    }

    fn get(&self, target: &str) -> Result<Option<Secret<String>>, CredentialError> {
        let Some(main) = read_raw(target)? else {
            return Ok(None);
        };
        let bytes = match parts_of(&main) {
            None => main,
            Some(n) => {
                let mut all = Vec::new();
                for i in 0..n {
                    let part = read_raw(&part_target(target, i))?.ok_or_else(|| {
                        CredentialError::Os(format!("missing part {i} of {target}"))
                    })?;
                    all.extend(part);
                }
                all
            }
        };
        String::from_utf8(bytes)
            .map(|s| Some(Secret::new(s)))
            .map_err(|_| CredentialError::Os("credential is not UTF-8".into()))
    }

    fn delete(&self, target: &str) -> Result<(), CredentialError> {
        delete_raw(target)?;
        delete_parts(target, 0)
    }

    fn list(&self, prefix: &str) -> Result<Vec<String>, CredentialError> {
        let filter = to_wide(&format!("{prefix}*"));
        let mut count = 0u32;
        let mut creds: *mut *mut CREDENTIALW = std::ptr::null_mut();
        match unsafe {
            CredEnumerateW(
                PCWSTR(filter.as_ptr()),
                Some(CRED_ENUMERATE_FLAGS(0)),
                &mut count,
                &mut creds,
            )
        } {
            Ok(()) => {}
            Err(e) if is_not_found(&e) => return Ok(vec![]),
            Err(e) => return Err(err(e)),
        }
        let mut out = Vec::new();
        unsafe {
            for i in 0..count as usize {
                let c = &**creds.add(i);
                let name = from_wide(std::slice::from_raw_parts(
                    c.TargetName.0,
                    c.TargetName.len() + 1,
                ));
                if name.starts_with(prefix) && !is_part(&name) {
                    out.push(name);
                }
            }
            CredFree(creds as *const _);
        }
        out.sort();
        Ok(out)
    }
}
