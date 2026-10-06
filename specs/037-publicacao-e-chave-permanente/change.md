---
schema: hybrid/change
schema_version: 1.0
effort_id: 037-publicacao-e-chave-permanente
revision: 2
status: accepted
profile: compact
---

# Change: Auditoria para abertura pública e chave permanente do updater

## Objetivo e limites

Renomear a branch de trabalho para release/0.2.0, auditar arquivos/histórico/releases/artefatos antes de tornar VIDORETTO/aura-windows público e preparar atualizações contínuas com a chave nova permanente. Não publicar release nesta etapa. Preservar modificações locais e arquivos de QA não versionados.

## Contrato de comportamento

- Entradas: [tipo e pré-condições]
- Saída: [resultado observável]
- Erros/invariantes: [casos relevantes]
- Compatibilidade: [o que deve permanecer igual]

## Requisitos e aceite

- **FR-001** — A branch MUST ser renomeada no GitHub e localmente, preservando o SHA.
- **AC-001** — release/0.2.0 foi renomeada preservando SHA 3c972e2cc0f2dbdc850a7d6533b40ae54de6c4a8; após anonimização autorizada, SHA 7694d6ef23bdcaa5fcee448f77f3f57e4c506025 tem a mesma árvore, com upstream correspondente e diff local preservado.
- **FR-002** — A abertura pública MUST ocorrer somente após revisão redigida dos refs/histórico, arquivos atuais, release e artefatos expostos; decisões de dados pessoais devem ser respeitadas.
- **AC-002** — Nenhum segredo real não remediado é identificado no escopo auditado; false positives identificados; acesso anônimo ao repositório e latest.json funciona após a mudança de visibilidade.
- **FR-003** — A chave nova MUST permanecer estável nas versões futuras; usar o mesmo endpoint já gravado na 0.1.0.
- **AC-003** — pubkey atual não muda; endpoint é aura-windows/releases/latest/download/latest.json; documento de rotação explica assinatura de transição pela antiga SOMENTE se autorizada pelo usuário, e chave nova em diante. Não persistir segredos no Git ou logs.
- Decisão do usuário: único instalador da 0.1.0, liberdade para ajustar a migração; Gmail deve ser anonimizado antes da abertura. Chave nova permanente reutilizada, antiga aposentada; aprovação adicional pendente para resolver caches de commits no GitHub por repositório limpo na mesma URL ou suporte.

## Leitura e mapa de alterações

- `[path]` → `[symbol]` — [motivo]; [existing/new].

## Plano breve

Seam: [interface sob teste]. Abordagem: [decisão técnica local]. Dependências: [none ou refs].

## Sequência e tarefas

- [ ] C-001 Escrever caso relevante e observar red pelo motivo esperado.
- [ ] C-002 Implementar o mínimo e observar green.
- [ ] C-003 Executar regressão e registrar evidência.

## Validação e evidência

Comando/procedimento: `[exact command]`

Resultado executado: [pending]

Limitações: [o que não foi verificado]. Comando apenas identificado na configuração: [none ou registro].

## Condição de retorno

Retornar se o contrato, símbolo, dependência ou decisão material não puder ser satisfeito sem ampliar o escopo.

## Estado

`change.md` é a fonte canônica deste esforço compacto. Não crie `tasks.md` ou tickets paralelos para esta mudança.
