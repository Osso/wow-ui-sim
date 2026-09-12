# Duration curve evaluation

The four existing duration evaluation methods apply modeled elapsed/remaining seconds or fractions to an existing scalar or color curve. Duration queries live in `src/lua_api/globals/lua_duration_object/core.rs`; the curve model remains in `src/c_api/c_curve_util.rs`.

## What it must do

- [x] `EvaluateElapsedDuration` and `EvaluateRemainingDuration` evaluate the corresponding modeled seconds with the optional duration modifier, defaulting to `RealTime`.
- [x] `EvaluateElapsedPercent` and `EvaluateRemainingPercent` evaluate the corresponding modeled fractions; their existing modifier-invariant fraction policy remains unchanged.
- [x] Return the existing curve evaluator's scalar number or ColorMixin result, preserving observed color channels and live point changes.
- [x] Reject missing/non-curve inputs, propagate duration modifier/clock errors and unsupported curve-evaluation errors, and leave duration configuration and curve points unchanged on tested failures.

Cached duration documentation describes a `LuaCurveObjectBase` input and `LuaCurveEvaluatedResult` return. Cached scalar and color curve declarations respectively return a number and a ColorMixin value. This does not establish a separate result-object representation. Tests exercise existing simulator curve behavior, not native historical-client conformance.

## How it works

- [Duration core](duration-core.md)
- [Curve objects](curve-objects.md)

## Implementation inventory

- `src/lua_api/globals/lua_duration_object/core.rs`: invokes the matching duration query, then curve evaluation.
- `src/lua_api/globals/lua_duration_object.rs`: duration method registry and receiver validation.
- `src/c_api/c_curve_util.rs`: accepts registered scalar/color curves and invokes their existing evaluator without duplicating interpolation.

## Tests asserting this spec

`tests/duration_core.rs` contains ten `duration_curve_` cases for seconds/fractions, before-start through expiry and rewind, modifiers, color channels, live point changes, invalid inputs, and propagated interpolation errors. RED records all ten failing against the constant-zero placeholders; `/tmp/duration-curve-green-ledger.json` records 10/10 passing at `aae5996eb`. Final proof `/tmp/verify-duration-curve-ledger.json` passes all 25 `duration_core::` cases with exit 0 on 12.0.0, 12.0.5, 12.0.7, and Mists; it also passes 11 PTR curve/userdata cases, format, default check, both default binaries, zero-error startup, affected validators, and the all-manifest hash scan.

## Known gaps (current cycle)

- [ ] Native clock/modifier/fraction policies, curve extrapolation, coercion/error precedence, color identity, secret/taint behavior, and lifecycle semantics remain unverified.
- [ ] No actual cached Blizzard consumer of these four evaluation methods was found; focused API tests are not real consumer proof.

## Out of scope

Interpolation redesign, later-only `EvaluateTotalDuration`, new curve APIs, vendor edits, secret/security enforcement, and VM changes. Existing unsupported curve types remain unsupported and must report their existing errors rather than returning zero.
