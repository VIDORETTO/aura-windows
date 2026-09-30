---
status: accepted
---

# Tauri 2 com host Rust e UI React

O app é um host Tauri 2 em Rust com UI React 19/TypeScript em WebView2. Rust dá acesso direto a Win32/WinRT (WGC, WASAPI, UI Automation, DPAPI) e consumo ocioso baixo; WebView2 já vem no Windows 10/11 e mantém o instalador pequeno. A UI conversa com o host apenas por comandos/eventos tipados gerados com `tauri-specta`.

## Considered options

- Electron: memória ociosa alta e runtime embutido grande, contra o requisito de leveza.
- WinUI 3/.NET nativo: ótima integração visual, porém ecossistema de chat/markdown menor e sem reaproveitar referências Tauri (ThukiWin, Handy, AI Overlay).
- Qt/Python (Clicky): distribuição frágil.

## Consequences

- Build, E2E e medição de desempenho exigem Windows; crates de domínio compilam e testam em Linux.
- Efeitos visuais (Acrylic/Mica) via `window-vibrancy`, com fallback no Windows 10.
