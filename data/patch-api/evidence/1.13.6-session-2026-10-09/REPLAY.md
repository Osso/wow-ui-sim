# Original frozen SOURCE replay

`seals.json` pins88 original inputs; `replay-archive.tar.gz` contains89 members,138222bytes, independently pinned by `archive-pin.json`. Original sealed files/map/archive never change. `receipt-seals.json` pins later receipts/current snapshots and original seal-map/archive hashes separately. Current getters do not alter SOURCE measurements.

Use fresh receipt paths; portable replay refuses overwrite. Every command must use explicit owned cwd and absolute paths:

```python
cli.command('python3','-B',
 '/home/osso/.worktrees/wow-ui-sim-p1136-page/data/patch-api/evidence/1.13.6-session-2026-10-09/test_portable.py',
 '--archive','/home/osso/.worktrees/wow-ui-sim-p1136-page/data/patch-api/evidence/1.13.6-session-2026-10-09/replay-archive.tar.gz',
 '--scratch','/home/osso/.worktrees/wow-ui-sim-p1136-page/build/new-p1136-replay',
 '--receipts','/home/osso/.worktrees/wow-ui-sim-p1136-page/build/new-p1136-receipts.json') \
 .cwd('/home/osso/.worktrees/wow-ui-sim-p1136-page').run()
```

Fresh archive copy contains no Git, target or current tools. Child processes keep explicit owned cwd with empty PATH. Copied SOURCE7 validates exact frozen inputs; unchanged historical generator and default extractor reproduce saved bytes. Serialized ledger omission and fabricated log fail original seals; tests restore exact bytes, recheck every original seal and rerun copied validator. Linked/transcluded pages remain unexpanded. This is bounded SOURCE testing, not model/native/integration/final acceptance.
