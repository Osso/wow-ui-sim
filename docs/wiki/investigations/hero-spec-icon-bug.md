# Hero Spec Icon Bug

Status: retired — the original report is stale in the current build.

## Original Claim

Hero talent spec icon rendered at the bottom-right of the talent panel instead of top-center.

## Investigation Result

The current build does not reproduce this bug. Five layers of evidence ruled it out:

1. **dump-tree**: `HeroTalentsContainer` at `x=711, y=61`; `HeroSpecButton` at `x=752, y=151` — correct top-center position.
2. **Layout rects match**: `layout_rect` on every frame in the hero subtree matches freshly computed rects after `ensure_layout_rects()`.
3. **Quad emission matches layout rect**: `tests/hero_talents_render.rs` verifies `Icon1` emits exactly one textured quad whose vertex bounds exactly match the layout rect (x=481.58, y=111.41, w=60.83).
4. **Atlas crop correct**: Emitted crop request matches atlas DB entry for `talents-heroclass-paladin-lightsmith` exactly.
5. **Hiding `HeroTalentsContainer`** only changes the top-center region — the old bottom-right point `(1000, 610)` is unaffected and resolves only to `framegeneral/ui-background-marble`.

## Forever 70707 regression (2026-11-27)

This is separate from the retired bottom-right report. **VERIFIED (simulator at `0381a4920`, diagnostic at `44b67f003`):** the original HeroSpecButton had the correct `talents-heroclass-paladin-lightsmith` atlas path and crop `(0.395020, 0.492676, 0.198242, 0.393555)`, but the quad used cached y=235.21992 instead of the resolved y=103.84163 (x=471.46042, size=81.07919). The read-only quad builder consumed cached layout after the test requested only strata order. `SimState::get_strata_buckets` now resolves pending layout before returning the render buckets; this also supplies current geometry to the mask and clip paths. The atlas catalog and selection are unchanged.

**VERIFIED (simulator):** the XML placeholder `Interface\Icons\INV_Misc_QuestionMark` (134400) survived in `texture_file_data_id` after successful `SetAtlas`, even though the path and UVs were replaced. Successful resolved-atlas assignment now discards that old ID, matching the existing source representation of a fresh atlas texture. The native return shapes of `GetTexture` and `GetTextureFileID` on an atlas-backed texture remain **UNVERIFIED**; the test checks that neither getter reports the replaced placeholder ID, not a specific native return type.

**VERIFIED (Forever GPU test):** after layout repair the crop existed, but the old isolated render omitted a black 50%-alpha quad at the sampled point. The selected `Camelot/ClassTalents/Blizzard_ClassTalentsFrame.lua:183-190` shows `DisabledOverlay` when the spec-group configuration query has no match; the [existing contract](../../specs/forever-spec-group-config.md) specifies an empty default collection. `Camelot/ClassTalents/Blizzard_ClassTalentsFrame.xml:166` sets alpha .5; `Blizzard_SharedTalentUI/Blizzard_SharedTalentFrame.xml:79-89` creates its black texture. The runtime probe identified the visible overlay within `PlayerSpellsFrame.TalentsFrame`, not the hidden hero selection dialog. The old visual fixture also copied only the first quad of multi-quad texture requests and only one mask. The corrected fixture selects overlapping draw triangles in order, remaps all texture and mask ranges to a smaller batch, and renders that batch in an independent atlas. The original five sample coordinates, source alpha ≥200, and RGB tolerance 12 are unchanged. All five full/isolated RGBA values matched exactly at `63239f9f7`; top-center was `[57, 86, 78, 255]` in both.

## Frame Hierarchy

`HeroTalentsContainer` is a sibling of `ButtonsParent` (not a child), anchored to its TOP. `UpdateSpecBackground()` uses a 4-arg `SetPoint("TOP", ButtonsParent, heroContainerOffset, 0)` — relativePoint defaults to "TOP".

## What Was Ruled Out in the Original Report

- Stale layout_rect
- SetPoint parsing error (4-arg form correctly parsed)
- UI_SCALE transform (is 1.0)
- Duplicate frames
- Pan offset (moves individual buttons, not ButtonsParent)
- clipChildren clipping (HeroTalentsContainer is a sibling, not a child of ButtonsParent)

## Debug Tool Added

`--dump-tree` flag on the `screenshot` subcommand.

## Sources

- [hero-spec-icon-bug.md](../../hero-spec-icon-bug.md) — historical investigation with test coverage
- [Render snapshot](../../../src/lua_api/state_render.rs) and [atlas assignment](../../../src/lua_api/frame/methods/widgets/texture/atlas.rs) — Forever regression fixes
- [Quad bounds tests](../../../tests/hero_talents_render.rs) and [GPU isolation test](../../../tests/hero_talents_render_visual.rs) — original consumer regressions

## See Also

- [[class-talents-artifact]] — separate visual oddity near lower-right, ruled out as screenshot encoding artifact
