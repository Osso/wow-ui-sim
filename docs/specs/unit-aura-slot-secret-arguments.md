# Unit aura slot secret arguments

Bounded Retail 12.0.5 contract for **row376 only**, `global api-C_UnitAuras-GetAuraDataBySlot-376`: `SecretArguments AllowedWhenTainted -> AllowedWhenUntainted`. Fixtures target `C_UnitAuras.GetAuraDataBySlot` over existing player/party stores; no producer changes. [Lua API system](../wiki/systems/lua-api.md) describes the runtime. Written-only batch44: parent owns compiled RED before producer, GREEN, independent gates and accounting; no passing/native parity claim.

## What it must do

### Source-grounded argument boundary

Retained [source extract](../../data/patch-api/sources/12.0.5-api-changes.txt), lines375–376, and [register](../../data/patch-api/sources/12.0.5-register.json) identify the single caller-policy delta. Source plaintext SHA256: `4da3872aa566695f46e2dacd4e79992f5b06be9541f0d19cf0e8dba45cea8329`.

Actual retail cache: `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua`, lines207–222; cached file SHA256 `39405809f92e74a945034529d2a2962ebebe9a70fe581b2f411586fc2b72d5d5`. Complete declaration: required `unit: UnitTokenRestrictedForAddOns`, **`NeverSecret = true`**; required `slot: number`; one nullable `AuraData` return; `SecretArguments = "AllowedWhenUntainted"`, `RequiresUnitAuraAccess = true`, `SecretWhenUnitAuraRestricted = true`.

- [ ] Reject authentic secret STRING unit even for secure callers: this declaration retains NeverSecret. Indexed-row unit-marker removals do not apply here. Recognize actual VM secrets; do not treat arbitrary userdata as a secret marker.
- [ ] Authenticate authentic secret NUMBER slot for untainted callers and use its actual stored ID value; tainted callers must reject secret slot before lookup, including unknown units/IDs. Do not declassify the original input or reset caller taint.
- [ ] Reject secret unit independently of slot security, including combined secret arguments and secret unknown-unit strings. Public populated/missing-result recovery must succeed inside the same tainted closure.

### Inferred representations; retained modeled lookup

- [ ] **Inferred:** require an actual string unit and finite integral signed-i32 number slot. Reject missing/nil required arguments, wrong types, numeric coercion, numeric-string slot, fractions, nonfinite and out-of-range values before lookup, including unknown unit. Native validation ordering and exact error messages are unproved; fixtures require a nonempty error, not particular wording.
- [ ] **Inferred representation/miss policy:** negative/zero/past-unknown signed-i32 slots and unknown units return exactly one nil. No invented positive-only restriction or cap. Current slots are 1:1 `AuraInfo.aura_instance_id`, not list ordinals.
- [ ] Preserve actual `GetAuraSlots` roundtrip for player/party helpful and harmful records. Current enumeration returns all visible slots in one batch with nil continuation even when requested batch size is one; this is a retained control, not new pagination/API coverage.
- [ ] Preserve blocked-inclusive slot lookup: enumeration omits blocked records but slot retrieval still finds them. Preserve existing helpful-then-harmful traversal across player buffs and seeded party buff/debuff stores, without changing shared helpers.
- [ ] Switching the AuraUtil provider suppresses its instance-ID query, not C slot retrieval; query paths must leave provider state unchanged.
- [ ] Preserve complete existing DTO fields: identity, count aliases, timing, source, polarity/player-source flags, remaining boolean flags, nilable dispel name and independent empty points table. Player DTO source remains normalized to `player` despite stored pet/party1 source; party DTO source remains stored pet/party1. DTO mutations must not affect later reads or records.

### Security lifetime and isolation

- [ ] Host-created STRING/NUMBER secrets are rooted before global insertion and survive forced GC with identical references/security. Secure → tainted denial/public recovery → secure queries preserve original input secrecy and caller taint; tainted `secretunwrap` remains denied.
- [ ] Across successes, misses, malformed inputs, secure secret slots and tainted denials, all stored aura fields/order and block-table identity/content remain unchanged. Block/provider/global secret roots remain isolated per environment.

## How it works

