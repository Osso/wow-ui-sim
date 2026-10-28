# PTR startup event registration

PTR accepts the following event names for frame registration and event validity. This contract does not define event delivery or payloads. An undeclared invented name remains rejected.

The declarations are in the pinned PTR `Blizzard_APIDocumentationGenerated` cache:

| Event | Declaration |
|---|---|
| ACTION_COMBAT_ACTION_RANGE_CHECK_UPDATE | ActionBarFrameDocumentation.lua:1090 |
| EXTERNAL_EVENT_LAUNCH_URL_RESULT | ExternalEventURLDocumentation.lua:49 |
| GAME_MODE_EXP_BAR_DATA_UPDATED | GameModeExpBarUIDocumentation.lua:44 |
| GAME_RULES_CHANGED | GameRulesDocumentation.lua:447 |
| PLAYER_PVP_FLAG_CHANGED | PvpInfoDocumentation.lua:1386 |
| SECURE_TRANSFER_CONFIRM_VC_PRODUCT_PURCHASE | SecureTransferDocumentation.lua:161 |
| SHARD_TRANSFER_CANCELLED | SystemDocumentation.lua:183 |
| UNIT_PET_TRAINING_POINTS | UnitDocumentation.lua:4471 |
| VOICE_CHAT_VOICE_PROVIDER_CHANGED | VoiceChatDocumentation.lua:1398 |

`FORCE_DECLINE_CHARACTER` appears in a PTR consumer but has no declaration in that cache and is not added by this contract.
