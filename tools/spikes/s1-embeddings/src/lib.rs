//! Spike S1 (esforço 046) — PROTÓTIPO. Partes puras: dobra de acentos,
//! tokens, termo raro, fusão por posição recíproca e métricas.

use std::collections::HashMap;

/// Minúsculas sem acento (português).
pub fn fold(s: &str) -> String {
    s.chars()
        .flat_map(|c| c.to_lowercase())
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            other => other,
        })
        .collect()
}

/// Palavras de 2+ caracteres alfanuméricos, dobradas.
pub fn tokens(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.chars().count() >= 2)
        .map(fold)
        .collect()
}

pub const STOP: &[&str] = &[
    "a", "o", "e", "de", "da", "do", "das", "dos", "que", "em", "no", "na", "nos", "nas", "um",
    "uma", "uns", "umas", "para", "pra", "por", "com", "sem", "se", "sobre", "qual", "quais",
    "quando", "onde", "como", "quem", "quanto", "quanta", "quantos", "quantas", "foi", "era",
    "ser", "esta", "estava", "eu", "meu", "minha", "me", "mim", "ao", "aos", "as", "os", "ou",
    "mais", "menos", "isso", "esse", "essa", "este", "aquele", "aquela", "la", "ja", "tem",
    "ter", "tinha", "havia", "ha", "pelo", "pela", "pelos", "pelas", "num", "numa", "dele",
    "dela", "nao", "voce", "aquilo", "mesmo", "entao", "ate", "the", "of",
];

/// Frequência de documento por token dobrado.
pub fn document_frequency<'a>(docs: impl IntoIterator<Item = &'a str>) -> HashMap<String, usize> {
    let mut df = HashMap::new();
    for d in docs {
        let mut seen: Vec<String> = tokens(d);
        seen.sort();
        seen.dedup();
        for t in seen {
            *df.entry(t).or_insert(0) += 1;
        }
    }
    df
}

/// Termo raro: fora das palavras vazias e (com dígito, ou presente em no
/// máximo `max_df` documentos do corpus).
pub fn has_rare_term(query: &str, df: &HashMap<String, usize>, max_df: usize) -> bool {
    tokens(query).iter().any(|t| {
        !STOP.contains(&t.as_str())
            && (t.chars().any(|c| c.is_ascii_digit())
                || df.get(t).is_some_and(|&n| n <= max_df))
    })
}

/// Fusão por posição recíproca ponderada: soma de `peso / (k + posição)`,
/// posição a partir de 1. Empate desempata pelo índice do documento.
pub fn rrf(lists: &[(&[usize], f64)], k: f64) -> Vec<usize> {
    let mut score: HashMap<usize, f64> = HashMap::new();
    for (list, w) in lists {
        for (r, &d) in list.iter().enumerate() {
            *score.entry(d).or_insert(0.0) += w / (k + r as f64 + 1.0);
        }
    }
    let mut v: Vec<(usize, f64)> = score.into_iter().collect();
    v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
    v.into_iter().map(|(d, _)| d).collect()
}

/// Posição (a partir de 1) do documento certo, ou `None` se ausente.
pub fn position(list: &[usize], gold: usize) -> Option<usize> {
    list.iter().position(|&d| d == gold).map(|p| p + 1)
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct Metrics {
    pub hit1: f64,
    pub hit3: f64,
    pub hit10: f64,
    pub mrr: f64,
    pub mean_position: f64,
    pub missing: usize,
}

/// Métricas sobre as posições; ausente conta como `n_docs + 1` na média.
pub fn metrics(positions: &[Option<usize>], n_docs: usize) -> Metrics {
    let n = positions.len() as f64;
    let hit = |k: usize| positions.iter().filter(|p| p.is_some_and(|p| p <= k)).count() as f64 / n;
    Metrics {
        hit1: hit(1),
        hit3: hit(3),
        hit10: hit(10),
        mrr: positions.iter().map(|p| p.map_or(0.0, |p| 1.0 / p as f64)).sum::<f64>() / n,
        mean_position: positions.iter().map(|p| p.unwrap_or(n_docs + 1) as f64).sum::<f64>() / n,
        missing: positions.iter().filter(|p| p.is_none()).count(),
    }
}

/// p50/p95 por posição arredondada (mesma regra das medições anteriores).
pub fn pct(v: &mut [f64], p: f64) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[((p / 100.0) * (v.len() - 1) as f64).round() as usize]
}

/// Escolha do modelo: maior MRR; diferença menor que `tie` desempata pela
/// menor latência por documento.
pub fn choose<'a>(cands: &[(&'a str, f64, f64)], tie: f64) -> &'a str {
    let best = cands.iter().map(|c| c.1).fold(f64::MIN, f64::max);
    cands
        .iter()
        .filter(|c| best - c.1 < tie)
        .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
        .unwrap()
        .0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_removes_portuguese_accents() {
        assert_eq!(fold("Ação Revisão Ônibus"), "acao revisao onibus");
    }

    #[test]
    fn tokens_drop_short_and_split_codes() {
        assert_eq!(tokens("Ticket PAG-1187: é a."), vec!["ticket", "pag", "1187"]);
    }

    #[test]
    fn rare_term_by_digit_or_low_document_frequency() {
        let df = document_frequency(["contrato de embalagens", "contrato renovado", "fornecedor"]);
        assert!(has_rare_term("qual o ticket 1187", &df, 2));
        assert!(has_rare_term("quem é o fornecedor", &df, 1));
        assert!(!has_rare_term("o contrato", &df, 1));
        assert!(!has_rare_term("palavrainexistente", &df, 2));
    }

    #[test]
    fn rrf_equal_and_weighted() {
        // vetor [1,2,3], lexical [3,1]; k = 60.
        // iguais: d1 = 1/61 + 1/62; d3 = 1/63 + 1/61; d2 = 1/62 → [1, 3, 2].
        assert_eq!(rrf(&[(&[1, 2, 3], 1.0), (&[3, 1], 1.0)], 60.0), vec![1, 3, 2]);
        // lexical com peso 0,5: d1 = 1/61 + 0,5/62 = 0,02446; d3 = 1/63 + 0,5/61 = 0,02407;
        // d2 = 1/62 = 0,01613 → [1, 3, 2].
        assert_eq!(rrf(&[(&[1, 2, 3], 1.0), (&[3, 1], 0.5)], 60.0), vec![1, 3, 2]);
        // lexical sozinho promove o 3: vetor [1,2,3], lexical [3] com peso 1:
        // d3 = 1/63 + 1/61 = 0,03227 > d1 = 1/61 = 0,01639 → 3 primeiro.
        assert_eq!(rrf(&[(&[1, 2, 3], 1.0), (&[3], 1.0)], 60.0)[0], 3);
    }

    #[test]
    fn metrics_literals() {
        let m = metrics(&[Some(1), Some(2), Some(11), None], 50);
        assert_eq!(m.hit1, 0.25);
        assert_eq!(m.hit3, 0.5);
        assert_eq!(m.hit10, 0.5);
        assert!((m.mrr - (1.0 + 0.5 + 1.0 / 11.0) / 4.0).abs() < 1e-12);
        assert_eq!(m.mean_position, (1.0 + 2.0 + 11.0 + 51.0) / 4.0);
        assert_eq!(m.missing, 1);
    }

    #[test]
    fn choose_by_mrr_then_latency() {
        assert_eq!(choose(&[("a", 0.80, 5.0), ("b", 0.90, 20.0)], 0.02), "b");
        assert_eq!(choose(&[("a", 0.89, 5.0), ("b", 0.90, 20.0)], 0.02), "a");
    }
}
