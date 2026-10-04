# Retail 12.0.7 B39–B44 author handoff

Base: `7702befe8`. Author-only; no integration, compilation, tests or simulator execution.

Active goal: evidence for every B39–B44 row, uniquely anchored state/producer/test-support edits, staged public-API behavioral tests and spec. Writes restricted to this audit cache. B44 requires explicit blocker reconciliation.

Status: author deliverables complete; static checks only. No integration or executed behavior proof.

## Early blocking evidence

B44: source says “Fixed secret value errors in SetFrameStrata.” Later cache `Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:1292–1300` explicitly says `SecretArguments = "NotAllowed"` and `IsProtectedFunction = true`. No authenticated 12.0.7 selector policy; cannot choose AllowedWhenUntainted/Always based on a bugfix sentence. Current `src/lua_api/frame/methods/core_state/strata_level.rs:13–23` converts receiver and string before protected-state checks. Replacing string conversion with secret unwrapping would contradict available declaration. Preserve behavior until historical declaration/native secret matrix reconciles conflict; no B44 producer edit.

B41 cache contradicts temporary defaults: GetEventCPUUsage declares no arguments and two results (time/count), GetFunctionCPUUsage no arguments and two results, GetScriptCPUUsage no arguments and one result. A keyed function/frame model would invent undeclared arguments. Audit consumer use before selecting bounded state model.

## Model boundary selected

B39–B40: empty per-environment host ingress queue; nine exemptions affect only classification caused by lockdown. Independently secret host arguments remain authentic VM secrets. `CHAT_MSG_SAY` is the unchanged restricted control. No blanket unwrap at dispatch.

B41: bounded no-argument snapshot contract following later cache, independent event/function time-count and script scalar. Empty inputs yield 0/0 or 0. INFERRED units, attribution, extra-argument ignoring and NeverSecret policy; no native timing/reset parity. No cached Lua consumers of these three profiling getters were found.

B42: cached event is synchronous and carries actual SimpleTexture identity plus enum; enum has Requested/NotAllowed, NOT Cancelled. A terminal-only/cancelled model would contradict cached consumers. Preserve request semantics as blocked until restrictions and lifecycle are authenticated; this handoff will not invent a network request implementation.

## Per-row evidence — 16 retained rows

All generated declarations below are later-cache evidence, possibly postdating 12.0.7. No historical/native acceptance inferred. Source quotes come from the retained excerpt, not a web fetch.

### `events-CHAT_MSG_COMBAT_FACTION_CHANGE-149`

Source L149:
> CHAT_MSG_COMBAT_FACTION_CHANGE - SecretInChatMessagingLockdown

Cached declaration (quoted):
```text
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1347: 		{
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1348: 			Name = "ChatMsgCombatFactionChange",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1349: 			Type = "Event",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1350: 			LiteralName = "CHAT_MSG_COMBAT_FACTION_CHANGE",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1351: 			SynchronousEvent = true,
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1352: 			Payload =
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1353: 			{
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1354: 				{ Name = "text", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1355: 				{ Name = "playerName", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1356: 				{ Name = "languageName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1357: 				{ Name = "channelName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1358: 				{ Name = "playerName2", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1359: 				{ Name = "specialFlags", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1360: 				{ Name = "zoneChannelID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1361: 				{ Name = "channelIndex", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1362: 				{ Name = "channelBaseName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1363: 				{ Name = "languageID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1364: 				{ Name = "lineID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1365: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1366: 				{ Name = "bnSenderID", Type = "number", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1367: 				{ Name = "isMobile", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1368: 				{ Name = "isSubtitle", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1369: 				{ Name = "hideSenderInLetterbox", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1370: 				{ Name = "suppressRaidIcons", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1371: 				{ Name = "discordInfo", Type = "DiscordChatInfo", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1372: 			},
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1373: 		},
```

Exists on master: `src/event/valid_events_a.rs:332`.

Required change / limit: Add gated, empty host chat input queue and rooted Rust delivery. Exempt this name from lockdown-caused secrecy only; retain independent host source secrets and NeverSecret fields. Four-value primitive projection only; complete 18-field/DiscordChatInfo and all native ingress paths remain unproved.

### `events-CHAT_MSG_COMBAT_HONOR_GAIN-150`

Source L150:
> CHAT_MSG_COMBAT_HONOR_GAIN - SecretInChatMessagingLockdown

Cached declaration (quoted):
```text
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1374: 		{
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1375: 			Name = "ChatMsgCombatHonorGain",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1376: 			Type = "Event",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1377: 			LiteralName = "CHAT_MSG_COMBAT_HONOR_GAIN",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1378: 			SynchronousEvent = true,
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1379: 			Payload =
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1380: 			{
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1381: 				{ Name = "text", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1382: 				{ Name = "playerName", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1383: 				{ Name = "languageName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1384: 				{ Name = "channelName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1385: 				{ Name = "playerName2", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1386: 				{ Name = "specialFlags", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1387: 				{ Name = "zoneChannelID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1388: 				{ Name = "channelIndex", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1389: 				{ Name = "channelBaseName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1390: 				{ Name = "languageID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1391: 				{ Name = "lineID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1392: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1393: 				{ Name = "bnSenderID", Type = "number", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1394: 				{ Name = "isMobile", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1395: 				{ Name = "isSubtitle", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1396: 				{ Name = "hideSenderInLetterbox", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1397: 				{ Name = "suppressRaidIcons", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1398: 				{ Name = "discordInfo", Type = "DiscordChatInfo", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1399: 			},
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1400: 		},
```

