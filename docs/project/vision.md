# Project vision: Aura

## Problem and audience

Quem trabalha no Windows alterna constantemente entre o que está fazendo e uma aba de IA: copia texto, tira print, descreve a tela, cola de volta. Isso quebra o foco e ainda obriga a expor mais dados do que o necessário. Assistentes existentes são locais demais (qualidade limitada), pesados (Electron/Python), presos a um provedor, ou gravam tudo sem controle claro.

Público: profissionais do conhecimento, desenvolvedores, analistas, estudantes e criadores que usam Windows o dia inteiro e já pagam ChatGPT ou têm chaves de API.

## Desired outcomes

1. **Invocação instantânea**: atalho → janela flutuante translúcida pronta para digitar ou falar em menos de 100 ms, sobre qualquer app, inclusive tela cheia.
2. **Contexto sem esforço**: a IA vê a tela atual, a seleção, o áudio recente e os anexos que o usuário escolher, com chips visíveis e removíveis antes do envio.
3. **Agente de verdade**: a IA usa ferramentas (tela, áudio, arquivos, web, MCP, skills), mostra plano e progresso, e pede aprovação para ações com efeito.
4. **Use o que você já paga**: "Continue with ChatGPT" (Sign in with ChatGPT) consumindo o plano Plus/Pro do usuário; ou BYOK com qualquer endpoint.
5. **Voz de primeira classe**: push-to-talk com ASR local escolhido e baixado pelo usuário.
6. **Privacidade verificável**: modos de captura explícitos por fonte, buffer local criptografado, exclusão de janelas sensíveis, overlay invisível em capturas.
7. **Leveza**: residente com pegada mínima; processos pesados só sobem quando usados.

## Non-goals

- Ser uma IDE ou substituir o Codex/ChatGPT desktop para programação longa.
- Rodar LLMs locais embutidos na V1 (suportados via BYOK Ollama/LM Studio).
- Nuvem própria, contas próprias, sincronização entre dispositivos na V1.
- Controle autônomo de mouse/teclado na V1.
- macOS/Linux na V1.

## Constraints

- Windows 10 2004+ x64 / Windows 11; sem privilégios de administrador para instalar e usar.
- Orçamentos de desempenho (verificados no esforço 010): ocioso com overlay oculto e capturas desligadas ≤ 150 MB de working set privado somando host e WebView2, CPU ≤ 0,5%; overlay visível em ≤ 100 ms p95; primeiro caractere da resposta exibido em ≤ 50 ms após chegar do provedor.
- Nenhum dado capturado sai da máquina fora de um turno do usuário ou de uma ferramenta permitida pela política.
- Licenças de dependências permissivas (MIT/Apache/BSD/Unlicense).

## Hypotheses and evidence

| ID | Hypothesis | Impact | Check | State |
| --- | --- | --- | --- | --- |
| H-001 | Overlay pré-aquecido abre em ≤ 100 ms p95 | Fluidez percebida | Medição no esforço 001 | pending |
| H-002 | Termos da OpenAI permitem usar o plano ChatGPT via Codex app-server em app de terceiros | Lançamento comercial | Doc SIWC: permitido para open-source/local; pago/fechado via formulário de interesse (Q-001) | partially_confirmed |
| H-004 | Orçamento de memória ociosa é atingível com Tauri + app-server sob demanda | Promessa de leveza | Medição no esforço 010 | pending |
| H-010 | Usuários preferem "chips de contexto" explícitos a captura automática invisível | Confiança e adoção | Teste de usabilidade com 5 usuários no beta | pending |

(IDs completos em `docs/discovery.md`.)

## Continue/stop criterion

Continuar após o Marco 1 se a conversa via assinatura ChatGPT funcionar ponta a ponta com persona própria e os orçamentos H-001/H-004 forem atingidos ou tiverem caminho claro. Reavaliar o harness caso contrário.

## First marco

**Marco 1 — "Pergunte sobre a sua tela"**: com o app residente na bandeja, o usuário pressiona o atalho, a janela translúcida aparece sobre o app atual, ele faz login com o ChatGPT (uma vez), digita uma pergunta, anexa a tela atual com um clique/`/screen`, e recebe a resposta em streaming. Demonstração: vídeo curto no Windows 11 mostrando atalho → login → pergunta com print → resposta, e o relatório de medição de abertura e memória.

Tickets do marco 1: esforço 001 (TK-001…TK-005), esforço 002 (TK-001…TK-005), esforço 004 (TK-001).
