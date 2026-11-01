# Plunderstorm addon selection

Retail selects standard game type for addon loading. The pinned retail `Blizzard_EndOfMatchUI/Blizzard_EndOfMatchUI.toc:4` and `Blizzard_PlunderstormPrematchUI/Blizzard_PlunderstormPrematchUI.toc:4` restrict these addons to `plunderstorm`.

## What it must do

- [ ] Exclude these addons from standard Retail Game-screen discovery.

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

## Out of scope

- Implementing Plunderstorm game-type selection and the addons' missing APIs.
