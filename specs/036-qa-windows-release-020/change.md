---
schema: hybrid/change
schema_version: 1.0
effort_id: 036-qa-windows-release-020
revision: 3
status: accepted
profile: compact
---

# Change: QA real Windows e atualização 0.1.0 para 0.2.0

## Objetivo e limites

Validar no Windows a branch serene-cori-t5pt94 (baseline 3c972e2), preservar dados existentes e preparar um candidato 0.2.0. Decisão do usuário em 05/10: interromper publicação, trocar a chave exposta e manter o código privado; planejar um canal público de binários e reinstalação de transição da 0.1.0.

## Contrato de comportamento

- Entradas: build de produção, perfil AURA_HOME isolado, app-server fixado e instalador 0.1.0 publicado.
- Saída: jornadas Windows executadas, evidências e candidato assinado.
- Erros/invariantes: nenhum segredo em evidências; não usar os dados pessoais como fixtures.
- Compatibilidade: identifier e dados anteriores preservados; chave nova deliberadamente incompatível com a 0.1.0, que requer reinstalação de transição.

## Requisitos e aceite

- **FR-001** — Executar regressão e jornadas reais do shell com dados isolados.
- **AC-001** — Rust, UI e tipos passam; build Windows de produção compila.
- **AC-002** — Notas, lembretes, biblioteca/reunião e configurações novas atravessam o IPC real e persistem após reiniciar, ou seus defeitos ficam reproduzidos e registrados.
- **FR-002** — O updater usa chave nova, mantém a identidade do app e comunica o resultado verdadeiro da consulta. O novo canal planejado é VIDORETTO/aura-releases; nenhuma publicação nesta sessão.
- **AC-003** — Com chave configurada, Verificar atualizações retorna atualização disponível, nenhuma atualização ou erro explícito de rede; nunca diz que falta chave.
- **AC-005** — Uma falha ao consultar atualizações permanece na página Sobre até nova tentativa; durante a consulta o botão fica desabilitado e a rede tem timeout de 15 segundos.
- **FR-003** — A gravação de segmentos e a limpeza de órfãos não podem apagar um arquivo ainda em publicação por outra fonte do mesmo gravador.
- **AC-006** — Duas fontes gravam segmentos concorrentes enquanto a retenção é aplicada: todo segmento dentro da retenção deve continuar legível e nenhuma gravação pode falhar por limpeza concorrente. Repetir reunião com áudio sintético no pacote após a correção.
- **AC-004** — Candidato 0.2.0 assinado com chave nova, com reinstalação de transição documentada. Publicação retida; instalação sobre 0.1.0 e atualização futura pelo canal público exigem execução real antes de liberar.

## Leitura e mapa de alterações

- `apps/desktop/e2e/` — jornadas do shell real, incluindo teste obsoleto do updater.
- `scripts/prepare-sidecars.ps1`, `.github/workflows/release.yml` — composição do instalador e voz local.
- `docs/qa/` — resultados e defeitos observados.

## Plano breve

Seam: interface do app em WebView2 e IPC Tauri, app-server externo real, instalador/updater. Separar inferência simulada de execução real do shell. Usar tauri-driver e msedgedriver existentes. Registrar indisponibilidade da inspeção nativa sem substituí-la por uma alegação de validação visual.

## Sequência e tarefas

- [ ] C-001 Escrever caso relevante e observar red pelo motivo esperado.
- [ ] C-002 Implementar o mínimo e observar green.
- [ ] C-003 Executar regressão e registrar evidência.

## Validação e evidência

Comandos: `cargo test --workspace --exclude aura-desktop`, `pnpm -C apps/desktop test`, `pnpm -C apps/desktop typecheck`, `pnpm -C apps/desktop tauri build --no-bundle`, `pnpm -C apps/desktop/e2e test` com specs selecionadas e AURA_HOME isolado.

Resultado executado: [pending]

Limitações: [o que não foi verificado]. Comando apenas identificado na configuração: [none ou registro].

## Condição de retorno

Retornar se o contrato, símbolo, dependência ou decisão material não puder ser satisfeito sem ampliar o escopo.

## Estado

`change.md` é a fonte canônica deste esforço compacto. Não crie `tasks.md` ou tickets paralelos para esta mudança.
