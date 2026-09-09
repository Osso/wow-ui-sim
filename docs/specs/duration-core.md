# Duration core

Ordinary clock-driven duration state for the existing Lua table proxy in `src/lua_api/globals/lua_duration_object.rs`. This models duration configuration and numeric queries; formulas below are simulator choices, not confirmed native WoW semantics.

## What it must do

- [ ] Preserve table-proxy identity, hidden metatable, method names, and current profile publication; instances hold independent timing state.
- [ ] Configure start/base-duration/rate with `SetTimeFromStart`; configure the equivalent interval from its end with `SetTimeFromEnd`; configure an unmodified interval with `SetTimeSpan`.
- [ ] Query endpoints, total, elapsed, remaining, rate, and zero/activity status consistently while a bound manual clock advances or rewinds.
- [ ] Use simulator elapsed time (the `GetTime` time source) for an unbound clock and `C_DurationUtil.GetCurrentTime`.
- [ ] Reset timing/rate without changing the selected clock; `SetToDefaults` also clears that clock binding.
- [ ] Reject nonfinite endpoints/durations/rates, negative durations, reversed spans, nonpositive rates, and overflowed derived intervals before changing timing state.

### Chosen formulas — native-unverified

Store start `s`, base duration `D >= 0`, and finite rate `r > 0`. Real span `T = D / r`; end `e = s + T`. `SetTimeFromEnd(e,D,r)` derives `s=e-D/r`. `SetTimeSpan(s,e)` stores `D=e-s`, `r=1`.

`RealTime=0` (default) returns total `T`, elapsed `clamp(now-s,0,T)`, remaining `T-elapsed`. `BaseTime=1` multiplies these duration results by `r`. Endpoint queries remain clock coordinates regardless of modifier. Unknown duration-query modifiers error; native modifier/coercion behavior is unverified.

`IsZero` means configured `D==0`, not expiration. `HasStarted` means `now>=s`; `HasExpired` means `now>=e`; `IsActive` requires a nonzero interval and `s<=now<e`. Before start, elapsed is zero and remaining is the full interval. After end, elapsed is the full interval and remaining is zero. Clock rewind reverses these observations without modifying configured timing.

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

## Known gaps (current cycle)

- [ ] Native rate/modifier, endpoint, zero-state, reset, validation, and clock-binding semantics require real-client evidence.
- [ ] Cooldown `SetCooldownFromDurationObject` reads methods with raw table lookup; the duration proxy supplies them through `__index`. Real-proxy consumption fails although standalone timing queries work. Consumer correction belongs to a separate slice.

## Out of scope

- Curve evaluation, percent queries, `Assign`/`Copy`, and rendering behavior remain existing placeholders; this slice adds no evaluation features.
- Secret values, taint, protected/forbidden calls, and immutable proxy internals: not inferred from ordinary numeric behavior.
- Consumer redesign or changes to duration-text-binding identity: preserve current proxy representation.
