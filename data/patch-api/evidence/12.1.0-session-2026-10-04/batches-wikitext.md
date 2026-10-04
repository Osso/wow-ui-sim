# Retail 12.1.0 wikitext batches

Planning only. No tests, cargo, builds or simulator runs performed. All 778 ledger additions remain audit-pending; this is a breadth-first future-work queue, not acceptance evidence.

Order: tests-only presence/call and absence candidates first; bounded modelable groups next (at most eight symbols); needs-look deferred last, with no hard-case designs. Namespace/file grouping does not mean one presence assertion proves all contracts. Triage/citations are in [triage-wikitext.md](triage-wikitext.md); annotations/identities in `data/patch-api/sources/12.1.0-wikitext-register.json`.

Plain FrameXML cache helpers: load their owning addon, including LoD addons, then resolve/call each helper with valid fixture inputs. Keep vendor Lua unmodified; group proofs per listed file. Native registrations: check object/global type, call with concrete input and observable output/state; changed rows must check all annotation deltas. Event acceptance alone does not prove payload or trigger; CVar existence alone does not prove its listed default/scope. Deprecated wrappers need feature/setting-aware proof, not unconditional nil expectations.

Static absence batches require runtime ordinary lookup after startup, including generated/native and cached surfaces. If runtime contradicts triage, reclassify before claiming proof. No prior PTR machine status is inherited.

## Batch totals

| Phase | Batches | Rows |
|---|---:|---:|
| Tests-only candidates | 215 | 690 |
| Modelable | 34 | 72 |
| Deferred needs-look | 3 | 16 |

## W01 — Tests-only candidate: src/cvars.rs

Section: cvars; triage: already-removed/absent-as-required; 2 symbols.

Required later proof: absence/rejection after startup on Retail; static search is provisional.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-cvars-lastLockedDelvesCompanionAbilities-1341` | `lastLockedDelvesCompanionAbilities` | `src/cvars.rs:381` |
| `wt-cvars-SlugSupersampling-1342` | `SlugSupersampling` | `src/cvars.yaml:1300`; `src/cvars.rs:381` |

## W02 — Tests-only candidate: src/cvars.rs

Section: cvars; triage: implemented-needs-proof; 20 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-cvars-accessibilityScreenNarrationEnabled-1313` | `accessibilityScreenNarrationEnabled` | `src/cvars.rs:385` |
| `wt-cvars-accessibilityScreenNarrationSpeechRate-1314` | `accessibilityScreenNarrationSpeechRate` | `src/cvars.rs:386` |
| `wt-cvars-accessibilityScreenNarrationSpeechVolume-1315` | `accessibilityScreenNarrationSpeechVolume` | `src/cvars.rs:387` |
| `wt-cvars-accessibilityScreenNarrationVoice-1316` | `accessibilityScreenNarrationVoice` | `src/cvars.rs:388` |
| `wt-cvars-AftermathShaderDebug-1317` | `AftermathShaderDebug` | `src/cvars.rs:389` |
| `wt-cvars-discordClientEnabled-1318` | `discordClientEnabled` | `src/cvars.rs:390` |
| `wt-cvars-discordDisplayName-1319` | `discordDisplayName` | `src/cvars.rs:391` |
| `wt-cvars-nameplateCheckDistanceForTarget-1320` | `nameplateCheckDistanceForTarget` | `src/cvars.rs:392` |
| `wt-cvars-nameplateForceShowUnitName-1321` | `nameplateForceShowUnitName` | `src/cvars.rs:393` |
| `wt-cvars-nameplateNotSelectedAlpha-1322` | `nameplateNotSelectedAlpha` | `src/cvars.rs:394` |
| `wt-cvars-nameplatePlayRemovalAnimation-1323` | `nameplatePlayRemovalAnimation` | `src/cvars.rs:395` |
| `wt-cvars-nameplateShowAllPersonalAuras-1324` | `nameplateShowAllPersonalAuras` | `src/cvars.rs:396` |
| `wt-cvars-nameplateShowFriendlyRealmName-1325` | `nameplateShowFriendlyRealmName` | `src/cvars.rs:397` |
| `wt-cvars-nameplateShowFriends-1326` | `nameplateShowFriends` | `src/cvars.rs:398` |
| `wt-cvars-pingTarget-1327` | `pingTarget` | `src/cvars.rs:399` |
| `wt-cvars-raidFramesDispelIndicatorOverlayAnimation-1328` | `raidFramesDispelIndicatorOverlayAnimation` | `src/cvars.rs:400` |
| `wt-cvars-showPingsOnRaidFrames-1329` | `showPingsOnRaidFrames` | `src/cvars.rs:401` |
| `wt-cvars-showScreenNarrationDialog-1330` | `showScreenNarrationDialog` | `src/cvars.rs:402` |
| `wt-cvars-taintLogObjectSecrets-1331` | `taintLogObjectSecrets` | `src/cvars.rs:403` |
| `wt-cvars-userFontScaleGlue-1333` | `userFontScaleGlue` | `src/cvars.rs:404` |

## W03 — Tests-only candidate: BATTLE events

Section: events; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-BATTLE_NET_FRIEND_TAG_ENABLED_STATUS_UPDATED-1251` | `BATTLE_NET_FRIEND_TAG_ENABLED_STATUS_UPDATED` | `src/event/valid_events.rs:105`; `retail/AddOns/Blizzard_APIDocumentationGenerated/FriendListDocumentation.lua:417` |
| `wt-events-BATTLE_NET_TITLE_FRIEND_CUSTOM_NAME_ENABLED_STATUS_UPDATED-1252` | `BATTLE_NET_TITLE_FRIEND_CUSTOM_NAME_ENABLED_STATUS_UPDATED` | `src/event/valid_events.rs:106`; `retail/AddOns/Blizzard_APIDocumentationGenerated/FriendListDocumentation.lua:423` |

## W04 — Tests-only candidate: BATTLETAG events

Section: events; triage: already-removed/absent-as-required; 1 symbols.

Required later proof: absence/rejection after startup on Retail; static search is provisional.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-BATTLETAG_INVITE_SHOW-1296` | `BATTLETAG_INVITE_SHOW` | `src/event/valid_events_a.rs:181`; `src/event/valid_events.rs:101` |

## W05 — Tests-only candidate: CHAT events

Section: events; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-CHAT_MSG_GUILD_DISCORD-1253` | `CHAT_MSG_GUILD_DISCORD` | `src/event/valid_events.rs:34`; `retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1651` |

## W06 — Tests-only candidate: CONFIRM events

Section: events; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-CONFIRM_BATTLE_NET_FRIEND_INVITE_SHOW-1254` | `CONFIRM_BATTLE_NET_FRIEND_INVITE_SHOW` | `src/event/valid_events.rs:35`; `retail/AddOns/Blizzard_APIDocumentationGenerated/FriendListDocumentation.lua:572` |

## W07 — Tests-only candidate: DISCORD events

Section: events; triage: implemented-needs-proof; 6 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-DISCORD_GUILD_ACHIEVEMENT-1255` | `DISCORD_GUILD_ACHIEVEMENT` | `src/event/valid_events.rs:109`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:230` |
| `wt-events-DISCORD_GUILD_LOBBY_UPDATE-1256` | `DISCORD_GUILD_LOBBY_UPDATE` | `src/event/valid_events.rs:37`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:240` |
| `wt-events-DISCORD_GUILD_SETTINGS_UPDATE-1257` | `DISCORD_GUILD_SETTINGS_UPDATE` | `src/event/valid_events.rs:38`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:246` |
| `wt-events-DISCORD_LINK_UPDATE-1258` | `DISCORD_LINK_UPDATE` | `src/event/valid_events.rs:39`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:252` |
| `wt-events-DISCORD_SERVER_LIST_UPDATE-1259` | `DISCORD_SERVER_LIST_UPDATE` | `src/event/valid_events.rs:40`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:258` |
| `wt-events-DISCORD_STATUS_UPDATE-1260` | `DISCORD_STATUS_UPDATE` | `src/event/valid_events.rs:41`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:264` |

## W08 — Tests-only candidate: EXTERNAL events

Section: events; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-EXTERNAL_EVENT_LAUNCH_URL_FAILED-1261` | `EXTERNAL_EVENT_LAUNCH_URL_FAILED` | `src/event/valid_events.rs:42`; `retail/AddOns/Blizzard_APIDocumentationGenerated/ExternalEventURLDocumentation.lua:40` |

## W09 — Tests-only candidate: GROUP events

Section: events; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-GROUP_BUFF_VISUAL_ALERTS_CHANGED-1264` | `GROUP_BUFF_VISUAL_ALERTS_CHANGED` | `src/event/valid_events.rs:46`; `retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:582` |

## W10 — Tests-only candidate: GUILD events

Section: events; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-GUILD_RANKS_UPDATE_ACTIVE_PLAYER-1265` | `GUILD_RANKS_UPDATE_ACTIVE_PLAYER` | `src/event/valid_events.rs:48`; `retail/AddOns/Blizzard_APIDocumentationGenerated/GuildInfoDocumentation.lua:482` |

## W11 — Tests-only candidate: HIDDEN events

Section: events; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-HIDDEN_GROUP_BUFFS_CHANGED-1266` | `HIDDEN_GROUP_BUFFS_CHANGED` | `src/event/valid_events.rs:49`; `retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:592` |

## W12 — Tests-only candidate: HOUSE events

Section: events; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-HOUSE_RESET_COMPLETED-1267` | `HOUSE_RESET_COMPLETED` | `src/event/valid_events.rs:119`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingUIDocumentation.lua:715` |
| `wt-events-HOUSE_RESET_FAILED-1268` | `HOUSE_RESET_FAILED` | `src/event/valid_events.rs:120`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingUIDocumentation.lua:721` |

## W13 — Tests-only candidate: HOUSING events

Section: events; triage: implemented-needs-proof; 15 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-HOUSING_BLUEPRINT_COLLECTION_FAILURE-1269` | `HOUSING_BLUEPRINT_COLLECTION_FAILURE` | `src/event/valid_events.rs:122`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:256` |
| `wt-events-HOUSING_BLUEPRINT_COLLECTION_RECEIVED-1270` | `HOUSING_BLUEPRINT_COLLECTION_RECEIVED` | `src/event/valid_events.rs:123`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:266` |
| `wt-events-HOUSING_BLUEPRINT_CONTENTS_FAILURE-1271` | `HOUSING_BLUEPRINT_CONTENTS_FAILURE` | `src/event/valid_events.rs:124`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:276` |
| `wt-events-HOUSING_BLUEPRINT_CONTENTS_RECEIVED-1272` | `HOUSING_BLUEPRINT_CONTENTS_RECEIVED` | `src/event/valid_events.rs:125`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:287` |
| `wt-events-HOUSING_BLUEPRINT_DELETE_FAILURE-1273` | `HOUSING_BLUEPRINT_DELETE_FAILURE` | `src/event/valid_events.rs:126`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:297` |
| `wt-events-HOUSING_BLUEPRINT_DELETE_SUCCESS-1274` | `HOUSING_BLUEPRINT_DELETE_SUCCESS` | `src/event/valid_events.rs:127`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:308` |
| `wt-events-HOUSING_BLUEPRINT_EXPORT_FAILURE-1275` | `HOUSING_BLUEPRINT_EXPORT_FAILURE` | `src/event/valid_events.rs:128`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:318` |
| `wt-events-HOUSING_BLUEPRINT_EXPORT_SUCCESS-1276` | `HOUSING_BLUEPRINT_EXPORT_SUCCESS` | `src/event/valid_events.rs:129`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:328` |
| `wt-events-HOUSING_BLUEPRINT_IMPORT_FAILURE-1277` | `HOUSING_BLUEPRINT_IMPORT_FAILURE` | `src/event/valid_events.rs:130`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:338` |
| `wt-events-HOUSING_BLUEPRINT_IMPORT_STARTED-1278` | `HOUSING_BLUEPRINT_IMPORT_STARTED` | `src/event/valid_events.rs:131`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:348` |
| `wt-events-HOUSING_BLUEPRINT_IMPORT_SUCCESS-1279` | `HOUSING_BLUEPRINT_IMPORT_SUCCESS` | `src/event/valid_events.rs:132`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:354` |
| `wt-events-HOUSING_BLUEPRINT_RENAME_FAILURE-1280` | `HOUSING_BLUEPRINT_RENAME_FAILURE` | `src/event/valid_events.rs:133`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:360` |
| `wt-events-HOUSING_BLUEPRINT_RENAME_SUCCESS-1281` | `HOUSING_BLUEPRINT_RENAME_SUCCESS` | `src/event/valid_events.rs:134`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:371` |
| `wt-events-HOUSING_BLUEPRINTS_AVAILABILITY_CHANGED-1282` | `HOUSING_BLUEPRINTS_AVAILABILITY_CHANGED` | `src/event/valid_events.rs:121`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:382` |
| `wt-events-HOUSING_NEW_DECOR_PLACE_COMPLETE-1284` | `HOUSING_NEW_DECOR_PLACE_COMPLETE` | `src/event/valid_events.rs:135`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingDecorUIDocumentation.lua:484` |

## W14 — Tests-only candidate: IGNORE events

Section: events; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-IGNORE_NEIGHBORHOOD_RESPONSE-1285` | `IGNORE_NEIGHBORHOOD_RESPONSE` | `src/event/valid_events.rs:136`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingUIDocumentation.lua:743` |

## W15 — Tests-only candidate: LEGACY events

Section: events; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-LEGACY_FRIEND_SYSTEM_STATUS_UPDATED-1286` | `LEGACY_FRIEND_SYSTEM_STATUS_UPDATED` | `src/event/valid_events.rs:51`; `retail/AddOns/Blizzard_APIDocumentationGenerated/FriendListDocumentation.lua:595` |

## W16 — Tests-only candidate: LFG events

Section: events; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-LFG_LIST_CENSORED_ACTIVE_ENTRY_UPDATE-1287` | `LFG_LIST_CENSORED_ACTIVE_ENTRY_UPDATE` | `src/event/valid_events.rs:138`; `retail/AddOns/Blizzard_APIDocumentationGenerated/LFGListInfoDocumentation.lua:747` |
| `wt-events-LFG_LIST_REVEALED_CENSORED_ACTIVE_ENTRY-1288` | `LFG_LIST_REVEALED_CENSORED_ACTIVE_ENTRY` | `src/event/valid_events.rs:52`; `retail/AddOns/Blizzard_APIDocumentationGenerated/LFGListInfoDocumentation.lua:786` |

## W17 — Tests-only candidate: SOCIAL events

Section: events; triage: implemented-needs-proof; 3 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-SOCIAL_UI_FRIENDS_LIST_SYSTEM_STATUS_UPDATED-1289` | `SOCIAL_UI_FRIENDS_LIST_SYSTEM_STATUS_UPDATED` | `src/event/valid_events.rs:58`; `retail/AddOns/Blizzard_APIDocumentationGenerated/FriendListDocumentation.lua:623` |
| `wt-events-SOCIAL_UI_SOCIAL_QUEUE_SYSTEM_STATUS_UPDATED-1290` | `SOCIAL_UI_SOCIAL_QUEUE_SYSTEM_STATUS_UPDATED` | `src/event/valid_events.rs:141`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SocialQueueSystemStatusDocumentation.lua:35` |
| `wt-events-SOCIAL_UI_SYSTEM_STATUS_UPDATED-1291` | `SOCIAL_UI_SYSTEM_STATUS_UPDATED` | `src/event/valid_events.rs:59`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SocialUIDocumentation.lua:26` |

## W18 — Tests-only candidate: SPELL events

Section: events; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-SPELL_UPDATE_COOLDOWN-1302` | `SPELL_UPDATE_COOLDOWN` | `src/event/valid_events_c.rs:201`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SpellBookDocumentation.lua:862` |

## W19 — Tests-only candidate: UNIT events

Section: events; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-UNIT_PING_PIN_ADDED-1292` | `UNIT_PING_PIN_ADDED` | `src/event/valid_events.rs:63`; `retail/AddOns/Blizzard_APIDocumentationGenerated/PingManagerSecureDocumentation.lua:253` |

## W20 — Tests-only candidate: HousingControlsUtil

Section: framexml; triage: already-removed/absent-as-required; 1 symbols.

Required later proof: absence/rejection after startup on Retail; static search is provisional.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-HousingControlsUtil.CanActivateHousingControls-1007` | `HousingControlsUtil.CanActivateHousingControls` | INFERRED; no declaration found |

## W21 — Tests-only candidate: HousingTutorialUtil

Section: framexml; triage: already-removed/absent-as-required; 1 symbols.

Required later proof: absence/rejection after startup on Retail; static search is provisional.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-HousingTutorialUtil.HousingQuestTutorialComplete-1008` | `HousingTutorialUtil.HousingQuestTutorialComplete` | INFERRED; no declaration found |

## W22 — Tests-only candidate: framexml

Section: framexml; triage: already-removed/absent-as-required; 93 symbols.

Required later proof: absence/rejection after startup on Retail; static search is provisional.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-AddFrameLock-963` | `AddFrameLock` | INFERRED; no declaration found |
| `wt-framexml-AnimateTexCoords-965` | `AnimateTexCoords` | INFERRED; no declaration found |
| `wt-framexml-BattleTagInviteFrame_Show-966` | `BattleTagInviteFrame_Show` | INFERRED; no declaration found |
| `wt-framexml-BetterDate-967` | `BetterDate` | INFERRED; no declaration found |
| `wt-framexml-BFAMissionFrame_EscapePressed-968` | `BFAMissionFrame_EscapePressed` | INFERRED; no declaration found |
| `wt-framexml-BoostTutorial_AttemptLoad-969` | `BoostTutorial_AttemptLoad` | INFERRED; no declaration found |
| `wt-framexml-BuildColoredListString-970` | `BuildColoredListString` | INFERRED; no declaration found |
| `wt-framexml-BuildIconArray-971` | `BuildIconArray` | INFERRED; no declaration found |
| `wt-framexml-BuildListString-972` | `BuildListString` | INFERRED; no declaration found |
| `wt-framexml-BuildMultilineTooltip-973` | `BuildMultilineTooltip` | INFERRED; no declaration found |
| `wt-framexml-BuildNewLineListString-974` | `BuildNewLineListString` | INFERRED; no declaration found |
| `wt-framexml-CanAccessObject-976` | `CanAccessObject` | INFERRED; no declaration found |
| `wt-framexml-ClassTrainerFrame_Hide-977` | `ClassTrainerFrame_Hide` | INFERRED; no declaration found |
| `wt-framexml-ClassTrainerFrame_Show-978` | `ClassTrainerFrame_Show` | INFERRED; no declaration found |
| `wt-framexml-CloseCalendarMenus-979` | `CloseCalendarMenus` | INFERRED; no declaration found |
| `wt-framexml-CommunitiesFrame_IsEnabled-980` | `CommunitiesFrame_IsEnabled` | INFERRED; no declaration found |
| `wt-framexml-CompactUnitFrame_GetOptionDisplayOnlyHealerPowerBars-981` | `CompactUnitFrame_GetOptionDisplayOnlyHealerPowerBars` | INFERRED; no declaration found |
| `wt-framexml-CompactUnitFrame_GetOptionDisplayPowerBar-982` | `CompactUnitFrame_GetOptionDisplayPowerBar` | INFERRED; no declaration found |
| `wt-framexml-CompactUnitFrame_GetOptionShowDispelIndicatorOverlay-983` | `CompactUnitFrame_GetOptionShowDispelIndicatorOverlay` | INFERRED; no declaration found |
| `wt-framexml-ConvertRGBtoColorString-984` | `ConvertRGBtoColorString` | INFERRED; no declaration found |
| `wt-framexml-DisplayTypeUnassignedSupported-985` | `DisplayTypeUnassignedSupported` | INFERRED; no declaration found |
| `wt-framexml-ExpansionTrial_CheckLoadUI-987` | `ExpansionTrial_CheckLoadUI` | INFERRED; no declaration found |
| `wt-framexml-FriendsFrame_CloseQuickJoinHelpTip-988` | `FriendsFrame_CloseQuickJoinHelpTip` | INFERRED; no declaration found |
| `wt-framexml-FriendsFrameAddFriendButton_OnClick-989` | `FriendsFrameAddFriendButton_OnClick` | INFERRED; no declaration found |
| `wt-framexml-FriendsFriends_InitButton-990` | `FriendsFriends_InitButton` | INFERRED; no declaration found |
| `wt-framexml-FriendsFriends_SetSelection-991` | `FriendsFriends_SetSelection` | INFERRED; no declaration found |
| `wt-framexml-FriendsFriendsButton_SetSelected-992` | `FriendsFriendsButton_SetSelected` | INFERRED; no declaration found |
| `wt-framexml-FriendsFriendsFrame_Close-993` | `FriendsFriendsFrame_Close` | INFERRED; no declaration found |
| `wt-framexml-GetDungeonNameWithDifficulty-994` | `GetDungeonNameWithDifficulty` | INFERRED; no declaration found |
| `wt-framexml-GetNotchHeight-995` | `GetNotchHeight` | INFERRED; no declaration found |
| `wt-framexml-GetPlayerKeyState-996` | `GetPlayerKeyState` | INFERRED; no declaration found |
| `wt-framexml-GetScaledCursorPositionForFrame-998` | `GetScaledCursorPositionForFrame` | INFERRED; no declaration found |
| `wt-framexml-GetScreenHeightScale-1000` | `GetScreenHeightScale` | INFERRED; no declaration found |
| `wt-framexml-GetScreenWidthScale-1001` | `GetScreenWidthScale` | INFERRED; no declaration found |
| `wt-framexml-GetSmoothProgressChange-1002` | `GetSmoothProgressChange` | INFERRED; no declaration found |
| `wt-framexml-GetSocialColoredName-1003` | `GetSocialColoredName` | INFERRED; no declaration found |
| `wt-framexml-GetSortedSelfResurrectOptions-1004` | `GetSortedSelfResurrectOptions` | INFERRED; no declaration found |
| `wt-framexml-GetUIParentOffset-1005` | `GetUIParentOffset` | INFERRED; no declaration found |
| `wt-framexml-HelpPlatesSupported-1006` | `HelpPlatesSupported` | INFERRED; no declaration found |
| `wt-framexml-IsFrameLockActive-1009` | `IsFrameLockActive` | INFERRED; no declaration found |
| `wt-framexml-IsFrameSmartShown-1010` | `IsFrameSmartShown` | INFERRED; no declaration found |
| `wt-framexml-IsLevelAtEffectiveMaxLevel-1011` | `IsLevelAtEffectiveMaxLevel` | INFERRED; no declaration found |
| `wt-framexml-KeyBindingFrame_LoadUI-1013` | `KeyBindingFrame_LoadUI` | INFERRED; no declaration found |
| `wt-framexml-LocalizePlayerFrame-1014` | `LocalizePlayerFrame` | INFERRED; no declaration found |
| `wt-framexml-LocalizezhCN-1015` | `LocalizezhCN` | INFERRED; no declaration found |
| `wt-framexml-LocalizezhTW-1016` | `LocalizezhTW` | INFERRED; no declaration found |
| `wt-framexml-MajorFactions_LoadUI-1018` | `MajorFactions_LoadUI` | INFERRED; no declaration found |
| `wt-framexml-MouseIsOver-1019` | `MouseIsOver` | INFERRED; no declaration found |
| `wt-framexml-NPE_CheckTutorials-1020` | `NPE_CheckTutorials` | INFERRED; no declaration found |
| `wt-framexml-OpenAchievementFrameToAchievement-1022` | `OpenAchievementFrameToAchievement` | INFERRED; no declaration found |
| `wt-framexml-OrderHallMissionFrame_EscapePressed-1023` | `OrderHallMissionFrame_EscapePressed` | INFERRED; no declaration found |
| `wt-framexml-OrderHallTalentFrame_EscapePressed-1024` | `OrderHallTalentFrame_EscapePressed` | INFERRED; no declaration found |
| `wt-framexml-OutfitterUI_LoadUI-1025` | `OutfitterUI_LoadUI` | INFERRED; no declaration found |
| `wt-framexml-QuickJoin_JoinQueueButtonOnClick-1028` | `QuickJoin_JoinQueueButtonOnClick` | INFERRED; no declaration found |
| `wt-framexml-RaidBossEmoteFrame_OnEvent-1029` | `RaidBossEmoteFrame_OnEvent` | INFERRED; no declaration found |
| `wt-framexml-RaidBossEmoteFrame_OnLoad-1030` | `RaidBossEmoteFrame_OnLoad` | INFERRED; no declaration found |
| `wt-framexml-RaidBrowser_IsEmpowered-1031` | `RaidBrowser_IsEmpowered` | INFERRED; no declaration found |
| `wt-framexml-RaidNotice_ClearSlot-1033` | `RaidNotice_ClearSlot` | INFERRED; no declaration found |
| `wt-framexml-RaidNotice_OnUpdate-1036` | `RaidNotice_OnUpdate` | INFERRED; no declaration found |
| `wt-framexml-RaidNotice_SetSlot-1037` | `RaidNotice_SetSlot` | INFERRED; no declaration found |
| `wt-framexml-RaidWarningFrame_OnEvent-1039` | `RaidWarningFrame_OnEvent` | INFERRED; no declaration found |
| `wt-framexml-RaidWarningFrame_OnLoad-1040` | `RaidWarningFrame_OnLoad` | INFERRED; no declaration found |
| `wt-framexml-RecentTimeDate-1041` | `RecentTimeDate` | INFERRED; no declaration found |
| `wt-framexml-RefreshAuras-1042` | `RefreshAuras` | INFERRED; no declaration found |
| `wt-framexml-RegisterNewFrameLock-1043` | `RegisterNewFrameLock` | INFERRED; no declaration found |
| `wt-framexml-RemoveFrameLock-1044` | `RemoveFrameLock` | INFERRED; no declaration found |
| `wt-framexml-ReverseQuestObjective-1045` | `ReverseQuestObjective` | INFERRED; no declaration found |
| `wt-framexml-SetFrameLock-1056` | `SetFrameLock` | INFERRED; no declaration found |
| `wt-framexml-ShowResurrectRequest-1057` | `ShowResurrectRequest` | INFERRED; no declaration found |
| `wt-framexml-SmartHide-1058` | `SmartHide` | INFERRED; no declaration found |
| `wt-framexml-SmartShow-1059` | `SmartShow` | INFERRED; no declaration found |
| `wt-framexml-TargetFrame_UpdateBuffAnchor-1061` | `TargetFrame_UpdateBuffAnchor` | INFERRED; no declaration found |
| `wt-framexml-TargetFrame_UpdateDebuffAnchor-1062` | `TargetFrame_UpdateDebuffAnchor` | INFERRED; no declaration found |
| `wt-framexml-ToggleLFGFrame-1063` | `ToggleLFGFrame` | INFERRED; no declaration found |
| `wt-framexml-ToggleRafPanel-1064` | `ToggleRafPanel` | INFERRED; no declaration found |
| `wt-framexml-ToggleRaidBrowser-1065` | `ToggleRaidBrowser` | INFERRED; no declaration found |
| `wt-framexml-ToggleWoWHackCharacterUI-1066` | `ToggleWoWHackCharacterUI` | INFERRED; no declaration found |
| `wt-framexml-TokenFrame_LoadUI-1067` | `TokenFrame_LoadUI` | INFERRED; no declaration found |
| `wt-framexml-TrialAccountCapReached_Inform-1068` | `TrialAccountCapReached_Inform` | INFERRED; no declaration found |
| `wt-framexml-UIDoFramesIntersect-1069` | `UIDoFramesIntersect` | INFERRED; no declaration found |
| `wt-framexml-UIParent_OnEvent-1071` | `UIParent_OnEvent` | INFERRED; no declaration found |
| `wt-framexml-UIParent_OnHide-1072` | `UIParent_OnHide` | INFERRED; no declaration found |
| `wt-framexml-UIParent_OnLoad-1073` | `UIParent_OnLoad` | INFERRED; no declaration found |
| `wt-framexml-UIParent_OnShow-1074` | `UIParent_OnShow` | INFERRED; no declaration found |
| `wt-framexml-UIParent_Shared_OnEvent-1075` | `UIParent_Shared_OnEvent` | INFERRED; no declaration found |
| `wt-framexml-UIParent_Shared_OnLoad-1076` | `UIParent_Shared_OnLoad` | INFERRED; no declaration found |
| `wt-framexml-UIParent_UpdateTopFramePositions-1077` | `UIParent_UpdateTopFramePositions` | INFERRED; no declaration found |
| `wt-framexml-UnitHasMana-1079` | `UnitHasMana` | INFERRED; no declaration found |
| `wt-framexml-UpdateFrameLock-1080` | `UpdateFrameLock` | INFERRED; no declaration found |
| `wt-framexml-UpdateUIElementsForClientScene-1081` | `UpdateUIElementsForClientScene` | INFERRED; no declaration found |
| `wt-framexml-WorldFrame_OnLoad-1082` | `WorldFrame_OnLoad` | INFERRED; no declaration found |
| `wt-framexml-WorldFrame_OnUpdate-1083` | `WorldFrame_OnUpdate` | INFERRED; no declaration found |
| `wt-framexml-WoWHackSpellsUI_LoadUI-1084` | `WoWHackSpellsUI_LoadUI` | INFERRED; no declaration found |

## W23 — Tests-only candidate: framexml

Section: framexml; triage: implemented-needs-proof; 5 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-CooldownManagerLayout_GetGroupBuffVisualAlerts-690` | `CooldownManagerLayout_GetGroupBuffVisualAlerts` | `src/ptr/compat_bootstrap.lua:132`; `retail/AddOns/Blizzard_CooldownViewer/CooldownViewerSettingsLayoutManager.lua:81` |
| `wt-framexml-CooldownManagerLayout_GetHiddenGroupBuffs-691` | `CooldownManagerLayout_GetHiddenGroupBuffs` | `src/ptr/compat_bootstrap.lua:142`; `retail/AddOns/Blizzard_CooldownViewer/CooldownViewerSettingsLayoutManager.lua:73` |
| `wt-framexml-CooldownManagerLayout_SetGroupBuffVisualAlerts-692` | `CooldownManagerLayout_SetGroupBuffVisualAlerts` | `src/ptr/compat_bootstrap.lua:137`; `retail/AddOns/Blizzard_CooldownViewer/CooldownViewerSettingsLayoutManager.lua:85` |
| `wt-framexml-CooldownManagerLayout_SetHiddenGroupBuffs-693` | `CooldownManagerLayout_SetHiddenGroupBuffs` | `src/ptr/compat_bootstrap.lua:147`; `retail/AddOns/Blizzard_CooldownViewer/CooldownViewerSettingsLayoutManager.lua:77` |
| `wt-framexml-LoadAddOnWithErrorHandling-834` | `LoadAddOnWithErrorHandling` | `src/ptr/compat_bootstrap.lua:153`; `retail/AddOns/Blizzard_SharedXML/AddOnUtil.lua:3` |

