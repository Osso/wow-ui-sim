# Patch 1.5.0 SOURCE replay — pre-portable epoch

Copy archive into fresh directory, safely extract only relative regular-file members. Run `/usr/bin/python3 -I -B <copied>/audit.py`, `test_source_accounting.py`, `test_portable.py` from owned worktree via `cli.command(...).cwd(...)`, PATH=/nonexistent and PYTHONDONTWRITEBYTECODE=1. Expected SOURCE6/portable3 and exact default bytes/errors; serialized ledger/log changes must reject and restore exact bytes/hash/map without resealing. Portable execution pending at original snapshot epoch; later receipts must remain separate.
