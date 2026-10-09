# Patch 1.7.0 immutable SOURCE replay

Extract `replay-archive.tar.gz` into a fresh directory with a safe data-only tar extractor. Archive contains46 original sealed files plus `seals.json`:47 members,55,573 compressed bytes. Archive SHA256 `955255a99902a3076c3eebe50798b4fa78f399251e80afbdc3c4e246400822c7`; original map SHA256 `38129730211271ef3223ec1b3a76ec92ebdd0d9eb2c8a444a81c2acc07e9b153`. Never rewrite originals/map/archive or backfill later receipts.

Run isolated Python against absolute copied paths; every CLI cwd remains `/home/osso/.worktrees/wow-ui-sim-p170-page`:

```text
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B /ABSOLUTE/COPY/audit.py
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B /ABSOLUTE/COPY/test_source_accounting.py
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B /ABSOLUTE/COPY/test_portable.py /ABSOLUTE/NEW-RECEIPT.json
```

Use argv-style `cli.command(...).cwd('/home/osso/.worktrees/wow-ui-sim-p170-page').capture().run()` via Pyrun; no Bash/session/cwd switch. Receipt path must not exist. Modules resolve their copied evidence only: no Git/target/addons/current tools/network. Fresh copied execution at `702ddca36` passes validator,6 SOURCE tests and3 portable tests. Own-base generator CLI defaults and extractor `extract_text(raw)` defaults reproduce exact bytes; malformed `parse_symbol`/null `extract_text` preserve exception types/messages. No extractor CLI coverage-ledger or general shared-tool suite claim.

Portable tests copy originals again, reject an omitted serialized ledger reference and appended fabricated native log, then restore exact bytes/hashes/original map without resealing. [Portable controls](portable-proof.json) record both reject/restoration pairs. [Later actual ledger](later-proof-ledger.json) retains actual cwd/argv/revision/time/full streams/hash scopes; [archive identity](archive-identity.json) and [context](portable-context.json) preserve original identity. Temporary paths identify historical executions, not retained directories. Later receipts have separate `receipt-seals.json`; originals remain unchanged.

SOURCE development only: redirect target unexpanded/UNPROVEN, zero meaningful model/runtime/native or parent acceptance. [HANDOFF](HANDOFF.md) names main-owned ordered integration and primary-target research/native/final gates.
