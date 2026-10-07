# Patch 11.1.0 API page audit

Page 616105, revision 6726776 (May 25, 2026), retrieved October 7, 2026 UTC. Audit accounts for 116 inventory occurrences and 100 non-inventory rows. Requested evidence path keeps October 6. Default retail carries 12.1.0, not a reconstructed 11.1.0 client.

## Source and supersession

All eight added/removed header counts match. All ten later registers, 11.1.5 through 12.1.0, apply chronologically; latest add/remove wins. Eight reversals receive metadata-only credit. Rebased onto p1115-page because master lacked its required supersession register/test. Existing later sources, coverage ledgers and gap fixtures remain unchanged against that baseline. All ten later registers regenerate byte-identically with the corrected parser.

The first register had 115 rows. A two-space-indented C_PlayerInfo.GetSex entry was omitted, attaching its annotation to the preceding mount API. Regression fixture fails before fixing indentation recognition; corrected register has 116 rows and a distinct GetSex annotation. This correction exposes a PlayerLocation query gap, not a local-player-only getter opportunity. Wikitext stays authoritative; symbolic TOC interface is not expanded to today's value.

## Root causes and bounded fixes

Initial cached sweep: 94 OK / 21 gaps across 115 rows. Two removed namespace members were fabricated by ordinary namespace autostub lookup: C_BarberShop.GetCustomizationScope and C_TransmogCollection.CanAppearanceBeDisplayedOnPlayer. Full cached retail Lua search finds no consumers; existing epoch-scoped retirement policy blocks repeated ordinary/raw lookup. Cached deprecation wrappers and classic registration remain unchanged.

GetSpecializationNameForSpecID was absent despite an existing specialization catalog. Return that catalog's English name for valid IDs, nil for unknown IDs. Concrete 70/65/577 IDs and unknown/zero cases pass. Gender/localized name variants remain unproven. Two new behavioral tests fail before implementation and pass afterward. Three bounded publication gaps closed; register correction adds one discovery; final fixture retains 19 gaps.

**Retained historical removal:** SetSpecialization remains published because current cached Blizzard_TalentUI/Mists/Blizzard_TalentUI.lua:70 calls it. Mainline's C_SpecializationInfo.SetSpecialization does not authorize deleting a classic consumer. Retain as exact gap, not cached deprecation-alias acceptance.

Remaining gaps: achievement telemetry; per-addon load-error query; commentator charge classification; debug character view; delve preview quality; pet specialization mutation/catalog; profession-respec eligibility/transaction; native console echo; name declensions; native realm identity; PlayerLocation sex; seen-curio command. [Per-ID review](../../../data/patch-api/evidence/11.1.0-session-2026-10-06/p1110-gap-review.json) records all 22 initial/discovered defect outcomes and precise boundaries. No placeholders added.

## Coverage matrix

| Statements/features | Count | Proof / boundary |
|---|---:|---|
| Unsuperseded add/change publication | 71 | Partial-development-green; signatures/output/security/behavior unproven |
| Unsuperseded removals | 18 | Bounded strict absence/registration rejection; zero alias acceptances |
| Later-superseded inventory OK | 8 | Metadata-only; no historical credit |
| Inventory gaps | 19 | Missing producer/policy or retained cached consumer |
| Non-inventory contracts / editorial context | 87 / 13 | Exhaustive ranked scout / metadata-only; no runtime credit |

216 unique IDs: 71 partial, 18 bounded, 106 pending, 21 metadata. Page-accounting/publication audit complete; ledger remains in-progress because producer and behavioral gaps remain. Extract allocation: summary 3, addon metadata/grouping 12, enums 45, DTOs 27, editorial 13. Every extract ID assigned once; source-name matches never count as behavior.

## Development proof

[Spec and sweep table](../../specs/patch-11-1-0-publication-sweep.md#local-proof) and [proof ledger](../../../data/patch-api/evidence/11.1.0-session-2026-10-06/p1110-proof.json) retain exact revision/command scopes. All eleven sweeps run alone and pass. Later observations match fixtures already present on p1115-page, including its 11.2.0 27-gap and 12.0.0 21-gap baseline; no later fixture edited. One-row negative control changes C_WarbandScene.SetFavorite added to removed: exactly one new gap, no resolutions, 19 → 20, expected exit 101.

Two new behavioral tests GREEN after RED; two isolated prefork cases (Collections explicit load and deprecated specialization wrappers) pass. Fifteen extractor/register fixtures, deterministic extract reproduction, register regeneration and formatting pass. Mists test check has zero non-vendor warnings; six existing iced manifest warnings plus vendor summary stay unsuppressed. Separate retail build and bounded exit-0 startup return []. Changed Rust lines manually reviewed for readability. Later source changes are evidence/docs only; current runtime proofs stay valid. No full suite or independent/native acceptance.

## Process exception

[Build-cache incident](../../../data/patch-api/evidence/11.1.0-session-2026-10-06/p1110-build-cache-incident.json): all commands used explicit required worktree cwd, but the initial sweep set CARGO_TARGET_DIR to the sibling p1115-page target. That wrote build artifacts outside the authorized worktree. Subsequent commands used p1110-page target; its cache was hardlinked from the sibling. Source files and canonical checkout remained untouched; artifact-path compliance was violated. This does not erase the violation. No agents/models, push, merge, vendor edits or Blizzard monkey-patching.

## Sources

- [Publication contract](../../specs/patch-11-1-0-publication-sweep.md)
- [Source provenance](../../../data/patch-api/sources/11.1.0-api-changes.provenance.json)
- [Page ledger](../../../data/patch-api/sources/11.1.0-page-coverage.json)
- [Extract scout](../../../data/patch-api/evidence/11.1.0-session-2026-10-06/p1110-extract-scout.md)
- [Retirement consumer searches](../../../data/patch-api/evidence/11.1.0-session-2026-10-06/p1110-retirement-consumers.json)
- [Artifact validator](../../../data/patch-api/evidence/11.1.0-session-2026-10-06/p1110-validate.py)

## See Also

- [[patch-11-1-5-api-audit]] — template and immediate supersession boundary.
- [[patch-11-1-7-api-audit]] — exact-gap and publication-proof conventions.
- [[client-profiles]] — supported profiles and current retail epoch.
