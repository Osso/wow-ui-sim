# Patch 1.3.0 immutable SOURCE replay

Safely extract replay-archive.tar.gz into a fresh data-only directory. Archive:46 original sealed files plus seals.json,47 members,62,573 bytes. SHA256 `41e92ad0d3d1eeecf60ce65790cd1ccc93e27d5c3242e1e13089762197db95f6`; original map SHA256 `c3e8100a8741778755a0ed7ebdfcbd952c37624429a9b6d36a2f44b6f0cbac70`. Never rewrite originals/map/archive or backfill later receipts. Original-docs preserve pre-copy epoch; current instructions are later, separately sealed.

```text
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B /ABSOLUTE/COPY/audit.py
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B /ABSOLUTE/COPY/test_source_accounting.py
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B /ABSOLUTE/COPY/test_portable.py /ABSOLUTE/NEW-RECEIPT.json
```

Every argv-style cli.command(...).cwd(...) invocation uses owned absolute cwd `/home/osso/.worktrees/wow-ui-sim-p130-page`; no Bash/session/cwd switch. Receipt destination must not exist. Copied modules resolve retained evidence and unchanged own-base tools only: no Git/target/addons/current tools/network.

Fresh copied execution at `3ea5b20ba` passes validator exit0, own SOURCE8/8 and portable3/3. Generator CLI default bytes and extractor extract_text(raw) default932 bytes reproduce exactly; malformed-symbol ValueError/null-input AttributeError reproduce exact messages. Portable controls copy originals again, reject both serialized ledger omission and appended fabricated native log, then restore exact bytes/hashes/original map without resealing. [Controls](portable-proof.json), [later ledger](later-proof-ledger.json) and [context](portable-context.json) retain actual epochs; receipt-seals.json covers later receipts separately. Originals/archive unchanged. Temporary paths in receipts are historical execution paths, not retained directories.

SOURCE development only:76 historical contracts UNPROVEN, model/runtime/native0/0/0. Main owns contract research, ordered integration, independent acceptance and parent/final/native closure. [HANDOFF](HANDOFF.md) records identity/counts/commits/limits.
