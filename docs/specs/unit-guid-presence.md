# Unit GUID presence

Public `UnitGUID` exposes existing modeled identities through `src/lua_api/globals/unit_misc.rs`. See [Lua API architecture](../wiki/systems/lua-api.md).

## What it must do

- [x] Return nil for absent target/focus, including after clearing an assigned unit, rather than a fabricated creature GUID.
- [x] Return nil for unknown unit tokens and party slots removed from the active roster.
- [x] Preserve exact player/active-party GUIDs and assigned target/focus GUID families; targeting controls retain GUID-based aliases.
- [x] Let GUID-truthiness visibility guards hide frames without a modeled unit identity and show frames with a player identity.

These are bounded simulator contracts corroborated by Wowless's `data/impl/UnitGUID.lua` (`unit and unit.guid or nil`). Cached retail and Mists `Blizzard_Deprecated_ArenaUI/Deprecated_ArenaUI.lua` use `UnitGUID(self.unit)` to gate visibility. Its remote-update comment also shows why native GUID availability must not be universally equated with `UnitExists`; the simulator has no remote arena identity model. No native-client probe establishes this slice.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/lua_api/globals/unit_misc.rs`: public GUID query and shared existing-identity resolver; internal GUID construction remains unchanged.
- `src/lua_api/globals/group_queries.rs`: modeled unit-presence policy used by the existing resolver.

## Tests asserting this spec

`tests/unit_api.rs`, `test_unit_guid*`: default absence, assignment/clear, unknown-token frame visibility, roster shrink, and present-unit controls. Grouped integration target; no new Cargo target.

## Known gaps (current cycle)

Independent verification of `2938de6e1` passes 79 unit API cases (six GUID cases), 24 targeting cases plus one nested consumer, and two retained nested-timer controls; format/check and changed-function readability pass. Exact revisions and logs: `/tmp/cross-version-unit-guid-verification-ledger.md`. Tests-only RED at `147a6a776`: four missing-identity failures and two present-unit controls; `/tmp/cross-version-unit-guid-proof.md`.

- [ ] The targeting consumer fixture emitted 12 Lua-error lines concerning missing Blizzard dependencies while its assertions passed. Their provenance is unestablished; this is not a clean full-UI startup claim.
- [ ] Exact one-nil return arity and preservation of arbitrary stored target/focus GUID strings are not directly asserted by the initial regression group.

## Out of scope

New remote/arena/pet/vehicle/raid identity models, GUID-based UnitIsUnit redesign, other unit queries, coercion/error contracts, secret-value policy changes, and native or all-profile compatibility claims. Existing internal GUID construction and the separately gated aura-caster query retain their current behavior.
