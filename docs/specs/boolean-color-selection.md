# Boolean color selection

The ordinary-value `C_CurveUtil` selectors choose a color or color component from a boolean. The 12.0.0 occurrence register declares both functions; cached `CurveUtilDocumentation.lua` specifies their arguments and returns. Implementation belongs to `src/c_api/c_curve_util.rs`, alongside [curve objects](curve-objects.md).

## What it must do

- [x] Publish both selectors on retail 12.0.0 and later, without adding them to earlier client profiles (Mists preservation control).
- [x] `EvaluateColorFromBoolean(boolean, valueIfTrue, valueIfFalse)` selects the corresponding RGBA value and returns a ColorMixin-compatible color without mutating inputs.
- [x] `EvaluateColorValueFromBoolean(boolean, valueIfTrue, valueIfFalse)` selects the corresponding numeric component, including zero.
- [x] Simulator policy: return a fresh color so later result mutations do not alter input colors. Native identity/copy behavior is unverified.
- [x] Simulator validation policy: require a boolean condition, numeric components, and color tables containing numeric `r`, `g`, `b`, and `a` fields in both branches. Reject invalid inputs without mutating colors. Native coercion and errors are unverified.

## How it works

- [C API boundary](../lua-api.md) and [curve implementation](curve-objects.md).

## Implementation inventory

- `src/c_api/c_curve_util.rs` — ordinary selectors and profile-scoped publication.
- `tests/userdata_proxy.rs` — grouped behavior and earlier-profile preservation tests.

## Tests asserting this spec

- `tests/userdata_proxy.rs` — five `curve_boolean_*` cases passed on 12.0.0 with all 27 existing cases in that grouped module at `977d30276` (32/32). Independent controls passed 5/5 on 12.0.5, 5/5 on 12.0.7, and 1/1 on Mists. Format/check/build and current-retail startup passed; historical-profile warnings are pre-existing. Proof: `/tmp/verify-curve-boolean-ledger.json`, code/metadata at `be38a2c52`.

## Known gaps (current cycle)

- [ ] Native identity, coercion, validation order, channel range handling, and exact errors remain unverified.
- [ ] No direct unmodified Blizzard consumer was found in the current retail cache; proof is API-level, not consumer-level.

## Out of scope

- Secret values, taint, and caller-authority semantics are deferred.
- `EvaluateGameCurve` and other curve methods are separate contracts.
