---
schema: hybrid/ticket
schema_version: 1.0
id: TK-006
effort: 001-fundacao-overlay
type: delivery
status: implemented
ticket_revision: 4
requires: ["TK-004"]
requirement_refs: ["FR-006"]
acceptance_refs: ["AC-016", "AC-017"]
spec_revision: 2
plan_revision: 1
owned_areas: ["crates/aura-store/src/vault.rs", "crates/aura-store/src/protect.rs", "crates/aura-core/src/logging.rs", "crates/aura-core/src/secret.rs"]
verification_status: stale
last_update: Evidence invalidated after an input changed.
---




# TK-006 — Cofre local cifrado e logs com redação

## Objetivo e limites

Entrega o `Vault` (AES-256-GCM com chave de dados protegida por DPAPI), a abstração `SecretProtector` com adapters DPAPI e de teste, o tipo `Secret` sem vazamento em `Debug`, a camada de logging com redação e rotação.

Não inclui: Credential Manager para chaves BYOK (003 usa `keyring`; o `Vault` serve para dados locais do Aura como chaves de segmentos de captura e campos sensíveis do banco).

## Leitura em ordem

1. `specs/001-fundacao-overlay/plan.md` → "Chosen approach" (cofre) e OT-003.
2. `crates/aura-store/src/lib.rs` → `Store::open` (TK-004).
3. Docs atuais: `aes-gcm`, `secrecy`, Win32 `CryptProtectData/CryptUnprotectData`, `tracing-appender` (rotação).

## Decisões já resolvidas

- Chave de dados (DEK) de 32 bytes gerada com `OsRng`, guardada em `vault_keys.protected_key` via `SecretProtector::protect`; nonce de 12 bytes aleatório por valor; AAD = id do segredo.
- `Vault::seal(id, plaintext) -> ()` grava em `secrets`; `Vault::open(id) -> Option<Secret<Vec<u8>>>`.
- API pública expõe `seal_bytes/open_bytes` para outros módulos (segmentos de captura em 004/005 usam a mesma DEK derivada por HKDF com contexto `"capture-segment"`).
- Logs: `tracing` + `tracing-appender` com rotação por tamanho (implementar writer próprio se o crate só rotacionar por tempo), 10 MB × 5; campos com nome em `{api_key, token, secret, password, authorization}` e valores `Secret` são redigidos.
- Liberdade local: organização interna, nomes de erros.

## Mapa de alterações

- Novo: `crates/aura-store/src/protect.rs` → `SecretProtector`, `DpapiProtector` (`#[cfg(windows)]`), `StaticKeyProtector` (testes).
- Novo: `crates/aura-store/src/vault.rs` → `Vault::{new, seal, open, seal_bytes, open_bytes, derive_key(context)}`.
- Novo: `crates/aura-core/src/secret.rs` → `Secret<T>` (wrapper de `secrecy`).
- Novo: `crates/aura-core/src/logging.rs` → `init_logging(dir)`, `RedactionLayer`, `SizeRotatingWriter`.
- Existente: `crates/aura-store/src/migrations/0001_init.sql` já contém `vault_keys`/`secrets` (TK-004).

## Contrato técnico

- Entradas: `id: &str`, `plaintext: &[u8]`.
- Saídas: ciphertext persistido; leitura devolve `Secret`.
- Invariantes: texto claro nunca é escrito em disco nem em log; mesma DEK para a vida da instalação (rotação futura possível por `key_id`).
- Erros: `VaultError::{Protect, Unprotect, Decrypt, NotFound, Storage}`; falha de `Unprotect` (outro usuário/perfil corrompido) é distinguível.
- Efeitos: escrita transacional.

## Exemplos de aceite

- **AC-016**: `seal("teste", b"sk-test-SEGREDO-123")` → bytes de `aura.db` (+ `-wal`) não contêm `sk-test-SEGREDO-123`; `Store::open` novo + `open("teste")` → `sk-test-SEGREDO-123` (com `StaticKeyProtector` em Linux e `DpapiProtector` no Windows); Windows: blob protegido por usuário A passado a `CryptUnprotectData` em processo executado como usuário B (`runas`/serviço de teste) → erro. Se o ambiente impedir usuário B, registrar `not_run` com limitação e roteiro manual.
- **AC-017**: `tracing::error!(api_key = %"sk-live-XYZ", "falha")` → linha do arquivo contém `api_key=[REDACTED]` e não contém `sk-live-XYZ`; `format!("{:?}", Secret::new("abc"))` → `"[REDACTED]"`; escrever 55 MB de logs → exatamente 5 arquivos, cada um ≤ 10 MB.

## Dependências e sequência de execução

Depende de: TK-004.

- [ ] TK-006.1 Unit `Secret` Debug (red→green).
- [ ] TK-006.2 Integração `Vault` com `StaticKeyProtector` (AC-016 parte 1) red→green.
- [ ] TK-006.3 `DpapiProtector` + teste Windows (AC-016 parte 2 e 3).
- [ ] TK-006.4 `RedactionLayer` e `SizeRotatingWriter` (AC-017) red→green.
- [ ] TK-006.5 Regressão e evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-store vault -p aura-core logging secret`; `cargo nextest run -p aura-store --test dpapi` (Windows).
- Estado esperado: verdes; limitação de usuário B registrada se aplicável.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: ausência de segundo usuário no runner.

## Condição de retorno à planejadora

Retornar se DPAPI não estiver disponível em perfis móveis/corporativos testados, exigindo outra estratégia de proteção de chave.

## Relatório de saída

Relatar APIs públicas criadas, formato de armazenamento, resultados, EV refs e limitações.
