---
schema: hybrid/change
schema_version: 1.0
effort_id: 023-motor-de-reuniao
revision: 1
status: draft
profile: compact
---

# Change: Motor de Reunião

## Objetivo e limites

Candidatos: CAND-035, CAND-038. Origem: plano `docs/project/plano-reuniao-e-configuracao-assistida.md`. Reunião como sessão opt-in com transcrição contínua, biblioteca local e "esqueci de iniciar"; consentimento e privacidade desde o início.

Nota: Depende dos motores de voz locais validados no Windows; nuvem como plano B. **Feito (05/10):** migração 0011, `MeetingService`/`MeetingRepo` (início, pausa, fim, falas únicas com tempo e falante, "esqueci de iniciar", recuperação após queda), gravação de áudio só durante a Reunião, comandos IPC e evento `meeting`, ferramentas `meeting_search`/`meeting_get`. **Falta:** transcrição contínua real (Windows), apagar áudio ao encerrar, retenção por reunião, aviso de consentimento (CAND-038), UI (esforço 024).

## Contrato de comportamento

- Entradas: ver requisitos e critérios abaixo.
- Saída: ver critérios de aceite.
- Erros/invariantes: segredos nunca expostos; captura só pela Política de privacidade.
- Compatibilidade: comportamento existente do Overlay, Comandos rápidos e Configurações.

## Requisitos e aceite

- **FR-001** — Uma Reunião MUST começar e terminar só por ação do usuário.
- **AC-001** — Dado o Aura aberto por um dia sem Reunião iniciada, então nenhuma Fonte de áudio é capturada além do que a Política já permitia.
- **FR-002** — O Aura MUST transcrever continuamente microfone ("Você") e áudio do sistema ("Eles").
- **AC-002** — Dada uma reunião com falas nas duas fontes, então a transcrição rotula cada fala e o atraso p95 respeita o orçamento medido (H-019).
- **FR-003** — O Aura MUST criar uma Reunião a partir do Buffer recente e apagar o áudio ao encerrar por padrão.
- **AC-003** — Dado 15 min de buffer de áudio, quando o usuário escolhe "Salvar como reunião", então a Reunião existe com transcrição completa; ao encerrar, o áudio é apagado conforme a retenção.

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
