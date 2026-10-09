# Historical replay

From any ordinary checkout, execute own `audit.py` using Python3.11+. It validates160 original seals plus complete literal ledger equality. Original `proof-ledger.json` records SOURCE development scope; `current-proof.json` records later portable/direct-state results separately.

`test_portable.py --archive <absolute replay-archive.tar.gz> --scratch <absolute scratch> --receipts <new absolute JSON>` creates fresh copies, strips PATH, runs copied SOURCE8/default register/default extract replay, rejects ledger and log tampering, restores exact bytes and revalidates every original seal. No Git, target or current tools required. The retained161-member archive contains all original inputs and seal map; it intentionally excludes later receipts/current tests.

Original seal map and archive are immutable. Validate later files against `receipt-seals.json` independently; never backfill original evidence with later GREEN receipts.

Existing-model test (not historical/native replay): Cargo target `patch_1_14_1_text_scale`, offline/no-default-features/client-era, own worktree target and normal existing package home. Exact command/revision/full warning log in current receipts. No runtime implementation changes or native numerical/default inference.
