# TDD

AC-001: abrir A, arrastar B mantendo previous_app A; expandir deve manter B. Red nativo no código anterior, green no corrigido. Alternar compacto/expandido/Minibar e posição salva oposta, sem mock interno.

AC-002: guardar A, arrastar/salvar B, restaurar em ambas pelas APIs públicas de abertura/mode; largura/posição literais. Sem consultar SQLite lateralmente.

AC-003: A=(0,0,1920,1040)/96, B=(-2560,0,2560,1400)/144. Rect(-1000,100,640,100) pertence B; Rect(-100,100,640,100) pertence A. Fora de todos/lista vazia None. Um caso por vez, regressão placement completa. SO/foreground/binário ausente não são red comportamental. DPI real sempre identificado.
