# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `—`
- Acceptance: `AC-002`
- Procedure: `Gitleaks redigido history e snapshot; revisão de alertas; busca literal e base64 em blobs; artefato bench e metadata release`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:cbe8da963cd7eb0e2e86174734e6064fa1eded901bccc01899264ff69bb466ad`
- Timestamp: `2026-10-06T01:15:35+00:00`
- Observations: 61 commits, 1857 blobs e snapshot994 arquivos; 20 alertas falsos positivos revisados, nenhum segredo real encontrado. GitHub mantém commit antigo com Gmail por SHA, verificado via API; abertura suspensa até decisão de caches.
- Evidence refs: none
- Limitations: none recorded

## EV-002 — passed

- Ticket: `—`
- Acceptance: `AC-001`
- Procedure: `GitHub rename branch API; git-filter-repo em mirror isolado; force-with-lease de branches/tag; comparação de árvores e alinhamento de refs locais`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:e86c1b90bc7e43418d856978bcf6c776092db4f95d285308f95b6c032f84a6c1`
- Timestamp: `2026-10-06T01:15:49+00:00`
- Observations: release/0.2.0 no GitHub e local; árvores iguais; diff não commitado idêntico; nenhum autor/committer Gmail nos refs sanitizados; backup bundle preservado fora da publicação
- Evidence refs: none
- Limitations: none recorded

## EV-003 — partial

- Ticket: `—`
- Acceptance: `AC-003`
- Procedure: `Comparação da pubkey configurada com pública permanente; endpoint estável; node scripts/check-updater-key.mjs; pnpm -C apps/desktop typecheck`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:c929eaa82966889f8d0ff88ee37e1a88158dfad98c191b21b41c088469060349`
- Timestamp: `2026-10-06T01:17:11+00:00`
- Observations: Par novo existente reutilizado, cópia canônica fora do Git, endpoint aura-windows restaurado, guarda de chave e tipos passaram. Chave antiga aposentada.
- Evidence refs: none
- Limitations: Secret no repositório final, reabertura pública, build para endpoint restaurado e nova release ainda pendentes; nenhum upload de release realizado
