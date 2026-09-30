---
schema: hybrid/ticket
schema_version: 1.0
id: TK-001
effort: 010-distribuicao-e-qualidade
type: delivery
status: in_progress
ticket_revision: 2
requires: []
requirement_refs: ["FR-001"]
acceptance_refs: ["AC-001", "AC-002", "AC-003"]
spec_revision: 1
plan_revision: 1
owned_areas: ["apps/desktop/src-tauri/tauri.conf.json", "apps/desktop/src-tauri/installer", ".github/workflows/release.yml", "docs/qa/instalacao.md"]
verification_status: not_run
last_update: Implementação iniciada nesta sessão Linux (ver docs/HANDOFF.md).
---


# TK-001 — Instalador por usuário assinado e desinstalação limpa

## Objetivo e limites

Entrega a configuração NSIS `currentUser`, bootstrapper do WebView2, assinatura (Azure Trusted Signing ou certificado disponível), o workflow `release.yml`, a desinstalação com escolha de apagar dados (inclui revogar a sessão ChatGPT e apagar credenciais do Aura no cofre) e o roteiro de QA em VMs.

Não inclui: updater (TK-002).

## Leitura em ordem

1. `specs/010-distribuicao-e-qualidade/plan.md` → bundle, riscos.
2. `apps/desktop/src-tauri/tauri.conf.json` (001 TK-001).
3. Docs atuais Tauri 2: NSIS (`installMode`, hooks `NSIS_HOOK_PREUNINSTALL`), `webviewInstallMode`, assinatura no Windows.
4. `crates/aura-auth/src/siwc.rs` (002 TK-002) → `revoke`.

## Decisões já resolvidas

- `installMode: currentUser`; destino `%LOCALAPPDATA%\Programs\Aura`; sem serviço/driver.
- `webviewInstallMode: downloadBootstrapper` (silencioso por usuário).
- Desinstalação: página com caixa "Apagar também meus dados do Aura"; se marcada, executa `aura.exe --purge` (revoga SIWC, apaga cofre `Aura/*`, dados e modelos) antes de remover arquivos.
- Sem certificado disponível → build não assinado marcado "interno" (não publicar no canal Estável).
- Liberdade local: textos do instalador.

## Mapa de alterações

- Existente: `apps/desktop/src-tauri/tauri.conf.json` → `bundle.windows.nsis`, `bundle.windows.signCommand`/`certificateThumbprint`.
- Novo: `apps/desktop/src-tauri/installer/hooks.nsh`; comando CLI `--purge` no `main.rs` (antes de inicializar a UI e o single-instance).
- Novo: `.github/workflows/release.yml`; `docs/qa/instalacao.md`.

## Contrato técnico

- Entradas: build de release.
- Saídas: `Aura_<versão>_x64-setup.exe` assinado ≤ 15 MB.
- Invariantes: nenhum passo requer UAC.
- Erros: WebView2 indisponível offline → mensagem com link oficial.

## Exemplos de aceite

- **AC-001**: VM Win11 limpa, usuário padrão → instalar sem UAC; `Get-Item` do setup ≤ 15 MB; atalho no Menu Iniciar; ícone na bandeja; `Get-AuthenticodeSignature` = `Valid` (quando assinado).
- **AC-002**: VM Win10 22H2 sem WebView2 → instalador baixa e instala por usuário; Aura abre.
- **AC-003**: desinstalar sem marcar → `%LOCALAPPDATA%\Aura` preservado; marcando → pasta removida, `cmdkey /list | findstr Aura` vazio, e o log do `--purge` mostra tentativa de revogação.

## Dependências e sequência de execução

Depende de: 001-fundacao-overlay/TK-001 e 002-conversa-agente-codex/TK-002 (outros esforços).

- [ ] TK-001.1 Configurar NSIS e gerar setup (tamanho).
- [ ] TK-001.2 `--purge` com teste de integração (cofre de teste) red→green.
- [ ] TK-001.3 Hooks de desinstalação; assinatura; workflow.
- [ ] TK-001.4 Roteiro em VMs; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `pnpm -C apps/desktop tauri build`; `cargo nextest run -p aura-desktop purge`; roteiro `docs/qa/instalacao.md` em VM Win10 e Win11.
- Estado esperado: setup ≤ 15 MB, instalação/desinstalação conforme exemplos.
- Comando identificado na configuração mas não executado: `signtool`/Trusted Signing depende do certificado (Q-005).
- Distinguir defeito de ambiente: ausência de certificado → assinatura `not_run`.

## Condição de retorno à planejadora

Retornar se o setup ultrapassar 15 MB por causa do worker/PDFium (decidir download sob demanda do worker).

## Relatório de saída

Relatar tamanho, assinatura, roteiro executado, EV refs.
