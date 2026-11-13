---
status: proposed
---

# Memória de tela por Momentos: evento, texto fora dos navegadores, OCR na resolução nativa

A memória de tela do Segundo Cérebro registra **Momentos**, não vídeo nem foto periódica. Um Momento nasce de um evento: troca de janela em primeiro plano, troca de título da janela em primeiro plano, pausa de digitação ou fim de rolagem. A captura acontece ~1,5 s depois do último evento. Os gatilhos vêm de `SetWinEventHook` fora do processo, só para `EVENT_SYSTEM_FOREGROUND` e `EVENT_OBJECT_NAMECHANGE`, **em dois ganchos separados, nunca uma faixa**. O texto vem por UI Automation **com orçamento de tempo** (desiste acima de ~200 ms e cai para OCR), **exceto em navegadores e apps Chromium/Electron/WebView2** (classe `Chrome_WidgetWin_*`). Esses vão por captura da janela + OCR do Windows, com título e endereço tirados do título da janela. Onde o build permitir (26100+), "algo visível mudou" vem do Windows.Graphics.Capture por janela: só chega quadro quando há mudança, e `DirtyRegions` traz o retângulo exato. O GDI do ADR 0008 fica como reserva. O OCR roda **sobre a captura na resolução nativa, antes de qualquer redução**. A imagem guardada de telas de texto é WebP sem perdas na resolução nativa; uma miniatura reduzida é opcional. Complementa o ADR 0004 (buffer de vídeo) e o ADR 0008 (GDI).

Evidência: `docs/research/segundo-cerebro-medicoes-2026-11.md` §4–§6.

- **UIA:** a leitura completa da janela levou 2,6 s no WhatsApp e 1,4 s de p95 no Word. Uma única leitura do Chrome ligou a acessibilidade completa dele; numa página muito dinâmica, cada tecla passou a levar ~12 s e o navegador ocupou ~2,3 núcleos.
- **Ganchos:** cada evento assinado custa ~0,19 ms a mais a quem o dispara; o gancho de faixa larga ainda encarece eventos não assinados.
- **OCR:** reduzir de 1.920 para 1.280 px antes do OCR derrubou o código de 91% para 24% das palavras reconhecidas. Na resolução nativa, o WebP sem perdas teve 44 KB em média.
- **WGC:** janela parada não gera quadro; uma mudança pequena gera um quadro por segundo, com o retângulo alterado.

## Considered options

- **Foto a cada N segundos** (Recall, OpenRecall): simples, mas guarda repetidos e exige comparar imagens o tempo todo.
- **UIA para todas as janelas, inclusive navegadores**: texto exato, mas liga a acessibilidade completa do navegador e trava a entrada em páginas dinâmicas. Fica como opção explícita, com aviso.
- **Reduzir a 1.280 px antes do OCR** (plano original): imagem menor, mas o OCR de letra pequena e código deixa de funcionar.
- **JPEG q80** (screenpipe): 223 KB médios em telas de texto, contra 44 KB do WebP sem perdas. **AVIF**: 0,3–0,9 s para codificar cada imagem, sem ganho de OCR.
- **Hash perceptual para "nada mudou"**: exige ler e comparar pixels a cada verificação; o WGC entrega essa informação de graça.

## Consequences

- OCR passa a ser o caminho principal nos navegadores: ~100–125 ms por imagem no worker. O agrupamento de eventos (1,5 s) limita a frequência.
- Janela anônima: o título da janela anônima do Chrome é igual ao normal. O nome UIA da raiz traz "(Modo anônimo)" (traduzido conforme o idioma), e o Edge marca "[InPrivate]" no título. O S3 mede se ler **só o nome da raiz** liga a acessibilidade web do Chrome; se ligar, a detecção de janela anônima do Chrome vira risco aberto. Na dúvida, não capturar.
- O WGC exige dispositivo D3D11 e build 26100+. Em builds anteriores, o GDI segue com o filtro por texto.
- O cursor de texto para de piscar após `CaretTimeout` (5 s neste PC), então uma janela parada com cursor não gera captura contínua.
- Assinar `NAMECHANGE` custa ~0,3% de um núcleo espalhado pelos processos que disparam o evento, a ~18 eventos/s. O S3 avalia restringir ao processo em primeiro plano (`idProcess`).
- Tudo foi medido em uso sintético num desktop rápido. O S3 oficial repete com um dia de uso humano real e num notebook modesto antes de este ADR ser aceito.
