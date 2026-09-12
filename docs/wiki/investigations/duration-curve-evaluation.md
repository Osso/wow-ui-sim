# Duration curve evaluation

Commit `aae5996eb` replaces four `LuaDurationObject` constant-zero evaluation stubs: elapsed/remaining seconds and elapsed/remaining fractions now call their existing duration getter, then the registered scalar or color curve evaluator. It does not duplicate interpolation.

## Evidence boundary

`/tmp/duration-curve-red-ledger.json` records ten focused 12.0.0 cases failing before the change: all four scalar methods returned zero, color results were numeric zero, and modifier, clock, missing-curve, live-curve, and unsupported-interpolation errors were suppressed by the stub.

The tests use existing scalar and color curve models, concrete timing boundaries, live point changes, invalid inputs, and existing interpolation errors. GREEN and independent verification are pending.

Cached documentation names the return `LuaCurveEvaluatedResult`; cached scalar and color curve declarations respectively return a number and a ColorMixin value. This does not establish a separate result-object representation.

## Scope

The implementation reuses current duration modifier and clock validation plus existing curve behavior. It adds no `EvaluateTotalDuration`, interpolation policy, vendor code, VM changes, or security enforcement.

## Open boundaries

Native clock/modifier/fraction policies, curve extrapolation, coercion/error precedence, color identity, secret/taint propagation, lifecycle, and real Blizzard consumer behavior remain unproven. Cached Blizzard UI has no practical consumer of these four methods.

## Sources

- [Duration curve evaluation spec](../../specs/duration-curve-evaluation.md)
- [Duration core](../systems/duration-core.md)
- `tests/duration_core.rs`
- `/tmp/duration-curve-red-ledger.json`
