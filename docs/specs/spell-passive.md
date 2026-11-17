# C_Spell.IsSpellPassive

This spec covers Retail's spell-passive query and its use by the original PlayerSpells action-bar eligibility helper.

**VERIFIED (R88i.14):** The native local result is false for identifier zero or a missing record; otherwise it tests mask 64 in the selected record. **UNKNOWN (R88i.14):** The exact DB2 table and field are not identified. **VERIFIED (Retail runtime TOC):** `Blizzard_PlayerSpells` is LoadOnDemand and lists `ClassTalents/Blizzard_ClassTalentUtil.lua`; that helper calls `C_Spell.IsSpellPassive` at line 183. **VERIFIED (`crates/sim-data/data/spells.rs`):** `SpellInfo` exposes `Attributes_9`, but R88i.14 does not map mask 64 to that field, so this API does not use it.

## What it must do

- [x] VERIFIED (R88i.14): For Retail, return false for a zero identifier or an absent record.
- [x] VERIFIED (R88i.14): For an existing record, return true only when bit 64 is set; UNKNOWN (R88i.14): the exact table and field.
- [x] VERIFIED (simulator test contract): Keep unknown mask data as an empty-default host input; its false result is simulator policy, not native missing-field behavior.
- [ ] UNKNOWN (R88i.14): Keep full argument conversion, error, and event behavior unclaimed until evidence identifies them.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md)
- [Client profiles](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/c_api/c_spell.rs` — Retail registration and mask query.
- `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs` — empty-default host mask input.
- `src/lua_api/workarounds/temporary/spell_metadata_defaults.rs` — retains unrelated metadata defaults and no longer provides the false-only method.

## Tests asserting this spec

- `tests/blizzard_player_spells_loads.rs` — original PlayerSpells consumer with passive, active, and absent spell fixtures; callable false/true overlays.
- `tests/c_spell_probes.rs` — zero-identifier result.
- `tests/spell_api.rs` — empty-default host input.

## Known gaps (current cycle)

- [ ] Exact native source table, field, and host population for mask 64 remain UNKNOWN (R88i.14).
- [ ] Full argument conversion, error, and event behavior remain UNKNOWN (R88i.14).

## Out of scope

- VERIFIED (R88i.14): Other client profiles are out of scope because this evidence covers Retail 12.1.5 only.
