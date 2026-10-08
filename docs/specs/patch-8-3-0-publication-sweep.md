# Patch 8.3.0 publication sweep

Audit Warcraft Wiki page 109654 revision 6471393 against current retail 12.1.0. Source lives in `data/patch-api/sources/8.3.0-*`; [audit](../wiki/investigations/patch-8-3-0-api-audit.md) records mechanisms and proof boundaries.

## What it must do

- [x] Retain consolidated and late-build inventory identities, numerical headers, command-column kind, every extract statement and inline citation with recorded opt-in flags.
- [x] Apply all later master registers chronologically and require exact reviewed gap identities, not zero gaps.
- [x] Preserve earlier register/source bytes and extraction outcomes; reproduce own saved extract and all registers with recorded or explicitly inferred recipes.
- [x] Keep removed members/globals absent under raw, ordinary and repeated retail lookup, including unmodified cached full-UI loading; preserve Mists legacy lookup and defaults.
- [x] Prove publication sweeps, exact negative control, formatting, default/Mists checks and bounded retail startup `[]`; report unrelated failures without suppressing them.

## How it works

- [Publication accounting and sweep table](../wiki/investigations/patch-8-3-0-api-audit.md).

## Implementation inventory

- `tests/patch_8_3_0_publication_sweep.rs` — current-retail register-driven publication/absence sweep.
- `tests/data/patch_8_3_0_sweep_known_gaps.json` — exact reviewed publication failures.
- `tools/gen_patch_wikitext_register.py` — opt-in prose headers, command-column kind and explicit late-build additions.
- `tools/extract_patch_non_inventory.py` — opt-in inline reference retention.
- `src/c_api/patch_retired_members.rs` — separate retail-only five-member 8.3.0 list.
- `src/lua_api/globals/stubs/global_stubs.rs` — exclude removed deposit-rate global on retail only.
- `data/patch-api/evidence/8.3.0-session-2026-10-08/validate.py` — dynamic source/ledger/fixture/results/proof validation.

## Tests asserting this spec

- `tests/patch_8_3_0_publication_sweep.rs` via `cargo test --test prefork_full_ui -- publication_sweep`.
- `tests/patch_8_3_0_publication_fixes.rs`, `patch_8_3_0_cached_surfaces.rs`, `patch_8_3_0_classic_surfaces.rs` — bare/cached-retail and Mists regressions.
- Existing `tests/mists_product_choice_compat.rs` and legacy startup API-shape test in `tests/mists_compat_bootstrap.rs` — every affected classic caller.
- `tools/test_gen_patch_wikitext_register.py`, `tools/test_extract_patch_non_inventory.py` — concrete serialized-output fixtures.
- [Proof ledger](../../data/patch-api/evidence/8.3.0-session-2026-10-08/p830-proof.json).

## Known gaps (current cycle)

- [ ] Forty-one exact publication gaps remain, each with precise producer/lifecycle/profile-safe migration boundary in the audit review.
- [ ] Broad Auction House revamp summary remains behaviorally unproven; publication is not historical overhaul parity.
- [ ] Unrelated unchanged Mists HonorFrame diagnostic-string assertion fails in the broader selection; five affected cases pass. No unrelated test/VM/vendor fix authorized.

## Out of scope

Historical surface reconstruction, native signature/output/security parity from publication alone, invented placeholders, vendor/cache/canonical edits, push, merge and delegation. Wrath/Era/Anniversary are preserved by existing feature gates, not claimed executed. Inherited 12.0.5/12.0.7/12.1.0 extract non-reproducibility remains untouched.
