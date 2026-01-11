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
- [ ] Load a simple addon (LibStub already works)
- [x] Add missing APIs: GetRealmName, GetLocale, UnitName, UnitGUID, UnitLevel, UnitExists, UnitFactionGroup, GetCurrentRegion, UnregisterAllEvents
- [ ] Test more complex addons (Details, WeakAuras, etc.)

## Decisions Needed

None currently.

## Parked

- [ ] Blizzard_FrameXML loading (requires more APIs)
- [ ] Game API stubs (GetUnitName, UnitHealth, combat events, etc.)
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

- **65 tests passing** (7 ignored)
- Blizzard_SharedXMLBase: 100% loaded (34 Lua, 2 XML)
- Blizzard_SharedXML: 100% loaded (155 Lua, 72 XML)
- Ace3: 100% loaded (43 Lua, 15 XML)
- Details: Partial (65 Lua, 2 XML - needs more game APIs)
- DBM-Core: Partial (19 Lua - needs bundled Libs folder)
