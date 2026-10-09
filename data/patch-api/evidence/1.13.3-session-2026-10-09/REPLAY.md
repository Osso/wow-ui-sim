# Historical SOURCE replay

`originals.tar.gz` contains84 original sealed inputs plus `seals.json` (85members,181311compressed bytes). Original mapSHA256 `8325cb7578f6007d1533fa06360001be6a0b2667ce3100925814e5e7c623ed9e`. No Git metadata, target, installed current tools, Blizzard/runtime cache or dependency builds are copied. Python stdlib and the retained historical generator/extractor suffice.

Use an absolute own cwd and unused own receipt path. Example in Pyrun:

```python
own = '/home/osso/.worktrees/wow-ui-sim-p1133-page'
e = own + '/data/patch-api/evidence/1.13.3-session-2026-10-09'
cli.command('python3', '-B', e + '/test_portable.py',
    '--archive', e + '/originals.tar.gz',
    '--scratch', own + '/target/p1133-portable',
    '--receipts', own + '/target/p1133-new-receipt.json',
    '--cwd', own).cwd(own).run()
```

The three tests extract fresh copies, run their `audit.py` and SOURCE9 fixtures in fresh processes with empty PATH, replay22 historical generator entries byte-for-byte, and reproduce the exact default extractor ValueError/no output. Two disk controls omit serialized ledger inventory and fabricate GREEN log; validator rejects each under the original map, bytes restore exactly, all84 copied originals and post-restoration validator pass. The child cwd is explicit own root to honor command scope; all source/tool/data reads resolve from copied script `__file__`, not that cwd. `--cwd` is supplied, never an embedded historical path dependency.

`portable-proof.json` retains eight process runs/two restorations; `portable-command.json` records epoch/argv/cwd. Later receipts and model observations are separately sealed by `receipt-seals.json`, never added to or replacing84 original seals. `historical-spec.md` preserves the original proof-time state; current checked spec/wiki are later docs, not backfilled history. Historical red fixtures retain exact first-commit script bytes.

[Current derived proof ledger](current-proof-ledger.json) records development proof only. Current Era NPC-health test requires the separately declared standalone Cargo target, not this SOURCE replay; no native/loaded-UI/final acceptance is implied.
