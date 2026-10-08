# Retail 5.4.1 publication audit

Audit the pinned [2013 retail source](../../data/patch-api/sources/5.4.1-api-changes.wikitext), not Mists Classic. [Audit methodology and limits](../wiki/investigations/patch-5-4-1-api-audit.md).

## What it must do

- [ ] Probe every registered inventory occurrence in the prefork Game environment and compare the exact non-ok ID set with reviewed gaps.
- [ ] Apply later retail registers in order; keep queued 5.4.2, 5.4.7, 5.4.8 comments before 6.0.1/6.0.2. Never include Classic 5.5.x.
- [ ] Preserve both prose statements and all editorial context in the supplemental ledger.
- [ ] Assert only default Game-state absence of the `realmName` CVar through global and namespace reads; do not claim realm identity parity.
- [ ] Reproduce saved registers/extracts without changing previously recorded behavior and pass historical-validator portability.

## How it works

- [Audit and evidence](../wiki/investigations/patch-5-4-1-api-audit.md).
- [Shared publication probe](../../tests/common/publication_sweep.rs).

## Implementation inventory

- `tests/patch_5_4_1_publication_sweep.rs`: ordered current-retail publication/absence probes.
- `tests/patch_5_4_1_behavior.rs`: default CVar absence, backed by existing CVar state reads.
- `tests/data/patch_5_4_1_sweep_known_gaps.json`: exact reviewed publication gaps.
- `data/patch-api/sources/5.4.1-page-coverage.json`: occurrence accounting.
- `tools/gen_patch_wikitext_register.py`, `tools/extract_patch_non_inventory.py`: opt-in parser originating in 5.4.2, plus opt-in lowercase reference retention.

## Tests asserting this spec

- `tests/patch_5_4_1_publication_sweep.rs` and `tests/patch_5_4_1_behavior.rs` (prefork).
- `tools/test_patch_mists_register.py` (caption inventories, bare removals, lowercase reference opt-in).
- `data/patch-api/evidence/5.4.1-session-2026-10-08/validate.py` (pinned historical proof).

## Known gaps (current cycle)

- [ ] `PRODUCT_CHOICE_UPDATE`: absent from current strict event catalog; no product-choice selection producer remains after later namespace retirement. Adding a registerable name alone would not model delivery.
- [ ] `SelectedRealmName`: absent; no selected-realm navigation/session state distinct from player/account realm identity. Returning the existing placeholder would invent that state.
- [ ] `GetRealmName` is an existing temporary constant, not player-realm modeling; historical CVar rejection after explicit registration is unproved.
- [ ] Protected parental-control query/tainted `UpdateMicroButtons`: later removal supersedes publication; no historical parental-control secure-service policy is modeled.

## Out of scope

Classic registers, real service/network transactions, historical output signatures inferred from inventory names, vendor modifications, full integration suite, and changes to existing later-patch retirement policy. No new runtime shim or retirement is introduced.
