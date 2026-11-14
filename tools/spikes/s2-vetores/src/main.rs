//! Spike S2 (esforço 046) — PROTÓTIPO, fora do produto. Desenho do ADR 0012:
//! `memory.db` cifrado (SQLCipher), tabelas `vetores_bin` e `vetores_i8` em
//! ordem de inserção (rowid = tempo), filtro de período por faixa de rowid,
//! varredura binária em Rust, reordenação em int8.
//!
//! Uso: s2-vetores bench <base.f32> <queries.f32> <N> <dir_db> <consultas> <saida.jsonl> [rotulo]
use anyhow::Result;
use rusqlite::{Connection, params};
use s2_vetores::*;
use std::io::Write;
use std::time::Instant;

const PASSPHRASE: &str = "senha-de-teste-s2";
// Chave crua de 256 bits (hex): o SQLCipher pula a derivação PBKDF2.
const RAW_KEY: &str = "x'2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99'";
const PERIOD_FRACTION: f64 = 30.0 / 365.0;

fn read_f32(path: &str, rows: usize) -> Vec<Vec<f32>> {
    let b = std::fs::read(path).unwrap();
    let n = (b.len() / 4 / D).min(rows);
    (0..n)
        .map(|i| b[i * D * 4..(i + 1) * D * 4].chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect())
        .collect()
}

fn open(path: &str, raw_key: bool, cache_kib: Option<i64>) -> Result<(Connection, f64)> {
    let t = Instant::now();
    let c = Connection::open(path)?;
    if raw_key {
        c.execute_batch(&format!("PRAGMA key = \"{RAW_KEY}\";"))?;
    } else {
        c.execute_batch(&format!("PRAGMA key = '{PASSPHRASE}';"))?;
    }
    if let Some(k) = cache_kib {
        c.execute_batch(&format!("PRAGMA cache_size = -{k};"))?;
    }
    // Força a leitura do cabeçalho (derivação da chave acontece aqui).
    let _: i64 = c.query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get(0))?;
    Ok((c, t.elapsed().as_secs_f64() * 1000.0))
}

fn build(path: &str, base: &[Vec<f32>]) -> Result<f64> {
    let t = Instant::now();
    let (mut c, _) = open(path, true, None)?;
    c.execute_batch(
        "PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;
         CREATE TABLE vetores_bin(id INTEGER PRIMARY KEY, v BLOB NOT NULL);
         CREATE TABLE vetores_i8(id INTEGER PRIMARY KEY, v BLOB NOT NULL);",
    )?;
    let tx = c.transaction()?;
    {
        let mut b = tx.prepare("INSERT INTO vetores_bin(id, v) VALUES (?1, ?2)")?;
        let mut q = tx.prepare("INSERT INTO vetores_i8(id, v) VALUES (?1, ?2)")?;
        for (i, v) in base.iter().enumerate() {
            let id = i as i64 + 1;
            b.execute(params![id, to_binary(v)])?;
            let q8: Vec<u8> = to_int8(v).into_iter().map(|x| x as u8).collect();
            q.execute(params![id, q8])?;
        }
    }
    tx.commit()?;
    c.execute_batch("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA journal_mode = DELETE;")?;
    Ok(t.elapsed().as_secs_f64())
}

/// Busca do ADR 0012: binário (faixa de rowid) → `k1` candidatos → int8 → top 10.
fn search(c: &Connection, q: &[f32], k1: usize, from_id: i64) -> Result<Vec<i64>> {
    let qb = to_binary(q);
    let qi = to_int8(q);
    let mut top = TopK::new(k1);
    {
        let mut st = c.prepare_cached("SELECT id, v FROM vetores_bin WHERE id >= ?1")?;
        let mut rows = st.query([from_id])?;
        while let Some(r) = rows.next()? {
            let id: i64 = r.get(0)?;
            let d = hamming(r.get_ref(1)?.as_blob()?, &qb);
            top.push(d as i64, id);
        }
    }
    let mut best = TopK::new(10);
    let mut st = c.prepare_cached("SELECT v FROM vetores_i8 WHERE id = ?1")?;
    for id in top.ids() {
        let s = st.query_row([id], |r| {
            let b = r.get_ref(0)?.as_blob()?;
            let v: Vec<i8> = b.iter().map(|&x| x as i8).collect();
            Ok(dot_i8(&v, &qi))
        })?;
        best.push(-(s as i64), id);
    }
    Ok(best.ids())
}

