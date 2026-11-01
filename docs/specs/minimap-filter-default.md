# Minimap filter default

Retail `Blizzard_APIDocumentationGenerated/MinimapDocumentation.lua:136-148` declares `C_Minimap.IsFilteredOut(filterType)` with a required `MinimapTrackingFilter` argument and a Boolean result. `Blizzard_WorldMap/Blizzard_WorldMapTemplates.lua:97` reads it from `WorldMapFilterMixin:Get`.

## Best-guess basis (best-guess, native UNKNOWN)

The simulator has no minimap tracking model and reports zero tracking types. Until that model exists, Retail treats every numeric filter as shown, so `IsFilteredOut` returns false. Non-numeric arguments are rejected. Native per-filter defaults are unknown.

## Tests

`blizzard_world_map_loads::world_map_tracking_filters_default_to_visible` builds the original Blizzard filter table, reads two filters through `Get`, and checks invalid input.

## Known gaps

Per-filter defaults through `GetDefaultTrackingValue`, minimap tracking state, and mutation through `MinimapUtil.SetTrackingFilterByFilterID` remain unmodeled.
