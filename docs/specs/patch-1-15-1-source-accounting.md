# Patch 1.15.1 literal source and alias accounting

Account frozen Warcraft Wiki page577687/revision5998991/timestamp `2024-04-03T08:43:49Z` without expanding links. [Audit](../wiki/investigations/patch-1-15-1-api-audit.md).

## What it must do

- [x] Validate response/raw identity, frozen manifest hashes and 101-page registry ending1.0.0.
- [x] Preserve all nine nonblank rows, two named enum occurrences, three prose contracts, four unexpanded links, two headers and navigation template; invent no signatures/numeric values/subset membership.
- [x] Preserve concrete Placeholder → SeasonOfDiscovery alias-continuity contract, separate from unspecified Dragonflight10.2.5 subset and deprecated-file link.
- [x] Separate configured Era/Anniversary11507, in-flight1.15.2, queued1.15.3 and integrated1.15.4–9 inputs from native/integration proof.
- [x] Reject omissions, invented credit/values/foreign supersession and disk tampering; replay copied historical inputs without Git/target/current tools, retaining immutable original seals.
- [x] Execute unchanged pinned official deprecation with existing headless Era IsPublicBuild=true and raw finite SeasonOfDiscovery=2; assert raw Placeholder=2 and numeric alias equality. No mock flags or native numeric inference.

## How it works

- [Literal coverage matrix and loading boundary](../wiki/investigations/patch-1-15-1-api-audit.md).

## Implementation inventory

`data/patch-api/evidence/1.15.1-session-2026-10-09/`: literal adapter, frozen inputs, source/portable tests and independent external-primary-source pin. `patch-tests/patch_1_15_1_alias.rs` and Cargo standalone Era target: direct existing-state alias proof. No simulator implementation changes.

## Tests asserting this spec

Own `test_source_accounting.py`: RED8/8 retained; SOURCE GREEN8/8 at `3045ff2a9`, 29 omission controls and fabricated-credit/value/supersession rejection. `test_portable.py` RED3 retained; GREEN3/3 at `714277147`, copied SOURCE8/default-byte replay and both serialized seal tamper rejections/exact restorations. `patch-tests/patch_1_15_1_alias.rs` GREEN1/1 at `714277147` under `--no-default-features --features client-era --test patch_1_15_1_alias`; unchanged official deprecation against actual headless Era state proves raw named numeric values2/equality. No native numerical contract asserted. Original57 seals/1069714bytes remain immutable; later receipts separately sealed. Seven existing simulator warnings and six vendor-manifest deprecations retained; no warning-free/final-gate claim.

## Known gaps (current cycle)

- [ ] Native numeric SeasonID values, current Era official cache, full deprecated-addon loading/public-build branches and season detection state remain unproven. Bare initializer absence is not an established defect.
- [ ] Main owns successor integration/native/final gates.

## Out of scope

Runtime edits, invented aliases/defaults/flags, linked-page body expansion, vendor/cache/Wowless edits, foreign-history supersession, shared-tool changes, broad/check/lint/type/final gates, delegation/model CLI, push/merge/deploy.