## W24 — Tests-only candidate: retail/AddOns/Blizzard_AchievementUI/Blizzard_AchievementUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ShowAchievementFrameForAchievement-889` | `ShowAchievementFrameForAchievement` | `retail/AddOns/Blizzard_AchievementUI/Blizzard_AchievementUI_Bootstrap.lua:19` |

## W25 — Tests-only candidate: retail/AddOns/Blizzard_AchievementUI/Mainline/Blizzard_AchievementUI.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-AchievementFrame_RefreshBackButton-624` | `AchievementFrame_RefreshBackButton` | `retail/AddOns/Blizzard_AchievementUI/Mainline/Blizzard_AchievementUI.lua:408` |
| `wt-framexml-AchievementFrame_SetComparisonMode-625` | `AchievementFrame_SetComparisonMode` | `retail/AddOns/Blizzard_AchievementUI/Mainline/Blizzard_AchievementUI.lua:3148` |

## W26 — Tests-only candidate: retail/AddOns/Blizzard_ActionBar/Shared/SpellFlyout.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-SpellFlyout_EscapePressed-936` | `SpellFlyout_EscapePressed` | `retail/AddOns/Blizzard_ActionBar/Shared/SpellFlyout.lua:8` |

## W27 — Tests-only candidate: retail/AddOns/Blizzard_AddFriend/AddFriendTemplates.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-AddFriendFrame_Show-627` | `AddFriendFrame_Show` | `retail/AddOns/Blizzard_AddFriend/AddFriendTemplates.lua:71` |

## W28 — Tests-only candidate: retail/AddOns/Blizzard_AlliedRacesUI/Blizzard_AlliedRacesUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-AlliedRacesFrame_TryShow-631` | `AlliedRacesFrame_TryShow` | `retail/AddOns/Blizzard_AlliedRacesUI/Blizzard_AlliedRacesUI_Bootstrap.lua:18` |

## W29 — Tests-only candidate: retail/AddOns/Blizzard_AnimaDiversionUI/Blizzard_AnimaDiversionUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-TryShowAnimaDiversionFrame-945` | `TryShowAnimaDiversionFrame` | `retail/AddOns/Blizzard_AnimaDiversionUI/Blizzard_AnimaDiversionUI_Bootstrap.lua:7` |

## W30 — Tests-only candidate: retail/AddOns/Blizzard_ArchaeologyUI/Blizzard_ArchaeologyUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ArchaeologyFrame_ToggleUI-635` | `ArchaeologyFrame_ToggleUI` | `retail/AddOns/Blizzard_ArchaeologyUI/Blizzard_ArchaeologyUI_Bootstrap.lua:13` |
| `wt-framexml-ArcheologyDigsiteProgressBar_OnSurveyCast-636` | `ArcheologyDigsiteProgressBar_OnSurveyCast` | `retail/AddOns/Blizzard_ArchaeologyUI/Blizzard_ArchaeologyUI_Bootstrap.lua:7` |

## W31 — Tests-only candidate: retail/AddOns/Blizzard_ArdenwealdGardening/Blizzard_ArdenwealdGardening_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ArdenwealdGardening_LoadUI-637` | `ArdenwealdGardening_LoadUI` | `retail/AddOns/Blizzard_ArdenwealdGardening/Blizzard_ArdenwealdGardening_Bootstrap.lua:3` |

## W32 — Tests-only candidate: retail/AddOns/Blizzard_ArtifactUI/Blizzard_ArtifactUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 3 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ArtifactFrame_OnTraitsRefunded-638` | `ArtifactFrame_OnTraitsRefunded` | `retail/AddOns/Blizzard_ArtifactUI/Blizzard_ArtifactUI_Bootstrap.lua:19` |
| `wt-framexml-ShowArtifactFrame-891` | `ShowArtifactFrame` | `retail/AddOns/Blizzard_ArtifactUI/Blizzard_ArtifactUI_Bootstrap.lua:7` |
| `wt-framexml-ShowArtifactRelicForgeFrame-892` | `ShowArtifactRelicForgeFrame` | `retail/AddOns/Blizzard_ArtifactUI/Blizzard_ArtifactUI_Bootstrap.lua:13` |

## W33 — Tests-only candidate: retail/AddOns/Blizzard_AuctionHouseUI/Shared/Blizzard_AuctionHouseUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-HideAuctionHouseFrame-785` | `HideAuctionHouseFrame` | `retail/AddOns/Blizzard_AuctionHouseUI/Shared/Blizzard_AuctionHouseUI_Bootstrap.lua:13` |
| `wt-framexml-ShowAuctionHouseFrame-893` | `ShowAuctionHouseFrame` | `retail/AddOns/Blizzard_AuctionHouseUI/Shared/Blizzard_AuctionHouseUI_Bootstrap.lua:7` |

## W34 — Tests-only candidate: retail/AddOns/Blizzard_AzeriteEssenceUI/Blizzard_AzeriteEssenceUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-AzeriteEssenceUI_LoadUI-651` | `AzeriteEssenceUI_LoadUI` | `retail/AddOns/Blizzard_AzeriteEssenceUI/Blizzard_AzeriteEssenceUI_Bootstrap.lua:3` |

## W35 — Tests-only candidate: retail/AddOns/Blizzard_AzeriteUI/Blizzard_AzeriteUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-AzeriteEmpoweredItemUI_LoadUI-650` | `AzeriteEmpoweredItemUI_LoadUI` | `retail/AddOns/Blizzard_AzeriteUI/Blizzard_AzeriteUI_Bootstrap.lua:3` |

## W36 — Tests-only candidate: retail/AddOns/Blizzard_BarbershopUI/Blizzard_BarberShopUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-HideBarberShopFrame-786` | `HideBarberShopFrame` | `retail/AddOns/Blizzard_BarbershopUI/Blizzard_BarberShopUI_Bootstrap.lua:13` |
| `wt-framexml-ShowBarberShopFrame-894` | `ShowBarberShopFrame` | `retail/AddOns/Blizzard_BarbershopUI/Blizzard_BarberShopUI_Bootstrap.lua:7` |

## W37 — Tests-only candidate: retail/AddOns/Blizzard_BattlefieldMap/Blizzard_BattlefieldMap_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-BattlefieldMap_ToggleUI-652` | `BattlefieldMap_ToggleUI` | `retail/AddOns/Blizzard_BattlefieldMap/Blizzard_BattlefieldMap_Bootstrap.lua:7` |

## W38 — Tests-only candidate: retail/AddOns/Blizzard_BehavioralMessaging/Blizzard_BehavioralMessaging_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 3 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-AddBehavioralMessagingTrayToStatusFrames-626` | `AddBehavioralMessagingTrayToStatusFrames` | `retail/AddOns/Blizzard_BehavioralMessaging/Blizzard_BehavioralMessaging_Bootstrap.lua:13` |
| `wt-framexml-BehavioralMessaging_LoadUI-653` | `BehavioralMessaging_LoadUI` | `retail/AddOns/Blizzard_BehavioralMessaging/Blizzard_BehavioralMessaging_Bootstrap.lua:3` |
| `wt-framexml-BehavioralMessagingTray_OnNotification-654` | `BehavioralMessagingTray_OnNotification` | `retail/AddOns/Blizzard_BehavioralMessaging/Blizzard_BehavioralMessaging_Bootstrap.lua:7` |

## W39 — Tests-only candidate: retail/AddOns/Blizzard_BlackMarketUI/Blizzard_BlackMarketUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-HideBlackMarketFrame-787` | `HideBlackMarketFrame` | `retail/AddOns/Blizzard_BlackMarketUI/Blizzard_BlackMarketUI_Bootstrap.lua:13` |
| `wt-framexml-ShowBlackMarketFrame-895` | `ShowBlackMarketFrame` | `retail/AddOns/Blizzard_BlackMarketUI/Blizzard_BlackMarketUI_Bootstrap.lua:7` |

## W40 — Tests-only candidate: retail/AddOns/Blizzard_BoostTutorial/Blizzard_BoostTutorial_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-BoostTutorial_LoadUI-662` | `BoostTutorial_LoadUI` | `retail/AddOns/Blizzard_BoostTutorial/Blizzard_BoostTutorial_Bootstrap.lua:12` |

## W41 — Tests-only candidate: retail/AddOns/Blizzard_ChallengesUI/Blizzard_ChallengesUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ChallengeModeCompleteBanner_OnChallengeModeCompleted-664` | `ChallengeModeCompleteBanner_OnChallengeModeCompleted` | `retail/AddOns/Blizzard_ChallengesUI/Blizzard_ChallengesUI_Bootstrap.lua:7` |
| `wt-framexml-ShowChallengesKeystoneFrame-896` | `ShowChallengesKeystoneFrame` | `retail/AddOns/Blizzard_ChallengesUI/Blizzard_ChallengesUI_Bootstrap.lua:13` |

## W42 — Tests-only candidate: retail/AddOns/Blizzard_ChatFrame/Mainline/ChatConfigFrame.lua

Section: framexml; triage: implemented-needs-proof; 3 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ChatAdditionalColor_OpenColorPicker-665` | `ChatAdditionalColor_OpenColorPicker` | `retail/AddOns/Blizzard_ChatFrame/Mainline/ChatConfigFrame.lua:1539` |
| `wt-framexml-GetChatAdditionalColor-757` | `GetChatAdditionalColor` | `retail/AddOns/Blizzard_ChatFrame/Mainline/ChatConfigFrame.lua:1601` |
| `wt-framexml-IsTypeAdditionalChatColor-822` | `IsTypeAdditionalChatColor` | `retail/AddOns/Blizzard_ChatFrame/Mainline/ChatConfigFrame.lua:1597` |

## W43 — Tests-only candidate: retail/AddOns/Blizzard_ChatFrameBase/Shared/ChatFrameUtil.lua

Section: framexml; triage: implemented-needs-proof; 3 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ChatFrameUtil.DiscordNameColorize-666` | `ChatFrameUtil.DiscordNameColorize` | `retail/AddOns/Blizzard_ChatFrameBase/Shared/ChatFrameUtil.lua:1049` |
| `wt-framexml-ChatFrameUtil.FormatDiscordMessage-667` | `ChatFrameUtil.FormatDiscordMessage` | `retail/AddOns/Blizzard_ChatFrameBase/Shared/ChatFrameUtil.lua:1079` |
| `wt-framexml-ChatFrameUtil.GetNameForDiscordMessage-668` | `ChatFrameUtil.GetNameForDiscordMessage` | `retail/AddOns/Blizzard_ChatFrameBase/Shared/ChatFrameUtil.lua:1059` |

## W44 — Tests-only candidate: retail/AddOns/Blizzard_Collections/Blizzard_Collections_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ShowHeirloomsJournalToClosestUpgradeablePage-903` | `ShowHeirloomsJournalToClosestUpgradeablePage` | `retail/AddOns/Blizzard_Collections/Blizzard_Collections_Bootstrap.lua:42` |

## W45 — Tests-only candidate: retail/AddOns/Blizzard_ColorPickerFrame/Shared/ColorPickerFrame.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-OpacityFrame_EscapePressed-858` | `OpacityFrame_EscapePressed` | `retail/AddOns/Blizzard_ColorPickerFrame/Shared/ColorPickerFrame.lua:1` |

## W46 — Tests-only candidate: retail/AddOns/Blizzard_CombatText/Blizzard_CombatText_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-CombatText_LoadUI-680` | `CombatText_LoadUI` | `retail/AddOns/Blizzard_CombatText/Blizzard_CombatText_Bootstrap.lua:3` |

## W47 — Tests-only candidate: retail/AddOns/Blizzard_Contribution/Blizzard_Contribution_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ContributionCollectionFrame_LoadUI-689` | `ContributionCollectionFrame_LoadUI` | `retail/AddOns/Blizzard_Contribution/Blizzard_Contribution_Bootstrap.lua:3` |

## W48 — Tests-only candidate: retail/AddOns/Blizzard_CooldownViewer/CooldownViewer.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-CDMDebugGetDebugger-663` | `CDMDebugGetDebugger` | `retail/AddOns/Blizzard_CooldownViewer/CooldownViewer.lua:10` |

## W49 — Tests-only candidate: retail/AddOns/Blizzard_CooldownViewer/CooldownViewerDraggedItemBase.lua

Section: framexml; triage: implemented-needs-proof; 3 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-CooldownViewerDraggedItem_Clear-697` | `CooldownViewerDraggedItem_Clear` | `retail/AddOns/Blizzard_CooldownViewer/CooldownViewerDraggedItemBase.lua:24` |
| `wt-framexml-CooldownViewerDraggedItem_Pickup-698` | `CooldownViewerDraggedItem_Pickup` | `retail/AddOns/Blizzard_CooldownViewer/CooldownViewerDraggedItemBase.lua:16` |
| `wt-framexml-CooldownViewerDraggedItem_SetIsLegalTarget-699` | `CooldownViewerDraggedItem_SetIsLegalTarget` | `retail/AddOns/Blizzard_CooldownViewer/CooldownViewerDraggedItemBase.lua:31` |

## W50 — Tests-only candidate: retail/AddOns/Blizzard_CooldownViewer/CooldownViewerItemData.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-CooldownViewer_MarkAuraCacheDirty-694` | `CooldownViewer_MarkAuraCacheDirty` | `retail/AddOns/Blizzard_CooldownViewer/CooldownViewerItemData.lua:690` |

## W51 — Tests-only candidate: retail/AddOns/Blizzard_CooldownViewer/CooldownViewerSettingsAlerts.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-CooldownViewerContextMenu_AddAlertEntryButton-695` | `CooldownViewerContextMenu_AddAlertEntryButton` | `retail/AddOns/Blizzard_CooldownViewer/CooldownViewerSettingsAlerts.lua:193` |
| `wt-framexml-CooldownViewerContextMenu_AddNewAlertButton-696` | `CooldownViewerContextMenu_AddNewAlertButton` | `retail/AddOns/Blizzard_CooldownViewer/CooldownViewerSettingsAlerts.lua:175` |

## W52 — Tests-only candidate: retail/AddOns/Blizzard_CooldownViewer/CooldownViewerUtil.lua

Section: framexml; triage: implemented-needs-proof; 4 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-CooldownViewerUtil.AddSoundAlertRadio-700` | `CooldownViewerUtil.AddSoundAlertRadio` | `retail/AddOns/Blizzard_CooldownViewer/CooldownViewerUtil.lua:87` |
| `wt-framexml-CooldownViewerUtil.BuildSoundMenus-701` | `CooldownViewerUtil.BuildSoundMenus` | `retail/AddOns/Blizzard_CooldownViewer/CooldownViewerUtil.lua:114` |
| `wt-framexml-CooldownViewerUtil.GetSoundTypeSoundKit-702` | `CooldownViewerUtil.GetSoundTypeSoundKit` | `retail/AddOns/Blizzard_CooldownViewer/CooldownViewerUtil.lua:136` |
| `wt-framexml-CooldownViewerUtil.GetSoundTypeText-703` | `CooldownViewerUtil.GetSoundTypeText` | `retail/AddOns/Blizzard_CooldownViewer/CooldownViewerUtil.lua:131` |

## W53 — Tests-only candidate: retail/AddOns/Blizzard_CovenantCallings/Blizzard_CovenantCallings_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-CovenantCallings_LoadUI-704` | `CovenantCallings_LoadUI` | `retail/AddOns/Blizzard_CovenantCallings/Blizzard_CovenantCallings_Bootstrap.lua:3` |

## W54 — Tests-only candidate: retail/AddOns/Blizzard_CovenantPreviewUI/Blizzard_CovenantPreviewUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-TryShowCovenantPreviewFrame-946` | `TryShowCovenantPreviewFrame` | `retail/AddOns/Blizzard_CovenantPreviewUI/Blizzard_CovenantPreviewUI_Bootstrap.lua:7` |

## W55 — Tests-only candidate: retail/AddOns/Blizzard_DebugTools/Blizzard_DebugTools_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-DebugTools_LoadUI-705` | `DebugTools_LoadUI` | `retail/AddOns/Blizzard_DebugTools/Blizzard_DebugTools_Bootstrap.lua:3` |

## W56 — Tests-only candidate: retail/AddOns/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-getglobal-1085` | `getglobal` | `retail/AddOns/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:9`; `retail/AddOns/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:9` |
| `wt-framexml-setglobal-1086` | `setglobal` | `retail/AddOns/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:14`; `retail/AddOns/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:14` |

## W57 — Tests-only candidate: retail/AddOns/Blizzard_EditMode/Shared/EditModeManager.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-EditModeManagerFrame_EscapePressed-711` | `EditModeManagerFrame_EscapePressed` | `retail/AddOns/Blizzard_EditMode/Shared/EditModeManager.lua:12` |

## W58 — Tests-only candidate: retail/AddOns/Blizzard_EncounterJournal/Blizzard_EncounterJournal_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-OpenEncounterJournalToJourney-859` | `OpenEncounterJournalToJourney` | `retail/AddOns/Blizzard_EncounterJournal/Blizzard_EncounterJournal_Bootstrap.lua:19` |
| `wt-framexml-OpenEncounterJournalToTieredEntrance-860` | `OpenEncounterJournalToTieredEntrance` | `retail/AddOns/Blizzard_EncounterJournal/Blizzard_EncounterJournal_Bootstrap.lua:25` |

## W59 — Tests-only candidate: retail/AddOns/Blizzard_EncounterJournal/Mainline/Blizzard_EncounterJournal.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-EncounterJournal_OpenToTieredEntrance-712` | `EncounterJournal_OpenToTieredEntrance` | `retail/AddOns/Blizzard_EncounterJournal/Mainline/Blizzard_EncounterJournal.lua:2728` |

## W60 — Tests-only candidate: retail/AddOns/Blizzard_EventTrace/Blizzard_EventTrace_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-EventTrace_LoadUI-713` | `EventTrace_LoadUI` | `retail/AddOns/Blizzard_EventTrace/Blizzard_EventTrace_Bootstrap.lua:3` |

