CREATE TABLE IF NOT EXISTS recipes (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  notes_template TEXT NOT NULL,
  help_level TEXT NOT NULL DEFAULT 'onDemand',
  updated_at INTEGER NOT NULL
);
