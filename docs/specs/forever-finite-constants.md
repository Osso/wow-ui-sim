# Forever finite UI constants

## Contract

Forever 1.60.1.69913 publishes these additions from its generated API documentation, without changing other client profiles:

- `MinimapTrackingFilter.TrainerClass = 8388608`, `VendorAmmo = 16777216`; metadata: 26 values, range 0–16777216.
- `PingResult.FailedSilent = 8`; metadata: nine values, range 0–8.
- `Constants.Transmog.NoTransmogID = 0`.
- `GamepadPossessBarOverride`: SpecialPageTopBar=1; Page1LeftBar/RightBar/BottomBar=2/3/4; Page2TopBar/LeftBar/RightBar/BottomBar=5/6/7/8; Page3TopBar/LeftBar/RightBar/BottomBar=9/10/11/12. Metadata: twelve values, range 1–12.

Sources: authenticated Forever `MinimapConstantsDocumentation.lua`, `PingConstantsDocumentation.lua`, `TransmogConstantsDocumentation.lua`, and `GamepadUIDocumentation.lua` under `Blizzard_APIDocumentationGenerated`.

## Stance override enum

Forever publishes `Enum.GamepadStanceBarOverride`: `None=1`, Page1 Left/Right/Bottom=2/3/4, Page2 Top/Left/Right/Bottom=5/6/7/8, Page3 Top/Left/Right/Bottom=9/10/11/12. `GamepadStanceBarOverrideMeta` has MinValue=1, MaxValue=12, NumValues=12. Publication remains Forever-only.

Evidence: cached `GamepadUIDocumentation.lua` and `Resources/LuaEnum.lua` in Ketho/BlizzardInterfaceResources revision `659e8042049df854c114714f8ecd640823a1cd5c`, whose README identifies build 1.60.1.69913. Unlike possession, the first stance value means no override.

The grouped regression executes unchanged `StanceBar.lua` and `GamepadOverrideBarMixin.lua` against frame-backed fixture anchors. It exercises all twelve mappings, default right-anchor positioning, linked-bar identity, and unparented `None` behavior. Page-owner getters are fixture inputs; this is not full PageUnit startup proof.

## Legacy and level constants

Forever `LegacyConstantsDocumentation.lua` publishes `Constants.LegacyConsts`: `LEGACY_REWARD_TRACK_FACTION_ID=2802`, `LEGACY_POINTS_TRAIT_CURRENCY_ID=4225`, `LEGACY_TREE_PROFESSIONS_ID=1187`, `LEGACY_TREE_ADVENTURE_ID=1188`, `LEGACY_TREE_PROGRESSION_ID=1189`, and `LEGACY_TREE_ADVENTURE_TALENTED_NODE_ID=110298`.

Forever `LevelConstantsDocumentation.lua` publishes `Constants.LevelConstsExposed.MIN_RES_SICKNESS_LEVEL`, `MIN_ACHIEVEMENT_LEVEL`, and `MIN_TALENT_LEVEL`, each `10`.

The actual Camelot `PlayerSpellsMicroButtonMixin:GetTalentUnlockLevel()` returns `10` when adventure tree `1188` has no config. An unknown tree returning nil remains valid; this publication does not change the trait model or claim configured-tree behavior. Grouped tests assert all nine values and execute the unchanged Camelot override file against the real `C_Traits` API.

## Quest log limit

Forever `QuestConstantsDocumentation.lua` publishes `Constants.QuestLogConsts.MAXIMUM_NUM_QUESTS_LOG_CAN_ACCEPT = 40`. The Camelot quest-map count refresh compares the current quest count against this value while `WorldMapFrame:Show()` initializes the map. It remains Forever-only.

The world-map runtime regression shows the real failure boundary: without the constant, `QuestLogQuests_ShowQuestCount()` aborts `WorldMapMixin:OnShow()` before `SetMapID()`. The scroll container consequently retains a nil `targetScale`, and every later `OnUpdate` fails in `IsZoomingOut()`. The test shows the map and runs sixty GUI-style updates, asserting a positive target scale and no collected errors; no vendor guard or scale fallback is added.

## Verification

`tests/wowforever_finite_constants.rs` checks publication, actual Camelot minimap filter construction, and full PingManager/TransmogShared source loading in an initialized simulator environment. Initial tests reproduced missing PingResult data and the MinimapConstants nil table key (0/2); both passed after publication. TransmogShared loaded in the focused fixture, so its full-startup failure is not proven to arise solely from NoTransmogID. These additions do not establish complete Transmog initialization or gamepad possession behavior; remaining consumer failures must be diagnosed independently.
