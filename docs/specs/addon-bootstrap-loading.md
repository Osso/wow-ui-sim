# Addon bootstrap loading

Retail 12.1 and cumulative PTR startup execute annotated files from eligible LoadOnDemand addons without completing their full load. The [live bootstrap capture](../addons/BootstrapOrderProbe_A/README.md) establishes ordering and repeated-load behavior for build 69594.

## What it must do

- [ ] Visit eligible retail 12.1 and PTR bootstrap-only addons in the normal dependency-ordered startup stream, not a global bootstrap pre-pass. Preserve enabled/profile/screen eligibility; the retail gate extension awaits GREEN verification.
- [x] Keep eager addon files in literal TOC order, including normal files before and after `[Bootstrap]` entries.
- [ ] Strip every trailing inline annotation from file paths across spaces, tabs, and mixed whitespace, regardless of annotation order. Preserve `[Game]`/`[Family]` path substitutions, game-type inclusion/exclusion, and per-file environment flags. The native Forever TargetFrame load reproduces malformed filenames before the fix; parser and loader GREEN remain pending.
- [x] Execute only annotated files during a LoD bootstrap operation. Expose `IsAddOnLoaded` as `true,false` during execution and `false,false` afterward.
- [x] On the first subsequent full load, execute remaining files in TOC order without repeating completed bootstrap files; report `true,true` after completion.
- [x] On repeated full loads, execute no files again. Classic startup selection remains outside the retail epoch gate.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md)
- [PTR capture and compatibility investigation](../wiki/investigations/ptr-pixel-rounding-probe.md)

## Implementation inventory

- `src/loader/addon.rs` — bootstrap file selection and completion separate from full loading.
- `src/loader/mod.rs`, `startup_addons.rs` — `StartupAddon` records and `retail-12-1-0`-gated `Full`/`BootstrapOnly` ordered discovery.
- `src/bin/wow_sim/addon_loading.rs` — startup dispatch, timing, and `ADDON_LOADED` delivery only for full loads.

## Tests asserting this spec

- `src/toc/tests.rs` — native TargetFrame multiannotation lines, reordered annotation combinations, unchanged file order/environment flags, and game-type exclusions.
- `tests/secureenv_isolation.rs::toc_multiannotations_load_files_in_order_with_secure_environment` — actual annotated file loading, ordered execution, and secure/public environment isolation.

- `tests/load_order.rs` — bootstrap lifecycle through runtime `C_AddOns.LoadAddOn`, eager TOC ordering, and exact per-profile discovery snapshots. Retail's 219-addon order excludes the Classic-only `Blizzard_FrameXML` dependencies on UnitPopup and MirrorTimer; their transitive prerequisites consequently move later. PTR's 211-addon fixture is pinned separately.
- Startup loader binary tests — actual scan/load boundary, ordering, and disabled-addon filtering under retail 12.1 and PTR.
- `tests/micro_menu.rs::micro_menu_ej_button_loads_and_opens_panel` — process-level startup and actual Encounter Journal OnClick twice, with no injected bootstrap helper, completed LoD state, open/close assertions, and empty Lua-error JSON.

## Known gaps (current cycle)

- [ ] Private-table identity and SavedVariables visibility during bootstrap have not been live-probed; no parity claim is made for those boundaries.

## Out of scope

Changing vendor Lua, eagerly loading full LoD addons to hide missing globals, or reordering eager TOC entries. The probe does not establish new dependency rules or ADDON_LOADED delivery during bootstrap-only execution.
