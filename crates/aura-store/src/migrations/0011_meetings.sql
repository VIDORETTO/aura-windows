CREATE TABLE IF NOT EXISTS meetings (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  kind TEXT NOT NULL DEFAULT 'other',
  briefing TEXT NOT NULL DEFAULT '',
  origin TEXT NOT NULL DEFAULT 'live',
  status TEXT NOT NULL DEFAULT 'active',
  started_at INTEGER NOT NULL,
  ended_at INTEGER
);
CREATE INDEX IF NOT EXISTS meetings_started ON meetings(started_at);

CREATE TABLE IF NOT EXISTS meeting_utterances (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
  t0 INTEGER NOT NULL,
  t1 INTEGER NOT NULL,
  speaker TEXT NOT NULL,
  text TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS meeting_utterances_meeting ON meeting_utterances(meeting_id, t0);