Exists on master: `src/event/valid_events_a.rs:333`.

Required change / limit: Add gated, empty host chat input queue and rooted Rust delivery. Exempt this name from lockdown-caused secrecy only; retain independent host source secrets and NeverSecret fields. Four-value primitive projection only; complete 18-field/DiscordChatInfo and all native ingress paths remain unproved.

### `events-CHAT_MSG_COMBAT_MISC_INFO-151`

Source L151:
> CHAT_MSG_COMBAT_MISC_INFO - SecretInChatMessagingLockdown

Cached declaration (quoted):
```text
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1401: 		{
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1402: 			Name = "ChatMsgCombatMiscInfo",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1403: 			Type = "Event",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1404: 			LiteralName = "CHAT_MSG_COMBAT_MISC_INFO",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1405: 			SynchronousEvent = true,
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1406: 			Payload =
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1407: 			{
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1408: 				{ Name = "text", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1409: 				{ Name = "playerName", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1410: 				{ Name = "languageName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1411: 				{ Name = "channelName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1412: 				{ Name = "playerName2", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1413: 				{ Name = "specialFlags", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1414: 				{ Name = "zoneChannelID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1415: 				{ Name = "channelIndex", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1416: 				{ Name = "channelBaseName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1417: 				{ Name = "languageID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1418: 				{ Name = "lineID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1419: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1420: 				{ Name = "bnSenderID", Type = "number", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1421: 				{ Name = "isMobile", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1422: 				{ Name = "isSubtitle", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1423: 				{ Name = "hideSenderInLetterbox", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1424: 				{ Name = "suppressRaidIcons", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1425: 				{ Name = "discordInfo", Type = "DiscordChatInfo", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1426: 			},
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1427: 		},
```

Exists on master: `src/event/valid_events_a.rs:334`.

Required change / limit: Add gated, empty host chat input queue and rooted Rust delivery. Exempt this name from lockdown-caused secrecy only; retain independent host source secrets and NeverSecret fields. Four-value primitive projection only; complete 18-field/DiscordChatInfo and all native ingress paths remain unproved.

### `events-CHAT_MSG_COMBAT_XP_GAIN-152`

Source L152:
> CHAT_MSG_COMBAT_XP_GAIN - SecretInChatMessagingLockdown

Cached declaration (quoted):
```text
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1428: 		{
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1429: 			Name = "ChatMsgCombatXpGain",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1430: 			Type = "Event",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1431: 			LiteralName = "CHAT_MSG_COMBAT_XP_GAIN",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1432: 			SynchronousEvent = true,
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1433: 			Payload =
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1434: 			{
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1435: 				{ Name = "text", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1436: 				{ Name = "playerName", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1437: 				{ Name = "languageName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1438: 				{ Name = "channelName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1439: 				{ Name = "playerName2", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1440: 				{ Name = "specialFlags", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1441: 				{ Name = "zoneChannelID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1442: 				{ Name = "channelIndex", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1443: 				{ Name = "channelBaseName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1444: 				{ Name = "languageID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1445: 				{ Name = "lineID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1446: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1447: 				{ Name = "bnSenderID", Type = "number", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1448: 				{ Name = "isMobile", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1449: 				{ Name = "isSubtitle", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1450: 				{ Name = "hideSenderInLetterbox", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1451: 				{ Name = "suppressRaidIcons", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1452: 				{ Name = "discordInfo", Type = "DiscordChatInfo", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1453: 			},
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1454: 		},
```

Exists on master: `src/event/valid_events_a.rs:335`.

Required change / limit: Add gated, empty host chat input queue and rooted Rust delivery. Exempt this name from lockdown-caused secrecy only; retain independent host source secrets and NeverSecret fields. Four-value primitive projection only; complete 18-field/DiscordChatInfo and all native ingress paths remain unproved.

### `events-CHAT_MSG_CURRENCY-153`

Source L153:
> CHAT_MSG_CURRENCY - SecretInChatMessagingLockdown

Cached declaration (quoted):
```text
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1483: 		{
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1484: 			Name = "ChatMsgCurrency",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1485: 			Type = "Event",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1486: 			LiteralName = "CHAT_MSG_CURRENCY",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1487: 			SynchronousEvent = true,
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1488: 			Payload =
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1489: 			{
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1490: 				{ Name = "text", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1491: 				{ Name = "playerName", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1492: 				{ Name = "languageName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1493: 				{ Name = "channelName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1494: 				{ Name = "playerName2", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1495: 				{ Name = "specialFlags", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1496: 				{ Name = "zoneChannelID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1497: 				{ Name = "channelIndex", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1498: 				{ Name = "channelBaseName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1499: 				{ Name = "languageID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1500: 				{ Name = "lineID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1501: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1502: 				{ Name = "bnSenderID", Type = "number", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1503: 				{ Name = "isMobile", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1504: 				{ Name = "isSubtitle", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1505: 				{ Name = "hideSenderInLetterbox", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1506: 				{ Name = "suppressRaidIcons", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1507: 				{ Name = "discordInfo", Type = "DiscordChatInfo", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1508: 			},
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1509: 		},
```

Exists on master: `src/event/valid_events_a.rs:337`.

Required change / limit: Add gated, empty host chat input queue and rooted Rust delivery. Exempt this name from lockdown-caused secrecy only; retain independent host source secrets and NeverSecret fields. Four-value primitive projection only; complete 18-field/DiscordChatInfo and all native ingress paths remain unproved.

