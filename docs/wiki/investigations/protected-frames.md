# Protected Frames

Protected frame enforcement: blocks insecure addon code from moving, showing, or modifying protected frames during combat.

## Enforcement Rules

A method call is blocked when **all three** are true:
1. The calling code is insecure (addon code — `issecure()` returns false)
2. `InCombatLockdown()` is true (between `PLAYER_REGEN_DISABLED` and `PLAYER_REGEN_ENABLED`)
3. The target frame is protected, or an ancestor/descendant/anchor-relative of a protected frame

Blocked calls: silently no-op and fire `ADDON_ACTION_BLOCKED`. Blizzard secure code can still call these methods during combat.

## Covered Methods

- Anchor/movement: `SetPoint`, `ClearAllPoints`, `AdjustPointsOffset`, `SetPointsOffset`, `StartMoving`, `StopMovingOrSizing`
- Visibility: `Show`, `Hide`, `SetShown`
- Hierarchy/size: `SetParent`, `SetSize`, `SetWidth`, `SetHeight`
- Strata/level: `SetFrameLevel`, `SetFrameStrata`, `SetFixedFrameLevel`, `SetFixedFrameStrata`, `SetToplevel`
- Other: `SetClampedToScreen`, `SetHitRectInsets`, `SetScrollChild`, `SetHyperlinksEnabled`, `SetPropagateKeyboardInput`, `SetForbidden`

## Offset mutation audit — `ad01e3d8e`

`AdjustPointsOffset` and `SetPointsOffset` now use the existing protected-frame write policy. For insecure combat calls on protected or protected-anchor-related frames, each leaves offsets unchanged and emits `ADDON_ACTION_BLOCKED` naming the attempted method. The same calls remain allowed for a secure caller in combat and an insecure caller out of combat; ordinary plain-frame calls remain allowed.

`/tmp/cross-version-anchor-protection-proof.md` records a RED 0/1 runtime reproduction before the production edit, then GREEN 3/3 for the `protected_anchor` selection plus two 1/1 existing secure/out-of-combat controls. The paired compile logs pass before RED and after GREEN. The ledger explicitly limits this to the scoped source before commit `ad01e3d8e`; unrelated `tests/spacing_roundtrip.rs` changes were present only during GREEN compilation. Independent verification subsequently passes 13 protected-frame, 33 anchor-method and 47 security cases, plus format/check. The combined FontObject snapshot follow-up passes 16 focused cases and its new readability findings are resolved. Exact evidence: `/tmp/cross-version-anchor-font-proof.md`; no broad suite or deployment claim.

This records simulator policy and source/runtime coverage only. It makes no native-client claim and does not state `ClearPointsOffset` reset semantics.

## Remaining Gaps

**No-op stubs needing real implementations before enforcement matters:**
- `SetClampRectInsets`, `SetUsingParentLevel`, `StartSizing`

**Read-only restricted APIs not yet enforced:**
- `GetRect`, `GetLeft`, `GetPoint`, `GetBoundsRect`

## Open Questions

Exact live-WoW error behavior is unknown:
- Does blocked call silently no-op, or raise a Lua error?
- Is `[ADDON_ACTION_BLOCKED]` sent to the UI error handler or just chat?
- Does it fire a specific event?

## Note

Wowless does not enforce protected frames — only this simulator and live WoW enforce the rule.

## Sources

- [protected-frame-enforcement.md](../../protected-frame-enforcement.md) — full method list and open questions
- [protected-anchor-offsets.md](../../specs/protected-anchor-offsets.md) — bounded offset-mutation contract
- `/tmp/cross-version-anchor-protection-proof.md` — RED/GREEN command ledger and actual logs
