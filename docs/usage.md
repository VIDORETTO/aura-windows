# Guia de uso do Aura

[Voltar ao README](../README.md)

## 🧑‍💻 Uso

### Atalhos globais (padrões, alteráveis em Configurações › Atalhos)

| Atalho | Ação |
|---|---|
| `Ctrl+Shift+Space` | Abrir/fechar o Aura |
| `Ctrl+Space` (segurar, com o Aura aberto) | Falar em vez de digitar |
| `Ctrl+Alt+Space` | Ditar em qualquer app (pressione para começar, de novo para inserir o texto) |
| `Ctrl+Shift+Alt+P` | Pausar/retomar toda captura |

### No Overlay

| Tecla | Ação |
|---|---|
| `Enter` / `Shift+Enter` | Enviar / nova linha |
| `Ctrl+Enter` durante a resposta | Direcionar a resposta atual |
| `Ctrl+.` | Interromper |
| `Ctrl+Shift+S` | Anexar a tela |
| `Ctrl+Shift+L` | Ouvir a última resposta (de novo para parar) |
| `Ctrl+Shift+Enter` | Inserir a última resposta (ou o trecho selecionado nela) no app anterior |
| `Ctrl+V` com imagem | Anexar a imagem da área de transferência |
| `Ctrl+N` / `Ctrl+Shift+E` | Nova conversa / conversa efêmera (não fica no histórico) |
| `Ctrl+H` / `Ctrl+,` | Histórico / Configurações |
| `Ctrl+↑` / `Ctrl+↓` | Compactar / expandir |
| `Esc` | Fechar menu → cancelar ditado → fechar histórico (nunca esconde o Overlay; use o atalho ou **Minimizar para a bandeja**) |

### Adicionar contexto com `@`

Digite `@` para anexar **tela**, **região** (seleção sobre a tela congelada), **janela ativa**, **seleção**, **arquivo** ou **últimos minutos** (buffer recente de tela e/ou áudio: microfone, sistema ou ambos). Arquivos também podem ser arrastados para o Overlay e imagens coladas com `Ctrl+V`.

A seleção vem do app anterior via UI Automation: ao abrir o Overlay pelo atalho, ou ao voltar a ele depois de selecionar texto em outro app, ela aparece como um único Chip (uma nova seleção substitui a anterior). Apps cujos campos não expõem o texto pela acessibilidade do Windows não fornecem a seleção.

### Comandos rápidos com `/`

| Comando | O que faz |
|---|---|
| `/tldr` | Resume em até três frases |
| `/formal` · `/curto` · `/amigavel` | Reescreve a seleção no tom pedido |
| `/golpe` | Analisa se uma mensagem, e-mail, link ou boleto parece golpe |
| `/responder [tom]` | Anexa a tela e escreve um rascunho de resposta |
| `/lembrar <o quê e quando>` | Cria um lembrete com notificação do Windows (únicos ou recorrentes) |
| ⭐ Salvar (em cada resposta) · `/salvos <busca>` | Guarda respostas e textos prontos e os procura depois |
| `/ensaio <cenário>` | Treina uma conversa difícil (entrevista, vendas, negociação) com feedback |
| `/estudo` | Notas, glossário, flashcards e quiz do conteúdo que você estuda |
| `/carreira` | Currículo sob medida, carta e histórias de entrevista, só com os seus fatos |
| `/documento` | Explica contrato, boleto, bula ou termos em linguagem simples, com riscos e prazos |
| `/ajuda` | "Me ajude agora": responde sobre a tela e o áudio recente |
| `/agendar <o quê e quando>` | Agenda uma instrução recorrente ("todo dia útil às 8h, resuma meus compromissos"); roda numa conversa de Chat, só leitura |
| `/anota <texto>` · `/notas <busca>` | Guarda e procura anotações rápidas |
| `/texto` | Arraste sobre uma área da tela e o texto (OCR) vai para a área de transferência |
| `/colar [formato]` | Converte o texto da área de transferência (lista, tabela, texto limpo, formal…) para você colar |
| `/ler` | Lê em voz alta o texto selecionado em outro app |
| `/configurar <pedido>` | A IA configura o Aura por você, mostrando antes → depois |
| Botão Reunião (microfone) | Painel para preparar (⚡ uma frase, 🧭 grill-me ou sem preparo), iniciar, pausar e encerrar uma reunião, ou salvar os últimos minutos do buffer ("esqueci de iniciar") |
| `/preparo <assunto>` | A IA entrevista você (⚡ uma frase ou 🧭 grill-me) antes de uma tarefa |
| `/parei` | Resume o que você fazia nos últimos minutos (precisa do buffer de tela) |
| `/traduzir <idioma>` | Traduz (padrão: inglês) |
| `/reescrever` | Reescreve com mais clareza |
| `/explicar` | Explica de forma simples |
| `/corrigir` | Corrige ortografia e gramática |
| `/resumir-tela` | Anexa a tela e resume o que está nela |
| `/plano` | Entra no modo Plano |
| `/tela` | Anexa a tela |
| `/compactar` | Resume a conversa atual para liberar contexto |

Os comandos usam a seleção, o texto digitado ou os anexos. Você pode criar os seus em Configurações › Extensões, e suas Skills também aparecem no menu `/`.

### Modos de conversa

| Modo | O agente pode |
|---|---|
| **Chat** | Responder e ler; nunca altera nada |
| **Tarefa** | Criar e alterar arquivos no workspace da conversa e em pastas que você liberar, com aprovação |
| **Plano** | Pesquisar e planejar; não executa |

Dá para trocar de modo no meio da conversa pelo mesmo seletor: aparece "Modo alterado para …" e, a partir da mensagem seguinte, o agente segue o modo novo (as permissões do turno mudam e ele é avisado de que o modo anterior não vale mais).

O esforço de raciocínio fica no mesmo seletor (modelo e modo). O valor escolhido é lembrado para aquele modelo e modo e volta sozinho na próxima vez; em **Configurações › Conta e modelos › Esforço por modelo e modo** você vê e muda todos de uma vez. Modelos de provedores personalizados mostram os esforços que você cadastrou para eles.

