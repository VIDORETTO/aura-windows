//! Spike S1 (esforço 046) — PROTÓTIPO, fora do produto.
//! Uso: s1-embeddings <modelo> <dir_modelos> <corpus.json> <queries.json> <saida.json>
use anyhow::{Context, Result};
use s1_embeddings::*;
use serde::Deserialize;
use std::time::Instant;

#[derive(Deserialize)]
struct Doc {
    id: String,
    fonte: String,
    titulo: String,
    texto: String,
}
#[derive(Deserialize)]
struct Query {
    id: String,
    pergunta: String,
    doc: String,
    estilo: String,
}

#[derive(Clone, Copy)]
enum Pool {
    Mean,
    Cls,
}

struct Cfg {
    dir: &'static str,
    pool: Pool,
    qpref: &'static str,
    dpref: &'static str,
}

fn cfg(model: &str) -> Cfg {
    match model {
        "e5-small" => Cfg { dir: "e5-small", pool: Pool::Mean, qpref: "query: ", dpref: "passage: " },
        "granite-97m" => Cfg { dir: "granite-97m", pool: Pool::Cls, qpref: "", dpref: "" },
        "bekko-a8m" => Cfg { dir: "bekko-a8m", pool: Pool::Mean, qpref: "", dpref: "" },
        "bekko-a25m-q8" => Cfg { dir: "bekko-a25m-q8", pool: Pool::Mean, qpref: "", dpref: "" },
        other => panic!("modelo desconhecido: {other}"),
    }
}

fn peak_working_set_mb() -> f64 {
    use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows::Win32::System::Threading::GetCurrentProcess;
    let mut pmc = PROCESS_MEMORY_COUNTERS {
        cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        ..Default::default()
    };
    unsafe {
        let _ = GetProcessMemoryInfo(GetCurrentProcess(), &mut pmc, pmc.cb);
    }
    pmc.PeakWorkingSetSize as f64 / 1e6
}

struct Encoder {
    session: ort::session::Session,
    tok: tokenizers::Tokenizer,
    pool: Pool,
    type_ids: bool,
}

impl Encoder {
    fn encode(&mut self, text: &str) -> Result<Vec<f32>> {
        let enc = self.tok.encode(text, true).map_err(|e| anyhow::anyhow!("{e}"))?;
        let n = enc.get_ids().len();
        let ids: Vec<i64> = enc.get_ids().iter().map(|&x| x as i64).collect();
        let mask: Vec<i64> = enc.get_attention_mask().iter().map(|&x| x as i64).collect();
        let ids_t = ort::value::Tensor::from_array(([1usize, n], ids))?;
        let mask_t = ort::value::Tensor::from_array(([1usize, n], mask.clone()))?;
        let outputs = if self.type_ids {
            let tt = ort::value::Tensor::from_array(([1usize, n], vec![0i64; n]))?;
            self.session.run(ort::inputs!["input_ids" => ids_t, "attention_mask" => mask_t, "token_type_ids" => tt])?
        } else {
            self.session.run(ort::inputs!["input_ids" => ids_t, "attention_mask" => mask_t])?
        };
        let (shape, data) = outputs[0].try_extract_tensor::<f32>()?;
        let dim = shape[2] as usize;
        let mut v = vec![0f32; dim];
        match self.pool {
            Pool::Cls => v.copy_from_slice(&data[..dim]),
            Pool::Mean => {
                let mut count = 0f32;
                for t in 0..n {
                    if mask[t] == 1 {
                        count += 1.0;
                        for j in 0..dim {
                            v[j] += data[t * dim + j];
                        }
                    }
                }
                v.iter_mut().for_each(|x| *x /= count);
            }
        }
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        v.iter_mut().for_each(|x| *x /= norm);
        Ok(v)
    }
}

