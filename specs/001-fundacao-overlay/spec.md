---
schema: hybrid/spec
schema_version: "1.0"
effort_id: 001-fundacao-overlay
revision: 2
status: accepted
profile: standard
---

# Specification: Fundação do app residente e Overlay

## Problem and desired result

O usuário precisa de um assistente que esteja sempre disponível sem ocupar a tela nem pesar no computador. Hoje ele troca de janela para falar com uma IA. O resultado desta entrega é um app residente na bandeja que mostra, por um Atalho de invocação, um Overlay translúcido pronto para digitar sobre qualquer aplicativo, e o esconde devolvendo o foco — rápido, bonito, persistente e invisível em capturas de tela.

## Consumers and actors

- Usuário do Windows que invoca e dispensa o Overlay.
- Os esforços seguintes (002–010), que consomem a janela, as configurações, o store e o registro do Aplicativo anterior.

## Scope

### Included

- Processo residente com ícone na bandeja, instância única, saída limpa.
- Atalho de invocação configurável e gesto opcional de duplo toque em Ctrl.
- Overlay translúcido, sempre no topo, fora da taskbar/Alt+Tab e excluído de capturas.
- Estados compacto/expandido, arrastar, redimensionar, posição por monitor, comportamento ao perder foco.
- Janela de Configurações com tema, opacidade, atalho, iniciar com Windows e comportamento de foco.
- Store local com campos sensíveis cifrados e logs com redação de segredos.
- Medição de latência de abertura e memória ociosa (verificação final no esforço 010).

### Excluded

- Conversa com o agente (002), captura de tela (004), áudio (005), voz (006).
- Instalador, atualização automática e onboarding (010).
- Minibar e notificações (009).

## User journeys and scenarios

### US-001 — Aura residente e discreto (Priority: P1)

Como usuário, quero que o Aura rode em segundo plano sem janela aberta, para que esteja disponível sem me atrapalhar.

Independent demonstration: iniciar o app, ver apenas o ícone da bandeja, tentar abrir de novo, sair pelo menu.

#### Acceptance scenarios

- **AC-001** — Dado que o Aura não está em execução, quando o usuário o inicia, então aparece um ícone na bandeja com o menu Abrir, Nova conversa, Configurações e Sair, e nenhuma janela fica visível.
- **AC-002** — Dado que o Aura está em execução, quando o usuário o inicia novamente, então nenhuma segunda instância permanece e o Overlay da instância existente é mostrado.
- **AC-003** — Dado que o Aura está em execução com processos filhos, quando o usuário escolhe Sair na bandeja, então o ícone desaparece e nenhum processo do Aura (incluindo filhos) continua vivo após 3 segundos.

### US-002 — Invocar e dispensar o Overlay (Priority: P1)

Como usuário, quero chamar o Overlay com um atalho sobre qualquer aplicativo e dispensá-lo voltando exatamente onde estava.

Independent demonstration: com o Bloco de Notas em foco, pressionar o atalho, digitar, pressionar Esc e continuar digitando no Bloco de Notas.

#### Acceptance scenarios

- **AC-004** — Dado o Aura ocioso há pelo menos 1 minuto e outro aplicativo em primeiro plano, quando o usuário pressiona o Atalho de invocação (padrão `Ctrl+Shift+Space`), então o Overlay fica visível no monitor do Aplicativo anterior com o cursor na barra de entrada, em até 100 ms no p95 de 50 invocações medidas.
- **AC-005** — Dado o Overlay visível, quando o usuário pressiona o Atalho de invocação ou o botão "Minimizar para a bandeja", então o Overlay some e o Aplicativo anterior volta a ter o foco do teclado; `Esc` só fecha menus, histórico ou gravação de voz e nunca esconde o Overlay (revisão 2).
- **AC-006** — Dado o gesto de duplo toque ativado, quando o usuário pressiona e solta `Ctrl` duas vezes em até 400 ms sem outra tecla entre elas, então o Overlay alterna; dado um único `Ctrl`, `Ctrl+C`, ou dois toques separados por mais de 400 ms, então nada acontece; após uma ativação, toques nos 600 ms seguintes são ignorados.
- **AC-007** — Dado que o usuário escolhe um atalho já registrado por outro aplicativo, quando tenta salvar, então a configuração mostra "Atalho em uso por outro aplicativo" e o atalho anterior continua funcionando.

