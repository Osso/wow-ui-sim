# Edit Mode layout selection

Current Retail/PTR startup must preserve the active-layout index supplied by the character cache in Blizzard's combined preset-plus-custom selection space. The unchanged Blizzard manager prepends its source-selected presets without offsetting `activeLayout`: Modern/Classic on pinned Retail, Modern/Classic/Gamepad on pinned PTR. In the observed cache, index2 therefore selects Classic, not custom Widescreen. User WTF data must not be changed or orientation forced to hide a selection error.

## What it must do

- [x] Import provided account/character cache files before whole unchanged Game UI startup; preserve the character's active index through preset insertion. Native cache index2 selects Classic with horizontal Micro Menu geometry, even when the second custom layout is vertical Widescreen.
- [x] Preserve saved layout content, names/identifiers and existing named environment/snapshot override selection and precedence. Resolve an explicit saved-layout name to the combined selection space; do not reinterpret native character indices as custom-list positions.
- [x] Derive combined-list test expectations from the selected original preset manager, not a Retail-only count. PTR `Blizzard_EditMode/Mainline/EditModePresetLayouts.lua:1267–1285` supplies Gamepad; `Shared/EditModePresetLayoutsManager.lua:5–32` inserts all three maps; `Shared/EditModeManager.lua:1018–1020` prepends that list to saved layouts. The pinned Retail manager inserts only Modern/Classic at lines5–21.
- [x] When the account cache declares more layouts than its token stream contains, do not fabricate layouts after exhaustion; emit a warning. This is a simulator scenario decision: native handling of the mismatch, and whether the client writes such a file, are unknown. Handling of partially started records is outside this contract.
- [x] Keep selection, cache decoding and original consumer behavior distinct. No original WTF writes, vendor patches, forced orientation or guessed new numeric-override contract.

## How it works

- [Edit Mode investigation](../wiki/investigations/editmode-layout.md)
- [Anchor replay contract](edit-mode-initial-anchors.md)

## Implementation inventory

- `src/saved_variables.rs`: existing file import into the modeled cache API.
- `src/lua_api/workarounds/temporary/edit_mode_cache_defaults.rs`: existing Lua-owned cache state, active selection and explicit override resolution.
- `src/lua_api/workarounds/editmode/setup_layout_info.lua`: manager initialization preserves the active index while prepending presets; explicit override conversion belongs to cache import. Bounded Retail and PTR GREEN below; native/pixel parity remains separate.

## Tests asserting this spec

- [edit_mode_cache_startup.rs](../../tests/edit_mode_cache_startup.rs): truncated account cache emits a warning without phantom layouts and preserves PlayerFrame anchors; two distinct complete layouts remain intact through the registered specialization event.
- [edit_mode_cache_startup.rs](../../tests/edit_mode_cache_startup.rs): fresh environment, isolated real-format file inputs, import before whole Game loading, original startup, selected Classic and public Micro Menu geometry. Native selection genuinely failed at `653e829f1`. At `7d9d9f621`, Retail native Classic/horizontal and four explicit Widescreen/vertical variants completed in two test names; parent/child recaps count once.
- [edit_mode_api/cache.rs](../../tests/edit_mode_api/cache.rs): 10/10 Retail cache/parser/env/snapshot controls PASS at `7d9d9f621`; low-level cache proof is not whole consumer proof.

## Current broad-suite status — 2026-10-12

[Source-suite terminal state and fixture repair](../wiki/investigations/editmode-layout.md#test-only-fixture-repair--pending-verification-2026-10-12): main independently read source `41cbf5005` terminal JSON, integration exit0/prefork exit0/lib exit100. Broad suite FAIL: `setup_layout_info_remaps_active_saved_layout_after_prepending_presets` and `setup_layout_info_preserves_distinct_saved_action_bar_profile_settings`; [retained terminal packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/fullsuite-41cb-2026-10-12/README.md) records 15,123 PASS / 2 FAIL / 19 reported skips, independently reconciled without replay.

Test-only `b41b42fc1` sets active4 for the second custom layout, active3 for the first custom layout (not layoutID9), and retains native active2/Classic control. Production combined-index behavior is unchanged. Gate3095 found cached manager state invalidated the new preset control; follow-up `ac170ecbf` resets it. At that historical checkpoint runtime remained resource-BLOCKED, zero tests invoked; no repaired-test PASS was credited then. Earlier bounded Retail/PTR and native-user evidence below retains its scope, not broad-suite acceptance.

## Focused fixture execution — 2026-10-15

At source `fb4b31b8f7b97bc460f15c0554fb39b91e8c6e2a`, Retail GUI-feature library cases `setup_layout_info_preserves_active_combined_index_after_prepending_presets` and `setup_layout_info_preserves_distinct_saved_action_bar_profile_settings` each passed 1/1, 0 FAIL, exit0 (focused total2 PASS). Repairs `b41b42fc1`/`ac170ecbf` now have bounded fixture execution: active4 selects Saved Wide; active2 selects Classic after manager-cache reset; active3 selects Ultrawide despite saved layoutID9, preserving distinct action-bar settings.

Private evidence uses the [shared library epoch/seal](slider-bound-thumb-layout.md#tests-asserting-this-spec): `focused-runtime-results.json` and full `editmode-combined-index/stdout`/`stderr`, `editmode-distinct-settings/stdout`/`stderr` were read. Independent saved-runtime reconciliation **PASS** verifies both actual names/counts, full streams, shared artifact and repaired fixture assertions. Historical source41cb broad FAIL remains historical FAIL; these two passes are not a full-suite rerun, fresh whole-startup, PTR or native/pixel acceptance. Existing source701 default check remains separate and was not rerun.

## Known gaps (current cycle)

- [x] Genuine fresh-startup RED at `653e829f1`: selected-layout assertion failed after real file import/full startup; geometry unreached. Enter3 GREEN and separate strict publication stale-row FAIL do not establish selection GREEN.
- [x] Corrected source and bounded Retail startup2/cache10 GREEN; corrected normal startup Classic329×40/zero Lua errors, fmt/default check PASS. [Packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/edit-mode-enter-2026-10-12/README.md). Later PTR startup2/cache10 PASS at `f3145424c`, normal zero Lua errors and PTR fmt/check PASS ([packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/edit-mode-enter-ptr-2026-10-12/README.md)).
- [ ] Current-artifact owned GUI query observed Classic/horizontal329×40 at native1920×1080 window; user subsequently accepted layout. Keyboard initially failed until native window focus was assigned; user then confirmed chat and `/command` worked. Native focus runtime configuration—not new Rust keyboard logic after `7d9d9f621`—corrected delivery. Fresh startup via private `configure-window.py` and independent3045 ownership/socket/focus/geometry/wiring/lifetime gate PASS ([packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/native-window-focus-2026-10-12/README.md)); test app stopped. Universal native/PTR-GUI/pixel parity remains unproved; configured90s expiry was replaced by user-authorized stop-after-test lifecycle. Universal pixel/all-key/all-profile acceptance and whole handoff remain OPEN. Native-focus packet retention/link/privacy gate PASS at `3fce068f0`; earlier Edit Mode packet gate PASS through `b1c7c4348`; [gate](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/edit-mode-enter-2026-10-12/public-retention-gate.json).

## Out of scope

Rewriting user layouts, native pixel/account parity, new invalid-index policies, new numeric configuration semantics and unrelated anchor/renderer redesign.
