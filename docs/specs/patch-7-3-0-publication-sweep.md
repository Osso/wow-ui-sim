# Patch 7.3.0 publication sweep

Audit the pinned [page source](../../data/patch-api/sources/7.3.0-api-changes.wikitext), revision 5335723, without claiming full historical client parity. [Investigation](../wiki/investigations/patch-7-3-0-api-audit.md) records evidence and limitations.

## What it must do

- [x] Probe all three named table additions through raw and ordinary lookup in a cached retail environment; require exact reviewed failure IDs after later-register supersession.
- [x] Keep the 7.3.2 integration placeholder first, then 8.0.1 and all newer registers in chronological order.
- [x] Account for every retained nonblank prose line, separately from table publication.
- [x] Both `C_Sound.PlaySound` and the legacy global must use the same simulator request model; cached Blizzard aliasing must retain numeric validation and recording.
- [x] Verify a concrete SOUNDKIT numeric ID reaches the existing sound-request state; reject the old string name without replacing the accepted request.
- [x] Exercise the actual Table Inspector window with concrete root/child tables, navigation and close lifecycle, without vendor overrides.
- [x] Preserve every earlier source and reproduce registers using recorded flags; inherited extract failures remain explicitly distinguished.
- [x] Validate historical proof read-only from any checkout, with fixed historical register scope and file-derived counts.

## How it works

- [Audit investigation](../wiki/investigations/patch-7-3-0-api-audit.md).
- [Historical validator policy](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: opt-in bare table-summary parser.
- `tests/patch_7_3_0_publication_sweep.rs`: prefork publication observations.
- `src/c_api/c_sound.rs`: shared numeric sound-kit request model; existing options retain their feature gate.
- `tests/patch_7_3_0_sound_model.rs`: bare namespace/global request and rejection behavior.
- `tests/patch_7_3_0_cached_behavior.rs`: actual sound input/request-state boundary.
- `tests/data/patch_7_3_0_sweep_known_gaps.json`: reviewed exact gap IDs.
- `data/patch-api/evidence/7.3.0-session-2026-10-08/`: source, scans and proof receipts.

## Tests asserting this spec

- `tools/test_gen_patch_wikitext_register.py::InventoryTests.test_legacy_summary_tables_keep_names_not_prose_or_addons`.
- `tests/patch_7_3_0_publication_sweep.rs`.
- `tests/patch_7_3_0_cached_behavior.rs`.
- `tests/patch_7_3_0_sound_model.rs`.
- `data/patch-api/evidence/7.3.0-session-2026-10-08/validate.py`.

## Known gaps (current cycle)

- [x] Discovery and retained-gap review complete: zero inventory failures; one pending prose row.
- [ ] Exact `/tinspect` slash dispatch/consent and complete Table Inspector behavior are not inferred from the direct window lifecycle test.
- [ ] Historical SOUNDKIT catalog/name similarity and full artifact-forge behavior are not implied by table existence.

## Out of scope

Native audio fidelity, unspecified namespace members, full historical client emulation, vendor changes, shims, full integration suite, push and merge.
