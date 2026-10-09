# Cataclysm Classic 4.4.2 source accounting

Account for the pinned Warcraft Wiki page `619773`, revision `6303388`, without claiming simulator compatibility. The [source-only ledger](../../data/patch-api/sources/4.4.2-page-coverage.json) records literal TOC `40402` and the named `C_AuctionHouse` contract. See the [audit](../wiki/investigations/patch-4-4-2-api-audit.md) for source/runtime boundaries.

## What it must do

- [ ] Preserve returned response identity, exact wikitext bytes/hash, reproducible plaintext, and all six source rows: five metadata-only and one substantive statement. Acceptance command is listed below; checkbox awaits recorded proof.
- [ ] Keep the modern auction house and associated `C_AuctionHouse` APIs enabled in Cataclysm Classic contract **UNPROVEN**: no supported Cataclysm Classic runtime profile exists. Source identity proof cannot check off API compatibility.
- [ ] Preserve both linked diffs as unexpanded external references; navigation is not runtime supersession evidence.

## How it works

- [Source audit and proof matrix](../wiki/investigations/patch-4-4-2-api-audit.md).
- [5.5.4 profile/line decisions](../wiki/investigations/patch-5-5-4-api-audit.md).

## Implementation inventory

- `data/patch-api/sources/4.4.2-api-changes.wikitext`: pinned returned content, not a runtime surface.
- `data/patch-api/sources/4.4.2-api-changes.txt`: plaintext extraction.
- `data/patch-api/sources/4.4.2-page-coverage.json`: source-only ledger; not a publication register.
- `data/patch-api/evidence/4.4.2-session-2026-10-08/`: committed response/provenance and source-proof validator.

## Tests asserting this spec

`python3 data/patch-api/evidence/4.4.2-session-2026-10-08/validate.py` checks external serialized data only: returned revision/content, exact row matrix and rejected source/revision/credit/row mutations. `python3 tools/extract_patch_non_inventory.py --patch 4.4.2 --text-only --canonical-patch-navigation --check` checks plaintext reproduction. Neither command probes runtime behavior.

## Known gaps (current cycle)

- [ ] `C_AuctionHouse` namespace contract UNPROVEN due to unsupported Cata profile; individual members are not enumerated in this revision. No signatures, outputs, events, security, auction behavior or native-client parity measured. This limit remains open after source accounting completes.

## Out of scope

Runtime/new-profile/shared-classifier implementation; publication register generation; retail/Mists stand-in probes; external diff expansion; inferred API members or removals; numeric cross-line supersession; Cargo builds/runtime tests; native game probing. Existing tooling cannot represent Cata runtime classification; coordinator must own any future expansion.
