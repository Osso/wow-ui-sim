# Patch 10.2.0 publication sweep

Account for Warcraft Wiki page 12983, revision 6473470 (2025-09-15T16:50:03Z). [Audit](../wiki/investigations/patch-10-2-0-api-audit.md) owns implementation evidence. Default retail carries 12.1.0, not reconstructed 10.2.0.

## What it must do

- [x] Probe all 150 inventory occurrences against seventeen chronological later registers, 10.2.6 through 12.1.0; require exact reviewed gaps. Adding 10.2.5 is one list entry, not a new classifier.
- [x] Account for every inventory/extract occurrence once. Publication/absence, default values and event registration never imply signatures, populated DTOs, security or native parity.
- [x] Retail/PTR must not fabricate three unused C_Console members (`GetFontHeight`, `PrintAllMatchingCommands`, `SetFontHeight`) or register five retired legacy addon globals (`GetNumAddOns`, `IsAddOnLoaded`, `GetAddOnEnableState`, `IsAddOnLoadOnDemand`, `LoadAddOn`). Classic registrations, C_AddOns successors, GetAddOnMetadata and cached deprecation wrappers remain unchanged.
- [x] ClearParentKey removes existing child mappings from both Lua parent fields and Rust widget state; repeated clears preserve mappings owned by another child. Existing SetParentKey clearOtherKeys behavior remains.
- [x] Object methods use a Frame implementing that interface; removed movie scripts use MovieFrame:HasScript, never failed construction of a script-name frame.

## How it works

- [Page audit](../wiki/investigations/patch-10-2-0-api-audit.md) — source accounting, bounded fixes and retained contracts.
- [Client profiles](../wiki/systems/client-profiles.md) — retail/classic boundaries.

## Implementation inventory

- `src/c_api/patch_retired_members.rs` — retail-only namespace absence markers.
- `src/c_api/c_addons.rs` — profile-scoped legacy global registration.
- `src/lua_api/frame/methods/button_anchor_hierarchy/` — modeled parent-key clearing.
- `tools/gen_patch_wikitext_register.py`, `tools/extract_patch_non_inventory.py` — retained widget-script metadata, category labels, XML example and reference-list extraction.
- `tests/common/publication_sweep.rs` — unchanged general publication policy plus correct interface/script factories.

## Tests asserting this spec

- `tests/patch_10_2_0_publication_sweep.rs` — exact cached Game publication fixture.
- `tests/patch_10_2_0_publication_fixes.rs` — parent-key state, repeated retirement lookup and classifier proof.
- `tests/patch_10_2_0_cached_surfaces.rs` — same boundary after full cached Game loading.
- `tools/test_gen_patch_wikitext_register.py`, `tools/test_extract_patch_non_inventory.py` — parser/extractor behavior fixtures.

## Known gaps (current cycle)

- [ ] Exact retained publication gaps and per-ID reasons live in `data/patch-api/evidence/10.2.0-session-2026-10-07/p1020-gap-review.json`.
- [ ] Substantive extract contracts remain pending in the exhaustive extract scout; bounded parent-key proof does not establish texture slicing/rendering or migration parity.
- [ ] Five float CVar defaults are numerically equal but differ in serialized precision. Classifier records literal mismatch separately; no historical default-string credit.

## Out of scope

Historical epochs, native parity, linked-page expansion, new placeholders, vendor/Blizzard cache changes, 3D rendering, full suites, push, merge and agent/model review.

## Local proof

Runtime/test revision `06326ad3e`. [Proof ledger](../../data/patch-api/evidence/10.2.0-session-2026-10-07/p1020-proof.json) binds commands, revisions and output hashes. Eight retirements and parent-key state have RED/GREEN proof; three behavior/probe tests, cached Game prefork, four existing SetParentKey cases and one cached groupfinder case pass at their recorded scopes. Negative control produces exactly one new failure, 30 → 31, expected exit 101. Twenty parser/extractor fixtures and eighteen byte-identical register regenerations pass. Formatting, Mists tests check (zero non-vendor warnings), retail build and exit-0 startup `[]` pass. Only existing iced vendor manifest warnings remain. Local targeted proof is not native or independent acceptance.

| Patch | Rows | OK | Gaps | Isolated exit |
|---|---:|---:|---:|---:|
| 10.2.0 | 150 | 120 | 30 | 0 |
| 10.2.6 | 220 | 200 | 20 | 0 |
| 10.2.7 | 104 | 68 | 36 | 0 |
| 11.0.0 | 495 | 329 | 166 | 0 |
| 11.0.2 | 34 | 22 | 12 | 0 |
| 11.0.5 | 48 | 38 | 10 | 0 |
| 11.0.7 | 98 | 70 | 28 | 0 |
| 11.1.0 | 116 | 97 | 19 | 0 |
| 11.1.5 | 125 | 89 | 36 | 0 |
| 11.1.7 | 48 | 40 | 8 | 0 |
| 11.2.0 | 162 | 135 | 27 | 0 |
| 11.2.5 | 163 | 118 | 45 | 0 |
| 11.2.7 | 508 | 414 | 94 | 0 |
| 12.0.0 | 1010 | 989 | 21 | 0 |
| 12.0.1 | 225 | 222 | 3 | 0 |
| 12.0.5 | 363 | 352 | 11 | 0 |
| 12.0.7 | 174 | 171 | 3 | 0 |
| 12.1.0 | 778 | 773 | 5 | 0 |
