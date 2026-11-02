# Textures and text: source-backed rework

All entries are **static discrepancies**, not executed reproductions or native-client verdicts. [Pinned revisions and profile limits](README.md) apply. External paths start at `~/Repos/wowdecomp`; simulator paths start at the project root.

## T1 — Reject invalid texture-coordinate arity and range before mutation

**External evidence:** `Engine/Source/Frame/CSimpleTexture_Scripts.cpp:455-503`, `Script_SetTexCoord`, claimed. Counts supplied arguments, accepts exactly four or eight, and errors for other counts. Every converted coordinate must satisfy comparisons against `-10000` and `10000` before `SetTexCoord` is called. The method is exposed by the texture method table in the same file.

**Simulator difference:** `src/lua_api/frame/methods/widgets/texture/coords.rs:39-60` collects numeric values from slots 2–9 with `filter_map`, then silently ignores counts other than four/eight. It neither checks the actual supplied arity nor rejects out-of-range coordinates. Extra arguments after slot 9 are ignored.

**Observable examples:** after setting ordinary UVs, `t:SetTexCoord(0, 1, 0)` returns without an error; `t:SetTexCoord(-10001, 1, 0, 1)` stores an out-of-range value. The reconstruction errors before mutation. Nine numeric coordinates are accepted as the first eight by the simulator but rejected by the reconstruction.

**Required change:** validate supplied arity and converted coordinates before touching UV state. Preserve four/eight-corner support and existing atlas remapping. Do not infer strict numeric-type or finite-number validation: the reconstruction uses `lua_tonumber`, and its comparisons do not themselves reject NaN. Resolve coercion/non-finite policy separately against the target client contract.

**Regression:** establish previous UVs, assert `pcall` failure for three/five/nine arguments and values just beyond each bound, assert previous coordinates unchanged; include exact-boundary and ordinary four/eight-value controls. Existing `tests/texture_atlas_regressions.rs:23-151` exercises valid UV storage/remapping, not proof of these rejection cases. No tests run here.

**Applicability:** historical 3.3.5a script-contract evidence; confirm target-profile error policy before implementation.

## T2 — Blend-mode validation and rendering must agree

**External evidence:** `Engine/Source/Frame/CSimpleTexture_Scripts.cpp:85-102`, `Script_SetBlendMode`; `Engine/Source/FrameXML/LoadXML.cpp:55-62,197-207`, `StringToBlendMode`. Recognized names, case-insensitively: `DISABLE`, `BLEND`, `ALPHAKEY`, `ADD`, `MOD`. Unknown/missing input errors rather than modifying the texture. The setter forwards the selected `EGxBlend`, not one catch-all alpha mode.

**Simulator difference:** `src/lua_api/frame/methods/widgets/texture/coords.rs:9-35` accepts arbitrary strings, returns silently for missing/non-string input, stores the name for `GetBlendMode`, and maps only `ADD` to additive; **every other name maps to Alpha**. `src/lib.rs:128-137` models only Alpha and Additive. Thus `GetBlendMode()` can report `MOD`, `DISABLE` or `ALPHAKEY` while the backing rendering choice remains ordinary alpha.

**Observable examples:** `t:SetBlendMode("NOT_A_BLEND")` succeeds and can be read back. Valid `MOD` and `BLEND` select the same simulator rendering enum despite distinct source enum choices. Exact GPU equations/pixels were not compared in this audit; the demonstrable issue is validation plus collapse of distinct modes.

**Required change:** implement the target profile's accepted vocabulary and rejection boundary; represent and render its actual distinct modes rather than merely retaining their names. Trace the source graphics backend and target-profile evidence before assigning exact blend equations or alpha-key thresholds. Do not add permissive fallback-to-alpha behavior.

**Regression:** reject missing/invalid inputs without changing prior state; test case-insensitive accepted names; add deterministic overlapping foreground/background pixel fixtures for each supported mode so a name-only implementation cannot pass. Existing alpha/additive colour tests in `src/render/headless_colour_tests.rs` are adjacent coverage, not proof of the other three modes. No GPU/tests executed here.

**Applicability:** five-name vocabulary is reconstructed 3.3.5a evidence, not a complete modern vocabulary. Preserve any separately established modern-profile contract.

## T3 — Indented wrapping must reach layout/rendering, not private getter storage

**External evidence:** `Engine/Source/Frame/CSimpleFont.cpp:299-307` consumes XML `indented`; `Engine/Source/Frame/CSimpleFont_Scripts.cpp:377-390` implements the Font object's setter. Both set style bit `0x20000`. `Engine/Source/Services/TextBlock.cpp:25-74`, `ConvertStringFlags`, explicitly converts that bit to renderer flag `0x2000`. `Engine/Source/Gxu/GxuFontString.cpp:822-824,854-862` reduces continuation-line width and offsets non-first lines for left/right justification. This closes the flag-conversion trace; it does not rely on the anomalous FontString registration discussed in the [index](README.md#reconstruction-counterexample).

**Simulator difference:** `src/lua_api/globals/font_strings_collection/fonts.rs:256-270,301-320` stores/copies `__indentedWordWrap` in Font tables. `src/lua_api/frame/methods/text_attribute_event/text/style.rs:91-108` stores an independent private key for a text-type argument; it does not set render/layout state. `src/xml/types_elements.rs:233-280` and `src/loader/xml_fontstring.rs:51-84` have no corresponding indented-wrap setting. A source-wide search found these private fields only in Lua storage/copy/getter paths, not renderer consumers.

**Observable fixture:** apply an indented Font to a width-constrained, left-justified string long enough for at least three wrapped lines. Compare false/true using the Font object (or equivalent Font XML inheritance). Reconstruction's continuation lines have an inset and less available width; simulator private booleans cannot affect emitted layout. This is a source-derived fixture, not a rendered comparison performed here.

**Required change:** propagate the effective inherited/local setting into text measurement and rendering, including XML parsing and invalidation. Keep FontString, Font and SimpleHTML argument forms distinct; do not reproduce the reconstruction's swapped FontString getter/setter registrations. Determine pixel-unit scaling and supported-profile behavior before fixing an inset constant.

**Regression:** render/measure multiline false/true controls via Font inheritance and XML; assert continuation x-offset, reduced continuation width, line breaks, and matching getter state. Preserve default/false layout and existing text wrapping/spacing contracts. Existing private getter/setter state is not layout proof; no such proof was run here.

**Applicability:** static 3.3.5a Font/XML-to-render evidence; modern wrapping policy and exact metrics remain unverified.

## Not promoted to rework

| Candidate | Disposition |
|---|---|
| Four/eight-value UV support | Already implemented. T1 is rejection policy, not absent eight-corner support. |
| Rotation angle/pivot equivalence | Partial trace only; no exact pixel counterexample established. |
| Gradient numeric arguments versus colour-table arguments | Historical/modern API difference unresolved; do not replace modern contract merely to match 3.3.5a. |
| Horizontal/vertical tiling | Axis flags and tiling paths exist; permissive/default coercion not traced far enough to assert a defect. |
