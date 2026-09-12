# Duration core

Ordinary clock-driven duration state for the existing Lua table proxy in `src/lua_api/globals/lua_duration_object.rs`. This models duration configuration and numeric queries; formulas below are simulator choices, not confirmed native WoW semantics.

## What it must do

- [x] Preserve table-proxy identity, hidden metatable, method names, and current profile publication; instances hold independent timing state.
- [x] Configure start/base-duration/rate with `SetTimeFromStart`; configure the equivalent interval from its end with `SetTimeFromEnd`; configure an unmodified interval with `SetTimeSpan`.
- [x] Query endpoints, total, elapsed, remaining, rate, and zero/activity status consistently while a bound manual clock advances or rewinds.
- [x] Use simulator elapsed time (the `GetTime` time source) for an unbound clock and `C_DurationUtil.GetCurrentTime`.
- [x] Reset timing/rate without changing the selected clock; `SetToDefaults` also clears that clock binding.
- [x] Reject tested nonfinite endpoints/durations, negative durations, reversed spans, and zero rates before changing timing state. Additional overflow/nonfinite-rate rejection is implemented but not separately proven.

- [x] Return elapsed/remaining fractions from the same timing state, with clock-boundary clamping, rewind, and modifier validation. Focused proof passed 7/7 on retail 12.0.0 (reused matching hashes), 12.0.5, and 12.0.7; see `/tmp/verify-duration-percent-ledger.json`.

### Chosen formulas — native-unverified

Store start `s`, base duration `D >= 0`, and finite rate `r > 0`. Real span `T = D / r`; end `e = s + T`. `SetTimeFromEnd(e,D,r)` derives `s=e-D/r`. `SetTimeSpan(s,e)` stores `D=e-s`, `r=1`.

`RealTime=0` (default) returns total `T`, elapsed `clamp(now-s,0,T)`, remaining `T-elapsed`. `BaseTime=1` multiplies these duration results by `r`. Endpoint queries remain clock coordinates regardless of modifier. Unknown duration-query modifiers error; native modifier/coercion behavior is unverified.

Percentage queries return dimensionless fractions in `[0,1]`: `GetElapsedPercent` is elapsed divided by `T`, and `GetRemainingPercent` is remaining divided by `T`. Both return zero when `T==0`. Omitted/nil, `RealTime`, and `BaseTime` give the same fractions because numerator and denominator share the time scale; invalid modifiers and invalid bound clocks still error. Division uses real-time quantities before scaling. Fraction scale, clamping, zero-span results, validation, and rewind behavior are simulator policies, not native conformance claims.

`IsZero` means configured `D==0`, not expiration. Zero durations report false for `HasStarted`, `HasExpired`, and `IsActive`, preserving existing default behavior. For nonzero durations, `HasStarted` means `now>=s`; `HasExpired` means `now>=e`; `IsActive` means `s<=now<e`. Before start, elapsed is zero and remaining is the full interval. After end, elapsed is the full interval and remaining is zero. Clock rewind reverses these observations without modifying configured timing.

`Reset` produces `s=0,D=0,r=1` and retains the clock. `SetToDefaults` additionally selects the default simulator clock. Existing manual clocks use their mutable `time` field; invalid/nonfinite bound clock time errors on clock-dependent queries. Clock setter validation remains unchanged.

## How it works

- [Lua API architecture](../lua-api.md)
- [Duration text binding](duration-text-binding.md)

## Implementation inventory

- `src/lua_api/globals/lua_duration_object.rs`: existing proxy factory, clock binding, method registration.
- `src/lua_api/globals/lua_duration_object/core.rs`: ordinary timing configuration and queries.

## Tests asserting this spec

- `tests/duration_core.rs`: manual progression/rewind, rate modifiers, end/span configuration, reset, atomic validation, default time source, independent instances; percentage boundary/rewind, zero-span, invalid modifier, and invalid clock policies.
- Existing `tests/cooldown_widget.rs` and duration-text-binding tests: bounded consumer regression checks; these do not establish native core formulas.

Focused proof at `24fe9d746` and `/tmp/verify-duration-percent-ledger.json` passes all seven `duration_core::` tests on retail 12.0.0 (reused matching hashes), 12.0.5, and 12.0.7. It also records `cargo fmt --check`, default `cargo check`, default `wow-sim`/`wow-cli` build, and no-addons/no-saved-vars startup `lua-errors` as exit 0; startup reported `[]`. This proves ordinary simulator policies only. Focused proof at `89a71308d`: `duration_core::` has three passing tests on PTR and retail; `test_patch_12_0_7_duration_objects_and_text_binding` passes on PTR. Bounded consumer run at `9aa4a1eb7` passed eight tests and failed one forbidden-object AuraContainer fixture; these are not a clean full consumer acceptance result.

## Cooldown zero-duration option

`SetCooldownFromDurationObject(duration, clearIfZero)` defaults `clearIfZero` to `true` in both pinned base and PTR declarations. The simulator interprets explicit `false` with a zero duration as preserving the cooldown's current start, duration, display duration, and rate. Omitted/true clears timing; nonzero duration objects update timing regardless of the flag. `tests/cooldown_widget.rs` covers these cases with real duration proxies and independent frames. This interpretation does not establish native, protected-call, or type semantics.

## Known gaps (current cycle)

- [ ] Native rate/modifier, endpoint, zero-state, reset, validation, and clock-binding semantics require real-client evidence.
- [ ] `SetCooldownFromDurationObject` now resolves duration-proxy methods through Lua indexing and ordinary proxy transfer/reset behavior is covered. Its new PTR protected-function contract, secret handling, and forbidden-object behavior remain unproven.
- [ ] Existing `test_patch_12_1_duration_binding_reference_lifetime_and_identity` expects a table from the separate duration-text-binding factory and fails with `type`; the core preserves its own table identity. Existing AuraContainer binding integration fails with `expected forbidden object reference`. These are separate consumer boundaries; this slice establishes no cause beyond their observed failures.

## Out of scope

- Curve evaluation, `Assign`/`Copy`, and rendering behavior remain outside this slice; no curve-evaluation features are added.
- Secret values, taint, protected/forbidden calls, and immutable proxy internals: not inferred from ordinary numeric behavior.
- Consumer redesign or changes to duration-text-binding identity: preserve current proxy representation.
