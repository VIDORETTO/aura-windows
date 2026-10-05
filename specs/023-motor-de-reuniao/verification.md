# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `—`
- Acceptance: `AC-001`, `AC-003`
- Procedure: `cargo test -p aura-app (meeting::tests, meeting_is_opt_in_and_records_audio_only_while_it_runs); pnpm -C apps/desktop test`
- Execution: `executed`
- Environment: Linux x86_64 (fonte de falas falsa; áudio sintético)
- Tested revision: `local:4b8ad035e4f3c4fa4ad4949824a923a787dea24abf1269254c389aaa6f8a949a`
- Timestamp: `2026-10-05T17:32:32+00:00`
- Observations: Reunião só começa por chamada explícita, grava mic+sistema só enquanto roda e volta ao que a política diz; privacidade pausada bloqueia; falas guardadas uma vez com tempo e falante; pausa pula o intervalo; 'esqueci de iniciar' cria Reunião do buffer; reuniões interrompidas por queda são marcadas
- Evidence refs: none
- Limitations: AC-002 (transcrição contínua real Você/Eles) sem verificação: depende dos motores de voz locais ou da nuvem no Windows; apagar o áudio ao encerrar e retenção por reunião ainda não implementados; sem UI de iniciar/parar (esforço 024)

## EV-002 — partial

- Ticket: `—`
- Acceptance: `AC-003`
- Procedure: `cargo test -p aura-app (meeting_audio_is_erased_at_the_end_unless_kept_and_pii_is_masked, pii); vitest MeetingPrivacy/MeetingPanel`
- Execution: `executed`
- Environment: Linux x86_64 (áudio sintético)
- Tested revision: `local:4e7f116bc41ea12dd13cf58ec81f6ee7ff17c4eadb56e012c67a4ab5d80fe4c7`
- Timestamp: `2026-10-05T17:45:00+00:00`
- Observations: Áudio da reunião é apagado ao encerrar por padrão e guardado só se pedido; CPF/CNPJ/cartão (dígitos verificadores)/e-mail/telefone mascarados para o modelo quando ligado; aviso aos participantes copiável; opções em Configurações › Privacidade
- Evidence refs: none
- Limitations: Modo 'só local' (modelo local) não implementado; redação não cobre nomes nem endereços; a transcrição que o usuário vê continua completa; apagar áudio apaga também o buffer do usuário no mesmo intervalo

## EV-003 — partial

- Ticket: `—`
- Acceptance: `AC-002`
- Procedure: `cargo test -p aura-audio (mixed::tests); cargo clippy -p aura-win --target x86_64-pc-windows-msvc`
- Execution: `executed`
- Environment: Linux x86_64, fontes sintéticas
- Tested revision: `local:f5b9234f96ad8b9411a9de74bfc70fa074b1903df06930749791357ed0444928`
- Timestamp: `2026-10-05T17:58:34+00:00`
- Observations: Captura de todas as saídas: um loopback por dispositivo, convertidos a 16 kHz mono e somados; dispositivo com falha é ignorado; id '*' no hub; Reunião usa todas as saídas quando nenhum dispositivo foi escolhido
- Evidence refs: none
- Limitations: Não validado com WASAPI real; dispositivos plugados depois não entram até reiniciar a captura; soma simples sem sincronização fina entre dispositivos
