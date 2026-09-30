//! Credential Manager blobs are limited to 2560 bytes
//! (`CRED_MAX_CREDENTIAL_BLOB_SIZE`); ChatGPT token sets (access + refresh +
//! id token) exceed that. Large secrets are split: the main entry holds a
//! header `AURA-CHUNKED:v1:<n>` and parts live in `<target>~<i>`.

pub const MAX_BLOB: usize = 2560;
const HEADER: &str = "AURA-CHUNKED:v1:";
pub const PART_SEP: char = '~';

pub fn part_target(target: &str, i: usize) -> String {
    format!("{target}{PART_SEP}{i}")
}

pub fn is_part(target: &str) -> bool {
    target
        .rsplit_once(PART_SEP)
        .is_some_and(|(_, n)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

/// What to write: `(target, bytes)` pairs, main entry first.
pub fn plan_write(target: &str, value: &[u8]) -> Vec<(String, Vec<u8>)> {
    if value.len() <= MAX_BLOB && !value.starts_with(HEADER.as_bytes()) {
        return vec![(target.to_string(), value.to_vec())];
    }
    let parts: Vec<&[u8]> = value.chunks(MAX_BLOB).collect();
    let mut out = vec![(
        target.to_string(),
        format!("{HEADER}{}", parts.len()).into_bytes(),
    )];
    out.extend(
        parts
            .iter()
            .enumerate()
            .map(|(i, p)| (part_target(target, i), p.to_vec())),
    );
    out
}

/// Number of parts if `main` is a chunk header.
pub fn parts_of(main: &[u8]) -> Option<usize> {
    std::str::from_utf8(main)
        .ok()?
        .strip_prefix(HEADER)?
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_values_are_stored_inline() {
        assert_eq!(
            plan_write("Aura/x/y", b"abc"),
            vec![("Aura/x/y".to_string(), b"abc".to_vec())]
        );
    }

    #[test]
    fn large_values_are_split_and_rejoined() {
        let value: Vec<u8> = (0..6000u32).map(|i| (i % 251) as u8).collect();
        let plan = plan_write("Aura/chatgpt/c", &value);
        assert_eq!(plan.len(), 4);
        assert_eq!(parts_of(&plan[0].1), Some(3));
        assert!(
            plan[1..]
                .iter()
                .all(|(t, b)| is_part(t) && b.len() <= MAX_BLOB)
        );
        let joined: Vec<u8> = plan[1..].iter().flat_map(|(_, b)| b.clone()).collect();
        assert_eq!(joined, value);
        assert!(!is_part("Aura/chatgpt/c"));
        assert!(is_part("Aura/chatgpt/c~2"));
    }
}
