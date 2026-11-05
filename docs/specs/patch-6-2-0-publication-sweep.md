# Patch 6.2.0 publication sweep

Audit pageid 149103, revision 1460518. This is a narrative page, not a redirect/stub; it has no inventory entries.

## Requirements

- [x] Pin parent-page response, revision, wikitext and digest; preserve the automated-diff reference without pretending it was expanded.
- [x] Account for every retained extract statement and generate a zero-entry publication register.
- [x] Add a prefork publication sweep with chronological merged 6.2.2/6.2.4 registers.
- [x] Preserve modern eligible spell-link costs under the [WM98 shared cost contract](spell-tooltip-cost.md); the original omission requirement is superseded.
- [x] Verify existing difficulty identifiers 23/24 without claiming native five-player difficulty metadata.
- [x] Complete targeted tests, source reproduction, warning-clean Mists check and portable historical validators.
- [ ] Historical item-link modifier/upgrade scaling and complete native spell/difficulty parity remain unmodeled.

## WM98 correction — 2026-11-04

The retained 6.2.0 extract at `data/patch-api/sources/6.2.0-api-changes.txt:12`
describes frame methods, not measured modern `C_TooltipInfo` getters. WM98
rejects blanket cost omission for Forever 70707, Retail 70688 and PTR 70668.
The [shared contract](spell-tooltip-cost.md) now preserves eligible costs and
applies native mode/content/attribute gates. PTR 70493 remains a native-evidence
gap. Historical source/accounting records and past proof are not rewritten.

## Proof and implementation

[Wiki audit](../wiki/investigations/patch-6-2-0-api-audit.md), [occurrence ledger](../../data/patch-api/sources/6.2.0-page-coverage.json), `src/c_api/spell_tooltip_cost.rs`, `tests/patch_6_2_0_behavior.rs`, `tests/patch_6_2_0_publication_sweep.rs`.

## Exclusions

No vendor edits, shims, invented server catalogs, diff-page expansion, full suite, push, merge or agents.
