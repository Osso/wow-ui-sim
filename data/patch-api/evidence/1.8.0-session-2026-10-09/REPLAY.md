# Patch 1.8.0 immutable SOURCE replay

Safely extract `replay-archive.tar.gz` into a fresh directory with a data-only tar extractor. Archive:40 original sealed files plus `seals.json`,41 members,52,231 compressed bytes. [Archive identity](archive-identity.json) records exact archive/map hashes. Never rewrite originals, map or archive; later actual receipts remain outside.

Run absolute copied paths, with every CLI cwd `/home/osso/.worktrees/wow-ui-sim-p180-page`:

```text
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B /ABSOLUTE/COPY/audit.py
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B /ABSOLUTE/COPY/test_source_accounting.py
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B /ABSOLUTE/COPY/test_portable.py /ABSOLUTE/NEW-RECEIPT.json
```

Actual fresh copied replay at `a3e47eb41`: validator exits0, SOURCE5/5, portable3/3. Copied modules resolve only their evidence directory; no Git/target/current-tools/addons/network inputs. Generator CLI defaults and extractor `extract_text(raw)` defaults reproduce exact bytes. Malformed `parse_symbol` and null `extract_text` preserve retained exception types/messages; no extractor CLI coverage-ledger/general shared-tool acceptance claim.

Portable tests copy original sealed evidence again, reject an omitted serialized ledger reference and appended fabricated native log claim, restore exact original bytes/hashes/map and replay without resealing. Optional new receipt must not exist. [Portable controls](portable-proof.json) retain rejection/restoration hashes; [context](portable-context.json) records actual copy identity (temporary directory subsequently removed). [Later proof ledger](later-proof-ledger.json) records actual revision/cwd/argv/times/full streams/scoped hashes. [Receipt seals](receipt-seals.json) are separate, never inserted into the original archive/map.

[Original proof ledger](proof-ledger.json): SOURCE RED5/portable RED3 at base `a9d2433b6`; SOURCE GREEN5 at `44ebe2cb2`. Exact RED test/scaffold fixtures retained. [Original docs](original-docs/) retain the earlier epoch where portable proof was pending; current tracked docs record later results without backfilling them.

SOURCE development only. Redirect target remains unexpanded/UNPROVEN, zero meaningful model/runtime/native subset. [HANDOFF](HANDOFF.md) identifies main-owned integration and acceptance.
