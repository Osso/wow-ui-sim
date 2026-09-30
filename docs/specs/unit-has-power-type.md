# Unit power-type capability

`UnitHasPowerType(unitToken, powerType)` returns exactly one non-secret boolean indicating whether a unit uses that power type. Source: Blizzard's March 25, 2026 “Midnight 12.0.5 PTR Changes Episode 2: Attack of the Secrets”, preserved at `/tmp/patch-12.0.5-api-source-plain.txt`: “a new UnitHasPowerType API that returns a non-secret bool indicating if the unit uses a power type.” The source establishes capability and non-secret output, not malformed-input or absent-unit semantics. See [Lua API](../lua-api.md) for runtime context.

## What it must do

- [ ] Publish only with cumulative `retail-12-0-5`; older epochs/profiles without that feature retain no global. Older-profile execution remains unverified in this slice.
- [x] Return one ordinary, non-secret boolean for matching and nonmatching capabilities.
- [x] Inferred simulator capability: a present unit's primary type matches even with zero current and maximum resources. Player/self also match any explicitly modeled secondary-map entry, even with zero current or maximum; removal removes capability.
- [x] Inferred simulator absence: absent target/focus, inactive group tokens, unknown tokens and empty strings return false, never player capability. Present target/group units expose only their primary type. Pet/vehicle retain existing player-primary vitals aliasing but do not inherit player secondary entries.
- [x] Inferred argument policy: require string unitToken and a finite numeric integral powerType representable as i32; missing/nil, wrong types, numeric strings, fractions, nonfinite numbers and overflow error, including for absent units. Valid unmodeled integers return false. No generic secondary maximum invents capability.

## How it works

- [Lua API](../lua-api.md)
- [Unit vitals lookup](unit-vitals-lookup.md)

## Implementation inventory

- `src/lua_api/globals/utility_system_spell/spell_api.rs`: feature-gated capability query, strict numeric validation and registration beside power queries.

## Tests asserting this spec

- `tests/unit_has_power_type.rs`: five focused current-retail tests with concrete primary/secondary, target and group state; real Lua capability, arity, secrecy and error queries. Generated grouping in existing `integration` target adds no Cargo target. A feature-off publication test is available but not run in this slice.
- Corrected RED: five API-missing failures before production changes. Implementation `5bfca2e16` passes targeted GREEN 5/5 with the shared default-retail integration binary compiled at `72220958b`; `timeout 90` direct filtered execution exits 0 in 0.23 seconds. Checked requirements describe tested simulator behavior, not native verification. Commands, source hashes, results and logs: `/tmp/patch-12.0.5-unit-power-ledger.md` and `/tmp/patch-12.0.5-unit-power-green.log`. Final readability/check verifier belongs to main.

## Known gaps (current cycle)

- [ ] Future native probe: zero current/maximum primary pools, empty or temporarily unavailable secondary pools, class/spec transitions and whether a zero-capacity secondary remains a capability.
- [ ] Future native probe: absent/unknown/empty unit tokens; self alias; pet/vehicle secondary ownership; group/target primary and secondary capability; integer values outside the published power enum.
- [ ] Future native probe: omitted/nil/wrong-type tokens or power types, numeric strings, fractions, nonfinite and out-of-i32 values, error conventions and validation order. Strict local behavior follows nearby `SetRaidTarget`'s numeric-integer/error convention; native equivalence is inferred, not verified.
- [ ] Future native probe: true/false return count and non-secret output during combat and unit secrecy, including secret arguments. Local ordinary boolean assertions do not establish native secret-input policy.

## Out of scope

New pet/vehicle/target secondary models, changes to existing `UnitPower`/`UnitPowerMax` defaults or vitals aliases, historical-profile compilation repairs, vendor edits, and native-client parity claims.
