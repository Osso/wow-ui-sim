# Historical retail Patch 3.3.5 API audit

Frozen page 25049, revision 247986, timestamp 2010-07-10T15:47:20Z; supplied October 9, 2026. Manifest identity and literal response/wikitext SHA-256 matched before copying. Historical retail 2010 is separate from Wrath Classic 3.4.x (2022+).

## Source format

Existing section/bullet flags do not parse `New API functions`, `New FrameXML API`, `New Events`, `API Changes` and `Removed FrameXML API` colon inventories. New `--legacy-section-lists` is opt-in and preserves original line numbers, explicit parentheses and return prefixes. Full-page extraction uses unchanged default flags. No numeric header counts occur; five named section headers are retained separately in extraction.

124 inventory occurrences: 68 global additions, six FrameXML additions, 47 events, one changed `NotifyInspect`, two FrameXML removals. 130 nonempty full-extract rows and 69 explicit signature fragments remain independent accounting targets. Identity-only lines do not acquire signatures from linked pages.

## Sources

- [Frozen source](../../../data/patch-api/sources/3.3.5-api-changes.wikitext).
- [Literal source receipt](../../../data/patch-api/evidence/3.3.5-session-2026-10-09/source-response.json).
- [Spec](../../specs/patch-3-3-5-publication-sweep.md).

## See Also

- [[patch-4-1-0-api-audit]] — actual later retail successor.
- 4.0.1 source audit is queued separately; main adds its actual register at integration. No placeholder supersession credit.

## Proof boundary

Parser RED fails on the missing opt-in flag. Complete publication/semantic accounting and targeted receipts follow separately. No runtime change or native 2010-client parity claim.
