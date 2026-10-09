# Original SOURCE replay

Original `seals.json` covers72 inputs; archive contains those plus the seal map. `archive-pin.json` pins73 members/145111bytes. No Git/target/current tools needed. Current Era getter observations and later proof receipts are separate, not backfilled into original source proof.

From explicit cwd `/home/osso/.worktrees/wow-ui-sim-p1142-page`, run Python with absolute evidence paths:

```text
python -B <evidence>/audit.py
python -B <evidence>/test_portable.py --archive <evidence>/replay-archive.tar.gz --scratch <own-scratch> --receipts <new-receipts.json>
```

Portable test extracts a fresh copy; subprocess cwd remains the explicit owned worktree, PATH empty. It validates historical seals, runs copied8 SOURCE tests, regenerates original default register bytes, rejects copied serialized ledger omission/log fabrication, then restores exact bytes and revalidates. Receipts refuse overwrite. Original archive/map/logs remain untouched.

`receipt-seals.json` covers later artifacts separately, including original seal-map hash. No final/native acceptance implied.
