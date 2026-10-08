# Patch 6.2.0 publication sweep

Audit pageid 149103, revision 1460518. This is a narrative page, not a redirect/stub; it has no inventory entries.

## Requirements

- [x] Pin parent-page response, revision, wikitext and digest; preserve the automated-diff reference without pretending it was expanded.
- [x] Account for every retained extract statement and generate a zero-entry publication register.
- [x] Add a prefork publication sweep with chronological 6.2.2/6.2.4 integration placeholders.
- [x] Model retail spell-link cost omission while retaining direct spell costs and all non-cost payload lines/identity; retain pre-Warlords behavior.
- [x] Verify existing difficulty identifiers 23/24 without claiming native five-player difficulty metadata.
- [ ] Complete targeted tests, source reproduction, warning-clean Mists check and portable historical validators.
- [ ] Historical item-link modifier/upgrade scaling and complete native spell/difficulty parity remain unmodeled.

## Proof and implementation

[Wiki audit](../wiki/investigations/patch-6-2-0-api-audit.md), [occurrence ledger](../../data/patch-api/sources/6.2.0-page-coverage.json), `src/c_api/spell_tooltip_cost.rs`, `tests/patch_6_2_0_behavior.rs`, `tests/patch_6_2_0_publication_sweep.rs`.

## Exclusions

No vendor edits, shims, invented server catalogs, diff-page expansion, full suite, push, merge or agents.
