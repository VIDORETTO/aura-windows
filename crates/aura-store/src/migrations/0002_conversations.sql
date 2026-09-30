CREATE TABLE IF NOT EXISTS conversations_meta (
  thread_id TEXT PRIMARY KEY,
  conversation_uuid TEXT NOT NULL UNIQUE,
  workspace_path TEXT NOT NULL,
  mode TEXT NOT NULL,
  provider_id TEXT NOT NULL,
  granted_folders_json TEXT NOT NULL DEFAULT '[]',
  extra_instructions TEXT NOT NULL DEFAULT '',
  created_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS pending_deletions (
  path TEXT PRIMARY KEY,
  queued_at INTEGER NOT NULL
);