/// `embed <modelo> <dir_modelos> <entrada.txt (um trecho por linha)> <saida.f32> <threads>`:
/// codifica trechos como documento e grava vetores f32 crus (para o S2).
fn embed(a: &[String]) -> Result<()> {
    let (model, models_dir, inp, out, threads) = (&a[2], &a[3], &a[4], &a[5], a[6].parse::<usize>()?);
    let c = cfg(model);
    let dir = std::path::Path::new(models_dir).join(c.dir);
    let e = |e: ort::Error<ort::session::builder::SessionBuilder>| anyhow::anyhow!("{e}");
    let session = ort::session::Session::builder()?
        .with_intra_threads(threads)
        .map_err(e)?
        .commit_from_file(dir.join("model.onnx"))?;
    let type_ids = session.inputs().iter().any(|i| i.name() == "token_type_ids");
    let mut tok = tokenizers::Tokenizer::from_file(dir.join("tokenizer.json")).map_err(|e| anyhow::anyhow!("{e}"))?;
    tok.with_truncation(Some(tokenizers::TruncationParams { max_length: 256, ..Default::default() }))
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    tok.with_padding(None);
    let mut enc = Encoder { session, tok, pool: c.pool, type_ids };
    let text = std::fs::read_to_string(inp)?;
    let mut w = std::io::BufWriter::new(std::fs::File::create(out)?);
    let t = Instant::now();
    let mut n = 0usize;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let v = enc.encode(&format!("{}{}", c.dpref, line))?;
        for x in v {
            std::io::Write::write_all(&mut w, &x.to_le_bytes())?;
        }
        n += 1;
        if n % 10_000 == 0 {
            eprintln!("{n} trechos, {:.0} s", t.elapsed().as_secs_f64());
        }
    }
    eprintln!("{n} trechos em {:.0} s", t.elapsed().as_secs_f64());
    Ok(())
}

