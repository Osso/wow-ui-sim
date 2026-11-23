# Script bucket throttle limits

R88bn verifies the registered throttle CVars and Retail refill rule for Retail 12.1.5.70688 and Forever 1.60.1.70707. The simulator registers the values in both profiles. `GetScriptBucketThrottleLimits()` remains a PTR-only temporary mock; its zero fields are placeholders, not native limits or an indication that throttling is disabled.

## What it must do

- [x] Publish the getter only for the PTR API epoch; preserve earlier-retail absence.
- [x] Return exactly one fresh table with exactly four numeric zero fields:
  - `luaScriptBucketThrottleMaxMsPerSecondNormal`
  - `luaScriptBucketThrottleMaxMsPerSecondRestricted`
  - `luaScriptBucketThrottleMaxMsBurstNormal`
  - `luaScriptBucketThrottleMaxMsBurstRestricted`
- [x] Keep returned tables independent: modifying a result cannot affect another result or a later call.

## How it works

- [Lua API](../lua-api.md)
- [Client profiles](client-profiles.md)

## Implementation inventory

- `src/lua_api/workarounds/temporary/script_bucket_throttle_limits.rs`: fresh zero-valued mock result, gated by `retail-12-1-5`.
- `src/lua_api/workarounds/{mod.rs,temporary/mod.rs}`: temporary bootstrap wiring.
- `src/cvars.rs`: Retail and WowForever CVar defaults from R88bn.
- `src/lua_api/execution_budget.rs` and rilua `vm/state/execution_watchdog.rs`: inferred restricted-addon bucket accounting and exhaustion error.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/patch_12_1_5_script_throttle_mock.rs`: exact keys, numeric zeros, arity, independent results, and profile behavior before/after bootstrap.
- `src/cvars.rs` tests: registered defaults by profile.
- `tests/rilua_rows.rs`: exhaustion through a loaded addon event callback.
- Focused development proof at `ec51f3a7e`: one PTR test and one retail test passed using `cargo test --lib --offline --no-default-features --features sound,gui,client-<profile> script_throttle_mock_ -- --nocapture`.

## Bucket enforcement

- VERIFIED (R88bn): Retail refills each bucket by whole elapsed seconds multiplied by its rate, capped at its burst value. Fractional elapsed time accumulates; a bucket already at cap advances its timestamp without adding credit.
- INFERRED (R88bn): Forever uses the same refill rule.
- INFERRED (R88bn; native evidence pending): do not charge the restricted bucket while `startup_script_watchdog_phase` is active. This reuses the existing phase that selects the startup watchdog threshold and remains active through the initial `PLAYER_ENTERING_WORLD` dispatch. The first post-phase addon invocation starts with the full burst.
- INFERRED (R88bn): tainted addon execution uses the restricted bucket. Rilua stores one bucket per addon, charges elapsed execution time at the 10,001-dispatch polling interval and invocation end, and raises `insecure scripts exceeded execution limit for addon <name>` when exhausted.
- VERIFIED (R88bn): `hardcoreScriptThrottlingEnabled` is registered as enabled. Its relation to the per-invocation watchdog is UNKNOWN, so the simulator does not use it to select or change watchdog limits.

## Known gaps

- UNKNOWN (R88bn): which execution classes use normal versus restricted buckets, native balance ownership, whether charging is exempt during loading, native initial bucket balance, and exact charge points. The addon-restricted mapping, per-addon bucket, loading-phase exemption, post-loading burst initialization, and Forever refill mirror are simulator inferences.
- Normal-bucket enforcement is not modeled. Do not treat the CVar's presence as proof that normal execution is charged.
- The per-invocation elapsed-time watchdog is separate. It does not read or change bucket values.
