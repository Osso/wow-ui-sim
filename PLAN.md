# Project Plan

Phases are executed in order: 1 → 2 → 3 → 4 → 5.

## Current Blocker

None - Phase 4 complete. Starting Phase 5.

## Active TODO

### Phase 4: Rendering [COMPLETE]

- [x] Add iced dependency and basic app scaffold
- [x] Expose widget registry to renderer
- [x] Render frames as rectangles with position/size from anchors
- [x] Render textures (solid colors, differentiated from frames)
- [x] Render FontStrings with text content
- [x] Handle frame strata and level for z-ordering

### Phase 5: Real Addon Testing [IN PROGRESS]

- [x] Load Ace3 library suite (43 Lua, 15 XML - 100%)
- [x] Load a simple addon (LibStub already works)
- [x] Add missing APIs: GetBuildInfo, GetPhysicalScreenSize, UnitPlayerControlled, UnitIsTapDenied, PixelUtil, math utilities
- [x] Add frame methods: SetBackdrop, SetBackdropColor, SetID, HookScript
- [x] Add built-in frames: Minimap, AddonCompartmentFrame
- [x] Improve debugstack to include file paths (for LibGraph-2.0)
- [x] Add C_AddOns namespace (GetAddOnMetadata, EnableAddOn, etc.)
- [x] Test WeakAuras Init.lua (loads successfully)
- [ ] Load full WeakAuras addon (needs more libs)
- [ ] Improve Details addon loading (currently 67 Lua, 92 warnings)

## Decisions Needed

None currently.

## Parked

- [ ] Blizzard_FrameXML loading (requires more APIs)
- [ ] Game API stubs (UnitHealth, combat events, etc.)
- [ ] Full template inheritance system

## Reference Addons

Located at `~/Projects/wow/reference-addons/`:
- `Ace3/` - Popular addon framework
- `DeadlyBossMods/` - Raid encounter alerts
- `Details/` - Damage meter
- `Plater/` - Nameplate addon
- `WeakAuras2/` - Custom display framework
- `wow-ui-source/` - Blizzard's official UI code

## Current Statistics

- **68 tests passing** (7 ignored)
- Blizzard_SharedXMLBase: 100% loaded (34 Lua, 2 XML)
- Blizzard_SharedXML: 100% loaded (155 Lua, 72 XML)
- Ace3: 100% loaded (43 Lua, 15 XML)
- WeakAuras Init.lua: loads successfully (reports missing libs, expected)
- Plater: Partial (7 Lua, 1 XML - needs DetailsFramework library)
- Details: Improved (67 Lua, 2 XML, 92 warnings - most libs load)
- DBM-Core: Partial (19 Lua - needs bundled Libs folder)
- GameMenu: 100% loaded (3 Lua, 2 XML)
