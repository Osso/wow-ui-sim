# Retail combo points

Retail `GetComboPoints(unit, target)` reads the existing player-owned combo-power pool. Unlike [Forever](forever-combo-points.md), changing the target does not reassign or hide the player's count. The modeled query lives in `src/lua_api/globals/real/combo_points.rs`.

## What it must do

- [ ] Register on retail without widening other client profiles or changing Forever's target-bound policy.
- [ ] Require both string unit-token arguments, return one number, and read `secondary_powers[Enum.PowerType.ComboPoints]` for the modeled player owner.
- [ ] Preserve count across target selection changes and no target; resolve player aliases by GUID. An unknown owner returns zero; resolved nonplayer ownership remains explicitly unsupported.
- [ ] Reflect nonzero then zero admin combo-power inputs independently of energy updates.
- [ ] Load unchanged retail Blizzard ComboFrame with `comboPointLocation=1` and update it through `PLAYER_ENTERING_WORLD` for nonzero display and zero hiding without Lua errors.

## How it works

- [Forever ownership investigation](../wiki/investigations/forever-combo-points.md) — contrasting legacy ownership policy.
- [Existing power inputs](../admin-api/health-power.md).

## Implementation inventory

- `src/lua_api/globals/real/combo_points.rs` — shared player pool read with Forever-only target association.
- `src/lua_api/globals/real/mod.rs`, `src/lua_api/globals/register.rs` — retail/Forever availability.

## Tests asserting this spec

- `tests/retail_unit_queries.rs` — retail API and unchanged Blizzard ComboFrame event lifecycle; GREEN pending.
- `tests/combo_points.rs`, `tests/click_targeting/forever_regressions.rs` — existing Forever target-bound controls.

## Known gaps (current cycle)

- [ ] Complete focused retail RED/GREEN and existing Forever controls. Retail default addon replay already reproduces missing calls at ExwindCore:1650 and ComboFrame:65.

## Out of scope

- PvP-secret values, nonplayer-owned pools, combat generation, native-client conformance, vendor edits, Era/Mists compatibility changes, and broad addon acceptance.
