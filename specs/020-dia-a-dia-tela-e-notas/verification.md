# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `—`
- Acceptance: `AC-002`, `AC-003`
- Procedure: `cargo test -p aura-app (reminders, notes, reminders_and_notes_through_agent_tools); pnpm -C apps/desktop test`
- Execution: `executed`
- Environment: Linux x86_64
- Tested revision: `local:7131c2afdb472998fe4b16269483ba9b86fac48822e220e82bb94490eb491cb7`
- Timestamp: `2026-10-05T17:17:48+00:00`
- Observations: Lembretes únicos e recorrentes (diário, dias úteis, semanal) disparam uma vez e pulam ocorrências perdidas; evento chega à UI que chama notify; notas e salvos pesquisáveis sem acento; ferramentas MCP criadas
- Evidence refs: none
- Limitations: Toast real do Windows, persistência após reiniciar o Aura (só SQLite testado em memória), tela de lista/estrela em Salvos e ditado global para 'anota' não verificados; AC-001 (OCR de região) não iniciado

## EV-002 — partial

- Ticket: `—`
- Acceptance: `AC-001`
- Procedure: `cargo test -p aura-app region_text_goes_to_the_clipboard_without_a_chip; vitest RegionSelector`
- Execution: `executed`
- Environment: Linux x86_64 (OCR e área de transferência falsos)
- Tested revision: `local:b1ce01d5eaaf16e5ab93d0b71732220fc29f66cabefe4e6c5492e7e93b5239e0`
- Timestamp: `2026-10-05T17:20:03+00:00`
- Observations: Região congelada recortada, OCR, texto copiado, nenhum Chip criado, token de uso único, região minúscula recusada; seletor em modo cópia e comando /texto na UI
- Evidence refs: none
- Limitations: OCR real do Windows, área de transferência real, toast e janela do seletor não verificados (exigem Windows); tradução opcional não implementada

## EV-003 — partial

- Ticket: `—`
- Acceptance: `AC-003`
- Procedure: `vitest Save.test.tsx; cargo test -p aura-app (notes)`
- Execution: `executed`
- Environment: Linux x86_64
- Tested revision: `local:327af0fc60be3c835d0c3aba496df7bc66c56341f71042fda5653e37382bd91a`
- Timestamp: `2026-10-05T17:53:23+00:00`
- Observations: Estrela em cada resposta salva o texto (kind=saved); /salvos procura com note_search; notas rápidas por /anota e /notas
- Evidence refs: none
- Limitations: Sem tela de lista/apagar de notas e lembretes na UI; 'anota' por voz global não verificado

## EV-004 — partial

- Ticket: `—`
- Acceptance: `AC-001`
- Procedure: `cargo test -p aura-app region_text_becomes_the_selection_chip_for_translate; vitest RegionSelector`
- Execution: `executed`
- Environment: Linux x86_64
- Tested revision: `local:ca7001ff9c9915ddd321dcfedc4ae6c607d8947e92e4f88211a24debc8287cdd`
- Timestamp: `2026-10-05T18:08:58+00:00`
- Observations: /texto-traduzir: OCR da região vira o Chip de seleção e o shell pede /traduzir sobre ele; nada vai à área de transferência
- Evidence refs: none
- Limitations: Fluxo do shell (evento aura://run-quick) e OCR reais exigem Windows
