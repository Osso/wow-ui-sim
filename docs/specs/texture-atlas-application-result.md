# Texture atlas application result

`Texture:SetAtlas` application-result policy supports unchanged cached Blizzard TextureKit consumers. This is an inferred **simulator compatibility policy**, not established native return parity. Source: [atlas methods](../../src/lua_api/frame/methods/widgets/texture/atlas.rs); [rendering architecture](../texture-atlas-system.md).

## What it must do

- [x] A successfully resolved and applied atlas returns exactly one boolean `true`.
- [x] Like SetTexture, SetAtlas authenticates genuine secret-wrapper inputs before type validation, permits an untainted caller to decode them, and preserves secret-origin read restrictions until a public replacement. Retail 70448 original UnitFrameUtil arena-spec display exercises this path.
- [x] An accepted input that does not apply an atlas returns exactly one boolean `false`; existing argument errors and mutation behavior remain unchanged.
- [x] Real cached CaptureBar TextureKit consumers keep available neutral glow/spark textures visible, then consume updates, hidden/restored records, removal and unregister without vendor changes.

## How it works

- [Texture atlas system](../texture-atlas-system.md).
- [UI widget supplied-state contract](ui-widget-manager-state.md).

## Implementation inventory

- `src/lua_api/frame/methods/widgets/texture/atlas.rs`: atlas argument validation, lookup and application.

## Tests asserting this spec

- `tests/set_atlas_arguments.rs`: exact return count, known/missing/inactive/restored application and existing argument-validation regression.
- `tests/ui_widget_manager_bars_consumer.rs`: actual cached Lua/XML initial geometry/visibility, updates, zero neutral zone, hidden/restore, removal and unregister.
- `tests/texture_atlas_regressions.rs`: ten mutation/button-slot controls; additional known/unknown/numeric atlas controls preserve their original scopes.

At `5bd208dba`, seventeen scoped cases pass (argument2, bar1, atlas-regression10, HUD2, lifecycle2); three separate known/unknown/numeric controls also pass. Fresh normal startup is clean across all219 stderr lines; default offline/locked check exits0 with six inherited manifest deprecations and no source warnings. [Retained evidence](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ui-widget-manager-state-2026-10-13/README.md#atlas-application-result-red-and-retail-green) owns exact receipts and historical failures. Native and parent acceptance remain OPEN.

## Known gaps (current cycle)

Historical genuine RED at `21aabeaf1`: existing argument validation PASS, application-result assertion FAIL with `actual=0`; cached bar assertion59 observes NeutralBar=true, Glow3=false and SparkNeutral=false despite expected atlases. Later phases were unreached in that attempt. Initial delimiter compilation failure at `d1c5b4cb8` executed zero cases and is not behavioral RED. Later Retail GREEN does not rewrite either failure.

- [x] Bounded PTR argument2/atlas-regression10 and actual cached bar1/HUD2 pass in separate executed scopes; [checkpoint](../wiki/investigations/integrated-source-and-factory-proof-2026-10-09.md#separate-ptr-and-owning-checkpoints--2026-10-13) owns receipts and reuse boundaries.
- [ ] Other-profile, native return parity and broader parent gates remain pending.

## Out of scope

- Native return parity: cached generated `SimpleTextureBaseAPIDocumentation.lua` has no Returns for SetAtlas; current Wowless `TextureBase.SetAtlas` schema has empty outputs. Cached `TextureUtil.lua::SetupTextureKitOnFrame` nevertheless assigns SetAtlas's result to `success` and uses it for `SetShown`. This conflict is retained, not resolved by invented native evidence.
- Vendor edits, atlas-catalog fabrication or unrelated texture behavior. Secret-wrapper authentication precedes the existing underlying atlas argument validation.
