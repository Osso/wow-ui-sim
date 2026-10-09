# Patch 1.15.0 SOURCE replay

Original seals.json pins all original inputs, adapter, fixtures and RED/GREEN logs once. Never rewrite originals/map or append later proof to it. Archive and portable receipts sealed separately. Exact original commands/revisions/results in proof-ledger.json; copied replay results in portable-proof.json after execution.

```text
python3 -B /home/osso/.worktrees/wow-ui-sim-p1150-page/data/patch-api/evidence/1.15.0-session-2026-10-09/test_portable.py --archive /home/osso/.worktrees/wow-ui-sim-p1150-page/data/patch-api/evidence/1.15.0-session-2026-10-09/replay-archive.tar.gz --scratch /home/osso/.worktrees/wow-ui-sim-p1150-page/target/p1150-source-replay --receipts /home/osso/.worktrees/wow-ui-sim-p1150-page/target/p1150-source-replay/new-receipts.json
```

Explicit harness cwd `/home/osso/.worktrees/wow-ui-sim-p1150-page`. Use new receipt path; overwrites refused. Three disposable copies contain no Git/target/current-tool/vendor/cache/build outputs. Fresh absolute Python subprocesses use empty PATH; adapter/historical-generator inputs resolve relative to copied files, not cwd. Static bootstrap snapshot is retained inspection evidence, not loaded simulator/runtime proof. Scratch belongs to own worktree target.

Copied validator, SOURCE10/10, historical default-generator flags[] byte replay, serialized ledger/log seal rejection and exact restoration are development proof only. Empty default register does not erase explicit namespace prose. Historical extractor retained unexecuted. All behavior UNPROVEN; main owns actual ordered Era integration/native/security/final gates.
