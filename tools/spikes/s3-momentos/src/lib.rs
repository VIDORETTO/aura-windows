//! Spike S3/S4 (esforço 046) — PROTÓTIPO. Agregação dos números da ferramenta
//! de campo. Nada aqui guarda texto: só contagens, tempos e tamanhos.

use serde::Serialize;
use std::collections::BTreeMap;

/// Caminho usado para ler o texto do Momento.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// UI Automation dentro do orçamento.
    Uia,
    /// UIA estourou o orçamento (ou estava ocupada) e caiu para OCR.
    UiaTimeoutOcr,
    /// UIA voltou vazia e caiu para OCR.
    UiaEmptyOcr,
    /// Navegador/app Chromium: direto para OCR.
    Ocr,
}

/// Classe de janela que vai direto para OCR (ADR 0013): navegadores e apps
/// Chromium/Electron/WebView2 e o Firefox, que ligam a árvore de
/// acessibilidade inteira quando um cliente UIA aparece.
pub fn goes_to_ocr(class: &str) -> bool {
    class.starts_with("Chrome_WidgetWin") || class == "MozillaWindowClass"
}

#[derive(Default, Debug, Clone)]
pub struct AppStats {
    pub momentos: u64,
    pub por_rota: BTreeMap<&'static str, u64>,
    pub uia_ms: Vec<f64>,
    pub uia_chars: Vec<f64>,
    pub ocr_ms: Vec<f64>,
    pub ocr_chars: Vec<f64>,
    pub captura_ms: Vec<f64>,
    pub webp_kb: Vec<f64>,
    pub webp_ms: Vec<f64>,
    pub largura_px: Vec<f64>,
    pub wgc_quadros: u64,
    pub wgc_area_px: f64,
    pub batidas: u64,
}

impl AppStats {
    pub fn route(&mut self, r: Route) {
        let k = match r {
            Route::Uia => "uia",
            Route::UiaTimeoutOcr => "uia_estourou_ocr",
            Route::UiaEmptyOcr => "uia_vazia_ocr",
            Route::Ocr => "ocr_direto",
        };
        *self.por_rota.entry(k).or_insert(0) += 1;
        self.momentos += 1;
    }
}

/// p50/p95/máx de uma série (vazia → `null`).
pub fn summary(v: &[f64]) -> serde_json::Value {
    if v.is_empty() {
        return serde_json::Value::Null;
    }
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let at = |p: f64| s[((p / 100.0) * (s.len() - 1) as f64).round() as usize];
    serde_json::json!({"n": s.len(), "p50": at(50.0), "p95": at(95.0), "max": s[s.len() - 1]})
}

#[derive(Serialize)]
pub struct Global {
    pub inicio_unix: u64,
    pub segundos: f64,
    pub ev_foreground: u64,
    pub ev_namechange_titulo: u64,
    pub ev_namechange_total: u64,
    pub cpu_s: f64,
    pub ws_pico_mb: f64,
    pub ws_atual_mb: f64,
}

/// Resumo serializável: só números por executável.
pub fn report(global: &Global, apps: &BTreeMap<String, AppStats>) -> serde_json::Value {
    let mut a = serde_json::Map::new();
    for (exe, s) in apps {
        a.insert(
            exe.clone(),
            serde_json::json!({
                "momentos": s.momentos, "por_rota": s.por_rota,
                "uia_ms": summary(&s.uia_ms), "uia_caracteres": summary(&s.uia_chars),
                "ocr_ms": summary(&s.ocr_ms), "ocr_caracteres": summary(&s.ocr_chars),
                "captura_ms": summary(&s.captura_ms), "webp_kb": summary(&s.webp_kb), "webp_ms": summary(&s.webp_ms),
                "largura_px": summary(&s.largura_px),
                "wgc_quadros": s.wgc_quadros, "wgc_area_mpx": s.wgc_area_px / 1e6, "batidas": s.batidas,
            }),
        );
    }
    serde_json::json!({"versao": 1, "global": global, "apps": a})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_classes_go_to_ocr() {
        assert!(goes_to_ocr("Chrome_WidgetWin_1"));
        assert!(goes_to_ocr("MozillaWindowClass"));
        assert!(!goes_to_ocr("OpusApp"));
    }

    #[test]
    fn summary_literals() {
        assert_eq!(summary(&[]), serde_json::Value::Null);
        let s = summary(&[5.0, 1.0, 3.0, 2.0, 4.0]);
        assert_eq!(s["p50"], 3.0);
        assert_eq!(s["p95"], 5.0);
        assert_eq!(s["n"], 5);
    }

    #[test]
    fn report_has_only_numbers_and_names() {
        let mut apps = BTreeMap::new();
        let mut s = AppStats::default();
        s.route(Route::Uia);
        s.uia_ms.push(12.0);
        s.uia_chars.push(340.0);
        apps.insert("winword.exe".to_string(), s);
        let g = Global { inicio_unix: 0, segundos: 1.0, ev_foreground: 1, ev_namechange_titulo: 0, ev_namechange_total: 3, cpu_s: 0.1, ws_pico_mb: 20.0, ws_atual_mb: 18.0 };
        let r = report(&g, &apps);
        let text = r.to_string();
        // Nenhuma string além dos nomes de chave, do executável e das rotas.
        fn strings(v: &serde_json::Value, out: &mut Vec<String>) {
            match v {
                serde_json::Value::String(s) => out.push(s.clone()),
                serde_json::Value::Array(a) => a.iter().for_each(|x| strings(x, out)),
                serde_json::Value::Object(o) => o.values().for_each(|x| strings(x, out)),
                _ => {}
            }
        }
        let mut found = Vec::new();
        strings(&r, &mut found);
        assert!(found.is_empty(), "valores de texto no relatório: {found:?}");
        assert!(text.contains("winword.exe"));
        assert_eq!(r["apps"]["winword.exe"]["por_rota"]["uia"], 1);
    }
}
