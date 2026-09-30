---
status: accepted
---

# Captura de tela via GDI na primeira versão

A primeira versão captura com GDI: `BitBlt` + `CAPTUREBLT` para monitores e `PrintWindow(PW_RENDERFULLCONTENT)` para janelas (com fallback para `BitBlt` na área da janela quando o resultado vem preto). A política de privacidade e a redação continuam no `aura-capture`; o adapter fica em `aura-win::capture::GdiSource`, atrás do trait `FrameSource`, então trocar por Windows.Graphics.Capture não afeta o resto.

## Considered options

- Windows.Graphics.Capture (planejado): melhor para janelas aceleradas e HDR, mas exige dispositivo D3D11, mostra a borda amarela em versões anteriores ao Windows 11 e não pode ser validado sem Windows nesta fase.
- DXGI Desktop Duplication: eficiente para captura contínua, mas só monitores e com complexidade de reconexão.

## Consequences

- Captura sob demanda e buffer a 1 fps cabem no orçamento (≈15–40 ms por quadro 1080p).
- Janelas que renderizam só via DirectComposition podem sair pretas no modo "janela"; o fallback copia a área visível.
- Janelas com `WDA_EXCLUDEFROMCAPTURE` (o próprio Overlay) ficam de fora — a validar no Windows (docs/HANDOFF.md §4.2).
- Quando a gravação contínua precisar de mais fps ou HDR, implementar `WgcSource` no mesmo trait.
