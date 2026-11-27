# UI widget ButtonHeader supplied records

`C_UIWidgetManager.GetButtonHeaderWidgetVisualizationInfo(widgetID)` must expose complete supplied records as independent Lua snapshots. Inputs live in `src/c_api/ui_widget_manager.rs`; the C API query is registered for Retail, PTR and Forever. Bounded Retail execution is recorded below. See [C API architecture](../lua-api.md) and [cache provenance](../updating-blizzard-ui-to-a-new-patch.md).

## What it must do

- [x] Publish `GetButtonHeaderWidgetVisualizationInfo` on WoW Forever too: its pinned UIWidgetManagerDocumentation.lua function and structure are byte-identical to Retail's. `tests/ui_widget_forever_records.rs` feeds two record rounds to the Forever `UIWidgetTopCenterContainerFrame` and reads the frames that its template `Setup` produces.
- [ ] Keep supplied records environment-local. Empty inputs contain no records; never fabricate default records or derive a catalog from ID thresholds.
- [ ] Return nil for unsupplied numeric IDs and select exact supplied keys without integer narrowing. Numeric-key storage is simulator policy, not a native acceptance-range claim.
- [ ] Preserve all 17 required header fields and five required nested button fields from cached `UIWidgetManagerDocumentation.lua`, including false, enum zeroes, empty strings, empty arrays and supplied button order. Records represent strings, numbers, booleans and enum values literally; only the complete result is nullable.
- [ ] Return fresh outer, button-array and nested button tables. Lua mutation cannot change supplied state, other widgets, retained snapshots or later results. Live environments remain independent.
- [ ] Apply documented `AllowedWhenUntainted` to original secret arguments before numeric extraction/lookup. Untainted secret supplied/absent IDs may query; tainted secret IDs fail. Ordinary numeric IDs remain usable under taint. Preserve wrappers and caller context.
- [ ] Exercise the actual registered cached ButtonHeader provider: reject hidden/unknown records and preserve shown/empty records. Raw query must still return supplied hidden records. Never replace the query or copy provider logic.

Snapshot, absence and isolation requirements are simulator policy, not native-captured semantics.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md).
- [Addon loading](../addon-loading-pipeline.md).

## Implementation inventory

- `src/c_api/ui_widget_manager.rs`, `src/c_api/mod.rs`: public complete records and numeric-key inputs.
- `src/c_api/ui_widget_manager/button_header.rs`, `src/lua_api/globals/missing_surface.rs`: authenticated query and fresh complete snapshot projection, registered for Retail/PTR.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: environment field and empty initialization.

## Tests asserting this spec

`tests/c_ui_widget_manager_button_header.rs`: nine behavioral cases selected for Retail and PTR; secret cases additionally gated by `retail-12-1-0`. Both cached metadata declarations match exactly for the query, all 17 header fields and all five nested button fields. Cached header Lua/XML source also matches; the bounded consumer resolves the active profile cache, not a hardcoded Retail path. This supports shared test selection, not historical availability, compilation or runtime parity. At `ff72f3c3b`, nine selected cases produce one unsupplied-ID PASS and eight behavioral failures on supplied-result type/snapshot behavior, including the untainted secret result. No observed fixture failures; cached provider dependencies and later tainted-secret assertions remain unreached in that RED where the query fails. Executed scoped Retail producer GREEN: 9/9 PASS at `6b522c595`, including actual cached-provider filtering and VM-secret policy; [investigation SSOT](../wiki/investigations/integrated-source-and-factory-proof-2026-10-09.md#executed-staged-queries-and-new-producers).

## Known gaps (current cycle)

- [x] Compile and obtain genuine query-boundary RED before implementing the producer.
- [x] Implement the query after genuine RED.
- [x] Verify bounded supplied projection, snapshots, isolation and secret policy.
- [x] Execute actual cached manager/base/header dependencies in the bounded provider fixture; setup failures are not query RED or full-provider acceptance.
- [ ] Startup/default-check/PTR acceptance remains open; startup still fails with 12 SMOOTH setup errors, not clean.

## Out of scope

Native widget feed/catalog, invented numeric ranges, Classic/other-profile or native parity, unrelated quest paths, full widget Setup/pools/anchors/atlases/timers/rendering. Provider filtering is not rendered visibility. Parent startup/profile acceptance remains separate and open.