### `events-CHAT_MSG_FILTERED-154`

Source L154:
> CHAT_MSG_FILTERED - SecretInChatMessagingLockdown

Cached declaration (quoted):
```text
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1566: 		{
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1567: 			Name = "ChatMsgFiltered",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1568: 			Type = "Event",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1569: 			LiteralName = "CHAT_MSG_FILTERED",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1570: 			SynchronousEvent = true,
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1571: 			Payload =
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1572: 			{
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1573: 				{ Name = "text", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1574: 				{ Name = "playerName", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1575: 				{ Name = "languageName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1576: 				{ Name = "channelName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1577: 				{ Name = "playerName2", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1578: 				{ Name = "specialFlags", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1579: 				{ Name = "zoneChannelID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1580: 				{ Name = "channelIndex", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1581: 				{ Name = "channelBaseName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1582: 				{ Name = "languageID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1583: 				{ Name = "lineID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1584: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1585: 				{ Name = "bnSenderID", Type = "number", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1586: 				{ Name = "isMobile", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1587: 				{ Name = "isSubtitle", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1588: 				{ Name = "hideSenderInLetterbox", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1589: 				{ Name = "suppressRaidIcons", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1590: 				{ Name = "discordInfo", Type = "DiscordChatInfo", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1591: 			},
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1592: 		},
```

Exists on master: `src/event/valid_events_a.rs:340`.

Required change / limit: Add gated, empty host chat input queue and rooted Rust delivery. Exempt this name from lockdown-caused secrecy only; retain independent host source secrets and NeverSecret fields. Four-value primitive projection only; complete 18-field/DiscordChatInfo and all native ingress paths remain unproved.

### `events-CHAT_MSG_LOOT-155`

Source L155:
> CHAT_MSG_LOOT - SecretInChatMessagingLockdown

Cached declaration (quoted):
```text
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1786: 		{
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1787: 			Name = "ChatMsgLoot",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1788: 			Type = "Event",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1789: 			LiteralName = "CHAT_MSG_LOOT",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1790: 			SynchronousEvent = true,
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1791: 			Payload =
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1792: 			{
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1793: 				{ Name = "text", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1794: 				{ Name = "playerName", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1795: 				{ Name = "languageName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1796: 				{ Name = "channelName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1797: 				{ Name = "playerName2", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1798: 				{ Name = "specialFlags", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1799: 				{ Name = "zoneChannelID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1800: 				{ Name = "channelIndex", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1801: 				{ Name = "channelBaseName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1802: 				{ Name = "languageID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1803: 				{ Name = "lineID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1804: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1805: 				{ Name = "bnSenderID", Type = "number", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1806: 				{ Name = "isMobile", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1807: 				{ Name = "isSubtitle", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1808: 				{ Name = "hideSenderInLetterbox", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1809: 				{ Name = "suppressRaidIcons", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1810: 				{ Name = "discordInfo", Type = "DiscordChatInfo", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1811: 			},
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1812: 		},
```

Exists on master: `src/event/valid_events_a.rs:347`.

Required change / limit: Add gated, empty host chat input queue and rooted Rust delivery. Exempt this name from lockdown-caused secrecy only; retain independent host source secrets and NeverSecret fields. Four-value primitive projection only; complete 18-field/DiscordChatInfo and all native ingress paths remain unproved.

### `events-CHAT_MSG_MONEY-156`

Source L156:
> CHAT_MSG_MONEY - SecretInChatMessagingLockdown

Cached declaration (quoted):
```text
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1813: 		{
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1814: 			Name = "ChatMsgMoney",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1815: 			Type = "Event",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1816: 			LiteralName = "CHAT_MSG_MONEY",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1817: 			SynchronousEvent = true,
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1818: 			Payload =
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1819: 			{
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1820: 				{ Name = "text", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1821: 				{ Name = "playerName", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1822: 				{ Name = "languageName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1823: 				{ Name = "channelName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1824: 				{ Name = "playerName2", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1825: 				{ Name = "specialFlags", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1826: 				{ Name = "zoneChannelID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1827: 				{ Name = "channelIndex", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1828: 				{ Name = "channelBaseName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1829: 				{ Name = "languageID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1830: 				{ Name = "lineID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1831: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1832: 				{ Name = "bnSenderID", Type = "number", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1833: 				{ Name = "isMobile", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1834: 				{ Name = "isSubtitle", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1835: 				{ Name = "hideSenderInLetterbox", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1836: 				{ Name = "suppressRaidIcons", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1837: 				{ Name = "discordInfo", Type = "DiscordChatInfo", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1838: 			},
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1839: 		},
```

Exists on master: `src/event/valid_events_a.rs:348`.

Required change / limit: Add gated, empty host chat input queue and rooted Rust delivery. Exempt this name from lockdown-caused secrecy only; retain independent host source secrets and NeverSecret fields. Four-value primitive projection only; complete 18-field/DiscordChatInfo and all native ingress paths remain unproved.

### `events-CHAT_MSG_RESTRICTED-157`

Source L157:
> CHAT_MSG_RESTRICTED - SecretInChatMessagingLockdown

