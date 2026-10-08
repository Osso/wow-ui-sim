# Patch 9.0.2 API page audit

Page 66933, revision 659986 (2021-05-06T09:01:47Z), retrieved 2026-10-08. Current retail carries 12.1.0. Original 9.0.2 page exists; no 9.0.1 substitution.

## Bounded retirement

Nine removed namespace members have zero qualified whole-word and bare-name cached retail Lua consumers and zero whole src/tests callers. Separate RETIRED_9_0_2_MEMBERS uses existing retail-12-0-0 module gate; classic profiles and Blizzard deprecation wrappers remain untouched. Initial sweep 42 OK / 35 gaps; nine absence closures predict 51 OK / 26 exact gaps. Bare behavioral RED fails on fabricated GetRenownMilestones after correcting an invalid prefork marker. Acceptance remains pending.

C_Soulbinds.MatchesCurrentSpecSet is retained conservatively: bare-name searches find C_SpecializationInfo.MatchesCurrentSpecSet in Blizzard_Soulbinds/Blizzard_SoulbindsConduitList.lua:135,396. No namespace-qualified old API consumer is claimed.

## Sources

- [Pinned response](../../../data/patch-api/evidence/9.0.2-session-2026-10-08/p902-fetch.json).
- [Removal scans](../../../data/patch-api/evidence/9.0.2-session-2026-10-08/p902-removal-consumers.json).
- [Sweep contract](../../specs/patch-9-0-2-publication-sweep.md).

## See Also

- [[patch-9-1-0-api-audit]] — publication-only proof boundaries.
