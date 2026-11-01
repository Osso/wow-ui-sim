# CinematicModel camera inputs

CinematicModel objects, including XML-created objects, expose `SetFacingLeft`, `SetTargetDistance` and `SetHeightFactor`; PlayerModel, DressUpModel and ordinary Frame objects do not gain these methods. `SetFacingLeft` accepts a boolean or omitted value; the other methods require a number. Neither changes rendered output because 3D model rendering is intentionally unsupported.

## Best-guess basis (best-guess, native UNKNOWN)

The pinned Retail, PTR and WoW Forever `Blizzard_APIDocumentationGenerated/FrameAPICinematicModelDocumentation.lua:92,154`, and Mists, Era and Anniversary versions at lines 85 and 141, declare these CinematicModel methods. `SetHeightFactor` is declared at line 113 (Retail, PTR, WoW Forever) or 104 (Mists, Era, Anniversary). `Blizzard_GarrisonMissionTemplates.xml:1172` calls it. `Blizzard_GarrisonTemplates/Blizzard_GarrisonMissionTemplates.lua:2947` calls `SetFacingLeft` on its model children; `Blizzard_GarrisonMissionTemplates.xml:907` calls `SetTargetDistance`. The simulator's no-render behavior is an explicit permanent limitation, not native evidence.

## Known gaps

3D camera and model rendering are unsupported. Native validation beyond the documented argument types is unknown.