## W61 — Tests-only candidate: retail/AddOns/Blizzard_ExpansionTrial/Classic/Blizzard_ExpansionTrial_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ExpansionTrial_LoadUI-714` | `ExpansionTrial_LoadUI` | `retail/AddOns/Blizzard_ExpansionTrial/Classic/Blizzard_ExpansionTrial_Bootstrap.lua:1` |

## W62 — Tests-only candidate: retail/AddOns/Blizzard_FlightMap/Blizzard_FlightMap_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ShowFlightMapFrame-897` | `ShowFlightMapFrame` | `retail/AddOns/Blizzard_FlightMap/Blizzard_FlightMap_Bootstrap.lua:7` |

## W63 — Tests-only candidate: retail/AddOns/Blizzard_FrameXML/GhostFrame.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-SetGhostFrameShown-882` | `SetGhostFrameShown` | `retail/AddOns/Blizzard_FrameXML/GhostFrame.lua:3` |

## W64 — Tests-only candidate: retail/AddOns/Blizzard_FrameXML/QuestSession.lua

Section: framexml; triage: implemented-needs-proof; 3 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-HandleQuestSessionInviteToPartyConfirmation-783` | `HandleQuestSessionInviteToPartyConfirmation` | `retail/AddOns/Blizzard_FrameXML/QuestSession.lua:924` |
| `wt-framexml-ShowQuestSessionGroupInviteConfirmation-914` | `ShowQuestSessionGroupInviteConfirmation` | `retail/AddOns/Blizzard_FrameXML/QuestSession.lua:916` |
| `wt-framexml-ShowQuestSessionGroupInviteReceivedConfirmation-915` | `ShowQuestSessionGroupInviteReceivedConfirmation` | `retail/AddOns/Blizzard_FrameXML/QuestSession.lua:920` |

## W65 — Tests-only candidate: retail/AddOns/Blizzard_FrameXMLUtil/AuraUtil.lua

Section: framexml; triage: implemented-needs-proof; 11 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-AuraUtil.AuraInstanceIDOnlyAuraCompare-639` | `AuraUtil.AuraInstanceIDOnlyAuraCompare` | `retail/AddOns/Blizzard_FrameXMLUtil/AuraUtil.lua:266` |
| `wt-framexml-AuraUtil.ExpirationAuraCompare-640` | `AuraUtil.ExpirationAuraCompare` | `retail/AddOns/Blizzard_FrameXMLUtil/AuraUtil.lua:196` |
| `wt-framexml-AuraUtil.ExpirationOnlyAuraCompare-641` | `AuraUtil.ExpirationOnlyAuraCompare` | `retail/AddOns/Blizzard_FrameXMLUtil/AuraUtil.lua:221` |
| `wt-framexml-AuraUtil.GetAuraBorderColor-642` | `AuraUtil.GetAuraBorderColor` | `retail/AddOns/Blizzard_FrameXMLUtil/AuraUtil.lua:609` |
| `wt-framexml-AuraUtil.GetAuraDispelTypeIcon-643` | `AuraUtil.GetAuraDispelTypeIcon` | `retail/AddOns/Blizzard_FrameXMLUtil/AuraUtil.lua:628` |
| `wt-framexml-AuraUtil.GetUnitAuras-644` | `AuraUtil.GetUnitAuras` | `retail/AddOns/Blizzard_FrameXMLUtil/AuraUtil.lua:35` |
| `wt-framexml-AuraUtil.ImportantOnlyAuraCompare-645` | `AuraUtil.ImportantOnlyAuraCompare` | `retail/AddOns/Blizzard_FrameXMLUtil/AuraUtil.lua:185` |
| `wt-framexml-AuraUtil.IsValidFilterString-646` | `AuraUtil.IsValidFilterString` | `retail/AddOns/Blizzard_FrameXMLUtil/AuraUtil.lua:298` |
| `wt-framexml-AuraUtil.NameAuraCompare-647` | `AuraUtil.NameAuraCompare` | `retail/AddOns/Blizzard_FrameXMLUtil/AuraUtil.lua:232` |
| `wt-framexml-AuraUtil.NameOnlyAuraCompare-648` | `AuraUtil.NameOnlyAuraCompare` | `retail/AddOns/Blizzard_FrameXMLUtil/AuraUtil.lua:256` |
| `wt-framexml-AuraUtil.SetAuraDispelTypeIcon-649` | `AuraUtil.SetAuraDispelTypeIcon` | `retail/AddOns/Blizzard_FrameXMLUtil/AuraUtil.lua:633` |

## W66 — Tests-only candidate: retail/AddOns/Blizzard_FrameXMLUtil/FadingFrame.lua

Section: framexml; triage: implemented-needs-proof; 7 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-FadingFrame_CopyTextScalingTime-715` | `FadingFrame_CopyTextScalingTime` | `retail/AddOns/Blizzard_FrameXMLUtil/FadingFrame.lua:75` |
| `wt-framexml-FadingFrame_GetTextScalingMinHeight-716` | `FadingFrame_GetTextScalingMinHeight` | `retail/AddOns/Blizzard_FrameXMLUtil/FadingFrame.lua:63` |
| `wt-framexml-FadingFrame_InitSlot-717` | `FadingFrame_InitSlot` | `retail/AddOns/Blizzard_FrameXMLUtil/FadingFrame.lua:103` |
| `wt-framexml-FadingFrame_SetTextScaling-718` | `FadingFrame_SetTextScaling` | `retail/AddOns/Blizzard_FrameXMLUtil/FadingFrame.lua:56` |
| `wt-framexml-FadingFrame_StartTextScaling-719` | `FadingFrame_StartTextScaling` | `retail/AddOns/Blizzard_FrameXMLUtil/FadingFrame.lua:67` |
| `wt-framexml-FadingFrame_StopTextScaling-720` | `FadingFrame_StopTextScaling` | `retail/AddOns/Blizzard_FrameXMLUtil/FadingFrame.lua:71` |
| `wt-framexml-FadingFrame_UpdateTextScaling-721` | `FadingFrame_UpdateTextScaling` | `retail/AddOns/Blizzard_FrameXMLUtil/FadingFrame.lua:79` |

## W67 — Tests-only candidate: retail/AddOns/Blizzard_FrameXMLUtil/ItemUtil.lua

Section: framexml; triage: implemented-needs-proof; 5 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ItemUtil.DisplayEquipSlotTooltip-824` | `ItemUtil.DisplayEquipSlotTooltip` | `retail/AddOns/Blizzard_FrameXMLUtil/ItemUtil.lua:386` |
| `wt-framexml-ItemUtil.GetEmptyEquipSlotTooltipForSlotName-825` | `ItemUtil.GetEmptyEquipSlotTooltipForSlotName` | `retail/AddOns/Blizzard_FrameXMLUtil/ItemUtil.lua:369` |
| `wt-framexml-ItemUtil.GetEmptyEquipSlotTooltip-826` | `ItemUtil.GetEmptyEquipSlotTooltip` | `retail/AddOns/Blizzard_FrameXMLUtil/ItemUtil.lua:377` |
| `wt-framexml-ItemUtil.GetEquipSlotTexture-827` | `ItemUtil.GetEquipSlotTexture` | `retail/AddOns/Blizzard_FrameXMLUtil/ItemUtil.lua:407` |
| `wt-framexml-ItemUtil.GetValidatedItemLocation-828` | `ItemUtil.GetValidatedItemLocation` | `retail/AddOns/Blizzard_FrameXMLUtil/ItemUtil.lua:358` |

## W68 — Tests-only candidate: retail/AddOns/Blizzard_FrameXMLUtil/Mainline/DifficultyUtil.lua

Section: framexml; triage: implemented-needs-proof; 5 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-DifficultyUtil.GetCreatureDifficultyColor-706` | `DifficultyUtil.GetCreatureDifficultyColor` | `retail/AddOns/Blizzard_FrameXMLUtil/Mainline/DifficultyUtil.lua:29` |
| `wt-framexml-DifficultyUtil.GetDifficultyColor-707` | `DifficultyUtil.GetDifficultyColor` | `retail/AddOns/Blizzard_FrameXMLUtil/Mainline/DifficultyUtil.lua:1` |
| `wt-framexml-DifficultyUtil.GetQuestDifficultyColor-708` | `DifficultyUtil.GetQuestDifficultyColor` | `retail/AddOns/Blizzard_FrameXMLUtil/Mainline/DifficultyUtil.lua:17` |
| `wt-framexml-DifficultyUtil.GetRelativeDifficultyColor-709` | `DifficultyUtil.GetRelativeDifficultyColor` | `retail/AddOns/Blizzard_FrameXMLUtil/Mainline/DifficultyUtil.lua:33` |
| `wt-framexml-DifficultyUtil.GetScalingQuestDifficultyColor-710` | `DifficultyUtil.GetScalingQuestDifficultyColor` | `retail/AddOns/Blizzard_FrameXMLUtil/Mainline/DifficultyUtil.lua:48` |

## W69 — Tests-only candidate: retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsFrame.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-FriendsFrame_HideAllPotentialSubFrames-722` | `FriendsFrame_HideAllPotentialSubFrames` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsFrame.lua:90` |
| `wt-framexml-ToggleRAFPanel-943` | `ToggleRAFPanel` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsFrame.lua:1663` |

## W70 — Tests-only candidate: retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua

Section: framexml; triage: implemented-needs-proof; 29 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-FriendsListUtil.BuildCharacterClassDisplayText-723` | `FriendsListUtil.BuildCharacterClassDisplayText` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:384` |
| `wt-framexml-FriendsListUtil.BuildCharacterLevelDisplayText-724` | `FriendsListUtil.BuildCharacterLevelDisplayText` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:374` |
| `wt-framexml-FriendsListUtil.BuildCharacterNameDisplayText-725` | `FriendsListUtil.BuildCharacterNameDisplayText` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:359` |
| `wt-framexml-FriendsListUtil.BuildFriendNameDisplayText-726` | `FriendsListUtil.BuildFriendNameDisplayText` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:345` |
| `wt-framexml-FriendsListUtil.BuildLocationDisplayText-727` | `FriendsListUtil.BuildLocationDisplayText` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:427` |
| `wt-framexml-FriendsListUtil.BuildTooltipBroadcastText-728` | `FriendsListUtil.BuildTooltipBroadcastText` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:455` |
| `wt-framexml-FriendsListUtil.GameStateUsesFactions-729` | `FriendsListUtil.GameStateUsesFactions` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:31` |
| `wt-framexml-FriendsListUtil.GetBattleNetFriendGameAccountInfoIfExactlyOneDirectInviteTargetExists-730` | `FriendsListUtil.GetBattleNetFriendGameAccountInfoIfExactlyOneDirectInviteTargetExists` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:169` |
| `wt-framexml-FriendsListUtil.GetBattleNetFriendInviteInfo-731` | `FriendsListUtil.GetBattleNetFriendInviteInfo` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:501` |
| `wt-framexml-FriendsListUtil.GetBattleNetFriendInviteTypeLabel-732` | `FriendsListUtil.GetBattleNetFriendInviteTypeLabel` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:485` |
| `wt-framexml-FriendsListUtil.GetBattleNetFriendPartyInviteRestrictionText-733` | `FriendsListUtil.GetBattleNetFriendPartyInviteRestrictionText` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:224` |
| `wt-framexml-FriendsListUtil.GetBattleNetFriendPartyInviteRestriction-734` | `FriendsListUtil.GetBattleNetFriendPartyInviteRestriction` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:125` |
| `wt-framexml-FriendsListUtil.GetFormattedCharacterName-735` | `FriendsListUtil.GetFormattedCharacterName` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:314` |
| `wt-framexml-FriendsListUtil.GetFriendAccountNameText-736` | `FriendsListUtil.GetFriendAccountNameText` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:351` |
| `wt-framexml-FriendsListUtil.GetFriendNameColorForFriendType-737` | `FriendsListUtil.GetFriendNameColorForFriendType` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:333` |
| `wt-framexml-FriendsListUtil.GetFriendNameDisplayColor-738` | `FriendsListUtil.GetFriendNameDisplayColor` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:337` |
| `wt-framexml-FriendsListUtil.GetFriendNameOfflineDisplayColor-739` | `FriendsListUtil.GetFriendNameOfflineDisplayColor` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:329` |
| `wt-framexml-FriendsListUtil.GetGameAccountPartyInviteRestriction-740` | `FriendsListUtil.GetGameAccountPartyInviteRestriction` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:69` |
| `wt-framexml-FriendsListUtil.GetLastOnlineText-741` | `FriendsListUtil.GetLastOnlineText` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:299` |
| `wt-framexml-FriendsListUtil.GetRegionName-742` | `FriendsListUtil.GetRegionName` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:447` |
| `wt-framexml-FriendsListUtil.GetRelativeTimeText-743` | `FriendsListUtil.GetRelativeTimeText` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:268` |
| `wt-framexml-FriendsListUtil.HasMultipleGameAccounts-744` | `FriendsListUtil.HasMultipleGameAccounts` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:565` |
| `wt-framexml-FriendsListUtil.InviteOrRequestToJoin-745` | `FriendsListUtil.InviteOrRequestToJoin` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:534` |
| `wt-framexml-FriendsListUtil.IsPlayingDifferentWoWProject-746` | `FriendsListUtil.IsPlayingDifferentWoWProject` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:11` |
| `wt-framexml-FriendsListUtil.IsPlayingSameWoWProject-747` | `FriendsListUtil.IsPlayingSameWoWProject` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:19` |
| `wt-framexml-FriendsListUtil.IsPlayingWoW-748` | `FriendsListUtil.IsPlayingWoW` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:3` |
| `wt-framexml-FriendsListUtil.IsRequestInviteType-749` | `FriendsListUtil.IsRequestInviteType` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:495` |
| `wt-framexml-FriendsListUtil.IsTitleFriend-750` | `FriendsListUtil.IsTitleFriend` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:27` |
| `wt-framexml-FriendsListUtil.ShouldShowRichPresenceOnly-751` | `FriendsListUtil.ShouldShowRichPresenceOnly` | `retail/AddOns/Blizzard_FriendsFrame/Mainline/FriendsListUtil.lua:228` |

## W71 — Tests-only candidate: retail/AddOns/Blizzard_GMChatUI/Blizzard_GMChatUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 3 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-AddGMChatStatusFrameToStatusFrames-628` | `AddGMChatStatusFrameToStatusFrames` | `retail/AddOns/Blizzard_GMChatUI/Blizzard_GMChatUI_Bootstrap.lua:28` |
| `wt-framexml-GMChatFrame_OnWhisperFromGM-767` | `GMChatFrame_OnWhisperFromGM` | `retail/AddOns/Blizzard_GMChatUI/Blizzard_GMChatUI_Bootstrap.lua:7` |
| `wt-framexml-RestoreGMChatFrameSession-881` | `RestoreGMChatFrameSession` | `retail/AddOns/Blizzard_GMChatUI/Blizzard_GMChatUI_Bootstrap.lua:15` |

## W72 — Tests-only candidate: retail/AddOns/Blizzard_GameMenu/Shared/GameMenuFrame.lua

Section: framexml; triage: implemented-needs-proof; 3 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-GameMenuFrame_EscapePressed-752` | `GameMenuFrame_EscapePressed` | `retail/AddOns/Blizzard_GameMenu/Shared/GameMenuFrame.lua:26` |
| `wt-framexml-GameMenuFrame_IsShown-753` | `GameMenuFrame_IsShown` | `retail/AddOns/Blizzard_GameMenu/Shared/GameMenuFrame.lua:22` |
| `wt-framexml-GameMenuFrame_Show-754` | `GameMenuFrame_Show` | `retail/AddOns/Blizzard_GameMenu/Shared/GameMenuFrame.lua:38` |

## W73 — Tests-only candidate: retail/AddOns/Blizzard_GameMenuEsc/Blizzard_GameMenuEsc.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-RegisterGameMenuEscHandler-877` | `RegisterGameMenuEscHandler` | `retail/AddOns/Blizzard_GameMenuEsc/Blizzard_GameMenuEsc.lua:76` |

## W74 — Tests-only candidate: retail/AddOns/Blizzard_GarrisonBase/GarrisonBaseUtils.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-GetGarrisonMissionFrameNameForFollowerType-760` | `GetGarrisonMissionFrameNameForFollowerType` | `retail/AddOns/Blizzard_GarrisonBase/GarrisonBaseUtils.lua:335` |
| `wt-framexml-GetGarrisonTypeForFollowerType-761` | `GetGarrisonTypeForFollowerType` | `retail/AddOns/Blizzard_GarrisonBase/GarrisonBaseUtils.lua:345` |

## W75 — Tests-only candidate: retail/AddOns/Blizzard_GarrisonUI/Mainline/Blizzard_GarrisonUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 7 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-HideGarrisonMissionFrames-788` | `HideGarrisonMissionFrames` | `retail/AddOns/Blizzard_GarrisonUI/Mainline/Blizzard_GarrisonUI_Bootstrap.lua:44` |
| `wt-framexml-HideGarrisonShipyardFrame-789` | `HideGarrisonShipyardFrame` | `retail/AddOns/Blizzard_GarrisonUI/Mainline/Blizzard_GarrisonUI_Bootstrap.lua:59` |
| `wt-framexml-ShowAdventureMapFrameForFollowerType-890` | `ShowAdventureMapFrameForFollowerType` | `retail/AddOns/Blizzard_GarrisonUI/Mainline/Blizzard_GarrisonUI_Bootstrap.lua:34` |
| `wt-framexml-ShowGarrisonCapacitiveDisplayFrame-898` | `ShowGarrisonCapacitiveDisplayFrame` | `retail/AddOns/Blizzard_GarrisonUI/Mainline/Blizzard_GarrisonUI_Bootstrap.lua:65` |
| `wt-framexml-ShowGarrisonMissionFrameForFollowerType-899` | `ShowGarrisonMissionFrameForFollowerType` | `retail/AddOns/Blizzard_GarrisonUI/Mainline/Blizzard_GarrisonUI_Bootstrap.lua:19` |
| `wt-framexml-ShowGarrisonRecruiterFrame-900` | `ShowGarrisonRecruiterFrame` | `retail/AddOns/Blizzard_GarrisonUI/Mainline/Blizzard_GarrisonUI_Bootstrap.lua:71` |
| `wt-framexml-ShowGarrisonShipyardFrame-901` | `ShowGarrisonShipyardFrame` | `retail/AddOns/Blizzard_GarrisonUI/Mainline/Blizzard_GarrisonUI_Bootstrap.lua:52` |

## W76 — Tests-only candidate: retail/AddOns/Blizzard_GroupFinder/Mainline/LFGList.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-LFGListApplicationViewer_OpenEditMode-832` | `LFGListApplicationViewer_OpenEditMode` | `retail/AddOns/Blizzard_GroupFinder/Mainline/LFGList.lua:2120` |
| `wt-framexml-LFGListApplicationViewerRemoveEntryButton_OnClick-833` | `LFGListApplicationViewerRemoveEntryButton_OnClick` | `retail/AddOns/Blizzard_GroupFinder/Mainline/LFGList.lua:2126` |

## W77 — Tests-only candidate: retail/AddOns/Blizzard_GuildBankUI/Blizzard_GuildBankUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-HideGuildBankFrame-791` | `HideGuildBankFrame` | `retail/AddOns/Blizzard_GuildBankUI/Blizzard_GuildBankUI_Bootstrap.lua:16` |
| `wt-framexml-ShowGuildBankFrame-902` | `ShowGuildBankFrame` | `retail/AddOns/Blizzard_GuildBankUI/Blizzard_GuildBankUI_Bootstrap.lua:7` |

## W78 — Tests-only candidate: retail/AddOns/Blizzard_GuildControlUI/Blizzard_GuildControlUI.lua

Section: framexml; triage: implemented-needs-proof; 13 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-GuildControlDiscord_Loaded_OnEvent-769` | `GuildControlDiscord_Loaded_OnEvent` | `retail/AddOns/Blizzard_GuildControlUI/Blizzard_GuildControlUI.lua:305` |
| `wt-framexml-GuildControlDiscord_Loaded_OnLoad-770` | `GuildControlDiscord_Loaded_OnLoad` | `retail/AddOns/Blizzard_GuildControlUI/Blizzard_GuildControlUI.lua:297` |
| `wt-framexml-GuildControlDiscord_SetGuildSettingsCheckboxes-771` | `GuildControlDiscord_SetGuildSettingsCheckboxes` | `retail/AddOns/Blizzard_GuildControlUI/Blizzard_GuildControlUI.lua:301` |
| `wt-framexml-GuildControlRankDiscord_OnLoad-772` | `GuildControlRankDiscord_OnLoad` | `retail/AddOns/Blizzard_GuildControlUI/Blizzard_GuildControlUI.lua:276` |
| `wt-framexml-GuildControlUI_Discord_HideAll-773` | `GuildControlUI_Discord_HideAll` | `retail/AddOns/Blizzard_GuildControlUI/Blizzard_GuildControlUI.lua:259` |
| `wt-framexml-GuildControlUI_Discord_Update-774` | `GuildControlUI_Discord_Update` | `retail/AddOns/Blizzard_GuildControlUI/Blizzard_GuildControlUI.lua:311` |
| `wt-framexml-GuildControlUI_DiscordFrame_OnLoad-775` | `GuildControlUI_DiscordFrame_OnLoad` | `retail/AddOns/Blizzard_GuildControlUI/Blizzard_GuildControlUI.lua:615` |
| `wt-framexml-GuildControlUI_OnShow-777` | `GuildControlUI_OnShow` | `retail/AddOns/Blizzard_GuildControlUI/Blizzard_GuildControlUI.lua:14` |
| `wt-framexml-GuildControlUI_SetupDiscord-778` | `GuildControlUI_SetupDiscord` | `retail/AddOns/Blizzard_GuildControlUI/Blizzard_GuildControlUI.lua:249` |
| `wt-framexml-GuildControlUI_SetupSelected-779` | `GuildControlUI_SetupSelected` | `retail/AddOns/Blizzard_GuildControlUI/Blizzard_GuildControlUI.lua:20` |
| `wt-framexml-GuildControlUI_Setup-780` | `GuildControlUI_Setup` | `retail/AddOns/Blizzard_GuildControlUI/Blizzard_GuildControlUI.lua:77` |
| `wt-framexml-GuildControlUI_UnlinkDiscord-782` | `GuildControlUI_UnlinkDiscord` | `retail/AddOns/Blizzard_GuildControlUI/Blizzard_GuildControlUI.lua:254` |
| `wt-framexml-ResetDiscordSettings-880` | `ResetDiscordSettings` | `retail/AddOns/Blizzard_GuildControlUI/Blizzard_GuildControlUI.lua:7` |

## W79 — Tests-only candidate: retail/AddOns/Blizzard_GuildControlUI/Blizzard_GuildControlUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-GuildControlUI_LoadUI-776` | `GuildControlUI_LoadUI` | `retail/AddOns/Blizzard_GuildControlUI/Blizzard_GuildControlUI_Bootstrap.lua:3` |
| `wt-framexml-GuildControlUI_Show-781` | `GuildControlUI_Show` | `retail/AddOns/Blizzard_GuildControlUI/Blizzard_GuildControlUI_Bootstrap.lua:7` |

## W80 — Tests-only candidate: retail/AddOns/Blizzard_HelpFrame/HelpFrame.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-AddTicketStatusFrameToStatusFrames-629` | `AddTicketStatusFrameToStatusFrames` | `retail/AddOns/Blizzard_HelpFrame/HelpFrame.lua:280` |
| `wt-framexml-HelpFrame_EscapePressed-784` | `HelpFrame_EscapePressed` | `retail/AddOns/Blizzard_HelpFrame/HelpFrame.lua:116` |

## W81 — Tests-only candidate: retail/AddOns/Blizzard_HousingBulletinBoard/Blizzard_HousingBulletinBoard_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-HousingBulletinBoardFrame_LoadUI-798` | `HousingBulletinBoardFrame_LoadUI` | `retail/AddOns/Blizzard_HousingBulletinBoard/Blizzard_HousingBulletinBoard_Bootstrap.lua:3` |

## W82 — Tests-only candidate: retail/AddOns/Blizzard_HousingControls/Blizzard_HousingControls_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-HousingControls_LoadUI-799` | `HousingControls_LoadUI` | `retail/AddOns/Blizzard_HousingControls/Blizzard_HousingControls_Bootstrap.lua:3` |

