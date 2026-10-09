# Frozen 1.14.4 SOURCE replay

Original `seals.json` pins the exact original input/adapter/ledger/log bytes. Never rewrite originals or append later receipts to that map. Archive/current portable receipts are separately sealed in `receipt-seals.json`. Original commands/revisions in `proof-ledger.json`; copied commands/output in later `portable-controls.json`.

Extract `replay-archive.tar.gz` into a disposable directory. Copied inputs contain no Git/target/current tools/runtime/vendor/cache. Use absolute Python3.12+ and `-B`, empty PATH; run absolute copied `audit.py` and `test_source_accounting.py`. Generate a new register with copied `historical-tools/gen_patch_wikitext_register.py 1.14.4 <copied-source.wikitext> 2581777 <new-output>`; compare exact bytes with copied `default-register.json`. Historical extractor retained but unexecuted; no all-flags/shared-tool proof.

Harness command (explicit cwd `/home/osso/.worktrees/wow-ui-sim-p1144-page`):

```text
python3 -B /home/osso/.worktrees/wow-ui-sim-p1144-page/data/patch-api/evidence/1.14.4-session-2026-10-09/test_portable.py --archive /home/osso/.worktrees/wow-ui-sim-p1144-page/data/patch-api/evidence/1.14.4-session-2026-10-09/replay-archive.tar.gz --scratch /home/osso/.worktrees/wow-ui-sim-p1144-page/target/p1144-source-replay --receipts <new-absolute-receipt-path>
```

Three fresh disposable copies: validator/SOURCE11/default byte replay; serialized ledger omission rejection/restoration; fabricated GREEN log rejection/restoration. Exact original seals checked before and after. Scratch belongs to own target; harness refuses receipt overwrite. Subprocesses use absolute Python/empty PATH/explicit own-worktree cwd; copied adapters resolve files beside their script, not from cwd. Standalone copied replay can use any cwd with absolute paths.

SOURCE development only: no runtime/model/native/security/loaded-UI/linked-expansion/foreign-supersession/integration/final-gate credit. Main owns actual ordered Era successors and acceptance.
