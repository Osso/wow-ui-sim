# Historical retail Patch 3.3.3 API audit

Frozen page 261776, revision 2531935, timestamp 2010-05-13T22:15:52Z. Local manifest identity, exact response content, SHA-256 and byte length validated before copying. This is historical retail 2010, not Wrath Classic 3.4.x (2022).

## Source format

Existing heading/bullet parsers produce no inventory or misclassify all identities as changes. `--historical-api-headings` is opt-in and retains 36 occurrences: 21 new globals, nine new events, three changed globals, one removed global and two removed Texture methods. Original signature/prose lines remain annotations. Default extraction can retain the full page; no extractor change. Two concrete output fixtures first fail on the missing flag; GREEN is pending this implementation commit.

## Sources

- [Pinned source](../../../data/patch-api/sources/3.3.3-api-changes.wikitext).
- [Source pin](../../../data/patch-api/evidence/3.3.3-session-2026-10-09/source-pin.json).
- [Spec](../../specs/patch-3-3-3-publication-sweep.md).

## See Also

- [[patch-4-1-0-api-audit]] — first already-integrated actual retail successor; 3.3.5 and 4.0.1 are coordinator-owned placeholders.
