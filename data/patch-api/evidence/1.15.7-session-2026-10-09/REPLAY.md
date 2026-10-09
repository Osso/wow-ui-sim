# Frozen 1.15.7 SOURCE replay

Extract `replay-archive.tar.gz` into a fresh directory. It contains 28 historical originals plus their seal map; no Git checkout, target, current tools, runtime/cache or network required. `portable-proof.json` preserves exact fresh-process argv/cwd/outputs and rejection/restoration hashes; `portable-context.json` records tested revision `ac1465079`. Do not regenerate original logs/archive/map or rewrite one-time receipts.

Run copied `audit.py` and `test_source_accounting.py` via `python3 -B` with the copied directory as cwd. Validator checks 28 historical input seals and exact serialized ledger. SOURCE8/8 is not native/runtime/final acceptance.

Run copied `historical-tools/gen_patch_wikitext_register.py` via `python3 -B`, positional argv `1.15.7`, absolute copied `source.wikitext`, `6778069`, and a new output JSON path. Recorded optional flags `[]`; resulting bytes must equal `default-register.json`. Empty inventory/header_counts does not prove empty linked changes or compatibility. Frozen extractor is retained unchanged, not executed or expanded.

`test_portable.py` outside the archive provides three bounded fixtures: fresh copied SOURCE8/default-byte replay, serialized ledger contract omission and green-log fabrication. Processes use `PATH=/nonexistent`, no PYTHONPATH/PYTHONHOME and no user site packages. Each copy is isolated under this worktree's own `target/source-replay-1.15.7/`, with neither Git nor target/runtime inside copied inputs. Both disk controls reject at exact seals, restore original bytes/hashes and validate afterward. Re-running fixtures without an output argument does not rewrite historical receipts; a supplied output must be new.

Original `seals.json` stays immutable. Later portable test/logs/context/receipts/archive/map/instructions have separate `receipt-seals.json`; it does not extend original proof. All 28 original inputs/map remain unchanged after controls.

Client name is absent from literal1.15.7; TOC11507 is literal, not native evidence. Era/Anniversary11507 static configuration and task-scoped same-Era1.15.8/1.15.9 pending context grant no native, model or successor credit. Main owns actual successor integration and final gates.
