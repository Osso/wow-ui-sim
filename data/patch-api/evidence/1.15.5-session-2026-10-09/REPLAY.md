# Patch 1.15.5 SOURCE replay

Original `seals.json` pins 38 files; never rewrite originals or append later proof to this map. `receipt-seals.json` separately pins archive, original map and later receipts. Exact identities/counts in `ledger.json`; commands/revisions in `proof-ledger.json` and `portable-proof.json`.

```text
python3 -B /home/osso/.worktrees/wow-ui-sim-p1155-page/data/patch-api/evidence/1.15.5-session-2026-10-09/test_portable.py --archive /home/osso/.worktrees/wow-ui-sim-p1155-page/data/patch-api/evidence/1.15.5-session-2026-10-09/replay-archive.tar.gz --scratch /home/osso/.worktrees/wow-ui-sim-p1155-page/target/p1155-source-replay --receipts /home/osso/.worktrees/wow-ui-sim-p1155-page/target/p1155-source-replay/new-receipts.json
```

Run with explicit cwd `/home/osso/.worktrees/wow-ui-sim-p1155-page`. Use a new receipt path: harness refuses overwrites. Three disposable archive copies contain no Git, target, current tools, runtime, vendor or cache; fresh processes use absolute copied script paths and empty PATH. Copies are deleted after each fixture.

Fixtures exercise historical validator, SOURCE8/8, frozen default-generator flags[] byte replay, serialized ledger/log tamper rejection and exact restoration. Extractor is sealed but unexecuted. Evidence is SOURCE-only; no native/model/runtime or final-gate credit. Integration belongs to main.
