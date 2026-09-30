CREATE TABLE IF NOT EXISTS app_profiles (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  process_pattern TEXT NOT NULL,
  title_glob TEXT,
  instructions TEXT NOT NULL DEFAULT '',
  attach_screen INTEGER NOT NULL DEFAULT 0,
  default_mode TEXT,
  default_model TEXT,
  updated_at INTEGER NOT NULL
);
