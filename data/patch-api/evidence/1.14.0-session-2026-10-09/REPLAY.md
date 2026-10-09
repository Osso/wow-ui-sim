# Frozen Era1.14.0 SOURCE replay

Original inputs, ledger, SOURCE GREEN/RED logs and historical code are sealed once in `seals.json`; never rewrite that map or original receipts. New observations use separate current receipt files/seals. `archive-pin.json` pins the archive and original map.

Use explicit cwd `/home/osso/.worktrees/wow-ui-sim-p1140-page` and an absolute Python executable. Unpack `replay-archive.tar.gz` into a new disposable directory. No Git, build target, current tools, package cache, vendor, CASC or network required. All adapter and generator inputs resolve from copied file locations; portable harness keeps explicit own worktree cwd and empties PATH in fresh child processes.

1. Run `/absolute/python -B /copied/audit.py`: validate every original seal and exact literal ledger.
2. Run `/absolute/python -B /copied/test_source_accounting.py`: SOURCE10 tests, not runtime/model/native proof.
3. Default `/copied/historical-tools/gen_patch_wikitext_register.py 1.14.0 /copied/source.wikitext 710568 /copied/reproduced.json` must fail at `: Scripts`, creating no output. Same invocation with existing `--skip-plain-scripts-label` must reproduce `opt-in-register.json` byte-for-byte. Defaults unchanged.
4. `/copied/audit.py extract` stdout must reproduce `default-extract.txt` byte-for-byte using historical extractor flags[].
5. Only in disposable copies, omit a serialized inventory row from `ledger.json` or replace `green.log`. Historical validator must reject at the corresponding seal, then accept after restoring exact original bytes. Never re-seal tampered data.

Own harness: `test_portable.py --archive /absolute/replay-archive.tar.gz --scratch /absolute/disposable --receipts /absolute/new-receipts.json`; refuses existing receipt overwrite. Archive inputs are independent own SOURCE evidence, not copied sibling results. Integration/native/final acceptance belongs to main.
