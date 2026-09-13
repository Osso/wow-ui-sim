# Curve object identity and aura options

## Required behavior

- `C_CurveUtil.CreateCurve()` and `CreateColorCurve()` return userdata handles, as specified by the cached `LuaCurveObjectAPIDocumentation.lua` and `LuaColorCurveObjectAPIDocumentation.lua` (`ObjectType = "Userdata"`).
- `securecopy` copies surrounding option tables without copying curve handles or losing their methods.
- `Copy()` creates a distinct handle with independent points and the same interpolation configuration. Per-instance addon fields remain writable; methods remain read-only.
- Aura option processors accept registered curve handles of the required kind, not tables impersonating curves with `Evaluate` and `Copy` fields.
- Numeric linear interpolation remains unchanged. Color linear interpolation operates on RGBA channels, returning a ColorMixin value. Color Step interpolation holds the preceding point and switches at the next point, covering BetterBlizzFrames' duration-color initialization.

- `GetType()` returns the stored interpolation mode as one number for scalar and color curves: default Linear (`0`), changes to Step (`1`) and back, and independently copied mode state. This is a best-effort simulator storage contract, not proof of native defaults or non-linear evaluation.

- Color curves expose `GetPoint(index)` as one point record `{x, y}` or one nil. The simulator uses one-based stored order, returns nil for an unconfigured ordinary integer index, and copies the record and RGBA color so returned-data mutation cannot alter curve state. These indexing and copy rules are best-effort policies; the pinned 12.0.0 contract establishes a numeric index and nilable `LuaColorCurvePoint`, not their complete native semantics.

- Color curves expose `GetPoints()` as one array of `{x, y}` records, empty when no points are configured. Stored insertion order and fresh array/record/RGBA snapshots are simulator policies. Returned-data mutation cannot change curve evaluation; snapshots survive Copy/Clear/reuse. The pinned contract establishes an array of `LuaColorCurvePoint`, not native ordering or identity rules.

- Color `RemovePoint(index)` removes a configured point with zero return values. Valid one-based indices compact remaining stored points and update subsequent evaluation without mutating copied curves. Indexing and compaction are simulator policies; the pinned contract establishes numeric input and no outputs, not invalid-index or coercion behavior.

## Proof boundary

`tests/userdata_proxy.rs` covers handle types, copying, secure option identity, numeric and color interpolation, and forged-table rejection. At `77310f806`, the existing scalar and color `Copy` tests prove one distinct userdata return, copied values, independent later point mutation, and independent interpolation-mode changes. The color fixture observes point counts `0 → 1 → 2 → 0`, copied `Step` configuration, and independent `Linear`/`Step` changes. `3d804b9c5` credits only `LuaCurveObjectBase.SetType` for those color `Step`/`Linear` changes and copied-curve isolation; it does not credit scalar modes, `GetType`, or the native base interface. `/tmp/verify-curve-settype-ledger.json` passes the metadata gate: one status credit, one added test reference, zero hash renewals, 14,803 fresh evidence references, and exact checklist/inventory matching at **2273 best-effort / 1135 evidence-required / 2 exceptions**. It reuses hash-matched 2/2 retail 12.0.0, 12.0.5, and 12.0.7 copy proof without Cargo reruns; format, readability, and unchanged production proof remain valid. Historical warnings remain 6/6/1. Strengthened Copy assertions were not run on PTR or Mists. `tests/duration_text_binding_copy.rs` exercises the real `CustomAuraButton` provider initializer and its internal `securecopy(options)` before assigning a duration color curve.

The test-only commit reproduces the actual processor rejection, not merely a type mismatch. Targeted GREEN passed 9/9 in `/tmp/pi-curve-userdata-green.*`; actual addon/SavedVariables startup then returned `[]`, exit 0 in `/tmp/pi-accepted-final-startup.*`.

Getter proof at `b241590df`: scalar/color single numeric mode, default Linear, Step/Linear changes, and independent Copy modes pass in grouped userdata tests (34/34 each retail 12.0.0/12.0.5/12.0.7; 30/30 Mists). `/tmp/verify-curve-get-type-ledger.json` also records format/check/build, startup `[]`, readability and metadata validation. Earlier SetType/Copy entries above retain their historical proof scope.

Color `GetPoint` development: `6b7bd754f` reproduces three missing-method failures; `d2e29052f` implements retrieval; `3c5ecaefa` corrects an unavailable fixture-only `SetRGBA` call to direct channel mutation. Final retail 12.0.0 proof passes 3/3 for fractional points, return count, missing-index nil, output mutation isolation with unchanged evaluation, and Copy/Clear/reuse. Independent verification at `539f9151f` passes grouped userdata tests 37/37 each retail 12.0.0/12.0.5/12.0.7 and 33/33 Mists; format/check/build, startup `[]`, readability, both validators and 14,831 evidence hashes pass. Ledger: `/tmp/verify-color-curve-get-point-ledger.json`. Scalar `GetPoint` has a separate vector contract and remains unimplemented by this slice.

Color `GetPoints`: tests `4fae6e8af` reproduce the missing method (RED 0/3); runtime `ba2d6e2a0` passes retail 12.0.0 GREEN 3/3. `/tmp/color-curve-get-points-development-ledger.json` records one-array cardinality, empty/populated order, fractional values, mutation isolation, and Copy/Clear/reuse. Independent verification at `221869fde` passes grouped userdata tests 40/40 each retail 12.0.0/12.0.5/12.0.7 and 36/36 Mists; format/default check/build, startup with zero Lua errors, readability, both validators and 14,834 evidence hashes pass. Ledger: `/tmp/verify-color-curve-get-points-ledger.json`. Only the collection getter receives credit; scalar/vector and point-structure rows remain open.

Color `RemovePoint`: tests `84d091201` reproduce the missing method (RED 0/3); runtime `17bad77a3` passes retail 12.0.0 GREEN 3/3. Tests assert zero returns, valid first/middle/last removal, compacted order/count, empty state, changed evaluation, and independent copied state. Ledger: `/tmp/color-curve-remove-point-development-ledger.json`. Independent verification pending; invalid inputs and native semantics remain open.

## Remaining limits

This slice does not claim complete native curve semantics. Numeric non-linear modes retain their previous behavior. Color Cosine/Cubic interpolation and evaluation below the first configured point explicitly report unsupported behavior. Existing segment ordering and upper endpoint selection remain; native extrapolation modes, duplicate-point ordering, secrecy, and unimplemented curve methods are not established here.

## Implementation

`src/c_api/c_curve_util.rs` owns curve userdata, private backing state, and the separately scoped ordinary [boolean color selectors](boolean-color-selection.md). The previous table factories are removed from the temporary proxy factory module. See [aura option processing](aura-container-options.md) and [duration binding](duration-text-binding.md).
