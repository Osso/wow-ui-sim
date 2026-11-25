# Video capabilities

These inputs supply Forever graphics capabilities to the original Settings consumer. The simulator does not query the renderer or device for them.

## Image-based antialiasing

**VERIFIED — Forever 1.60.1.70707 (R88fy):** `AntiAliasingSupported()` returns three booleans. The first is always `true`; the next two come from runtime renderer/device helpers. **INFERRED — Forever 1.60.1.70707 (R88fy):** the cached `Graphics.lua` consumer maps those helper results to CMAA and CMAA2 options; the source-build identity was not verified in the answer.

**INFERRED — simulator default:** both supplied CMAA values default to `false`. The native answer does not establish the second helper's initial result or a universal runtime default.

`A_Admin.SetAntiAliasingSupported(cmaa, cmaa2)` supplies the second and third return values. The first return value remains the verified constant `true`.

```lua
A_Admin.SetAntiAliasingSupported(true, false) -- offer CMAA, but not CMAA2
```

## Multisample antialiasing

**VERIFIED — Forever 1.60.1.70707 (R88fy):** `MultiSampleAntiAliasingSupported()` returns three values for each supported level and returns no values when no candidate passes. **INFERRED — Forever 1.60.1.70707 (R88fy):** the cached Settings consumer uses each triple as MSAA quality, sample count, and coverage count; its source-build identity was not verified in the answer.

**INFERRED — simulator default:** the supplied ordered group list is empty. This does not claim that native MSAA is unsupported before renderer initialization; the native answer marks that state unknown.

Supply groups in display order. `msaaQuality` is passed to the original consumer's `SplitMSAA`; `sampleCount` and `coverageCount` are formatted with `ADVANCED_ANTIALIASING_MSAA_FORMAT`.

```lua
A_Admin.SetMultiSampleAntiAliasingSupported({
    { msaaQuality = "1,0", sampleCount = 4, coverageCount = 0 },
    { msaaQuality = "2,2", sampleCount = 8, coverageCount = 2 },
})
```
