# Script bucket throttle limit mock

PTR `GetScriptBucketThrottleLimits()` is an explicit temporary mock in `src/lua_api/workarounds/temporary/script_bucket_throttle_limits.rs`. The [pinned register](../../data/patch-api/sources/12.1.5-register.json) specifies the return fields, not their values. Zero placeholders are the user-requested policy, not native limits or an indication that throttling is disabled.

## What it must do

- [ ] Publish the getter only for the PTR API epoch; preserve earlier-retail absence.
- [ ] Return exactly one fresh table with exactly four numeric zero fields:
  - `luaScriptBucketThrottleMaxMsPerSecondNormal`
  - `luaScriptBucketThrottleMaxMsPerSecondRestricted`
  - `luaScriptBucketThrottleMaxMsBurstNormal`
  - `luaScriptBucketThrottleMaxMsBurstRestricted`
- [ ] Keep returned tables independent: modifying a result cannot affect another result or a later call.

## How it works

- [Lua API](../lua-api.md)
- [Client profiles](client-profiles.md)

## Implementation inventory

- `src/lua_api/workarounds/temporary/script_bucket_throttle_limits.rs`: fresh zero-valued mock result, gated by `retail-12-1-5`.
- `src/lua_api/workarounds/{mod.rs,temporary/mod.rs}`: temporary bootstrap wiring.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/patch_12_1_5_script_throttle_mock.rs`: exact keys, numeric zeros, arity, independent results, and profile behavior before/after bootstrap.

## Known gaps (current cycle)

- Native values are unknown. Replace placeholders only when authoritative return values are available.

## Out of scope

Per explicit user override, throttling enforcement is out of scope, not a pending completion gate. No setters, mode selectors, configuration, accounting, budget state, or VM changes are introduced.
