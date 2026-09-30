# FontString smooth scaling

Retail 12.0.5+ `GetSmoothScaling` / `SetSmoothScaling` configure a per-FontString text-height policy. Cached `Blizzard_APIDocumentationGenerated/SimpleFontStringAPIDocumentation.lua` declares a boolean getter and setter (`SecretArguments = AllowedWhenUntainted`): true avoids whole-number text-height snapping for scaled FontStrings. Cached `Blizzard_SharedXML/UI.xsd:775` declares `smoothScaling`, boolean, XML default false; current NamePlates XML uses it. Public creation default and exact shaping semantics remain inferred, not native-proven.

## What it must do

- [ ] Round-trip a boolean independently per FontString, return one getter result and no setter results, initialize false (inferred public-creation policy).
- [ ] Reject missing/nil/nonboolean inputs before mutation. Authenticated secret booleans require an untainted caller; ordinary booleans remain usable by tainted callers. Getter returns configuration, not input secrecy.
- [ ] Keep `FontStringScaleAnimationMode` independent.
- [ ] False preserves existing `ceil(fontSize * 1.2)` shaping height. True uses fractional `fontSize * 1.2` with no minimum whole-pixel clamp. This multiplier and ceiling choice are simulator policy, not a claim about native font metrics or native rounding direction.
- [ ] Apply policy consistently to shared font measurement, regular and segmented glyph layout, wrapping, and shape-cache identity. Changing mode refreshes auto text height even for changes smaller than half a pixel.
- [ ] Apply verified XML `smoothScaling` values in ordinary and runtime-template FontString creation, including inherited FontString templates.

## How it works

- [Rendering pipeline](../wiki/systems/rendering-pipeline.md)
- [FontString spacing](fontstring-spacing.md)

## Implementation inventory

Pending required pre-implementation RED. Intended paths: `src/widget/{frame,frame_defaults}.rs`, text style/measurement methods, `src/xml/types_elements.rs`, `src/loader/xml_fontstring.rs`, FontString template fields, `src/render/{font,glyph}.rs`, `src/render/glyph/text_emit.rs`, `src/iced_app/quad_builders.rs`.

## Tests asserting this spec

- `tests/fontstring_smooth_scaling.rs` — public API, errors, secrets, height/auto-height, wrapping, XML ordinary/runtime creation.
- `src/iced_app/quad_builders_tests.rs::smooth_scaling_render_measure_and_cache_flips` — font size 12, effective scale 1.1, emitted two-line separation, mode flips with shared glyph cache, scaled smooth measurement agreement.

Tests-only revision `03ac97b97`; RED pending shared Cargo slot release. No native proof.

## Known gaps (current cycle)

- [ ] Run targeted RED, implement, and run targeted GREEN after parent releases Cargo ownership.

## Out of scope

Native font metrics, default/validation parity, glyph bitmap snapping, animation-mode execution, unrelated text scaling redesign, GPU-pixel/native visual parity, vendor changes, and non-FontString domains. Legacy false measurement shapes in local units before multiplying text scale, while rendering shapes at effective scale; existing false scale-rounding differences are not redesigned. True's unsnapped linear height supports local-to-screen scale agreement. Future native recorder: [SmoothScalingProbe](../addons/SmoothScalingProbe/README.md).
