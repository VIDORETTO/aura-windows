//! Agent extensions (`specs/008-extensoes-do-agente`).
//!
//! Codex already runs skills, MCP servers, plans and diffs; this crate holds
//! the rules Aura adds on top: Skill validation and installation, quick
//! command expansion, MCP server configuration (secrets stay in the vault)
//! and importers for other apps' MCP configs.

pub mod import;
pub mod mcp_config;
pub mod quick;
pub mod skills;
