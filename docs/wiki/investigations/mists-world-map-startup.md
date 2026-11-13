# Mists World Map Startup

Mists addon startup can fail when profile-scoped Blizzard cache files are absent, when XML paths escape through `Interface/AddOns`, or when Mists-era FrameXML mixes classic helper Lua with newer XML/templates. The fix is to load the real profile files where they exist, resolve escaped addon-root paths against the active cache, and keep narrow Mists compatibility only for helpers missing from the current source tree.

## Content

The visible failures were:

- `WorldMapFrame` / `WorldMapTrackQuest` nil errors from addons such as AllTheThings and Leatrix_Maps.
- `UpdateUIPanelPositions` nil from `Blizzard_WorldMap/Cata/Blizzard_WorldMap.lua`.
- `FogOfWarFrameMixin` nil while Leatrix loaded `Blizzard_BattlefieldMap`.
- `MapCanvasMixin:AddDataProvider()` calling `dataProvider:OnAdded()` on Mists providers that lacked copied base callbacks.

Root causes:

- The runtime cache had only the retail/Mainline WorldMap TOC, while Mists discovery skips `_Mainline` TOCs. The Mists profile needs `Blizzard_WorldMap_Mists.toc`, `Cata/Blizzard_WorldMap.*`, `Wrath/QuestLogOwnerMixin.lua`, `Blizzard_WorldMapTooltip.xml`, and the Mists `Blizzard_SharedMapDataProviders` TOC/files.
- Loading the real Mists `Blizzard_UIParentPanelManager` still reintroduces the `bottomEdgeExtent` layout failure, so WorldMap cannot depend on fully enabling that addon yet.
- Cata WorldMap calls panel-position globals (`UpdateUIPanelPositions`, `MaximizeUIPanel`, `RestoreUIPanelArea`) and expects `WorldMapLevelDropDown.header`.
- Mists shared map providers reference `FogOfWarFrameMixin`, but the Mists source tree does not ship a matching `FogOfWarFrameTemplates` implementation.
- `Blizzard_MapCanvas.lua` is copied into XML templates before post-load wrappers can affect it, so provider default callback backfills must happen in a source patch on `MapCanvasMixin:AddDataProvider()`.

Implementation notes:

- Cache membership is guarded in `src/blizzard_ui_sync/profile_cache.rs` so stale retail files do not satisfy the Mists profile.
- `src/mists/post_load.lua` owns Mists-only panel-position, FogOfWar, and WorldMap dropdown header compatibility.
- `src/lua_api/workarounds/temporary/source_patches.rs` owns the Mists `Blizzard_MapCanvas.lua` source patch that backfills provider callbacks before `OnAdded`.
- The final startup baseline after this fix kept only the unrelated Syndicator `tradeskill` and Baganator `TokenFramePopup` errors.

## Mists 5.5.4 lua-errors cleanup

A later Mists 5.5.4 refresh exposed a second set of startup roots. The key falsifier was that most visible nils were downstream of missing vendor files or loader path resolution, not missing simulator stubs.

Observed roots and fixes:

- `MapCanvasMixin`/WorldMap nils were caused by XML script paths like `..\..\..\Interface\AddOns\Blizzard_MapCanvas\...` escaping the profile cache tree. `resolve_path_with_fallback()` now detects `Interface/AddOns/` after normalization and resolves it from the nearest `AddOns` ancestor.
- `UIParentBottomManagedFrameContainer` / `UIParentRightManagedFrameContainer` nils came from missing real Mists Classic `Blizzard_UIParent/Classic/*.lua|xml` files in the manifest/profile-cache required list.
- `GameTooltipTemplate` method errors (`OnLoadGameTooltip`) came from `Blizzard_SharedXML/Classic/GameTooltipTemplate.xml` loading without its paired `Classic/GameTooltipTemplate.lua`; the manifest/profile-cache required list now includes the real Lua file.
- `QuestUtil.CanCreateQuestGroup` was not fixed by a broader global stub. `Blizzard_FrameXMLUtil/Classic/QuestUtils.lua` resets `QuestUtil = {}` during addon load, so quest objective defaults must be reapplied on the startup loader's `Blizzard_FrameXMLUtil` post-load path, not only on runtime `C_AddOns.LoadAddOn`; the helper now delegates to modeled `C_LFGList.CanCreateQuestGroup` when available.
- `WorldStateProvingGrounds_*` functions are referenced by `Blizzard_FrameXML/Mists/WorldStateFrame.xml`, but the current Mists Classic source cache does not ship matching Lua helpers. The Mists bootstrap supplies profile-scoped handlers that register/update the proving-grounds world-state frame and no-op safely when the simulator has no scenario state.
- **Correction (2026-11-13):** **VERIFIED from the selected Mists source:** `IsEditModeDragging`, `IsInitialized`, `IsInDefaultPosition`, and `IsSystemSettingDefault` are owned by EditMode mixins, not every Frame. The generic registration made the original managed-frame presence check reject plain frames; see [[frame-editmode-method-surface]].
- `UNIT_LEVEL_NON_ATTACKABLE` is a named color global needed by Blizzard font/color code; it belongs with the existing color global registry.

Verification after these fixes:

- `WOW_SIM_NO_ADDONS=1 WOW_SIM_NO_SAVED_VARS=1 timeout 90 target/debug/wow-sim lua-errors` under `client-mists` returned `[]`.
- The normal default retail build was restored with `cargo build --bin wow-sim --bin wow-cli`.

## Sources

- [data/blizzard-ui-files/mists.txt](../../../data/blizzard-ui-files/mists.txt) — Mists WorldMap and SharedMapDataProviders cache manifest entries.
- [profile_cache.rs](../../../src/blizzard_ui_sync/profile_cache.rs) — Mists cache-entry allowlist and usability checks.
- [helpers.rs](../../../src/loader/helpers.rs) — XML script path resolution for escaped `Interface/AddOns` paths.
- [compat_bootstrap.lua](../../../src/mists/compat_bootstrap.lua) — Mists-only startup compatibility surfaces, including proving-grounds world-state handlers.
- [post_load.lua](../../../src/mists/post_load.lua) — Mists-only post-load compatibility surfaces.
- [source_patches.rs](../../../src/lua_api/workarounds/temporary/source_patches.rs) — MapCanvas provider callback source patch.

## See Also

- [[client-profiles]] — profile-specific TOC and Blizzard UI cache behavior.
- [[addon-loading]] — Blizzard addon discovery and runtime loading.
- [[world-map-fog-of-war-overlay-model]] — related WorldMap/FogOfWar data-provider behavior.
