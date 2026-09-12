# Unit raid target icons

`SetRaidTarget`, the standalone `SetRaidTargetIcon` alias, and `GetRaidTargetIndex` expose bounded simulator unit-icon state in `src/lua_api/globals/targeting_verbs.rs`. These are unit icons, not world markers. The following mutation, validation, and notification rules are explicit simulator policies, not claims of native conformance.

## What it must do

- [x] Assign integer indices 1–8, replace an existing assignment, and clear with 0. Each unit has at most one icon; each icon belongs to at most one unit. Assigning an occupied icon removes its prior assignment.
- [x] Key assignments by resolved GUID. Target/focus aliases share assignments; switching target does not transfer icons. Player and party targeting snapshots use the existing `UnitGUID` identity convention, including nearest-friend selection.
- [x] Return exactly one number or nil from the getter and no results from the setter. Nil/unknown units are unmarked and their setters remain no-ops, without validating the icon argument.
- [x] Reject missing, non-number, nonfinite, fractional, or out-of-range indices for valid units before mutation or notification.
- [x] Preserve one zero-payload queued `RAID_TARGET_UPDATE` record and one synchronous notification after every valid explicit call, including repeated same-index calls. Both follow mutation; callbacks see updated state. Queue consumption does not imply automatic callback dispatch.

## How it works

- [Event dispatch](../event-system.md)
- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/lua_api/globals/targeting_verbs.rs` — unit resolution, aligned targeting snapshot GUIDs, icon query/mutation, and synchronous notification.
- `src/lua_api/state/sim_state.rs` — GUID-keyed icon assignments.
- `src/lua_api/state.rs` — empty initial assignments.

## Tests asserting this spec

- `tests/targeting_verbs.rs` — assignment/moving/replacement/clearing, alias identity/isolation, invalid input, callback-visible state, and preserved queued records consumed through the public drain API without re-dispatch; existing targeting controls remain intact.

## Known gaps (current cycle)

- [ ] Independent final verification of restored queue recording. Prior icon/sprite/profile proof does not establish the restored event side effects; the two corrected event tests have separate development evidence in `/tmp/raid-icon-event-records-ledger.json`.
- [ ] Actual unmodified Blizzard consumer proof, arranged separately by the parent.

## Out of scope

- World-marker APIs and `RemoveRaidTargets`: separate contracts, unchanged.
- `CanBeRaidTarget` eligibility: existing token-existence policy remains unchanged.
- Permission, combat/group restrictions, security, native event timing/coalescing, and native validation/error wording: no retained conformance evidence is claimed.
- GUID lifecycle redesign, party removal/rejoin identity, and persistence: existing simulator identity conventions are reused.
- Blizzard's loaded `SetRaidTargetIcon` override: vendor code may toggle an already-selected icon off. The simulator retains its standalone alias but does not replace the vendor override after loading.
