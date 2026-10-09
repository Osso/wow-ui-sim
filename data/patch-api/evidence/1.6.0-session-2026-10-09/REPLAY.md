# Patch 1.6.0 SOURCE replay

Immutable original `seals.json` and `replay-archive.tar.gz` will contain own inputs, tests, ledger, historical tools/default bytes/errors and original proof streams only. Later proof context/receipts remain separate. Never regenerate original seals or backfill later results.

Use Python standard-library `tarfile` to extract into a fresh temporary directory, inspect that every member is a relative file without `..`, `.git`, `target`, `Interface` or `__pycache__`. Run copied `audit.py`, `test_source_accounting.py` and `test_portable.py` with `/usr/bin/python3 -I -B`, `PATH=/nonexistent`, `PYTHONDONTWRITEBYTECODE=1`. All CLI calls from Pyrun must retain `.cwd('/home/osso/.worktrees/wow-ui-sim-p160-page')`; absolute copied file paths select isolated evidence, not current tools.

The validator checks original seals, exact frozen identity, serialized ledger equality and byte/error replay. Portable3 copies only sealed evidence, rejects a serialized ledger reference omission and fabricated serialized log, restores exact original bytes/hash/map and revalidates without resealing. SOURCE5 covers literal boundaries/counts/inventions/history/defaults, not native behavior.

Original seal/archive construction and copied execution still pending at this epoch. Main owns independent acceptance and ordered integration; target intentionally unexpanded.
