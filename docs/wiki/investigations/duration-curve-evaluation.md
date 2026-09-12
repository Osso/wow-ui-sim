# Duration curve evaluation

Commit `aae5996eb` replaces four `LuaDurationObject` constant-zero evaluation stubs: elapsed/remaining seconds and elapsed/remaining fractions now call their existing duration getter, then the registered scalar or color curve evaluator. It does not duplicate interpolation.

## Evidence boundary

`/tmp/duration-curve-red-ledger.json` records ten focused 12.0.0 cases failing before the change: all four scalar methods returned zero, color results were numeric zero, and modifier, clock, missing-curve, live-curve, and unsupported-interpolation errors were suppressed by the stub.

The tests use existing scalar and color curve models, concrete timing boundaries, live point changes, invalid inputs, and existing interpolation errors. `/tmp/duration-curve-green-ledger.json` records 10/10 passing on retail 12.0.0 at `aae5996eb`. Final proof `/tmp/verify-duration-curve-ledger.json` passes all 25 `duration_core::` cases with exit 0 on 12.0.0, 12.0.5, 12.0.7, and Mists, plus 11 PTR curve/userdata cases. Format, default check, both default binaries, zero-error startup, both affected validators, and the all-manifest hash scan pass. Historical warnings remain 6/6/1/6; PTR and default verification emit none.

Cached documentation names the return `LuaCurveEvaluatedResult`; cached scalar and color curve declarations respectively return a number and a ColorMixin value. This does not establish a separate result-object representation.

## Scope

The implementation reuses current duration modifier and clock validation plus existing curve behavior. It adds no `EvaluateTotalDuration`, interpolation policy, vendor code, VM changes, or security enforcement.

## Existing integration credits

`4393823e1` credits existing focused integration assertions for scalar `LuaCurveObject.ClearPoints` and color `LuaColorCurveObject.AddPoint`, `ClearPoints`, and `Evaluate`; no runtime or test changed. A rebuilt scalar curve changes the observed midpoint from `20` to `70`. A rebuilt color curve produces midpoint RGBA `(0.5, 0.5, 0.5, 0.25)`, and the existing color evaluation checks midpoint/endpoint ColorMixin channels with tested modifiers. `9957fc863` keeps the credit limited to those fixture values. The prior cross-profile/PTR proof at `/tmp/verify-duration-curve-ledger.json` covers the reused test file; independent validation of the metadata-only credit remains pending.

## Open boundaries

Native clock/modifier/fraction policies, curve extrapolation, coercion/error precedence, color identity, secret/taint propagation, lifecycle, and real Blizzard consumer behavior remain unproven. Cached Blizzard UI has no practical consumer of these four methods.

## Sources

- [Duration curve evaluation spec](../../specs/duration-curve-evaluation.md)
- [Duration core](../systems/duration-core.md)
- `tests/duration_core.rs`
- `/tmp/duration-curve-red-ledger.json`
