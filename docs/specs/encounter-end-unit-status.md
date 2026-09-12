# Encounter-end unit status

`A_Admin.SimulateBossKill` supplies explicit non-secret encounter-end snapshots for the 12.0.7 `ENCOUNTER_END.encounterUnitStatus` addition. The pinned [API-change snapshot](../../data/patch-api/sources/12.0.7-api-changes.txt) names the list and its three fields. This is simulator input, not native encounter tracking.

## What it must do

- [ ] On retail 12.0.7 and later epochs, emit six `ENCOUNTER_END` arguments followed by the existing two-argument `BOSS_KILL` event.
- [ ] Preserve encounter ID, name, difficulty, group size, and success (`1`).
- [ ] Accept an optional fifth admin argument: an ordered dense list of `{creatureID, creatureName, remainingHealthPercent}` records. Copy the records before dispatch; later changes to the caller's list do not change the event snapshot.
- [ ] Omitted or nil status produces a fresh empty list, not inferred boss records.
- [ ] Reject malformed input before either event. Simulator validation requires positive integral finite creature IDs, string names, and finite health percentages from 0 through 100; this validation is not a claim about native error behavior.
- [ ] Earlier epochs retain the five-argument event and existing four-argument admin call. The additional admin input is unused on those profiles.

## How it works

- [12.0.7 audit](../wiki/investigations/patch-12-0-7-api-audit.md)
- [Event dispatch](../event-system.md)

## Implementation inventory

- `src/lua_api/globals/admin_encounter.rs`: simulator input and ordered event dispatch.
- `src/lua_api/globals/admin_encounter/unit_status.rs`: validation and snapshot copying.

## Tests asserting this spec

- `tests/admin_encounter_api.rs`: exact arity/order, populated snapshots, input isolation, empty lists, atomic rejection, and earlier-profile control.

## Known gaps (current cycle)

- [ ] Focused post-implementation verification and historical-profile control.

## Out of scope

- Native boss engagement tracking, encounter lifecycle, success/failure determination, and health sampling: callers supply explicit records.
- Native timing, validation, security, and secret payload semantics: not established by simulator tests.
- Blizzard consumer fidelity: no consuming path for the new field established in the current cached UI.
