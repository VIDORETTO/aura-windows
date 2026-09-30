CREATE TABLE IF NOT EXISTS quick_commands (
  name TEXT PRIMARY KEY,
  template TEXT NOT NULL,
  builtin INTEGER NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE IF NOT EXISTS mcp_servers_meta (
  name TEXT PRIMARY KEY,
  spec_json TEXT NOT NULL,
  updated_at INTEGER NOT NULL
);
