# Historical retail Patch 3.3.3 API audit

Frozen page 261776, revision 2531935, timestamp 2010-05-13T22:15:52Z. Local manifest identity, exact response content, SHA-256 and byte length validated before copying. This is historical retail 2010, not Wrath Classic 3.4.x (2022).

## Source format

Existing heading/bullet parsers produce no inventory or misclassify all identities as changes. `--historical-api-headings` is opt-in and retains 36 occurrences: 21 new globals, nine new events, three changed globals, one removed global and two removed Texture methods. Original signature/prose lines remain annotations. Default extraction can retain the full page; no extractor change. Two concrete output fixtures first fail on the missing flag, then pass 2/2 at `ac338fe2d`. Default output bytes restored identically. Own retail discovery test includes actual retail 4.1.0 onward with separate ordered 3.3.5 / 4.0.1 placeholders; known gaps are initially empty for discovery, not a zero-gap claim.

## Removal boundary

Qualified and bare whole-word scans across all cached retail Lua find zero references to the three source-removed identities. Whole `src/` and `tests/` scans also find no implementation or consumer. No retirement edit is needed or performed; runtime absence is not inferred from these scans. No Classic or deprecation wrapper changes. Scan receipts are under [own evidence](../../../data/patch-api/evidence/3.3.3-session-2026-10-09/).

## Capability matrix — development discovery

| Source contract | Accounted | Missing / proof limit |
|---|---|---|
| Named inventory | 36 occurrences; 25 current publication/absence matches | 11 exact gaps; discovery RED at `27cb4e078` |
| Full raw/rendered extract | 41 nonempty rows: navigation, four headers, 36 bullets | Three changed prose contracts remain unproven; 33 signature cross-references get no duplicate credit |
| Literal signatures | 36 separately retained records | Historical/native input/output and nine event producers unproven |
| Source removals | Three current absence observations | No retirement edit or native timing proof |
| Existing pet scalar | State-backed legacy global in `globals/real/pet_stats.rs` | Own full cached-UI arity/value test pending; no new model |

[Ledger](../../../data/patch-api/sources/3.3.3-page-coverage.json): 113 IDs; 25 bounded publication, 50 pending, 38 metadata. Nine missing globals and two retained callable/no-op identities against later-removal expectations form the 11 publication gaps. `GetQuestWatchIndex` and `SortQuestWatches` are not source removals in 3.3.3, so no retirement is authorized. `UninviteUnit` is a cached Blizzard deprecation wrapper and preserved. All gaps name the missing contract/state/producer; no invented sorter, low-level-raid preference lifecycle or reward fixture.

Own runtime compiled pinned Git dependencies and executed successfully; absent reference clones are not a runtime blocker. Only discovery assertion fails as expected. Six inherited iced manifest warnings remain untouched. No broad gate or host configuration change.

## Sources

- [Pinned source](../../../data/patch-api/sources/3.3.3-api-changes.wikitext).
- [Source pin](../../../data/patch-api/evidence/3.3.3-session-2026-10-09/source-pin.json).
- [Spec](../../specs/patch-3-3-3-publication-sweep.md).

## See Also

- [[patch-4-1-0-api-audit]] — first already-integrated actual retail successor; 3.3.5 and 4.0.1 are coordinator-owned placeholders.
