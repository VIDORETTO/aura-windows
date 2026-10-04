-- 011 TK-031: thumbnail of what the agent received, sealed by the vault.
ALTER TABLE access_log ADD COLUMN thumb BLOB;
