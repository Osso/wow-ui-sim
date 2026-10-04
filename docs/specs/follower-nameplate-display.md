# Follower player-display classification

Bounded host-backed input for retail 12.0.5 prose row `prose-2026-03-31-174` in [patch source](../../data/patch-api/sources/12.0.5-api-changes.txt). This slice exposes follower classification through `UnitTreatAsPlayerForDisplay`; it does **not** establish that loaded nameplates consume both CVars successfully. Row-level completion remains blocked by the unsupported live-nameplate lifecycle/hit-test boundary documented in the authoring handoff. No executed proof yet.

## What it must do

- [ ] Explicit host follower GUID input changes a resolved NPC's display classification without changing `UnitIsPlayer`, `UnitIsHumanPlayer`, class or friendliness.
- [ ] Classification follows resolved target/focus GUID identity, does not follow target-slot replacement, and does not create absent units.
- [ ] Clearing the host input clears the next observable classification; environments remain independent.
- [ ] Public calls preserve secure/addon caller taint. Cached `AllowedWhenUntainted` authenticates secret arguments before state access; authenticated host-secret secure/addon/GC proof is authored but unexecuted.
- [ ] INFERRED: nil/unknown tokens return false; wrong public type errors; unmodeled pet/vehicle tokens are not fabricated display players. Existing player identity remains eligible.

## How it works

- [Lua API/state architecture](../lua-api.md)
- [Widget architecture](../widget-system.md)

## Implementation inventory

- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: per-environment follower GUID input and empty initialization.
- `src/lua_api/globals/real/nameplate_display.rs`: state-backed non-C global output and argument authentication.
- `src/lua_api/globals/real/mod.rs`: retail feature module.
- `src/lua_api/globals/group_queries.rs`: retail producer routing; prior behavior retained only for older feature sets.
- `src/lua_api/globals/unit_probes.rs`: shares the existing player-identity resolver, without changing either identity API.

## Tests asserting this spec

- `tests/follower_nameplate_display.rs`: seven behavioral tests auto-included in integration.

## Known gaps (current cycle)

- [ ] Main must execute RED with state/tests applied and producers withheld, then GREEN with producers applied.
- [ ] Authored secret-input roundtrip needs main's executable native-VM proof; authentication code alone earns no security acceptance.
- [ ] Unchanged loaded nameplate classification, class-color FontString output, name-only visibility, CVAR_UPDATE callbacks, and reversible CVar changes remain unproven.

## Out of scope

- Live nameplateN entity creation, 3D placement and native nameplate lifecycle: current C_NamePlate subsystem intentionally exposes no plates.
- Party follower/human identity remodeling: current PartyMember entries are human players by definition.
- EditMode secure delegation: no safe generic delegate trust boundary authored in this slice.
- Native-client parity, full-row/page acceptance, all-profile claims.
