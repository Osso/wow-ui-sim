# Patch 7.3.0 publication sweep

Audit the pinned [page source](../../data/patch-api/sources/7.3.0-api-changes.wikitext), revision 5335723, without claiming full historical client parity. [Investigation](../wiki/investigations/patch-7-3-0-api-audit.md) records evidence and limitations.

## What it must do

- [ ] Probe all three named table additions through raw and ordinary lookup in a cached retail environment; require exact reviewed failure IDs after later-register supersession.
- [ ] Keep the 7.3.2 integration placeholder first, then 8.0.1 and all newer registers in chronological order.
- [ ] Account for every retained nonblank prose line, separately from table publication.
- [ ] Verify a concrete SOUNDKIT numeric ID reaches the existing sound-request state; reject the old string name without replacing the accepted request.
- [ ] Exercise the actual Table Inspector window with concrete root/child tables, navigation and close lifecycle, without vendor overrides.
- [ ] Preserve every earlier source and reproduce registers using recorded flags; inherited extract failures remain explicitly distinguished.
- [ ] Validate historical proof read-only from any checkout, with fixed historical register scope and file-derived counts.

## How it works

- [Audit investigation](../wiki/investigations/patch-7-3-0-api-audit.md).
- [Historical validator policy](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: opt-in bare table-summary parser.
- `tests/patch_7_3_0_publication_sweep.rs`: prefork publication observations.
- `tests/patch_7_3_0_cached_behavior.rs`: actual sound input/request-state boundary.
- `tests/data/patch_7_3_0_sweep_known_gaps.json`: reviewed exact gap IDs.
- `data/patch-api/evidence/7.3.0-session-2026-10-08/`: source, scans and proof receipts.

## Tests asserting this spec

- `tools/test_gen_patch_wikitext_register.py::InventoryTests.test_legacy_summary_tables_keep_names_not_prose_or_addons`.
- `tests/patch_7_3_0_publication_sweep.rs`.
- `tests/patch_7_3_0_cached_behavior.rs`.

## Known gaps (current cycle)

- [ ] Discovery and retained-gap review pending.
- [ ] Blizzard_Console load/interaction and full Table Inspector behavior require independent lifecycle proof.
- [ ] Historical SOUNDKIT catalog/name similarity and full artifact-forge behavior are not implied by table existence.

## Out of scope

Native audio fidelity, unspecified namespace members, full historical client emulation, vendor changes, shims, full integration suite, push and merge.