## W83 — Tests-only candidate: retail/AddOns/Blizzard_HousingEventHandler/Blizzard_HousingEventHandler.lua

Section: framexml; triage: implemented-needs-proof; 6 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-HousingFramesUtil.IsBlueprintCollectionAvailable-800` | `HousingFramesUtil.IsBlueprintCollectionAvailable` | `retail/AddOns/Blizzard_HousingEventHandler/Blizzard_HousingEventHandler.lua:316` |
| `wt-framexml-HousingFramesUtil.IsBlueprintOperationInProgress-801` | `HousingFramesUtil.IsBlueprintOperationInProgress` | `retail/AddOns/Blizzard_HousingEventHandler/Blizzard_HousingEventHandler.lua:282` |
| `wt-framexml-HousingFramesUtil.ShowBlueprintExport-802` | `HousingFramesUtil.ShowBlueprintExport` | `retail/AddOns/Blizzard_HousingEventHandler/Blizzard_HousingEventHandler.lua:292` |
| `wt-framexml-HousingFramesUtil.ShowBlueprintImport-803` | `HousingFramesUtil.ShowBlueprintImport` | `retail/AddOns/Blizzard_HousingEventHandler/Blizzard_HousingEventHandler.lua:308` |
| `wt-framexml-HousingFramesUtil.ShowBlueprintRoomExport-804` | `HousingFramesUtil.ShowBlueprintRoomExport` | `retail/AddOns/Blizzard_HousingEventHandler/Blizzard_HousingEventHandler.lua:300` |
| `wt-framexml-HousingFramesUtil.TryOpenBlueprintCollection-805` | `HousingFramesUtil.TryOpenBlueprintCollection` | `retail/AddOns/Blizzard_HousingEventHandler/Blizzard_HousingEventHandler.lua:324` |

## W84 — Tests-only candidate: retail/AddOns/Blizzard_HousingHouseFinder/Blizzard_HousingHouseFinder_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-HouseFinderFrame_LoadUI-797` | `HouseFinderFrame_LoadUI` | `retail/AddOns/Blizzard_HousingHouseFinder/Blizzard_HousingHouseFinder_Bootstrap.lua:3` |

## W85 — Tests-only candidate: retail/AddOns/Blizzard_HousingTemplates/Blizzard_HousingCatalogUtil.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-Blizzard_HousingCatalogUtil.AddDecorEntryTooltipTrackingText-655` | `Blizzard_HousingCatalogUtil.AddDecorEntryTooltipTrackingText` | `retail/AddOns/Blizzard_HousingTemplates/Blizzard_HousingCatalogUtil.lua:92` |
| `wt-framexml-Blizzard_HousingCatalogUtil.TrackHousingDecorID-656` | `Blizzard_HousingCatalogUtil.TrackHousingDecorID` | `retail/AddOns/Blizzard_HousingTemplates/Blizzard_HousingCatalogUtil.lua:106` |

## W86 — Tests-only candidate: retail/AddOns/Blizzard_HousingTutorials/Blizzard_HousingTutorialsUtil.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-HousingTutorialUtil.HousingDecorQuestTutorialComplete-806` | `HousingTutorialUtil.HousingDecorQuestTutorialComplete` | `retail/AddOns/Blizzard_HousingTutorials/Blizzard_HousingTutorialsUtil.lua:48` |

## W87 — Tests-only candidate: retail/AddOns/Blizzard_HybridMinimap/Blizzard_HybridMinimap_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-HybridMinimap_LoadUI-807` | `HybridMinimap_LoadUI` | `retail/AddOns/Blizzard_HybridMinimap/Blizzard_HybridMinimap_Bootstrap.lua:3` |

## W88 — Tests-only candidate: retail/AddOns/Blizzard_ItemSocketingUI/Blizzard_ItemSocketingUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ShowItemSocketingFrame-906` | `ShowItemSocketingFrame` | `retail/AddOns/Blizzard_ItemSocketingUI/Blizzard_ItemSocketingUI_Bootstrap.lua:7` |

## W89 — Tests-only candidate: retail/AddOns/Blizzard_ItemUpgradeUI/Blizzard_ItemUpgradeUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-HideItemUpgradeFrame-794` | `HideItemUpgradeFrame` | `retail/AddOns/Blizzard_ItemUpgradeUI/Blizzard_ItemUpgradeUI_Bootstrap.lua:13` |
| `wt-framexml-ShowItemUpgradeFrame-907` | `ShowItemUpgradeFrame` | `retail/AddOns/Blizzard_ItemUpgradeUI/Blizzard_ItemUpgradeUI_Bootstrap.lua:7` |

## W90 — Tests-only candidate: retail/AddOns/Blizzard_Kiosk/Blizzard_Kiosk_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-Kiosk_LoadUI-829` | `Kiosk_LoadUI` | `retail/AddOns/Blizzard_Kiosk/Blizzard_Kiosk_Bootstrap.lua:3` |
| `wt-framexml-KioskFrame_HandlePlayerEnteringWorld-830` | `KioskFrame_HandlePlayerEnteringWorld` | `retail/AddOns/Blizzard_Kiosk/Blizzard_Kiosk_Bootstrap.lua:7` |

## W91 — Tests-only candidate: retail/AddOns/Blizzard_LandingSoulbinds/Blizzard_LandingSoulbinds_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-LandingSoulbinds_LoadUI-831` | `LandingSoulbinds_LoadUI` | `retail/AddOns/Blizzard_LandingSoulbinds/Blizzard_LandingSoulbinds_Bootstrap.lua:3` |

## W92 — Tests-only candidate: retail/AddOns/Blizzard_MacroUI/Blizzard_MacroUI.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-MacroFrame_SaveMacro-839` | `MacroFrame_SaveMacro` | `retail/AddOns/Blizzard_MacroUI/Blizzard_MacroUI.lua:20` |

## W93 — Tests-only candidate: retail/AddOns/Blizzard_ManagedFrameSystem/Shared/ManagedFrameSystem.lua

Section: framexml; triage: implemented-needs-proof; 3 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-GetBottomManagedFrameContainer-756` | `GetBottomManagedFrameContainer` | `retail/AddOns/Blizzard_ManagedFrameSystem/Shared/ManagedFrameSystem.lua:1` |
| `wt-framexml-GetPlayerBottomManagedFrameContainer-762` | `GetPlayerBottomManagedFrameContainer` | `retail/AddOns/Blizzard_ManagedFrameSystem/Shared/ManagedFrameSystem.lua:9` |
| `wt-framexml-GetRightManagedFrameContainer-763` | `GetRightManagedFrameContainer` | `retail/AddOns/Blizzard_ManagedFrameSystem/Shared/ManagedFrameSystem.lua:5` |

## W94 — Tests-only candidate: retail/AddOns/Blizzard_MatchCelebrationPartyPoseUI/Blizzard_MatchCelebrationPartyPoseUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ShowMatchCelebrationPartyPoseFrame-908` | `ShowMatchCelebrationPartyPoseFrame` | `retail/AddOns/Blizzard_MatchCelebrationPartyPoseUI/Blizzard_MatchCelebrationPartyPoseUI_Bootstrap.lua:7` |

## W95 — Tests-only candidate: retail/AddOns/Blizzard_Menu/MenuUtil.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-MenuUtil.CreateHighlightButton-841` | `MenuUtil.CreateHighlightButton` | `retail/AddOns/Blizzard_Menu/MenuUtil.lua:233` |

## W96 — Tests-only candidate: retail/AddOns/Blizzard_MovePad/Blizzard_MovePad_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-MovePad_LoadUI-842` | `MovePad_LoadUI` | `retail/AddOns/Blizzard_MovePad/Blizzard_MovePad_Bootstrap.lua:3` |

## W97 — Tests-only candidate: retail/AddOns/Blizzard_Narration/Blizzard_NarrationUtil.lua

Section: framexml; triage: implemented-needs-proof; 14 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-NarrationUtil.CreateNarrationInfo-843` | `NarrationUtil.CreateNarrationInfo` | `retail/AddOns/Blizzard_Narration/Blizzard_NarrationUtil.lua:191` |
| `wt-framexml-NarrationUtil.GetCheckboxContext-844` | `NarrationUtil.GetCheckboxContext` | `retail/AddOns/Blizzard_Narration/Blizzard_NarrationUtil.lua:133` |
| `wt-framexml-NarrationUtil.MakeIndexInfo-845` | `NarrationUtil.MakeIndexInfo` | `retail/AddOns/Blizzard_Narration/Blizzard_NarrationUtil.lua:125` |
| `wt-framexml-NarrationUtil.MakeNarrationStringForMoney-846` | `NarrationUtil.MakeNarrationStringForMoney` | `retail/AddOns/Blizzard_Narration/Blizzard_NarrationUtil.lua:161` |
| `wt-framexml-NarrationUtil.MakeNarrationStringFromIndexInfo-847` | `NarrationUtil.MakeNarrationStringFromIndexInfo` | `retail/AddOns/Blizzard_Narration/Blizzard_NarrationUtil.lua:174` |
| `wt-framexml-NarrationUtil.MakeNarrationStringFromInfo-848` | `NarrationUtil.MakeNarrationStringFromInfo` | `retail/AddOns/Blizzard_Narration/Blizzard_NarrationUtil.lua:186` |
| `wt-framexml-NarrationUtil.MakeNarrationString-849` | `NarrationUtil.MakeNarrationString` | `retail/AddOns/Blizzard_Narration/Blizzard_NarrationUtil.lua:145` |
| `wt-framexml-NarrationUtil.NarrateCurrentScreen-850` | `NarrationUtil.NarrateCurrentScreen` | `retail/AddOns/Blizzard_Narration/Blizzard_NarrationUtil.lua:233` |
| `wt-framexml-NarrationUtil.RegionToNarrationInfo-851` | `NarrationUtil.RegionToNarrationInfo` | `retail/AddOns/Blizzard_Narration/Blizzard_NarrationUtil.lua:217` |
| `wt-framexml-NarrationUtil.ResolveForwardedRegion-852` | `NarrationUtil.ResolveForwardedRegion` | `retail/AddOns/Blizzard_Narration/Blizzard_NarrationUtil.lua:209` |
| `wt-framexml-NarrationUtil.SetStaticDescription-853` | `NarrationUtil.SetStaticDescription` | `retail/AddOns/Blizzard_Narration/Blizzard_NarrationUtil.lua:274` |
| `wt-framexml-NarrationUtil.SetStaticName-854` | `NarrationUtil.SetStaticName` | `retail/AddOns/Blizzard_Narration/Blizzard_NarrationUtil.lua:257` |
| `wt-framexml-NarrationUtil.ShouldBeEnabled-855` | `NarrationUtil.ShouldBeEnabled` | `retail/AddOns/Blizzard_Narration/Blizzard_NarrationUtil.lua:121` |
| `wt-framexml-NarrationUtil.ShouldRegionNavigationSkipTooltips-856` | `NarrationUtil.ShouldRegionNavigationSkipTooltips` | `retail/AddOns/Blizzard_Narration/Blizzard_NarrationUtil.lua:201` |

## W98 — Tests-only candidate: retail/AddOns/Blizzard_NewPlayerExperience/Blizzard_NewPlayerExperience_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-NPE_InitializeIfLoaded-857` | `NPE_InitializeIfLoaded` | `retail/AddOns/Blizzard_NewPlayerExperience/Blizzard_NewPlayerExperience_Bootstrap.lua:14` |

## W99 — Tests-only candidate: retail/AddOns/Blizzard_OrderHallUI/Blizzard_OrderHallUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-OpenOrderHallTalentUI-862` | `OpenOrderHallTalentUI` | `retail/AddOns/Blizzard_OrderHallUI/Blizzard_OrderHallUI_Bootstrap.lua:13` |

## W100 — Tests-only candidate: retail/AddOns/Blizzard_PVPUI/Mainline/Blizzard_PVPUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-PVPUI_LoadUI-869` | `PVPUI_LoadUI` | `retail/AddOns/Blizzard_PVPUI/Mainline/Blizzard_PVPUI_Bootstrap.lua:3` |

## W101 — Tests-only candidate: retail/AddOns/Blizzard_PartyPoseUI/Blizzard_PartyPoseUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-IslandsPartyPoseFrame_TryShow-819` | `IslandsPartyPoseFrame_TryShow` | `retail/AddOns/Blizzard_PartyPoseUI/Blizzard_PartyPoseUI_Bootstrap.lua:7` |
| `wt-framexml-WarfrontsPartyPoseFrame_TryShow-958` | `WarfrontsPartyPoseFrame_TryShow` | `retail/AddOns/Blizzard_PartyPoseUI/Blizzard_PartyPoseUI_Bootstrap.lua:14` |

## W102 — Tests-only candidate: retail/AddOns/Blizzard_PerksProgram/Blizzard_PerksProgram_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ShowPerksProgramFrame-910` | `ShowPerksProgramFrame` | `retail/AddOns/Blizzard_PerksProgram/Blizzard_PerksProgram_Bootstrap.lua:7` |

## W103 — Tests-only candidate: retail/AddOns/Blizzard_PhotoSharing/Blizzard_PhotoSharing.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-PhotoSharingFrame_EscapePressed-864` | `PhotoSharingFrame_EscapePressed` | `retail/AddOns/Blizzard_PhotoSharing/Blizzard_PhotoSharing.lua:5` |

## W104 — Tests-only candidate: retail/AddOns/Blizzard_PingUI/Blizzard_PingUtil.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-PingUtil.SendMacroPing-865` | `PingUtil.SendMacroPing` | `retail/AddOns/Blizzard_PingUI/Blizzard_PingUtil.lua:3` |
| `wt-framexml-PingUtil.TogglePingTarget-866` | `PingUtil.TogglePingTarget` | `retail/AddOns/Blizzard_PingUI/Blizzard_PingUtil.lua:10` |

## W105 — Tests-only candidate: retail/AddOns/Blizzard_PlayerChoice/Blizzard_PlayerChoice_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 3 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-PlayerChoiceFrame_TryShow-867` | `PlayerChoiceFrame_TryShow` | `retail/AddOns/Blizzard_PlayerChoice/Blizzard_PlayerChoice_Bootstrap.lua:20` |
| `wt-framexml-PlayerChoiceToggle_TryShow-868` | `PlayerChoiceToggle_TryShow` | `retail/AddOns/Blizzard_PlayerChoice/Blizzard_PlayerChoice_Bootstrap.lua:26` |
| `wt-framexml-ShowPendingPlayerChoiceResponseUI-909` | `ShowPendingPlayerChoiceResponseUI` | `retail/AddOns/Blizzard_PlayerChoice/Blizzard_PlayerChoice_Bootstrap.lua:43` |

## W106 — Tests-only candidate: retail/AddOns/Blizzard_PlayerSpells/Blizzard_PlayerSpells_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-OpenPlayerSpellsToGlyphTarget-863` | `OpenPlayerSpellsToGlyphTarget` | `retail/AddOns/Blizzard_PlayerSpells/Blizzard_PlayerSpells_Bootstrap.lua:7` |

## W107 — Tests-only candidate: retail/AddOns/Blizzard_Professions/Blizzard_Professions_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ShowProfessionEquipmentHelpTip-911` | `ShowProfessionEquipmentHelpTip` | `retail/AddOns/Blizzard_Professions/Blizzard_Professions_Bootstrap.lua:24` |
| `wt-framexml-ShowProfessionsFrame-913` | `ShowProfessionsFrame` | `retail/AddOns/Blizzard_Professions/Blizzard_Professions_Bootstrap.lua:7` |

## W108 — Tests-only candidate: retail/AddOns/Blizzard_ProfessionsCustomerOrders/Blizzard_ProfessionsCustomerOrders_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-HideProfessionsCustomerOrdersFrame-795` | `HideProfessionsCustomerOrdersFrame` | `retail/AddOns/Blizzard_ProfessionsCustomerOrders/Blizzard_ProfessionsCustomerOrders_Bootstrap.lua:13` |
| `wt-framexml-ShowProfessionsCustomerOrdersFrame-912` | `ShowProfessionsCustomerOrdersFrame` | `retail/AddOns/Blizzard_ProfessionsCustomerOrders/Blizzard_ProfessionsCustomerOrders_Bootstrap.lua:7` |

## W109 — Tests-only candidate: retail/AddOns/Blizzard_RaidWarning/RaidWarningUtil.lua

Section: framexml; triage: implemented-needs-proof; 3 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-RaidWarningUtil.AddMessage-870` | `RaidWarningUtil.AddMessage` | `retail/AddOns/Blizzard_RaidWarning/RaidWarningUtil.lua:28` |
| `wt-framexml-RaidWarningUtil.ClearBossEmotes-871` | `RaidWarningUtil.ClearBossEmotes` | `retail/AddOns/Blizzard_RaidWarning/RaidWarningUtil.lua:32` |
| `wt-framexml-RaidWarningUtil.UpdateCenterScreenAnchors-872` | `RaidWarningUtil.UpdateCenterScreenAnchors` | `retail/AddOns/Blizzard_RaidWarning/RaidWarningUtil.lua:39` |

## W110 — Tests-only candidate: retail/AddOns/Blizzard_RecentAllies/Blizzard_RecentAlliesUtil.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-RecentAlliesUtil.GetBestSocialUIPresenceTypeForStateData-873` | `RecentAlliesUtil.GetBestSocialUIPresenceTypeForStateData` | `retail/AddOns/Blizzard_RecentAllies/Blizzard_RecentAlliesUtil.lua:124` |

## W111 — Tests-only candidate: retail/AddOns/Blizzard_RecruitAFriend/RecruitAFriendSocialView.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-RecruitAFriendFrameSocialViewInitializeAADC-874` | `RecruitAFriendFrameSocialViewInitializeAADC` | `retail/AddOns/Blizzard_RecruitAFriend/RecruitAFriendSocialView.lua:321` |

## W112 — Tests-only candidate: retail/AddOns/Blizzard_RemixArtifactUI/Blizzard_RemixArtifactUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ShowRemixArtifactFrame-916` | `ShowRemixArtifactFrame` | `retail/AddOns/Blizzard_RemixArtifactUI/Blizzard_RemixArtifactUI_Bootstrap.lua:17` |

## W113 — Tests-only candidate: retail/AddOns/Blizzard_ReportFrame/ReportFrame.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ReportFrame_EscapePressed-879` | `ReportFrame_EscapePressed` | `retail/AddOns/Blizzard_ReportFrame/ReportFrame.lua:37` |

## W114 — Tests-only candidate: retail/AddOns/Blizzard_RuneforgeUI/Blizzard_RuneforgeUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ShowRuneforgeFrame-917` | `ShowRuneforgeFrame` | `retail/AddOns/Blizzard_RuneforgeUI/Blizzard_RuneforgeUI_Bootstrap.lua:7` |

## W115 — Tests-only candidate: retail/AddOns/Blizzard_SettingsDefinitions_Frame/CombatAudioAlertUtil.lua

Section: framexml; triage: implemented-needs-proof; 10 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-CombatAudioAlertUtil.EnumerateInterruptCastInfo-670` | `CombatAudioAlertUtil.EnumerateInterruptCastInfo` | `retail/AddOns/Blizzard_SettingsDefinitions_Frame/CombatAudioAlertUtil.lua:222` |
| `wt-framexml-CombatAudioAlertUtil.EnumerateInterruptCastSuccessInfo-671` | `CombatAudioAlertUtil.EnumerateInterruptCastSuccessInfo` | `retail/AddOns/Blizzard_SettingsDefinitions_Frame/CombatAudioAlertUtil.lua:238` |
| `wt-framexml-CombatAudioAlertUtil.EnumerateSayCombatEndInfo-672` | `CombatAudioAlertUtil.EnumerateSayCombatEndInfo` | `retail/AddOns/Blizzard_SettingsDefinitions_Frame/CombatAudioAlertUtil.lua:214` |
| `wt-framexml-CombatAudioAlertUtil.EnumerateSayCombatStartInfo-673` | `CombatAudioAlertUtil.EnumerateSayCombatStartInfo` | `retail/AddOns/Blizzard_SettingsDefinitions_Frame/CombatAudioAlertUtil.lua:206` |
| `wt-framexml-CombatAudioAlertUtil.EnumeratetWhenTargetDiesInfo-674` | `CombatAudioAlertUtil.EnumeratetWhenTargetDiesInfo` | `retail/AddOns/Blizzard_SettingsDefinitions_Frame/CombatAudioAlertUtil.lua:230` |
| `wt-framexml-CombatAudioAlertUtil.GetInterruptCastInfo-675` | `CombatAudioAlertUtil.GetInterruptCastInfo` | `retail/AddOns/Blizzard_SettingsDefinitions_Frame/CombatAudioAlertUtil.lua:218` |
| `wt-framexml-CombatAudioAlertUtil.GetInterruptCastSuccessInfo-676` | `CombatAudioAlertUtil.GetInterruptCastSuccessInfo` | `retail/AddOns/Blizzard_SettingsDefinitions_Frame/CombatAudioAlertUtil.lua:234` |
| `wt-framexml-CombatAudioAlertUtil.GetSayCombatEndInfo-677` | `CombatAudioAlertUtil.GetSayCombatEndInfo` | `retail/AddOns/Blizzard_SettingsDefinitions_Frame/CombatAudioAlertUtil.lua:210` |
| `wt-framexml-CombatAudioAlertUtil.GetSayCombatStartInfo-678` | `CombatAudioAlertUtil.GetSayCombatStartInfo` | `retail/AddOns/Blizzard_SettingsDefinitions_Frame/CombatAudioAlertUtil.lua:202` |
| `wt-framexml-CombatAudioAlertUtil.GetWhenTargetDiesInfo-679` | `CombatAudioAlertUtil.GetWhenTargetDiesInfo` | `retail/AddOns/Blizzard_SettingsDefinitions_Frame/CombatAudioAlertUtil.lua:226` |

## W116 — Tests-only candidate: retail/AddOns/Blizzard_SettingsDefinitions_Frame/Mainline/CombatOverrides.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-IsMouseoverCastSupported-820` | `IsMouseoverCastSupported` | `retail/AddOns/Blizzard_SettingsDefinitions_Frame/Mainline/CombatOverrides.lua:69` |

## W117 — Tests-only candidate: retail/AddOns/Blizzard_Settings_Shared/Blizzard_SettingsPanel.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-SettingsPanel_EscapePressed-884` | `SettingsPanel_EscapePressed` | `retail/AddOns/Blizzard_Settings_Shared/Blizzard_SettingsPanel.lua:40` |

## W118 — Tests-only candidate: retail/AddOns/Blizzard_Settings_Shared/NewDefinitionsFramework.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-IsUnseenNewSettingInCurrentVersion-823` | `IsUnseenNewSettingInCurrentVersion` | `retail/AddOns/Blizzard_Settings_Shared/NewDefinitionsFramework.lua:32` |

## W119 — Tests-only candidate: retail/AddOns/Blizzard_ShakeUtil/Blizzard_ShakeUtil.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ShakeFrameRandom-886` | `ShakeFrameRandom` | `retail/AddOns/Blizzard_ShakeUtil/Blizzard_ShakeUtil.lua:1` |
| `wt-framexml-ShakeFrame-887` | `ShakeFrame` | `retail/AddOns/Blizzard_ShakeUtil/Blizzard_ShakeUtil.lua:15` |

## W120 — Tests-only candidate: retail/AddOns/Blizzard_SharedXML/AccountUtil.lua

Section: framexml; triage: implemented-needs-proof; 5 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-BNet_GetBattleTagComponents-657` | `BNet_GetBattleTagComponents` | `retail/AddOns/Blizzard_SharedXML/AccountUtil.lua:92` |
| `wt-framexml-BNet_GetBattleTagSelf-658` | `BNet_GetBattleTagSelf` | `retail/AddOns/Blizzard_SharedXML/AccountUtil.lua:82` |
| `wt-framexml-BNet_GetBroadcastTextSelf-659` | `BNet_GetBroadcastTextSelf` | `retail/AddOns/Blizzard_SharedXML/AccountUtil.lua:87` |
| `wt-framexml-BNet_GetFriendLevelRank-660` | `BNet_GetFriendLevelRank` | `retail/AddOns/Blizzard_SharedXML/AccountUtil.lua:131` |
| `wt-framexml-BNet_IsFriendLevelEqualOrHigher-661` | `BNet_IsFriendLevelEqualOrHigher` | `retail/AddOns/Blizzard_SharedXML/AccountUtil.lua:135` |

## W121 — Tests-only candidate: retail/AddOns/Blizzard_SharedXML/GameRulesUtil.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-GameRulesUtil.IsPlayerAtEffectiveMaxLevel-755` | `GameRulesUtil.IsPlayerAtEffectiveMaxLevel` | `retail/AddOns/Blizzard_SharedXML/GameRulesUtil.lua:134` |