Cached declaration (quoted):
```text
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2341: 		{
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2342: 			Name = "ChatMsgRestricted",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2343: 			Type = "Event",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2344: 			LiteralName = "CHAT_MSG_RESTRICTED",
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2345: 			SynchronousEvent = true,
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2346: 			Payload =
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2347: 			{
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2348: 				{ Name = "text", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2349: 				{ Name = "playerName", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2350: 				{ Name = "languageName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2351: 				{ Name = "channelName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2352: 				{ Name = "playerName2", Type = "cstring", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2353: 				{ Name = "specialFlags", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2354: 				{ Name = "zoneChannelID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2355: 				{ Name = "channelIndex", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2356: 				{ Name = "channelBaseName", Type = "cstring", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2357: 				{ Name = "languageID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2358: 				{ Name = "lineID", Type = "number", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2359: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2360: 				{ Name = "bnSenderID", Type = "number", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2361: 				{ Name = "isMobile", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2362: 				{ Name = "isSubtitle", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2363: 				{ Name = "hideSenderInLetterbox", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2364: 				{ Name = "suppressRaidIcons", Type = "bool", Nilable = false, NeverSecret = true },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2365: 				{ Name = "discordInfo", Type = "DiscordChatInfo", Nilable = false },
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2366: 			},
Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2367: 		},
```

Exists on master: `src/event/valid_events_a.rs:367`.

Required change / limit: Add gated, empty host chat input queue and rooted Rust delivery. Exempt this name from lockdown-caused secrecy only; retain independent host source secrets and NeverSecret fields. Four-value primitive projection only; complete 18-field/DiscordChatInfo and all native ingress paths remain unproved.

### `prose-undated-012`

Source L12:
> - Profiling APIs available to addons again: GetEventCPUUsage, GetFunctionCPUUsage, GetScriptCPUUsage.

Cached declaration (quoted):
```text
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:39: 		{
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:40: 			Name = "GetEventCPUUsage",
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:41: 			Type = "Function",
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:42: 
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:43: 			Returns =
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:44: 			{
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:45: 				{ Name = "call_time", Type = "number", Nilable = false },
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:46: 				{ Name = "call_count", Type = "number", Nilable = false },
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:47: 			},
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:48: 		},
```

Exists on master: `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:50`.

Required change / limit: Replace 12.0.7 constant shim with independent explicit host snapshots read live by Rust. Follow cached no-argument totals and declared arity; INFERRED units/attribution/NeverSecret extras. No keyed-function/frame/reset/native-timing proof.

### `global api-GetEventCPUUsage-052`

Source L52:
> GetEventCPUUsage

Cached declaration (quoted):
```text
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:39: 		{
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:40: 			Name = "GetEventCPUUsage",
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:41: 			Type = "Function",
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:42: 
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:43: 			Returns =
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:44: 			{
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:45: 				{ Name = "call_time", Type = "number", Nilable = false },
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:46: 				{ Name = "call_count", Type = "number", Nilable = false },
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:47: 			},
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:48: 		},
```

Exists on master: `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:49`; `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:50`.

Required change / limit: Replace 12.0.7 constant shim with independent explicit host snapshots read live by Rust. Follow cached no-argument totals and declared arity; INFERRED units/attribution/NeverSecret extras. No keyed-function/frame/reset/native-timing proof.

### `global api-GetFunctionCPUUsage-053`

Source L53:
> GetFunctionCPUUsage

Cached declaration (quoted):
```text
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:66: 		{
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:67: 			Name = "GetFunctionCPUUsage",
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:68: 			Type = "Function",
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:69: 
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:70: 			Returns =
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:71: 			{
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:72: 				{ Name = "call_time", Type = "number", Nilable = false },
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:73: 				{ Name = "call_count", Type = "number", Nilable = false },
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:74: 			},
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:75: 		},
```

Exists on master: `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:55`; `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:56`.

Required change / limit: Replace 12.0.7 constant shim with independent explicit host snapshots read live by Rust. Follow cached no-argument totals and declared arity; INFERRED units/attribution/NeverSecret extras. No keyed-function/frame/reset/native-timing proof.

### `global api-GetScriptCPUUsage-054`

Source L54:
> GetScriptCPUUsage

Cached declaration (quoted):
```text
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:76: 		{
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:77: 			Name = "GetScriptCPUUsage",
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:78: 			Type = "Function",
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:79: 
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:80: 			Returns =
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:81: 			{
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:82: 				{ Name = "result", Type = "number", Nilable = false },
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:83: 			},
Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:84: 		},
```

Exists on master: `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:61`; `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:62`.

Required change / limit: Replace 12.0.7 constant shim with independent explicit host snapshots read live by Rust. Follow cached no-argument totals and declared arity; INFERRED units/attribution/NeverSecret extras. No keyed-function/frame/reset/native-timing proof.

### `events-URL_TEXTURE_REQUEST_RESULT-146`

Source L146:
> URL_TEXTURE_REQUEST_RESULT

Cached declaration (quoted):
```text
Blizzard_APIDocumentationGenerated/TextureUtilsDocumentation.lua:161: 		{
Blizzard_APIDocumentationGenerated/TextureUtilsDocumentation.lua:162: 			Name = "UrlTextureRequestResult",
Blizzard_APIDocumentationGenerated/TextureUtilsDocumentation.lua:163: 			Type = "Event",
Blizzard_APIDocumentationGenerated/TextureUtilsDocumentation.lua:164: 			LiteralName = "URL_TEXTURE_REQUEST_RESULT",
Blizzard_APIDocumentationGenerated/TextureUtilsDocumentation.lua:165: 			SynchronousEvent = true,
Blizzard_APIDocumentationGenerated/TextureUtilsDocumentation.lua:166: 			Payload =
Blizzard_APIDocumentationGenerated/TextureUtilsDocumentation.lua:167: 			{
Blizzard_APIDocumentationGenerated/TextureUtilsDocumentation.lua:168: 				{ Name = "texture", Type = "SimpleTexture", Nilable = false },
Blizzard_APIDocumentationGenerated/TextureUtilsDocumentation.lua:169: 				{ Name = "result", Type = "UrlTextureResult", Nilable = false },
Blizzard_APIDocumentationGenerated/TextureUtilsDocumentation.lua:170: 			},
Blizzard_APIDocumentationGenerated/TextureUtilsDocumentation.lua:171: 		},
```

