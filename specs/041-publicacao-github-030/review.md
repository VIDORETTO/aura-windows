# Revisão de publicação — r1

Baseline fixo: `ed9a9654c6e76e48186fc010a12eba77023b00f5`. Inspeção inclui diff committed desde baseline (vazio), índice, unstaged e arquivos novos. Os esforços 038–040 têm revisões próprias; esta rodada revisa sua inclusão e os ajustes de distribuição, sem declarar uma nova auditoria linha a linha dos 30 mil lines de código/documentação acumulados.

## Standards

- README preserva logo e informações de uso (detalhes em docs/usage.md), contém links reais e não afirma CI habilitado, instalação limpa ou DPI não executados.
- Versão minor 0.3.0 coerente no workspace, lockfile, npm, Tauri e recursos reais. Versão do app-server fixada, sem alteração de contrato IPC nesta publicação.
- Os dois metadados de aura-web acompanham CARGO_PKG_VERSION; não alteram protocolo/capacidades ou algoritmo. Reavaliação explícita em qa.md/input-revalidation.json.
- `.gitignore` impede incluir caches, perfis operacionais de QA e capturas. Auditoria do índice não detectou privada operacional ou novos segredos. Os arquivos pessoais no disco foram preservados.
- Autoria noreply mantida. Não há force-push, substituição de tag/release antiga, troca da chave permanente ou ativação de Actions.
- Testes e build reais registrados em qa.md; limitações declaradas. Nenhum teste novo que apenas reproduza a edição editorial foi introduzido.

Sem achado bloqueante de Standards na preparação; controles remotos ainda devem ser executados antes do fechamento.

## Spec

- **AC-001:** documentação/fonte/versão e checks locais preparados; main remota pendente até push.
- **AC-002:** instaladores reais assinados e hash/version conferidos; rascunho/upload/download anônimo/publicação pendentes.
- **AC-003:** ambas branches remotas no baseline; excluir somente após main preservar esse histórico e a publicação ser conferida.

Não fechar os aceites remotos com evidência apenas local. Próxima ação: publicar main/tag, validar assets em rascunho e públicos, excluir a branch redundante e completar a revisão com resultados observados.
