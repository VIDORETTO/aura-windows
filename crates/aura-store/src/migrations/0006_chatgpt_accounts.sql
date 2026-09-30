CREATE TABLE IF NOT EXISTS chatgpt_accounts (
  client_id TEXT PRIMARY KEY,
  subject TEXT NOT NULL,
  email TEXT,
  scopes TEXT NOT NULL,
  plan_usage_enabled INTEGER NOT NULL,
  expires_at INTEGER,
  active INTEGER NOT NULL DEFAULT 0,
  welcomed INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL
);
