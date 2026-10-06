# Diagnostico geral

`VIDORETTO/aura-windows` foi reconstruído com outro ID, histórico anonimizado e
aberto ao público em 06/10/2026. O repositório anterior está arquivado e privado
como `aura-windows-private-backup-2026-10`; seus objetos e metadados pessoais não
fazem parte do novo histórico. O SHA antigo consultado anonimamente no repositório
público retorna HTTP 404. `main` e `release/0.2.0` apontam para o código revisado.

Gitleaks 8.30.1, pacote oficial com SHA-256 conferido: 61 commits e 1.857 blobs
de histórico; 20 alertas revisados. Busca adicional pelas chaves privadas reais
do updater e por chaves privadas codificadas: nenhum resultado. O snapshot final
contém 999 arquivos; os alertas são os mesmos fixtures e exemplos sintéticos já
classificados. `.tauri`, `target`, modelos, sidecars e capturas locais ficaram fora
do Git. Esta análise cobre exposição de conteúdo e distribuição, não substitui uma
auditoria completa de segurança do aplicativo.

A versão 0.1.0 e seus quatro artefatos foram restaurados no endereço público; hashes
dos binários coincidem com a release anterior. O feed anônimo responde HTTP 200.
O candidato 0.2.0 usa o endpoint definitivo e inclui a chave pública permanente.
Sua assinatura de transição foi validada com as duas chaves e com dados adulterados.

# Vulnerabilidades encontradas

## Critico

Nenhuma credencial operacional foi confirmada nos arquivos ou no histórico público
novo. A cópia arquivada que contém os objetos antigos permanece privada.

## Alto

A chave privada anterior do updater foi exposta na conversa. O usuário autorizou
seu uso único para assinar a atualização 0.2.0, permitindo que instalações 0.1.0
verifiquem o salto. Essa chave fica aposentada imediatamente depois. A chave
permanente nova já está cadastrada em `TAURI_SIGNING_PRIVATE_KEY` no repositório e
sua pública embutida no binário 0.2.0; não gerar outro par a cada release.

## Medio

O histórico antigo permanece armazenado apenas na cópia privada arquivada. O novo
repositório tem outro ID; a consulta anônima pelo SHA com e-mail pessoal retorna
404. O histórico público foi reescrito com `Aura contributor` e endereço noreply.

## Baixo

O scanner acusa uma chave RSA exclusiva da fixture de OAuth local, exemplos
sintéticos de logging/diagnóstico, chave pública sintética e hashes SHA-256 de
evidências. Esses 20 alertas não são credenciais operacionais; nenhuma exceção
global foi adicionada para ocultar detecções futuras.

# TO-DO DE SEGURANCA E CORRECAO DE VULNERABILIDADES

## Prioridade 1 - Corrigir imediatamente

- Não usar mais a chave exposta após a única assinatura de transição 0.2.0.
- Proteger cópias offline da chave permanente em `%USERPROFILE%\.tauri\aura-updater-permanent.key`.

## Prioridade 2 - Alta prioridade

- Verificar o update ponta a ponta em instalação 0.1.0 e, depois, confirmar que
  a versão 0.2.0 verifica um artefato assinado pela chave permanente.
- Manter o endpoint `aura-windows/releases/latest/download/latest.json` estável.

## Prioridade 3 - Revisao estrutural

- Workflow corrigido para incluir engines DirectML e LLVM 20.1.8 com hash fixo.
  O workflow do repositório fica desativado até validar o build hospedado; o
  gatilho de release permanece manual e exige uma tag `v*`.
- Quando ativado, testar a saída do workflow contra a chave permanente antes de
  publicar futuras versões.

## Prioridade 4 - Auditoria avancada

- Fechar os gates de Windows da QA 036: instalação em Windows limpo, acesso real
  de ChatGPT, notificação de lembrete e verificações de acessibilidade/foco/DPI.

# Vulnerabilidades potenciais a investigar

Scanners não detectam todos os segredos possíveis. A chave real foi buscada
literalmente e os alertas foram examinados. Logs antigos de CI no backup privado
podem conter referências aos commits anteriores; não tornar esse backup público.
Nenhuma captura ou modelo foi incluído no Git.

# Revisao manual necessaria

O usuário informou ser o único instalador da 0.1.0 e autorizou usar a chave antiga
uma vez. A assinatura antiga foi testada: ela aceita o instalador 0.2.0; a chave
permanente rejeita essa assinatura e os bytes alterados também são rejeitados.
O binário 0.2.0 contém a chave pública permanente, usada em releases seguintes.

# Resumo executivo

Branch profissional `release/0.2.0`, autores anonimizados, repositório público no
endereço original e backup antigo privado/arquivado. Acesso anônimo à release
0.1.0 e ao feed foi verificado. O candidato 0.2.0 ainda não foi publicado neste
relatório; o teste de atualização dentro do app continua sendo gate antes de
declarar a distribuição concluída.
