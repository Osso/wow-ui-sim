# Death recap events

Retail `C_DeathRecap.GetRecapEvents` publishes host-supplied recap events to the original Blizzard consumer and its deprecated alias.

## What it must do

- [ ] A missing or nil ID selects the newest host-marked eligible recap among the last 60. An explicit ID selects a matching recap, including an ineligible one.
- [ ] A missing recap returns zero values. A found recap returns one array with at most ten events.
- [ ] Each event exposes supplied `currentHP` and, when supplied, `amount`, `timestamp` and `sourceGUID`. No missing event field is invented.
- [ ] Ordinary nonnumeric values select ID zero; numeric inputs convert to signed integers and overflow reports an error.

## Has recap events

- [ ] Retail `C_DeathRecap.HasRecapEvents([recapID])` returns one boolean. Omitted or nil ID selects the newest eligible retained recap; an explicit ID selects a matching retained recap regardless of eligibility. It tests record presence, not event count.
- [ ] Signed 32-bit numeric conversion accepts ordinary numeric strings and truncates toward zero after range checking. Invalid types, nonnumeric strings and out-of-range numbers raise argument errors. Secret arguments obey VM access restrictions.
- [ ] `DeathRecap_HasEvents` is the loaded Blizzard deprecated alias of this function.

## Recap link

- [ ] Retail `C_DeathRecap.GetRecapLink([recapID])` uses the same strict optional ID conversion and retained-record selection as `HasRecapEvents`.
- [ ] A selected record returns one localized `DEATH_RECAP_LINK` string formatted with its stored ID. A missing record returns one `DEATH_RECAP_TEXT` string.
- [ ] `GetDeathRecapLink` is the loaded Blizzard deprecated alias of this function.

## Best-guess basis

Retail build 70688 static evidence identifies lookup, eligibility and ten-event limit. The connection from lookup to Lua return is emulated rather than observed in a running client. `DeathRecapDocumentation.lua:11–22` declares an optional ID and event table; `Deprecated_CombatLog.lua:28` publishes the alias. `Blizzard_DeathRecap.lua:317–340` consumes the event array.

For `HasRecapEvents`, `DeathRecapDocumentation.lua:56` declares the optional number and boolean result; `Deprecated_CombatLog.lua:29` publishes the alias. The WM23-hasrecap handoff verifies local validation and emulates the lookup-to-publication connection. The simulator's last-60 vector ordering approximates the native circular store.

For `GetRecapLink`, `DeathRecapDocumentation.lua:26` declares the optional number and string; `Deprecated_CombatLog.lua:30` publishes the alias. The WM23-rules handoff verifies local template selection and emulates publication. The Retail generated global strings supply both localization keys; the simulator formats the selected template with Lua `string.format`.

## How it works

- [Death recap current state](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/c_api/c_death_recap.rs` registers and reads the Retail API.
- `src/lua_api/state_types/mythic_plus_scenario.rs` holds host-supplied recap records.

## Tests asserting this spec

- `tests/publication_deprecated_alias.rs` checks two supplied states through the loaded Blizzard events alias.
- `tests/death_recap_has_events.rs` checks supplied recap presence and argument handling through the loaded Blizzard presence alias.
- `tests/death_recap_link.rs` checks localized output and argument handling through the loaded Blizzard link alias.

## Known gaps (current cycle)

- [ ] Live return behavior, complete event schema, eligibility transitions, circular cursor and secret-value handling remain unverified.
- [ ] Native protected-call wrapper behavior for `GetRecapLink`, localization formatting beyond the supplied template, and addon mutation of formatting globals remain unverified.
- [ ] Other event fields and host input paths are not modeled by this contract.

## Out of scope

Runtime combat collection and recap persistence are not modeled here.
