//! Spike S2 (esforço 046) — PROTÓTIPO. Partes puras do desenho do ADR 0012:
//! quantização binária e int8, distâncias, top-k, recall e ruído determinístico.

pub const D: usize = 384;
pub const BIN_BYTES: usize = D / 8;

/// Bit por dimensão: 1 quando o componente é positivo (bit menos
/// significativo primeiro dentro de cada byte).
pub fn to_binary(v: &[f32]) -> Vec<u8> {
    let mut out = vec![0u8; v.len() / 8];
    for (i, &x) in v.iter().enumerate() {
        if x > 0.0 {
            out[i / 8] |= 1 << (i % 8);
        }
    }
    out
}

/// int8 simétrico com escala fixa 254 (componentes de vetores normalizados de
/// 384 dimensões ficam bem abaixo de ±0,5), saturado em ±127.
pub fn to_int8(v: &[f32]) -> Vec<i8> {
    v.iter().map(|&x| (x * 254.0).round().clamp(-127.0, 127.0) as i8).collect()
}

pub fn hamming(a: &[u8], b: &[u8]) -> u32 {
    a.chunks_exact(8)
        .zip(b.chunks_exact(8))
        .map(|(x, y)| (u64::from_le_bytes(x.try_into().unwrap()) ^ u64::from_le_bytes(y.try_into().unwrap())).count_ones())
        .sum()
}

pub fn dot_i8(a: &[i8], b: &[i8]) -> i32 {
    a.iter().zip(b).map(|(&x, &y)| x as i32 * y as i32).sum()
}

pub fn dot_f32(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Mantém os `k` menores por chave (empate: menor id).
pub struct TopK {
    k: usize,
    v: Vec<(i64, i64)>,
}
impl TopK {
    pub fn new(k: usize) -> Self {
        Self { k, v: Vec::with_capacity(k + 1) }
    }
    pub fn push(&mut self, key: i64, id: i64) {
        if self.v.len() == self.k && (key, id) >= self.v[self.k - 1] {
            return;
        }
        let pos = self.v.partition_point(|x| *x < (key, id));
        self.v.insert(pos, (key, id));
        self.v.truncate(self.k);
    }
    pub fn ids(&self) -> Vec<i64> {
        self.v.iter().map(|x| x.1).collect()
    }
}

/// Fração dos `truth` encontrados em `got`.
pub fn recall(got: &[i64], truth: &[i64]) -> f64 {
    truth.iter().filter(|t| got.contains(t)).count() as f64 / truth.len() as f64
}

/// Gerador determinístico com normal por Box–Muller (sem dependência nova).
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }
    fn next_f64(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 11) as f64 + 0.5) / (1u64 << 53) as f64
    }
    pub fn normal(&mut self) -> f64 {
        let (u1, u2) = (self.next_f64(), self.next_f64());
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

/// Vetor derivado (só para medir tempo em escala): v + ruído, normalizado.
pub fn jitter(v: &[f32], sigma: f64, rng: &mut Rng) -> Vec<f32> {
    let mut w: Vec<f32> = v.iter().map(|&x| x + (sigma * rng.normal()) as f32).collect();
    let n = w.iter().map(|x| x * x).sum::<f32>().sqrt();
    w.iter_mut().for_each(|x| *x /= n);
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_sets_bits_of_positive_components() {
        let mut v = vec![-1.0f32; 16];
        v[0] = 0.5;
        v[9] = 0.1;
        assert_eq!(to_binary(&v), vec![0b0000_0001, 0b0000_0010]);
    }

    #[test]
    fn int8_scales_and_saturates() {
        assert_eq!(to_int8(&[0.1, -0.25, 0.6, -0.9, 0.0]), vec![25, -64, 127, -127, 0]);
    }

    #[test]
    fn hamming_and_dot_literals() {
        assert_eq!(hamming(&[0xFF, 0, 0, 0, 0, 0, 0, 0], &[0x0F, 0, 0, 0, 0, 0, 0, 1]), 5);
        assert_eq!(dot_i8(&[1, -2, 3], &[4, 5, -6]), 4 - 10 - 18);
    }

    #[test]
    fn topk_keeps_smallest_with_id_tiebreak() {
        let mut t = TopK::new(3);
        for (k, id) in [(5, 1), (2, 2), (9, 3), (2, 1), (1, 4)] {
            t.push(k, id);
        }
        assert_eq!(t.ids(), vec![4, 1, 2]);
    }

    #[test]
    fn recall_literal() {
        assert_eq!(recall(&[1, 2, 3, 9], &[1, 2, 5, 6]), 0.5);
    }

    #[test]
    fn jitter_stays_normalized_and_deterministic() {
        let v = vec![1.0f32 / (D as f32).sqrt(); D];
        let a = jitter(&v, 0.05, &mut Rng::new(1));
        let b = jitter(&v, 0.05, &mut Rng::new(1));
        assert_eq!(a, b);
        assert!((a.iter().map(|x| x * x).sum::<f32>() - 1.0).abs() < 1e-5);
    }
}
