# Item-level color

The Retail `GetItemLevelColor` global supplies RGB values to the original Paper Doll consumer at `Blizzard_UIPanels_Game/Mainline/PaperDollFrame.lua:1907`. The contract follows R88l.1 in `pr46-49-decomp-answers.md`; color-provider state is described in [C_ColorOverrides](../wiki/systems/color-overrides.md).

## What it must do

- [x] VERIFIED (R88l.1, Retail 12.1.5.70688): `GetItemLevelColor` ignores its arguments and returns three numeric channels in red, green, blue order. It normalizes each selected channel through single-precision arithmetic before publication.
- [x] VERIFIED (R88l.1, Retail 12.1.5.70688): select `ITEM_EPIC_COLOR` when condition set 24 passes; otherwise select `ITEM_SUPERIOR_COLOR` when condition set 23 passes; otherwise select `ITEM_STANDARD_COLOR`. Condition 24 has priority over condition 23.
- [x] VERIFIED (R88l.1, Retail 12.1.5.70688): resolve an active color-provider entry before the selected named color. Return magenta when neither yields RGB channels.
- [x] VERIFIED (simulator behavior): absent condition-result host inputs select no override. This neutral input does not prove the native default condition result.

## How it works

- [C_ColorOverrides state and GlobalColor data](../wiki/systems/color-overrides.md)
- [GlobalColor catalog](../wiki/systems/global-color-catalog.md)

## Implementation inventory

- `src/lua_api/globals/real/item_legacy.rs` — registers and publishes the Retail legacy global.
- `src/c_api/c_color_overrides.rs` — resolves explicit quality-color override records.
- `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs` — hold empty-default condition-result inputs.
- `src/lua_api/workarounds/temporary/inert_global_defaults.rs` — no longer supplies a guessed white result.

## Tests asserting this spec

- `tests/blizzard_ui_unit.rs` — calls the original `PaperDollFrame_UpdateStats` and checks the displayed item-level color for absent, condition-23, condition-24, provider, and unresolved-color inputs.

## Known gaps (current cycle)

- [ ] UNKNOWN (R88l.1, Retail 12.1.5.70688): native condition-set records, their live values, and the subject accessor are not modeled. The condition results remain explicit host inputs.

## Out of scope

- UNKNOWN for non-Retail behavior; the evidence and selected consumer cover Retail only.
- Evaluating the underlying player-condition records.
