# Roteiro de validação no Windows — funções novas (esforços 019–026)

Tudo abaixo foi implementado e testado no Linux (lógica, host e interface com backend simulado). **Nada disto foi executado no Windows.** Registre cada resultado com `python .hybrid/hybrid.py evidence add --effort <id> --acceptance-refs AC-xxx --executed ...`. Use `AURA_HOME` isolado.

## 021 — Ocultar em transmissões e Modo transmissão

| # | Passo | Esperado |
| --- | --- | --- |
| 1 | Configurações › Geral › "Testar ocultação" com o Overlay e as Configurações abertos | "Tudo oculto"; cada janela "oculta" |
| 2 | Desligar "Ocultar em transmissões" e testar de novo | "Como configurado: o Aura aparece em capturas"; janelas "visível" |
| 3 | Com a opção ligada, compartilhar a tela e abrir o Overlay em: **Google Meet (Chrome)**, **Teams**, **Zoom**, **Discord (tela e janela)**, **OBS (captura de tela e de janela)**, **AnyDesk**, Gravador de Tela do Windows, Print Screen, **RDP** | O Aura não aparece para quem assiste nem na captura. Registrar por app; AnyDesk e RDP **ainda não foram validados** |
| 4 | Câmera apontada para o monitor, placa de captura HDMI | O Aura **aparece** (limite conhecido, avisado na opção) |
| 5 | Ligar o Modo transmissão e criar um lembrete para 1 min | Nenhum toast do Windows; o aviso aparece dentro do Overlay |

## 019 — Dia a Dia (texto)

| # | Passo | Esperado |
| --- | --- | --- |
| 1 | Selecionar um texto no Word/Bloco de Notas, abrir o Aura, `/curto` | Resposta mais curta; botão **Substituir seleção** com Antes/Depois no hover |
| 2 | Clicar em Substituir seleção; depois **Desfazer substituição** | Texto trocado no app; Ctrl+Z devolve o original |
| 3 | `/golpe` com uma mensagem suspeita de WhatsApp | Veredito, sinais, o que fazer, como confirmar; sem pedir dados |
| 4 | `/responder` com um e-mail na tela | Anexa a tela e devolve rascunho; **Inserir no app** cola |
| 5 | Copiar uma lista, `/colar tabela` | Converte a área de transferência; com a área vazia, explica |
| 6 | Selecionar um texto, `/ler` | Lê em voz alta (voz offline) |
| 7 | `/parei` com o buffer de tela ligado | Resume o que você fazia nos últimos minutos |

## 020 — Texto da tela, lembretes, notas, salvos

| # | Passo | Esperado |
| --- | --- | --- |
| 1 | `/texto`, arrastar sobre texto na tela | Toast "Texto copiado"; colar em outro app confere (OCR pt-BR instalado) |
| 2 | "me lembra daqui a 2 minutos de ligar para o João" (aprovar a ferramenta) | Toast do Windows em ~2 min, uma vez; sobrevive a fechar e abrir o Aura antes |
| 3 | Lembrete "toda segunda às 9h" | Dispara na segunda; não repete se o Aura ficou fechado |
| 4 | `/anota renovar o seguro em março` e `/notas seguro marco` | Nota guardada e achada sem acento |
| 5 | Estrela **Salvar** numa resposta e `/salvos` | O texto aparece |

## 022 — Preparo e Configurar com IA

| # | Passo | Esperado |
| --- | --- | --- |
| 1 | Busca das Configurações (Ctrl+K) por algo inexistente → "Pedir à IA" | Abre conversa em modo Tarefa com `$aura-configurar` |
| 2 | Pedir "deixa o Aura mais discreto" | O agente chama `settings_propose`, mostra antes→depois, só aplica com aprovação; "Desfazer" restaura |
| 3 | Pedir "não veja o KeePass" com o KeePass aberto | Usa `open_windows`, propõe exclusão por processo, `exclusion_add` com aprovação; a janela passa a ser coberta |
| 4 | Primeiros passos, último passo "Conte como você trabalha" | Frase ou grill-me chegam ao agente |
| 5 | Tentar pedir para ligar o YOLO ou mudar atalhos | O agente recusa e diz onde o usuário faz isso |

## 023–026 — Reunião

| # | Passo | Esperado |
| --- | --- | --- |
| 1 | Abrir o Aura sem iniciar reunião, deixar 10 min | Nenhum indicador de gravação de áudio (a menos que a política do usuário já ligasse) |
| 2 | Botão Reunião › Começar sem preparo, falar e tocar um vídeo | Em ~20 s aparecem falas "Você" e "Eles" (**exige modelo de voz local ou nuvem**); pausa não transcreve o intervalo |
| 3 | ⚡ uma frase → cartão "Entendi assim" → o agente chama `meeting_brief_save` | O briefing aparece no painel e **não** inicia a reunião; só Começar inicia |
| 4 | Notas e ★ durante a reunião | Entram na linha do tempo; `Perdi o fio` cita minutos `[mm:ss]` |
| 5 | Encerrar | Áudio apagado (padrão); com "Guardar o áudio" ligado, fica |
| 6 | "Esqueci de iniciar" com buffer de áudio de 15 min ligado | Cria a reunião com a transcrição |
| 7 | Ligar "Ocultar dados pessoais" e pedir resumo de uma reunião com CPF falado | O modelo vê `[CPF]` |
| 8 | Receita: escolher "1:1", encerrar, "Resumo e ações"; "Gerar ata (agente)" | Resumo no formato da Receita, com minutos; `ata.md` criada no workspace após aprovação |
| 9 | "Criar uma Receita com IA" | `recipe_save` com aprovação; a Receita aparece no seletor |

## Áudio do sistema de todas as saídas

| # | Passo | Esperado |
| --- | --- | --- |
| 1 | Conectar fone e alto-falantes; em Configurações › Voz, escolher "Todas as saídas" no áudio do sistema e usar o teste com medidor | O medidor reage ao som tocando em **qualquer** saída |
| 2 | Iniciar uma Reunião sem escolher dispositivo e tocar áudio só no fone (não padrão) | A fala aparece como "Eles" (a Reunião ouve todas as saídas) |
| 3 | Plugar um fone novo **durante** a gravação | Limite conhecido: só entra após reiniciar a captura |

## Itens que dependem do ASR local

Compilar o worker com `--features engines` (HANDOFF §7) e repetir 023-2 com o modelo local; sem isso, usar a transcrição na nuvem (BYOK) e anotar a diferença de latência.
