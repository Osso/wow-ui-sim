# Patch 8.3.0 publication sweep

Audit Warcraft Wiki page 109654 revision 6471393 against current retail 12.1.0. Source lives in `data/patch-api/sources/8.3.0-*`; [audit](../wiki/investigations/patch-8-3-0-api-audit.md) records proof boundaries.

## What it must do

- [ ] Retain all consolidated and late-build inventory occurrences, numerical headers and command-column identity; preserve every non-inventory statement and citation with recorded opt-in flags.
- [ ] Apply every later master register chronologically and require exact reviewed gap identities, not zero gaps.
- [ ] Preserve previous registers byte-identically and prior extraction outcomes; prove own saved extract reproduces.
- [ ] Prove all publication sweeps, negative control, formatting, Mists tests check and retail startup `[]`.

## How it works

- [Publication accounting](../wiki/investigations/patch-8-3-0-api-audit.md).

## Implementation inventory

- `tests/patch_8_3_0_publication_sweep.rs` — current-retail publication/absence sweep.
- `tests/data/patch_8_3_0_sweep_known_gaps.json` — exact reviewed failures.
- `tools/gen_patch_wikitext_register.py` — opt-in legacy numerical headers and command-column identity.
- `tools/extract_patch_non_inventory.py` — opt-in inline reference retention.

- `src/c_api/patch_retired_members.rs` — retail-only five-member 8.3.0 retirement.
- `src/lua_api/globals/stubs/global_stubs.rs` — exclude removed deposit-rate global on retail only.
- `tests/patch_8_3_0_publication_fixes.rs`, `patch_8_3_0_cached_surfaces.rs`, `patch_8_3_0_classic_surfaces.rs` — bare/cached-retail and Mists behavioral regression.

## Tests asserting this spec

- `tests/patch_8_3_0_publication_sweep.rs` via `cargo test --test prefork_full_ui -- publication_sweep`.
- `tools/test_gen_patch_wikitext_register.py` and `tools/test_extract_patch_non_inventory.py` — concrete serialized-output fixtures.

## Known gaps (current cycle)

- [ ] Forty reviewed-gap identities require final per-ID review and proof.

## Out of scope

Historical surface reconstruction, native signature/output/security parity from publication alone, invented placeholders, vendor/cache/canonical edits, push, merge and delegation.
