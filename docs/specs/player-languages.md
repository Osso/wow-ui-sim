# Player chat languages

Retail and WoW Forever expose the player's chat languages through the four legacy language globals. The implementation lives in [`player_languages.rs`](../../src/lua_api/globals/real/player_languages.rs); descriptive notes belong in the [player languages system page](../wiki/systems/player-languages.md).

## What it must do

- [x] **INFERRED (R88eg, native evidence pending):** The real Blizzard ChatFrame language menu lists Common for a supplied Human and Orcish plus Troll for a supplied Troll; its selected default language matches each player's faction language.
- [x] **INFERRED (R88eg, native evidence pending):** The ChatFrame consumer stores the alternative default language for a supplied Human as nil and a supplied Troll as Troll.
- [ ] **INFERRED (R88eg, native evidence pending):** Supply the correct faction and racial language names, IDs, ordering, and alternative default for every supported race.
- [ ] Register the modeled globals only on Retail and WoW Forever. Keep other profile registrations and the legacy fallback source unchanged.

## How it works

- [Player languages system](../wiki/systems/player-languages.md)
- [Lua API architecture](../lua-api.md)

## Implementation inventory

- [`src/lua_api/globals/real/player_languages.rs`](../../src/lua_api/globals/real/player_languages.rs) — derives language queries from player race and faction state.
- [`src/lua_api/globals/real/mod.rs`](../../src/lua_api/globals/real/mod.rs) — declares the modeled global module.
- [`src/lua_api/globals/register.rs`](../../src/lua_api/globals/register.rs) — registers the globals for supported profiles.

## Tests asserting this spec

- [`tests/player_languages.rs`](../../tests/player_languages.rs) — loads real Blizzard UI and checks the chat-language menu and its defaults for two supplied races.

## Known gaps (current cycle)

- [ ] **INFERRED (R88eg, native evidence pending):** Verify the complete race-to-language table and alternative-default behavior against native evidence.

## Out of scope

- Changing language fallback behavior on profiles other than Retail and WoW Forever.
- Modeling chat message delivery or language-learning mutations.