- [Lua API system](../wiki/systems/lua-api.md).
- [Aura classification flags](aura-classification-flags.md): existing DTO context.
- [Indexed aura arguments](unit-aura-index-secret-arguments.md): separate contract; unit NeverSecret removal does **not** transfer to slots.

## Implementation inventory

Unchanged inventory at fixture base `04cf872ce`:

- `src/lua_api/globals/register.rs`: existing aura namespace registrar.
- `src/lua_api/globals/auras.rs`: `get_aura_data_by_slot` currently defaults/coerces unit/slot then calls `push_aura_by_instance_id` → `find_aura_by_instance_id`; blocked-inclusive helpful-then-harmful collection and `build_aura_table` serialization remain unchanged. `GetAuraSlots` uses visible collection; AuraUtil alone checks provider switch.
- `tests/unit_aura_slot_secret_arguments.rs`: new epoch-gated fixtures, automatically discovered by existing grouped integration harness; no Cargo/build/registration edits.
- `docs/specs/unit-aura-slot-secret-arguments.md`: row identity, contract, inference labels and proof boundary.

## Tests asserting this spec

`tests/unit_aura_slot_secret_arguments.rs`: **12 concrete fixtures**, existing `integration` target, filter `unit_aura_slot_secret_arguments::`, requires `retail-12-0-5`. No builds/tests/gates run in this fixture-only slice.

| Fixture(s) | Exact coverage | Proof level |
| --- | --- | --- |
| `player_enumerated_instance_slots_keep_full_dto_and_single_batch`, `party_enumerated_helpful_harmful_slots_keep_full_dto` | Actual enumerated IDs101/102/103/104 and201/202/203/204, both polarities, nonordinal/cross-unit misses, full DTO incl. dispel nilability, nil continuation/current one batch | Written only; retained controls |
| `blocked_records_disappear_from_enumeration_but_remain_slot_retrievable`, `switched_aura_util_provider_does_not_disable_c_slot_lookup`, `valid_signed_i32_misses_return_exactly_one_nil_without_positive_cap` | Blocked101/104/201/204, visible later slots, switched AuraUtil control, signed extremes/zero/unknown exact nullable arity | Written only; retained lookup plus inferred miss domain |
| `required_unit_is_actual_string_without_default_or_numeric_coercion`, `required_slot_is_finite_integral_signed_i32_before_unknown_lookup` | Required/missing/nil/types/coercion, fractions/nonfinite/range, validation before unknown-unit result | Written only; representation inferred |
| `secure_authentic_secret_slot_numbers_use_actual_stored_ids`, `never_secret_unit_rejects_authentic_secret_strings_even_when_secure`, `tainted_secret_denials_precede_lookup_and_public_recovery_preserves_taint`, `gc_rooted_secret_identity_survives_secure_tainted_public_secure_roundtrip` | Authentic secret IDs101/103/201/203/99999, secure populated/missing lookup, NeverSecret unit secure+tainted rejection, same-closure public recovery, input/taint preservation, rooted GC identity | Written only; source-grounded argument policy |
| `query_paths_preserve_records_dto_block_provider_and_environment_isolation` | All stored fields/order, independent DTO/points mutation, block identity/content, provider state, per-env isolation across public/error/secret/tainted paths | Written only; retained-state assertions |

## Known gaps (current cycle)

- [ ] Parent compiled RED before producer; fixture correctness and genuine boundary failures are not established yet.
- [ ] Parent producer/GREEN, independent security/readability/acceptance gates and exact-row accounting remain pending. No existing batch43 proof transfers to these fixtures.

## Out of scope

- Native `RequiresUnitAuraAccess` permission and `SecretWhenUnitAuraRestricted` conditional output policy: backing restriction/access state unmodeled. Public modeled DTO checks do not prove permission enforcement or restricted-output secrecy.
- Indexed/display/duration producers, shared lookup/DTO/store changes, registration/Cargo/build changes, batching redesign, invented visibility predicates, native error/parity and all-profile coverage.
- Batch43 source/spec/wiki/accounting, PLAN, unowned `src/c_api/aura_duration.rs` changes, push/deploy/operations. This slice changes only the two new fixture/spec files; parent owns remaining proof and credit.
