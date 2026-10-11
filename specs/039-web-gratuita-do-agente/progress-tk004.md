# Checkpoint de implementação — TK-004

Data: 10/10/2026. Spec r1 / plan r7. Baseline fixo: `ed9a9654c6e76e48186fc010a12eba77023b00f5`. Ticket **blocked/partial**, com evidência atual **EV-031** e gate falho **EV-032**, achado FD-001; não aprovado para done. EV-020 a EV-027 ficaram stale após código/plano mudar. TK-001/TK-002/TK-003 renovados por EV-028/EV-029/EV-030, somente em seus escopos. TK-006 extrai a avaliação backend independente; TK-005 continua exigindo TK-004/TK-006 e todos os gates integrados.

## Entregue nesta fatia

- Preferência `webEnabled` com default true para configurações antigas, persistência de false após reiniciar, opção localizada em Settings e interação por teclado.
- Desativação cancela operações, bloqueia novas chamadas e acesso ao cache. Reativação no mesmo Turno preserva os orçamentos; a autoridade antiga não é reutilizada.
- Cache de buscas com TTL de 15 minutos, refresh explícito, isolamento por Conversa e consumo de orçamento mesmo em acertos. Documentos e buscas compartilham LRU de 32 entradas e contagem de memória do cache.
- Evento confiável `webSource`, contrato IPC Rust/TS e estado por Conversa/Turno; consultar novamente uma página já lida não rebaixa a fonte a snippet.
- Atividade, painel de fontes e citações em pt-BR/en, com abertura por teclado somente após ação do usuário. IDs desconhecidos, fontes de outra Conversa e URLs locais não produzem citação acionável. Títulos são texto e botões fabricados no HTML recebido são removidos antes de criar as citações conhecidas.
- Histórico: novo `web_history.rs` guarda somente metadados das fontes citadas, com snippets vazios, FK para exclusão em cascata e nenhuma linha para Conversa efêmera. Host restaura o DTO `sources` opcional e mantém compatibilidade role/text. Dois Hosts reais sobre o mesmo banco em disco comprovaram restauração de W1, sem W2 não citado e sem rede. A UI também restaura painel/citação e abre por teclado.
- Conversa efêmera com pesquisa real no fixado: depois de shutdown, nenhuma URL de fonte, conteúdo não citado ou rollout nos arquivos locais.
- Identidade após reinício: W1 citada mantém sua URL/ID mesmo em outra ordem de busca; W2 não citada tem somente seu número reservado em web_source_sequence, e uma URL nova recebe W3. Nenhuma URL/título/trecho não citado é gravado por essa sequência. Citação após desligar web continua restaurável, sem permitir ferramentas/cache; UI reconhece fonte histórica em Turno posterior.
- Registry, strings de sessão/Turno e a entrada recente compartilham limites de 8/32 MiB. Rejeitar antes de atribuir ID/inserir fonte/cache, inclusive crescimento do contexto sem rede. Fechar remove todo estado; conclusão de slot de uma sessão antiga não libera slots de uma nova. Testes públicos atravessam esses casos.
- Metadata em SQLite real: duas Conversas com mesmos turn/item/W1 ficam isoladas após reabrir banco; exclusão remove citações e sequência somente da Conversa removida e impede writes tardios. Restauração libera lock e para após primeiro lote rejeitado. Três casos passaram de início; são cobertura, sem red artificial.

## Execuções e evidências

Red comportamental foi observado antes das correções de default, cancelamento, TTL, eventos de fontes, painel, botão HTML falso e atividade. Casos que já passaram foram registrados como cobertura, sem red artificial.

