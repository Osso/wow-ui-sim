# Map art help-text position

`C_Map.GetMapArtHelpTextPosition(uiMapID)` requires a number and returns `Enum.MapCanvasPosition.BottomRight` for every map ID. The query is published for Retail, PTR, WoW Forever, Mists, Era and Anniversary.

## Best-guess basis (best-guess, native UNKNOWN)

`Blizzard_SharedMapDataProviders/ClickToZoomDataProvider.lua:107–116` uses BottomRight when no map ID is available. Applying that value to map IDs is a user-approved best guess, not a claim about native per-map data. The pinned Retail, PTR and WoW Forever `Blizzard_APIDocumentationGenerated/MapDocumentation.lua:104–118` and Mists, Era and Anniversary versions at lines 77–90 declare the numeric input and non-nil `MapCanvasPosition` result.

## Known gaps

Per-map positions require a UiMapArt DB2 export. Native position data and behavior for individual map IDs are unknown.
