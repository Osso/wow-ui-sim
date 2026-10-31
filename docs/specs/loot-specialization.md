# Loot specialization

The loot-specialization globals use per-simulator player state. Native validity, persistence and event behavior remain unknown.

## What it must do

- [x] Return 0 initially, selecting the current specialization.
- [ ] Accept 0 or a specialization belonging to the player's class; reject other numeric values without changing state. The Retail test covers one class specialization, 0, one foreign ID and one unknown ID.
- [ ] Fire `PLAYER_LOOT_SPEC_UPDATED` synchronously with no payload on a change, after storing the new value; do not fire for an unchanged or invalid selection. The Retail test covers one change and unchanged, foreign and unknown IDs.
- [x] Reflect a non-default choice and a return to default in the original player popup radio buttons.
- [ ] Report availability from the player's class specialization catalog.

## Best-guess basis — best-guess, native UNKNOWN

The Retail pinned `Blizzard_UnitPopupShared/UnitPopupSharedButtonMixins.lua:1823-1825,1837-1863,1877-1903` shows that 0 is the current-spec choice and that other choices come from the player class's specialization information. `PlayerScriptDocumentation.lua:461-467,1493-1501` declares a numeric getter and setter. `LootDocumentation.lua:298-302` declares a synchronous event with no documented payload; its triggering conditions remain unknown. `SpecializationSharedDocumentation.lua:84` declares the separate `HasLootSpecializations` query, used at popup line 1824. [Warcraft Wiki's SetLootSpecialization](https://warcraft.wiki.gg/wiki/API_SetLootSpecialization) reports rejection of invalid or other-class IDs and an update event after a successful change. [Blizzard's 2013 off-spec loot announcement](https://worldofwarcraft.blizzard.com/en-gb/news/10058095/patch-5-3-adds-off-spec-loot-and-rewards) describes player choice. [Player reports](https://www.reddit.com/r/wownoob/comments/nem3m6) describe current specialization as the default. These sources do not establish native persistence, exact event conditions, or availability criteria; those remain simulator best guesses.

## How it works

- [Specialization system](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/lua_api/globals/real/loot_specialization.rs`: global functions.
- `src/lua_api/state/sim_state.rs`: selected ID.

## Tests asserting this spec

- `tests/loot_specialization.rs`: original popup selection and state.

## Known gaps (current cycle)

- [ ] Native persistence, validation, availability and event conditions are unverified.
- [ ] Warcraft Wiki reports a `CHAT_MSG_SYSTEM` message for specID > 0; it is not implemented or natively verified.

## Out of scope

- Persistence across simulator sessions and actual loot generation.
