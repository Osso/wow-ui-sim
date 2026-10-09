# Patch 1.9.0 historical replay

Original `seals.json` covers30 files. `replay-archive.tar.gz` contains those30 files plus original map,31 members/51,931bytes. Never alter/reseal originals or archive. Later logs/receipts/HANDOFF/current docs are separate and do not replace original proof epochs. Expected bounded results: SOURCE6 tests, portable3 tests, one row/link/UNPROVEN contract, zero inventory/model/runtime/native.

## Portable replay

1. Copy only `replay-archive.tar.gz` to an empty directory; safely extract its regular relative-path members. Do not copy `.git`, `target`, current `tools` or `Interface`.
2. Run extracted `audit.py` using isolated Python: `python3 -I -B /absolute/copied/audit.py`. This checks all original seals, exact serialized literal ledger and historical default generator/extractor bytes/error classes/messages.
3. Run `python3 -I -B /absolute/copied/test_source_accounting.py`. This covers occurrences/counts/inventions/history/defaults and exact source-mutation rejection/restoration.
4. Run `python3 -I -B /absolute/copied/test_portable.py /absolute/copied/new-portable-proof.json`. Destination must not exist. Tests copy sealed files, reject both serialized ledger omission/log fabrication, restore exact bytes/map and replay after each restoration.
5. Keep new revision/cwd/argv/env-key/time/fullstream/hash receipts separate. Compare archive and map hashes from `portable-context.json`; never backfill originals with later results.

Within this child session every command must retain explicit CLI cwd `/home/osso/.worktrees/wow-ui-sim-p190-page`; scripts use their own resolved paths. Example Pyrun call, after choosing actual extracted absolute paths:

```python
cli.command('/usr/bin/python3', '-I', '-B', '/absolute/copied/audit.py').cwd('/home/osso/.worktrees/wow-ui-sim-p190-page').env({'PATH': '/nonexistent'}).capture().run()
```

Python standard library only; no Git, Cargo, assets, current repo tooling or network required by replay. Own retained replay used `/usr/bin/python3`3.14.7 and isolated `PATH=/nonexistent`; actual ephemeral paths/full streams live in `later-proof-ledger.json`. Artifact replay is SOURCE development proof, not independent/native/runtime/full-goal acceptance. [HANDOFF](HANDOFF.md) bounds missing contracts and main-owned work.
