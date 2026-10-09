# Patch 1.10.0 immutable SOURCE replay

Extract `replay-archive.tar.gz` into a fresh directory using a safe data-only tar extractor. Archive holds 25 original sealed files plus `seals.json`, 26 members, 49,534 compressed bytes. Do not rewrite original files, map or archive; later actual receipts remain outside it.

Run isolated Python against absolute copied paths:

```text
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B /ABSOLUTE/COPY/audit.py
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B /ABSOLUTE/COPY/test_source_accounting.py
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B /ABSOLUTE/COPY/test_portable.py /ABSOLUTE/NEW-RECEIPT.json
```

Recorded CLI cwd is `/home/osso/.worktrees/wow-ui-sim-p1100-page`; modules resolve only their copied evidence directory. No cwd/session switch, Git, target, current tools, addons or network inputs. Fresh copied execution at `a60968689` passes validator/5 SOURCE tests/3 portable tests. Original generator CLI defaults and extractor `extract_text(raw)` defaults reproduce exact bytes; malformed `parse_symbol` and null `extract_text` preserve retained exception types/messages. No extractor CLI coverage-ledger or general shared-tool suite claim.

Portable tests copy original sealed files again, reject an omitted serialized ledger reference and appended fabricated log claim, then restore original bytes, hashes and unchanged original map without resealing. Optional new receipt must not exist. [Later proof ledger](later-proof-ledger.json) retains actual argv/cwd/revision/times/full streams/scoped hashes; [portable context](portable-context.json) and [controls](portable-proof.json) retain archive/map and restoration hashes. Six actual later receipts plus supplemental `red-test_source_accounting.py` have separate [receipt seals](receipt-seals.json), seven files total. Supplemental RED fixture matches the original pre-format test hash; it is not inserted into the immutable archive. Temporary replay paths are historical invocation identities, not retained directories.

SOURCE development only. Redirect target remains unexpanded/UNPROVEN; zero meaningful model/runtime/native subset or main acceptance. [HANDOFF](HANDOFF.md) identifies main-owned gates.
