# TDD planejado — modelo e raciocínio

Spec r1 / plan r2. Cenários preparados na etapa documental e agora em execução autorizada pelo usuário. Estado e evidências atuais em `verification.md` (projeção do runner); este arquivo não aprova aceites. As seams documentadas foram adotadas no pedido de execução dos esforços/tickets.

## Seams e ciclo

UI pública `OverlayApp` com IPC controlado; APIs Host/app-server falso para persistência/envio; E2E WebView2 para geometria. Um caso → red pelo comportamento incorreto → mínimo → green → próximo. Caso já verde é cobertura existente; não fabricar red.

## TK-001 — acesso/recuperação

1. **AC-001**: provedor QA, `qa-reasoner`, compacto sem Conversa → botão abre diálogo modelo/esforço antes de `conversation_start`.
2. **AC-002**: Conversa em expandido → modelo/esforço selecionáveis, provedor bloqueado; não muda Turno em execução.
3. **AC-003**, casos separados: promessa pendente → carregando; rejeição `agent_unavailable` → erro/repetir; sucesso `[]` → vazio; botão sempre presente. Textos acessíveis distintos; sem snapshot de classes.
4. **AC-004**: catálogo A e `qa-reasoner/max`; refresh falha → conserva A/escolha e avisa; repetir com B/`qa-fast` → diálogo aberto atualiza. Outro caso: A/B concorrentes, B resolve primeiro → B permanece. Erro ChatGPT com BYOK válido não apaga BYOK.

## TK-002 — escolha/memória

5. **AC-005**: `qa-reasoner`, `[low, high, max]`, padrão `high`; escolher max/enviar oi → campos relevantes `model=qa-reasoner, effort=max`; medium ausente. Modelo sem esforço → explicação e esforço null/omitido conforme IPC existente.
6. **AC-006**: salvar low/Chat e max/Tarefa para QA/qa-reasoner; alternar modos/modelos, reiniciar e selecionar o mesmo modelo → Chat low, Tarefa max. Outro modelo não herda valor incompatível. Persistência pelas APIs públicas com store real temporário.
7. **AC-007**: A suporta max, sucesso B só `[low, high]` → aviso e padrão válido no envio. Controle: erro de refresh mantém max. Caso adicional: catálogo chega depois do render → preferência válida salva é restaurada.

## TK-003 — janela/acessibilidade

8. **AC-008**: Windows, área cliente 480 pixels lógicos, nomes longos, compacto/expandido, DPI 100/150/200% → retângulo positivo/visível/acionável, menu e última opção alcançáveis sem corte físico. Inspeção visual complementa DOM; incluir monitor com menor área útil. Não mockar layout para alegar DPI.
9. **AC-009**: Tab, Enter abrir, Tab/Enter escolher, Escape → menu fecha/foco volta/Overlay permanece. pt-BR/en e paridade de i18n. Navegação por setas adicional somente se exigida pelo contrato acessível existente.

## Evidência futura

Registrar revisão, caso/filtro, motivo do red, resultado green, ambiente, limites demo/conta e AC com `evidence add`. Não copiar EV antigos. Falta de driver/conta/executável é not_run, não red. Comandos no plano/tickets.
