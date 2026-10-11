---
schema: hybrid/spec
schema_version: "1.0"
effort_id: 038-restaurar-seletores-modelo-raciocinio
revision: 1
status: accepted
profile: standard
---

# Restaurar a escolha de modelo e raciocínio no Overlay

## Problema e resultado desejado

Pedido de 06/10/2026: o usuário relata que a escolha do modelo de IA e do seu esforço de raciocínio desapareceu. Recuperar um controle visível, compreensível e utilizável antes da primeira mensagem e nas Conversas abertas, com catálogo, recuperação de falhas e envio das escolhas corretas ao agente.

Esta revisão aceita o comportamento solicitado, preservando os contratos dos esforços 002, 003, 013, 014 e 017. A preparação inicial foi somente documental; o usuário posteriormente autorizou executar todos os esforços e tickets, um por vez, deixando bloqueios para o final. Esta atualização editorial de autorização não altera requisitos ou aceites da revisão 1.

## Consumidores e atores

Usuário no Overlay; Provedor ChatGPT; Provedores BYOK; agente que recebe o modelo e esforço no próximo Turno.

## Escopo

Incluído: visibilidade em compacto/expandido, estados carregando/erro/vazio, recuperação do catálogo, modelos do provedor atual, esforços suportados, memória por modelo/modo, acessibilidade e regressão Windows.

Excluído: atualizar app-server, oferecer modelos indisponíveis na conta, redesenhar todo o Overlay, criar preferência global nova de modelo ou implementar web (esforço 039).

## Jornadas e cenários

### US-001 — Escolher antes de enviar (P1)

- **AC-001** — Dado Overlay compacto, onboarding concluído e provedor configurado, ao abrir sem Conversa existe botão visível com nome acessível para escolha de modelo; um clique abre opções de modelo e esforço, sem enviar mensagem ou expandir primeiro.
- **AC-002** — Dada Conversa aberta no Overlay expandido, abrir o controle permite escolher modelo e esforço para o próximo Turno; provedor permanece fixo conforme contrato atual. Minibar e onboarding não precisam conter o seletor.

### US-002 — Entender e recuperar falhas (P1)

- **AC-003** — Durante primeiro carregamento pendente, menu informa carregamento; após erro informa indisponibilidade e oferece tentar novamente; lista vazia obtida com sucesso tem mensagem distinta. Nenhum desses estados remove o botão ou apresenta catálogo fictício como confirmado.
- **AC-004** — Após catálogo válido, atualização falha preserva opções e escolha, indica falha e oferece repetição; repetição bem-sucedida atualiza menu já aberto. Respostas antigas não substituem resultado mais recente; falhas ChatGPT não apagam modelos BYOK.

### US-003 — Aplicar escolhas válidas (P1)

- **AC-005** — Modelo literal `qa-reasoner`, esforços `[low, high, max]`, padrão `high`: escolher `max` e enviar resulta em `model=qa-reasoner` e `effort=max` no agente; `medium` não aparece. Modelo sem suporte tem explicação de ausência de esforço configurável e não envia esforço incompatível.
- **AC-006** — Mesmo provedor/modelo com `low` em Chat e `max` em Tarefa: alternar modos, trocar modelo e voltar ou reiniciar restaura preferências ao selecionar esse modelo. Não cria obrigação de restaurar o último modelo na inicialização.
- **AC-007** — Esforço salvo perdeu suporte em atualização bem-sucedida ou modelo novo não o aceita: próximo Turno usa padrão compatível, explica mudança e nunca envia valor inválido. Falha transitória não significa remoção de suporte.

### US-004 — Usar no tamanho real da janela (P1)

- **AC-008** — Largura mínima 480 pixels lógicos, nomes longos e todos os controles do cabeçalho, compacto/expandido em DPI 100%, 150%, 200%: botão e seta visíveis e acionáveis; menu, rolagem e opções dentro da área útil do monitor. Nome acessível completo mesmo quando texto visual abreviado.
- **AC-009** — Teclado abre controle e seleciona opção; Escape fecha menu, devolve foco e mantém Overlay. Textos/estados pt-BR/en com paridade de chaves e nomes acessíveis.

## Requisitos

- **FR-001** — Acesso ao modelo/esforço DEVE permanecer visível em compacto/expandido, exceto Minibar e onboarding.
- **FR-002** — Catálogo DEVE distinguir carregamento inicial, sucesso, sucesso vazio e falha; erro não vira vazio silencioso.
- **FR-003** — Atualização DEVE preservar último catálogo válido na falha e permitir recuperação, sem respostas obsoletas.
- **FR-004** — Escolha DEVE respeitar capacidades do modelo/transporte e chegar ao próximo Turno.
- **FR-005** — Preferências existentes de esforço por provedor/modelo/modo DEVEM sobreviver às transições e ao reinício.
- **FR-006** — Valores invalidados por mudança confirmada DEVEM ser reconciliados e comunicados antes do envio.
- **FR-007** — Controle DEVE funcionar na largura mínima, DPI suportado, mouse/teclado e ambos os idiomas.

## Limites, erros e compatibilidade

Provedor de Conversa iniciada continua fixo. Mudança durante Turno vale para o próximo. Não hardcodar catálogo novo nem alterar capacidades para mascarar falhas. Não expor credenciais/conteúdo de conversas/capturas em diagnóstico.

## Hipóteses e dependências

Inspeção estática em `discovery.md`; relato não reproduzido em janela nativa. Hipóteses: catálogo silenciosamente vazio e compressão do cabeçalho. Dependências: IPC, catálogo 017, `Settings.effortPresets` 013 e app-server rust-v0.159.0. Atualizar sidecar não é necessário nem autorizado.

## Critérios de sucesso

- **SC-001** — Todos os AC têm teste comportamental ou inspeção nativa explícita; aprovação somente após execução real.
- **SC-002** — Regressões do plano e interação Windows passam, incluindo compacto antes da primeira mensagem.
- Observação posterior: relatos de desaparecimento e falhas de catálogo; nenhum resultado de uso alegado agora.

## Decisões e questões

Primeiro disponibilizar/recuperar controle, depois reconciliar escolhas e validar janela real. Causa exata no aplicativo instalado permanece investigação de execução. Mudança de contratos de catálogo/acesso exige voltar à spec. Nenhuma questão de produto impede estes documentos.