## W122 — Tests-only candidate: retail/AddOns/Blizzard_SharedXML/InputUtil.lua

Section: framexml; triage: implemented-needs-proof; 5 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-InputUtil.CursorOnUpdate-811` | `InputUtil.CursorOnUpdate` | `retail/AddOns/Blizzard_SharedXML/InputUtil.lua:40` |
| `wt-framexml-InputUtil.CursorUpdate-812` | `InputUtil.CursorUpdate` | `retail/AddOns/Blizzard_SharedXML/InputUtil.lua:28` |
| `wt-framexml-InputUtil.GetCursorDelta-813` | `InputUtil.GetCursorDelta` | `retail/AddOns/Blizzard_SharedXML/InputUtil.lua:18` |
| `wt-framexml-InputUtil.IsMouseOver-814` | `InputUtil.IsMouseOver` | `retail/AddOns/Blizzard_SharedXML/InputUtil.lua:24` |
| `wt-framexml-InputUtil.ShowInspectCursor-815` | `InputUtil.ShowInspectCursor` | `retail/AddOns/Blizzard_SharedXML/InputUtil.lua:4` |

## W123 — Tests-only candidate: retail/AddOns/Blizzard_SharedXML/InterfaceUtil.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-InterfaceUtil.GetScreenHeightScale-816` | `InterfaceUtil.GetScreenHeightScale` | `retail/AddOns/Blizzard_SharedXML/InterfaceUtil.lua:19` |
| `wt-framexml-InterfaceUtil.GetScreenWidthScale-817` | `InterfaceUtil.GetScreenWidthScale` | `retail/AddOns/Blizzard_SharedXML/InterfaceUtil.lua:24` |

## W124 — Tests-only candidate: retail/AddOns/Blizzard_SharedXML/Interpolator.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-InterpolatorUtil.GetSmoothProgressChange-818` | `InterpolatorUtil.GetSmoothProgressChange` | `retail/AddOns/Blizzard_SharedXML/Interpolator.lua:13` |

## W125 — Tests-only candidate: retail/AddOns/Blizzard_SharedXML/RegionUtil.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-RegionUtil.GetTopLeftMost-875` | `RegionUtil.GetTopLeftMost` | `retail/AddOns/Blizzard_SharedXML/RegionUtil.lua:152` |
| `wt-framexml-RegionUtil.SortByTopLeft-876` | `RegionUtil.SortByTopLeft` | `retail/AddOns/Blizzard_SharedXML/RegionUtil.lua:128` |

## W126 — Tests-only candidate: retail/AddOns/Blizzard_SharedXML/Shared/InputBox/InputBoxTemplates.lua

Section: framexml; triage: implemented-needs-proof; 3 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-InputBoxInstructions_OnEnter-808` | `InputBoxInstructions_OnEnter` | `retail/AddOns/Blizzard_SharedXML/Shared/InputBox/InputBoxTemplates.lua:149` |
| `wt-framexml-InputBoxInstructions_OnLeave-809` | `InputBoxInstructions_OnLeave` | `retail/AddOns/Blizzard_SharedXML/Shared/InputBox/InputBoxTemplates.lua:155` |
| `wt-framexml-InputBoxInstructions_ShowTooltipIfTruncated-810` | `InputBoxInstructions_ShowTooltipIfTruncated` | `retail/AddOns/Blizzard_SharedXML/Shared/InputBox/InputBoxTemplates.lua:163` |

## W127 — Tests-only candidate: retail/AddOns/Blizzard_SharedXML/StringUtil.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-StringUtil.JoinAlternatingConditionalColor-939` | `StringUtil.JoinAlternatingConditionalColor` | `retail/AddOns/Blizzard_SharedXML/StringUtil.lua:33` |

## W128 — Tests-only candidate: retail/AddOns/Blizzard_SharedXML/TimeUtil.lua

Section: framexml; triage: implemented-needs-proof; 3 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-GetTimeStringFromSeconds-764` | `GetTimeStringFromSeconds` | `retail/AddOns/Blizzard_SharedXML/TimeUtil.lua:342` |
| `wt-framexml-TimeUtil.BetterDate-941` | `TimeUtil.BetterDate` | `retail/AddOns/Blizzard_SharedXML/TimeUtil.lua:46` |
| `wt-framexml-TimeUtil.GetRecentTimeDate-942` | `TimeUtil.GetRecentTimeDate` | `retail/AddOns/Blizzard_SharedXML/TimeUtil.lua:24` |

## W129 — Tests-only candidate: retail/AddOns/Blizzard_SharedXMLBase/AnchorUtil.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-AnchorUtil.ApplyFlowLayout-632` | `AnchorUtil.ApplyFlowLayout` | `retail/AddOns/Blizzard_SharedXMLBase/AnchorUtil.lua:637` |
| `wt-framexml-AnchorUtil.CreateFlowLayout-633` | `AnchorUtil.CreateFlowLayout` | `retail/AddOns/Blizzard_SharedXMLBase/AnchorUtil.lua:606` |

## W130 — Tests-only candidate: retail/AddOns/Blizzard_SharedXMLBase/ErrorUtil.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-assertf-960` | `assertf` | `retail/AddOns/Blizzard_SharedXMLBase/ErrorUtil.lua:32` |

## W131 — Tests-only candidate: retail/AddOns/Blizzard_SharedXMLBase/LocaleUtil.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-LocaleUtil.GetLocaleDisplayName-835` | `LocaleUtil.GetLocaleDisplayName` | `retail/AddOns/Blizzard_SharedXMLBase/LocaleUtil.lua:12` |

## W132 — Tests-only candidate: retail/AddOns/Blizzard_SharedXMLBase/Mixin.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ApplySecureDelegatesToTable-634` | `ApplySecureDelegatesToTable` | `retail/AddOns/Blizzard_SharedXMLBase/Mixin.lua:50` |

## W133 — Tests-only candidate: retail/AddOns/Blizzard_SharedXMLBase/TextureUtil.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-TextureUtil.AnimateTexCoords-940` | `TextureUtil.AnimateTexCoords` | `retail/AddOns/Blizzard_SharedXMLBase/TextureUtil.lua:28` |

## W134 — Tests-only candidate: retail/AddOns/Blizzard_SimpleCheckout/Blizzard_SimpleCheckout_Inbound.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-SimpleCheckout_EscapePressed-920` | `SimpleCheckout_EscapePressed` | `retail/AddOns/Blizzard_SimpleCheckout/Blizzard_SimpleCheckout_Inbound.lua:7` |

## W135 — Tests-only candidate: retail/AddOns/Blizzard_SocialUIShared/SocialUIControl.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ToggleSocialUI-944` | `ToggleSocialUI` | `retail/AddOns/Blizzard_SocialUIShared/SocialUIControl.lua:1` |

## W136 — Tests-only candidate: retail/AddOns/Blizzard_SocialUIShared/SocialUISharedTemplates.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-SocialUIContactsFrameInitializeAADC-921` | `SocialUIContactsFrameInitializeAADC` | `retail/AddOns/Blizzard_SocialUIShared/SocialUISharedTemplates.lua:158` |

## W137 — Tests-only candidate: retail/AddOns/Blizzard_SocialUIShared/SocialUIUtil.lua

Section: framexml; triage: implemented-needs-proof; 13 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-SocialUIUtil.AddSeparatorToTooltip-922` | `SocialUIUtil.AddSeparatorToTooltip` | `retail/AddOns/Blizzard_SocialUIShared/SocialUIUtil.lua:145` |
| `wt-framexml-SocialUIUtil.GetBattleNetFriendTagInterestsUIOrder-923` | `SocialUIUtil.GetBattleNetFriendTagInterestsUIOrder` | `retail/AddOns/Blizzard_SocialUIShared/SocialUIUtil.lua:33` |
| `wt-framexml-SocialUIUtil.GetBattleNetFriendTagRoleUIOrder-924` | `SocialUIUtil.GetBattleNetFriendTagRoleUIOrder` | `retail/AddOns/Blizzard_SocialUIShared/SocialUIUtil.lua:44` |
| `wt-framexml-SocialUIUtil.GetBlockedName-925` | `SocialUIUtil.GetBlockedName` | `retail/AddOns/Blizzard_SocialUIShared/SocialUIUtil.lua:133` |
| `wt-framexml-SocialUIUtil.GetIconForPresenceType-926` | `SocialUIUtil.GetIconForPresenceType` | `retail/AddOns/Blizzard_SocialUIShared/SocialUIUtil.lua:58` |
| `wt-framexml-SocialUIUtil.GetLabelForBattleNetFriendTag-927` | `SocialUIUtil.GetLabelForBattleNetFriendTag` | `retail/AddOns/Blizzard_SocialUIShared/SocialUIUtil.lua:18` |
| `wt-framexml-SocialUIUtil.GetLabelForPresenceType-928` | `SocialUIUtil.GetLabelForPresenceType` | `retail/AddOns/Blizzard_SocialUIShared/SocialUIUtil.lua:72` |
| `wt-framexml-SocialUIUtil.GetPresenceTypeForBattleNetAccountInfo-929` | `SocialUIUtil.GetPresenceTypeForBattleNetAccountInfo` | `retail/AddOns/Blizzard_SocialUIShared/SocialUIUtil.lua:90` |
| `wt-framexml-SocialUIUtil.GetPresenceTypeSelf-930` | `SocialUIUtil.GetPresenceTypeSelf` | `retail/AddOns/Blizzard_SocialUIShared/SocialUIUtil.lua:76` |
| `wt-framexml-SocialUIUtil.InitializeUserScaledDropdownButton-931` | `SocialUIUtil.InitializeUserScaledDropdownButton` | `retail/AddOns/Blizzard_SocialUIShared/SocialUIUtil.lua:155` |
| `wt-framexml-SocialUIUtil.InitializeUserScaledDropdownMainTitle-932` | `SocialUIUtil.InitializeUserScaledDropdownMainTitle` | `retail/AddOns/Blizzard_SocialUIShared/SocialUIUtil.lua:169` |
| `wt-framexml-SocialUIUtil.InitializeUserScaledDropdownTitle-933` | `SocialUIUtil.InitializeUserScaledDropdownTitle` | `retail/AddOns/Blizzard_SocialUIShared/SocialUIUtil.lua:162` |
| `wt-framexml-SocialUIUtil.SetBattleNetPresenceFromSocialUIPresence-934` | `SocialUIUtil.SetBattleNetPresenceFromSocialUIPresence` | `retail/AddOns/Blizzard_SocialUIShared/SocialUIUtil.lua:104` |

## W138 — Tests-only candidate: retail/AddOns/Blizzard_Soulbinds/Blizzard_Soulbinds_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-SoulbindViewer_LoadUI-935` | `SoulbindViewer_LoadUI` | `retail/AddOns/Blizzard_Soulbinds/Blizzard_Soulbinds_Bootstrap.lua:3` |

## W139 — Tests-only candidate: retail/AddOns/Blizzard_SplashFrame/Mainline/SplashFrame.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-SplashFrame_EscapePressed-937` | `SplashFrame_EscapePressed` | `retail/AddOns/Blizzard_SplashFrame/Mainline/SplashFrame.lua:103` |

## W140 — Tests-only candidate: retail/AddOns/Blizzard_StaticPopup_Game/GameDialogDefs.lua

Section: framexml; triage: implemented-needs-proof; 12 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ConfirmDisenchantRollDialog_Show-686` | `ConfirmDisenchantRollDialog_Show` | `retail/AddOns/Blizzard_StaticPopup_Game/GameDialogDefs.lua:92` |
| `wt-framexml-ConfirmLootRollDialog_Show-687` | `ConfirmLootRollDialog_Show` | `retail/AddOns/Blizzard_StaticPopup_Game/GameDialogDefs.lua:88` |
| `wt-framexml-ConfirmTalentWipeDialog_Show-688` | `ConfirmTalentWipeDialog_Show` | `retail/AddOns/Blizzard_StaticPopup_Game/GameDialogDefs.lua:96` |
| `wt-framexml-GossipConfirmDialog_Show-768` | `GossipConfirmDialog_Show` | `retail/AddOns/Blizzard_StaticPopup_Game/GameDialogDefs.lua:101` |
| `wt-framexml-HideInstanceBootDialog-792` | `HideInstanceBootDialog` | `retail/AddOns/Blizzard_StaticPopup_Game/GameDialogDefs.lua:114` |
| `wt-framexml-HideInstanceLockDialog-793` | `HideInstanceLockDialog` | `retail/AddOns/Blizzard_StaticPopup_Game/GameDialogDefs.lua:123` |
| `wt-framexml-HideSummonConfirmationDialogs-796` | `HideSummonConfirmationDialogs` | `retail/AddOns/Blizzard_StaticPopup_Game/GameDialogDefs.lua:141` |
| `wt-framexml-IsSummonConfirmationDialogVisible-821` | `IsSummonConfirmationDialogVisible` | `retail/AddOns/Blizzard_StaticPopup_Game/GameDialogDefs.lua:127` |
| `wt-framexml-ShowInstanceBootDialog-904` | `ShowInstanceBootDialog` | `retail/AddOns/Blizzard_StaticPopup_Game/GameDialogDefs.lua:106` |
| `wt-framexml-ShowInstanceLockDialog-905` | `ShowInstanceLockDialog` | `retail/AddOns/Blizzard_StaticPopup_Game/GameDialogDefs.lua:119` |
| `wt-framexml-ShowSummonConfirmationDialog-918` | `ShowSummonConfirmationDialog` | `retail/AddOns/Blizzard_StaticPopup_Game/GameDialogDefs.lua:131` |
| `wt-framexml-UpdateQuestAcceptLogFullDialog-953` | `UpdateQuestAcceptLogFullDialog` | `retail/AddOns/Blizzard_StaticPopup_Game/GameDialogDefs.lua:64` |

## W141 — Tests-only candidate: retail/AddOns/Blizzard_StoreUI/Blizzard_Shared_StoreUIToggle.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-CheckActiveStoreForFree-669` | `CheckActiveStoreForFree` | `retail/AddOns/Blizzard_StoreUI/Blizzard_Shared_StoreUIToggle.lua:51` |
| `wt-framexml-StoreEscapePressed-938` | `StoreEscapePressed` | `retail/AddOns/Blizzard_StoreUI/Blizzard_Shared_StoreUIToggle.lua:76` |

## W142 — Tests-only candidate: retail/AddOns/Blizzard_UIModes/Blizzard_UIModeUtil.lua

Section: framexml; triage: implemented-needs-proof; 5 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-UIModeUtil.CreateExtendedBlocklist-947` | `UIModeUtil.CreateExtendedBlocklist` | `retail/AddOns/Blizzard_UIModes/Blizzard_UIModeUtil.lua:43` |
| `wt-framexml-UIModeUtil.CreateModifiedBlocklist-948` | `UIModeUtil.CreateModifiedBlocklist` | `retail/AddOns/Blizzard_UIModes/Blizzard_UIModeUtil.lua:53` |
| `wt-framexml-UIModeUtil.IsModeActive-949` | `UIModeUtil.IsModeActive` | `retail/AddOns/Blizzard_UIModes/Blizzard_UIModeUtil.lua:38` |
| `wt-framexml-UIModeUtil.RegisterMode-950` | `UIModeUtil.RegisterMode` | `retail/AddOns/Blizzard_UIModes/Blizzard_UIModeUtil.lua:20` |
| `wt-framexml-UIModeUtil.SetModeActive-951` | `UIModeUtil.SetModeActive` | `retail/AddOns/Blizzard_UIModes/Blizzard_UIModeUtil.lua:26` |

## W143 — Tests-only candidate: retail/AddOns/Blizzard_UIPanels_Game/Mainline/GossipFrameAPI.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-HideGossipFrame-790` | `HideGossipFrame` | `retail/AddOns/Blizzard_UIPanels_Game/Mainline/GossipFrameAPI.lua:1` |

## W144 — Tests-only candidate: retail/AddOns/Blizzard_UIPanels_Game/Mainline/ItemRef.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-GetDiscordUserCommunityLink-758` | `GetDiscordUserCommunityLink` | `retail/AddOns/Blizzard_UIPanels_Game/Mainline/ItemRef.lua:126` |
| `wt-framexml-GetDiscordUserLink-759` | `GetDiscordUserLink` | `retail/AddOns/Blizzard_UIPanels_Game/Mainline/ItemRef.lua:122` |

## W145 — Tests-only candidate: retail/AddOns/Blizzard_UIPanels_Game/Mainline/LootFrame.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-LootFrame_EscapePressed-838` | `LootFrame_EscapePressed` | `retail/AddOns/Blizzard_UIPanels_Game/Mainline/LootFrame.lua:13` |

## W146 — Tests-only candidate: retail/AddOns/Blizzard_UIPanels_Game/Shared/PlayerInteractionFrameManager.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-RegisterPlayerInteraction-878` | `RegisterPlayerInteraction` | `retail/AddOns/Blizzard_UIPanels_Game/Shared/PlayerInteractionFrameManager.lua:73` |
| `wt-framexml-SetPlayerInteractionConditions-883` | `SetPlayerInteractionConditions` | `retail/AddOns/Blizzard_UIPanels_Game/Shared/PlayerInteractionFrameManager.lua:92` |

## W147 — Tests-only candidate: retail/AddOns/Blizzard_UIPanels_Game/Shared/TaxiFrame.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ShowTaxiMapFrame-919` | `ShowTaxiMapFrame` | `retail/AddOns/Blizzard_UIPanels_Game/Shared/TaxiFrame.lua:4` |

## W148 — Tests-only candidate: retail/AddOns/Blizzard_UIParentPanelManager/Shared/UIPanelLayoutFrame.lua

Section: framexml; triage: implemented-needs-proof; 3 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-GetUIPanelLayoutAttribute-765` | `GetUIPanelLayoutAttribute` | `retail/AddOns/Blizzard_UIParentPanelManager/Shared/UIPanelLayoutFrame.lua:21` |
| `wt-framexml-GetUIPanelLayoutFrame-766` | `GetUIPanelLayoutFrame` | `retail/AddOns/Blizzard_UIParentPanelManager/Shared/UIPanelLayoutFrame.lua:17` |
| `wt-framexml-SetUIPanelLayoutAttribute-885` | `SetUIPanelLayoutAttribute` | `retail/AddOns/Blizzard_UIParentPanelManager/Shared/UIPanelLayoutFrame.lua:25` |

## W149 — Tests-only candidate: retail/AddOns/Blizzard_UIParentPanelManager/Shared/UIParentPanelManager.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ManageFramePositions-840` | `ManageFramePositions` | `retail/AddOns/Blizzard_UIParentPanelManager/Shared/UIParentPanelManager.lua:790` |

## W150 — Tests-only candidate: retail/AddOns/Blizzard_UnitFrame/Shared/CompactUnitFrame.lua

Section: framexml; triage: implemented-needs-proof; 3 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-CompactUnitFrame_GetOptionDispelIndicatorOverlayAnimation-681` | `CompactUnitFrame_GetOptionDispelIndicatorOverlayAnimation` | `retail/AddOns/Blizzard_UnitFrame/Shared/CompactUnitFrame.lua:1775` |
| `wt-framexml-CompactUnitFrame_GetOptionDispelIndicatorOverlayType-682` | `CompactUnitFrame_GetOptionDispelIndicatorOverlayType` | `retail/AddOns/Blizzard_UnitFrame/Shared/CompactUnitFrame.lua:1771` |
| `wt-framexml-CompactUnitFrameLayoutTemplates_LayoutFrameElement-683` | `CompactUnitFrameLayoutTemplates_LayoutFrameElement` | `retail/AddOns/Blizzard_UnitFrame/Shared/CompactUnitFrame.lua:1925` |

## W151 — Tests-only candidate: retail/AddOns/Blizzard_UnitFrame/Shared/CompactUnitFrameUtil.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-CompactUnitFrameUtil.ApplyConfig-684` | `CompactUnitFrameUtil.ApplyConfig` | `retail/AddOns/Blizzard_UnitFrame/Shared/CompactUnitFrameUtil.lua:262` |
| `wt-framexml-CompactUnitFrameUtil.GenerateNewConfig-685` | `CompactUnitFrameUtil.GenerateNewConfig` | `retail/AddOns/Blizzard_UnitFrame/Shared/CompactUnitFrameUtil.lua:251` |

## W152 — Tests-only candidate: retail/AddOns/Blizzard_UnitFrame/Shared/Localization.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-LocalizePlayerFrame_zhCN-836` | `LocalizePlayerFrame_zhCN` | `retail/AddOns/Blizzard_UnitFrame/Shared/Localization.lua:11` |
| `wt-framexml-LocalizePlayerFrame_zhTW-837` | `LocalizePlayerFrame_zhTW` | `retail/AddOns/Blizzard_UnitFrame/Shared/Localization.lua:15` |

## W153 — Tests-only candidate: retail/AddOns/Blizzard_UnitPopupShared/UnitPopupSharedUtils.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-UnitPopupSharedUtil.IsFriendshipUpgrade-952` | `UnitPopupSharedUtil.IsFriendshipUpgrade` | `retail/AddOns/Blizzard_UnitPopupShared/UnitPopupSharedUtils.lua:176` |

## W154 — Tests-only candidate: retail/AddOns/Blizzard_VisualAlerts/VisualAlertTemplates.lua

Section: framexml; triage: implemented-needs-proof; 4 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-VisualAlert_GetTypeTemplate-954` | `VisualAlert_GetTypeTemplate` | `retail/AddOns/Blizzard_VisualAlerts/VisualAlertTemplates.lua:108` |
| `wt-framexml-VisualAlert_GetTypeText-955` | `VisualAlert_GetTypeText` | `retail/AddOns/Blizzard_VisualAlerts/VisualAlertTemplates.lua:103` |
| `wt-framexml-VisualAlertData_ForEach-956` | `VisualAlertData_ForEach` | `retail/AddOns/Blizzard_VisualAlerts/VisualAlertTemplates.lua:113` |
| `wt-framexml-VisualAlerts_RegisterAll-957` | `VisualAlerts_RegisterAll` | `retail/AddOns/Blizzard_VisualAlerts/VisualAlertTemplates.lua:119` |

## W155 — Tests-only candidate: retail/AddOns/Blizzard_WorldMap/Blizzard_WorldMap.lua

Section: framexml; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-OpenMapToUserWaypoint-861` | `OpenMapToUserWaypoint` | `retail/AddOns/Blizzard_WorldMap/Blizzard_WorldMap.lua:612` |

## W156 — Tests-only candidate: retail/AddOns/Blizzard_WowSurveyUI/Blizzard_WowSurveyUI_Bootstrap.lua

Section: framexml; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-AddWowSurveyStatusFrameToStatusFrames-630` | `AddWowSurveyStatusFrameToStatusFrames` | `retail/AddOns/Blizzard_WowSurveyUI/Blizzard_WowSurveyUI_Bootstrap.lua:13` |
| `wt-framexml-WowSurveyStatusFrame_OnSurveyDelivered-959` | `WowSurveyStatusFrame_OnSurveyDelivered` | `retail/AddOns/Blizzard_WowSurveyUI/Blizzard_WowSurveyUI_Bootstrap.lua:7` |

## W157 — Tests-only candidate: C_ActionBar

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_ActionBar.ForceUpdateAction-584` | `C_ActionBar.ForceUpdateAction` | `src/lua_api/globals/action_bar_api/registration.rs:71`; `retail/AddOns/Blizzard_APIDocumentationGenerated/ActionBarFrameDocumentation.lua:83` |

## W158 — Tests-only candidate: C_AuraContainerUtil

