# Adventure Guide Layout

Adventure Guide Suggested Content cards can overlap text and buttons when runtime geometry queries resolve a resized frame without also resolving frames anchored to it.

## Content

Blizzard's `EJSuggestFrame_RefreshDisplay` resizes the title and description frames, then immediately reads `GetTop()` and `GetBottom()` to recenter each card's `centerDisplay`. The secondary cards chain their layout through anchors:

- `title`
- `description`, anchored to `title:BOTTOMLEFT`
- `button`, anchored to `description:BOTTOMLEFT`

The simulator's synchronous dirty-rect fast path resolved only the queried frame when Lua called geometry methods such as `GetBottom()`. If `title:SetHeight()` made the title dirty, `title:GetBottom()` returned the new title rect, but dependent siblings like `description` still kept their old cached rects. That made Adventure Guide text and buttons overlap even though the Blizzard XML anchors were correct.

The fix is for `SimState::resolve_rect_if_dirty` to use the anchor-dependent layout path for directly dirty frames, and for dirty ancestor resolution to recompute anchor dependents before clearing dirty roots. The hit-grid incremental queue must include those same transitive anchor dependents, because moved sibling buttons are not descendants of the resized anchor target. Regression coverage lives in `tests/layout_resolve.rs` as `querying_resized_anchor_target_updates_dependent_siblings`.

Follow-up: the cards could still overlap after the stale-rect fix because `GetNumLines` was registered by the tooltip widget methods after the general text methods on the shared FrameRef metatable. Non-tooltip FontStrings therefore called the tooltip line counter and returned `0`, so Blizzard sized title/description containers to zero lines. `GameTooltip` keeps tooltip-backed line counting, while regular FontStrings now compute line count from measured wrapped text height. Regression coverage for wrapped line counts lives in `tests/fontstring_smooth_scaling.rs` as `smooth_scaling_wrap_keeps_line_count_and_fractional_height`.

Follow-up: the card portraits had two separate source issues. Blizzard calls `Texture:SetMask("Interface\\CharacterFrame\\TempPortraitAlphaMask")` for the circular card portraits, but the simulator's `SetMask` method was still a no-op, so icons rendered square instead of clipping to the gold circle. The seeded `C_AdventureJournal` suggestions also used two missing texture paths (`Achievement_Raid_Nerubian` and `Inv_Archaeology_70_Scroll`), which left the Nerub-ar Palace and Delves portraits blank. `SetMask` now creates a real mask texture wired through the existing mask renderer, and the seed icons use manifest-backed 64x64 paths.

## Sources

- [state_render.rs](../../../src/lua_api/state_render.rs) - synchronous dirty rect resolution and anchor-dependent recomputation
- [text.rs](../../../src/lua_api/frame/methods/text_attribute_event/text.rs) - FontString `GetNumLines` measurement
- [tooltip.rs](../../../src/lua_api/frame/methods/widgets/tooltip.rs) - GameTooltip-specific `GetNumLines` dispatch
- [rotation_mask.rs](../../../src/lua_api/frame/methods/widgets/texture/rotation_mask.rs) - `Texture:SetMask` mask texture creation
- [encounter_journal.rs](../../../src/lua_api/globals/missing_surface/encounter_journal.rs) - Adventure Journal seed suggestion data
- [layout_resolve.rs](../../../tests/layout_resolve.rs) - regression coverage for resized anchor targets updating sibling dependents
- [fontstring_smooth_scaling.rs](../../../tests/fontstring_smooth_scaling.rs) - regression coverage for FontString wrapped line counts
- [methods_texture.rs](../../../tests/methods_texture.rs) - regression coverage for `Texture:SetMask`
- [c_encounter_journal_probes.rs](../../../tests/c_encounter_journal_probes.rs) - regression coverage for Adventure Journal suggestion icon paths
- [Blizzard_EncounterJournal.xml](</syncthing/World of Warcraft/_retail_/BlizzardInterfaceCode/Interface/AddOns/Blizzard_EncounterJournal/Mainline/Blizzard_EncounterJournal.xml>) - Adventure Journal Suggested Content card anchors
- [Blizzard_EncounterJournal.lua](</syncthing/World of Warcraft/_retail_/BlizzardInterfaceCode/Interface/AddOns/Blizzard_EncounterJournal/Mainline/Blizzard_EncounterJournal.lua>) - runtime text sizing and geometry queries

## See Also

- [[layout-system]] - anchor resolution and dirty layout behavior
- [[adventure-guide-boss-icons]] - separate Encounter Journal image fallback issue
