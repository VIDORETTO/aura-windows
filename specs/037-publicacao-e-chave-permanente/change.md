---
schema: hybrid/change
schema_version: 1.0
effort_id: 037-publicacao-e-chave-permanente
revision: 3
status: accepted
profile: compact
---

# Change: abertura pública e atualização permanente do Aura

## Objetivo e limites

Publicar o código auditado em `VIDORETTO/aura-windows`, com autoria anonimizada,
e entregar Aura 0.2.0 na mesma URL de atualização já gravada na 0.1.0. O único
usuário da 0.1.0 deve poder atualizar dentro do app. O binário 0.2.0 deve levar a
chave permanente usada nas versões seguintes. Preservar os arquivos locais não
versionados do usuário e manter privado o backup do histórico anterior.

## Contrato de comportamento

- Entradas: branch `claude/serene-cori-t5pt94`, histórico auditado, decisão explícita
  de anonimizar e autorização de uso único da chave anterior para a transição.
- Saída: branch `release/0.2.0`, repositório público na URL original com novo ID,
  release 0.2.0 e feed do updater disponível sem autenticação.
- Invariantes: nenhuma chave privada no Git; endpoint permanece
  `https://github.com/VIDORETTO/aura-windows/releases/latest/download/latest.json`;
  assinatura antiga só na transição 0.2.0; chave pública permanente embutida no
  0.2.0 e chave privada permanente em secret do GitHub.
- Compatibilidade: preservar dados da instalação 0.1.0 e manter o pacote MSI/NSIS
  da versão anterior acessível no release 0.1.0.

## Requisitos e aceite

- **FR-001** — Renomear a branch profissionalmente e remover autoria pessoal do
  histórico que será público, preservando as árvores de conteúdo.
- **AC-001** — `release/0.2.0` existe local e remotamente; o histórico público não
  contém os endereços Gmail; o SHA antigo com autoria pessoal retorna 404 no repo
  novo; arquivos locais não versionados do usuário continuam presentes.
- **FR-002** — Auditar histórico, snapshot, releases e assets antes de abrir a URL.
- **AC-002** — Nenhuma credencial operacional não remediada nos 61 commits, blobs,
  arquivos e assets auditados; os 20 alertas sintéticos estão classificados; repo,
  feed e release respondem a solicitações anônimas.
- **FR-003** — Preservar a chave pública permanente e o endpoint gravado na 0.1.0.
- **AC-003** — A pública configurada corresponde ao par permanente, o secret do
  repositório contém esse par, e uma assinatura futura de prova passa apenas com
  essa chave; endpoint é idêntico na configuração e no feed publicado.
- **FR-004** — Permitir o salto assinado da 0.1.0 para a 0.2.0 usando a chave que
  a versão instalada já confia; após atualizar, validar com a chave permanente.
- **AC-004** — Instalar 0.1.0 em pasta e perfil de QA isolados, acionar atualização
  na interface, observar `aura.exe` avançar para 0.2.0 e confirmar que o perfil do
  provedor sobrevive; abrir o 0.2.0 instalado e confirmar que ele consulta o feed.
- **FR-005** — Publicar os instaladores, assinaturas e manifesto da versão 0.2.0.
- **AC-005** — Release não draft `v0.2.0` tem NSIS, MSI, assinaturas, `latest.json`
  e hashes públicos; o feed anuncia 0.2.0 e aponta ao instalador daquela release.

## Decisões registradas

O usuário pediu anonimizar antes de abrir, escolheu recriar na mesma URL com backup
privado e autorizou o uso único da chave antiga por ser o único instalador da 0.1.0.
O repo antigo fica arquivado e privado; Actions permanece desativado até validar o
build hospedado. A release já publicada atualiza a 0.1.0; a chave antiga não será
usada nas releases seguintes.

## Leitura e mapa de alterações

- `.github/workflows/release.yml` — build Windows manual, LLVM fixo e worker DirectML.
- `apps/desktop/src-tauri/tauri.conf.json` — chave pública permanente e endpoint estável.
- `apps/desktop/e2e/specs/updates.e2e.ts` — verificação e instalação pela interface.
- `docs/qa/publicacao-e-chave-permanente.md` e `docs/qa/release-020-2026-10-05.md` —
  auditoria, decisões e limitações restantes.

## Plano breve

Seam: release pública do GitHub e botão de atualização na interface Tauri instalada.
Validar por acesso anônimo, assinatura Minisign e instalação isolada real 0.1.0 → 0.2.0.

## Validação e evidência

Procedimentos executados: `gitleaks git/dir`, verificações GitHub anônimas, checker da
chave, typecheck, verificação de assinaturas com as duas chaves, WebDriver sobre os
instaladores 0.1.0 e 0.2.0, atualização no app e nova consulta do feed após reinício.

Resultado: AC-001..005 passaram; evidências atuais EV-004..EV-007.

Limitações: build hospedado com LLVM/DirectML ainda não foi executado; gates de QA não
relacionados à transição estão discriminados no relatório 036. Nenhuma chave privada
ou dado de usuário foi incluído nos artefatos públicos.

## Estado

`change.md` é a fonte canônica deste esforço compacto. Não crie `tasks.md` ou tickets
paralelos para esta mudança.
