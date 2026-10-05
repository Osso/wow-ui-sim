# C_Secrets publication queries

The 20 assigned Retail 12.0.0 queries describe the simulator's existing secrecy policies. Authoritative cached signatures live in `SecretPredicateAPIDocumentation.lua`; the requested `SecretsDocumentation.lua` does not exist. [Implementation and boundaries](../wiki/systems/secrets-publication-queries.md).

## What it must do

- [ ] Publish all 20 assigned queries from the Retail 12.0.0 epoch.
- [ ] Return public query results without changing caller taint; authenticate every selector under `AllowedWhenUntainted` before policy lookup.
- [ ] Describe aura access and spell exemptions using exactly the predicates used by `C_UnitAuras`.
- [ ] Describe spell/action/book cooldown outputs using the shared cooldown restriction and actual book-entry resolution.
- [ ] Describe identity outputs using existing live identity classification, including explicit GUID overrides and instance exemptions.
- [ ] INFERRED bounded defaults for unmodeled aspects must describe current public outputs, not fabricate a restriction from unrelated combat/stat/aura state.

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

## Out of scope

Native data acquisition and new secrecy producers beyond existing policies; transmog; vendor patches; source page-coverage ledger edits.
