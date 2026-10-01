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

- `src/widget/{frame,frame_defaults}.rs` — independent boolean state, initialized false.
- `src/lua_api/frame/methods/text_attribute_event/{mod,text}.rs`, `text/{style,metrics}.rs` — 12.0.5 API registration, checked secret inputs, measurement and exact auto-height refresh.
- `src/xml/types_elements.rs`, `src/loader/xml_fontstring.rs`, `src/lua_api/frame/methods/button_anchor_hierarchy/font_strings.rs` — verified attribute and ordinary/runtime/inherited FontString application, bounded to 12.0.5+.
- `src/render/{font,glyph}.rs`, `src/render/glyph/text_emit.rs`, `src/iced_app/quad_builders.rs` — fractional shared line height, smooth baseline-span measurement, regular/segmented emission and mode-keyed shape caches. False-mode multiline measurement keeps its existing first-baseline contribution; smooth measurement matches rendering's baseline span.
- `src/iced_app/{message_frame_render,tooltip}.rs` — explicitly retain false on unrelated text paths.

## Tests asserting this spec

- `tests/fontstring_smooth_scaling.rs` — public API, errors, secrets, height/auto-height, wrapping, XML ordinary/runtime creation.
- `src/iced_app/quad_builders_tests.rs::smooth_scaling_render_measure_and_cache_flips` — font size 12, effective scale 1.1, emitted two-line separation, mode flips with shared glyph cache, scaled smooth measurement agreement.
- `src/iced_app/quad_builders_tests.rs::smooth_scaling_wrapped_render_and_cached_measurement_agree` — supplemental post-implementation assertions for real wrapped glyph output, cache-backed measured height and public smooth height through repeated mode flips; not independently observed pre-change RED.

Valid pre-implementation RED at build revision `a3ba2a23a1567150ca239efcec39fa6f43918257`: six API cases and one renderer case fail on absent methods. The first attempted build failed on renderer fixture mistakes; tests-only `3c4cffda6` corrected borrowed-Frame use and float literal syntax. Logs: `/tmp/patch-12.0.5-font-{api,render}-red.log`; build identity `/tmp/patch-12.0.5-corrected-pin-red-build.json`. Renderer RED reaches the missing setter, not its later fractional/cache assertions. The multiline expected value was subsequently corrected from legacy-baseline height to the chosen smooth baseline-span policy. Targeted GREEN pending parent-owned shared batch; no native proof.

## Known gaps (current cycle)

- [ ] Obtain targeted GREEN from parent-owned shared batch.

## Out of scope

Native font metrics, default/validation parity, glyph bitmap snapping, animation-mode execution, unrelated text scaling redesign, GPU-pixel/native visual parity, vendor changes, and non-FontString domains. XML smooth scaling predates the API (12.0.0 according to the cached patch source), but this implementation enables both only in 12.0.5+; earlier XML behavior remains a separate coverage gap rather than publishing the Lua methods early. Legacy false measurement shapes in local units before multiplying text scale, while rendering shapes at effective scale; existing false scale-rounding differences are not redesigned. True's unsnapped linear height supports local-to-screen scale agreement. Future native recorder: [SmoothScalingProbe](../addons/SmoothScalingProbe/README.md).
