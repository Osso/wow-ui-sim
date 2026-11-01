# CinematicModel camera inputs

CinematicModel objects expose `SetFacingLeft` and `SetTargetDistance`; PlayerModel, DressUpModel and ordinary Frame objects do not gain these methods. `SetFacingLeft` accepts a boolean or omitted value, and `SetTargetDistance` requires a number. Neither changes rendered output because 3D model rendering is intentionally unsupported.

## Best-guess basis (best-guess, native UNKNOWN)

The pinned Retail, PTR and WoW Forever `Blizzard_APIDocumentationGenerated/FrameAPICinematicModelDocumentation.lua:92,154`, and Mists, Era and Anniversary versions at lines 85 and 141, declare these CinematicModel methods. `Blizzard_GarrisonTemplates/Blizzard_GarrisonMissionTemplates.lua:2947` calls `SetFacingLeft` on its model children; `Blizzard_GarrisonMissionTemplates.xml:907` calls `SetTargetDistance`. The simulator's no-render behavior is an explicit permanent limitation, not native evidence.

## Known gaps

3D camera and model rendering are unsupported. Native validation beyond the documented argument types is unknown.
