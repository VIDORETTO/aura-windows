---
schema: hybrid/ticket
schema_version: 1.0
id: TK-006
effort: 004-contexto-de-tela
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-005"]
requirement_refs: ["FR-005"]
acceptance_refs: ["AC-015", "AC-016"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-capture/src/recordings.rs", "apps/desktop/src/recordings", "apps/desktop/src-tauri/src/media_protocol.rs"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-006 — Gravação manual, modo Contínuo, Retenção e gerenciador de Gravações

## Objetivo e limites

Entrega Gravar/Parar (Modo Manual), Modo Contínuo com Retenção (dias e GB), a tela "Gravações" (listar, renomear, reproduzir, anexar a Conversa, excluir) e o protocolo `aura-media://` com player MSE.

Não inclui: exportar para MP4 não cifrado (futuro, exige confirmação), timeline pesquisável (CAND-015).

## Leitura em ordem

1. `specs/004-contexto-de-tela/spec.md` → AC-015, AC-016.
2. `crates/aura-capture/src/{segments.rs,retention.rs}` (TK-005).
3. `specs/004-contexto-de-tela/plan.md` → "Player de Gravações".
4. Docs atuais: Tauri `register_asynchronous_uri_scheme_protocol`, Media Source Extensions no WebView2.

## Decisões já resolvidas

- Gravação = sequência de segmentos `kind=recording` com `recording_id`; Contínuo = gravação automática que rola por dia (uma `recording` por dia/Fonte).
- Retenção Contínuo padrão 7 dias / 20 GB; ordem de exclusão: mais antigos primeiro; gravações manuais só entram na retenção se "aplicar retenção às manuais" estiver ativo (padrão desligado).
- `aura-media://recording/<id>/<n>` retorna segmento decifrado com `Content-Type: video/mp4`; só aceito a partir das janelas do Aura.
- Anexar Gravação → Chip de Recorte (TK-007 define amostragem; aqui o anexo usa a mesma função quando TK-007 estiver pronto — se não, anexar só a miniatura e registrar pendência).
- Liberdade local: layout da lista e do player.

## Mapa de alterações

- Novo: `crates/aura-capture/src/recordings.rs` → `Recordings::{start_manual, stop, list, rename, delete}`, `ContinuousScheduler`.
- Existente: `retention.rs` → política Contínuo (dias/GB).
- Novo: `apps/desktop/src-tauri/src/media_protocol.rs`.
- Novo: `apps/desktop/src/recordings/{RecordingsView.tsx,Player.tsx,RecordButton.tsx}`.

## Contrato técnico

- Entradas: ações Gravar/Parar; configuração de Retenção.
- Saídas: `Recording{id, source, started_at, ended_at, duration, bytes, title}`.
- Invariantes: OT-003; excluir gravação remove todos os segmentos.
- Erros: disco cheio → gravação encerrada com aviso e parte salva mantida.

## Exemplos de aceite

- **AC-015**: Gravar, aguardar 30 s, Parar → `Recordings::list` contém 1 item com duração 30 ± 1 s e 3 segmentos; player reproduz (manual); excluir → nenhum `.seg` com esse `recording_id`.
- **AC-016** (unit `retention::plan` contínuo): 10 dias de segmentos de 2 GB/dia, política 7 dias/20 GB → remove dias 1–3; com 3 GB/dia → remove até ≤ 20 GB (dias 1–4). Integração: inicializar com segmentos antigos fora da política → removidos no startup.

## Dependências e sequência de execução

Depende de: TK-005.

- [ ] TK-006.1 Unit retenção contínua red→green.
- [ ] TK-006.2 `Recordings` manual com `NullEncoder` (AC-015) red→green.
- [ ] TK-006.3 Protocolo de mídia + player (manual).
- [ ] TK-006.4 UI Gravações; Vitest; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-capture recordings retention`; `pnpm -C apps/desktop test -- recordings`; roteiro manual do player.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: codecs H.264 ausentes em Windows "N" (Media Feature Pack) → documentar requisito.

## Condição de retorno à planejadora

Retornar se o WebView2 não reproduzir fMP4 via MSE a partir de protocolo customizado.

## Relatório de saída

Relatar resultados e EV refs.
