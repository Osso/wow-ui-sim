# Patch 9.1.5 API page audit

Page 219137, revision 5920444 (January 1, 2024, 01:31:49 UTC), retrieved October 7, 2026. Branch `p915-page` starts at master `c3c3c8036`, containing the 9.2.5 sweep. Current retail carries 12.1.0; historical reconstruction is excluded.

## Source accounting

169 inventory occurrences: 109 added, 60 removed; no changed inventory. Global API 70/46, widgets 8/0, events 8/9, CVars including commands 23/5. Every published header count matches. Register provenance records `--expand-shared-changes --capture-span-defaults --plain-command-labels`. Plain inline Commands recognition reuses p920-page's markup interpretation, behind a new opt-in flag so prior registers do not change. Twenty-eight master later registers supersede; a one-line 9.2.0 integration placeholder precedes the later-register list.

The unchanged extractor retains 18 nonblank occurrences. Publication/absence is not signature, DTO, security or runtime policy proof. Initial sweep is 111 OK / 58 gaps.

## Bounded retirements

Eleven removed namespace members selected after qualified and bare-name cached Lua scans:

- C_Commentator.SetBlacklistedAuras and SetBlacklistedCooldowns.
- C_ItemUpgrade.GetItemLevelIncrement; C_LFGList.GetActivityInfo and GetCategoryInfo.
- C_LFGuildInfo.GetRecruitingGuildTabardInfo; C_PlayerMentorship.GetMentorOptionalAchievementIDs.
- C_Soulbinds.GetConduitChargesCapacity, GetConduitCharges, GetTotalConduitChargesPendingInSoulbind and GetTotalConduitChargesPending.

GetActivityInfo substring hits are GetActivityInfoTable; exact word-boundary scans have zero matches. GetCategoryInfo bare hits belong to other namespaces/global achievement API, not C_LFGList. No actual retired endpoint caller appears in the whole src/tests scan; no caller migration is needed. Classic profiles do not load the retirement module. Blizzard deprecation wrappers and current successor APIs remain untouched.

Cached full-UI RED reproduces fabricated C_Commentator.SetBlacklistedAuras. GREEN and acceptance evidence pending.

## Sources

- [Pinned API response](../../../data/patch-api/evidence/9.1.5-session-2026-10-07/p915-fetch.json).
- [Register](../../../data/patch-api/sources/9.1.5-wikitext-register.json).
- [Qualified/bare scan](../../../data/patch-api/evidence/9.1.5-session-2026-10-07/p915-removal-consumers.json).
- [Exact-name scan](../../../data/patch-api/evidence/9.1.5-session-2026-10-07/p915-removal-exact-consumers.json).
- [Spec](../../specs/patch-9-1-5-publication-sweep.md).

## See Also

- [[patch-9-2-5-api-audit]] — accounting and later supersession.
- [[patch-9-2-7-api-audit]] — publication-only proof conventions.