- Serviço: `cargo test -p aura-web --test search --test fetch` — 21 buscas + 29 leituras offline; dois testes live ignorados. EV-014/EV-018 renovam os predecessores no escopo original.
- Aplicação: `cargo test -p aura-app --test web_tools --test host --test ipc_contract` — 14 + 74 + 2 passaram. Codex/gateway/MCP — 90 testes passaram. EV-015.
- App-server fixado real: filtro `web::` — 12/12 passaram, incluindo nove combinações de protocolo/modo, comparação, recusa de ação induzida e retomada usando cache. EV-016. Sandbox legado executado separadamente — 1/1 passou. EV-017. Provedores de inferência e páginas são externos scripted; não é prova de qualidade de busca ao vivo.
- Renovação plan r5: EV-020/EV-021 — 50 offline; EV-022 — 90 app + 90 Codex/gateway/MCP, 12 integrações web reais em 8,95 s e sandbox separado 1/1 em 23,79 s. Escopos anteriores, sem aprovação do TK-004.
- UI atual: `pnpm -C apps/desktop test` — 253 testes em 33 arquivos passaram. `typecheck`, Clippy do workspace sem o shell Tauri, rustfmt e `git diff --check` passaram. Histórico real 1/1 e privacidade efêmera 1/1. EV-024 parcial.
- Gate normal: `cargo test -p aura-app --test real_app_server web::web_normal_research -- --ignored` — falhou 0/1, por trecho web não citado persistido no rollout. EV-023 failed; ausência de erro de ambiente. O esperado foi mantido.
- Atualização r7: 54 serviço (25 busca/29 leitura), 144 aplicação (54 unidade/74 Host/14 web/2 IPC), 90 Codex/gateway/MCP passaram. Fixado real16/16 em12,37s; sandbox1/1 em23,71s. Clippy workspace sem shell passou. EV-031 parcial cobre identidade/metadata/limites/efemeridade, sem aprovação final Windows/rollout. UI253/types passaram nesta rodada antes da decomposição r7, sem alterações UI posteriores; renovação final continua pendente.
- Gate atual EV-032: teste normal reexecutado no r7, 0/1 em0,69s por conteúdo não citado no rollout; é falha de comportamento. FD-001 registrado; nenhum esperado enfraquecido.

## Pendências reais

1. Impedir retenção dos resultados MCP não citados nos rollouts do Codex fixado. Auditoria, EV-032 e FD-001 comprovam a retenção normal. O [código oficial da política de rollout do fixado](https://github.com/openai/codex/blob/rust-v0.159.0/codex-rs/rollout/src/policy.rs) também persiste FunctionCallOutput; o teste local é a prova do executável. Não alegar solução por flag de histórico estendido ou limpeza depois da escrita. Resolver a arquitetura respeitando a política de Conversa e sem atualizar sidecar automaticamente. Este é o bloqueio atual do TK-004, deixado para o final conforme instrução do usuário.
2. TK-006: executar qualidade de busca/extração na internet real, com lista/metas originais. Avaliação independente não dispensa retenção normal nem os demais gates.
3. Depois de TK-004/TK-006: concluir TK-005, cadeia determinística, três jornadas de síntese e build/E2E Windows da versão atual, com revisão/verificação integral.
4. Voltar às pendências do esforço 038: DPI 150%/200%, redimensionamento/foreground e renovação das evidências de Host/UI compartilhados.

Nenhum commit, publicação ou atualização de sidecar foi feito. Nenhum processo de QA ficou pendente neste checkpoint. As evidências anteriores do esforço 038 precisam ser reavaliadas após as alterações compartilhadas de Host/UI.
# Continuação r8 — 10/10/2026

Novo caminho investigado em `persistence-research.md`: protótipo fixado real demonstrou entrega transitória no gateway e manutenção de threadId/histórico após reinício. Resultado parcial EV-034; não mudou a aplicação. TK-007 implementará escopo, origem, orçamento e ciclo de vida; TK-004 passa a depender dele. ADR 0010 proposto. O gate normal foi reexecutado após os documentos r8 e continua falhando 0/1 em0,74s por conteúdo não citado no rollout; nenhuma dispensa de expected.

Regressões r8: WebService54/app144/Codex+gateway+MCP90 e fixado real16/16 passaram; sandbox real1/1. TK-006 tem dados históricos r7 e evidência invalidada; renovar gate ao final, sem apresentá-los como uma nova execução. Próxima fronteira é TK-007, depois retornar à retenção integral e às provas de TK-005/038.

## Retomada após TK007 r9

TK007 done com EV068:504workspace,25integrações fixadas,16HTTP,diagnóstico isolado,AEAD independente e revisão Standards/Spec. Reinício/compactação e privacidade de raciocínio exercitados nos três formatos; gate W2 preservado. FD001 resolved; ADR0010 accepted. Os bloqueios de retenção descritos acima são históricos. Retomar TK004 com package atual e renovar UI/typecheck/contrato/fontes/controle antes de concluir seus aceites. Live TK006,integração/síntese/Windows TK005 eDPI038 continuam pendentes.

Renovação da fatia:packageTK004 ready:true no checkpoint63;253/253UI em37,05s,33arquivos e typecheck atuais passaram. Backend/contrato/histórico/controle atuais cobertos por504workspace e25fixados;mesmos oráculos e sem alteração produtiva após essas provas. Revisão Standards/Spec da fatia emreview.md,sem achado bloqueante. A prova UI verifica renderização/teclado/IPC em pt-BR/en,sem afirmar janela Windows/DPI. TK004 pode concluir;seguir renovação liveTK006 e TK005 conforme dependências.

