# Patch 1.11.0 immutable SOURCE replay

`replay-archive.tar.gz` contains only original sealed inputs and `seals.json`. Extract into a fresh directory, then use an isolated Python process:

```text
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B COPY/audit.py
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B COPY/test_source_accounting.py
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B COPY/test_portable.py NEW-RECEIPT.json
```

No Git, target, current tools, network, addons or build required. Generator CLI defaults and extractor function `extract_text(raw)` defaults reproduce retained bytes; no extractor CLI coverage-ledger claim. Original seals/archive must never be rewritten. Later actual process receipts are outside the original archive and separately sealed. SOURCE-only, no model/native or main acceptance.
