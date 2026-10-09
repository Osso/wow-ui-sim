# Historical retail Patch 3.3.3 API audit

Frozen page 261776, revision 2531935, timestamp 2010-05-13T22:15:52Z. Local manifest identity, exact response content, SHA-256 and byte length validated before copying. This is historical retail 2010, not Wrath Classic 3.4.x (2022).

## Source format

Existing heading/bullet parsers produce no inventory or misclassify all identities as changes. `--historical-api-headings` is opt-in and retains 36 occurrences: 21 new globals, nine new events, three changed globals, one removed global and two removed Texture methods. Original signature/prose lines remain annotations. Default extraction can retain the full page; no extractor change. Two concrete output fixtures first fail on the missing flag, then pass 2/2 at `ac338fe2d`. Default output bytes restored identically. Own retail discovery test includes actual retail 4.1.0 onward with separate ordered 3.3.5 / 4.0.1 placeholders; known gaps are initially empty for discovery, not a zero-gap claim.

## Removal boundary

Qualified and bare whole-word scans across all cached retail Lua find zero references to the three source-removed identities. Whole `src/` and `tests/` scans also find no implementation or consumer. No retirement edit is needed or performed; runtime absence is not inferred from these scans. No Classic or deprecation wrapper changes. Scan receipts are under [own evidence](../../../data/patch-api/evidence/3.3.3-session-2026-10-09/).

## Capability matrix — development discovery

| Source contract | Accounted | Missing / proof limit |
|---|---|---|
| Named inventory | 36 occurrences; 25 current publication/absence matches | 11 exact gaps; own reviewed GREEN and negative 11 → 12 at `ddfaa9d45` |
| Full raw/rendered extract | 41 nonempty rows: navigation, four headers, 36 bullets | Three changed prose contracts remain unproven; 33 signature cross-references get no duplicate credit |
| Literal signatures | 36 separately retained records; one bounded existing pet scalar read | 35 pending signature rows; historical/native input/output and all nine event producers unproven |
| Source removals | Three current absence observations | No retirement edit or native timing proof |
| Existing pet scalar | Full cached-retail UI test reads scalar 137.25 then 58.5 from `pet.spell_bonus_damage` | Existing `globals/real/pet_stats.rs` model; zero new models; no native/event producer credit |

[Ledger](../../../data/patch-api/sources/3.3.3-page-coverage.json): 113 IDs; 26 bounded (25 publication plus one existing scalar read), 49 pending, 38 metadata. Nine missing globals and two retained callable/no-op identities against later-removal expectations form the 11 publication gaps. `GetQuestWatchIndex` and `SortQuestWatches` are not source removals in 3.3.3, so no retirement is authorized. `UninviteUnit` is a cached Blizzard deprecation wrapper and preserved. All gaps name the missing contract/state/producer; no invented sorter, low-level-raid preference lifecycle or reward fixture.

Own runtime compiled pinned Git dependencies and executed successfully; absent reference clones are not a runtime blocker. Discovery assertion fails as expected; reviewed own publication and pet scalar tests pass 2/2. Negative control replaces one successful row with a fabricated global and fails exactly on that additional gap. Six inherited iced manifest warnings remain untouched. No broad gate or host configuration change.

## Portable historical evidence

[Validator](../../../data/patch-api/evidence/3.3.3-session-2026-10-09/validate.py) is stdlib-only and reads only its evidence directory. [Frozen manifest](../../../data/patch-api/evidence/3.3.3-session-2026-10-09/historical-inputs.json) SHA-seals 22 original input/receipt files and a 292,043-byte archive of selected parser/extractor, 68 actual later retail registers, test declarations and pet backing code. It derives accounting/status/gap/successor sets, original RED/GREEN summaries and negative counts; no hard-coded count receipts. All 113 original ledger IDs and 11-gap fixture are frozen in separate `historical-page-coverage.json` / `historical-known-gaps.json` files, not current mutable source/test ledgers. This avoids the 4.1 mutable-input boundary and any target-directory dependency.

Own portability fixture first fails because the validator is missing. Fresh-process replay, current-ledger drift independence, serialized source/log/ledger/manifest/archive tamper rejection and byte-exact restoration are pending the validator implementation commit. No Git objects, target directory, current files, runtime replay or native proof is required or claimed by historical validation. Main owns integration and final gates; historical proof remains separate from later closures.

## Sources

- [Pinned source](../../../data/patch-api/sources/3.3.3-api-changes.wikitext).
- [Source pin](../../../data/patch-api/evidence/3.3.3-session-2026-10-09/source-pin.json).
- [Spec](../../specs/patch-3-3-3-publication-sweep.md).

## See Also

- [[patch-4-1-0-api-audit]] — first already-integrated actual retail successor; 3.3.5 and 4.0.1 are coordinator-owned placeholders.
