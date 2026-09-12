# Unit raid-target icons

Simulator state for `SetRaidTarget`, standalone `SetRaidTargetIcon`, and `GetRaidTargetIndex`. It is GUID-keyed unit-icon state, independent of world markers. `104cb6fc9` added the model; focused GREEN and independent verification remain pending.

## Model

`SimState.unit_raid_target_icons` maps a resolved unit GUID to icon indices 1–8. `SetRaidTarget` resolves the unit first, then accepts integral finite indices 0–8: 0 clears the resolved unit; 1–8 assigns the icon. Assignment removes the target's prior icon and any prior owner of the requested icon. Valid calls synchronously dispatch one zero-payload `RAID_TARGET_UPDATE` after mutation, including repeated assignments. No event is queued for later delivery.

`GetRaidTargetIndex` resolves the same unit snapshot and returns its index or nil. Missing or unknown units keep the legacy no-op/nil boundary. Valid units with missing, invalid, fractional, nonfinite, or out-of-range indices fail before mutation or dispatch.

Targeting snapshots now reuse `unit_misc::guid_for_unit` for player and party GUIDs. This aligns marker identity with `UnitGUID`; target and focus aliases therefore share an assignment when they resolve to the same GUID. Target switching does not transfer an icon.

## Vendor interaction

Retail `TargetFrameMixin:UpdateRaidTargetIcon()` reads `GetRaidTargetIndex(self.unit)`, sets the target-icon sprite cell, and shows or hides its texture on `RAID_TARGET_UPDATE`. Loaded Blizzard Lua replaces the simulator's standalone `SetRaidTargetIcon` alias with a toggle wrapper: selecting the already selected icon calls `SetRaidTarget(unit, 0)`. The simulator does not overwrite that vendor behavior.

## Boundaries

The index mapping, one-icon-per-unit/one-unit-per-icon invariant, validation, notification timing, and repeated-call notification are explicit simulator policies. They are not native permission, group/combat eligibility, lifecycle, coalescing, error-wording, or persistence claims.

World-marker APIs—including `PlaceRaidMarker`, `ClearRaidMarker`, `IsRaidMarkerActive`, and `RemoveRaidTargets`—are separate and unchanged.

## Sources

- [unit raid-target icon contract](../../specs/unit-raid-target-icons.md) — bounded requirements and exclusions.
- [targeting implementation](../../../src/lua_api/globals/targeting_verbs.rs) — token resolution, mutation, and event dispatch.
- [state model](../../../src/lua_api/state/sim_state.rs) — GUID-keyed assignments.
- [unit identity helper](../../../src/lua_api/globals/unit_misc.rs) — existing `UnitGUID` conventions.

## See Also

- [[patch-12-0-0-api-audit]] — audit classification remains unchanged pending proof.
- [[patch-api-audit-manifest]] — manifest evidence rules.
