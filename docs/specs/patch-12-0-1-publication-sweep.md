# Patch 12.0.1 publication sweep

## Contract

Probe all 225 consolidated inventory occurrences from retained page 659762, revision 6747895, in the unmodified cached Game environment. Publication/absence only: no signature, result, behavior, security or native parity credit. Non-inventory statements require separate behavioral evidence.

Use the shared classifier in `tests/common/publication_sweep.rs`, including exact cached deprecation-alias attribution. Apply later add/remove inventories from 12.0.5, 12.0.7 and 12.1.0 in order; changed rows alone do not reverse publication. The 12.0.0 sweep includes 12.0.1 as its earliest later register.

Require the exact observed non-OK ID set to equal `tests/data/patch_12_0_1_sweep_known_gaps.json`. `P1201_SWEEP_OUT` writes every observation before assertion; `P1201_SWEEP_REGISTER` accepts a full scratch register with the same row count. One flipped unsuperseded row must introduce exactly one new gap.

## Retirement epoch

`Cargo.toml` and `src/client_profile.rs` expose 12.0.0, 12.0.5, 12.0.7 and later epochs; no 12.0.1 feature exists. The source explicitly compares 12.0.0 (65655) to 12.0.1 (66838). `C_NamePlate.GetTargetClampingInsets` and `SetTargetClampingInsets` were observed raw-absent but fabricated by ordinary namespace lookup. Mark them retired at `retail-12-0-5`, the first supported epoch after removal. A 12.0.0 gate would incorrectly retire APIs on the preceding surface. Earlier-epoch preservation is source-gate inspected, not runtime-proven under this retail-only task.

## Verification

- [ ] Review every non-OK row and baseline exact IDs.
- [ ] Fix cheap root-cause publication defects without fabricated successor behavior.
- [ ] Prove one-row negative control.
- [ ] Run all five publication sweeps separately under debug retail, local build host.
- [ ] Startup Lua errors `[]`; formatting and local check.

## Inputs and tests

- `data/patch-api/sources/12.0.1-api-changes.wikitext` and provenance: source capture.
- `data/patch-api/sources/12.0.1-wikitext-register.json`: inventory occurrences; all eight added/removed header counts match.
- `tests/patch_12_0_1_publication_sweep.rs`: shared cached publication sweep.
- [Shared sweep contract](patch-12-0-7-publication-sweep.md).
