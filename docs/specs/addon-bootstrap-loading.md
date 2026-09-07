# Addon bootstrap loading

PTR 12.1.5 startup executes annotated files from eligible LoadOnDemand addons without completing their full load. The [live bootstrap capture](../addons/BootstrapOrderProbe_A/README.md) establishes ordering and repeated-load behavior for build 69594.

## What it must do

- [ ] Visit bootstrap-only addons in the normal dependency-ordered startup stream, not a global bootstrap pre-pass. Preserve enabled/profile/screen eligibility.
- [ ] Keep eager addon files in literal TOC order, including normal files before and after `[Bootstrap]` entries.
- [ ] Execute only annotated files during a LoD bootstrap operation. Expose `IsAddOnLoaded` as `true,false` during execution and `false,false` afterward.
- [ ] On the first subsequent full load, execute remaining files in TOC order without repeating completed bootstrap files; report `true,true` after completion.
- [ ] On repeated full loads, execute no files again. Preserve existing non-PTR startup selection.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md)
- [PTR capture and compatibility investigation](../wiki/investigations/ptr-pixel-rounding-probe.md)

## Implementation inventory

- `src/loader/addon.rs` — bootstrap file selection and completion separate from full loading.
- `src/loader/mod.rs` — public loading and ordered startup discovery.
- `src/bin/wow_sim/addon_loading.rs` — startup dispatch for full versus bootstrap-only loads.

## Tests asserting this spec

- `tests/load_order.rs` — bootstrap lifecycle and normal eager TOC ordering.
- Startup loader binary tests — actual scan/load boundary, ordering, and disabled-addon filtering.

## Known gaps (current cycle)

- [ ] Runtime startup and Spellbook interaction require fresh verification after loader integration.
- [ ] Private-table identity and SavedVariables visibility during bootstrap have not been live-probed; no parity claim is made for those boundaries.

## Out of scope

Changing vendor Lua, eagerly loading full LoD addons to hide missing globals, or reordering eager TOC entries. The probe does not establish new dependency rules or ADDON_LOADED delivery during bootstrap-only execution.
