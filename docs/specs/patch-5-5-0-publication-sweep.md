# Patch 5.5.0 publication sweep

Audit the pinned Mists Classic launch page, not historical retail MoP. Source: `data/patch-api/sources/5.5.0-api-changes.wikitext`. See [audit](../wiki/investigations/patch-5-5-0-api-audit.md) for loader and evidence boundaries.

## What it must do

- [ ] Reproduce the pinned revision's extract and mists-classic register; account for every source row.
- [ ] Assert current Mists profile, interface 50504, Mists cache, and successful real SharedXML publication before classifying the empty inventory.
- [ ] Restrict later registers to Classic 5.5.1–5.5.4; reject an injected inventory row.
- [ ] Preserve retail publication observations and client-line controls; pass Mists page tests and warning-free non-vendor check.
- [ ] Validate committed session proof with pinned shared inputs in clean and synthetic-later-audit checkouts.

## How it works

- [Audit and loader decision](../wiki/investigations/patch-5-5-0-api-audit.md).
- [Shared classifier](../../tests/common/publication_sweep.rs).

## Implementation inventory

- `tests/patch_5_5_0_publication_sweep.rs`: Mists empty-inventory discovery boundary.
- `tests/data/patch_5_5_0_sweep_known_gaps.json`: empty identity-gap fixture, not the broad prose gap.
- `data/patch-api/sources/5.5.0-page-coverage.json`: five metadata rows and one problematic synchronization statement.
- `data/patch-api/evidence/5.5.0-session-2026-10-08/`: retained source response, receipts, reproduction and validator.

## Tests asserting this spec

- `tests/patch_5_5_0_publication_sweep.rs`.
- `tests/publication_sweep_client_lines.rs`.
- Session `validate.py` and `tools/check_patch_validators.py`.

## Known gaps (current cycle)

- [ ] Broad synchronization through Mainline 11.1.7 names no identities or contracts. No equivalence claim; retail linked-page and GitHub diffs remain unexpanded.

## Out of scope

Full-Game startup, native signature/output/security parity and external diff expansion. No source-listed identities or removals warrant runtime changes. Future pages with real Lua identities require their real cached publishing addons, not this empty-inventory loader alone.
