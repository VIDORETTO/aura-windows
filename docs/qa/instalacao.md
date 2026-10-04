# Roteiro de instalação e atualização (010 TK-001/TK-002)

| # | Passo | Esperado |
| --- | --- | --- |
| 1 | `pnpm -C apps/desktop tauri build` (com `TAURI_SIGNING_PRIVATE_KEY`) | Gera `*-setup.exe` (NSIS por usuário), `.msi`, `.sig` e `latest.json` |
| 2 | Instalar sem privilégios de administrador | Instala em `%LOCALAPPDATA%\Programs\Aura`; atalho no Menu Iniciar |
| 3 | Primeira execução | Overlay com "Continuar com ChatGPT"; bandeja com o ícone; download do app-server com progresso |
| 4 | "Iniciar com o Windows" ligado; reiniciar | Aura sobe com `--background` (Overlay oculto) |
| 5 | Sobre › Verificar atualizações com uma release mais nova publicada | Baixa, verifica a assinatura, instala e reinicia |
| 6 | Desinstalar | Remove o app; dados em `%LOCALAPPDATA%\Aura` ficam (documentado) — "Apagar meus dados" antes remove tudo, inclusive o Cofre |

Antes da primeira release: gerar o par de chaves com `pnpm tauri signer generate`,
pôr a pública em `tauri.conf.json` (`plugins.updater.pubkey`), a privada nos
secrets do repositório, e conferir o endpoint (`VIDORETTO/aura-windows`) do updater.

Enquanto a chave pública for o texto `REPLACE_WITH_…`, o app instalado mostra
"Atualizações automáticas não estão configuradas nesta compilação" ao verificar
atualizações, e o workflow `release.yml` falha em `scripts/check-updater-key.mjs`
(também falha sem o secret `TAURI_SIGNING_PRIVATE_KEY`). Depois de configurar a
chave, falta o teste de atualização assinada de ponta a ponta (010 AC-004):
publicar uma versão N, instalar, publicar N+1 e atualizar pelo botão.
