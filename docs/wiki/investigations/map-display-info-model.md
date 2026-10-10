# Map display info model

Verified 2026-10-10: supplied `hideIcons` model has bounded default 86-test GREEN; later exact P801 publication execution passes 1/1. PTR finalized independent audit admits 84 actual simulator tests PASS. Feature and parent acceptance remain open.

## Content

`SimState.map_display_hide_icons` stores per-map booleans, initially empty. `C_Map.GetMapDisplayInfo` returns one supplied boolean (including false), or zero values for absent/removed simulator input. Catalog presence supplies no input. Five model/security probes plus C_Map controls total 86 default PASS at producer `0979952073b64581903283c87d38ba2eae29f2b5`, epoch `20261010T194147Z`; PTR check is compile-only.

The earlier P801 failure was the stale known-gap entry, not a failing publication row. Saved native simulator test-binary epoch `20261010T195723Z`, revision `63f3a8a7373c8e52ffcc005bb443a9cf64b59fbf`, reaches the exact P801 selector: 1 PASS, exit 0, artifact unchanged; compile exit 0 and source equality true. Inventory is **269 rows / 253 OK / 16 gaps**; `wt-global-api-C_Map.GetMapDisplayInfo-30` is OK (`raw=function; lookup=function`). This supersedes only the current P801 pending receipt, not historical failures.

Frozen history remains **269 / 252 OK / 17 gaps**, validator PASS. Exact accepted fixture digest pair:

- Frozen 17: `bad5e7e4e77494f4e506684fb63b97d5b0d8ff9dd8279f714fbf00ad32bac55a`.
- Live 16: `618f9a1c663f16ce1df4613ff953e69b322e2cd04fe2868ad099189ab669e1b1`.

Only the MapDisplay ID was removed; no additions. Preservation accepts this exact byte-digest pair, not count-only equivalence. Frozen 8.0.1 source/evidence/receipts and prior later-gap closures were not rewritten. Retention contains sanitized reports/status, one P801 stdout, selected row, summary and SHA-256 manifest; no binaries or full inventory.

PTR exact epoch `20261010T200242Z`: initial33 PASS + continuation51 PASS = **84 actual PASS**, all five MapDisplay targets PASS; compile exit0/source equality true, unchanged artifacts, separate CLI `[]`/exit0 with CASC disabled. Original expected35/reached33 count error is preserved; two excluded party tests are Retail-only, not behavioral failures. [Sanitized receipts/correction/hashes](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/map-display-ptr-green/README.md) retain the bounded main observation. Final [independent audit](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/map-display-ptr-green/independent-report.md) admits bounded **84/84 PASS**, superseding the earlier incomplete inspection. Five other profiles' shared-state compile checks exit 0/source equality; current getter excluded, six dependency manifest deprecations each. Runtime acceptance remains separate. Authenticated native WoW semantics, malformed/unknown-input parity, return secrecy, dependency provenance and all-profile/whole-patch completion remain unproved.

## Sources

- [Map contract](../../specs/map-display-info.md) — default86 PASS, current P8011 PASS and independently audited PTR84 PASS.
- [Independent GREEN report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/map-display-publication-green/independent-green-report.md) — exact native epoch and historical preservation.
- [Preservation proof](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/map-display-publication-green/preservation-proof.json) — frozen/live digest pair.
- [Publication summary](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/map-display-publication-green/publication-summary.json) and [hashes](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/map-display-publication-green/hashes.json) — bounded retained receipts.

## See Also

- [[patch-8-0-1-api-audit]] — historical publication accounting.
- [[integrated-source-and-factory-proof-2026-10-09]] — bounded MapDisplay accounting and broader acceptance tracked separately.
