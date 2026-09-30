CREATE TABLE IF NOT EXISTS providers (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  preset TEXT NOT NULL,
  wire TEXT NOT NULL,
  base_url TEXT NOT NULL,
  extra_headers_json TEXT NOT NULL DEFAULT '{}',
  models_json TEXT NOT NULL DEFAULT '[]',
  credential_hint TEXT,
  status TEXT NOT NULL DEFAULT 'unverified',
  last_error TEXT,
  updated_at INTEGER NOT NULL
);
