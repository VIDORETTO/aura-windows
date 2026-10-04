//! Release configuration checks (010 AC-004, QA-038).

/// Whether the updater public key in `tauri.conf.json` is a real key and not
/// the template placeholder. Without it, update signatures cannot be checked,
/// so the app reports updates as not configured instead of trying.
pub fn updater_key_configured(pubkey: &str) -> bool {
    let k = pubkey.trim();
    !k.is_empty()
        && !k.contains("REPLACE_WITH")
        && k.len() >= 40
        && k.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
}

#[cfg(test)]
mod tests {
    use super::updater_key_configured;

    #[test]
    fn placeholder_and_empty_keys_are_not_configured() {
        assert!(!updater_key_configured(
            "REPLACE_WITH_THE_PUBLIC_KEY_FROM_`pnpm tauri signer generate`"
        ));
        assert!(!updater_key_configured("   "));
        assert!(!updater_key_configured("abc"));
    }

    #[test]
    fn a_minisign_public_key_is_configured() {
        // base64 of "untrusted comment: minisign public key: 0123456789ABCDEF\nRWQ…"
        let key = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDAxMjM0NTY3ODlBQkNERUYKUldRQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUE=";
        assert!(updater_key_configured(key));
    }
}
