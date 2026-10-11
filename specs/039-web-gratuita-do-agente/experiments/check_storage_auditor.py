"""Independent positive controls for compressed file and SQLite inspection."""
import gzip
import json
from pathlib import Path
import sqlite3
import tempfile
import zstandard
from audit_synthetic_storage import audit

with tempfile.TemporaryDirectory(prefix="aura-039-audit-controls-") as directory:
    root = Path(directory)
    # Literal fixture is independent of the scanner's marker dictionary.
    page = b"https://independent.example/report Independent summary Production: 30 units."
    (root / "compressed.gz").write_bytes(gzip.compress(page))
    (root / "compressed.zst").write_bytes(zstandard.ZstdCompressor().compress(page))
    database = sqlite3.connect(root / "fixture.sqlite")
    database.execute("create table items (payload blob)")
    database.execute("insert into items values (?)", (gzip.compress(page),))
    database.execute("insert into items values (?)", (zstandard.ZstdCompressor().compress(page),))
    database.commit()
    database.close()
    report = audit(root)
    rows = [hit for hit in report["hits"] if hit["format"] == "compressed_sqlite_cell"]
    files = [hit for hit in report["hits"] if hit["format"] == "compressed_file"]
    assert len(rows) == 2, "Both independent compressed SQLite cells must be detected"
    assert len(files) == 2, "Both independent compressed files must be detected"
    expected = {"uncited_url", "uncited_snippet", "uncited_page_fact"}
    assert all(set(hit["markers"]) == expected for hit in rows + files)
    print(json.dumps({"positiveControl": True, "gzipAndZstdFiles": 2,
                      "gzipAndZstdSqliteCells": 2, "threeKnownMarkersPerLocation": True,
                      "limitations": ["Auditor control only; no claim Codex produced compressed storage"]}))
