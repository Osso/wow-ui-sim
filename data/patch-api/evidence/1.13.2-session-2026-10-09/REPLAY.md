# Patch1.13.2 original SOURCE replay

`originals.tar.gz` is a self-contained historical snapshot: source/response/manifest/101-page registry, literal ledger, page-local validator/tests, unchanged default tools and bytes/error, owned state/model reviews, original SOURCE/current-model logs and proof ledger. No Git tree, compiled target or current tools required. Python3.11+ standard library only; original capture uses Python3.14. Archive members are regular relative files; extraction controls reject unsafe names.

## Commands

From owned checkout, use the absolute Python executable and absolute owned paths:

```text
python3 -B data/patch-api/evidence/1.13.2-session-2026-10-09/test_portable.py --archive <absolute-originals.tar.gz> --scratch <absolute-owned-scratch> --receipts <new-absolute-receipt.json> --cwd /home/osso/.worktrees/wow-ui-sim-p1132-page
```

Each fixture extracts a fresh copy below owned scratch. Child commands run inside the copy with empty PATH, absolute Python, no Git/target/current tools. SOURCE8 reruns and historical generator bytes/default extractor exception must match. Disk ledger omission and fabricated GREEN log must fail on exact original SHA seals, then restore original bytes and pass all seals and validator again.

`seals.json` is created once, never rewritten. It seals original files but not itself or the archive. Later portable/current/successor receipts get a separate `receipt-seals.json`; they cannot backfill original proof or mutate the frozen ledger/queue. Runtime/native/history credit remains zero; the current Era generic CVar test is a separately identified current-model epoch, not a historical contract closure.
