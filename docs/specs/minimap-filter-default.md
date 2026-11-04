# Minimap filter default

Retail `Blizzard_APIDocumentationGenerated/MinimapDocumentation.lua:136-148` declares `C_Minimap.IsFilteredOut(filterType)` with a required `MinimapTrackingFilter` argument and a Boolean result. `Blizzard_WorldMap/Blizzard_WorldMapTemplates.lua:97` reads it from `WorldMapFilterMixin:Get`.

## Best-guess basis (best-guess, native UNKNOWN)

The simulator has no minimap tracking model and reports zero tracking types. Until that model exists, Retail treats every numeric filter as shown, so `IsFilteredOut` returns false. Non-numeric arguments are rejected. Native per-filter defaults are unknown.

## Forever static filters

Forever exposes eligible static tracking rows in native order. Their filter bits, texture IDs, class restrictions and signed subtypes come from the 1.60.1.70707 native evidence. The initial mask comes from `minimapTrackedInfov4`, or 2162728 when unavailable, restricted to eligible rows. `IsFilteredOut` tests whether any requested bit is enabled; zero is not filtered out. Changes update the in-memory mask.

Forever 1.60.1.70707 WM77b establishes the CVar default as 2162728. `SetTracking` changes only the selected static bit. `ClearAllTracking` clears static bits 0–24 even without a player, preserving unrelated high bits. Neither operation immediately writes the CVar.

Dynamic tracking spell effects, persistence triggers and notification timing remain unknown; initialization from the CVar does not establish persistence after a change.

## Tests

`blizzard_world_map_loads::world_map_tracking_filters_default_to_visible` builds the original Blizzard filter table, reads two filters through `Get`, and checks invalid input.

## Known gaps

Retail minimap tracking state and mutation through `MinimapUtil.SetTrackingFilterByFilterID` remain unmodeled. The initialized-default query is specified in [world-map API inputs](world-map-api-inputs.md).
