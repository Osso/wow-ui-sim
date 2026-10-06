# Patch 11.2.7 publication sweep

## Contract

Probe all 508 consolidated inventory occurrences from page 649551, retained revision 6726771, in the unmodified cached Game environment. Publication/absence only: no signature, output, security, behavior or native parity credit. Preserve raw wikitext and source provenance. Regenerate all five existing registers byte-identically.

Use `tests/common/publication_sweep.rs`. Apply 12.0.0, 12.0.1, 12.0.5, 12.0.7 and 12.1.0 registers chronologically. Latest later add/remove wins; changed rows do not reverse publication. Source direction remains recorded alongside effective expectation and supersession ID. The shared implementation compares symbol keys, not the starting register's position; an older-than-all register requires no index special case.

Require the exact observed non-OK ID set to equal `tests/data/patch_11_2_7_sweep_known_gaps.json`. `P1127_SWEEP_OUT` writes observations before assertion; `P1127_SWEEP_REGISTER` accepts a full scratch register of the same size. A one-row negative control must add exactly one gap.

## Epoch and scope

Earliest supported retail epoch is `retail-12-0-0`; no 11.x runtime exists. Default retail carries 12.1.0. Unsuperseded 11.2.7 removals are baseline absence for every supported **retail epoch**, not every client profile: classic clients have independent historical contracts. Any publication fix must preserve those profile boundaries. No new 11.x feature, fabricated backing model, vendor rewrite or Blizzard monkey-patch is authorized.

## Acceptance

- [ ] Review every non-OK row; fix cheap root-cause publication defects and retain explicit model gaps.
- [ ] Exact known-gap GREEN and one-row negative control.
- [ ] Create unique inventory/extract source-ID ledger with publication-only credit.
- [ ] Scout every non-inventory statement without claiming behavioral parity.
- [ ] Run all six publication sweeps alone, local debug retail.
- [ ] Formatting, default local check, warning-free non-vendor Mists test check, startup Lua errors `[]`.

## Sources

- `data/patch-api/sources/11.2.7-api-changes.wikitext` and provenance.
- `data/patch-api/sources/11.2.7-wikitext-register.json`.
- [Session evidence](../../data/patch-api/evidence/11.2.7-session-2026-10-06/).

## Retirement fixes

Initial cached sweep exposes explicit global `JoinBattlefield` and fabricated lookup members `C_ReturningPlayerUI.AcceptPrompt` / `DeclinePrompt` after removal. Stop the legacy global publisher from `retail-12-0-0`; mark both namespace keys removed in the already retail-gated retirement module. Preserve classic publication and the existing state-backed `C_PvP.JoinBattlefield` queue producer. Regression asserts repeated raw/ordinary absence and concrete successor queue state. Existing queue tests use the live namespace successor rather than the retired global. No new backing model or fallback.

The same fabricated-member defect affects removed `C_CharacterServices.RPEResetCharacter`. Extend the retirement list and repeated-lookup regression; no character reset behavior is invented. All four initial unsuperseded removal defects now have producer/lookup fixes.

Ten missing 11.2.7 Command records are a catalog omission, not a missing execution model. Publish their exact source names/types on supported retail epochs using `c_api::c_console`; classic catalogs remain unchanged. Empty native metadata remains the existing documented inference. Concrete command RED observed zero of ten records.

## Extract compatibility

Validation exposed an unintended 12.0.7 extractor change: resuming at Structures on headed pages now parsed templates the old tool excluded. Restrict wikitable/Structures boundary handling to pages without a Global API heading; preserve all existing headed-page rendering, including pre-existing unsupported-template outcomes. New RED fixture proves the old headed inventory exclusion. This Python-only correction leaves Rust sweep/check/startup proof valid.
