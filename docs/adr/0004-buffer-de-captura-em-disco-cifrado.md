---
status: accepted
---

# Buffer de captura em disco, segmentado e cifrado

Os modos "últimos N minutos", gravação manual e contínua usam o mesmo mecanismo: segmentos curtos (tela: vídeo H.264 por hardware a 1–2 fps em segmentos de 10 s; áudio: Opus em segmentos de 10 s por fonte) gravados em `captures\`, cifrados com AES-256-GCM por segmento (chave protegida por DPAPI) e descartados por uma política de retenção. Nada é mantido em RAM além do segmento corrente, preservando o orçamento de memória; "salvar os últimos N minutos" apenas reclassifica segmentos existentes.

## Considered options

- Ring buffer em memória: rápido, mas custa centenas de MB para minutos de vídeo e perde tudo em falha.
- Screenshots JPEG por evento (screenpipe): excelente para busca, pior para reproduzir "o que aconteceu"; pode ser adicionado depois para a timeline (CAND-015).

## Consequences

- Recortar e anexar exige decodificar keyframes (worker, ADR 0005).
- A retenção é aplicada por tempo e por espaço máximo, com varredura na inicialização para remover sobras de falhas.
