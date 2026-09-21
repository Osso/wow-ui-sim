# Unit aura instance enumeration

Public aura instance enumeration is published from Retail 12.0.0 and on Forever through `aura-instance-enumeration`. Retail 12.1+ and Forever enable `aura-containers`, which adds private enumeration. Both native aura containers enumerate public and private aura instance IDs before fetching aura data. API implementations live in `src/c_api/c_unit_auras.rs`, reusing the existing public aura filter/block model and private-aura store.

## What it must do

- [x] `C_UnitAuras.GetUnitAuraInstanceIDs(unit, filter, maxCount?, sortRule?, sortDirection?)` returns one numeric-ID array, including an empty array for unknown units. It follows current aura state, polarity/player filters, blocked-aura visibility, documented sorting, and result limits.
- [x] `C_UnitAurasPrivate.GetAllPrivateAuraInstanceIDs(unit)` returns a fresh ID array from the existing private-aura list, without applying public filters.
- [x] Preserve Blizzard source-wrapper semantics: public enumeration returns `hasMatchedFilterString=true`; private enumeration returns `false`. This boolean is not an additional native API return value.
- [x] On Retail 12.1+, publish `Enum.UnitAuraSoundTrigger` (`Added=0`, `ApplicationsIncreased=1`, `Removed=2`) and matching metadata before the secure environment copies enums.

## How it works

- [Lua API](../lua-api.md)
- [Frame data flow](../frame-data-flow.md)

## Implementation inventory

- `src/c_api/c_unit_auras.rs` — ID queries, query options, sort rules, and native sound-trigger enum.
- `src/lua_api/globals/auras.rs` — existing modeled public aura sources, filter predicates, and blocked-aura state; shared filtered query adapter.
- `Cargo.toml` — Retail 12.0 and `aura-containers` enable public enumeration; only `aura-containers` enables private enumeration.
- `src/lua_api/globals/register.rs` — modeled registration after base aura registration, before secure-environment copying; public/secure namespaces retain identity.
- `src/lua_api/env_init/enums.rs` — unchanged Retail 12.1 sound-trigger publication.
- `src/lua_api/workarounds/temporary/private_aura_state.rs` — existing private-aura store; its model is not replaced by this change.

## Tests asserting this spec

`tests/aura_instance_ids.rs` covers concrete seeded player/target results, blocked/player filtering, state mutations, limits and each documented sort rule, private list copies, real Blizzard source-wrapper return flags, and public/secure enum values.

Public-query tests follow `aura-instance-enumeration`; private-query, native source-wrapper and public/secure admin lifecycle tests follow `aura-containers`. The sound-trigger test remains Retail 12.1-only. Historical tests at `33f7fdb39` reached RED 0/2: the first public query returned a non-array because the entire module and registration call were gated at 12.1.0, despite the query body being ungated. Publication now starts at 12.0.0; the enumeration algorithm and test bodies are unchanged. The first publication build (`1ac925277`) exposed a further 12.1.0 gate on `collect_filtered_unit_auras` (E0432). The next build (`02ae25971`) exposed the adapter's gated `aura_matches_filter_string` dependency (E0425). Both helpers are now gated at 12.0.0 without changing their bodies; `IsAuraFilteredOutByInstanceID` publication remains at 12.1.0.

Independent proof at `ba31c6b94` passes the two ordinary tests on retail 12.0.0/12.0.5/12.0.7 (2/2 each), and the five later-gated regression tests on default features (5/5), plus fmt/check/build/startup (`[]`)/readability. Final metadata proof at `e9f56b974` records 15,056 fresh hashes, zero stale, 59 renewals, five additions, one credit, validator exit 0 and all 3,410 rows matching. The credit covers only public ordinary filtering, live updates, sort rules and limits; native filter domains, ordering, defaults, identity, security and full-LoD behavior remain unproven.

Private enumeration and its existing store readers follow `aura-containers`. The pre-12.1.5 caster-GUID tombstone and sound-trigger enum remain Retail 12.1-only; `GetAuraCasterGUID` remains Retail 12.1.5-only. Sharing enumeration does not publish those later contracts to Forever.

## Known gaps (current cycle)

Historical development tests passed 5/5 after `31b0e0f95`; logs `/tmp/pi-aura-instance-ids-green-fixed-build.*`. Forever runtime RED records an injected helpful aura (instance 7, spell 19750), ten `pball|-` frames, and both native source queries returning nil. Direct public/secure queries resolve to the same namespaces but both remain nil-returning stubs because modeled registration was Retail-only. Evidence: `/tmp/ellesmere-forever/runtime-data-trace-ledger.json` and `aura-enumeration-publication-trace-retry.stderr`.

The sharing change adds an isolated admin add/remove fixture with concrete instance 1, spell 19750, icon 135907 and three applications; both environments must enumerate it and return empty harmful/private arrays. Existing populated private-store fixtures prevent replacing private enumeration with a constant empty array. Compiled GREEN and final checks are pending integration; no Cargo was run for this slice.

## Out of scope

New aura acquisition, new unit types, private-aura service/event generation, or secret-value restrictions. Enumeration inherits the existing public/private data models; it does not turn missing data into fabricated auras.

## Native sources

Retail and Forever cached `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua` define the public query and arguments; `UnitAuraSharedDocumentation.lua` defines sort behavior; `UnitAuraConstantsDocumentation.lua` defines sound-trigger literals/metadata. `Blizzard_AuraContainer/Blizzard_AuraContainerSources.lua` defines the source-wrapper matched-filter flags and private-query use.
