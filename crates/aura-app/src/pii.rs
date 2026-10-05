//! Personal-data redaction for meetings (038): CPF, CNPJ, card numbers,
//! e-mails and phone numbers become placeholders before a transcript is shown
//! to the model, when the user turned the option on. Numbers are only
//! replaced when they pass the check digits (CPF, CNPJ, Luhn) or look like a
//! phone, so dates, prices and ids are left alone.

use regex::Regex;
use std::sync::OnceLock;

fn email() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9\-]+(?:\.[A-Za-z0-9\-]+)*\.[A-Za-z]{2,}").unwrap()
    })
}

fn number() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"[+(]?\d(?:[\d .\-()/]{6,}\d)").unwrap())
}

fn digits(s: &str) -> Vec<u32> {
    s.chars().filter_map(|c| c.to_digit(10)).collect()
}

fn cpf_ok(d: &[u32]) -> bool {
    if d.len() != 11 || d.iter().all(|x| *x == d[0]) {
        return false;
    }
    let check = |n: usize| {
        let sum: u32 = d[..n]
            .iter()
            .enumerate()
            .map(|(i, x)| x * (n as u32 + 1 - i as u32))
            .sum();
        let r = (sum * 10) % 11;
        if r == 10 { 0 } else { r }
    };
    check(9) == d[9] && check(10) == d[10]
}

fn cnpj_ok(d: &[u32]) -> bool {
    if d.len() != 14 || d.iter().all(|x| *x == d[0]) {
        return false;
    }
    let check = |n: usize| {
        let w: &[u32] = if n == 12 {
            &[5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]
        } else {
            &[6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]
        };
        let sum: u32 = d[..n].iter().zip(w).map(|(x, w)| x * w).sum();
        let r = sum % 11;
        if r < 2 { 0 } else { 11 - r }
    };
    check(12) == d[12] && check(13) == d[13]
}

fn luhn_ok(d: &[u32]) -> bool {
    if !(13..=19).contains(&d.len()) || d.iter().all(|x| *x == d[0]) {
        return false;
    }
    let sum: u32 = d
        .iter()
        .rev()
        .enumerate()
        .map(|(i, x)| {
            if i % 2 == 1 {
                let y = x * 2;
                if y > 9 { y - 9 } else { y }
            } else {
                *x
            }
        })
        .sum();
    sum % 10 == 0
}

/// `123.456.789-00` / `12.345.678/0001-90` shapes are documents, never phones.
fn document_shape(raw: &str) -> bool {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"^\d{3}\.\d{3}\.\d{3}-\d{2}$|^\d{2}\.\d{3}\.\d{3}/\d{4}-\d{2}$").unwrap()
    })
    .is_match(raw)
}

fn phone_like(d: &[u32], raw: &str) -> bool {
    matches!(d.len(), 10..=13) && !d.iter().all(|x| *x == d[0]) && !document_shape(raw)
}

/// Replaces personal data in `text` with `[CPF]`, `[CNPJ]`, `[CARTÃO]`,
/// `[E-MAIL]` and `[TELEFONE]`.
pub fn redact(text: &str) -> String {
    let text = email().replace_all(text, "[E-MAIL]");
    number()
        .replace_all(&text, |c: &regex::Captures| {
            let raw = &c[0];
            let d = digits(raw);
            if cpf_ok(&d) {
                "[CPF]".to_string()
            } else if cnpj_ok(&d) {
                "[CNPJ]".to_string()
            } else if luhn_ok(&d) {
                "[CARTÃO]".to_string()
            } else if phone_like(&d, raw) {
                "[TELEFONE]".to_string()
            } else {
                raw.to_string()
            }
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpf_cnpj_and_cards_are_found_only_with_valid_check_digits() {
        assert_eq!(
            redact("meu CPF é 529.982.247-25, ok?"),
            "meu CPF é [CPF], ok?"
        );
        assert_eq!(redact("cpf 52998224725"), "cpf [CPF]");
        assert_eq!(redact("CNPJ 11.222.333/0001-81"), "CNPJ [CNPJ]");
        assert_eq!(redact("cartão 4539 1488 0343 6467"), "cartão [CARTÃO]");
        // Wrong check digits stay as they are (could be an order number).
        assert_eq!(redact("pedido 529.982.247-26"), "pedido 529.982.247-26");
        assert_eq!(redact("4539 1488 0343 6468"), "4539 1488 0343 6468");
    }

    #[test]
    fn phones_and_emails_are_masked_but_dates_and_prices_are_not() {
        assert_eq!(
            redact("me liga no (11) 98765-4321"),
            "me liga no [TELEFONE]"
        );
        assert_eq!(redact("+55 21 3333-4444"), "[TELEFONE]");
        assert_eq!(
            redact("escreve para ana.silva@empresa.com.br"),
            "escreve para [E-MAIL]"
        );
        assert_eq!(
            redact("em 05/10/2026 pagamos R$ 1.250,00"),
            "em 05/10/2026 pagamos R$ 1.250,00"
        );
        assert_eq!(redact("são 12 mil e 3 itens"), "são 12 mil e 3 itens");
        assert_eq!(
            redact("0000000000000"),
            "0000000000000",
            "repeated digits are not data"
        );
    }
}
