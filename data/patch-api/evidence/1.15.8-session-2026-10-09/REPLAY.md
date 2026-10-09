# Frozen 1.15.8 SOURCE replay

Extract `replay-archive.tar.gz` into a fresh directory. No Git checkout, target, current tools, runtime/cache or network is needed. `portable-proof.json` records original commands, hashes, copied root and revision; these receipts are one-time historical evidence, never regenerate them or rewrite original `seals.json`.

Run copied `audit.py` and `test_source_accounting.py` using `python3 -B` with the copied directory as cwd. Validator checks 23 original input seals and exact serialized literal ledger. Own eight SOURCE fixtures are not runtime/native/integration/final acceptance.

Run copied `historical-tools/gen_patch_wikitext_register.py` via `python3 -B` with positional argv `1.15.8`, absolute copied `source.wikitext`, `6778071`, and a new output JSON path. Recorded optional flags are `[]`; compare resulting bytes to `default-register.json`. Empty inventory is not proof of empty linked diffs or compatibility. Frozen extractor is retained unchanged, not executed/expanded in this slice.

Serialized original `ledger.json` contract omission and fabricated `green.log` success were each rejected at exact seals then restored byte-for-byte. Those logs were not rewritten after sealing. Later portable receipts/archives have separate `receipt-seals.json`; original source input seals remain unchanged.

Client name is absent from frozen 1.15.8 literal page. TOC11508 is literal; Era/Anniversary11507 static configuration and queued1.15.9 separately attributed Classic context are not native proof. Main owns actual same-history successor/integration/native/final gates.
