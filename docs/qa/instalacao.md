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

O par atual foi gerado em 05/10/2026: a pública está em `tauri.conf.json` (`plugins.updater.pubkey`);
a privada fica **fora do repositório** em `%USERPROFILE%\.tauri\aura-updater-2026-10.key` (sem senha).
O par anterior (`aura-updater.key`) está aposentado após exposição na saída da ferramenta;
nunca use a chave anterior para novas releases. Não execute a ajuda do CLI com uma chave
privada carregada no ambiente: o CLI pode imprimir o valor da variável.
Guarde uma cópia segura: sem ela, as versões instaladas não aceitam atualizações assinadas
(seria preciso reinstalar). Para o workflow `release.yml`, cadastre o conteúdo do arquivo no
secret `TAURI_SIGNING_PRIVATE_KEY` do repositório (e `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` vazio).
Falta o teste de atualização de ponta a ponta (010 AC-004): publicar a versão N, instalar,
publicar N+1 e atualizar pelo botão.

## Transição da 0.1.0 — publicação retida

A 0.1.0 aponta para `VIDORETTO/aura-windows`, que é privado: o endpoint do updater retorna
404 para acesso anônimo. Em 05/10 o usuário decidiu manter o código privado, interromper
a publicação, trocar a chave e usar um canal público separado para binários.
O endereço preparado no candidato é `VIDORETTO/aura-releases`; o canal ainda precisa ser
criado/configurado e receber os artefatos. Não há atualização automática funcional ainda.

Quem usa a 0.1.0 deverá encerrar o Aura e executar o instalador de transição por cima da
instalação existente, sem desinstalar nem apagar os dados. O identifier continua
`app.aura.desktop` e a migração de dados foi testada com o executável extraído do MSI
publicado, em perfil isolado. A instalação por cima e a atualização posterior pelo updater
ainda precisam de QA real antes de divulgar esse procedimento aos usuários.

Para liberar: conferir o canal público sem autenticação, configurar secrets com a chave
nova, adaptar o workflow ao repositório de distribuição e compilar o worker com motores
locais, validar o instalador em Windows limpo e sobre a 0.1.0, e testar uma atualização
assinada entre duas versões de transição. O workflow está sem gatilho automático por tag.
Relatório e evidências: [QA 0.2.0](release-020-2026-10-05.md).
