# Retail 5.3.0 publication sweep

Audit the pinned retail 2013 API page and its separately pinned diff. [Audit](../wiki/investigations/patch-5-3-0-api-audit.md) describes evidence and limits.

## What it must do

- [ ] Probe every inventory occurrence, with reviewed non-ok IDs exactly matching the known-gap fixture.
- [ ] Apply only later retail registers; retain queued 5.4.0, 5.4.1 and 5.4.2 placeholders in that order.
- [ ] Prove existing PvP role reads follow stored selection transitions in bare and cached environments.
- [ ] Verify the two historical removal names remain absent, without adding retirement gates.
- [ ] Reproduce every saved register and preserve inherited extract failures exactly.

## How it works

- [Publication audit](../wiki/investigations/patch-5-3-0-api-audit.md).
- [Validator portability](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `tests/patch_5_3_0_publication_sweep.rs`: register-driven cached discovery.
- `tests/patch_5_3_0_behavior.rs`: bounded existing role backing and absence assertions.
- `tools/gen_patch_wikitext_register.py`: opt-in caption/transclusion/handler parsers.
- `data/patch-api/sources/5.3.0-*`: source pins, register, extract and accounting.

## Tests asserting this spec

- `tests/patch_5_3_0_publication_sweep.rs`.
- `tests/patch_5_3_0_behavior.rs`.
- `tools/test_patch_mists_transclusion.py`.

## Known gaps (current cycle)

- [ ] Discovery, accounting and final gates are running; no completion claim yet.

## Out of scope

Historical full game-service parity, native embedded browsing, Classic 5.5.x, full integration suite, vendor edits, push and merge.
