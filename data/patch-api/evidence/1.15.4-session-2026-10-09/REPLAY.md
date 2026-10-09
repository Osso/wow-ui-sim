# Patch 1.15.4 SOURCE replay

Original `seals.json` pins44 files/701134 bytes. Never rewrite those files/logs or append later proof to that map. Current receipts and archive are sealed separately. Exact original commands/revisions in `proof-ledger.json`; current portable command/revision/result in `portable-proof.json` after replay.

```text
python3 -B /home/osso/.worktrees/wow-ui-sim-p1154-page/data/patch-api/evidence/1.15.4-session-2026-10-09/test_portable.py --archive /home/osso/.worktrees/wow-ui-sim-p1154-page/data/patch-api/evidence/1.15.4-session-2026-10-09/replay-archive.tar.gz --scratch /home/osso/.worktrees/wow-ui-sim-p1154-page/target/p1154-source-replay --receipts /home/osso/.worktrees/wow-ui-sim-p1154-page/target/p1154-source-replay/new-receipts.json
```

Explicit cwd `/home/osso/.worktrees/wow-ui-sim-p1154-page`. Use new receipt path; harness refuses overwrites. Three disposable copies contain no Git/target/current tools/runtime/vendor/cache. Fresh absolute-path Python processes run with empty PATH; adapter inputs resolve beside copied script, not cwd. Scratch belongs to this worktree’s own target.

Fixtures assert copied historical validator, SOURCE9/9, historical default-generator flags[] byte replay, serialized ledger/log seal rejection and byte-exact restoration. Original extractor retained unexecuted. SOURCE-only; no native/runtime/model/integration/final-gate credit. Main owns actual ordered Era successors and acceptance.
