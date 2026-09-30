---
schema: hybrid/spec
schema_version: "1.0"
effort_id: 007-anexos-multimodais
revision: 1
status: accepted
profile: standard
---

# Specification: Anexos multimodais

## Problem and desired result

O usuário quer arrastar para o Overlay um PDF, uma planilha, um áudio de reunião ou um vídeo e perguntar sobre ele. Os modelos aceitam texto e imagem; o restante precisa ser convertido com fidelidade e sem estourar o contexto. O resultado é anexar qualquer arquivo comum, ver um Chip com o que será enviado (páginas, abas, duração, estimativa de tokens), e o agente receber uma representação útil — texto estruturado, tabelas, imagens de páginas, keyframes e Transcrição — além do arquivo original no Workspace da conversa para inspeção detalhada com ferramentas.

## Consumers and actors

- Usuário (arrastar, colar, escolher arquivos).
- Agente (recebe conteúdo ingerido; pode ler o original no workspace no Modo Tarefa ou via ferramenta de leitura).
- 002 (`ContextTray`/`TurnInput`), 006 (`Transcriber`), 004 (`media.keyframes` no worker).

## Scope

### Included

- Pipeline de Ingestão com limites, estimativa de tokens e cache por hash.
- PDF: texto por página + imagens de páginas (todas ou selecionadas; escaneados via imagem/OCR).
- Planilhas: xlsx, xls, xlsm, ods, csv, tsv → esquema por aba + amostra em tabela Markdown + estatísticas.
- Documentos: docx, pptx, odt, rtf, txt, md, html, json, xml, yaml, código-fonte.
- Áudio: mp3, m4a, wav, ogg, opus, flac, webm → Transcrição com tempos.
- Vídeo: mp4, mov, mkv, webm, avi → keyframes + Transcrição da trilha.
- Cópia do original para o Workspace da conversa e ferramenta de leitura detalhada.

### Excluded

- Edição/geração de arquivos Office (o agente pode gerar arquivos no Modo Tarefa, fora deste esforço).
- RAG/índice vetorial persistente entre Conversas (futuro).
- Arquivos protegidos por senha (erro explicativo).

## User journeys and scenarios

### US-001 — Anexar e ver o que vai ser enviado (Priority: P1)

#### Acceptance scenarios

- **AC-001** — Dado o Overlay, quando o usuário arrasta, cola ou escolhe (`@arquivo`) um ou mais arquivos suportados, então cada um vira um Chip com ícone do tipo, nome, tamanho e resumo ("12 páginas", "3 abas · 1 240 linhas", "04:32 de áudio"), e uma estimativa de tokens; a Ingestão roda em segundo plano com progresso no Chip.
- **AC-002** — Dado um arquivo não suportado, protegido por senha, corrompido ou acima dos limites (200 MB; 2 h de áudio/vídeo; 500 páginas), quando anexado, então o Chip mostra o motivo e não é enviado.
- **AC-003** — Dado anexos cuja estimativa excede o orçamento do turno (padrão 60% da janela de contexto do modelo), quando o usuário tenta enviar, então o Aura oferece reduzir (páginas selecionadas, amostra menor, só Transcrição) antes de enviar.

### US-002 — Documentos e planilhas (Priority: P1)

#### Acceptance scenarios

- **AC-004** — Dado um PDF com texto, quando enviado, então o agente recebe o texto de cada página com marcador "[Página N]" e, para páginas com pouco texto (< 200 caracteres) ou marcadas pelo usuário, a imagem da página; um PDF escaneado vira imagens de páginas (até 20) + texto OCR.
- **AC-005** — Dado uma planilha, quando enviada, então o agente recebe, por aba: nome, dimensões, cabeçalhos inferidos com tipos (número, data, texto), as primeiras 50 linhas em tabela Markdown e estatísticas por coluna numérica (mín., máx., média, soma); fórmulas aparecem com o valor calculado salvo no arquivo.
- **AC-006** — Dado DOCX/PPTX/ODT/RTF/HTML/MD/TXT/código, quando enviados, então o agente recebe o texto com estrutura (títulos, listas, tabelas; slides numerados com notas) em Markdown; código mantém a linguagem no bloco.

### US-003 — Áudio e vídeo (Priority: P2)

#### Acceptance scenarios

- **AC-007** — Dado um arquivo de áudio, quando enviado, então o agente recebe a Transcrição com marcações de tempo a cada segmento, usando o Modelo ASR configurado (006).
- **AC-008** — Dado um vídeo, quando enviado, então o agente recebe até 12 keyframes distribuídos (com carimbo de tempo) e a Transcrição da trilha de áudio intercalada por tempo.

### US-004 — O original fica disponível (Priority: P2)

#### Acceptance scenarios

- **AC-009** — Dado qualquer anexo enviado, quando o agente precisa de detalhe (outra aba, página específica, trecho do vídeo), então pode chamar a ferramenta `attachment_read{attachment, selector}` (páginas, aba e intervalo de linhas, intervalo de tempo) e receber o conteúdo; o arquivo original está em `workspaces/<id>/attachments/`.
- **AC-010** — Dado o mesmo arquivo (mesmo hash) anexado de novo, quando ingerido, então o resultado vem do cache sem reprocessar.

## Requirements

- **FR-001** — O sistema MUST ingerir anexos localmente, mostrando resumo, progresso, estimativa de tokens e erros no Chip.
- **FR-002** — O sistema MUST converter PDFs, planilhas e documentos em texto estruturado e imagens de páginas quando útil.
- **FR-003** — O sistema MUST transcrever áudio e extrair keyframes + Transcrição de vídeos.
- **FR-004** — O sistema MUST respeitar limites de tamanho e orçamento de contexto, oferecendo redução antes de enviar.
- **FR-005** — O sistema MUST manter os originais no Workspace da conversa e oferecer ao agente leitura seletiva por ferramenta.
- **FR-006** — O sistema SHOULD reutilizar Ingestões pelo hash do conteúdo.

## Limits, errors, and compatibility

- Tudo roda localmente (worker para PDF/vídeo/áudio); nenhum arquivo é enviado a serviços de conversão.
- Estimativa de tokens: heurística de 4 caracteres/token para texto e custo fixo por imagem (configurável por modelo).
- Formatos legados (.doc, .ppt): tentar conversão por IFilter do Windows se disponível; senão não suportado.

## Hypotheses and dependencies

- H-015: PDFium + `calamine` + extração XML cobrem ≥ 95% dos arquivos de um conjunto de 100 arquivos reais sem erro. Check: TK-001–TK-003.
- Dependências: 002 TK-005 (Chips), 006 TK-003 (worker/Transcriber), 004 TK-007 (`media.keyframes`), 004 TK-004 (servidor MCP).

## Success criteria

### Delivery-verifiable

- **SC-001** — AC-001–AC-010 com evidência; corpus de teste com 30 arquivos (fixtures versionados de licença livre) cobrindo todos os formatos.

### Post-delivery observation

- **SC-002** — < 3% de anexos com erro de Ingestão no beta.

## Decisions and open questions

- Orçamento de anexos por turno: 60% da janela de contexto do modelo; o restante fica para conversa e resposta.
- Nenhuma pergunta pendente.
