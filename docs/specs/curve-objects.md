# Curve object identity and aura options

## Required behavior

- `C_CurveUtil.CreateCurve()` and `CreateColorCurve()` return userdata handles, as specified by the cached `LuaCurveObjectAPIDocumentation.lua` and `LuaColorCurveObjectAPIDocumentation.lua` (`ObjectType = "Userdata"`).
- `securecopy` copies surrounding option tables without copying curve handles or losing their methods.
- `Copy()` creates a distinct handle with independent points and the same interpolation configuration. Per-instance addon fields remain writable; methods remain read-only.
- Aura option processors accept registered curve handles of the required kind, not tables impersonating curves with `Evaluate` and `Copy` fields.
- Numeric linear interpolation remains unchanged. Color linear interpolation operates on RGBA channels, returning a ColorMixin value. Color Step interpolation holds the preceding point and switches at the next point, covering BetterBlizzFrames' duration-color initialization.

- `GetType()` returns the stored interpolation mode as one number for scalar and color curves: default Linear (`0`), changes to Step (`1`) and back, and independently copied mode state. This is a best-effort simulator storage contract, not proof of native defaults or non-linear evaluation.

## Proof boundary

`tests/userdata_proxy.rs` covers handle types, copying, secure option identity, numeric and color interpolation, and forged-table rejection. At `77310f806`, the existing scalar and color `Copy` tests prove one distinct userdata return, copied values, independent later point mutation, and independent interpolation-mode changes. The color fixture observes point counts `0 → 1 → 2 → 0`, copied `Step` configuration, and independent `Linear`/`Step` changes. `3d804b9c5` credits only `LuaCurveObjectBase.SetType` for those color `Step`/`Linear` changes and copied-curve isolation; it does not credit scalar modes, `GetType`, or the native base interface. `/tmp/verify-curve-settype-ledger.json` passes the metadata gate: one status credit, one added test reference, zero hash renewals, 14,803 fresh evidence references, and exact checklist/inventory matching at **2273 best-effort / 1135 evidence-required / 2 exceptions**. It reuses hash-matched 2/2 retail 12.0.0, 12.0.5, and 12.0.7 copy proof without Cargo reruns; format, readability, and unchanged production proof remain valid. Historical warnings remain 6/6/1. Strengthened Copy assertions were not run on PTR or Mists. `tests/duration_text_binding_copy.rs` exercises the real `CustomAuraButton` provider initializer and its internal `securecopy(options)` before assigning a duration color curve.

The test-only commit reproduces the actual processor rejection, not merely a type mismatch. Targeted GREEN passed 9/9 in `/tmp/pi-curve-userdata-green.*`; actual addon/SavedVariables startup then returned `[]`, exit 0 in `/tmp/pi-accepted-final-startup.*`.

## Remaining limits

This slice does not claim complete native curve semantics. Numeric non-linear modes retain their previous behavior. Color Cosine/Cubic interpolation and evaluation below the first configured point explicitly report unsupported behavior. Existing segment ordering and upper endpoint selection remain; native extrapolation modes, duplicate-point ordering, secrecy, and unimplemented curve methods are not established here.

## Implementation

`src/c_api/c_curve_util.rs` owns curve userdata, private backing state, and the separately scoped ordinary [boolean color selectors](boolean-color-selection.md). The previous table factories are removed from the temporary proxy factory module. See [aura option processing](aura-container-options.md) and [duration binding](duration-text-binding.md).
