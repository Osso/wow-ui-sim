# Patch 11.1.0 publication sweep

## Contract

Probe every inventory occurrence from Warcraft Wiki page 616105, revision 6726776, against unmodified cached Game UI. Default retail carries 12.1.0, not historical 11.1.0. Apply all ten later registers, 11.1.5 through 12.1.0, chronologically. Latest add/remove wins; changes preserve publication. Preserve original direction and supersession IDs. Require exact reviewed gap IDs; write all observations before asserting the fixture. P1110_SWEEP_OUT selects results; P1110_SWEEP_REGISTER selects a same-sized negative-control register.

Publication/absence only: no signature, output, security, behavior or native parity claim. Explicit registration may still be a placeholder. Generic namespace autostub lookup is not explicit publication. Never delete cached Blizzard deprecation wrappers.

## Bounded behavior

- Removed C_BarberShop.GetCustomizationScope and C_TransmogCollection.CanAppearanceBeDisplayedOnPlayer stay absent on repeated ordinary/raw namespace lookup. Current cached retail Lua has no consumers. Classic registration remains unchanged.
- GetSpecializationNameForSpecID returns the existing English specialization catalog name for valid IDs (70 Retribution, 65 Holy, 577 Havoc), and nil for unknown IDs. Gender does not alter this catalog's names; localized gender-specific names remain unproven.
- SetSpecialization stays published: cached Blizzard_TalentUI/Mists/Blizzard_TalentUI.lua:70 still calls it. Retain the historical removal as a gap, not deprecated-alias credit.
- Multiply indented changed API rows must be retained with their own annotation; C_PlayerInfo.GetSex is a distinct row, not an annotation on the preceding mount API.

## Acceptance

- [x] Exact 19-gap fixture and exhaustive 216-row inventory/non-inventory ledger.
- [x] Every publication sweep alone, new behavioral tests and one-row negative control.
- [x] Relevant isolated prefork cases, formatting, Mists test check without non-vendor warnings and startup [].

## Local proof

Runtime/test revision `3859f4566`; later changes are evidence/docs only. [Proof ledger](../../data/patch-api/evidence/11.1.0-session-2026-10-06/p1110-proof.json) retains command, exact revision, outcome and invalidated RED scopes. No independent/native acceptance; no full suite.

| Isolated sweep | Rows | OK | Exact gaps | Result |
|---|---:|---:|---:|---|
| 11.1.0 | 116 | 97 | 19 | PASS |
| 11.1.5 | 125 | 89 | 36 | PASS |
| 11.1.7 | 48 | 40 | 8 | PASS |
| 11.2.0 | 162 | 135 | 27 | PASS |
| 11.2.5 | 163 | 118 | 45 | PASS |
| 11.2.7 | 508 | 414 | 94 | PASS |
| 12.0.0 | 1010 | 989 | 21 | PASS |
| 12.0.1 | 225 | 222 | 3 | PASS |
| 12.0.5 | 363 | 352 | 11 | PASS |
| 12.0.7 | 174 | 171 | 3 | PASS |
| 12.1.0 | 778 | 773 | 5 | PASS |

Each sweep runs alone via generated integration target, `--nocapture --test-threads=1`. No one-target-per-test Cargo additions. Ten later registers regenerate byte-identically; their sources/fixtures/coverage ledgers remain unchanged from p1115-page. Negative control changes only C_WarbandScene.SetFavorite added → removed: exactly one new gap, no resolutions, 19 → 20, expected exit 101.

Two new behavioral tests fail before implementation and pass afterward. Separate prefork filters for Collections explicit load and deprecated specialization wrapper preservation each run one passing case. Fifteen extractor/register fixtures pass. `cargo fmt` / `cargo fmt --check` and Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` pass with zero non-vendor warnings; six pre-existing iced manifest deprecations plus vendor summary remain. Separate retail build and bounded exit-0 startup return []. Changed Rust lines manually reviewed for readability.

216 unique source IDs: 71 partial-development-green, 18 bounded-coverage, 106 audit-pending, 21 metadata-only. Non-inventory: 87 pending contracts, 13 editorial rows. Eight later reversals carry metadata-only credit. Ledger remains in-progress; publication accounting is not complete behavioral/native parity.

[Gap review](../../data/patch-api/evidence/11.1.0-session-2026-10-06/p1110-gap-review.json) records three closures and 19 precise retained boundaries; [extract scout](../../data/patch-api/evidence/11.1.0-session-2026-10-06/p1110-extract-scout.md) assigns all 100 extract rows. [Audit](../wiki/investigations/patch-11-1-0-api-audit.md) records the parser omission and sibling build-cache path violation. Local Cargo logs are ignored artifacts; retained JSON captures result/command/revision summaries.
