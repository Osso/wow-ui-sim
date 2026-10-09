# Frozen 1.12.0 historical replay

Extract `replay-archive.tar.gz` into an empty directory. It contains 24 files, no Git, target, vendor/cache or current tools. Python standard library only; historical tools are unchanged copied base bytes.

1. Run `python3 -I -B /absolute/copied/path/audit.py` for original 23 seals, exact source/ledger validation and byte-for-byte default replay.
2. Run `python3 -I -B /absolute/copied/path/test_source_accounting.py` for five source/history controls.
3. Run `python3 -I -B /absolute/copied/path/test_portable.py` for three copied-evidence controls, including serialized ledger/log rejection and exact restoration.

No working-directory dependency. Recorded development commands used the owning worktree cwd, absolute script paths, `PATH=/nonexistent` and isolated Python in fresh processes. Nested temporary copies are removed after testing. Historical generator `source.path` remains `source.wikitext`; extractor replay is the `extract_text` function with unchanged defaults, not extractor CLI coverage behavior.

`seals.json` and archive are original epoch `3364a24c8`, created once. `receipt-seals.json` covers only later receipts; it must not replace/update original hashes. Seal-map authenticity depends on the retained map/archive, not a claim of resistance to an attacker replacing all evidence. Later docs/spec edits leave frozen original-doc snapshots intact.

Evidence is SOURCE-only. Redirect target content/revision and native historical contracts remain UNPROVEN. Zero meaningful model/runtime/native observations, no foreign-history supersession, no final gate.
