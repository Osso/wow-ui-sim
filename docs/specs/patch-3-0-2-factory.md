# Patch 3.0.2 current-retail bare-factory measurement

Separate publication/designed-absence measurement of the exact historical retail register, not historical behavior or native parity. Source accounting remains [unchanged](patch-3-0-2-publication-sweep.md). [Audit](../wiki/investigations/patch-3-0-2-api-audit.md#separate-current-retail-factory-measurement--2026-10-09).

## What it must do
- [x] Observe all 373 literal occurrences with a bare current-retail environment; discovery starts with empty gaps and retains every observation before asserting.
- [x] Apply actual same-line retail successors oldest first, starting 3.0.3/3.0.8/3.1.0/3.2.0/3.3.0/3.3.3/3.3.5/4.0.1; never use Classic supersession.
- [x] Require exact reviewed gap identities; fabricated unknown global replaces one matching occurrence and yields one extra gap without changing the 373-row count.
- [x] Preserve original 36 seals, closure seal and two receipt seals; retain exact revisions, argv, environment overrides, full streams and hashes separately.
- [x] Report precise FrameXML, widget-handler, click-modifier, inherited-template and console/event classifier limits without runtime changes.

## How it works
- [Audit](../wiki/investigations/patch-3-0-2-api-audit.md).
- [Evidence](../../data/patch-api/evidence/3.0.2-factory-2026-10-09/).

## Implementation inventory
- `tests/patch_3_0_2_factory.rs`: standalone whole-file retail guard and exact register/successor inputs.
- `tests/common/publication_sweep.rs`: shared test-only classifier, no cached publishers for factory sweeps.
- `Cargo.toml`: explicit test target requiring client-retail.

## Tests asserting this spec
- `tests/patch_3_0_2_factory.rs`: seven own tests GREEN at `57c9f73e3`; literal inventory, publication, foreign-client rejection and exact limits.
- Separate command receipts: discovery RED, retained 6/7 console-control RED, corrected 7/7 GREEN, and same-373 fabricated-global RED (188 → 189 gaps).

## Known gaps (current cycle)
- [ ] 188 current classifier gaps remain; 185 matches do not close any historical behavior contract.
- [ ] Every historical/native/model claim remains zero; no loaded UI or inherited secure-template acceptance.

## Out of scope
Historical 3.0.2 runtime/native, signatures, substantive prose, meaningful models, loaded-UI parity, invented factories/aliases/shims, runtime/vendor/cache/Wowless writes, networking, delegation, push/merge/deploy and broad/final gates. Meaningful models require a literally backed state proposal to main first.
