# Frozen 1.15.3 SOURCE replay

Extract `replay-archive.tar.gz` into a disposable directory. Use an absolute Python3 executable (Python3.11+; portable harness tar extraction requires Python3.12+), empty PATH, `-B`. Run copied `audit.py` and `test_source_accounting.py` using absolute paths. Regenerate default register with copied `historical-tools/gen_patch_wikitext_register.py 1.15.3 <copied-source.wikitext> 6114194 <new-output>`; compare bytes to copied `default-register.json`.

`test_portable.py --archive <archive> --scratch <own-target/p1153-source> --receipts <new-file>` replays in fresh processes, removes one serialized ledger contract and fabricates GREEN log, checks exact seal rejection, restores bytes and validates again. Harness cwd is explicitly `/home/osso/.worktrees/wow-ui-sim-p1153-page`; copied adapter input resolution uses `__file__`, not Git/cwd/current tools/target/runtime/vendor/cache. Harness subprocesses use empty PATH and absolute Python. Original50 files and seal map immutable; archive and subsequent receipts sealed separately. `portable-red-archive.tar.gz` preserves empty-scaffold negative fixture, not production proof.

No simulator/native behavior, linked expansions, foreign supersession, shared-tool regression or final acceptance credit. Main owns integration/native gates.
