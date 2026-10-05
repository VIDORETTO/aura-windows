CREATE TABLE IF NOT EXISTS reminders (
  id TEXT PRIMARY KEY,
  text TEXT NOT NULL,
  due_at INTEGER NOT NULL,
  repeat TEXT NOT NULL DEFAULT 'none',
  done INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS reminders_due ON reminders(done, due_at);

-- Quick notes (kind 'note') and saved answers (kind 'saved').
CREATE TABLE IF NOT EXISTS notes (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL DEFAULT 'note',
  text TEXT NOT NULL,
  created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS notes_kind ON notes(kind, created_at);
