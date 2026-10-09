# Historical retail Patch 3.3.5 API audit

Frozen page 25049, revision 247986, timestamp 2010-07-10T15:47:20Z; supplied October 9, 2026. Manifest identity and literal response/wikitext SHA-256 matched before copying. Historical retail 2010 is separate from Wrath Classic 3.4.x (2022+).

## Source format

Existing section/bullet flags do not parse `New API functions`, `New FrameXML API`, `New Events`, `API Changes` and `Removed FrameXML API` colon inventories. New `--legacy-section-lists` is opt-in and preserves original line numbers, explicit parentheses and return prefixes. Full-page extraction uses unchanged default flags. No numeric header counts occur; five named section headers are retained separately in extraction.

124 inventory occurrences: 68 global additions, six FrameXML additions, 47 events, one changed `NotifyInspect`, two FrameXML removals. 130 nonempty full-extract rows and 70 explicit signature/return fragments (69 parenthetical, one return-only) remain independent accounting targets. Identity-only lines do not acquire signatures from linked pages.

## Sources

- [Frozen source](../../../data/patch-api/sources/3.3.5-api-changes.wikitext).
- [Literal source receipt](../../../data/patch-api/evidence/3.3.5-session-2026-10-09/source-response.json).
- [Spec](../../specs/patch-3-3-5-publication-sweep.md).

## See Also

- [[patch-4-1-0-api-audit]] — actual later retail successor.
- 4.0.1 source audit is queued separately; main adds its actual register at integration. No placeholder supersession credit.

## Proof boundary

Parser RED fails on the missing opt-in flag; GREEN fixture passes at 1658d0e36. Own retail publication discovery at f9b98435d observes 80 matches / 44 gaps across 124 rows. Exact known gaps are retained before GREEN; no missing-name shims added.

## Capability matrix

| Source scope | Accounted | Still unsupported | Proof |
|---|---|---|---|
| Publication inventory | 124 occurrences; 80 matches, 44 mismatches | 19 event, four FrameXML and 21 global gaps | Own RED results; GREEN/negative follow |
| Historical signatures | 70 literal fragments; missing parentheses remain missing | Historical input/coercion, output arity/value and native payload semantics | Source only; no linked-page expansion |
| Full-page prose/headers | 130 rows: 129 metadata/cross-references, one semantic gap; five named headers, no numeric counts | Server may throttle NotifyInspect and withhold availability events | Retained full extract; source-backed precise gap |
| Removals | UIFrameFlashSwitch / ToggleCombatLog already absent | No new runtime removal or non-retail parity claim | Full cached-retail/src/tests whole-word scans, no consumers |
| Cheap modeled closures | Zero | Legacy presence/toon identities, friend-of-friend/block/invite/selection/service lifecycles, authentication and saved account chat settings | Existing source/state examined; plausible constants or account IDs are not 2010 contracts |

[Ledger](../../../data/patch-api/sources/3.3.5-page-coverage.json): 324 IDs = 124 inventory + 130 full-extract + 70 signature/return. Dispositions: 32 publication-only, two absence-only, 46 superseded-publication, 44 publication-gap, 129 metadata-only, one semantic-gap, 70 signature-gap. Later-removal credit explains current absence, not historical behavior. Existing BNGetFriendInfo inert binding remains a later-removal mismatch; it is not a removal on this page, and no existing wrapper/default is deleted. Existing chat geometry remains temporary session-local Lua state, not account persistence.

[Consumer scans](../../../data/patch-api/evidence/3.3.5-session-2026-10-09/consumer-scans.json) retain complete source/test/cache hits; all identities are unqualified, so qualified/bare searches coincide. No vendor/cache or runtime writes. Actual retail 4.1.0+ successors only; main adds queued 4.0.1 later. Native 2010-client parity and current coordinator acceptance remain unproven.
