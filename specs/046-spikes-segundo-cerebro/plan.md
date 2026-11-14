---
schema: hybrid/plan
schema_version: 1.0
effort_id: 046-spikes-segundo-cerebro
revision: 1
spec_revision: 1
status: ready
---

# Plano — spikes do Segundo Cérebro

Consome o spec r1. Rota de pesquisa e protótipo: cada spike tem pergunta, experimento, limite, evidência e decisão. O status é **protótipo** em todo o código de `tools/spikes/`.

## Organização

- `tools/spikes/` guarda projetos Cargo independentes, cada um com seu próprio `[workspace]`. Assim, o `cargo build` da raiz não os inclui e o build do produto não passa a exigir Perl nem SQLCipher. Toolchain fixado pelo `rust-toolchain.toml` da raiz (1.98.1).
- Modelos ONNX, vetores, bancos e imagens ficam em `%LOCALAPPDATA%\aura-spikes\`, fora do git.
- O conjunto dourado sintético (`tools/spikes/s1-embeddings/golden/*.json`) é fixture de teste sem dado do usuário; entra no git porque vira o conjunto permanente do `aura-bench` (ADR 0014).
- Relatórios: `specs/046-spikes-segundo-cerebro/s1.md`, `s2.md` e `s3-s4.md`.

## S1 — `tools/spikes/s1-embeddings` (TK-001, TK-002)

- **Conjunto:** 240 documentos escritos por um subagente; 100 perguntas escritas por outro subagente que só vê o corpus. Pelo menos 30 perguntas por termo exato e 50 por paráfrase, todas com resposta única.
- **Binário Rust:**
  - `ort` = 2.0.0-rc.12, a mesma versão do lockfile, com o binário oficial do ONNX Runtime baixado pelo próprio `ort`; se o download não for permitido, usa a DLL do Python já usada nas medições;
  - `tokenizers` com `onig` desligado, para não ter dependência C;
  - `rusqlite` 0.40.2 `bundled` para o FTS5 com `unicode61 remove_diacritics 2`.
- **Execução:** 1 thread, lote 1, prefixos de cada cartão, pooling conforme o `1_Pooling/config.json`.
- **Saída:** JSON com qualidade (vetor puro e três fusões) e custo (latência p50/p95, RAM de pico pelo `GetProcessMemoryInfo`, carga).
- **Fusões:** RRF k = 60 com pesos (1, 1); RRF (1, 0,5); e "lexical só se a pergunta tem termo raro", com termo raro = token com IDF acima do percentil 90 do corpus, ou com dígito ou maiúscula interna.
- **Critério de escolha:** maior MRR no conjunto; empate (diferença < 0,02) decidido pela menor latência por documento.

## S2 — `tools/spikes/s2-vetores` (TK-003)

- Base: o `vecbench` das medições, migrado para o desenho do ADR 0012, só com BLOB binário + int8 e sem float32 guardado.
- Vetores:
  - qualidade: 100 mil trechos da Wikipédia pt (dump 20231101) codificados com o modelo do S1, com prioridade ociosa e 4 threads;
  - escala: até ~1 milhão por repetição desses vetores com ruído gaussiano pequeno. Isso só serve para medir tempo; o recall é medido só nos 100 mil reais.
- Medidas:
  - p50/p95 com e sem filtro de 30 dias, com cache padrão e com cache de ~80 MB por milhão;
  - candidatos 100/200/400 → recall@10;
  - frio: abrir o banco + primeira consulta; quente: as seguintes;
  - disco.
- **Aproximação de máquina modesta:** afinidade a 1 núcleo e `PROCTHROTTLEMAX` = 50% no plano ativo (`powercfg /setacvalueindex SCHEME_CURRENT SUB_PROCESSOR PROCTHROTTLEMAX 50` + `/setactive`), restaurado ao valor original no fim. Rotulado como aproximação.

## S3/S4 — `tools/spikes/s3-momentos` (TK-004)

- Binário Rust com o crate `windows` 0.62, derivado do `uiaprobe` e do `wgcprobe` das medições:
  - ganchos FOREGROUND + NAMECHANGE separados; assentamento de 1,5 s;
  - para a janela em primeiro plano: classe `Chrome_WidgetWin_*` → captura (`PrintWindow` com `PW_RENDERFULLCONTENT`) + OCR pt-BR na resolução nativa + tamanho em WebP sem perdas (crate `image-webp`, MIT/Apache) e descarte da imagem; demais → UIA com orçamento de 200 ms e, se estourar ou vier vazia, OCR;
  - sessão WGC por janela em primeiro plano, contando quadros e área de `DirtyRegions`.
- Grava a cada 5 min e no fim, em `%LOCALAPPDATA%\aura-spikes\s3\resumo-AAAAMMDD.json`, só números por app: eventos, leituras, ms p50/p95, caracteres (contagem), tamanhos de imagem, quadros WGC, CPU e RAM próprios. Encerra em 10 h ou com Ctrl+C, com o fechamento da janela de console, ou criando o arquivo `parar` na pasta.
- **Verificações curtas no Chrome (AC-011):** perfil temporário, página leve; (a) cliente lê só o `Name` da raiz → estado de `chrome://accessibility`; (b) depois de uma leitura completa, o cliente encerra → `chrome://accessibility` lido a cada 10 s até "Web accessibility" desligar, com limite de 10 min.

## Seams e validação

Protótipos não têm suíte de produto. Cada binário ganha:
- testes de unidade do que é puro: fusões, recall, critério de escolha, recuo, agregação do resumo;
- execução registrada como evidência, com comando, ambiente e saída.

Comandos:
- `cargo test --manifest-path tools/spikes/<spike>/Cargo.toml`
- `cargo run --release --manifest-path tools/spikes/<spike>/Cargo.toml -- <args>`

## Gates

- G1: spec aceito (delegação do usuário).
- G3: pacote pronto antes de cada ticket.
- Evidência só depois de execução real.
- Medições pesadas com prioridade baixa; nada de testes que travem o navegador do usuário: as páginas dinâmicas do T3 não se repetem.
- Restaurar o plano de energia no fim.
