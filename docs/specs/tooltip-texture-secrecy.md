# Tooltip texture insertion and text secrecy lifetime

Bounded contracts for Retail audit rows `prose-2026-03-31-136` (texture insertion must not incorrectly mark tooltips secret) and `prose-2026-03-25-075` (secret tooltip text lines must not remain secret permanently), from [the registered patch source](../../data/patch-api/sources/12.0.5-api-changes.txt). These are distinct statements, not duplicate wording. Producers live in `src/lua_api/frame/methods/widgets/tooltip/` and `src/lua_api/frame/methods/text_attribute_event/text.rs`; see [frame data flow](../frame-data-flow.md). This is authored, unexecuted coverage, not audit acceptance.

## What it must do

### Texture insertion — March 31

- [ ] An ordinary file-data-ID texture added through the registered `AddTexture` appends a line without making existing public tooltip text unreadable to an addon.
- [ ] An ordinary named atlas added through the registered `AddAtlas` appends a line without making existing public tooltip text unreadable to an addon.
- [ ] Texture and atlas insertion preserve an existing secret-origin text read restriction; neither operation declassifies private content or loses its secure-readable text.
- [ ] After texture lines are cleared, public text replacement and subsequent texture insertion remain readable; old texture history does not poison replacement content.
- [ ] Operations on one tooltip do not change another tooltip's text readability or secrecy.

### Text-line lifetime — March 25

- [ ] Replacing a cached tooltip FontString's authentic secret text with public text restores addon readability without declassifying a different still-secret line.
- [ ] Hiding/re-showing a tooltip or adding textures does not declassify unchanged secret text in cached line FontStrings.
- [ ] Clearing and repopulating a tooltip reuses its cached FontString with readable new public text, not the historical secret-origin denial.

The oracle is actual `GetText` success/content or secret-origin rejection under a genuinely tainted closure, with secure content and authentic VM-wrapper positive controls. `HasSecretValues`, a manually toggled prevention flag, and secret-wrapping the tooltip object are not substitutes for content secrecy.

## How it works

- [Lua API boundary](../lua-api.md)
- [Frame data flow](../frame-data-flow.md)
- [Adjacent UnitBuff input contract and unmodeled output secrecy](tooltip-unit-buff-security.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/tooltip.rs` — registers tooltip methods.
- `src/lua_api/frame/methods/widgets/tooltip/line_frames.rs` — appends texture/atlas lines and synchronizes cached FontStrings.
- `src/lua_api/frame/methods/widgets/tooltip/line_data.rs` — appends text lines and clears tooltip content.
- `src/lua_api/frame/methods/widgets/tooltip/sizing.rs` — updates geometry without changing secrecy flags.
- `src/lua_api/frame/methods/text_attribute_event/text.rs` — tracks secret-origin SetText input and gates GetText; public replacement resets the origin.
- `src/lua_api/frame/methods/secret_origin.rs` — authenticates wrapper inputs and denies tainted secret-origin readouts.
- `src/lua_api/frame/methods/misc/secret.rs` — queries explicit/prevention flags, not content-secret text.
- `src/lua_api/tooltip.rs` — tooltip/line payloads currently have no content-secrecy fields.

## Tests asserting this spec

`tests/tooltip_texture_secrecy.rs` contains six behavioral tests using registered methods, authenticated host-secret strings, concrete text, insertion line counts, pooled FontString identity, and actual caller taint. Gated by both `retail-12-0-5` and `forbidden-aspects` to avoid testing undecoded secret placeholders. Current `client-retail` enables both; historical 12.0.5-only builds do not.

## Known gaps (current cycle)

- [ ] Compile and run authored tests; execution is forbidden in this authoring task. No RED/GREEN or compile-success claim.
- [ ] Resolve the source-observed cached-line reuse bug: ClearLines does not clear cached child secret origins; later public synchronization retains the stale flag. Proposed bounded replacement is in the task handoff, not applied.
- [ ] Historical Retail 12.0.5-only secret-origin text ingestion is unmodeled because wrapper decoding requires the later forbidden-aspects capability. These tests cannot grant exact historical-epoch acceptance.
- [ ] General TooltipLine/AddLine/DTO secret-text ingestion and aggregate tooltip-content secrecy remain unmodeled. Passing public-only flag tests would not close these gaps.

## Out of scope

- Native-client parity, loaded Blizzard tooltip processing, and restricted C_TooltipInfo output propagation: not established by these fixtures.
- Secret texture/atlas arguments: source sentence addresses accidental classification from insertion, not their acceptance policy.
- Explicit secret-aspect declarations, prevention flags, geometry/visibility secrecy, and rendering parity: independent contracts.
- Broad tooltip pooling redesign or API-epoch backports: require separate scope; the handoff proposes only cached text cleanup at ClearLines.
