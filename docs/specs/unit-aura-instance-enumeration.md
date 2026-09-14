# Unit aura instance enumeration

Retail 12.1 aura containers enumerate public and private aura instance IDs before fetching aura data. Native API implementations live in `src/c_api/c_unit_auras.rs`, reusing the existing public aura filter/block model and private-aura store.

## What it must do

- [x] `C_UnitAuras.GetUnitAuraInstanceIDs(unit, filter, maxCount?, sortRule?, sortDirection?)` returns one numeric-ID array, including an empty array for unknown units. It follows current aura state, polarity/player filters, blocked-aura visibility, documented sorting, and result limits.
- [x] `C_UnitAurasPrivate.GetAllPrivateAuraInstanceIDs(unit)` returns a fresh ID array from the existing private-aura list, without applying public filters.
- [x] Preserve Blizzard source-wrapper semantics: public enumeration returns `hasMatchedFilterString=true`; private enumeration returns `false`. This boolean is not an additional native API return value.
- [x] Publish `Enum.UnitAuraSoundTrigger` (`Added=0`, `ApplicationsIncreased=1`, `Removed=2`) and matching metadata before the secure environment copies enums.

## How it works

- [Lua API](../lua-api.md)
- [Frame data flow](../frame-data-flow.md)

## Implementation inventory

- `src/c_api/c_unit_auras.rs` — ID queries, query options, sort rules, and native sound-trigger enum.
- `src/lua_api/globals/auras.rs` — existing modeled public aura sources, filter predicates, and blocked-aura state; shared filtered query adapter.
- `src/lua_api/globals/register.rs` and `env_init/enums.rs` — namespace and pre-secure-copy enum registration.
- `src/lua_api/workarounds/temporary/private_aura_state.rs` — existing private-aura store; its model is not replaced by this change.

## Tests asserting this spec

`tests/aura_instance_ids.rs` covers concrete seeded player/target results, blocked/player filtering, state mutations, limits and each documented sort rule, private list copies, real Blizzard source-wrapper return flags, and public/secure enum values.

The two ordinary public-query tests are enabled from `retail-12-0-0`; private-query, source-wrapper and secure-enum tests remain gated at `retail-12-1-0`. Historical 12.0.0 execution is pending; test bodies and runtime behavior are unchanged.

## Known gaps (current cycle)

Focused development tests passed 5/5 after implementation `31b0e0f95`; logs `/tmp/pi-aura-instance-ids-green-fixed-build.*`. Full-addon acceptance and independent final checks remain with the integration task.

## Out of scope

New aura acquisition, new unit types, private-aura service/event generation, or secret-value restrictions. Enumeration inherits the existing public/private data models; it does not turn missing data into fabricated auras.

## Native sources

Retail cached `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua` defines the public query and arguments; `UnitAuraSharedDocumentation.lua` defines sort behavior; `UnitAuraConstantsDocumentation.lua` defines sound-trigger literals/metadata. `Blizzard_AuraContainer/Blizzard_AuraContainerSources.lua` defines the source-wrapper matched-filter flags and private-query use.
