---
status: accepted
---

# Worker sob demanda para cargas nativas pesadas

Inferência ASR, renderização de PDF e decodificação de vídeo rodam em `aura-worker.exe`, processo filho iniciado sob demanda e encerrado após inatividade (padrão 2 min), falando JSON-RPC por stdio. Encerrar o processo é a forma confiável de devolver ao Windows a memória de modelos ONNX/GGUF e bibliotecas nativas (PDFium, Media Foundation), e isola falhas dessas bibliotecas do host residente.

## Consequences

- Primeira transcrição após ociosidade paga o custo de carga do modelo; a UI mostra estado "aquecendo" e o usuário pode manter o worker quente enquanto o overlay estiver aberto.
- A Interface `Transcriber`/`Ingestor` é a mesma no host; o adapter real fala com o worker e o de teste roda em processo.
