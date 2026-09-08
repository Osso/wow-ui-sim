# Controlled-player and group unit tokens

`UnitIsPlayerControlledOrGroupMember` classifies the native controlled-player/group token families for retail 12.1 aura filtering. Implementation belongs in `src/lua_api/globals/real/unit_relationships.rs`; it does not infer unit existence, ownership, or target aliases.

## What it must do

- [x] Return true for `player`, `pet`, and `vehicle`.
- [x] Return true for `party1`–`party4`, `partypet1`–`partypet4`, `raid1`–`raid40`, and `raidpet1`–`raidpet40`, including when no group roster is populated.
- [x] Return false for nonfamily tokens and invalid group indices; another player selected through `target` is not a controlled-player token.
- [x] Publish before secure environment copying so Blizzard's helpful-aura identity filtering can use the same classification.

## How it works

- [Lua API](../lua-api.md)

## Implementation inventory

- `src/lua_api/globals/real/unit_relationships.rs` — native token classification and registration.
- `src/lua_api/globals/real/mod.rs` and `src/lua_api/globals/register.rs` — retail 12.1 registration wiring.

## Tests asserting this spec

`tests/unit_player_controlled_or_group_member.rs`: concrete token families, invalid indices/nonfamily tokens, empty roster independence, and actual secure Blizzard aura filtering. Targeted run at `d80464a81`: 3/3 passed after all three failed on the missing global; logs `/tmp/pi-unit-controlled-{red,green}.{stdout,stderr}.log`.

## Known gaps (current cycle)

- [ ] Independent integration verification pending; targeted RED/GREEN completed.

## Out of scope

Unit existence, target/focus identity aliases, pet ownership state, and secret-argument enforcement. The documented function is not a replacement for `UnitExists` or `UnitPlayerControlled`.

## Native sources

Cached retail `Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:2197–2212` specifies: “Returns true for 'player', 'pet', 'vehicle', or any of 'partyn', 'partypetn', 'raidn', 'raidpetn'”. Existing `src/lua_api/globals/strings/string_data/core_strings.rs` supplies native `MAX_PARTY_MEMBERS=4` and `MAX_RAID_MEMBERS=40`.