Exists on master: `src/event/valid_events_c.rs:475`.

Required change / limit: Add empty host URL notification queue and rooted synchronous event carrying actual texture identity and cached enum. Partial notification boundary only; C_Texture.SetURLTexture request producer/restrictions remain blocked.

### `prose-undated-008`

Source L8:
> - Unit identity: APIs restricting unit token types (UnitGUID, UnitAura, health/power APIs when called with PvP-restricted tokens) no longer raise Lua errors for unsupported tokens; return nil/default.

Cached declaration (quoted):
```text
Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:1212: 		{
Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:1213: 			Name = "UnitGUID",
Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:1214: 			Type = "Function",
Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:1215: 			SecretWhenUnitIdentityRestricted = true,
Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:1216: 			SecretArguments = "AllowedWhenUntainted",
Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:1217: 
Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:1218: 			Arguments =
Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:1219: 			{
Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:1220: 				{ Name = "unit", Type = "UnitTokenPvPRestrictedForAddOns", Nilable = false },
Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:1221: 			},
Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:1222: 
Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:1223: 			Returns =
Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:1224: 			{
Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:1225: 				{ Name = "result", Type = "WOWGUID", Nilable = true },
Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:1226: 			},
Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:1227: 		},
```

Exists on master: `src/lua_api/globals/unit_misc.rs:144`.

Required change / limit: Add empty host-declared unsupported-token set and core-six GUID/vitals/legacy UnitAura defaults, authenticating all arguments/extras before validation. Exact token set and legacy aura policy INFERRED; remainder of broad prose remains blocked pending entry-point/epoch inventory.

### `prose-undated-020`

Source L20:
> - Fixed secret value errors in SetFrameStrata.

Cached declaration (quoted):
```text
Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:1291: 		{
Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:1292: 			Name = "SetFrameStrata",
Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:1293: 			Type = "Function",
Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:1294: 			IsProtectedFunction = true,
Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:1295: 			SecretArguments = "NotAllowed",
Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:1296: 
Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:1297: 			Arguments =
Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:1298: 			{
Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:1299: 				{ Name = "strata", Type = "FrameStrata", Nilable = false },
Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:1300: 			},
Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:1301: 		},
```

Exists on master: `src/lua_api/frame/methods/core_state/strata_level.rs:13`.

Required change / limit: BLOCKED; no edit. Cached SecretArguments="NotAllowed" conflicts with guessed AllowedWhenUntainted/Always. Obtain historical/native secret matrix rather than turning the bugfix into a new permission.

### Supplemental B43 declarations

`UnitDocumentation.lua`: UnitGUID L1213–1226 (AllowedWhenUntainted; nilable WOWGUID), UnitHealth L1452–1466 (AllowedWhenUntainted; SecretReturns), UnitHealthMax L1398–1411, UnitPower L2643–2658, UnitPowerMax L2712–2727. No generated declaration for legacy global UnitAura found. Legacy aura permissions are INFERRED, not inherited from C_UnitAuras by name.

Current actual core providers: `src/lua_api/globals/unit_misc.rs:144–171`, `src/lua_api/globals/utility_system_spell/spell_api.rs:27–45,137–148,175–181,221–227`, `src/lua_api/globals/auras.rs:601–610,722–733`. Unknown GUID already returns one nil; vitals already yield zero absent snapshot; legacy aura already yields one nil when absent. Existing target aura fixture does not depend on target presence: core-six host override is not a global UnitExists rewrite.

### Consumer constraints

`Blizzard_CatalogShop/Blizzard_CatalogShop_Elements.lua:482–491`: `local texture, result = ...;` followed by `if texture == self.PMTImageForNoModel then` and `if result == Enum.UrlTextureResult.Requested then self.Spinner:Show(); else self.Spinner:Hide(); end`. Empty notification queue causes no new event and does not claim loading success. A fabricated nil receiver, Cancelled enum or terminal-only request model would break this consumer. `TextureUtilsDocumentation.lua:197–200` declares Found=1, NotFound=2, Requested=3, NotAllowed=4, no Cancelled.

No cached non-documentation Lua use of GetEventCPUUsage/GetFunctionCPUUsage/GetScriptCPUUsage was found by a full cache scan. No exact zero-argument call to the core six unit globals was found in cached Lua; method names such as self:UpdateUnitHealth() are not those globals. Empty unsupported set preserves modeled player/target defaults; empty chat/URL queues dispatch nothing. Actual consumer execution is forbidden and remains unproved.

## Anchored edits against master — apply each against original bytes

Every OLD block below matches exactly once at `7702befe8`; same-file blocks are non-overlapping. Empty NEW blocks mean deletion. New files are separately listed, not falsely assigned an OLD block in an absent file. Withhold ALL `producer` edits/files for inputs-only RED; copy `state` and `test-support` files and edits first. Test-only traits supply missing host entry signatures during RED; they are not runtime fallback behavior. Apply all producer changes atomically for GREEN.

### E01 — `state` — `src/lua_api/mod.rs:11`

OLD (exact):
```text
pub mod chat_init;
```

