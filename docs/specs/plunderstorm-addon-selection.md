# Plunderstorm addon selection

Retail selects standard game type for addon loading. The pinned retail `Blizzard_EndOfMatchUI/Blizzard_EndOfMatchUI.toc:4` and `Blizzard_PlunderstormPrematchUI/Blizzard_PlunderstormPrematchUI.toc:4` restrict these addons to `plunderstorm`.

## What it must do

- [x] Exclude these addons from standard Retail Game-screen discovery.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md)

## Implementation inventory

- `src/toc/mod.rs`: profile game-type selection.
- `crates/sim-toc/src/lib.rs`: TOC game-type filtering.

## Tests asserting this spec

- `tests/blizzard_end_of_match_ui_loads.rs`: standard Retail discovery exclusion.
- `tests/blizzard_plunderstorm_prematch_ui_loads.rs`: standard Retail discovery exclusion.

## Known gaps (current cycle)

- [ ] The loader has no selectable Plunderstorm game type. The EndOfMatchUI standard-Retail force-load behavior tests were removed because `Blizzard_EndOfMatchUI.toc:4` selects Plunderstorm only. Restore behavioral coverage under a real Plunderstorm selection.
- [ ] The PlunderstormPrematchUI standard-Retail force-load behavior tests were removed because `Blizzard_PlunderstormPrematchUI.toc:4` selects Plunderstorm only. Restore behavioral coverage under a real Plunderstorm selection.

Removed EndOfMatchUI cases, all prefixed `blizzard_end_of_match_ui_`:

- `loads_explicitly_without_errors`
- `creates_named_singleton_frame`
- `singleton_starts_hidden_with_top_level_input`
- `publishes_three_mixins`
- `singleton_carries_three_frame_pools`
- `is_addon_loaded_flips_after_explicit_load`
- `singleton_is_parented_to_ui_parent`
- `singleton_registers_for_show_end_of_match_event`
- `singleton_resolves_xml_named_children`

Removed PlunderstormPrematchUI cases, all prefixed `blizzard_plunderstorm_prematch_ui_`: `is_addon_loaded_after_explicit_load`, `publishes_eight_mixins`, `creates_named_non_virtual_frame`, `does_not_leak_virtual_templates_to_globals`, and `loads_without_addon_specific_lua_errors`. Their non-retail wrappers were also removed because no profile selects Plunderstorm.

## Out of scope

- Implementing Plunderstorm game-type selection and the addons' missing APIs.
