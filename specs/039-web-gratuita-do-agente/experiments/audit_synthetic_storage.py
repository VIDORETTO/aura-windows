"""Read-only inspection of an explicitly supplied synthetic privacy fixture.

This is a research tool, not part of Aura's runtime. Reports marker names and
storage locations, never page bodies, arguments, queries or nonce values.
"""
import gzip
import json
import sqlite3
import sys
from pathlib import Path

import zstandard

MARKERS = {
    "uncited_url": b"https://independent.example/report",
    "uncited_snippet": b"Independent summary",
    "uncited_page_fact": b"Production: 30 units.",
    "uncited_page_instruction": "Ignore o usuário".encode(),
}


def decoded(data):
    if data.startswith(b"\x28\xb5\x2f\xfd"):
        return zstandard.ZstdDecompressor().decompress(data, max_output_size=64 * 1024 * 1024), True
    if data.startswith(b"\x1f\x8b"):
        return gzip.decompress(data), True
    return data, False


def audit(root):
    report = {"profile": "explicit synthetic test profile", "files": 0,
              "sqliteRows": 0, "compressedValues": 0, "hits": []}

    def inspect(data, location, format_name, **extra):
        found = [name for name, literal in MARKERS.items() if literal in data]
        if found:
            report["hits"].append({"location": location, "format": format_name,
                                   "markers": found, **extra})

    for path in root.rglob("*"):
        if not path.is_file():
            continue
        if path.is_symlink() or not path.resolve().is_relative_to(root):
            raise ValueError("fixture contains a path outside the supplied root")
        report["files"] += 1
        data = path.read_bytes()
        relative = str(path.relative_to(root))
        inspect(data, relative, "file_bytes")
        unpacked, compressed = decoded(data)
        if compressed:
            report["compressedValues"] += 1
            inspect(unpacked, relative, "compressed_file")
        if not data.startswith(b"SQLite format 3"):
            continue
        database = sqlite3.connect(path.as_uri() + "?mode=ro", uri=True)
        try:
            database.execute("pragma query_only=on")
            tables = list(database.execute("select name from sqlite_master where type='table'"))
            for (table,) in tables:
                quoted = '"' + table.replace('"', '""') + '"'
                columns = [row[1] for row in database.execute("pragma table_info(" + quoted + ")")]
                for row in database.execute("select * from " + quoted):
                    report["sqliteRows"] += 1
                    for index, value in enumerate(row):
                        cell = value if isinstance(value, bytes) else value.encode() if isinstance(value, str) else b""
                        cell, compressed = decoded(cell)
                        report["compressedValues"] += int(compressed)
                        inspect(cell, relative, "compressed_sqlite_cell" if compressed else "sqlite_cell",
                                table=table, column=columns[index])
        finally:
            database.close()
    report["limitations"] = ["Only four predefined synthetic literals; not a general data-loss proof",
                             "Reasoning outputs and compaction are not exercised by this fixture"]
    if report["compressedValues"] == 0:
        report["limitations"].append("No compressed frames observed; decompression branch not exercised")
    return report


if __name__ == "__main__":
    flags = set(sys.argv[3:])
    if len(sys.argv) < 3 or flags - {"--reasoning", "--compaction"}:
        raise SystemExit("usage: audit_synthetic_storage.py <own-fixture-root> <new-report.json> [--reasoning] [--compaction]")
    root = Path(sys.argv[1]).resolve(strict=True)
    output = Path(sys.argv[2])
    if output.exists():
        raise SystemExit("use a new report path")
    result = audit(root)
    if flags:
        result["fixtureModes"] = sorted(flag[2:] for flag in flags)
        result["limitations"] = [limitation for limitation in result["limitations"]
                                 if not limitation.startswith("Reasoning outputs")]
        if "--reasoning" in flags:
            result["limitations"].append("Provider reasoning fields exercised by the referenced scripted fixture; no commercial model execution")
        if "--compaction" not in flags:
            result["limitations"].append("Compaction not exercised by this fixture")
    output.write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(json.dumps(result, indent=2))