NEW (exact):
```text
pub mod chat_init;
#[cfg(feature = "retail-12-0-7")]
pub mod host_chat_inputs;
#[cfg(feature = "retail-12-0-7")]
pub mod performance_inputs;
```

### E02 — `producer` — `src/lua_api/mod.rs:15`

OLD (exact):
```text
mod env_events;
```

NEW (exact):
```text
mod env_events;
#[cfg(feature = "retail-12-0-7")]
mod host_chat_events;
#[cfg(feature = "retail-12-0-7")]
mod host_url_texture_events;
#[cfg(feature = "retail-12-0-7")]
mod unsupported_unit_inputs;
```

### E03 — `state` — `src/c_api/mod.rs:21`

OLD (exact):
```text
pub mod bag_info;
```

NEW (exact):
```text
#[cfg(feature = "retail-12-0-7")]
pub mod url_texture_inputs;
pub mod bag_info;
```

### E04 — `state` — `src/lua_api/state/sim_state.rs:46`

OLD (exact):
```text
    pub chat_edit_open_state: Option<ChatEditOpenState>,
```

NEW (exact):
```text
    pub chat_edit_open_state: Option<ChatEditOpenState>,
    /// INFERRED explicit host ingress; empty per environment.
    #[cfg(feature = "retail-12-0-7")]
    pub host_chat_inputs: crate::lua_api::host_chat_inputs::HostChatInputs,
    #[cfg(feature = "retail-12-0-7")]
    pub performance_inputs: crate::lua_api::performance_inputs::PerformanceInputs,
    #[cfg(feature = "retail-12-0-7")]
    pub url_texture_inputs: crate::c_api::url_texture_inputs::UrlTextureInputs,
    /// INFERRED unsupported-token set; empty default preserves modeled units.
    #[cfg(feature = "retail-12-0-7")]
    pub unsupported_unit_tokens: HashSet<String>,
```

### E05 — `state` — `src/lua_api/state.rs:60`

OLD (exact):
```text
            chat_edit_open_state: None,
```

NEW (exact):
```text
            chat_edit_open_state: None,
            #[cfg(feature = "retail-12-0-7")]
            host_chat_inputs: crate::lua_api::host_chat_inputs::HostChatInputs::default(),
            #[cfg(feature = "retail-12-0-7")]
            performance_inputs: crate::lua_api::performance_inputs::PerformanceInputs::default(),
            #[cfg(feature = "retail-12-0-7")]
            url_texture_inputs: crate::c_api::url_texture_inputs::UrlTextureInputs::default(),
            #[cfg(feature = "retail-12-0-7")]
            unsupported_unit_tokens: HashSet::new(),
```

### E06 — `producer` — `src/lua_api/globals/real/mod.rs:48`

OLD (exact):
```text
pub mod net_stats;
```

NEW (exact):
```text
pub mod net_stats;
#[cfg(feature = "retail-12-0-7")]
pub(crate) mod performance_inputs;
```

### E07 — `producer` — `src/lua_api/env_init/mod.rs:58`

OLD (exact):
```text
    crate::lua_api::workarounds::apply_temporary_bootstrap(lua)?;
```

NEW (exact):
```text
    #[cfg(feature = "retail-12-0-7")]
    super::globals::real::performance_inputs::register(lua)?;
    crate::lua_api::workarounds::apply_temporary_bootstrap(lua)?;
```

### E08 — `producer` — `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:49`

OLD (exact):
```text
if GetEventCPUUsage == nil then
  function GetEventCPUUsage(_event)
    return 0
  end
end

if GetFunctionCPUUsage == nil then
  function GetFunctionCPUUsage(_fn, _includeSubroutines)
    return 0, 0
  end
end

if GetScriptCPUUsage == nil then
  function GetScriptCPUUsage(_frame, _script, _includeChildren)
    return 0, 0
  end
end


```

NEW (exact):
```text

```

### E09 — `producer` — `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:88`

OLD (exact):
```text
pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PERFORMANCE_METRIC_DEFAULTS_LUA)?;
    Ok(())
}
```

NEW (exact):
```text
#[cfg(not(feature = "retail-12-0-7"))]
const LEGACY_PROFILING_DEFAULTS_LUA: &str = r#"
if GetEventCPUUsage == nil then
  function GetEventCPUUsage(_event)
    return 0
  end
end

if GetFunctionCPUUsage == nil then
  function GetFunctionCPUUsage(_fn, _includeSubroutines)
    return 0, 0
  end
end

if GetScriptCPUUsage == nil then
  function GetScriptCPUUsage(_frame, _script, _includeChildren)
    return 0, 0
  end
end

"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PERFORMANCE_METRIC_DEFAULTS_LUA)?;
    #[cfg(not(feature = "retail-12-0-7"))]
    lua.exec(LEGACY_PROFILING_DEFAULTS_LUA)?;
    Ok(())
}
```

### E10 — `test-support` — `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:99`

OLD (exact):
```text
        let env = WowLuaEnv::new().expect("lua env should initialize");
```

NEW (exact):
```text
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let script_arity = if cfg!(feature = "retail-12-0-7") { 1 } else { 2 };
        env.exec(&format!("ExpectedScriptArity = {script_arity}"))
            .expect("set profile-specific script metric arity");
```

### E11 — `test-support` — `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:116`

OLD (exact):
```text
                if scriptUsage ~= 0 or scriptChildren ~= 0 then return "script_cpu" end
```

NEW (exact):
```text
                if scriptUsage ~= 0 or select('#', GetScriptCPUUsage(UIParent, "OnShow", true)) ~= ExpectedScriptArity then return "script_cpu" end
```

