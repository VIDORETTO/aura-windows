# Roteiro de instalação e atualização (010 TK-001/TK-002)

| # | Passo | Esperado |
| --- | --- | --- |
| 1 | `pnpm -C apps/desktop tauri build` (com `TAURI_SIGNING_PRIVATE_KEY`) | Gera `*-setup.exe` (NSIS por usuário), `.msi`, `.sig` e `latest.json` |
| 2 | Instalar sem privilégios de administrador | Instala em `%LOCALAPPDATA%\Programs\Aura`; atalho no Menu Iniciar |
| 3 | Primeira execução | Overlay com "Continuar com ChatGPT"; bandeja com o ícone; download do app-server com progresso |
| 4 | "Iniciar com o Windows" ligado; reiniciar | Aura sobe com `--background` (Overlay oculto) |
| 5 | Sobre › Verificar atualizações com uma release mais nova publicada | Baixa, verifica a assinatura, instala e reinicia |
| 6 | Desinstalar | Remove o app; dados em `%LOCALAPPDATA%\Aura` ficam (documentado) — "Apagar meus dados" antes remove tudo, inclusive o Cofre |

## Gerar o instalador (Windows 10 e 11)

```powershell
./scripts/prepare-sidecars.ps1 -Release -DirectML   # worker com voz local (ou -Skip para reaproveitar o já compilado)
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content -Raw "$HOME\.tauri\aura-updater.key"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
pnpm -C apps/desktop tauri build
```

Saída em `target/release/bundle/`:

- `nsis/Aura_<versão>_x64-setup.exe` — **recomendado**: instala por usuário em
  `%LOCALAPPDATA%\Programs\Aura`, sem administrador; atalho no Menu Iniciar; desinstalador.
- `msi/Aura_<versão>_x64_en-US.msi` — para implantação gerenciada (por máquina, pede administrador).
- `*.sig` — assinaturas do updater (para publicar numa release com o `latest.json`).

O que o instalador resolve num PC limpo:

- **WebView2**: `embedBootstrapper` instala o runtime se faltar (Windows 10); o Windows 11 já o traz.
- **Runtime do Visual C++**: `aura.exe` liga o CRT estaticamente; o `aura-worker` (motores de voz)
  usa `msvcp140`/`vcruntime140`, copiados para a pasta do app por `prepare-sidecars.ps1`
  (implantação local dos arquivos Redist da Microsoft), junto com o `DirectML.dll` do onnxruntime.
- **App-server do Codex**: baixado e verificado no primeiro uso (precisa de internet).

Sem certificado de assinatura de código, o Windows SmartScreen mostra "O Windows protegeu o
computador" no primeiro uso: **Mais informações › Executar assim mesmo**. Para remover o aviso,
assinar o `setup.exe` (Azure Trusted Signing ou certificado OV/EV) — ver `tauri.conf.json`
`bundle.windows.signCommand`.

## Chave do updater

O par foi gerado em 04/10/2026: a pública está em `tauri.conf.json` (`plugins.updater.pubkey`);
a privada fica **fora do repositório** em `%USERPROFILE%\.tauri\aura-updater.key` (sem senha).
Guarde uma cópia segura: sem ela, as versões instaladas não aceitam atualizações assinadas
(seria preciso reinstalar). Para o workflow `release.yml`, cadastre o conteúdo do arquivo no
secret `TAURI_SIGNING_PRIVATE_KEY` do repositório (e `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` vazio).
Falta o teste de atualização de ponta a ponta (010 AC-004): publicar a versão N, instalar,
publicar N+1 e atualizar pelo botão.
