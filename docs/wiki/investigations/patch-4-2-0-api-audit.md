# Patch 4.2.0 API audit

Pinned historical retail inventory, page 315583/revision 3045158 (2021-08-22T03:01:41Z), retrieved 2026-10-09. Development accounting only; coordinator owns final gates.

## Source boundary

65 global occurrences: 63 added, two removed. Both literal headers match parsed counts. One extracted navigation metadata row; zero behavior prose or signature rows. Existing generator with `--client-line retail` and default extractor suffice; no parser changes.

## Publication scope

Own prefork case probes every occurrence using current retail runtime and actual 5.0.1–12.1.0 retail successors. Two explicit pending placeholders: 4.3.0 and 4.3.4. Classic histories excluded. Exact observed gaps and possible pending supersessions will be recorded after targeted development testing. No runtime changes or retirements yet.

## Bounded model fix

`BNGetFriendIndex(accountID)` now reads the current position in `SimState.bnet_friends`, consistent with `C_BattleNet.GetFriendAccountInfo(index)` rather than stale `friend_index` fields. Retail-only; Classic and PTR registration unchanged. Cached `Blizzard_FriendsFrame/Mainline/FriendsFrame.lua:2545` passes a Battle.net account ID. Two local model tests failed on missing global before implementation. Unknown-ID nil is inferred; this page provides no signatures and no native parity is claimed. No shims/fallbacks added. Development GREEN pending.

## Sources

- [Pinned source](../../../data/patch-api/sources/4.2.0-api-changes.wikitext)
- [Source pin](../../../data/patch-api/evidence/4.2.0-session-2026-10-09/source-pin.json)
- [Coverage](../../../data/patch-api/sources/4.2.0-page-coverage.json)
- [Spec](../../specs/patch-4-2-0-publication-sweep.md)

## See Also

- [[patch-5-0-1-api-audit]] — first available later retail register.
