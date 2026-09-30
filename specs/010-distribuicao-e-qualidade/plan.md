---
schema: hybrid/plan
schema_version: "1.0"
effort_id: 010-distribuicao-e-qualidade
revision: 1
spec_revision: 1
status: ready
---

# Plan: Distribuição e qualidade

## Summary

Configurar o bundler do Tauri para NSIS por usuário com assinatura, o `tauri-plugin-updater` com manifesto assinado e canais, um passo de pré-atualização que baixa e verifica a nova versão do app-server, o onboarding como janela própria, a automação do `aura-bench` no CI Windows com linha de base versionada, a auditoria de acessibilidade (axe + roteiro com Narrador) e o pacote de diagnóstico redigido.

## Technical context

- Language/runtime: Rust, React, GitHub Actions (`windows-latest`) + máquina de referência (runner self-hosted opcional).
- Dependencies: Tauri bundler NSIS (`installMode: currentUser`, `webviewInstallMode: downloadBootstrapper`), `tauri-plugin-updater`, `tauri-plugin-process`, assinatura via `signtool`/Azure Trusted Signing, `axe-core` (Vitest/E2E), `i18next` + `react-i18next`, `zip` crate.
- Storage/data: `bench/baseline.json` versionado; `docs/qa/` com roteiros.
- Test command: `pnpm tauri build`; `cargo run -p aura-bench -- all --out bench/latest.json --baseline bench/baseline.json`; `pnpm -C apps/desktop test -- a11y i18n`; `pnpm -C apps/desktop e2e`.
- Target/platform: Windows 10 22H2, Windows 11.

## Consumed contract

- Spec: `spec.md`, revision 1.
- Requirements and acceptance refs: FR-001–FR-006; AC-001–AC-011.

## Modules, interfaces, consumers, and seams

| Module | Interface | Consumidores | Seam de teste |
| --- | --- | --- | --- |
| `apps/desktop/src-tauri/tauri.conf.json` (bundle) | NSIS currentUser, ícones, `externalBin` (worker), recursos | CI release | Build + instalação em VM |
| `apps/desktop/src-tauri/src/update.rs` | `UpdateService::{check(channel), download, apply_when_idle}`; `pre_update_hook` baixa `codex-app-server` da nova versão (via `CodexBinary::ensure`) | host | `wiremock` servindo manifesto assinado com chave de teste |
| `apps/desktop/src/onboarding/` | Assistente de 5 passos, estado persistido | usuário | Vitest + E2E |
| `tools/aura-bench` (existente) | `all`, comparação com baseline, código de saída ≠ 0 em regressão > 20% | CI | Execução real |
| `apps/desktop/src/i18n/` | chaves pt-BR/en, detecção de idioma | UI | Teste de cobertura de chaves |
| `apps/desktop/src-tauri/src/diagnostics.rs` | `DiagnosticsReport`, `export_zip(path)` com redação | UI, suporte | Unit de redação + teste do conteúdo do zip |

## Chosen approach and alternatives

- **NSIS por usuário** (sem UAC) em vez de MSI por máquina: combina com "leve" e com usuários sem admin.
- **Updater do Tauri** com manifesto assinado; app-server atualizado junto (nunca sozinho) para manter o contrato fixado (ADR 0001).
- **Benchmark contínuo com linha de base** para impedir regressões de leveza.
- Alternativas descartadas: MSIX/Store (sandbox de pacote complica hooks/captura — reavaliar), Squirrel.

## Data, compatibility, and external dependencies

- Chaves: privada do updater e certificado de assinatura em segredos do CI; nunca no repositório.
- Manifesto por canal: `https://updates.<domínio>/aura/{stable|beta}/latest.json`.

## Verification strategy

| AC | Nível | Oráculo | Procedimento |
| --- | --- | --- | --- |
| AC-001–AC-003 | Roteiro em VM limpa (Win10 22H2, Win11) + script PowerShell de verificação | Spec | Instalação sem UAC, tamanho, atalhos, desinstalação com/sem dados |
| AC-004, AC-005 | Integração `wiremock` + manual com canal de teste | Spec | Assinatura inválida descartada; app-server novo verificado antes da troca |
| AC-006 | Vitest + E2E | Spec | Passos, pular/retomar |
| AC-007, AC-008 | `aura-bench` na máquina de referência + CI | Números da spec | Relatório anexado; CI falha em regressão |
| AC-009 | axe (0 violações sérias) + roteiro Narrador + teclado | WCAG 2.1 AA | `docs/qa/acessibilidade.md` |
| AC-010 | Teste de cobertura de chaves + E2E em en | Spec | Nenhuma chave faltando; UI em inglês |
| AC-011 | Unit de redação + inspeção do zip | Spec | Zip sem `sk-`, `eyJ`, conteúdo de conversa |

## Change map

| Path | Existing/new | Symbol or section | Purpose | Reference revision |
| --- | --- | --- | --- | --- |
| `apps/desktop/src-tauri/tauri.conf.json` | existing | `bundle`, `plugins.updater` | Distribuição | 001 TK-001 |
| `.github/workflows/release.yml` | new | build, assinatura, publicação | Release | 2026-09-29 |
| `apps/desktop/src-tauri/src/{update.rs,diagnostics.rs}` | new | serviços | Host | 2026-09-29 |
| `apps/desktop/src/{onboarding,diagnostics,i18n}/**` | new | UI | Interface | 2026-09-29 |
| `tools/aura-bench/src/{all.rs,baseline.rs}`, `bench/baseline.json` | new | benchmark contínuo | Qualidade | 2026-09-29 |
| `docs/qa/{instalacao.md,acessibilidade.md}` | new | roteiros | QA | 2026-09-29 |

## Derived technical obligations

- **OT-001** → FR-002: o app nunca aplica atualização com turno ativo ou gravação manual em andamento.
- **OT-002** → FR-006: exportação de diagnóstico passa pela mesma redação dos logs (001 TK-006) e por varredura final de padrões de segredo.
- **OT-003** → FR-004: `bench/baseline.json` só é atualizado por PR com justificativa.

## Risks and gates

- SmartScreen pode alertar em builds novos até ganhar reputação — Azure Trusted Signing/EV reduz.
- Runners do GitHub não refletem máquina real para memória/latência — baseline de CI é só anti-regressão; aceitação final na máquina de referência.
- G2: satisfeito. G3: TK-001 depende de 001 TK-001; TK-004 depende de 001 TK-003.
