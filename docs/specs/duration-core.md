# Duration core

Ordinary clock-driven duration state for the existing Lua table proxy in `src/lua_api/globals/lua_duration_object.rs`. This models duration configuration and numeric queries; formulas below are simulator choices, not confirmed native WoW semantics.

## What it must do

- [x] Preserve table-proxy identity, hidden metatable, method names, and current profile publication; instances hold independent timing state.
- [x] Configure start/base-duration/rate with `SetTimeFromStart`; configure the equivalent interval from its end with `SetTimeFromEnd`; configure an unmodified interval with `SetTimeSpan`.
- [x] Query endpoints, total, elapsed, remaining, rate, and zero/activity status consistently while a bound manual clock advances or rewinds.
- [x] Use simulator elapsed time (the `GetTime` time source) for an unbound clock and `C_DurationUtil.GetCurrentTime`.
- [x] Reset timing/rate without changing the selected clock; `SetToDefaults` also clears that clock binding.
- [x] Reject tested nonfinite endpoints/durations, negative durations, reversed spans, and zero rates before changing timing state. Additional overflow/nonfinite-rate rejection is implemented but not separately proven.

### Chosen formulas — native-unverified

Store start `s`, base duration `D >= 0`, and finite rate `r > 0`. Real span `T = D / r`; end `e = s + T`. `SetTimeFromEnd(e,D,r)` derives `s=e-D/r`. `SetTimeSpan(s,e)` stores `D=e-s`, `r=1`.

`RealTime=0` (default) returns total `T`, elapsed `clamp(now-s,0,T)`, remaining `T-elapsed`. `BaseTime=1` multiplies these duration results by `r`. Endpoint queries remain clock coordinates regardless of modifier. Unknown duration-query modifiers error; native modifier/coercion behavior is unverified.

`IsZero` means configured `D==0`, not expiration. Zero durations report false for `HasStarted`, `HasExpired`, and `IsActive`, preserving existing default behavior. For nonzero durations, `HasStarted` means `now>=s`; `HasExpired` means `now>=e`; `IsActive` means `s<=now<e`. Before start, elapsed is zero and remaining is the full interval. After end, elapsed is the full interval and remaining is zero. Clock rewind reverses these observations without modifying configured timing.

`Reset` produces `s=0,D=0,r=1` and retains the clock. `SetToDefaults` additionally selects the default simulator clock. Existing manual clocks use their mutable `time` field; invalid/nonfinite bound clock time errors on clock-dependent queries. Clock setter validation remains unchanged.

## How it works

- [Lua API architecture](../lua-api.md)
- [Duration text binding](duration-text-binding.md)

## Implementation inventory

- `src/lua_api/globals/lua_duration_object.rs`: existing proxy factory, clock binding, method registration.
- `src/lua_api/globals/lua_duration_object/core.rs`: ordinary timing configuration and queries.

## Tests asserting this spec

- `tests/duration_core.rs`: manual progression/rewind, rate modifiers, end/span configuration, reset, atomic validation, default time source, independent instances.
- Existing `tests/cooldown_widget.rs` and duration-text-binding tests: bounded consumer regression checks; these do not establish native core formulas.

Focused proof at `89a71308d`: `duration_core::` has three passing tests on PTR and retail; `test_patch_12_0_7_duration_objects_and_text_binding` passes on PTR. Bounded consumer run at `9aa4a1eb7` passed eight tests and failed one forbidden-object AuraContainer fixture; these are not a clean full consumer acceptance result.

## Known gaps (current cycle)

- [ ] Native rate/modifier, endpoint, zero-state, reset, validation, and clock-binding semantics require real-client evidence.
- [ ] `SetCooldownFromDurationObject` now resolves duration-proxy methods through Lua indexing and ordinary proxy transfer/reset behavior is covered. Its new PTR protected-function contract, secret handling, `clearIfZero = false`, and forbidden-object behavior remain unproven.
- [ ] Existing `test_patch_12_1_duration_binding_reference_lifetime_and_identity` expects a table from the separate duration-text-binding factory and fails with `type`; the core preserves its own table identity. Existing AuraContainer binding integration fails with `expected forbidden object reference`. These are separate consumer boundaries; this slice establishes no cause beyond their observed failures.

## Out of scope

- Curve evaluation, percent queries, `Assign`/`Copy`, and rendering behavior remain existing placeholders; this slice adds no evaluation features.
- Secret values, taint, protected/forbidden calls, and immutable proxy internals: not inferred from ordinary numeric behavior.
- Consumer redesign or changes to duration-text-binding identity: preserve current proxy representation.
