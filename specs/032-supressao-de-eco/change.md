---
schema: hybrid/change
schema_version: 1.0
effort_id: 032-supressao-de-eco
revision: 1
status: draft
profile: compact
---

# Change: Supressão de eco na transcrição da reunião

## Objetivo e limites

Parte do CAND-044. Com alto-falantes em vez de fone, o microfone ouve o outro lado de novo e a transcrição duplicava as falas como "você". Falas do usuário quase idênticas a uma fala "them" em até 4 s são descartadas. Não cobre áudio por aplicativo (process loopback), que segue candidato.

## Contrato de comportamento

- Entradas: [tipo e pré-condições]
- Saída: [resultado observável]
- Erros/invariantes: [casos relevantes]
- Compatibilidade: [o que deve permanecer igual]

## Requisitos e aceite

- **FR-001** — A transcrição MUST descartar fala "you" que repete (>=70% das palavras) uma fala "them" em até 4 s, e MUST manter falas curtas, tardias ou diferentes.
- **AC-001** — Dado "Podemos fechar o orçamento hoje" (them, 10 s), então o mesmo texto como you em 11,5 s some; "Sim, fecho com corte de dez por cento" (12 s), a repetição aos 30 s e "Sim" (curta) permanecem. **Feito e testado (05/10).**

## Leitura e mapa de alterações

- `crates/aura-app/src/meeting.rs` → `drop_echo` aplicado antes de gravar as falas da Reunião; new.

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
