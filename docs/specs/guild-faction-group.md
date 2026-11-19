# Retail `GetGuildFactionGroup`

`GetGuildFactionGroup()` returns the player's native faction-group classification for Retail 12.1.5. The public contract is the race-to-`FactionTemplate` flag lookup. See the [Guild faction group system page](../wiki/systems/guild-faction-group.md) for the simulator model.

## What it must do

- [x] **VERIFIED (pr46-49-decomp-answers.md §R88ap part A section 2, Retail 12.1.5.70688):** Return zero Lua values when the player-object accessor has no object; otherwise return one number.
- [x] **VERIFIED (pr46-49-decomp-answers.md §R88ap part A section 2, Retail 12.1.5.70688):** Look up the player's race row and its linked `FactionTemplate` row. Return `-1` when either row is absent.
- [x] **VERIFIED (pr46-49-decomp-answers.md §R88ap part A section 2, Retail 12.1.5.70688):** For a found template row, flag 4 has priority and returns 0; otherwise flag 2 returns 1; otherwise flag 1 returns 0; otherwise return -1.
- [x] **VERIFIED (pr46-49-decomp-answers.md §R88ap part A section 2, Retail 12.1.5.70688):** Do not test guild membership.
- [x] **VERIFIED (Retail 12.1.5.70783 `Blizzard_Communities/GuildNews.lua:35`):** GuildNews selects `UI-Calendar-Event-PVP01` only when the result is 0; every other result selects `UI-Calendar-Event-PVP02`.
- [x] **INFERRED (simulator mapping):** Read per-race flags from a host-supplied map that is empty by default. Do not ship inferred race rows as native facts.

**INFERRED (R88ap.2; DB2 export build 70448):** Human classifies as 1, Orc as 0, and Pandaren as 0. These outputs are context only. The raw group-flag rows are not supplied, so the simulator does not seed these classifications.

## How it works

- [Guild faction group system](../wiki/systems/guild-faction-group.md)
- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/lua_api/globals/real/player_identity.rs` — Retail global and flag-order classification.
- `src/lua_api/state_types/character_world.rs` — empty-default host-supplied race-to-flag rows.
- `src/lua_api/globals/admin.rs` — test/admin input setter for race flags.
- `src/lua_api/workarounds/temporary/inert_global_defaults.rs` — keeps the prior fallback on profiles outside Retail.

## Tests asserting this spec

- `tests/admin_pvp_guild_api.rs` — flag order, missing rows, absent player object, and empty-default result.
- `tests/blizzard_communities_loads.rs::guild_news_pvp_texture_uses_player_faction_group` — original GuildNews consumer.
- `tests/blizzard_communities_loads.rs::wrong_guild_faction_group_producers_fail_guild_news_assertions` — callable-wrong producer controls.

## Known gaps (current cycle)

- [ ] Raw Retail `ChrRaces`/`FactionTemplate` group-flag rows are not supplied by the simulator. Real race classifications remain unknown until the host supplies rows.
- [ ] The native conditions that make the player-object accessor unavailable are unknown.

## Out of scope

- Other client profiles keep their existing `GetGuildFactionGroup` fallback.
- No inferred DB2 row is added as simulator production data.
