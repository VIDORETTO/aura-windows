CREATE TABLE IF NOT EXISTS meeting_actions (
  id TEXT PRIMARY KEY,
  meeting_id TEXT REFERENCES meetings(id) ON DELETE SET NULL,
  text TEXT NOT NULL,
  owner TEXT NOT NULL DEFAULT 'you',
  due TEXT,
  status TEXT NOT NULL DEFAULT 'open',
  t0 INTEGER,
  created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS meeting_actions_status ON meeting_actions(status, created_at);