Section: global-api; triage: implemented-needs-proof; 9 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_AuraContainerUtil.ProcessAuraTooltipBackdropOptions-414` | `C_AuraContainerUtil.ProcessAuraTooltipBackdropOptions` | `src/c_api/c_aura_container_util.rs:224`; `retail/AddOns/Blizzard_APIDocumentationGenerated/AuraContainerUtilDocumentation.lua:11` |
| `wt-global-api-C_AuraContainerUtil.ProcessAuraTooltipNineSliceOptions-415` | `C_AuraContainerUtil.ProcessAuraTooltipNineSliceOptions` | `src/c_api/c_aura_container_util.rs:225`; `retail/AddOns/Blizzard_APIDocumentationGenerated/AuraContainerUtilDocumentation.lua:26` |
| `wt-global-api-C_AuraContainerUtil.ProcessAuraTooltipTextureSliceOptions-416` | `C_AuraContainerUtil.ProcessAuraTooltipTextureSliceOptions` | `src/c_api/c_aura_container_util.rs:226`; `retail/AddOns/Blizzard_APIDocumentationGenerated/AuraContainerUtilDocumentation.lua:41` |
| `wt-global-api-C_AuraContainerUtil.ProcessCustomAuraButtonApplicationBarOptions-417` | `C_AuraContainerUtil.ProcessCustomAuraButtonApplicationBarOptions` | `src/c_api/c_aura_container_util.rs:227`; `retail/AddOns/Blizzard_APIDocumentationGenerated/AuraContainerUtilDocumentation.lua:56` |
| `wt-global-api-C_AuraContainerUtil.ProcessCustomAuraButtonApplicationCountOptions-418` | `C_AuraContainerUtil.ProcessCustomAuraButtonApplicationCountOptions` | `src/c_api/c_aura_container_util.rs:228`; `retail/AddOns/Blizzard_APIDocumentationGenerated/AuraContainerUtilDocumentation.lua:71` |
| `wt-global-api-C_AuraContainerUtil.ProcessCustomAuraButtonDispelTypeTextOptions-419` | `C_AuraContainerUtil.ProcessCustomAuraButtonDispelTypeTextOptions` | `src/c_api/c_aura_container_util.rs:231`; `retail/AddOns/Blizzard_APIDocumentationGenerated/AuraContainerUtilDocumentation.lua:86` |
| `wt-global-api-C_AuraContainerUtil.ProcessCustomAuraButtonDispelTypeTextureOptions-420` | `C_AuraContainerUtil.ProcessCustomAuraButtonDispelTypeTextureOptions` | `src/c_api/c_aura_container_util.rs:232`; `retail/AddOns/Blizzard_APIDocumentationGenerated/AuraContainerUtilDocumentation.lua:101` |
| `wt-global-api-C_AuraContainerUtil.ProcessCustomAuraButtonDurationBarOptions-421` | `C_AuraContainerUtil.ProcessCustomAuraButtonDurationBarOptions` | `src/c_api/c_aura_container_util.rs:233`; `retail/AddOns/Blizzard_APIDocumentationGenerated/AuraContainerUtilDocumentation.lua:116` |
| `wt-global-api-C_AuraContainerUtil.ProcessCustomAuraButtonDurationTextOptions-422` | `C_AuraContainerUtil.ProcessCustomAuraButtonDurationTextOptions` | `src/c_api/c_aura_container_util.rs:234`; `retail/AddOns/Blizzard_APIDocumentationGenerated/AuraContainerUtilDocumentation.lua:131` |

## W159 — Tests-only candidate: C_BattleNet

Section: global-api; triage: implemented-needs-proof; 12 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_BattleNet.AreFriendTagsEnabled-423` | `C_BattleNet.AreFriendTagsEnabled` | `src/c_api/c_battle_net.rs:106`; `retail/AddOns/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:11` |
| `wt-global-api-C_BattleNet.AreTitleFriendCustomNamesEnabled-424` | `C_BattleNet.AreTitleFriendCustomNamesEnabled` | `src/c_api/c_battle_net.rs:112`; `retail/AddOns/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:20` |
| `wt-global-api-C_BattleNet.AreTitleFriendsEnabled-425` | `C_BattleNet.AreTitleFriendsEnabled` | `src/c_api/c_battle_net.rs:118`; `retail/AddOns/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:29` |
| `wt-global-api-C_BattleNet.BNCheckTitleFriendInviteToUnit-426` | `C_BattleNet.BNCheckTitleFriendInviteToUnit` | `src/c_api/c_battle_net.rs:161`; `retail/AddOns/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:48` |
| `wt-global-api-C_BattleNet.GetCustomTitleFriendName-428` | `C_BattleNet.GetCustomTitleFriendName` | `src/c_api/c_battle_net.rs:136`; `retail/AddOns/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:98` |
| `wt-global-api-C_BattleNet.GetFriendInviteInfo-429` | `C_BattleNet.GetFriendInviteInfo` | `src/c_api/c_battle_net.rs:149`; `retail/AddOns/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:146` |
| `wt-global-api-C_BattleNet.IsBattleNetFriendsListEnabled-430` | `C_BattleNet.IsBattleNetFriendsListEnabled` | `src/c_api/c_battle_net.rs:124`; `retail/AddOns/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:221` |
| `wt-global-api-C_BattleNet.IsBattleNetFriendsListSupported-431` | `C_BattleNet.IsBattleNetFriendsListSupported` | `src/c_api/c_battle_net.rs:130`; `retail/AddOns/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:230` |
| `wt-global-api-C_BattleNet.SendVerifiedBattleNetFriendInvite-434` | `C_BattleNet.SendVerifiedBattleNetFriendInvite` | `src/c_api/c_battle_net.rs:155`; `retail/AddOns/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:283` |
| `wt-global-api-C_BattleNet.SetAppearOffline-435` | `C_BattleNet.SetAppearOffline` | `src/c_api/c_battle_net.rs:167`; `retail/AddOns/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:316` |
| `wt-global-api-C_BattleNet.SetCustomTitleFriendName-436` | `C_BattleNet.SetCustomTitleFriendName` | `src/c_api/c_battle_net.rs:142`; `retail/AddOns/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:343` |
| `wt-global-api-C_BattleNet.SetFriendTags-437` | `C_BattleNet.SetFriendTags` | `src/c_api/c_battle_net.rs:145`; `retail/AddOns/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:365` |

## W160 — Tests-only candidate: C_CVar

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_CVar.AreCVarsLoaded-439` | `C_CVar.AreCVarsLoaded` | `src/lua_api/globals/set_cvar_verb.rs:30`; `retail/AddOns/Blizzard_APIDocumentationGenerated/CVarDocumentation.lua:11` |

## W161 — Tests-only candidate: C_CombatAudioAlert

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_CombatAudioAlert.SpeakText-586` | `C_CombatAudioAlert.SpeakText` | `src/lua_api/workarounds/temporary/sound_driver_defaults.rs:15`; `retail/AddOns/Blizzard_APIDocumentationGenerated/CombatAudioAlertDocumentation.lua:233` |

## W162 — Tests-only candidate: C_CooldownViewer

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_CooldownViewer.GetGroupBuffItems-442` | `C_CooldownViewer.GetGroupBuffItems` | `src/lua_api/workarounds/temporary/cooldown_viewer_defaults.rs:25`; `retail/AddOns/Blizzard_APIDocumentationGenerated/CooldownViewerDocumentation.lua:43` |

## W163 — Tests-only candidate: C_DelvesUI

Section: global-api; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_DelvesUI.GetFlavorNodeForCompanion-443` | `C_DelvesUI.GetFlavorNodeForCompanion` | `src/lua_api/globals/missing_surface/delves_ui.rs:88`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DelvesUIDocumentation.lua:199` |
| `wt-global-api-C_DelvesUI.GetFlavorNodeNameForCompanion-444` | `C_DelvesUI.GetFlavorNodeNameForCompanion` | `src/lua_api/globals/missing_surface/delves_ui.rs:90`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DelvesUIDocumentation.lua:214` |

## W164 — Tests-only candidate: C_Discord

Section: global-api; triage: implemented-needs-proof; 19 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_Discord.Authorize-448` | `C_Discord.Authorize` | `src/c_api/c_discord.rs:30`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:11` |
| `wt-global-api-C_Discord.GetDiscordChannelName-449` | `C_Discord.GetDiscordChannelName` | `src/c_api/c_discord.rs:34`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:16` |
| `wt-global-api-C_Discord.GetDiscordUserID-450` | `C_Discord.GetDiscordUserID` | `src/c_api/c_discord.rs:37`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:33` |
| `wt-global-api-C_Discord.GetDisplayNameType-452` | `C_Discord.GetDisplayNameType` | `src/c_api/c_discord.rs:38`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:59` |
| `wt-global-api-C_Discord.GetGuildLinkStatus-453` | `C_Discord.GetGuildLinkStatus` | `src/c_api/c_discord.rs:39`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:69` |
| `wt-global-api-C_Discord.GetNumDiscordChannels-454` | `C_Discord.GetNumDiscordChannels` | `src/c_api/c_discord.rs:43`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:81` |
| `wt-global-api-C_Discord.GetNumDiscordServers-455` | `C_Discord.GetNumDiscordServers` | `src/c_api/c_discord.rs:49`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:98` |
| `wt-global-api-C_Discord.GetServerLinkableChannels-456` | `C_Discord.GetServerLinkableChannels` | `src/c_api/c_discord.rs:55`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:108` |
| `wt-global-api-C_Discord.GetServerName-457` | `C_Discord.GetServerName` | `src/c_api/c_discord.rs:58`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:119` |
| `wt-global-api-C_Discord.GuildLink-458` | `C_Discord.GuildLink` | `src/c_api/c_discord.rs:59`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:135` |
| `wt-global-api-C_Discord.GuildUnlink-459` | `C_Discord.GuildUnlink` | `src/c_api/c_discord.rs:60`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:147` |
| `wt-global-api-C_Discord.IsEnabled-460` | `C_Discord.IsEnabled` | `src/c_api/c_discord.rs:61`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:152` |
| `wt-global-api-C_Discord.IsGuildChannelLinked-461` | `C_Discord.IsGuildChannelLinked` | `src/c_api/c_discord.rs:65`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:161` |
| `wt-global-api-C_Discord.IsGuildSettingSet-462` | `C_Discord.IsGuildSettingSet` | `src/c_api/c_discord.rs:68`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:171` |
| `wt-global-api-C_Discord.IsUserOAuthed-463` | `C_Discord.IsUserOAuthed` | `src/c_api/c_discord.rs:69`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:187` |
| `wt-global-api-C_Discord.RefreshAuth-464` | `C_Discord.RefreshAuth` | `src/c_api/c_discord.rs:70`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:197` |
| `wt-global-api-C_Discord.SetGuildSetting-465` | `C_Discord.SetGuildSetting` | `src/c_api/c_discord.rs:71`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:202` |
| `wt-global-api-C_Discord.UpdateDiscordServers-466` | `C_Discord.UpdateDiscordServers` | `src/c_api/c_discord.rs:75`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:214` |
| `wt-global-api-C_Discord.UpdateGuildLobby-467` | `C_Discord.UpdateGuildLobby` | `src/c_api/c_discord.rs:78`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:219` |

## W165 — Tests-only candidate: C_DyeColor

Section: global-api; triage: already-removed/absent-as-required; 2 symbols.

Required later proof: absence/rejection after startup on Retail; static search is provisional.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_DyeColor.GetDyeColorForItemLocation-565` | `C_DyeColor.GetDyeColorForItemLocation` | `src/ptr/strict_removals.lua:43`; `retail/AddOns/Blizzard_Deprecated/Mainline/Deprecated_12_1_0.lua:49` |
| `wt-global-api-C_DyeColor.GetDyeColorForItem-566` | `C_DyeColor.GetDyeColorForItem` | `src/ptr/strict_removals.lua:44`; `retail/AddOns/Blizzard_Deprecated/Mainline/Deprecated_12_1_0.lua:39` |

## W166 — Tests-only candidate: C_DyeColor

Section: global-api; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_DyeColor.GetDyeColorsForItemLocation-468` | `C_DyeColor.GetDyeColorsForItemLocation` | `src/lua_api/workarounds/temporary/dye_color_defaults.rs:16`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DyeColorInfoDocumentation.lua:80` |
| `wt-global-api-C_DyeColor.GetDyeColorsForItem-469` | `C_DyeColor.GetDyeColorsForItem` | `src/lua_api/workarounds/temporary/dye_color_defaults.rs:10`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DyeColorInfoDocumentation.lua:65` |

## W167 — Tests-only candidate: C_EncounterJournal

Section: global-api; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_EncounterJournal.GetBaseDifficultyID-470` | `C_EncounterJournal.GetBaseDifficultyID` | `src/lua_api/globals/missing_surface/encounter_journal.rs:146`; `retail/AddOns/Blizzard_APIDocumentationGenerated/EncounterJournalDocumentation.lua:11` |
| `wt-global-api-C_EncounterJournal.InstanceHasDifficultyID-471` | `C_EncounterJournal.InstanceHasDifficultyID` | `src/lua_api/globals/missing_surface/encounter_journal.rs:152`; `retail/AddOns/Blizzard_APIDocumentationGenerated/EncounterJournalDocumentation.lua:167` |

## W168 — Tests-only candidate: C_FriendList

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_FriendList.IsLegacyFriendSystemEnabled-472` | `C_FriendList.IsLegacyFriendSystemEnabled` | `src/lua_api/workarounds/temporary/inert_global_defaults.rs:82`; `retail/AddOns/Blizzard_APIDocumentationGenerated/FriendListDocumentation.lua:267` |

## W169 — Tests-only candidate: C_GuildInfo

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_GuildInfo.IsDiscordStreamSeparate-473` | `C_GuildInfo.IsDiscordStreamSeparate` | `src/lua_api/globals/guild_info.rs:225`; `retail/AddOns/Blizzard_APIDocumentationGenerated/GuildInfoDocumentation.lua:156` |

## W170 — Tests-only candidate: C_HouseEditor

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_HouseEditor.GetHouseEditorPlayerType-474` | `C_HouseEditor.GetHouseEditorPlayerType` | `src/c_api/c_housing.rs:228`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HouseEditorUIDocumentation.lua:72` |

## W171 — Tests-only candidate: C_Housing

Section: global-api; triage: already-removed/absent-as-required; 1 symbols.

Required later proof: absence/rejection after startup on Retail; static search is provisional.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_Housing.IsInsideOwnHouse-567` | `C_Housing.IsInsideOwnHouse` | `src/ptr/strict_removals.lua:45`; `retail/AddOns/Blizzard_Deprecated/Mainline/Deprecated_12_1_0.lua:59` |

## W172 — Tests-only candidate: C_Housing

Section: global-api; triage: implemented-needs-proof; 5 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_Housing.HouseFinderIgnoreNeighborhood-475` | `C_Housing.HouseFinderIgnoreNeighborhood` | `src/c_api/c_housing.rs:125`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingUIDocumentation.lua:246` |
| `wt-global-api-C_Housing.IsInsideOwnedHouseOrPlot-476` | `C_Housing.IsInsideOwnedHouseOrPlot` | `src/c_api/c_housing.rs:131`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingUIDocumentation.lua:334` |
| `wt-global-api-C_Housing.IsInsideOwnedHouse-477` | `C_Housing.IsInsideOwnedHouse` | `src/c_api/c_housing.rs:134`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingUIDocumentation.lua:325` |
| `wt-global-api-C_Housing.IsInsideOwnedPlot-478` | `C_Housing.IsInsideOwnedPlot` | `src/c_api/c_housing.rs:135`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingUIDocumentation.lua:343` |
| `wt-global-api-C_Housing.ResetHouse-479` | `C_Housing.ResetHouse` | `src/c_api/c_housing.rs:136`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingUIDocumentation.lua:443` |

## W173 — Tests-only candidate: C_HousingBlueprint

Section: global-api; triage: implemented-needs-proof; 16 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_HousingBlueprint.CanImportTypeFromCurrentLocation-482` | `C_HousingBlueprint.CanImportTypeFromCurrentLocation` | `src/c_api/c_housing.rs:144`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:43` |
| `wt-global-api-C_HousingBlueprint.DeleteBlueprint-483` | `C_HousingBlueprint.DeleteBlueprint` | `src/c_api/c_housing.rs:147`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:59` |
| `wt-global-api-C_HousingBlueprint.ExportBlueprint-484` | `C_HousingBlueprint.ExportBlueprint` | `src/c_api/c_housing.rs:148`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:70` |
| `wt-global-api-C_HousingBlueprint.ExportRoomBlueprint-485` | `C_HousingBlueprint.ExportRoomBlueprint` | `src/c_api/c_housing.rs:152`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:82` |
| `wt-global-api-C_HousingBlueprint.GetBlueprintHyperlink-486` | `C_HousingBlueprint.GetBlueprintHyperlink` | `src/c_api/c_housing.rs:158`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:94` |
| `wt-global-api-C_HousingBlueprint.GetBlueprintTypeForCode-487` | `C_HousingBlueprint.GetBlueprintTypeForCode` | `src/c_api/c_housing.rs:164`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:110` |
| `wt-global-api-C_HousingBlueprint.GetExportAvailability-488` | `C_HousingBlueprint.GetExportAvailability` | `src/c_api/c_housing.rs:170`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:126` |
| `wt-global-api-C_HousingBlueprint.GetFeatureAvailability-489` | `C_HousingBlueprint.GetFeatureAvailability` | `src/c_api/c_housing.rs:176`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:136` |
| `wt-global-api-C_HousingBlueprint.GetImportAvailability-490` | `C_HousingBlueprint.GetImportAvailability` | `src/c_api/c_housing.rs:182`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:146` |
| `wt-global-api-C_HousingBlueprint.ImportBlueprint-491` | `C_HousingBlueprint.ImportBlueprint` | `src/c_api/c_housing.rs:185`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:156` |
| `wt-global-api-C_HousingBlueprint.IsShareCodeValid-492` | `C_HousingBlueprint.IsShareCodeValid` | `src/c_api/c_housing.rs:186`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:167` |
| `wt-global-api-C_HousingBlueprint.RenameBlueprint-493` | `C_HousingBlueprint.RenameBlueprint` | `src/c_api/c_housing.rs:187`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:183` |
| `wt-global-api-C_HousingBlueprint.RequestBlueprintCollection-494` | `C_HousingBlueprint.RequestBlueprintCollection` | `src/c_api/c_housing.rs:191`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:195` |
| `wt-global-api-C_HousingBlueprint.RequestBlueprintContentsForContext-495` | `C_HousingBlueprint.RequestBlueprintContentsForContext` | `src/c_api/c_housing.rs:203`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:211` |
| `wt-global-api-C_HousingBlueprint.RequestBlueprintContents-496` | `C_HousingBlueprint.RequestBlueprintContents` | `src/c_api/c_housing.rs:197`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:200` |
| `wt-global-api-C_HousingBlueprint.StartImportRoomBlueprint-497` | `C_HousingBlueprint.StartImportRoomBlueprint` | `src/c_api/c_housing.rs:209`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:223` |

## W174 — Tests-only candidate: C_HousingCustomizeMode

Section: global-api; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_HousingCustomizeMode.ApplyPetToSelectedDecor-499` | `C_HousingCustomizeMode.ApplyPetToSelectedDecor` | `src/c_api/c_housing.rs:252`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingCustomizeModeUIDocumentation.lua:23` |
| `wt-global-api-C_HousingCustomizeMode.GetSelectedDecorPetInfo-500` | `C_HousingCustomizeMode.GetSelectedDecorPetInfo` | `src/c_api/c_housing.rs:258`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingCustomizeModeUIDocumentation.lua:194` |

## W175 — Tests-only candidate: C_HousingDecor

Section: global-api; triage: implemented-needs-proof; 7 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_HousingDecor.AnyDecorPlacedInRoom-501` | `C_HousingDecor.AnyDecorPlacedInRoom` | `src/c_api/c_housing.rs:290`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingDecorUIDocumentation.lua:11` |
| `wt-global-api-C_HousingDecor.GetDecorAssignedPetName-504` | `C_HousingDecor.GetDecorAssignedPetName` | `src/c_api/c_housing.rs:308`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingDecorUIDocumentation.lua:78` |
| `wt-global-api-C_HousingDecor.GetDecorCanAttachPet-505` | `C_HousingDecor.GetDecorCanAttachPet` | `src/c_api/c_housing.rs:314`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingDecorUIDocumentation.lua:94` |
| `wt-global-api-C_HousingDecor.GetMaxPetPlacementBudget-506` | `C_HousingDecor.GetMaxPetPlacementBudget` | `src/c_api/c_housing.rs:320`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingDecorUIDocumentation.lua:183` |
| `wt-global-api-C_HousingDecor.GetSpentPetPlacementBudget-507` | `C_HousingDecor.GetSpentPetPlacementBudget` | `src/c_api/c_housing.rs:326`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingDecorUIDocumentation.lua:232` |
| `wt-global-api-C_HousingDecor.GetMaxPlacementBudget-589` | `C_HousingDecor.GetMaxPlacementBudget` | `src/lua_api/workarounds/temporary/housing_catalog_state.lua:717`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingDecorUIDocumentation.lua:193` |
| `wt-global-api-C_HousingDecor.GetSpentPlacementBudget-591` | `C_HousingDecor.GetSpentPlacementBudget` | `src/lua_api/workarounds/temporary/housing_catalog_state.lua:660`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingDecorUIDocumentation.lua:242` |

## W176 — Tests-only candidate: C_HousingLayout

Section: global-api; triage: implemented-needs-proof; 7 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_HousingLayout.GetBaseRoomFloor-508` | `C_HousingLayout.GetBaseRoomFloor` | `src/c_api/c_housing.rs:333`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingLayoutUIDocumentation.lua:63` |
| `wt-global-api-C_HousingLayout.GetRoomPlayerIsIn-511` | `C_HousingLayout.GetRoomPlayerIsIn` | `src/c_api/c_housing.rs:334`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingLayoutUIDocumentation.lua:109` |
| `wt-global-api-C_HousingLayout.GetSelectedBlueprintFloorplan-512` | `C_HousingLayout.GetSelectedBlueprintFloorplan` | `src/c_api/c_housing.rs:338`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingLayoutUIDocumentation.lua:120` |
| `wt-global-api-C_HousingLayout.HasSelectedBlueprintFloorplan-513` | `C_HousingLayout.HasSelectedBlueprintFloorplan` | `src/c_api/c_housing.rs:344`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingLayoutUIDocumentation.lua:211` |
| `wt-global-api-C_HousingLayout.GetNumFloors-568` | `C_HousingLayout.GetNumFloors` | `src/lua_api/workarounds/temporary/housing_catalog_state.lua:669`; `retail/AddOns/Blizzard_Deprecated/Mainline/Deprecated_12_1_0.lua:63` |
| `wt-global-api-C_HousingLayout.GetRoomPlacementBudget-593` | `C_HousingLayout.GetRoomPlacementBudget` | `src/lua_api/workarounds/temporary/housing_catalog_state.lua:661`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingLayoutUIDocumentation.lua:99` |
| `wt-global-api-C_HousingLayout.GetSpentPlacementBudget-595` | `C_HousingLayout.GetSpentPlacementBudget` | `src/lua_api/workarounds/temporary/housing_catalog_state.lua:660`; `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingLayoutUIDocumentation.lua:172` |

## W177 — Tests-only candidate: C_Item

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_Item.DoesItemMatchSpellItemCondition-515` | `C_Item.DoesItemMatchSpellItemCondition` | `src/c_api/item_spell/c_item.rs:96`; `retail/AddOns/Blizzard_APIDocumentationGenerated/ItemDocumentation.lua:154` |

## W178 — Tests-only candidate: C_LFGList

Section: global-api; triage: implemented-needs-proof; 5 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_LFGList.ConfirmCensoredActiveEntry-517` | `C_LFGList.ConfirmCensoredActiveEntry` | `src/lua_api/globals/lfg_list.rs:688`; `retail/AddOns/Blizzard_APIDocumentationGenerated/LFGListInfoDocumentation.lua:62` |
| `wt-global-api-C_LFGList.DoesCensoredTextMatch-518` | `C_LFGList.DoesCensoredTextMatch` | `src/lua_api/globals/lfg_list.rs:690`; `retail/AddOns/Blizzard_APIDocumentationGenerated/LFGListInfoDocumentation.lua:105` |
| `wt-global-api-C_LFGList.IsCensoredActiveEntryUnresolved-519` | `C_LFGList.IsCensoredActiveEntryUnresolved` | `src/lua_api/globals/lfg_list.rs:693`; `retail/AddOns/Blizzard_APIDocumentationGenerated/LFGListInfoDocumentation.lua:480` |
| `wt-global-api-C_LFGList.RevealCensoredActiveEntry-520` | `C_LFGList.RevealCensoredActiveEntry` | `src/lua_api/globals/lfg_list.rs:697`; `retail/AddOns/Blizzard_APIDocumentationGenerated/LFGListInfoDocumentation.lua:548` |
| `wt-global-api-C_LFGList.RevealCensoredSearchResult-521` | `C_LFGList.RevealCensoredSearchResult` | `src/lua_api/globals/lfg_list.rs:699`; `retail/AddOns/Blizzard_APIDocumentationGenerated/LFGListInfoDocumentation.lua:552` |