### E12 — `producer` — `src/lua_api/globals/unit_misc.rs:144`

OLD (exact):
```text
fn unit_guid(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
```

NEW (exact):
```text
fn unit_guid(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "retail-12-0-7")]
    let unit = crate::lua_api::unsupported_unit_inputs::authenticate_unit_arguments(state)?;
    #[cfg(not(feature = "retail-12-0-7"))]
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    #[cfg(feature = "retail-12-0-7")]
    if crate::lua_api::unsupported_unit_inputs::is_unsupported(state, &unit)? {
        state.push(Val::Nil);
        return Ok(1);
    }
```

### E13 — `producer` — `src/lua_api/globals/utility_system_spell/spell_api.rs:139`

OLD (exact):
```text
fn unit_health(state: &mut LuaState) -> LuaResult<u32> {
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| "player".to_string());
```

NEW (exact):
```text
fn unit_health(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "retail-12-0-7")]
    let unit = crate::lua_api::unsupported_unit_inputs::authenticate_unit_arguments(state)?;
    #[cfg(not(feature = "retail-12-0-7"))]
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| "player".to_string());
    #[cfg(feature = "retail-12-0-7")]
    if crate::lua_api::unsupported_unit_inputs::is_unsupported(state, &unit)? {
        state.push(Val::Num(0.0));
        return Ok(1);
    }
```

### E14 — `producer` — `src/lua_api/globals/utility_system_spell/spell_api.rs:146`

OLD (exact):
```text
fn unit_health_max(state: &mut LuaState) -> LuaResult<u32> {
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| "player".to_string());
```

NEW (exact):
```text
fn unit_health_max(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "retail-12-0-7")]
    let unit = crate::lua_api::unsupported_unit_inputs::authenticate_unit_arguments(state)?;
    #[cfg(not(feature = "retail-12-0-7"))]
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| "player".to_string());
    #[cfg(feature = "retail-12-0-7")]
    if crate::lua_api::unsupported_unit_inputs::is_unsupported(state, &unit)? {
        state.push(Val::Num(0.0));
        return Ok(1);
    }
```

### E15 — `producer` — `src/lua_api/globals/utility_system_spell/spell_api.rs:177`

OLD (exact):
```text
fn unit_power(state: &mut LuaState) -> LuaResult<u32> {
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| "player".to_string());
```

NEW (exact):
```text
fn unit_power(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "retail-12-0-7")]
    let unit = crate::lua_api::unsupported_unit_inputs::authenticate_unit_arguments(state)?;
    #[cfg(not(feature = "retail-12-0-7"))]
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| "player".to_string());
    #[cfg(feature = "retail-12-0-7")]
    if crate::lua_api::unsupported_unit_inputs::is_unsupported(state, &unit)? {
        state.push(Val::Num(0.0));
        return Ok(1);
    }
```

### E16 — `producer` — `src/lua_api/globals/utility_system_spell/spell_api.rs:221`

OLD (exact):
```text
fn unit_power_max(state: &mut LuaState) -> LuaResult<u32> {
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| "player".to_string());
```

NEW (exact):
```text
fn unit_power_max(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "retail-12-0-7")]
    let unit = crate::lua_api::unsupported_unit_inputs::authenticate_unit_arguments(state)?;
    #[cfg(not(feature = "retail-12-0-7"))]
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| "player".to_string());
    #[cfg(feature = "retail-12-0-7")]
    if crate::lua_api::unsupported_unit_inputs::is_unsupported(state, &unit)? {
        state.push(Val::Num(0.0));
        return Ok(1);
    }
```

### E17 — `producer` — `src/lua_api/globals/auras.rs:601`

OLD (exact):
```text
fn unit_aura(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
```

NEW (exact):
```text
fn unit_aura(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED AllowedWhenUntainted for this undeclared legacy global.
    #[cfg(feature = "retail-12-0-7")]
    let unit = crate::lua_api::unsupported_unit_inputs::authenticate_unit_arguments(state)?;
    #[cfg(not(feature = "retail-12-0-7"))]
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
```

### E18 — `producer` — `src/lua_api/globals/auras.rs:605`

OLD (exact):
```text
    let aura = collect_visible_unit_auras(state, &unit, filter_from_str(&filter_str))
        .into_iter()
        .nth(index.saturating_sub(1) as usize);
```

NEW (exact):
```text
    #[cfg(feature = "retail-12-0-7")]
    if crate::lua_api::unsupported_unit_inputs::is_unsupported(state, &unit)? {
        state.push(Val::Nil);
        return Ok(1);
    }
    let aura = collect_visible_unit_auras(state, &unit, filter_from_str(&filter_str))
        .into_iter()
        .nth(index.saturating_sub(1) as usize);
```

## Full new files staged (copy verbatim; paths must be absent on master)