### US-003 — Aparência premium e privada (Priority: P1)

Como usuário, quero um Overlay translúcido e moderno que não apareça quando compartilho ou gravo a tela.

Independent demonstration: abrir o Overlay sobre um wallpaper colorido, ajustar opacidade e compartilhar a tela no Teams.

#### Acceptance scenarios

- **AC-008** — Dado o Windows 11, quando o Overlay aparece, então usa fundo Acrylic translúcido com cantos arredondados e sombra, fica acima de janelas normais e de apps em tela cheia sem bordas, e não aparece na taskbar nem no Alt+Tab; dado o Windows 10, então usa fundo desfocado ou sólido translúcido com os mesmos comportamentos.
- **AC-009** — Dado o Overlay visível, quando qualquer ferramenta captura a tela (Windows.Graphics.Capture, Ferramenta de Captura, compartilhamento do Teams/Zoom, OBS), então o Overlay não aparece no resultado capturado.
- **AC-010** — Dado o controle de aparência do próprio Overlay ou a janela de Configurações, quando o usuário ajusta a opacidade entre 50% e 100% ou escolhe tema Sistema/Claro/Escuro, então o Overlay inteiro (fundo, cartões e menus) aplica a mudança imediatamente e após reiniciar o app.

### US-004 — Overlay que se adapta ao meu jeito (Priority: P1)

Como usuário, quero expandir, mover e redimensionar o Overlay e reencontrá-lo onde deixei.

Independent demonstration: expandir, arrastar para outro canto, fechar, reabrir, desconectar o monitor secundário e reabrir.

#### Acceptance scenarios

- **AC-011** — Dado o Overlay compacto, quando o usuário pressiona `Ctrl+↓` (ou envia uma mensagem), então ele expande com animação de no máximo 200 ms; `Ctrl+↑` volta ao compacto; com "reduzir movimento" do Windows ativo, a transição é instantânea.
- **AC-012** — Dado que o usuário moveu/redimensionou o Overlay em um monitor, quando o reabre nesse monitor, então ele aparece na mesma posição e tamanho; dado que o monitor salvo não existe mais, então aparece na posição padrão (centro do terço superior) do monitor do Aplicativo anterior, inteiramente visível.
- **AC-013** — Dado o Overlay visível, quando o foco passa para outra janela, então ele permanece visível (padrão); dado "Esconder ao clicar fora" ativado, então ele se esconde sem resposta em andamento e vira Minibar com resposta em andamento (revisão 2).

### US-005 — Configurações que persistem (Priority: P1)

Como usuário, quero ajustar o Aura em uma janela própria e que as escolhas sobrevivam a reinícios.

#### Acceptance scenarios

- **AC-014** — Dado o Aura em execução, quando o usuário abre Configurações pela bandeja ou `Ctrl+,` no Overlay, altera qualquer opção e reinicia o app, então as opções alteradas continuam em vigor.
- **AC-015** — Dado "Iniciar com o Windows" ativado, quando o usuário faz logon, então o Aura inicia na bandeja sem mostrar janelas; desativado, então não inicia.

### US-006 — Dados locais protegidos (Priority: P1)

Como usuário, quero que dados sensíveis guardados pelo Aura fiquem ilegíveis fora da minha conta do Windows e nunca apareçam em logs.

#### Acceptance scenarios