## W179 — Tests-only candidate: C_Navigation

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_Navigation.GetNextWaypointForMap-522` | `C_Navigation.GetNextWaypointForMap` | `src/lua_api/workarounds/temporary/navigation_defaults.rs:36`; `retail/AddOns/Blizzard_APIDocumentationGenerated/InGameNavigationDocumentation.lua:39` |

## W180 — Tests-only candidate: C_PaperDollInfo

Section: global-api; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_PaperDollInfo.GetInventorySlotInfoForInvSlot-525` | `C_PaperDollInfo.GetInventorySlotInfoForInvSlot` | `src/c_api/c_paper_doll_info.rs:54`; `retail/AddOns/Blizzard_APIDocumentationGenerated/PaperDollInfoDocumentation.lua:173` |
| `wt-global-api-C_PaperDollInfo.GetInventorySlotInfo-526` | `C_PaperDollInfo.GetInventorySlotInfo` | `src/c_api/c_paper_doll_info.rs:48`; `retail/AddOns/Blizzard_APIDocumentationGenerated/PaperDollInfoDocumentation.lua:155` |

## W181 — Tests-only candidate: C_PetJournal

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_PetJournal.GetPetInfoTableBySpeciesID-528` | `C_PetJournal.GetPetInfoTableBySpeciesID` | `src/lua_api/globals/font_strings_collection/pet_journal.rs:246`; `retail/AddOns/Blizzard_APIDocumentationGenerated/PetJournalInfoDocumentation.lua:166` |

## W182 — Tests-only candidate: C_Ping

Section: global-api; triage: already-removed/absent-as-required; 1 symbols.

Required later proof: absence/rejection after startup on Retail; static search is provisional.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_Ping.GetContextualPingTypeForUnit-569` | `C_Ping.GetContextualPingTypeForUnit` | `src/ptr/strict_removals.lua:46` |

## W183 — Tests-only candidate: C_Ping

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_Ping.SendMacroPing-597` | `C_Ping.SendMacroPing` | `src/lua_api/workarounds/temporary/ping_defaults.rs:16`; `retail/AddOns/Blizzard_APIDocumentationGenerated/PingManagerDocumentation.lua:53` |

## W184 — Tests-only candidate: C_PvP

Section: global-api; triage: already-removed/absent-as-required; 1 symbols.

Required later proof: absence/rejection after startup on Retail; static search is provisional.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_PvP.JoinRandomTrainingGround-570` | `C_PvP.JoinRandomTrainingGround` | INFERRED; no declaration found |

## W185 — Tests-only candidate: C_PvP

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_PvP.CanSurrenderArena-529` | `C_PvP.CanSurrenderArena` | `src/c_api/c_pvp.rs:89`; `retail/AddOns/Blizzard_APIDocumentationGenerated/PvpInfoDocumentation.lua:67` |

## W186 — Tests-only candidate: C_QuestHub

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_QuestHub.IsAreaPOICurrentlyRelatedToHub-532` | `C_QuestHub.IsAreaPOICurrentlyRelatedToHub` | `src/lua_api/workarounds/temporary/quest_objective_defaults.rs:27`; `retail/AddOns/Blizzard_APIDocumentationGenerated/QuestHubInfoDocumentation.lua:11` |

## W187 — Tests-only candidate: C_RecruitAFriend

Section: global-api; triage: already-removed/absent-as-required; 1 symbols.

Required later proof: absence/rejection after startup on Retail; static search is provisional.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_RecruitAFriend.IsEnabled-571` | `C_RecruitAFriend.IsEnabled` | `src/lua_api/globals/missing_surface/recruit_a_friend.rs:20`; `src/ptr/strict_removals.lua:47` |

## W188 — Tests-only candidate: C_RecruitAFriend

Section: global-api; triage: implemented-needs-proof; 3 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_RecruitAFriend.IsSystemEnabled-534` | `C_RecruitAFriend.IsSystemEnabled` | `src/lua_api/globals/missing_surface/recruit_a_friend.rs:22`; `retail/AddOns/Blizzard_APIDocumentationGenerated/RecruitAFriendDocumentation.lua:150` |
| `wt-global-api-C_RecruitAFriend.IsSystemSupported-535` | `C_RecruitAFriend.IsSystemSupported` | `src/lua_api/globals/missing_surface/recruit_a_friend.rs:24`; `retail/AddOns/Blizzard_APIDocumentationGenerated/RecruitAFriendDocumentation.lua:159` |
| `wt-global-api-C_RecruitAFriend.CanSummonFriend-604` | `C_RecruitAFriend.CanSummonFriend` | `src/lua_api/globals/missing_surface/recruit_a_friend.rs:26`; `retail/AddOns/Blizzard_APIDocumentationGenerated/RecruitAFriendDocumentation.lua:11` |

## W189 — Tests-only candidate: C_Roleset

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_Roleset.ApplyRolesetFilters-536` | `C_Roleset.ApplyRolesetFilters` | `src/lua_api/workarounds/temporary/roleset_defaults.rs:11`; `retail/AddOns/Blizzard_APIDocumentationGenerated/RolesetSystemDocumentation.lua:11` |

## W190 — Tests-only candidate: C_SocialQueue

Section: global-api; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_SocialQueue.IsSystemEnabled-539` | `C_SocialQueue.IsSystemEnabled` | `src/lua_api/workarounds/temporary/social_queue_defaults.rs:16`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SocialQueueSystemStatusDocumentation.lua:11` |
| `wt-global-api-C_SocialQueue.IsSystemSupported-540` | `C_SocialQueue.IsSystemSupported` | `src/lua_api/workarounds/temporary/social_queue_defaults.rs:20`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SocialQueueSystemStatusDocumentation.lua:20` |

## W191 — Tests-only candidate: C_SocialRestrictions

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_SocialRestrictions.IsFriendsDisabled-541` | `C_SocialRestrictions.IsFriendsDisabled` | `src/lua_api/workarounds/temporary/inert_global_defaults.rs:60`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SocialRestrictionsDocumentation.lua:44` |

## W192 — Tests-only candidate: C_SocialUI

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_SocialUI.IsSystemEnabled-542` | `C_SocialUI.IsSystemEnabled` | `src/lua_api/workarounds/temporary/inert_global_defaults.rs:64`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SocialUIDocumentation.lua:11` |

## W193 — Tests-only candidate: C_Sound

Section: global-api; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_Sound.PlaySoundWithOptions-543` | `C_Sound.PlaySoundWithOptions` | `src/lua_api/workarounds/temporary/sound_driver_defaults.rs:48`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SoundDocumentation.lua:74` |
| `wt-global-api-C_Sound.PlaySound-608` | `C_Sound.PlaySound` | `src/lua_api/workarounds/temporary/sound_driver_defaults.rs:44`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SoundDocumentation.lua:52` |

## W194 — Tests-only candidate: C_Spell

Section: global-api; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_Spell.TargetSpellChecksItemCondition-547` | `C_Spell.TargetSpellChecksItemCondition` | `src/c_api/c_spell.rs:121`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SpellDocumentation.lua:976` |
| `wt-global-api-C_Spell.GetSpellTexture-610` | `C_Spell.GetSpellTexture` | `src/c_api/c_spell.rs:71`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SpellDocumentation.lua:554` |

## W195 — Tests-only candidate: C_SuperTrack

Section: global-api; triage: already-removed/absent-as-required; 1 symbols.

Required later proof: absence/rejection after startup on Retail; static search is provisional.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_SuperTrack.GetNextWaypointForMap-572` | `C_SuperTrack.GetNextWaypointForMap` | `src/ptr/strict_removals.lua:48`; `retail/AddOns/Blizzard_Deprecated/Mainline/Deprecated_12_1_0.lua:8` |

## W196 — Tests-only candidate: C_UnitAuras

Section: global-api; triage: already-removed/absent-as-required; 1 symbols.

Required later proof: absence/rejection after startup on Retail; static search is provisional.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_UnitAuras.TriggerPrivateAuraShowDispelType-575` | `C_UnitAuras.TriggerPrivateAuraShowDispelType` | `src/lua_api/workarounds/temporary/private_aura_state.rs:70`; `src/ptr/strict_removals.lua:49` |

## W197 — Tests-only candidate: C_UnitAuras

Section: global-api; triage: implemented-needs-proof; 8 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_UnitAuras.AddAuraSound-550` | `C_UnitAuras.AddAuraSound` | `src/c_api/private_aura_sounds.rs:30`; `retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:11` |
| `wt-global-api-C_UnitAuras.GetGroupBuffVisualAlerts-552` | `C_UnitAuras.GetGroupBuffVisualAlerts` | `src/lua_api/workarounds/temporary/unit_auras_state.rs:17`; `retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:357` |
| `wt-global-api-C_UnitAuras.GetHiddenGroupBuffs-553` | `C_UnitAuras.GetHiddenGroupBuffs` | `src/lua_api/workarounds/temporary/unit_auras_state.rs:31`; `retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:366` |
| `wt-global-api-C_UnitAuras.RemoveAuraSound-554` | `C_UnitAuras.RemoveAuraSound` | `src/c_api/private_aura_sounds.rs:36`; `retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:490` |
| `wt-global-api-C_UnitAuras.SetGroupBuffVisualAlerts-555` | `C_UnitAuras.SetGroupBuffVisualAlerts` | `src/lua_api/workarounds/temporary/unit_auras_state.rs:22`; `retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:514` |
| `wt-global-api-C_UnitAuras.SetHiddenGroupBuffs-556` | `C_UnitAuras.SetHiddenGroupBuffs` | `src/lua_api/workarounds/temporary/unit_auras_state.rs:36`; `retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:525` |
| `wt-global-api-C_UnitAuras.AddPrivateAuraAppliedSound-573` | `C_UnitAuras.AddPrivateAuraAppliedSound` | `src/c_api/private_aura_sounds.rs:26`; `retail/AddOns/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:22` |
| `wt-global-api-C_UnitAuras.RemovePrivateAuraAppliedSound-574` | `C_UnitAuras.RemovePrivateAuraAppliedSound` | `src/c_api/private_aura_sounds.rs:20`; `retail/AddOns/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:26` |

## W198 — Tests-only candidate: global-api

Section: global-api; triage: already-removed/absent-as-required; 4 symbols.

Required later proof: absence/rejection after startup on Retail; static search is provisional.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-BNGetFriendInviteInfo-563` | `BNGetFriendInviteInfo` | INFERRED; no declaration found |
| `wt-global-api-BNSendVerifiedBattleTagInvite-564` | `BNSendVerifiedBattleTagInvite` | INFERRED; no declaration found |
| `wt-global-api-CanSurrenderArena-576` | `CanSurrenderArena` | INFERRED; no declaration found |
| `wt-global-api-SetTableSecurityOption-581` | `SetTableSecurityOption` | INFERRED; no declaration found |

## W199 — Tests-only candidate: retail/AddOns/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-CancelItemTempEnchantment-577` | `CancelItemTempEnchantment` | `retail/AddOns/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:28` |

## W200 — Tests-only candidate: src/c_api/c_paper_doll_info.rs

Section: global-api; triage: already-removed/absent-as-required; 1 symbols.

Required later proof: absence/rejection after startup on Retail; static search is provisional.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-GetInventorySlotInfo-579` | `GetInventorySlotInfo` | `src/c_api/c_paper_doll_info.rs:48`; `src/ptr/strict_removals.lua:51`; `retail/AddOns/Blizzard_APIDocumentationGenerated/PaperDollInfoDocumentation.lua:155` |

## W201 — Tests-only candidate: src/c_api/weapon_enchants.rs

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-GetWeaponEnchantInfo-580` | `GetWeaponEnchantInfo` | `src/c_api/weapon_enchants.rs:23`; `retail/AddOns/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:44` |

## W202 — Tests-only candidate: src/lua_api/globals/real/specialization_legacy.rs

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-GetInspectSpecialization-578` | `GetInspectSpecialization` | `src/lua_api/globals/real/specialization_legacy.rs:42`; `retail/AddOns/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:75`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SpecializationInfoDocumentation.lua:97` |

## W203 — Tests-only candidate: src/lua_api/globals/real/unit_relationships.rs

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-UnitIsPlayerControlledOrGroupMember-558` | `UnitIsPlayerControlledOrGroupMember` | `src/lua_api/globals/real/unit_relationships.rs:37`; `retail/AddOns/Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:2198` |

## W204 — Tests-only candidate: src/lua_api/workarounds/temporary/debug_environment_defaults.rs

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-CreateSecureDelegate-612` | `CreateSecureDelegate` | `src/lua_api/workarounds/temporary/debug_environment_defaults.rs:14`; `retail/AddOns/Blizzard_APIDocumentationGenerated/FrameScriptDocumentation.lua:98` |

## W205 — Tests-only candidate: src/lua_api/workarounds/temporary/securecopy.lua

Section: global-api; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-securecopy-559` | `securecopy` | `src/lua_api/workarounds/temporary/securecopy.lua:20`; `retail/AddOns/Blizzard_APIDocumentationGenerated/FrameScriptDocumentation.lua:438` |

## W206 — Tests-only candidate: src/ptr/compat_bootstrap.lua

Section: global-api; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-GetSpecializationSystem-557` | `GetSpecializationSystem` | `src/ptr/compat_bootstrap.lua:118`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SpecializationSharedDocumentation.lua:75` |
| `wt-global-api-settablesecurity-560` | `settablesecurity` | `src/ptr/compat_bootstrap.lua:127`; `retail/AddOns/Blizzard_APIDocumentationGenerated/FrameScriptDocumentation.lua:466` |

## W207 — Tests-only candidate: DurationTextBinding

Section: scriptobjects; triage: implemented-needs-proof; 6 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-scriptobjects-DurationTextBinding:Assign-1097` | `DurationTextBinding:Assign` | `src/c_api/duration_text_binding.rs:139`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:11` |
| `wt-scriptobjects-DurationTextBinding:ClearTextColorCurve-1098` | `DurationTextBinding:ClearTextColorCurve` | `src/c_api/duration_text_binding.rs:269`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:50` |
| `wt-scriptobjects-DurationTextBinding:Copy-1099` | `DurationTextBinding:Copy` | `src/c_api/duration_text_binding.rs:146`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:59` |
| `wt-scriptobjects-DurationTextBinding:GetFormattedTextColor-1100` | `DurationTextBinding:GetFormattedTextColor` | `src/c_api/duration_text_binding.rs:273`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:148` |
| `wt-scriptobjects-DurationTextBinding:GetTextColorCurve-1101` | `DurationTextBinding:GetTextColorCurve` | `src/c_api/duration_text_binding.rs:274`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:163` |
| `wt-scriptobjects-DurationTextBinding:SetTextColorCurve-1102` | `DurationTextBinding:SetTextColorCurve` | `src/c_api/duration_text_binding.rs:275`; `retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:305` |

## W208 — Tests-only candidate: SecondsFormatter

Section: scriptobjects; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-scriptobjects-SecondsFormatter:SetRounding-1104` | `SecondsFormatter:SetRounding` | `src/c_api/seconds_formatter/configuration.lua:44`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SecondsFormatterAPIDocumentation.lua:451` |

## W209 — Tests-only candidate: Frame

Section: widgets; triage: implemented-needs-proof; 7 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-widgets-Frame:AddRoleset-1146` | `Frame:AddRoleset` | `src/lua_api/frame/methods/text_attribute_event/rolesets.rs:11`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:18` |
| `wt-widgets-Frame:CreateVectorGraphics-1147` | `Frame:CreateVectorGraphics` | `src/lua_api/frame/methods/button_anchor_hierarchy/mod.rs:287`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:149` |
| `wt-widgets-Frame:GetOnUpdateMode-1148` | `Frame:GetOnUpdateMode` | `src/lua_api/frame/methods/text_attribute_event/mod.rs:151`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:536` |
| `wt-widgets-Frame:GetRolesetNames-1149` | `Frame:GetRolesetNames` | `src/lua_api/frame/methods/text_attribute_event/rolesets.rs:12`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:606` |
| `wt-widgets-Frame:RemoveRoleset-1151` | `Frame:RemoveRoleset` | `src/lua_api/frame/methods/text_attribute_event/rolesets.rs:13`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:1100` |
| `wt-widgets-Frame:SetOnUpdateMode-1153` | `Frame:SetOnUpdateMode` | `src/lua_api/frame/methods/text_attribute_event/mod.rs:150`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:1413` |
| `wt-widgets-Frame:SetRolesets-1154` | `Frame:SetRolesets` | `src/lua_api/frame/methods/text_attribute_event/rolesets.rs:14`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:1458` |

## W210 — Tests-only candidate: FrameScriptObject

Section: widgets; triage: implemented-needs-proof; 12 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-widgets-FrameScriptObject:AddAccessRestrictions-1117` | `FrameScriptObject:AddAccessRestrictions` | `src/lua_api/frame/methods/text_attribute_event/access_restrictions.rs:15`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameScriptObjectAPIDocumentation.lua:10` |
| `wt-widgets-FrameScriptObject:AddForbiddenAspects-1118` | `FrameScriptObject:AddForbiddenAspects` | `src/lua_api/frame/methods/text_attribute_event/mod.rs:60`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameScriptObjectAPIDocumentation.lua:22` |
| `wt-widgets-FrameScriptObject:AddSecretAspect-1119` | `FrameScriptObject:AddSecretAspect` | `src/lua_api/frame/methods/misc/secret.rs:21`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameScriptObjectAPIDocumentation.lua:34` |
| `wt-widgets-FrameScriptObject:CanBeAccessedInContext-1120` | `FrameScriptObject:CanBeAccessedInContext` | `src/lua_api/frame/methods/misc/secret.rs:38`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameScriptObjectAPIDocumentation.lua:45` |
| `wt-widgets-FrameScriptObject:GetAccessRestrictions-1121` | `FrameScriptObject:GetAccessRestrictions` | `src/lua_api/frame/methods/text_attribute_event/access_restrictions.rs:21`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameScriptObjectAPIDocumentation.lua:60` |
| `wt-widgets-FrameScriptObject:GetForbiddenAspects-1122` | `FrameScriptObject:GetForbiddenAspects` | `src/lua_api/frame/methods/text_attribute_event/mod.rs:61`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameScriptObjectAPIDocumentation.lua:75` |
| `wt-widgets-FrameScriptObject:GetInheritableForbiddenAspects-1123` | `FrameScriptObject:GetInheritableForbiddenAspects` | `src/lua_api/frame/methods/text_attribute_event/mod.rs:65`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameScriptObjectAPIDocumentation.lua:90` |
| `wt-widgets-FrameScriptObject:GetObjectTable-1124` | `FrameScriptObject:GetObjectTable` | `src/lua_api/frame/methods/text_attribute_event/mod.rs:74`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameScriptObjectAPIDocumentation.lua:121` |
| `wt-widgets-FrameScriptObject:HasAccessConstraints-1125` | `FrameScriptObject:HasAccessConstraints` | `src/lua_api/frame/methods/text_attribute_event/mod.rs:461`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameScriptObjectAPIDocumentation.lua:148` |
| `wt-widgets-FrameScriptObject:HasAnyAccessRestrictions-1126` | `FrameScriptObject:HasAnyAccessRestrictions` | `src/lua_api/frame/methods/text_attribute_event/access_restrictions.rs:27`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameScriptObjectAPIDocumentation.lua:163` |
| `wt-widgets-FrameScriptObject:HasAnyForbiddenAspects-1127` | `FrameScriptObject:HasAnyForbiddenAspects` | `src/lua_api/frame/methods/text_attribute_event/mod.rs:71`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameScriptObjectAPIDocumentation.lua:180` |
| `wt-widgets-FrameScriptObject:SetToDefaults-1214` | `FrameScriptObject:SetToDefaults` | `src/lua_api/frame/methods/forbidden_aspects.rs:72`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameScriptObjectAPIDocumentation.lua:297` |

## W211 — Tests-only candidate: Minimap

Section: widgets; triage: implemented-needs-proof; 1 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-widgets-Minimap:SetIconScale-1155` | `Minimap:SetIconScale` | `src/lua_api/frame/methods/map_frames.rs:97`; `retail/AddOns/Blizzard_APIDocumentationGenerated/MinimapFrameAPIDocumentation.lua:131` |

## W212 — Tests-only candidate: ScriptRegion

Section: widgets; triage: implemented-needs-proof; 4 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-widgets-ScriptRegion:ClearScripts-1216` | `ScriptRegion:ClearScripts` | `src/lua_api/frame/methods/text_attribute_event/mod.rs:53`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleScriptRegionAPIDocumentation.lua:50` |
| `wt-widgets-ScriptRegion:GetScript-1218` | `ScriptRegion:GetScript` | `src/lua_api/frame/methods/text_attribute_event/mod.rs:756`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleScriptRegionAPIDocumentation.lua:220` |
| `wt-widgets-ScriptRegion:HookScript-1226` | `ScriptRegion:HookScript` | `src/lua_api/frame/methods/text_attribute_event/mod.rs:758`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleScriptRegionAPIDocumentation.lua:328` |
| `wt-widgets-ScriptRegion:SetScript-1236` | `ScriptRegion:SetScript` | `src/lua_api/frame/methods/text_attribute_event/mod.rs:755`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleScriptRegionAPIDocumentation.lua:640` |

## W213 — Tests-only candidate: StatusBar

