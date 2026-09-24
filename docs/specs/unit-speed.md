# Forever unit speed

`GetUnitSpeed(unit)` supplies the player-speed read used by cached Forever UI documentation and Camelot’s character panel. Source: `src/lua_api/globals/real/unit_speed.rs`. Native Forever probes are unavailable; see [[forever-character-panel]] for the triggering cached-binary investigation.

## What it must do

### Availability and identity

- [ ] Register `GetUnitSpeed(unit)` only for the Forever profile; other profiles retain their existing global surface.
- [ ] Resolve `player`, `self`, and aliases resolving to the player GUID from the existing unit snapshots.
- [ ] Do not give a different target, pet, party member, or unknown token the player’s speed.

### Returned values

- [ ] Return four numeric values: current, run, flight, and swim.
- [ ] Use configured player capabilities. Default run `7`, flight `7`, and swim `4.722222` yards/s are simulator assumptions inherited from Mists, not native-verified values.
- [ ] Return current speed `0` while stationary; otherwise prefer swimming, then flying, then running.
- [ ] Return `0, 0, 0, 0` for resolved nonplayer and unknown tokens. This is inferred simulator policy, not documented native behavior.
- [ ] Missing or non-string arguments retain existing string-conversion failure behavior.
- [ ] Retain the query and modeled state after post-cleanup global restoration.

## How it works

- → [[forever-character-panel]] — cached-binary failure and remaining panel boundary.

## Implementation inventory

- `src/lua_api/globals/real/unit_speed.rs` — Forever global and player-speed selection.
- `src/lua_api/globals/real/mod.rs` — Forever module declaration.
- `src/lua_api/globals/register.rs` — Forever global registration.
- `src/lua_api/state_types/character_world.rs` — player speed capabilities and inferred defaults.

## Tests asserting this spec

- `tests/wowforever_unit_interactions.rs` — unit identity, values, precedence, invalid tokens, and post-cleanup restoration; not yet verified in this cycle.
- `tests/wowforever_character_panel.rs` — open/close character-panel consumer regression; not yet verified in this cycle.

## Known gaps (current cycle)

- [ ] Run the focused unit-speed and character-panel regression tests; no GREEN result is recorded yet.
- [ ] Native Forever speed semantics, nonplayer values, and default capabilities remain unknown.

## Out of scope

- Persistence, other-profile behavior, and complete character-panel replay are not established by this player-speed slice.
