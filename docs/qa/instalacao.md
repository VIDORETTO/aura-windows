# Roteiro de instalação e atualização (010 TK-001/TK-002)

> A estratégia posterior está em [Publicação e chave permanente](publicacao-e-chave-permanente.md).
> O usuário é o único instalador da 0.1.0 e agora quer abrir o repositório após
> anonimização. Endpoint preparado: `aura-windows/releases/latest/download/latest.json`.
> Chave permanente: `%USERPROFILE%\.tauri\aura-updater-permanent.key`, mesma pública
> do candidato novo. As decisões anteriores de canal separado são históricas.

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
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content -Raw "$HOME\.tauri\aura-updater-permanent.key"
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

O par permanente foi gerado em 05/10/2026: a pública está em `tauri.conf.json` (`plugins.updater.pubkey`);
a privada fica **fora do repositório** em `%USERPROFILE%\.tauri\aura-updater-permanent.key` (sem senha).
O par anterior (`aura-updater.key`) foi exposto na saída da ferramenta e só pode assinar
a transição 0.2.0 para permitir que a 0.1.0 atualize pelo app. Nunca o use depois dessa
versão. Não execute a ajuda do CLI com uma chave
privada carregada no ambiente: o CLI pode imprimir o valor da variável.
Guarde uma cópia segura: sem ela, as versões instaladas não aceitam atualizações assinadas
(seria preciso reinstalar). Para o workflow `release.yml`, cadastre o conteúdo do arquivo no
secret `TAURI_SIGNING_PRIVATE_KEY` do repositório (e `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` vazio).
O workflow recebe apenas a chave permanente para releases futuras. O release 0.2.0
usa uma assinatura de transição única no `latest.json`; após atualizar, o app verifica
releases com a chave permanente.

## Transição da 0.1.0

A 0.1.0 aponta para `VIDORETTO/aura-windows`. O repositório agora está público no mesmo
endereço e o endpoint funciona sem autenticação. O 0.2.0 é assinado uma vez com a chave
que a 0.1.0 já confia; o binário 0.2.0 traz a chave permanente nova.

Quem usa a 0.1.0 pode verificar e instalar a atualização em Configurações › Sobre ›
Verificar atualizações, sem reinstalação manual nem remoção dos dados. O fluxo completo
foi testado com a versão 0.1.0 instalada em pasta isolada: o app instalou a 0.2.0 no
mesmo caminho e o perfil do provedor persistiu.

O workflow usa build com motores DirectML e recebe `TAURI_SIGNING_PRIVATE_KEY` permanente
como secret. A execução automática por tag continua retida; releases futuras exigem
dispatch manual com uma tag `v*`. Restam validar o build hospedado, instalação em Windows
limpo e testes de acessibilidade/cobertura listados no relatório.
Relatórios: [QA 0.2.0](release-020-2026-10-05.md) e
[publicação e chave permanente](publicacao-e-chave-permanente.md).