Section: widgets; triage: implemented-needs-proof; 2 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-widgets-StatusBar:GetRenderMode-1156` | `StatusBar:GetRenderMode` | `src/lua_api/frame/methods/widgets/statusbar.rs:523`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleStatusBarAPIDocumentation.lua:66` |
| `wt-widgets-StatusBar:SetRenderMode-1157` | `StatusBar:SetRenderMode` | `src/lua_api/frame/methods/widgets/statusbar.rs:521`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleStatusBarAPIDocumentation.lua:252` |

## W214 — Tests-only candidate: TextureBase

Section: widgets; triage: implemented-needs-proof; 13 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-widgets-TextureBase:ClearRadialProgressBar-1129` | `TextureBase:ClearRadialProgressBar` | `src/lua_api/frame/methods/widgets/texture/mod.rs:66`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleTextureBaseAPIDocumentation.lua:10` |
| `wt-widgets-TextureBase:ClearSVG-1130` | `TextureBase:ClearSVG` | `src/lua_api/frame/methods/widgets/texture/mod.rs:125`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleTextureBaseAPIDocumentation.lua:19` |
| `wt-widgets-TextureBase:GetRadialProgressBarEndOffset-1131` | `TextureBase:GetRadialProgressBarEndOffset` | `src/lua_api/frame/methods/widgets/texture/mod.rs:94`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleTextureBaseAPIDocumentation.lua:98` |
| `wt-widgets-TextureBase:GetRadialProgressBarFeather-1132` | `TextureBase:GetRadialProgressBarFeather` | `src/lua_api/frame/methods/widgets/texture/mod.rs:104`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleTextureBaseAPIDocumentation.lua:112` |
| `wt-widgets-TextureBase:GetRadialProgressBarPercent-1133` | `TextureBase:GetRadialProgressBarPercent` | `src/lua_api/frame/methods/widgets/texture/mod.rs:74`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleTextureBaseAPIDocumentation.lua:126` |
| `wt-widgets-TextureBase:GetRadialProgressBarReverse-1134` | `TextureBase:GetRadialProgressBarReverse` | `src/lua_api/frame/methods/widgets/texture/mod.rs:114`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleTextureBaseAPIDocumentation.lua:141` |
| `wt-widgets-TextureBase:GetRadialProgressBarStartOffset-1135` | `TextureBase:GetRadialProgressBarStartOffset` | `src/lua_api/frame/methods/widgets/texture/mod.rs:84`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleTextureBaseAPIDocumentation.lua:155` |
| `wt-widgets-TextureBase:SetRadialProgressBarEndOffset-1136` | `TextureBase:SetRadialProgressBarEndOffset` | `src/lua_api/frame/methods/widgets/texture/mod.rs:89`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleTextureBaseAPIDocumentation.lua:471` |
| `wt-widgets-TextureBase:SetRadialProgressBarFeather-1137` | `TextureBase:SetRadialProgressBarFeather` | `src/lua_api/frame/methods/widgets/texture/mod.rs:99`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleTextureBaseAPIDocumentation.lua:482` |
| `wt-widgets-TextureBase:SetRadialProgressBarPercent-1138` | `TextureBase:SetRadialProgressBarPercent` | `src/lua_api/frame/methods/widgets/texture/mod.rs:69`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleTextureBaseAPIDocumentation.lua:493` |
| `wt-widgets-TextureBase:SetRadialProgressBarReverse-1139` | `TextureBase:SetRadialProgressBarReverse` | `src/lua_api/frame/methods/widgets/texture/mod.rs:109`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleTextureBaseAPIDocumentation.lua:505` |
| `wt-widgets-TextureBase:SetRadialProgressBarStartOffset-1140` | `TextureBase:SetRadialProgressBarStartOffset` | `src/lua_api/frame/methods/widgets/texture/mod.rs:79`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleTextureBaseAPIDocumentation.lua:516` |
| `wt-widgets-TextureBase:SetSVG-1141` | `TextureBase:SetSVG` | `src/lua_api/frame/methods/widgets/texture/mod.rs:131`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleTextureBaseAPIDocumentation.lua:539` |

## W215 — Tests-only candidate: VectorGraphics

Section: widgets; triage: implemented-needs-proof; 4 symbols.

Required later proof: load/presence + valid calls per symbol; assert concrete results/state and changed annotations. Cached FrameXML helpers need only owning-file load/call proof.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-widgets-VectorGraphics:ClearSVG-1158` | `VectorGraphics:ClearSVG` | `src/lua_api/frame/methods/widgets/texture/mod.rs:125`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleVectorGraphicsAPIDocumentation.lua:10` |
| `wt-widgets-VectorGraphics:GetSVGFileID-1159` | `VectorGraphics:GetSVGFileID` | `src/lua_api/frame/methods/widgets/texture/mod.rs:127`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleVectorGraphicsAPIDocumentation.lua:18` |
| `wt-widgets-VectorGraphics:HasSVG-1160` | `VectorGraphics:HasSVG` | `src/lua_api/frame/methods/widgets/texture/mod.rs:129`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleVectorGraphicsAPIDocumentation.lua:31` |
| `wt-widgets-VectorGraphics:SetSVG-1161` | `VectorGraphics:SetSVG` | `src/lua_api/frame/methods/widgets/texture/mod.rs:131`; `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleVectorGraphicsAPIDocumentation.lua:44` |

## W216 — Modelable: src/cvars.rs

Section: cvars; triage: modelable; 6 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-cvars-tooltipShowAuraSpellIDs-1332` | `tooltipShowAuraSpellIDs` | INFERRED; no declaration found |
| `wt-cvars-worldMapShowCursorCoords-1334` | `worldMapShowCursorCoords` | INFERRED; no declaration found |
| `wt-cvars-worldMapShowPlayerCoords-1335` | `worldMapShowPlayerCoords` | INFERRED; no declaration found |
| `wt-cvars-auctionDisplayOnCharacter-1338` | `auctionDisplayOnCharacter` | `src/cvars.yaml:60` |
| `wt-cvars-auctionSortByBuyoutPrice-1339` | `auctionSortByBuyoutPrice` | `src/cvars.yaml:62` |
| `wt-cvars-auctionSortByUnitPrice-1340` | `auctionSortByUnitPrice` | `src/cvars.yaml:63` |

## W217 — Modelable: CHAT events

Section: events; triage: modelable; 1 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-CHAT_MSG_*-1300` | `CHAT_MSG_*` | INFERRED; no declaration found |

## W218 — Modelable: FULLSCREEN events

Section: events; triage: modelable; 2 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-FULLSCREEN_BROWSER_SPINNER_HIDE-1262` | `FULLSCREEN_BROWSER_SPINNER_HIDE` | `retail/AddOns/Blizzard_APIDocumentationGenerated/BrowserDocumentation.lua:21` |
| `wt-events-FULLSCREEN_BROWSER_SPINNER_SHOW-1263` | `FULLSCREEN_BROWSER_SPINNER_SHOW` | `retail/AddOns/Blizzard_APIDocumentationGenerated/BrowserDocumentation.lua:27` |

## W219 — Modelable: HOUSING events

Section: events; triage: modelable; 2 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-HOUSING_LAYOUT_OCCUPIED_FLOOR_RANGE_CHANGED-1283` | `HOUSING_LAYOUT_OCCUPIED_FLOOR_RANGE_CHANGED` | `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingLayoutUIDocumentation.lua:480` |
| `wt-events-HOUSING_LAYOUT_NUM_FLOORS_CHANGED-1297` | `HOUSING_LAYOUT_NUM_FLOORS_CHANGED` | `src/event/valid_events_b.rs:71` |

## W220 — Modelable: UNIT events

Section: events; triage: modelable; 1 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-events-UNIT_PING_PIN_REMOVED-1293` | `UNIT_PING_PIN_REMOVED` | `retail/AddOns/Blizzard_APIDocumentationGenerated/PingManagerSecureDocumentation.lua:265` |

## W221 — Modelable: EventUtil

Section: framexml; triage: modelable; 1 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-EventUtil.AreVariablesLoaded-986` | `EventUtil.AreVariablesLoaded` | `src/lua_api/env_init/shared_bootstrap.lua:663` |

## W222 — Modelable: framexml

Section: framexml; triage: modelable; 8 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-AnimatedShine_OnUpdate-964` | `AnimatedShine_OnUpdate` | `src/lua_api/env_init/runtime_surface_bootstrap.lua:1338` |
| `wt-framexml-ButtonPulse_OnUpdate-975` | `ButtonPulse_OnUpdate` | `src/lua_api/env_init/runtime_surface_bootstrap.lua:1325` |
| `wt-framexml-GetScaledCursorDelta-997` | `GetScaledCursorDelta` | `retail/AddOns/Blizzard_GlueParent/Mainline/GlueParent.lua:886` |
| `wt-framexml-GetScaledCursorPosition-999` | `GetScaledCursorPosition` | `retail/AddOns/Blizzard_GlueParent/Mainline/GlueParent.lua:880` |
| `wt-framexml-MacroFrame_SaveMacro-1017` | `MacroFrame_SaveMacro` | `retail/AddOns/Blizzard_MacroUI/Blizzard_MacroUI.lua:20` |
| `wt-framexml-PlayerChoiceToggle_TryShow-1027` | `PlayerChoiceToggle_TryShow` | `retail/AddOns/Blizzard_PlayerChoice/Blizzard_PlayerChoice_Bootstrap.lua:26` |
| `wt-framexml-RaidNotice_AddMessage-1032` | `RaidNotice_AddMessage` | `retail/AddOns/Blizzard_DeprecatedRaidWarning/Deprecated_RaidWarning.lua:8` |
| `wt-framexml-RaidNotice_Clear-1034` | `RaidNotice_Clear` | `retail/AddOns/Blizzard_DeprecatedRaidWarning/Deprecated_RaidWarning.lua:12` |

## W223 — Modelable: framexml

Section: framexml; triage: modelable; 3 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-RaidNotice_FadeInit-1035` | `RaidNotice_FadeInit` | `retail/AddOns/Blizzard_DeprecatedRaidWarning/Deprecated_RaidWarning.lua:30` |
| `wt-framexml-RaidNotice_UpdateSlot-1038` | `RaidNotice_UpdateSlot` | `retail/AddOns/Blizzard_DeprecatedRaidWarning/Deprecated_RaidWarning.lua:16` |
| `wt-framexml-TalentFrame_LoadUI-1060` | `TalentFrame_LoadUI` | `retail/AddOns/Blizzard_TalentUI/Blizzard_TalentUI_Bootstrap.lua:3` |

## W224 — Modelable: C_BattleNet

Section: global-api; triage: modelable; 3 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_BattleNet.CanToggleHighResTexturesWithoutClientReload-427` | `C_BattleNet.CanToggleHighResTexturesWithoutClientReload` | `retail/AddOns/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:58` |
| `wt-global-api-C_BattleNet.SearchFriends-432` | `C_BattleNet.SearchFriends` | `retail/AddOns/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:239` |
| `wt-global-api-C_BattleNet.SendTitleFriendInviteByName-433` | `C_BattleNet.SendTitleFriendInviteByName` | `retail/AddOns/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:272` |

## W225 — Modelable: C_Browser

Section: global-api; triage: modelable; 1 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_Browser.CloseFullscreenBrowser-438` | `C_Browser.CloseFullscreenBrowser` | `retail/AddOns/Blizzard_APIDocumentationGenerated/BrowserDocumentation.lua:11` |

## W226 — Modelable: C_ClientScene

Section: global-api; triage: modelable; 1 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_ClientScene.IsSceneTypeActive-440` | `C_ClientScene.IsSceneTypeActive` | `retail/AddOns/Blizzard_APIDocumentationGenerated/ClientSceneDocumentation.lua:11` |

## W227 — Modelable: C_Club

Section: global-api; triage: modelable; 1 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_Club.SendTitleFriendRequest-441` | `C_Club.SendTitleFriendRequest` | `retail/AddOns/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1041` |

## W228 — Modelable: C_DelvesUI

Section: global-api; triage: modelable; 3 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_DelvesUI.HasActiveLFGLair-445` | `C_DelvesUI.HasActiveLFGLair` | `retail/AddOns/Blizzard_APIDocumentationGenerated/DelvesUIDocumentation.lua:382` |
| `wt-global-api-C_DelvesUI.HasActiveLair-446` | `C_DelvesUI.HasActiveLair` | `retail/AddOns/Blizzard_APIDocumentationGenerated/DelvesUIDocumentation.lua:391` |
| `wt-global-api-C_DelvesUI.IsInLair-447` | `C_DelvesUI.IsInLair` | `retail/AddOns/Blizzard_APIDocumentationGenerated/DelvesUIDocumentation.lua:431` |

## W229 — Modelable: C_Discord

Section: global-api; triage: modelable; 1 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_Discord.GetDiscordUserName-451` | `C_Discord.GetDiscordUserName` | `retail/AddOns/Blizzard_APIDocumentationGenerated/DiscordDocumentation.lua:43` |

## W230 — Modelable: C_HousingBlueprint

Section: global-api; triage: modelable; 3 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_HousingBlueprint.CanExportRoom-480` | `C_HousingBlueprint.CanExportRoom` | `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:11` |
| `wt-global-api-C_HousingBlueprint.CanExportTypeFromCurrentLocation-481` | `C_HousingBlueprint.CanExportTypeFromCurrentLocation` | `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:27` |
| `wt-global-api-C_HousingBlueprint.UpdateBlueprintStringFromInput-498` | `C_HousingBlueprint.UpdateBlueprintStringFromInput` | `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingBlueprintUIDocumentation.lua:234` |

## W231 — Modelable: C_HousingDecor

Section: global-api; triage: modelable; 2 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_HousingDecor.GetAllMaxPlacementBudgets-502` | `C_HousingDecor.GetAllMaxPlacementBudgets` | `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingDecorUIDocumentation.lua:45` |
| `wt-global-api-C_HousingDecor.GetAllSpentPlacementBudgets-503` | `C_HousingDecor.GetAllSpentPlacementBudgets` | `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingDecorUIDocumentation.lua:67` |

## W232 — Modelable: C_HousingLayout

Section: global-api; triage: modelable; 3 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_HousingLayout.GetHighestOccupiedFloorIndex-509` | `C_HousingLayout.GetHighestOccupiedFloorIndex` | `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingLayoutUIDocumentation.lua:72` |
| `wt-global-api-C_HousingLayout.GetLowestOccupiedFloorIndex-510` | `C_HousingLayout.GetLowestOccupiedFloorIndex` | `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingLayoutUIDocumentation.lua:81` |
| `wt-global-api-C_HousingLayout.RoomHasStairs-514` | `C_HousingLayout.RoomHasStairs` | `retail/AddOns/Blizzard_APIDocumentationGenerated/HousingLayoutUIDocumentation.lua:341` |

## W233 — Modelable: C_LFGInfo

Section: global-api; triage: modelable; 1 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_LFGInfo.IsInMatchmadeRaidWithoutRoleRequirements-516` | `C_LFGInfo.IsInMatchmadeRaidWithoutRoleRequirements` | `retail/AddOns/Blizzard_APIDocumentationGenerated/LFGInfoDocumentation.lua:219` |

## W234 — Modelable: C_NeighborhoodInitiative

Section: global-api; triage: modelable; 1 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_NeighborhoodInitiative.GetInitiativeTaskRewardScaling-523` | `C_NeighborhoodInitiative.GetInitiativeTaskRewardScaling` | `retail/AddOns/Blizzard_APIDocumentationGenerated/NeighborhoodInitiativeDocumentation.lua:78` |

## W235 — Modelable: C_PaperDollInfo

Section: global-api; triage: modelable; 2 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_PaperDollInfo.CancelTemporaryEnchantment-524` | `C_PaperDollInfo.CancelTemporaryEnchantment` | `retail/AddOns/Blizzard_APIDocumentationGenerated/PaperDollInfoDocumentation.lua:35` |
| `wt-global-api-C_PaperDollInfo.GetTemporaryEnchantmentInfo-527` | `C_PaperDollInfo.GetTemporaryEnchantmentInfo` | `retail/AddOns/Blizzard_APIDocumentationGenerated/PaperDollInfoDocumentation.lua:217` |

## W236 — Modelable: C_PvP

Section: global-api; triage: modelable; 2 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_PvP.JoinRandomTrainingGroundArena-530` | `C_PvP.JoinRandomTrainingGroundArena` | `retail/AddOns/Blizzard_APIDocumentationGenerated/PvpInfoDocumentation.lua:1102` |
| `wt-global-api-C_PvP.JoinRandomTrainingGroundBattleground-531` | `C_PvP.JoinRandomTrainingGroundBattleground` | `retail/AddOns/Blizzard_APIDocumentationGenerated/PvpInfoDocumentation.lua:1106` |

## W237 — Modelable: C_QuestHub

Section: global-api; triage: modelable; 1 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_QuestHub.IsQuestCurrentlyRelatedToHub-602` | `C_QuestHub.IsQuestCurrentlyRelatedToHub` | `retail/AddOns/Blizzard_APIDocumentationGenerated/QuestHubInfoDocumentation.lua:27` |

## W238 — Modelable: C_RecentAllies

Section: global-api; triage: modelable; 1 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_RecentAllies.SearchRecentAllies-533` | `C_RecentAllies.SearchRecentAllies` | `retail/AddOns/Blizzard_APIDocumentationGenerated/RecentAlliesDocumentation.lua:145` |

## W239 — Modelable: C_Roleset

Section: global-api; triage: modelable; 2 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_Roleset.GetActiveAllowedRolesets-537` | `C_Roleset.GetActiveAllowedRolesets` | `retail/AddOns/Blizzard_APIDocumentationGenerated/RolesetSystemDocumentation.lua:23` |
| `wt-global-api-C_Roleset.GetActiveBlockedRolesets-538` | `C_Roleset.GetActiveBlockedRolesets` | `retail/AddOns/Blizzard_APIDocumentationGenerated/RolesetSystemDocumentation.lua:33` |

## W240 — Modelable: C_SpecializationInfo

Section: global-api; triage: modelable; 1 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_SpecializationInfo.GetInspectSpecialization-544` | `C_SpecializationInfo.GetInspectSpecialization` | `retail/AddOns/Blizzard_APIDocumentationGenerated/SpecializationInfoDocumentation.lua:97` |

## W241 — Modelable: C_Spell

Section: global-api; triage: modelable; 2 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_Spell.GetLastCategoryCooldownSource-545` | `C_Spell.GetLastCategoryCooldownSource` | `retail/AddOns/Blizzard_APIDocumentationGenerated/SpellDocumentation.lua:113` |
| `wt-global-api-C_Spell.GetSpellDescriptionForItemLocation-546` | `C_Spell.GetSpellDescriptionForItemLocation` | `retail/AddOns/Blizzard_APIDocumentationGenerated/SpellDocumentation.lua:321` |

## W242 — Modelable: C_TransmogOutfitInfo

Section: global-api; triage: modelable; 2 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_TransmogOutfitInfo.CanPlayerTransmogSlot-548` | `C_TransmogOutfitInfo.CanPlayerTransmogSlot` | `retail/AddOns/Blizzard_APIDocumentationGenerated/TransmogOutfitInfoDocumentation.lua:23` |
| `wt-global-api-C_TransmogOutfitInfo.IsTransmogEnabled-549` | `C_TransmogOutfitInfo.IsTransmogEnabled` | `retail/AddOns/Blizzard_APIDocumentationGenerated/TransmogOutfitInfoDocumentation.lua:631` |

## W243 — Modelable: C_UnitAuras

Section: global-api; triage: modelable; 1 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-global-api-C_UnitAuras.CancelAuraByInstanceID-551` | `C_UnitAuras.CancelAuraByInstanceID` | `retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:84` |

## W244 — Modelable: SecondsFormatter

Section: scriptobjects; triage: modelable; 1 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-scriptobjects-SecondsFormatter:GetRounding-1103` | `SecondsFormatter:GetRounding` | `retail/AddOns/Blizzard_APIDocumentationGenerated/SecondsFormatterAPIDocumentation.lua:282` |

## W245 — Modelable: Animation

Section: widgets; triage: modelable; 3 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-widgets-Animation:GetScript-1166` | `Animation:GetScript` | `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleAnimAPIDocumentation.lua:88` |
| `wt-widgets-Animation:HookScript-1174` | `Animation:HookScript` | `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleAnimAPIDocumentation.lua:176` |
| `wt-widgets-Animation:SetScript-1184` | `Animation:SetScript` | `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleAnimAPIDocumentation.lua:354` |

## W246 — Modelable: AnimationGroup

Section: widgets; triage: modelable; 3 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-widgets-AnimationGroup:GetScript-1190` | `AnimationGroup:GetScript` | `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleAnimGroupAPIDocumentation.lua:126` |
| `wt-widgets-AnimationGroup:HookScript-1198` | `AnimationGroup:HookScript` | `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleAnimGroupAPIDocumentation.lua:162` |
| `wt-widgets-AnimationGroup:SetScript-1208` | `AnimationGroup:SetScript` | `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleAnimGroupAPIDocumentation.lua:327` |

## W247 — Modelable: FontString

Section: widgets; triage: modelable; 1 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-widgets-FontString:SetDesaturateEmbeddedTextures-1128` | `FontString:SetDesaturateEmbeddedTextures` | `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFontStringAPIDocumentation.lua:480` |

## W248 — Modelable: Frame

Section: widgets; triage: modelable; 2 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-widgets-Frame:IsRolesetFiltered-1150` | `Frame:IsRolesetFiltered` | `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:900` |
| `wt-widgets-Frame:ResizeToBoundsRect-1152` | `Frame:ResizeToBoundsRect` | `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:1112` |

## W249 — Modelable: RadialProgress

Section: widgets; triage: modelable; 4 symbols.

Bounded work: use each cited declaration, or explicitly INFERRED name/sibling contract where none exists. Removed-name conflicts require deprecated-wrapper reconciliation first. No hard-case architecture/design proposed.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-widgets-RadialProgress:GetFromPercent-1142` | `RadialProgress:GetFromPercent` | `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleAnimRadialProgressAPIDocumentation.lua:10` |
| `wt-widgets-RadialProgress:GetToPercent-1143` | `RadialProgress:GetToPercent` | `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleAnimRadialProgressAPIDocumentation.lua:23` |
| `wt-widgets-RadialProgress:SetFromPercent-1144` | `RadialProgress:SetFromPercent` | `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleAnimRadialProgressAPIDocumentation.lua:36` |
| `wt-widgets-RadialProgress:SetToPercent-1145` | `RadialProgress:SetToPercent` | `retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleAnimRadialProgressAPIDocumentation.lua:46` |

## W250 — Deferred; no design: PingUtil

Section: framexml; triage: needs-look; 1 symbols.

Defer; no implementation design. Resolve producer/contract evidence before scheduling work.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-PingUtil.GetContextualPingTypeForUnit-1026` | `PingUtil.GetContextualPingTypeForUnit` | `src/ptr/strict_removals.lua:46` |

## W251 — Deferred; no design: framexml

Section: framexml; triage: needs-look; 6 symbols.

Defer; no implementation design. Resolve producer/contract evidence before scheduling work.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-ShouldDisplaySpellCooldown-888` | `ShouldDisplaySpellCooldown` | INFERRED; no declaration found |
| `wt-framexml-IsPlayerAtEffectiveMaxLevel-1012` | `IsPlayerAtEffectiveMaxLevel` | `src/iced_app/update.rs:269`; `src/lua_api/globals/real/xp_honor_rest.rs:130` |
| `wt-framexml-NPETutorial_AttemptToBegin-1021` | `NPETutorial_AttemptToBegin` | `src/lua_api/workarounds/temporary/source_patches.rs:185`; `src/lua_api/workarounds/temporary/source_patches.rs:186` |
| `wt-framexml-SetDesaturation-1055` | `SetDesaturation` | `src/lua_api/frame/methods/widgets/texture/mod.rs:42` |
| `wt-framexml-UIParent_ManageFramePositions-1070` | `UIParent_ManageFramePositions` | `src/lua_api/globals/stubs/global_stubs.rs:84`; `src/lua_api/workarounds/editmode/apply_system_anchors.lua:265` |
| `wt-framexml-UIParentLoadAddOn-1078` | `UIParentLoadAddOn` | `src/lua_api/workarounds/temporary/player_spells_onload_backfill.rs:48`; `src/lua_api/workarounds/temporary/player_spells_onload_backfill.rs:49` |

## W252 — Deferred; no design: retail/AddOns/Blizzard_RestrictedAddOnEnvironment/SecureAuraHeader.lua

Section: framexml; triage: needs-look; 9 symbols.

Defer; no implementation design. Resolve producer/contract evidence before scheduling work.

| Source ID | Symbol | Declaration / producer |
|---|---|---|
| `wt-framexml-SecureAuraHeader_GetUnit-1046` | `SecureAuraHeader_GetUnit` | `retail/AddOns/Blizzard_RestrictedAddOnEnvironment/SecureAuraHeader.lua:112` |
| `wt-framexml-SecureAuraHeader_OnAttributeChanged-1047` | `SecureAuraHeader_OnAttributeChanged` | `retail/AddOns/Blizzard_RestrictedAddOnEnvironment/SecureAuraHeader.lua:94` |
| `wt-framexml-SecureAuraHeader_OnEvent-1048` | `SecureAuraHeader_OnEvent` | `retail/AddOns/Blizzard_RestrictedAddOnEnvironment/SecureAuraHeader.lua:86` |
| `wt-framexml-SecureAuraHeader_OnHide-1049` | `SecureAuraHeader_OnHide` | `retail/AddOns/Blizzard_RestrictedAddOnEnvironment/SecureAuraHeader.lua:66` |
| `wt-framexml-SecureAuraHeader_OnLoad-1050` | `SecureAuraHeader_OnLoad` | `retail/AddOns/Blizzard_RestrictedAddOnEnvironment/SecureAuraHeader.lua:57` |
| `wt-framexml-SecureAuraHeader_OnShow-1051` | `SecureAuraHeader_OnShow` | `retail/AddOns/Blizzard_RestrictedAddOnEnvironment/SecureAuraHeader.lua:61` |
| `wt-framexml-SecureAuraHeader_OnUpdate-1052` | `SecureAuraHeader_OnUpdate` | `retail/AddOns/Blizzard_RestrictedAddOnEnvironment/SecureAuraHeader.lua:70` |
| `wt-framexml-SecureAuraHeader_UpdateEventRegistrations-1053` | `SecureAuraHeader_UpdateEventRegistrations` | `retail/AddOns/Blizzard_RestrictedAddOnEnvironment/SecureAuraHeader.lua:116` |
| `wt-framexml-SecureAuraHeader_Update-1054` | `SecureAuraHeader_Update` | `retail/AddOns/Blizzard_RestrictedAddOnEnvironment/SecureAuraHeader.lua:392` |
