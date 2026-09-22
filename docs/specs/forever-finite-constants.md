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

## Bag indices

Forever alone publishes nine character-bank tabs (`CharacterBankTab_1..9 = 6..14`) and nine account-bank enum slots (`AccountBankTab_1..9 = 15..23`). `BagIndexMeta` has MinValue=-3, MaxValue=23, NumValues=27. The nine non-bank-tab members retain their shared values (-3 through 5); enumeration terminates at each missing tenth tab. Other profiles retain their existing publication.

Authoritative evidence: `Blizzard_APIDocumentationGenerated/BagIndexConstantsDocumentation.lua` in the Forever 1.60.1.69913 cache, sourced from Gethe/wow-ui-source revision `70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e`, lines 18–51. BetterBags [Forever introduction `411a6f6ee1ea40eca8ac96927ccdd49a6aab3941`](https://github.com/Cidan/BetterBags/commit/411a6f6ee1ea40eca8ac96927ccdd49a6aab3941), `core/constants.lua`, motivated the audit: its `enumerateBagIndices` consumer walks contiguous named members. The regression uses that exact loop, asserting exact values, disjoint IDs, termination, all non-bank-tab values, member count, and metadata through `WowLuaEnv`.

Implementation: `src/c_api/forever_finite_constants.rs`, registered only for `client-wowforever`. Tests: `forever_bag_index_enumerates_disjoint_bank_tabs` and `forever_bag_index_preserves_non_bank_values_and_exact_metadata` in the existing grouped `tests/wowforever_finite_constants.rs` module. This is enum-publication coverage, not a full BetterBags load or proof of account-bank availability, purchased tabs, bank events, or money-display lifecycle.

## Finite event registration

Forever additionally accepts `CHAT_MSG_COLLECTED_APPEARANCE` and `UNIT_AURA_BLOCKED`, published by cached `ChatInfoDocumentation.lua:1348–1351` and `UnitAuraDocumentation.lua:641–645`. Both belong to the sorted Forever-only event list; arbitrary unknown names remain rejected. Other profiles and the undocumented `PLAYER_EQUIPED_SPELLS_CHANGED` name remain unchanged.

`forever_finite_events_register_deliver_and_reject_unknown` tests registration, injected callback delivery with payload preservation, unregistration, and unknown-name rejection in the existing grouped integration module. Injection proves simulator dispatch, not native event production or payload secrecy. The frozen pre-patch binary rejects both names (RED, isolated ledger `/tmp/forever-addon-audit/forever-finite-events-red-ww5p10a5/ledger.json`); compiled GREEN and addon replay remain pending.

## Guild Discord event and aura sound trigger enum

Forever also accepts `CHAT_MSG_GUILD_DISCORD`, explicitly published by cached `ChatInfoDocumentation.lua:1676–1680` and consumed by Chattynator. Registration, injected delivery, unregister suppression, and arbitrary-name rejection are covered by the existing finite-event regression. Other documented residual events are not added by this change.

`UnitAuraConstantsDocumentation.lua:6–16` publishes `Enum.UnitAuraSoundTrigger`: `Added=0`, `ApplicationsIncreased=1`, `Removed=2`, with `UnitAuraSoundTriggerMeta` MinValue=0, MaxValue=2, NumValues=3. Forever shares the existing retail 12.1 enum producer; other profiles retain their previous publication policy. The grouped regression checks all three values, exact member count, metadata, and repeated restoration after cleanup removes both tables. This proves enum publication, not aura sound playback.

Frozen `b8f0982be` rejects the event and lacks both enum tables: `/tmp/forever-addon-audit/next-registration-red-fc0so85f/ledger.json`. The owned `319725248` registration slice passes the existing grouped integration filter 10/10, including repeated enum restoration and guild Discord delivery/unregistration. Other agents edited unrelated source during compilation; the executable is not proof of an exact whole-repository revision. Build and test ledgers: `/tmp/forever-addon-audit/next-registration-green-q_hgyboa/`. Focused integration now passes 10/10, including guild Discord delivery/unregistration and repeated enum restoration. An exact `ebff90517` replay clean-starts Chattynator and EnhanceQoL with the same binary hash; this proves registration/publication only, not native event production, aura sound playback, or whole-addon workflows.

## Player swing event

Forever accepts exactly the documented `PLAYER_SWING` event with payload order `(swingDuration: number, swingType: PlayerSwingType)`. `Enum.PlayerSwingType` publishes `MainHand=0`, `OffHand=1`, `Ranged=2`; `PlayerSwingTypeMeta` publishes MinValue=0, MaxValue=2, NumValues=3. Source: cached `Blizzard_APIDocumentationGenerated/SwingTimerDocumentation.lua:43–51, 72–84`. The C API-owned enum producer runs through shared enum bootstrap so cleanup restoration republishes the same values. Publication remains Forever-only and unknown event names remain rejected.

AppelSwingsForever `8925606`, `Swing.lua:23–60`, registers this event and routes its duration/type payload to independent main-hand, off-hand, and ranged state. It ignores nonpositive durations and clears completed progress rather than holding a full bar. This change does not modify that addon or synthesize gameplay swings. The initial exclusion of range-event registration was superseded by the actual GUI replay: Blizzard's main-hand, off-hand, and ranged SwingTimer frames reject `PLAYER_SWING_RANGE_UPDATE` during OnLoad.

Two grouped regressions assert exact two-value synchronous Admin-injected delivery for all three types, callback order, unregister suppression, unknown-name rejection, exact enum membership/metadata, and restoration after normal bootstrap cleanup. Injection is bounded simulator dispatch evidence, not real gameplay production or native conformance. Frozen `5a1cc851` RED rejects `PLAYER_SWING` and lacks the enum: `/tmp/forever-addon-audit/player-swing-red-corrected-fwmpzx12/ledger.json`. Targeted development GREEN at clean `c818f1b915ba811a8aee0d8cda9f29acc3d7eb39` passes the full finite-constants group 12/12, including both swing regressions. The earlier 11/12 result exposed missing enum restoration; moving the same producer into shared enum bootstrap fixed that boundary. Build/test ledgers: `/tmp/forever-addon-audit/player-swing-final-green-avb1bbsv/`. The build has unchanged source hashes and no warnings; `wow-sim` SHA-256 is `8536d33c7f603fafd2db4d155c262c4c61be4de4776ed7324a33295092e8abf2`. Independent final verification and actual AppelSwingsForever GUI replay remain parent-owned and pending.

### Swing range registration follow-up

Forever also accepts the documented synchronous `PLAYER_SWING_RANGE_UPDATE(swingType, isInRange, checksRange)` event (`SwingTimerDocumentation.lua:54–63`). Both boolean positions retain false/true values independently; `checksRange=false` does not mean an out-of-range observation. Registration survives bootstrap cleanup, and arbitrary unknown events remain rejected.

Frozen `c818f1b9` reproduces rejection in Blizzard SwingTimer OnLoad and the focused delivery probe: `/tmp/forever-addon-audit/swing-range-red-u9ot06pm/ledger.json`. At clean `ff19ecca24d398580584d08f258e9a4134b2abae`, the grouped finite-constants target passes 13/13, including range payload and cleanup tests; source hashes match before/after the build. Evidence: `/tmp/forever-addon-audit/swing-range-green-8s22zw34/{build,test}-ledger.json`. Independent final verification and parent GUI replay remain pending. This models registration and injected payload delivery only—not range detection, `C_SwingTimer` methods, or a gameplay event producer.

## Verification

`tests/wowforever_finite_constants.rs` checks publication, actual Camelot minimap filter construction, and full PingManager/TransmogShared source loading in an initialized simulator environment. Initial tests reproduced missing PingResult data and the MinimapConstants nil table key (0/2); both passed after publication. TransmogShared loaded in the focused fixture, so its full-startup failure is not proven to arise solely from NoTransmogID. These additions do not establish complete Transmog initialization or gamepad possession behavior; remaining consumer failures must be diagnosed independently.
