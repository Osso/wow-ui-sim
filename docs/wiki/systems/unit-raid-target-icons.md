# Unit raid-target icons

Simulator state for `SetRaidTarget`, standalone `SetRaidTargetIcon`, and `GetRaidTargetIndex`. It is GUID-keyed unit-icon state, independent of world markers. `b269650ae` credits bounded best-effort `SetRaidTarget` behavior and expands existing `GetRaidTargetIndex` credit. The final runtime is `5bc5cdb6f`; metadata `8d1ad818c` remains unchanged through documentation commit `9a2892eec`.

## Model

`SimState.unit_raid_target_icons` maps a resolved unit GUID to icon indices 1–8. `SetRaidTarget` resolves the unit first, then accepts integral finite indices 0–8: 0 clears the resolved unit; 1–8 assigns the icon. Assignment removes the target's prior icon and any prior owner of the requested icon. Valid calls record one zero-payload `RAID_TARGET_UPDATE` and synchronously dispatch one notification after mutation, including repeated assignments. Draining records does not automatically dispatch callbacks.

`GetRaidTargetIndex` resolves the same unit snapshot and returns its index or nil. Missing or unknown units keep the legacy no-op/nil boundary. Valid units with missing, invalid, fractional, nonfinite, or out-of-range indices fail before mutation or dispatch.

Targeting snapshots now reuse `unit_misc::guid_for_unit` for player and party GUIDs. This aligns marker identity with `UnitGUID`; target and focus aliases therefore share an assignment when they resolve to the same GUID. Target switching does not transfer an icon.

## Vendor interaction

Retail `TargetFrameMixin:UpdateRaidTargetIcon()` reads `GetRaidTargetIndex(self.unit)`, calls `Texture:SetSpriteSheetCell(index, 4, 4)`, and shows or hides its texture on `RAID_TARGET_UPDATE`. Focused proof loaded the complete unmodified `Mainline/TargetFrame.lua`, dispatched the event, and observed cells 1 and 8 plus the vendor toggle wrapper. The normal historical XML path still aborts on unsupported `AuraContainer`, so this proves the Lua consumer boundary only—not XML construction, clean addon loading, or full UI integration. Loaded Blizzard Lua replaces the simulator's standalone `SetRaidTargetIcon` alias with a toggle wrapper: selecting the already selected icon calls `SetRaidTarget(unit, 0)`. The simulator does not overwrite that vendor behavior.

## Verification

Historical 12.0.0 development proof passed 23 core icon tests, three sprite tests, and one original-Lua consumer test; two corrected queued-record tests were run separately after `5bc5cdb6f`. Independent proof passed corrected record plus consumer cases 3/3 on 12.0.7 and record cases 2/2 on Mists. The earlier 12.0.5/12.0.7 51-case runs and Mists ordinary controls are reused only for assertions unaffected by restored queue records. Fresh format, check, build, and current standalone startup (`[]`) passed. No new readability issue was found; the `state.rs` length threshold was pre-existing.

Historical consumer closures still emit 71 known Lua-error headers, unchanged from the earlier consumer proof. Current consumer closures have their own known errors and do not alter the standalone-startup result.

## Boundaries

The index mapping, one-icon-per-unit/one-unit-per-icon invariant, validation, notification timing, repeated-call notification, and sprite-sheet row-major indexing are explicit simulator policies. `SetSpriteSheetCell` supports only the observed three-argument grid form; nonnil optional `cellWidth`/`cellHeight` are explicitly unsupported. These are not native permission, group/combat eligibility, lifecycle, coalescing, error-wording, persistence, optional-dimension, or pixel-cropping claims.

World-marker APIs—including `PlaceRaidMarker`, `ClearRaidMarker`, `IsRaidMarkerActive`, and `RemoveRaidTargets`—are separate and unchanged.

## Sources

- [unit raid-target icon contract](../../specs/unit-raid-target-icons.md) — bounded requirements and exclusions.
- [targeting implementation](../../../src/lua_api/globals/targeting_verbs.rs) — token resolution, mutation, and event dispatch.
- [state model](../../../src/lua_api/state/sim_state.rs) — GUID-keyed assignments.
- [unit identity helper](../../../src/lua_api/globals/unit_misc.rs) — existing `UnitGUID` conventions.

## See Also

- [sprite-sheet cell contract](../../specs/sprite-sheet-cell.md) — bounded coordinate model used by the consumer.
- [[patch-12-0-0-api-audit]] — exactly SetRaidTarget and TextureBase.SetSpriteSheetCell gained bounded credit; world-marker and RemoveRaidTargets rows remain evidence-required.
- [[patch-api-audit-manifest]] — manifest evidence rules.
