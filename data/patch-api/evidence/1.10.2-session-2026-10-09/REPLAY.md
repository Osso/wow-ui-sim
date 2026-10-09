# Patch 1.10.2 immutable SOURCE replay

Extract `replay-archive.tar.gz` into a fresh directory. Only original sealed inputs and `seals.json` are present. Use an isolated Python process:

```text
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B COPY/audit.py
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B COPY/test_source_accounting.py
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B COPY/test_portable.py NEW-RECEIPT.json
```

No Git, target, current tools, addons or network required. Generator CLI defaults and extractor `extract_text(raw)` defaults reproduce retained bytes; malformed `parse_symbol` and null `extract_text` replay retained exception types/messages. No extractor CLI coverage-ledger acceptance or general tool suite claim. Original seals/archive must not be rewritten. Later actual execution receipts stay outside the original archive and are separately sealed. SOURCE-only; no model/native or main acceptance.
