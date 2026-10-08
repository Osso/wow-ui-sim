# Patch 5.5.4 API page audit

Audit the pinned [5.5.4 source](../../data/patch-api/sources/5.5.4-api-changes.wikitext) as Mists Classic, not the original retail MoP line. [Audit decision](../wiki/investigations/patch-5-5-4-api-audit.md) describes evidence boundaries.

## What it must do

- [ ] Preserve the resources-only page's zero API inventory and four metadata rows; do not expand linked diffs.
- [ ] Probe any inventory rows under `client-mists`, using the Mists cached UI and active interface `50504`.
- [ ] Keep supersession within a register's explicit client line; legacy unlabeled registers remain retail.
- [ ] Reject a sweep executed under the wrong client profile.
- [ ] Preserve every existing retail sweep's observations and exact known-gap sets.
- [ ] Reproduce all historical registers/extracts, retaining precisely the inherited extract failures.
- [ ] Validate committed evidence from a clean relocated checkout and after unrelated later work.

## How it works

- [Audit](../wiki/investigations/patch-5-5-4-api-audit.md).
- [Profile selection](../wiki/systems/client-profiles.md).
- [Validator portability](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: opt-in `client_line` metadata.
- `tools/extract_patch_non_inventory.py`: opt-in positional navigation rendering.
- `tests/common/publication_sweep.rs`: client-line supersession boundary.
- `data/patch-api/evidence/5.5.4-session-2026-10-08/`: pinned source and proof artifacts.

## Tests asserting this spec

- `tools/test_gen_patch_wikitext_register.py`.
- `tools/test_extract_patch_non_inventory.py`.
- `tests/patch_5_5_4_publication_sweep.rs`: Mists cached-UI discovery and same-line controls.
- `tests/publication_sweep_client_lines.rs`: retail cross-line, ordering and wrong-profile controls.

## Known gaps (current cycle)

- [ ] Profile loading and targeted acceptance proofs pending.

## Out of scope

Linked GitHub diffs are pointers, not an enumerated API contract. No native-client signature/output/behavior parity follows from an empty publication inventory. No runtime API implementation or retirement is justified by this source revision.
