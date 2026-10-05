---
schema: hybrid/change
schema_version: 1.0
effort_id: 021-ocultacao-e-transmissao
revision: 1
status: draft
profile: compact
---

# Change: Ocultação em transmissões e Modo Transmissão

## Objetivo e limites

Candidatos: CAND-057. Origem: plano `docs/project/plano-reuniao-e-configuracao-assistida.md`. Aura invisível para quem assiste à tela compartilhada, com verificação e silêncio de notificações durante a transmissão.

Nota: **Feito (05/10):** opção `hideFromCapture`, aplicação em todas as janelas, botão "Testar ocultação" (lê a afinidade de cada janela com `GetWindowDisplayAffinity`). **Falta:** teste por WGC/DXGI/GDI com miniaturas, Modo Transmissão (silenciar notificações), roteiro nos apps reais. Fora do escopo por decisão: ocultar o processo e enganar softwares de prova.

## Contrato de comportamento

- Entradas: ver requisitos e critérios abaixo.
- Saída: ver critérios de aceite.
- Erros/invariantes: segredos nunca expostos; captura só pela Política de privacidade.
- Compatibilidade: comportamento existente do Overlay, Comandos rápidos e Configurações.

## Requisitos e aceite

- **FR-001** — Todas as janelas do Aura MUST poder ficar fora de compartilhamento, gravação e captura por uma opção (padrão ligado).
- **AC-001** — Dado `hideFromCapture = true`, quando qualquer janela do Aura está aberta, então `GetWindowDisplayAffinity` = `WDA_EXCLUDEFROMCAPTURE` (0x11); com `false`, = 0. **Opção implementada e testada em Rust/UI (05/10); validação com apps reais pendente no Windows.**
- **FR-002** — O Aura MUST oferecer "Testar ocultação" e Modo Transmissão.
- **AC-002** — Dado "Testar ocultação", quando o usuário o executa, então o Aura captura a tela por WGC, DXGI e GDI e mostra a miniatura de cada uma sem o Aura. Em Modo Transmissão, notificações nativas do Aura ficam em silêncio. **Pendente.**
- **FR-003** — A ocultação MUST ser validada em Meet, Teams, Zoom, Discord, OBS, AnyDesk e RDP.
- **AC-003** — Dado cada app, quando compartilha a tela com o Overlay aberto, então o Overlay não aparece para quem assiste; resultado registrado como evidência. **Pendente.**

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
