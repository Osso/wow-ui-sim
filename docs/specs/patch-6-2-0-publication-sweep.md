# Patch 6.2.0 publication sweep

Audit pageid 149103, revision 1460518. This is a narrative page, not a redirect/stub; it has no inventory entries.

## Requirements

- [x] Pin parent-page response, revision, wikitext and digest; preserve the automated-diff reference without pretending it was expanded.
- [x] Account for every retained extract statement and generate a zero-entry publication register.
- [x] Add a prefork publication sweep with chronological merged 6.2.2/6.2.4 registers.
- [x] Model Retail and PTR spell-link cost omission while retaining direct spell costs and all non-cost payload lines/identity; retain pre-Warlords behavior.
- [x] Verify existing difficulty identifiers 23/24 without claiming native five-player difficulty metadata.
- [x] Complete targeted tests, source reproduction, warning-clean Mists check and portable historical validators.
- [ ] Historical item-link modifier/upgrade scaling and complete native spell/difficulty parity remain unmodeled.

## PTR baseline correction

PTR uses the same post-Warlords hyperlink cost policy as Retail. The pinned
70493 `Blizzard_APIDocumentationGenerated/TooltipInfoDocumentation.lua:337–353`
declares `GetHyperlink`; the retained 6.2.0 extract at
`data/patch-api/sources/6.2.0-api-changes.txt:12` documents omission of spell-link
costs while direct spell queries retain them. This is historical compatibility
evidence, not a fresh native PTR measurement. The same integration fixture now
runs on PTR, preserving both short/full links, non-cost DTO lines, frame spell
identity and restoration of direct-query cost lines.

## Proof and implementation

[Wiki audit](../wiki/investigations/patch-6-2-0-api-audit.md), [occurrence ledger](../../data/patch-api/sources/6.2.0-page-coverage.json), `src/c_api/spell_tooltip_cost.rs`, `tests/patch_6_2_0_behavior.rs`, `tests/patch_6_2_0_publication_sweep.rs`.

## Exclusions

No vendor edits, shims, invented server catalogs, diff-page expansion, full suite, push, merge or agents.
