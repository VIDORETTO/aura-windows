---
schema: hybrid/spec
schema_version: 1.0
effort_id: 050-cofre-de-notas-v0
revision: 1
status: draft
profile: standard
---

# Cofre de notas v0

## Problem and desired result

O Aura deve publicar o que destila numa pasta Markdown compatível com o Obsidian e ler as notas do usuário como fonte do Acervo, sem exigir que a pessoa monte estrutura (plano §7, CAND-078, primeira fatia).

## Consumers and actors

Usuário com ou sem Obsidian; agente; Acervo (047).

## Scope

### Included

- Criar o Cofre com um clique numa pasta fora de Desktop e Documents (I-14).
- Publicar Reuniões e Notas em `Aura/` com propriedades YAML que respeitam o `.obsidian/types.json` (I-15).
- Indexar as notas do usuário no Acervo, em trechos por título de seção.
- Avisar quando houver Obsidian Sync, OneDrive ou git.

### Excluded

- Diário, Pessoas, Projetos e blocos gerenciados completos (Fase 4).

## User journeys and scenarios

### US-001 — Ver no Obsidian e buscar no Aura (P1)

#### Acceptance scenarios

- **AC-001** — Depois de uma Reunião, a nota dela aparece em `Aura/Reuniões/` com data e participantes nas propriedades, sem sobrescrever nenhum arquivo do usuário.
- **AC-002** — Uma nota do usuário no Cofre é encontrada por `memory_search`, com origem no caminho do arquivo.

## Requirements

- **FR-001** — Escrita só em `Aura/`, nunca reescrevendo texto humano; leitura do Cofre como fonte.

## Limits, errors, and compatibility

O Cofre é uma pasta Markdown; não depende do Obsidian aberto nem da CLI (complemento §7.3).

## Hypotheses and dependencies

Depende do 047.

## Success criteria

### Delivery-verifiable

- **SC-001** — AC-001 e AC-002 com evidência.

### Post-delivery observation

- **SC-002** — O usuário usa o Cofre sem conflitos de sincronização.

## Decisions and open questions

- Em discovery: a detecção de sincronizadores e o nome das pastas.