- `test-support` `docs/specs/patch-12-0-7-host-events-and-defaults.md` ← `staging/p1207-b39-b44/docs/specs/patch-12-0-7-host-events-and-defaults.md`
- `state` `src/c_api/url_texture_inputs.rs` ← `staging/p1207-b39-b44/src/c_api/url_texture_inputs.rs`
- `producer` `src/lua_api/globals/real/performance_inputs.rs` ← `staging/p1207-b39-b44/src/lua_api/globals/real/performance_inputs.rs`
- `producer` `src/lua_api/host_chat_events.rs` ← `staging/p1207-b39-b44/src/lua_api/host_chat_events.rs`
- `state` `src/lua_api/host_chat_inputs.rs` ← `staging/p1207-b39-b44/src/lua_api/host_chat_inputs.rs`
- `producer` `src/lua_api/host_url_texture_events.rs` ← `staging/p1207-b39-b44/src/lua_api/host_url_texture_events.rs`
- `state` `src/lua_api/performance_inputs.rs` ← `staging/p1207-b39-b44/src/lua_api/performance_inputs.rs`
- `producer` `src/lua_api/unsupported_unit_inputs.rs` ← `staging/p1207-b39-b44/src/lua_api/unsupported_unit_inputs.rs`
- `test-support` `tests/patch_1207_b39_b44.rs` ← `staging/p1207-b39-b44/tests/patch_1207_b39_b44.rs`

Count: 18 anchored edits; 8 staged Rust files plus one spec; 17 behavioral tests in one new top-level integration module.

## Existing tests affected / integration cautions

1. `src/lua_api/workarounds/temporary/performance_metric_defaults.rs::tests::installs_performance_metric_defaults` changes its script query arity expectation via E10–E11. Existing GetFunctionCPUUsage second zero now means call count, not “subroutines”; default value remains zero.
2. `src/loader/tests/wow_api_globals/startup_globals.rs:295–297` zero-first-result assertions remain valid but do not prove new arity/state. No source-substring test or occurrence status earns coverage.
3. `tests/instanced_identity.rs`, `tests/retail_unit_queries.rs`, `tests/unit_aura_filter_query.rs` remain existing controls. Empty unsupported set does not alter their configured snapshot data; malformed-unit and authentic-secret selector acceptance under 12.0.7 deliberately change. Review their expectations during authorized integration; no blanket fixture rewrites proposed.
4. `tests/xml_frame_strata.rs`, `tests/security_api.rs`, `tests/protected_frame_enforcement.rs` are unchanged. No B44 acceptance test is staged because its expected native secret permission is not authenticated.
5. All top-level `tests/*.rs` are modules in ONE integration binary (`build.rs:58–98`); the new module is automatically discovered. Target `--test integration patch_1207_b39_b44::` during a later authorized RED/GREEN run, not `--test patch_1207_b39_b44`. No command here was executed.

## Exclusions / precise blockers

- B44: No strata producer, security-policy change, wrapped strata constant or workaround. Source L20 only reports a bugfix; later cache NotAllowed cannot prove AllowedWhenUntainted/Always. Unblock with historical 12.0.7 declaration plus native authentic-secret valid/invalid selector outcomes for clean and tainted callers, protected combat denial, and child propagation. Keep protected permission checks unchanged.
- B42: Queued host notifications are authored, NOT a SetURLTexture implementation. Cached declaration marks HasRestrictions but does not define conditions; consumer explicitly needs Requested and real receiver identity. No URL fetch, cancellation, retry policy, texture mutation or success pixel claim. Need native/historical request lifecycle and restrictions before covering the whole row.
- B43: Core-six explicit input semantics only. No invented automatic PvP-token set, no UnitExists override, no rewrite of all C_UnitAuras or percent/missing/other health/power queries. Broad prose cannot close until that entry-point and historical-default inventory is established. Legacy UnitAura AllowedWhenUntainted is INFERRED; existing secret-return annotations are outside this authored input/default boundary.
- B39–B40: Primitive projection only; full 18-field payload (including DiscordChatInfo), native ingress and other dispatch entry paths remain open. Independent incoming-source secrecy is explicit host classification wrapped into authentic VM values, not proof of a network source. NeverSecret positions come from cache, not the word “restricted” in an event name.
- B41: No automatic CPU measurement, reset, function/frame-keyed bookkeeping or historical native attribution. Later-cache no-argument signature is intentionally chosen over inconsistent temporary Lua defaults, but historical shape and INFERRED extras/NeverSecret need independent authentication. Empty snapshots have no cached profiling-getter consumer break found by inspection.

## Proof ledger — author-only

| Evidence | Scope | Result / limit |
|---|---|---|
| Read-only master rev-parse | master | `7702befe8a567c225d9e8680594186e9689734f4` |
| Source/triage/cache reads | 16 row IDs and cited declarations/consumers | Static evidence only; later cache may postdate epoch |
| Python exact-anchor validation + `git show 7702befe8:<path>` | E01–E18, original master blobs | Every OLD matches once; same-file spans do not overlap; no repo writes |
| Standalone rustfmt, edition 2024 | staged new Rust files, URL follow-up formatted separately | Exit 0; syntax/format evidence, NOT compilation or behavior |
| Manual readability audit | new Rust files and proposed snippets | Small helpers, no warning suppression, bounded branches; no semantic compiler proof |
| Static fixture inventory | new integration module | 17 uniquely named behavioral tests; no u32 eval result, EventQueue unsupported methods, raw-string delimiter collision or tainted secretwrap call |
| Read-only git diff / cached diff | canonical tracked worktree/index | Both empty at inspection; no git mutation performed |
| Cargo/tests/builds/simulator | all | NOT RUN — explicitly forbidden |

## Final author status

Author deliverables persisted: 16 row audits; 18 anchored existing-file edits; 8 full new staged Rust files and 1 spec; 17 tests. Proposed behavioral scope is bounded and partial, not page-row completion. B44 blocked; B42 request and B43 remaining-entry-point parts blocked; historical/native and complete chat/profiling proof gaps remain explicit. No coverage ledger changes, integration, deployment or passing-test claims.
