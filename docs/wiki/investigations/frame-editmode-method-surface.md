# Frame EditMode method surface

**VERIFIED:** `IsEditModeDragging`, `IsInitialized`, `IsInDefaultPosition`, and `IsSystemSettingDefault` are owned by Blizzard EditMode mixins, not by every Frame.

## Root cause

**VERIFIED (checked-in method discovery):** `diff_methods_extra.txt` lists all four methods on all 16 widget types (64 rows). The former shared Frame registration made each method visible on ordinary frames.

**VERIFIED (pinned Retail, PTR, WoW Forever, and Mists source):** `EditModeSystemMixin` defines all four methods in `Blizzard_EditMode/Shared/EditModeSystemTemplates.lua`; `MainActionBarMixin` also defines `IsInDefaultPosition` in `Blizzard_ActionBar/Shared/MainActionBar.lua`. A plain frame has no per-instance method, so the presence guard leaves it eligible for managed layout.

**VERIFIED (original consumer and RED test):** `Blizzard_ManagedFrameSystem/Shared/ManagedFrameSystem.lua` checks `frame.IsInDefaultPosition` before calling it. The removed generic implementation returned false when `systemInfo` was absent, so a plain managed frame was rejected before `AddManagedFrame` could lay it out. The fix removes the generic registration rather than inventing a default `systemInfo` value.

## Mists managed-frame path

**VERIFIED (selected Mists source):** `UIParentManagedFrameContainerMixin:AddManagedFrame` in `Blizzard_UIParent/Shared/UIParent.lua` uses the same presence guard. Mists `PaladinPowerBar` inherits `PlayerFrameBottomManagedFrameTemplate`, not an EditMode system template, so it now reaches the original managed-frame layout.

**VERIFIED (Mists source and runtime fixture):** the managed container is 160 units wide and anchors to `PlayerFrame` at `(10,36)`; the Paladin bar's `leftPadding=30` and `topPadding=1` make the vertical layout offsets `(15,-1)`. Its rectangle remains `(341,915,136x39)`, while its parent and anchor now identify the managed container.

**VERIFIED (baseline comparison at `62509480f`):** the Mists `frame_positions` harness fails the same four unrelated cases—MicroMenu, MicroMenuContainer, RaidBossEmoteFrame, and PlayerCastingBarFrame—before and after the change. The corrected `paladin_power_bar` case passes after the change.

## Profile scope

**VERIFIED:** Retail, PTR, WoW Forever, and Mists ship the shared `EditModeSystemMixin` definitions, so these profiles need no generic Frame shim. Mists' unowned managed frames use a presence check and remain eligible for layout.

**UNVERIFIED:** Wrath runtime ownership and layout were not tested under the project rule that excludes Wrath proofs; no Wrath-specific behavior claim is made here.

## Sources

- [method discovery](../../wow-client-diff/diff_methods_extra.txt) — methods reported as extras on widget types.
- [frame method registration](../../../src/lua_api/frame/methods/misc/mod.rs) — shared method surface.
- [RED/GREEN consumer test](../../../tests/frame_edit_mode_methods.rs) — plain frame behavior through the original managed-frame consumer.
- [profile layout fixtures](../../../tests/frame_positions.rs) — tracker and Mists Paladin layout assertions.
- [frame helper defaults](../../../src/lua_api/workarounds/temporary/frame_helper_defaults.rs) — regression checks for generic frame helper methods.
- Pinned Blizzard source: `Blizzard_EditMode/Shared/EditModeSystemTemplates.lua`, `Blizzard_ActionBar/Shared/MainActionBar.lua`, `Blizzard_ManagedFrameSystem/Shared/ManagedFrameSystem.lua`, `Blizzard_UIParent/Shared/UIParent.lua`, `Blizzard_UnitFrame/Shared/PlayerFrameTemplates.xml`, `Blizzard_UnitFrame/Classic/PlayerFrame.xml`, `Blizzard_UnitFrame/Mists/PaladinPowerBar.xml`, and `Blizzard_SharedXML/LayoutFrame.lua`.

## See Also

- [[editmode-layout]] — EditMode anchor overrides and layout behavior.
- [[mists-world-map-startup]] — Mists cache selection and startup history.
- [[consumer-selection-and-red-proof]] — original-consumer applicability and RED proof.
