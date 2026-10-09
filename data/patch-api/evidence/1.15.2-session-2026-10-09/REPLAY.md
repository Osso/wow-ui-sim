# Frozen 1.15.2 SOURCE replay

Original 54 input/log/ledger seals are immutable in `seals.json`. Archive55 members/109266 bytes, SHA256 `c313939cc05d72dc0f24cf5ab056f509ffb3a79f3bc5357db50806691c11a7a3`. Later portable receipts/seals are separate; do not rewrite original proof-ledger/logs/map.

Portable harness (explicit own-worktree cwd; scratch copies contain no Git, target or current tools):

```text
python3 -B /home/osso/.worktrees/wow-ui-sim-p1152-page/data/patch-api/evidence/1.15.2-session-2026-10-09/test_portable.py --archive /home/osso/.worktrees/wow-ui-sim-p1152-page/data/patch-api/evidence/1.15.2-session-2026-10-09/replay-archive.tar.gz --scratch /home/osso/.worktrees/wow-ui-sim-p1152-page/target/p1152-source-replay --receipts /home/osso/.worktrees/wow-ui-sim-p1152-page/data/patch-api/evidence/1.15.2-session-2026-10-09/portable-controls.json
```

Receipts path must not already exist. Harness starts fresh absolute Python subprocesses with empty PATH; adapter/generator resolve inputs relative to their copied files, never Git or cwd. Harness cwd is this worktree; that path must exist when invoking this harness elsewhere. For independent standalone relocation, extract archive into an empty directory and run its absolute `audit.py` and `test_source_accounting.py` with explicit cwd set to that copy. Regenerate default register using copied `historical-tools/gen_patch_wikitext_register.py` positional arguments `1.15.2 <copied-source.wikitext> 6061806 <new-output.json>` and compare bytes to copied default-register.json. No simulator/build/native client needed.

Controls remove a serialized ledger contract and replace serialized GREEN log with fabricated native credit. Each must reject at its exact seal, restore byte-identically, then pass validator. These are SOURCE development receipts only; main owns integration/native/final gates.
