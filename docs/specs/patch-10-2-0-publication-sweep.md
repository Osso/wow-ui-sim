# Patch 10.2.0 publication sweep

Account for Warcraft Wiki page 12983, revision 6473470 (2025-09-15T16:50:03Z). [Audit](../wiki/investigations/patch-10-2-0-api-audit.md) owns implementation evidence. Default retail carries 12.1.0, not reconstructed 10.2.0.

## What it must do

- [ ] Probe all 150 inventory occurrences against seventeen chronological later registers, 10.2.6 through 12.1.0; require exact reviewed gaps. Adding 10.2.5 is one list entry, not a new classifier.
- [ ] Account for every inventory/extract occurrence once. Publication/absence, default values and event registration never imply signatures, populated DTOs, security or native parity.
- [ ] Retail/PTR must not fabricate three unused C_Console members (`GetFontHeight`, `PrintAllMatchingCommands`, `SetFontHeight`) or register five retired legacy addon globals (`GetNumAddOns`, `IsAddOnLoaded`, `GetAddOnEnableState`, `IsAddOnLoadOnDemand`, `LoadAddOn`). Classic registrations, C_AddOns successors, GetAddOnMetadata and cached deprecation wrappers remain unchanged.
- [ ] ClearParentKey removes existing child mappings from both Lua parent fields and Rust widget state; repeated clears preserve mappings owned by another child. Existing SetParentKey clearOtherKeys behavior remains.
- [ ] Object methods use a Frame implementing that interface; removed movie scripts use MovieFrame:HasScript, never failed construction of a script-name frame.

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
