# Roteiro de acessibilidade (010 TK-005)

Automático: `pnpm -C apps/desktop test` roda o axe (`src/a11y.test.tsx`) no
Overlay e em todas as páginas de Configurações — zero violações sérias ou
críticas. Contraste e leitores de tela exigem o app real:

| # | Passo | Esperado |
| --- | --- | --- |
| 1 | Narrador ligado; abrir o Overlay pelo atalho | Anuncia "Pergunte algo…" (campo de texto) |
| 2 | Enviar uma pergunta | A resposta é anunciada por frases (região `aria-live="polite"`), não token a token |
| 3 | Pedir algo que gere Aprovação | Cartão recebe foco; `A`/`R` funcionam; Narrador lê o comando |
| 4 | Pedido de permissão do agente | Anunciado como diálogo de alerta com as três opções |
| 5 | Tab por todo o Overlay e Configurações | Ordem lógica, foco visível com a cor de destaque |
| 6 | Alto contraste do Windows | Sem transparência; cores do sistema; tudo legível |
| 7 | "Reduzir animações" do Windows | Sem animações de entrada |
| 8 | Tema claro/escuro sobre papel de parede claro e escuro | Contraste AA do texto sobre o fundo translúcido (medir com o Accessibility Insights) |

Registre prints/vídeo e a versão do Windows como evidência do ticket.
