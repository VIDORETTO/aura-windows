# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — stale

- Ticket: `—`
- Acceptance: `AC-002`
- Procedure: `Gitleaks redigido history e snapshot; revisão de alertas; busca literal e base64 em blobs; artefato bench e metadata release`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:cbe8da963cd7eb0e2e86174734e6064fa1eded901bccc01899264ff69bb466ad`
- Timestamp: `2026-10-06T01:15:35+00:00`
- Observations: 61 commits, 1857 blobs e snapshot994 arquivos; 20 alertas falsos positivos revisados, nenhum segredo real encontrado. GitHub mantém commit antigo com Gmail por SHA, verificado via API; abertura suspensa até decisão de caches.
- Evidence refs: none
- Limitations: Evidence invalidated because an input changed.

## EV-002 — stale

- Ticket: `—`
- Acceptance: `AC-001`
- Procedure: `GitHub rename branch API; git-filter-repo em mirror isolado; force-with-lease de branches/tag; comparação de árvores e alinhamento de refs locais`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:e86c1b90bc7e43418d856978bcf6c776092db4f95d285308f95b6c032f84a6c1`
- Timestamp: `2026-10-06T01:15:49+00:00`
- Observations: release/0.2.0 no GitHub e local; árvores iguais; diff não commitado idêntico; nenhum autor/committer Gmail nos refs sanitizados; backup bundle preservado fora da publicação
- Evidence refs: none
- Limitations: Evidence invalidated because an input changed.

## EV-003 — stale

- Ticket: `—`
- Acceptance: `AC-003`
- Procedure: `Comparação da pubkey configurada com pública permanente; endpoint estável; node scripts/check-updater-key.mjs; pnpm -C apps/desktop typecheck`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:c929eaa82966889f8d0ff88ee37e1a88158dfad98c191b21b41c088469060349`
- Timestamp: `2026-10-06T01:17:11+00:00`
- Observations: Par novo existente reutilizado, cópia canônica fora do Git, endpoint aura-windows restaurado, guarda de chave e tipos passaram. Chave antiga aposentada.
- Evidence refs: none
- Limitations: Secret no repositório final, reabertura pública, build para endpoint restaurado e nova release ainda pendentes; nenhum upload de release realizado Evidence invalidated because an input changed.

## EV-004 — passed

- Ticket: `—`
- Acceptance: `AC-001`
- Procedure: `Consultar branch, autoria remota e acesso anônimo ao SHA pessoal original; confirmar arquivos locais do usuário preservados.`
- Execution: `executed`
- Environment: GitHub API/web anônimo; Windows 11; working tree local
- Tested revision: `local:86c55abb2e50d3c04e5bb1d73fa399823593243be21b85748dc69eb94e3c04b4`
- Timestamp: `2026-10-06T02:18:13+00:00`
- Observations: Branch release/0.2.0 publicada; URL nova pública; histórico sem Gmail; endpoint web para commit pessoal antigo retorna HTTP 404; dirs locais preexistentes permanecem.
- Evidence refs: `target/qa-tools/publication-final-status.json`
- Limitations: none recorded

## EV-005 — passed

- Ticket: `—`
- Acceptance: `AC-002`
- Procedure: `Auditar 61 commits/1.914 blobs e snapshot final; executar Gitleaks e revisão adicional de marcadores e chaves.`
- Execution: `executed`
- Environment: Gitleaks 8.30.1; snapshot final local; GitHub sem autenticação
- Tested revision: `local:346615056372c2f6279336fb0cb5c851d882009899b367ceba2798ba0fd31070`
- Timestamp: `2026-10-06T02:18:13+00:00`
- Observations: 20 alertas sintéticos classificados; auditoria adicional sem marcadores; repo e release HTTP 200; feed público anuncia 0.2.0 e URL correta.
- Evidence refs: `target/qa-tools/publication-final-gitleaks.json`
- Limitations: Os 20 alertas correspondem a fixtures e dados de teste sintéticos classificados; scanner não substitui auditoria integral do aplicativo.

## EV-006 — passed

- Ticket: `—`
- Acceptance: `AC-003`
- Procedure: `Verificar assinatura permanente e rejeição pela chave aposentada e bytes adulterados; conferir configuração/manifesto e secret pelo nome cadastrado.`
- Execution: `executed`
- Environment: Verificador Minisign local; Tauri config; GitHub secret metadata
- Tested revision: `local:ecb85b44ee3148911e8491e3de91779e4b537a4969065954d0d3a8f00405131f`
- Timestamp: `2026-10-06T02:18:13+00:00`
- Observations: Prova assinada validada apenas pela chave permanente; secret TAURI_SIGNING_PRIVATE_KEY configurado; updater endpoint permanece estável.
- Evidence refs: `target/qa-tools/release-020-signature-final.log`
- Limitations: none recorded

## EV-007 — passed

- Ticket: `—`
- Acceptance: `AC-004`, `AC-005`
- Procedure: `Executar atualização in-app real do instalador público 0.1.0 para 0.2.0 em pasta/perfil isolados; validar persistência do provedor, consulta posterior do feed e artefatos da release.`
- Execution: `executed`
- Environment: Windows 11; WebView2; instalador público; WebDriver; endpoint GitHub anônimo
- Tested revision: `local:5b3a0c5f1989a69450193c003d7d328a3399dbeb03e48bbafd61161410754a28`
- Timestamp: `2026-10-06T02:18:14+00:00`
- Observations: O mesmo executável atualizou a 0.2.0 pelo app; perfil Ollama persistiu; versão instalada consultou feed e mostrou atualizada; release pública não draft tem seis artefatos e manifesto correto.
- Evidence refs: `target/qa-tools/release020-live-updater-install.log`
- Limitations: Build hospedado de release com LLVM/DirectML ainda não validado; Actions desativado até essa verificação.