- **AC-016** — Dado um valor marcado como sensível salvo pelo store, quando os bytes do arquivo do banco são inspecionados, então o valor em texto claro não aparece; quando o mesmo usuário do Windows reabre o app, então o valor é lido corretamente; quando outro usuário do Windows tenta decifrar a chave, então a operação falha.
- **AC-017** — Dado um valor sensível presente num erro ou evento registrado, quando o log é escrito, então aparece `[REDACTED]` no lugar do valor; e os arquivos de log rotacionam em 10 MB mantendo no máximo 5 arquivos.

## Requirements

- **FR-001** — O sistema MUST rodar como processo residente de instância única com ícone e menu na bandeja e encerrar todos os seus processos ao sair.
- **FR-002** — O sistema MUST mostrar/esconder o Overlay por um Atalho de invocação configurável e, opcionalmente, pelo gesto de duplo toque em Ctrl, devolvendo o foco ao Aplicativo anterior ao esconder.
- **FR-003** — O Overlay MUST ser translúcido, sempre no topo, fora da taskbar e do Alt+Tab e excluído de capturas de tela.
- **FR-004** — O Overlay MUST ter estados compacto e expandido, ser movível pelo cabeçalho e redimensionável pelas bordas (no compacto só a largura; a altura segue o conteúdo e cresce para caber menus), lembrar posição/tamanho por monitor e seguir a regra configurada ao perder foco.
- **FR-005** — O sistema MUST persistir configurações (tema, opacidade, atalho, duplo toque, iniciar com Windows, esconder ao clicar fora) e aplicá-las sem reinício.
- **FR-006** — O sistema MUST cifrar valores sensíveis em repouso com chave atrelada ao usuário do Windows e MUST redigir valores sensíveis nos logs.
- **FR-007** — O sistema MUST abrir o Overlay em até 100 ms p95 a quente e SHOULD manter host + WebView2 ociosos em até 150 MB de working set privado.

## Limits, errors, and compatibility

- Mínimo: Windows 10 versão 2004 (build 19041), exigido por `WDA_EXCLUDEFROMCAPTURE`. Em versão inferior o app informa incompatibilidade e sai.
- Jogos em tela cheia exclusiva (não "sem bordas") podem impedir que qualquer janela fique por cima; isso é limitação do Windows e o Overlay deve aparecer quando o jogo perder o modo exclusivo.
- Se o registro do atalho falhar na inicialização, o app continua na bandeja e mostra notificação com link para Configurações.
- O WebView2 Runtime ausente é tratado pelo instalador (010); o app sai com mensagem clara se não encontrá-lo.
- Janelas elevadas (administrador) podem não permitir devolver o foco; nesse caso o Overlay apenas se esconde.

## Hypotheses and dependencies

- Hipótese H-001: com a janela do Overlay pré-criada e oculta, a abertura fica ≤ 100 ms p95. Impacto: fluidez. Check: medição do TK-003.
- Hipótese H-009: `WDA_EXCLUDEFROMCAPTURE` oculta o Overlay de todas as ferramentas listadas em AC-009. Check: roteiro manual do TK-002.
- Hipótese H-004 (parcial): host + WebView2 oculto cabem em 150 MB. Check: medição do TK-003; verificação final no 010.
- Dependência: WebView2 Runtime (presente no Windows 11 e na maioria dos Windows 10 atualizados). Estado: observado na documentação da Microsoft; confirmado pelo instalador no 010.

## Success criteria

### Delivery-verifiable

- **SC-001** — AC-001 a AC-017 com testes automatizados ou roteiro manual executado e evidência registrada em Windows 11 e Windows 10 22H2.
- **SC-002** — Relatório do `aura-bench` com p50/p95 de abertura e memória ociosa anexado como evidência.

### Post-delivery observation

- **SC-003** — Em uso real por 1 semana, o usuário não relata o Overlay "demorando" ou "roubando foco" (entrevista do beta).

## Decisions and open questions

- Atalho padrão `Ctrl+Shift+Space` (D-013); duplo toque em Ctrl desativado por padrão para evitar ativações acidentais.
- Ao perder foco, esconder é o padrão; "manter aberto" é opção.
- Nenhuma pergunta de produto pendente.
