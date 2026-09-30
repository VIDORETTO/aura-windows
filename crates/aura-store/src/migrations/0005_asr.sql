CREATE TABLE IF NOT EXISTS downloads (
  model_id TEXT PRIMARY KEY,
  bytes_done INTEGER NOT NULL,
  total INTEGER NOT NULL,
  etag TEXT,
  state TEXT NOT NULL,
  updated_at INTEGER NOT NULL
);
