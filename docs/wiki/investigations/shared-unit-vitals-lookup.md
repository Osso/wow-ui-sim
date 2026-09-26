# Shared unit-vitals lookup

Commit `bd6f091ac` stops unknown or absent units from reading player vitals. It centralizes health/max and power/max lookup for the shared numeric unit-vitals surface. The evidence is source-derived; the corrected `gui,client-wrath` integration binary has not yet run.

## Actual model

`lookup_unit_vitals()` in `spell_api.rs` reads player state for `player`, `self`, `pet`, and `vehicle`. It reads current target/focus snapshots through `resolve_unit_snapshot`; active `partyN` resolves through that same snapshot resolver, while active `raidN` directly indexes the corresponding modeled party member. Inactive party/raid aliases, cleared target/focus, and unknown tokens produce a zero-valued `UnitVitals` with `present = false` rather than falling through to player state.

The `present` flag makes explicit secondary-power `UnitPower` and `UnitPowerMax` return zero too, preventing the prior secondary-player-resource path from leaking a nonzero maximum for an absent unit. `UnitPowerType` retains the existing absent fallback `0, "MANA"`; that metadata behavior is outside the numeric-vitals conclusion.

No target-version selection, Blizzard UI cache, or CASC asset is consulted by this shared lookup. The proof ledger's planned `gui,client-wrath` build is verification scope, not a runtime prerequisite for the implementation.

## Covered source assertions

`admin_health_power_api.rs` adds two grouped assertions:

- Player, target, and focus use distinct seeded health/power values; focus percentage queries read focus values.
- A cleared target and `unknown-unit` are non-existent and return zero for health/max, power/max, both percentage queries, and explicit power type 9.
- An active `party1` and `raid1` read the seeded member; after the group is inactive, those aliases are absent and return zero.

These assertions establish the intended simulator behavior only. The ledger records the initial RED boundaries and fixture corrections, but the final committed `bd6f091ac` scope remains **GREEN pending** an independently built integration binary. This page does not claim a passing test run.

## Limits

No native client evidence establishes absent-unit numeric or power-type semantics. The new cases do not separately cover `self`/`pet`/`vehicle`, multiple party or raid slots, resolver-supported enemy tokens, cleared focus, or all power types. Broader completed unit-resource fixes and the wider API audit portfolio remain unchanged; this is a focused lookup correction, not a reduced audit scope.

## Sources

- [unit-vitals lookup contract](../../specs/unit-vitals-lookup.md) — requirements, exclusions, and pending grouped verification.
- [shared lookup implementation](../../../src/lua_api/globals/utility_system_spell/spell_api.rs) — token branches, zero fallback, and explicit-power guard.
- [target snapshot resolver](../../../src/lua_api/globals/targeting_verbs.rs) — target, focus, party, and resolver token mapping.
- [group unit existence](../../../src/lua_api/globals/group_queries.rs) — active party/raid existence policy.
- [grouped vitals assertions](../../../tests/admin_health_power_api.rs) — seeded focus, absent-unit, party, and raid cases.
- `/tmp/cross-version-vitals-proof.md` — supplied proof ledger and pending-GREEN boundary.

## See Also

- [[api-contract-probes]] — broader unit-query evidence collection remains separate.
- [[unit-raid-target-icons]] — another consumer of shared targeting snapshots.
