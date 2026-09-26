# Unit GUID presence

Public `UnitGUID` exposes existing modeled identities through `src/lua_api/globals/unit_misc.rs`. See [Lua API architecture](../wiki/systems/lua-api.md).

## What it must do

- [ ] Return one nil value for absent target/focus, including after clearing an assigned unit, rather than a fabricated creature GUID.
- [ ] Return nil for unknown unit tokens and party slots removed from the active roster.
- [ ] Preserve existing player GUIDs, assigned target/focus GUIDs, and active party GUIDs.
- [ ] Let GUID-truthiness visibility guards hide frames without a modeled unit identity and show frames with a player identity.

These are bounded simulator contracts corroborated by Wowless's `data/impl/UnitGUID.lua` (`unit and unit.guid or nil`). Cached retail and Mists `Blizzard_Deprecated_ArenaUI/Deprecated_ArenaUI.lua` use `UnitGUID(self.unit)` to gate visibility. Its remote-update comment also shows why native GUID availability must not be universally equated with `UnitExists`; the simulator has no remote arena identity model. No native-client probe establishes this slice.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/lua_api/globals/unit_misc.rs`: public GUID query and shared existing-identity resolver; internal GUID construction remains unchanged.
- `src/lua_api/globals/group_queries.rs`: modeled unit-presence policy used by the existing resolver.

## Tests asserting this spec

`tests/unit_api.rs`, `test_unit_guid*`: default absence, assignment/clear, unknown-token frame visibility, roster shrink, and present-unit controls. Grouped integration target; no new Cargo target.

## Known gaps (current cycle)

- [ ] Independent GREEN and format/check/readability proof pending. Tests-only RED at `147a6a776`: four missing-identity cases fail and two present-unit controls pass; `/tmp/cross-version-unit-guid-proof.md`.
- [ ] Exact one-nil return arity and preservation of arbitrary stored target/focus GUID strings are not directly asserted by the initial regression group.

## Out of scope

New remote/arena/pet/vehicle/raid identity models, GUID-based UnitIsUnit redesign, other unit queries, coercion/error contracts, secret-value policy changes, and native or all-profile compatibility claims. Existing internal GUID construction and the separately gated aura-caster query retain their current behavior.
