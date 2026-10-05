---
schema: hybrid/change
schema_version: 1.0
effort_id: 034-lembretes-e-notas-na-ui
revision: 1
status: draft
profile: compact
---

# Change: Página de lembretes e notas

## Objetivo e limites

Lacuna do esforço 020. Lembretes, notas e textos salvos só podiam ser vistos pelo agente. A página Configurações › Lembretes e notas lista, busca e apaga cada um, tudo local.

## Contrato de comportamento

- Entradas: [tipo e pré-condições]
- Saída: [resultado observável]
- Erros/invariantes: [casos relevantes]
- Compatibilidade: [o que deve permanecer igual]

## Requisitos e aceite

- **FR-001** — O usuário MUST poder listar e apagar lembretes, notas e textos salvos, e buscar nas notas.
- **AC-001** — Dado um lembrete, uma nota e um texto salvo, a página mostra os três; "Apagar" remove o lembrete da lista e do armazenamento; a busca filtra por palavras sem acento. **Feito e testado (05/10).**

## Leitura e mapa de alterações

- `apps/desktop/src/settings/Personal.tsx` → `PersonalSection` + `Host::reminders_all/reminder_remove/notes_all/note_remove` + comandos Tauri; new.

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
