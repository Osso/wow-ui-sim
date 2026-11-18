# Retail Guild faction group

Retail `GetGuildFactionGroup()` is a global query over player race and supplied `FactionTemplate` group flags. The simulator keeps raw race rows empty by default. This page describes the state model; the [contract](../../specs/guild-faction-group.md) lists the required behavior.

## Content

**VERIFIED (R88ap.2, Retail 12.1.5.70688):** The native getter returns zero values when its player-object accessor returns no object. With an object, it returns one number and does not test guild membership. It follows the race row's linked key to a `FactionTemplate` row. Missing either row returns -1. For a found row, flag 4 returns 0 first, then flag 2 returns 1, then flag 1 returns 0; no listed flag returns -1.

**VERIFIED (simulator source):** `PlayerState.guild_faction_group_flags` is an empty-default map keyed by race ID. Retail computes the result from that raw flag value. A missing row returns -1. The global returns zero values when the simulator's player object is absent. The admin setter supplies test fixture rows; it does not seed production data.

**UNKNOWN:** The repository does not supply the raw Retail race-to-`FactionTemplate` flag rows. The native conditions that make the player-object accessor unavailable are also unknown.

**VERIFIED (Retail 12.1.5.70783 consumer bytes):** `Blizzard_Communities/GuildNews.lua:35` selects `UI-Calendar-Event-PVP01` for result 0 and `UI-Calendar-Event-PVP02` for every other result. This consumer choice does not redefine the native numeric classification.

## Sources

- [Contract](../../specs/guild-faction-group.md) — public behavior and test coverage.
- `R88ap.2 GetGuildFactionGroup` decomp handoff — native behavior evidence; the handoff file is not part of the repository.
- `Blizzard_Communities/GuildNews.lua:35` — original Retail texture consumer.
- `src/lua_api/globals/real/player_identity.rs` — Retail producer.
- `src/lua_api/state_types/character_world.rs` — host-supplied raw flags.
- `tests/blizzard_communities_loads.rs` — original-consumer coverage.

## See Also

- [[lua-api]] — global registration and Lua API behavior.