fn exact(base: &[Vec<f32>], q: &[f32], from: usize) -> Vec<i64> {
    let mut top = TopK::new(10);
    for (i, v) in base.iter().enumerate().skip(from) {
        top.push(-((dot_f32(v, q) * 1e7) as i64), i as i64 + 1);
    }
    top.ids()
}

fn pct(v: &mut [f64], p: f64) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[((p / 100.0) * (v.len() - 1) as f64).round() as usize]
}

fn main() -> Result<()> {
    let a: Vec<String> = std::env::args().collect();
    assert_eq!(a[1], "bench");
    let (base_p, q_p, n, dir, nq, out_p) = (&a[2], &a[3], a[4].parse::<usize>()?, &a[5], a[6].parse::<usize>()?, &a[7]);
    let label = a.get(8).cloned().unwrap_or_else(|| "padrao".into());
    let mut out = std::fs::OpenOptions::new().create(true).append(true).open(out_p)?;
    let real = read_f32(base_p, usize::MAX);
    let queries = read_f32(q_p, nq);
    // Base: os vetores reais; acima disso, derivados com ruído (só tempo).
    let mut rng = Rng::new(7);
    let base: Vec<Vec<f32>> = (0..n)
        .map(|i| if i < real.len() { real[i].clone() } else { jitter(&real[i % real.len()], 0.05, &mut rng) })
        .collect();
    let derived = n > real.len();
    std::fs::create_dir_all(dir)?;
    let path = format!("{dir}/memory-{n}.db");
    if !std::path::Path::new(&path).exists() {
        let s = build(&path, &base)?;
        let mb = std::fs::metadata(&path)?.len() as f64 / 1e6;
        writeln!(out, "{}", serde_json::json!({"tipo": "carga", "N": n, "mb": mb, "carga_s": s, "derivados": derived}))?;
    }
    let from_period = n - (n as f64 * PERIOD_FRACTION) as usize;
    let gt: Vec<Vec<i64>> = queries.iter().map(|q| exact(&base, q, 0)).collect();
    let gt_p: Vec<Vec<i64>> = queries.iter().map(|q| exact(&base, q, from_period)).collect();
    let sized_kib = (n * (BIN_BYTES + 24) / 1024 + 4096) as i64;

    // Frio: abrir (senha ou chave crua) + primeira consulta; 5 repetições cada.
    for raw in [false, true] {
        let (mut open_ms, mut first_ms) = (Vec::new(), Vec::new());
        for i in 0..5 {
            let (c, o) = open(&path, raw, Some(sized_kib))?;
            let t = Instant::now();
            search(&c, &queries[i % queries.len()], 100, 1)?;
            open_ms.push(o);
            first_ms.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        writeln!(out, "{}", serde_json::json!({"tipo": "frio", "rotulo": label, "N": n, "chave_crua": raw,
            "abrir_ms_p50": pct(&mut open_ms, 50.0), "primeira_ms_p50": pct(&mut first_ms, 50.0)}))?;
    }
    // Quente: 3 aquecimento + `nq` consultas por caso.
    for (cache_name, cache) in [("padrao_2MB", None), ("dimensionado", Some(sized_kib))] {
        let (c, _) = open(&path, true, cache)?;
        for filtro in [false, true] {
            for k1 in [100usize, 200, 400] {
                let from = if filtro { from_period as i64 + 1 } else { 1 };
                for q in queries.iter().take(3) {
                    search(&c, q, k1, from)?;
                }
                let (mut lat, mut rec) = (Vec::new(), 0.0);
                for (i, q) in queries.iter().enumerate() {
                    let t = Instant::now();
                    let got = search(&c, q, k1, from)?;
                    lat.push(t.elapsed().as_secs_f64() * 1000.0);
                    rec += recall(&got, if filtro { &gt_p[i] } else { &gt[i] });
                }
                let line = serde_json::json!({"tipo": "quente", "rotulo": label, "N": n, "cache": cache_name, "filtro_30d": filtro,
                    "candidatos": k1, "nq": queries.len(), "p50_ms": pct(&mut lat.clone(), 50.0), "p95_ms": pct(&mut lat, 95.0),
                    "recall10": rec / queries.len() as f64, "base_derivada": derived});
                eprintln!("{line}");
                writeln!(out, "{line}")?;
            }
        }
    }
    Ok(())
}
