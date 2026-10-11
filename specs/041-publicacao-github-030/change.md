---
schema: hybrid/change
schema_version: 1.0
effort_id: 041-publicacao-github-030
revision: 1
status: accepted
profile: compact
---

# Publicação profissional do Aura 0.3.0

## Objetivo e limites

Pedido explícito de 11/10/2026: atualizar GitHub, remover outras branches, publicar o código atual e criar release. Preparar fonte/documentação/artefatos revisáveis, então publicar sem nova confirmação. Baseline ed9a9654c6e76e48186fc010a12eba77023b00f5; preservar os diffs implementados038/039/040 e arquivos pessoais não versionados.

## Contrato de comportamento

- Versão0.3.0 reúne seletores/raciocínio, busca/leitura web gratuita e correção de posição do Overlay. Não substituir a tag/artefatos históricos0.2.0.
- Repositório público VIDORETTO/aura-windows, branch principal main, README e metadados atuais. Manter autoria noreply já adotada.
- Release v0.3.0 com NSIS/MSI, assinaturas updater da chave permanente, latest.json e SHA256. Endpoint/pública/configuração pessoal preservados. Nunca exibir/commitar privada nem reutilizar chave antiga.
- Remover branches adicionais somente após seus commits estarem alcançáveis em main ou preservados; manter tags/releases. Na descoberta, só main e release/0.2.0, ambas em ed9a965.
- Publicar código e documentos técnicos, excluindo captures, modelos, bins, perfis SQLite/rollouts e caches Python. Preservar esses arquivos no disco.
- Actions continua desativado conforme037; build/teste/empacotamento locais comprovados. Gates150/200% de038 continuam declarados, sem aprovação fictícia.

## Requisitos e aceite

- **FR-001** — Publicar fonte atual e documentação profissional.
- **AC-001** — main remota contém o código atual038/039/040, versão0.3.0 coerente nos manifests, documentação e metadados verificáveis; paths/links e checks passam, sem artefatos pessoais selecionados para Git.
- **FR-002** — Publicar nova versão verificável pelo updater permanente.
- **AC-002** — release não draft v0.3.0 aponta ao commit publicado; assets/version/hashes reais; assinaturas conferidas com a pública permanente; manifesto anônimo aponta a esses assets, sem trocar endpoint.
- **FR-003** — Consolidar branches sem perder commits.
- **AC-003** — listagem remota/local ao final contém somente main, conservando o histórico dos commits da branch removida e as releases0.1/0.2 anteriores.

## Leitura e mapa de alterações

README.md/CHANGELOG.md/CONTRIBUTING.md/docs/HANDOFF.md e docs/usage.md; Cargo.toml/Cargo.lock/package.json/tauri.conf.json; aura-web metadata HTTP/MCP usa CARGO_PKG_VERSION; .gitignore; scripts/prepare-sidecars.ps1/check-updater-key.mjs; .github/workflows; specs038–041. Templates/support existentes apenas se necessário. Código funcional novo fora da publicação, exceto correção comprovada de gate.

## Plano breve

Seams: Git remoto/GitHub API pública e arquivos de instalador assinados; oráculos versão literal0.3.0, pública permanente atual e reachability Git. Incremento minor por funcionalidade web, não reutilizar tag. README upgrade com fatos/asset/logo existentes; detalhes extensos de uso podem ir para docs. Gitleaks redigido no snapshot selecionado e comparação com fixtures anteriores. Usar chave local protegida somente em variável do processo, sem log.

Empacotamento Tauri normal sem e2e/demo, worker DirectML atualizado. Se aura.exe estiver bloqueado por instância ativa, preparar artefato isolado sem encerrá-la. Release rascunho para verificar upload e assinaturas antes de publicar; update só após manifesto correto. Nada de force-push nem reescrita do histórico atual.

## Sequência e tarefas

- [ ] C-001 Descoberta, fonte limpa para publicação e documentos profissionais.
- [ ] C-002 Versão, checks, worker/app e instaladores assinados.
- [ ] C-003 Auditoria, commit/main/tag e assets em rascunho verificáveis.
- [ ] C-004 Publicar/validar endpoint anônimo e remover branch redundante.
- [ ] C-005 Evidências/revisão/checkpoint e relatório final.

## Validação e evidência

Executar testes Rust workspace Windows, UI/typecheck/build, fmt/clippy, guard updater, audit source snapshot, version/resource dos artefatos, verificação Ed25519 updater, hashes/download público e consulta Git/Release ao final. Comandos/exits concretos serão registrados após execução. Versão/readme editorial não exige teste unitário que apenas repete manifests.

## Condição de retorno

Se assinatura permanente inacessível/incompatível, conteúdo operacional sensível detectado ou branch com trabalho não preservável, concluir a preparação e relatar o bloqueio real. Nunca publicar arquivo unsigned como update verificado.

## Estado

Compacto: change.md é fonte canônica; sem tickets/plan/todo paralelos. Pedido já autoriza operações remotas necessárias, incluindo exclusão de branches redundantes após preservar commits.
