---
status: accepted
---

# Codex app-server como harness agêntico

O Aura usa o `codex-app-server` oficial da OpenAI, em versão fixada, como processo filho falando JSON-RPC por stdio, com `CODEX_HOME` isolado em `%LOCALAPPDATA%\Aura\codex-home`. É o harness documentado pela OpenAI para usar o plano ChatGPT do usuário em apps de terceiros (Sign in with ChatGPT → "Codex app-server", ver ADR 0007) e já entrega loop agêntico, MCP, skills, aprovações, sandbox Windows, histórico e compactação.

## Considered options

- OpenAI Agents SDK: sem login ChatGPT; exige Python/Node e Docker para sandbox no Windows.
- Codex SDK (TS): orientado a automação, sem aprovações interativas.
- Loop próprio: reimplementa o harness e continua sem login ChatGPT suportado.

## Consequences

- Provedores customizados só via Responses API → gateway local (ADR 0003).
- Cada release do Aura fixa uma versão do app-server, com SHA-256, schema TS gerado e suíte de contrato; atualizar o app-server é uma mudança versionada, nunca automática.
- `CODEX_HOME` isolado evita interferir no Codex pessoal do usuário; o login ChatGPT é feito pelo Aura (SIWC, ADR 0007), não pelo login interno do Codex.
- Recursos marcados como experimentais (`dynamicTools`, `collaborationMode`, `additionalContext`, realtime) só entram atrás de feature flag com teste de contrato.
