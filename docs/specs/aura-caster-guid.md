# PTR aura caster GUID

`C_UnitAuras.GetAuraCasterGUID` queries the caster of an aura on a specific unit. The pinned contract is in [the 12.1.5 register](../../data/patch-api/sources/12.1.5-register.json); implementation is in [c_unit_auras.rs](../../src/c_api/c_unit_auras.rs).

## What it must do

- [ ] PTR publishes `GetAuraCasterGUID(auraInstanceUnit, auraInstanceID)`; earlier retail preserves absence, including namespace fallback and post-load bootstrap.
- [ ] Required unit and numeric instance arguments use existing bridge validation. Numeric IDs use the existing aura-query conversion to `i32`.
- [ ] Look up the aura through the shared per-unit instance lookup, including helpful and harmful auras; identical IDs on different units do not alias.
- [ ] Return exactly one GUID string or nil. Missing units, missing auras, and unresolved source tokens return nil.
- [ ] Resolve `source_unit` through the existing unit GUID model at query time. Player and party sources match `UnitGUID`; changing or removing the target source is reflected without replacing the aura.

Live source-token resolution is a simulator choice, not a caster-identity snapshot. An aura whose source token later refers to another unit therefore reports that token's current GUID. Unit existence follows the existing `UnitExists` model; tokens without a modeled GUID return nil rather than the resolver's unknown-creature sentinel.

## How it works

- [Lua API architecture](../lua-api.md)
- [Unit aura instance enumeration](unit-aura-instance-enumeration.md)

## Implementation inventory

- `src/c_api/c_unit_auras.rs`: PTR registration, required arguments, nullable result.
- `src/lua_api/globals/auras.rs`: shared aura-instance lookup.
- `src/lua_api/globals/group_queries.rs`: shared existing-unit state predicate, also used by `UnitExists`.
- `src/lua_api/globals/unit_misc.rs`: live resolution through `guid_for_unit`, excluding unresolved GUIDs.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/patch_12_1_5_aura_caster_guid.rs`: player/party sources, unit and instance isolation, nil/error behavior, live target identity, earlier-retail absence.
- `tests/aura_instance_ids.rs` and `tests/c_unit_auras_admin.rs`: existing shared aura query regressions.

## Known gaps (current cycle)

- [ ] Native `RequiresUnitAuraAccess` and `RequiresValidUnitAuraInstance` preconditions are not enforced by this slice.
- [ ] Native `SecretArguments = AllowedWhenTainted` and `SecretWhenUnitAuraRestricted = true` are unverified and not implemented here.
- [ ] Native missing-instance behavior, identity lifetime, coercion and error details remain unverified; documented nil/validation behavior is simulator policy.

## Out of scope

Snapshot caster identity, additional GUID models, aura-filter changes, protected/tainted/secret-value enforcement, and native edge behavior require separate work.