fn main() -> Result<()> {
    let a: Vec<String> = std::env::args().collect();
    if a[1] == "embed" {
        return embed(&a);
    }
    let (model, models_dir, corpus_p, queries_p, out_p) = (&a[1], &a[2], &a[3], &a[4], &a[5]);
    let c = cfg(model);
    let docs: Vec<Doc> = serde_json::from_str(&std::fs::read_to_string(corpus_p)?)?;
    let queries: Vec<Query> = serde_json::from_str(&std::fs::read_to_string(queries_p)?)?;
    let base_mb = peak_working_set_mb();

    let t0 = Instant::now();
    let dir = std::path::Path::new(models_dir).join(c.dir);
    let e = |e: ort::Error<ort::session::builder::SessionBuilder>| anyhow::anyhow!("{e}");
    let session = ort::session::Session::builder()?
        .with_intra_threads(1)
        .map_err(e)?
        .with_inter_threads(1)
        .map_err(e)?
        .commit_from_file(dir.join("model.onnx"))
        .context("onnx")?;
    let type_ids = session.inputs().iter().any(|i| i.name() == "token_type_ids");
    let mut tok = tokenizers::Tokenizer::from_file(dir.join("tokenizer.json")).map_err(|e| anyhow::anyhow!("{e}"))?;
    tok.with_truncation(Some(tokenizers::TruncationParams { max_length: 512, ..Default::default() }))
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    tok.with_padding(None);
    let mut enc = Encoder { session, tok, pool: c.pool, type_ids };
    let load_s = t0.elapsed().as_secs_f64();

    // Documento = cabeçalho por regra (fonte e título) + texto, como no plano §5.3 (1).
    let dtexts: Vec<String> = docs.iter().map(|d| format!("{}{} — {}. {}", c.dpref, d.fonte, d.titulo, d.texto)).collect();
    let qtexts: Vec<String> = queries.iter().map(|q| format!("{}{}", c.qpref, q.pergunta)).collect();
    for t in dtexts.iter().take(5) {
        enc.encode(t)?;
    }
    let mut dvec = Vec::new();
    let mut dlat = Vec::new();
    for pass in 0..3 {
        for t in &dtexts {
            let s = Instant::now();
            let v = enc.encode(t)?;
            dlat.push(s.elapsed().as_secs_f64() * 1000.0);
            if pass == 0 {
                dvec.push(v);
            }
        }
    }
    let mut qvec = Vec::new();
    let mut qlat = Vec::new();
    for pass in 0..3 {
        for t in &qtexts {
            let s = Instant::now();
            let v = enc.encode(t)?;
            qlat.push(s.elapsed().as_secs_f64() * 1000.0);
            if pass == 0 {
                qvec.push(v);
            }
        }
    }
    let peak_mb = peak_working_set_mb();

    // Lexical: FTS5 com o tokenizador do plano, OR das palavras fora da lista vazia.
    let db = rusqlite::Connection::open_in_memory()?;
    db.execute_batch("CREATE VIRTUAL TABLE f USING fts5(t, tokenize='unicode61 remove_diacritics 2')")?;
    for (i, d) in docs.iter().enumerate() {
        db.execute("INSERT INTO f(rowid, t) VALUES (?1, ?2)", rusqlite::params![i as i64, format!("{} {}", d.titulo, d.texto)])?;
    }
    let df = document_frequency(docs.iter().map(|d| d.texto.as_str()));
    let ids: Vec<&str> = docs.iter().map(|d| d.id.as_str()).collect();

    let mut rows = Vec::new();
    let mut per_variant: std::collections::BTreeMap<&str, Vec<(Option<usize>, String)>> = Default::default();
    for (qi, q) in queries.iter().enumerate() {
        let gold = ids.iter().position(|&d| d == q.doc).context("doc da pergunta")?;
        let mut scored: Vec<(usize, f32)> = dvec
            .iter()
            .enumerate()
            .map(|(i, d)| (i, d.iter().zip(&qvec[qi]).map(|(a, b)| a * b).sum()))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
        let vec_list: Vec<usize> = scored.iter().map(|x| x.0).collect();
        let terms: Vec<String> = tokens(&q.pergunta)
            .into_iter()
            .filter(|t| !STOP.contains(&t.as_str()))
            .map(|t| format!("\"{t}\""))
            .collect();
        let lex_list: Vec<usize> = if terms.is_empty() {
            vec![]
        } else {
            let mut st = db.prepare_cached("SELECT rowid FROM f WHERE f MATCH ?1 ORDER BY bm25(f)")?;
            st.query_map([terms.join(" OR ")], |r| r.get::<_, i64>(0))?
                .filter_map(|r| r.ok())
                .map(|x| x as usize)
                .collect()
        };
        let rare = has_rare_term(&q.pergunta, &df, 2);
        let variants: Vec<(&str, Vec<usize>)> = vec![
            ("vetor", vec_list.clone()),
            ("fts5", lex_list.clone()),
            ("rrf_1_1", rrf(&[(&vec_list, 1.0), (&lex_list, 1.0)], 60.0)),
            ("rrf_1_05", rrf(&[(&vec_list, 1.0), (&lex_list, 0.5)], 60.0)),
            ("lexical_se_raro", if rare { rrf(&[(&vec_list, 1.0), (&lex_list, 1.0)], 60.0) } else { vec_list.clone() }),
        ];
        let mut row = serde_json::json!({"id": q.id, "estilo": q.estilo, "raro": rare});
        for (name, list) in &variants {
            let p = position(list, gold);
            row[*name] = serde_json::json!(p);
            per_variant.entry(name).or_default().push((p, q.estilo.clone()));
        }
        rows.push(row);
    }
    let mut summary = serde_json::Map::new();
    for (name, v) in &per_variant {
        let all: Vec<Option<usize>> = v.iter().map(|x| x.0).collect();
        let mut by_style = serde_json::Map::new();
        for style in ["exato", "parafrase", "livre"] {
            let s: Vec<Option<usize>> = v.iter().filter(|x| x.1 == style).map(|x| x.0).collect();
            if !s.is_empty() {
                by_style.insert(style.into(), serde_json::to_value(metrics(&s, docs.len()))?);
            }
        }
        summary.insert(name.to_string(), serde_json::json!({"todas": metrics(&all, docs.len()), "por_estilo": by_style}));
    }
    let n_rare = rows.iter().filter(|r| r["raro"] == true).count();
    let out = serde_json::json!({
        "modelo": model,
        "arquivo_mb": std::fs::metadata(dir.join("model.onnx"))?.len() as f64 / 1e6,
        "dim": dvec[0].len(),
        "carga_s": load_s,
        "doc_ms_p50": pct(&mut dlat.clone(), 50.0), "doc_ms_p95": pct(&mut dlat, 95.0),
        "pergunta_ms_p50": pct(&mut qlat.clone(), 50.0), "pergunta_ms_p95": pct(&mut qlat, 95.0),
        "pico_ws_mb": peak_mb, "ws_base_mb": base_mb,
        "ort": "2.0.0-rc.12 (binário oficial baixado pelo crate)",
        "perguntas_com_termo_raro": n_rare,
        "metricas": summary,
        "por_pergunta": rows,
    });
    std::fs::write(out_p, serde_json::to_string_pretty(&out)?)?;
    println!("{}", serde_json::to_string(&out["metricas"]["vetor"]["todas"])?);
    Ok(())
}
