# Shared unit-vitals lookup

Commit `bd6f091ac` centralizes health/max and power/max lookup for the shared numeric unit-vitals surface. Follow-up `19757e081` removes late duplicate `UnitHealthMax` and `UnitPowerMax` registrations from `unit_stats.rs`: registration order had overwritten the shared handlers with player/stat shortcuts. The evidence is source-derived; GREEN remains pending an independent `gui,client-wrath` integration verifier.

## Actual model

`lookup_unit_vitals()` in `spell_api.rs` reads player state for `player`, `self`, `pet`, and `vehicle`. It reads current target/focus snapshots through `resolve_unit_snapshot`; active `partyN` resolves through that same snapshot resolver, while active `raidN` directly indexes the corresponding modeled party member. Inactive party/raid aliases, cleared target/focus, and unknown tokens produce a zero-valued `UnitVitals` with `present = false` rather than falling through to player state.

The `present` flag makes explicit secondary-power `UnitPower` and `UnitPowerMax` return zero too, preventing the prior secondary-player-resource path from leaking a nonzero maximum for an absent unit. `UnitPowerType` retains the existing absent fallback `0, "MANA"`; that metadata behavior is outside the numeric-vitals conclusion.

The targeting-only seed regression and the `UnitPowerMax` single-result regression both reproduced RED before the duplicate-registration removal. The new authoritative path routes both maxima through `lookup_unit_vitals()` and `requested_power_values()`, so modeled focus/group data and absent-unit guards apply consistently.

Cached current `Blizzard_APIDocumentationGenerated/UnitDocumentation.lua` and read-only Wrath Classic 3.4.3 `UnitDocumentation.lua` independently declare one `maxPower` return. This establishes the modeled return count only; it does not establish absent-unit values, coercion, `unmodified` behavior, or secret/restriction metadata semantics.

No target-version selection, Blizzard UI cache, or CASC asset is consulted by the lookup. The current documentation file is return-shape evidence, not a runtime prerequisite. The proof ledger's planned `gui,client-wrath` build is verification scope, not a runtime prerequisite for the implementation.

## Covered source assertions

`admin_health_power_api.rs` adds two grouped assertions:

- Player, target, and focus use distinct seeded health/power values; focus percentage queries read focus values.
- A cleared target and `unknown-unit` are non-existent and return zero for health/max, power/max, both percentage queries, and explicit power type 9.
- An active `party1` and `raid1` read the seeded member; after the group is inactive, those aliases are absent and return zero.

These assertions establish intended simulator behavior only. `19757e081` adds the targeting-only seed and one-return maximum regressions after their RED boundary was reproduced; the focus-state refactor `8c5e7afcf` is adjacent revision context and does not alter vitals registration. The final `bd6f091ac` plus `19757e081` scope remains **GREEN pending** an independently built integration binary. This page does not claim a passing test run.

## Limits

No native client evidence establishes absent-unit numeric or power-type semantics. The new cases do not separately cover `self`/`pet`/`vehicle`, multiple party or raid slots, resolver-supported enemy tokens, cleared focus, or all power types. Broader completed unit-resource fixes and the full simulator audit scope remain unchanged; this correction imposes no loading-only or version-only priority.

## Sources

- [unit-vitals lookup contract](../../specs/unit-vitals-lookup.md) — requirements, exclusions, and pending grouped verification.
- [shared lookup implementation](../../../src/lua_api/globals/utility_system_spell/spell_api.rs) — authoritative maxima registration, token branches, zero fallback, and explicit-power guard.
- [former duplicate registration module](../../../src/lua_api/globals/unit_stats.rs) — retained combat-stat inventory after removal of late maxima handlers.
- [target snapshot resolver](../../../src/lua_api/globals/targeting_verbs.rs) — target, focus, party, and resolver token mapping.
- [group unit existence](../../../src/lua_api/globals/group_queries.rs) — active party/raid existence policy.
- [grouped vitals assertions](../../../tests/admin_health_power_api.rs) — seeded focus, absent-unit, party, and raid cases.
- `/tmp/cross-version-vitals-proof.md` — supplied proof ledger and pending-GREEN boundary.
- cached current and read-only Wrath 3.4.3 `UnitDocumentation.lua` — independent one-`maxPower` return declarations; no broader semantic credit.

## See Also

- [[api-contract-probes]] — broader unit-query evidence collection remains separate.
- [[unit-raid-target-icons]] — another consumer of shared targeting snapshots.
