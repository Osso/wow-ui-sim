# Patch 1.10.1 immutable SOURCE replay

Extract `replay-archive.tar.gz` into a fresh directory. Only original sealed evidence and `seals.json` are present. Commands (replace COPY and NEW-RECEIPT with absolute paths):

```text
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B COPY/audit.py
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B COPY/test_source_accounting.py
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B COPY/test_portable.py NEW-RECEIPT.json
```

No Git/target/current tools/addons/network needed. Own-base generator CLI defaults and extractor `extract_text(raw)` defaults reproduce retained bytes; malformed `parse_symbol` and null `extract_text` reproduce exception types/messages. Not extractor CLI coverage-ledger or general tool-suite acceptance.

Original seals/archive immutable. Later actual revision/cwd/argv/env-key/time/fullstream/hash receipts are separately retained and sealed, never backfilled into original archive. Portable tests omit a serialized ledger reference and append a fabricated log line, require original seal rejection, then exact byte/hash/map restoration without resealing. SOURCE-only, not model/native or parent final acceptance.
