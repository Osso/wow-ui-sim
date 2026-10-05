# C_Secrets publication queries

The 20 assigned Retail 12.0.0 queries describe the simulator's existing secrecy policies. Authoritative cached signatures live in `SecretPredicateAPIDocumentation.lua`; the requested `SecretsDocumentation.lua` does not exist. [Implementation and boundaries](../wiki/systems/secrets-publication-queries.md).

## What it must do

- [x] Publish all 20 assigned queries in the current Retail surface; registration is gated from the Retail 12.0.0 epoch. Historical-epoch execution was not separately tested.
- [x] Return public query results without changing caller taint; authenticate every selector under `AllowedWhenUntainted` before policy lookup.
- [x] Describe aura access and spell exemptions using exactly the predicates used by `C_UnitAuras`.
- [x] Describe spell/action/book cooldown outputs using the shared cooldown restriction and actual book-entry resolution.
- [x] Describe identity outputs using existing live identity classification, including explicit GUID overrides and instance exemptions.
- [x] INFERRED bounded defaults for unmodeled aspects must describe current public outputs, not fabricate a restriction from unrelated combat/stat/aura state.

## How it works

- [Shared predicates and missing native models](../wiki/systems/secrets-publication-queries.md).

## Implementation inventory

- `src/c_api/secrets_queries.rs`: selectors, registration and current-policy queries.
- `src/c_api/c_secrets.rs`: namespace registration and base aura classification.
- `src/c_api/unit_aura_access.rs`: shared aura predicates and output/access policy.
- `src/c_api/charge_state.rs`: cooldown restriction predicate.
- `src/c_api/c_spell_book.rs`: shared displayable book-entry selection.
- `src/c_api/c_spell.rs`: shared spell identifier resolution.
- `src/lua_api/globals/unit_misc.rs`: existing identity predicate.

## Tests asserting this spec

- `tests/secrets_publication_queries.rs`: data-driven query/output agreement, live state transitions, never-secret aura exemption and selector authentication.
- Four `patch_12_*` integration publication sweeps: exact known-gap reconciliation.

## Known gaps (current cycle)

- [ ] Native per-spell cast/cooldown and per-power secrecy attribute datasets are absent. Base cast/power `NeverSecret` defaults describe current simulator outputs; they do not claim native classifications.
- [ ] Health-max, power/max, unit-comparison and foreign-unit cast output restrictions are not modeled. Their queries currently report public outputs.
- [ ] Active totem slots and associated spells are absent; the empty totem surface produces no secret data.
- [ ] Existing spell-keyed aura output ignores always-secret flags outside the active aura context. Query preserves that existing limitation rather than silently changing vendor-facing behavior.

## Verification — 2026-10-05

Code/test/fixture revision `9a579550e8f3e8552275840a1d39fec7f59da62e`; later proof documentation does not invalidate that scope. All commands used the local build host. Before filters ran at `3b4f660c4`, then both HEAD and master. Master subsequently advanced independently; no canonical checkout or sibling worktree was modified.

Six new behavioral tests pass, including a concrete public player cast and map/group/player-owned/override identity transitions. Initial RED had four missing-query failures and one invalid `wrapsecret` fixture, corrected to authentic host wrappers before GREEN. Final secret filter includes all six new tests.

| Integration filter | Before pass/fail/ignored | After pass/fail/ignored |
|---|---|---|
| secret | 265/0/0 | 271/0/0 |
| aura | 435/1/0 | 436/1/0 |
| unit | 609/1/0 | 609/1/0 |
| cooldown | 201/1/1 | 202/1/1 |
| spell | 791/3/5 | 792/3/5 |

Unchanged pre-change master failures: `on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases`; `edit_mode_api::enums::unit_frame_edit_mode_setting_meta_includes_big_defensive_icon_size`; `wowforever_cooldown_categories::forever_cooldown_categories_preserve_other_profiles`; `c_spell_static_fallbacks::test_spell_static_fallback_shims_return_inert_values`; `spell_api::test_spell_get_maw_power_border_atlas_by_spell_id_is_stubbed`; `spell_api::test_spell_get_spell_charges`. No new filter failure occurred.

All four publication sweeps ran separately with `--test-threads=1` and passed. Counts below are publication/absence only, not behavioral/native secrecy parity.

| Patch | Rows | OK | Known gaps |
|---|---:|---:|---:|
| 12.0.0 | 1010 | 832 | 178 |
| 12.0.5 | 363 | 351 | 12 |
| 12.0.7 | 174 | 171 | 3 |
| 12.1.0 | 778 | 768 | 10 |

The assigned 20 IDs all have `ok: true`; 12.0.0 changed from 812 OK / 198 gaps to 832 / 178. Other fixtures contain no shared assigned C_Secrets IDs and were unchanged. Required result lives at `~/.cache/wow-ui-sim-audit/p1200-secrets-sweep.json`.

`cargo fmt --check`, local `cargo check` and startup `lua-errors` pass; startup output is `[]`. Six pre-existing vendor manifest deprecation warnings remain unchanged, without suppression or vendor edits. Rust readability analysis found no threshold violations in changed functions (new query cognitive maximum 1, cyclomatic maximum 6). Retained command/output/revision records live under this worktree's ignored `target/p1200-secrets-proof/`; manual scope review confirms no transmog, vendor or page-coverage ledger changes.

## Out of scope

Native data acquisition and new secrecy producers beyond existing policies; transmog; vendor patches; source page-coverage ledger edits.
