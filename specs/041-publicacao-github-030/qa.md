# QA — publicação 0.3.0

Baseline de revisão: `ed9a9654c6e76e48186fc010a12eba77023b00f5`.
Ambiente: Windows 11 x64, Rust 1.98.1, pnpm 12.5.1; build local em 11/10/2026.

## Checks executados antes da publicação

| Procedimento | Resultado observado |
| --- | --- |
| `cargo test --workspace --exclude aura-desktop` | Exit 0; 508 passed, 0 failed, 39 ignored (opt-in/ambiente). |
| `pnpm -C apps/desktop test` | Exit 0; 253 passed, 33 arquivos. |
| `pnpm -C apps/desktop typecheck` | Exit 0. |
| `cargo fmt --all -- --check` | Exit 0. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0, inclui o shell Tauri Windows. |
| cargo-deny 0.20.2 `check licenses sources` | Exit 0; ferramenta obtida da release oficial e checksum conferido. |
| `./scripts/prepare-sidecars.ps1 -Release -DirectML` | Exit 0; worker 0.3.0 com motores e DLLs app-local. |
| `node scripts/check-updater-key.mjs` | Exit 0. |
| `pnpm -C apps/desktop tauri build --ci --bundles 'nsis,msi'` | Exit 0; frontend e Rust de produção, sem demo/e2e, ambos instaladores e `.sig`. |
| Verificador independente `minisign-verify` 0.2.5 | NSIS e MSI aceitos pela pública permanente; pública aposentada da 0.1.0 e bytes adulterados rejeitados. |
| Recursos PE / banco MSI (leitura via Windows Installer) | App, NSIS e ProductVersion MSI = literal 0.3.0; MSI inclui worker, DirectML e CRT. |
| Pública/endpoint embarcados na tag 0.2.0 vs configuração atual | Iguais; não rotacionados. |
| Links locais / título README | Sem links locais quebrados; um H1 fora de blocos de código. |
| Snapshot exato do índice Git | 1.232 arquivos inicialmente; zero cópias literais da privada do updater ou endereços Gmail. Perfis, SQLite, rollouts, capturas e binários excluídos. |
| Gitleaks 8.30.1, relatório 100% redigido | Exit 1 por 20 detecções já auditadas. Mesmas linhas do snapshot anterior: fixture de teste não operacional, pública/placeholder e hashes. Zero novas detecções. Não foi declarado exit 0. |

Logs locais ficam em `target/release-030-*.log` e relatórios em `target/release-030-*.json`; não são publicados com conversas ou chaves. A tentativa inicial de passar `nsis,msi` sem aspas no PowerShell foi rejeitada pelo CLI, antes do build do app; a execução correta acima passou. O controle negativo inicial usou uma pública com nome de arquivo antigo que era idêntica à permanente; o controle definitivo usa a pública aposentada real da 0.1.0.

## Instaladores preparados

| Asset | Bytes | SHA-256 |
| --- | ---: | --- |
| Aura_0.3.0_x64-setup.exe | 25.925.219 | `074197fc38cf02e886466346ee977ae753fa827ec03a72968b1755c64f2fbe45` |
| Aura_0.3.0_x64_en-US.msi | 33.370.112 | `00ee2377b15d518eac962384a5bc3ed558a7e513c4cfdcf8ecbb8f29af9216c7` |

Manifesto inclui `windows-x86_64`, `windows-x86_64-nsis` e `windows-x86_64-msi`, cada um com URL e assinatura do pacote correspondente, conforme a seleção real do updater 2.13.1. `sha256.json`/`sha256.txt` cobrem os arquivos publicados. Publicação e download remoto serão registrados separadamente após execução.

## Reavaliação das evidências anteriores

O incremento de versão mudou Cargo.toml/Cargo.lock e dois identificadores de cliente em aura-web (User-Agent e clientInfo.version MCP). Reverter **somente esses dois identificadores em memória** produz exatamente o fingerprint anterior do diretório inteiro aura-web: `ef0449ffa2db99720e01f0a64e47ad6e9861a7c9b3e8ae7fb0125c9be58ebc2d`. Fingerprint atual: `0ba0ed840152236291677f4305cec21ee9191be49380f409a2faadce33bad82f`.

A comparação dos demais inputs das EV-087/088/089/090/092/093/096 não encontrou mudanças em algoritmos, gateways, consumidores, política de privacidade, oráculos, harness de qualidade ou resultados externos. Suítes locais foram executadas novamente na 0.3.0. A reavaliação explícita fica em `input-revalidation.json`: evidência externa anterior é histórica e representativa da mesma implementação, não uma nova rodada de buscas ou de UI nativa. Disponibilidade contínua de provedores não é inferida.

## Limites preservados

- 038: seletores nativos em 150%/200% continuam pendentes; não aprovar por publicação.
- Instalação/atualização em Windows limpo não executada nesta rodada. Leitura do MSI e verificação de assinatura não substituem esse teste.
- Pesquisa ao vivo com ChatGPT, privacidade e monitor a 100% têm evidência própria nos esforços 039/040. O build 0.3.0 não foi usado para repetir esses cenários nativos.
- Actions segue desativado. Nenhuma execução local é atribuída ao CI remoto.

## Publicação e verificação remota executadas

Commit da release: `15c11ff913ff4f28ab768d99e8e4164438ee6043`, alcançável pela main remota e tag anotada v0.3.0. Publicada em **2026-10-11T04:27:36Z**, não draft/prerelease: [release](https://github.com/VIDORETTO/aura-windows/releases/tag/v0.3.0).

1. Draft com sete assets enviado por gh; digests GitHub correspondem aos arquivos locais. Download autenticado de todos os assets passou nos hashes, assinaturas anexas e seleção dos três targets.
2. Publicação via `gh release edit ... --draft=false --latest` exit0.
3. Download de todos os sete assets por urllib sem token/cookies: todos HTTP200, hashes correspondentes ao build e aos dois arquivos SHA publicados. Assinaturas NSIS/MSI baixadas verificadas independentemente contra a pública permanente, com chave aposentada da 0.1.0 e bytes corrompidos rejeitados.
4. `.../releases/latest/download/latest.json` anônimo retorna HTTP200/version0.3.0, SHA `4a3206b4a2404db065581b13d0c412240467e1030399f56987125668d7669547`, bytes idênticos ao asset da tag. API pública latest = v0.3.0; URLs/signatures dos três targets coerentes.
5. `git merge-base --is-ancestor` remote/local release/0.2.0 → main exit0; só depois, exclusão remote/local exit0. Inventário final tem somente main. Tags/releases 0.1/0.2 preservadas.
6. Descrição/homepage/oito tópicos verificados na API; default main. Actions continua enabled=false.

O snapshot final antes do commit tinha **1.245 arquivos**, zero privadas operacionais/Gmail e as mesmas 20 linhas de fixture/placeholder/hashes do audit anterior; Gitleaks exit1 classificado explicitamente. `git -c core.whitespace=-blank-at-eof diff --cached --check` passou: linhas vazias ao fim de documentos históricos foram mantidas para preservar fingerprints; formatação Rust passa pelo rustfmt. `git diff --quiet` passou antes do commit.

Relatório público sem dados operacionais em `public-verification.json`. A confirmação HTTP/hash não substitui instalação limpa nem execução real do updater; esses limites continuam declarados acima. Os registros de fechamento podem deixar main à frente da tag por um commit de documentação, sem alteração no código compilado da 0.3.0.
