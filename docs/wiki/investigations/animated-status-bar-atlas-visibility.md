# Animated status bar atlas visibility

Forever XP/reputation bars showed blue gain/level-up flipbook textures while no animation group was playing. The failing boundary was `SetAtlas` on XML-hidden child textures, not XML inheritance or animation completion.

## Evidence and fix

- The unchanged cached `Blizzard_FrameXMLBase/GradualAnimatedStatusBar.xml` defines `GainFlareAnimationTexture` and `LevelUpTexture` with `hidden="true"`. A normally loaded `GradualAnimatedStatusBarTemplate` instance starts with both hidden and four animation groups idle.
- Calling the unchanged `GradualAnimatedStatusBarMixin:SetAnimationTextures` immediately showed the first texture before any animation played. This reproduced the failure in `/tmp/wow-xml-hidden-atlas.log` (RED).
- `src/lua_api/frame/methods/widgets/texture/atlas.rs` accepted any `parentKey` as a button texture slot. `apply_atlas` then set visibility from `button_texture_should_show`, whose default for nonstandard keys is `true`. Both status-bar child textures therefore became visible on atlas assignment.
- `83a43ca61` restricts button-slot propagation and visibility updates to recognized keys on actual Button/CheckButton parents. Custom-key and non-button child textures keep their visibility. Actual-source animation Play/OnFinished still shows and hides its texture normally. Focused six-test GREEN: `/tmp/wow-xml-atlas-green.log`.

This proves the isolated producer path, not the final full UI screenshot. A later CheckButton control invocation was blocked by a concurrently edited, unrelated integration test compile error; it supplies no new control proof.

## Sources

- [Texture atlas visibility contract](../../specs/texture-atlas-visibility.md) — required behavior and tests
- `src/lua_api/frame/methods/widgets/texture/atlas.rs` — changed producer
- `tests/xml_animation_group_onload.rs` — unchanged vendor XML/Lua reproduction and lifecycle assertions

## See Also

- [[xml-template-system]] — texture template creation
- [[texture-atlas]] — atlas resolution and rendering
