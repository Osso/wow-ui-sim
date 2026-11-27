# PvP lifetime stats

`GetPVPLifetimeStats` reads the simulator's supplied lifetime honor snapshot on Retail and WoW Forever. Its native output shape remains **INFERRED R88g9 (native evidence pending)**.

## What it must do

- [x] Return the supplied honorable-kill count, dishonorable-kill count, and highest rank in that order on Retail and WoW Forever.
- [x] Return different seeded snapshots without substituting a constant.

## How it works

- [PvP probes](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/lua_api/globals/pvp_probes.rs` — Retail and Forever query registration and state read.
- `src/lua_api/globals/admin_pvp_guild.rs` — validated host input for lifetime snapshot fields.
- `src/lua_api/workarounds/temporary/difficulty_pvp_util_defaults.rs` — fallback for profiles outside this scope.

## Tests asserting this spec

- `tests/pvp_lifetime_stats.rs`

## Known gaps (current cycle)

- [ ] Confirm the native return count and field order for Retail and Forever (R88g9).

## Out of scope

- Honor event production and lifetime counter mutation; this query reads only the supplied snapshot.
