CREATE TABLE IF NOT EXISTS capture_segments (
  id TEXT PRIMARY KEY,
  source TEXT NOT NULL,
  kind TEXT NOT NULL,
  recording_id TEXT,
  start_ms INTEGER NOT NULL,
  end_ms INTEGER NOT NULL,
  path TEXT NOT NULL,
  bytes INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS capture_segments_source_time ON capture_segments(source, end_ms);
CREATE TABLE IF NOT EXISTS recordings (
  id TEXT PRIMARY KEY,
  source TEXT NOT NULL,
  title TEXT NOT NULL,
  started_at INTEGER NOT NULL,
  ended_at INTEGER,
  manual INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS access_log (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  at INTEGER NOT NULL,
  source TEXT NOT NULL,
  requester TEXT NOT NULL,
  tool TEXT,
  conversation TEXT,
  decision TEXT NOT NULL,
  reason TEXT,
  artifact_path TEXT
);
CREATE TABLE IF NOT EXISTS exclusion_rules (
  id TEXT PRIMARY KEY,
  process TEXT,
  title_glob TEXT,
  class TEXT,
  enabled INTEGER NOT NULL,
  builtin INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS agent_grants (
  conversation TEXT NOT NULL,
  source TEXT NOT NULL,
  granted_at INTEGER NOT NULL,
  PRIMARY KEY (conversation, source)
);
