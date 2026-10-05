# C_Secrets publication queries

The Retail publication queries report existing simulator secrecy policies. Publication is not proof of full native secrecy coverage. [Contract and known gaps](../../specs/secrets-publication-queries.md) owns that distinction.

## Shared state

| Query family | Source |
|---|---|
| Aura global/index/instance/slot | `unit_aura_access::auras_restricted`, including epoch gate and `unit_auras_restricted` |
| Spell aura | `unit_aura_access::spell_keyed_aura_is_secret`, shared with the output finisher and the existing never-secret spell dataset |
| Spell/action cooldown | `charge_state::cooldowns_are_restricted`, including epoch/profile gates and `cooldowns_restricted` |
| Book cooldown | Same cooldown predicate plus `cooldown_spell_for_book_entry`, shared with the actual book cooldown API |
| Unit identity | Existing `unit_misc::unit_identity_is_secret`: GUID overrides and instanced-map/group/player-owned exemptions |

Queries authenticate original selectors, then compute public booleans/enums without clearing taint. Aura selectors report the pre-lookup access policy even for absent selectors, matching the corresponding access boundary; this interpretation is INFERRED. Action cooldown reports restriction even for an empty-slot zero-duration DTO because the current provider wraps its numeric fields.

`HasSecretRestrictions` describes build capability, not whether any current restriction is active. Native power/cast secrecy attributes and several secrecy producers are absent. Their explicitly INFERRED defaults remain bounded to current public output; see the contract's known gaps. No new restriction state or alternate Blizzard behavior is synthesized.

## Evidence

Cached contract is `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SecretPredicateAPIDocumentation.lua`, with `SecretPredicatesDocumentation.lua` and `SecretWrapperConstantsDocumentation.lua`. The supplied `SecretsDocumentation.lua` path is absent. Existing cast readers support only player data; health/power outputs are public; `UnitIsUnit` implements comparability without a secret-output policy; `GetTotemInfo` has only an empty active-slot default.

## Sources

- [Contract](../../specs/secrets-publication-queries.md)
- [State-flip tests](../../../tests/secrets_publication_queries.rs)
- [Registration](../../../src/c_api/secrets_queries.rs)

## See Also

- [[patch-12-0-0-api-audit]] — publication sweep, not native behavioral closure
