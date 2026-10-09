# ManagedAura dirty-phase fixture decision

Scope: read-only investigation in canonical `/home/osso/Projects/wow/wow-ui-sim`. No repo edits, tests/builds, network, delegation, vendor writes, or operational actions. Read `docs/wiki/index.md`, `/tmp/formatter-dirty-minimal-decision.md`, `/tmp/fail-dirty-format-map.md`, systematic-debugging skill, fixture, cached vendor sources, and the matching full-suite log. The saved full-suite artifact is `/home/osso/Projects/wow/full-suite-results/dd710c6fc469a53e3b77b97aa6340993ae83e46e.log`, failure `on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases` at log lines 8127–8144. No reproduction was run.

## Finding

**The observed failure is caused by a malformed test fixture, not evidence of an OnUpdate dispatcher defect.** The fixture replaces the real native `dirtyPhases` array with a one-entry phase list and then sends mask `1` through the native `ProcessDirtyFlags`. That producer leaves real bits unhandled, and the vendor completeness assertion reports remaining flags `18`. The requested test intent—hidden arming and one visible callback followed by Disabled—can be tested without replacing registration.

## Exact source/runtime boundary

Cached active-profile files inspected:

- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_AuraContainer/Blizzard_ManagedAuraContainer.lua`
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_AuraContainer/Blizzard_CustomAuraContainer.xml`
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_SharedXML/MixinUtil.lua`

`Blizzard_ManagedAuraContainer.lua:1–16` defines the phase flags from `bit.lshift(1, n)`, with comments identifying phases: `ParseAuras` (line 5), `ResetAuraFrames` (7), `RefreshAuraFrames` (9), `RefreshAuraFrameDisplay` (11), `RebuildLayoutGroups` (13), `ApplyLayout` (15). These are source-defined values, not inferred labels for residual `18`.

`Blizzard_ManagedAuraContainer.lua:59–74`: `ManagedAuraContainerPrivateMixin` includes `DirtyPhaseMixin`; `OnLoad_Intrinsic` calls `InitDirtyPhases()` then registers six ordered native phase handlers at lines 67–74, one for each named flag. `Blizzard_CustomAuraContainer.xml:4–13` loads `CustomAuraContainerTemplate` in a forbidden scoped modifier, mixes in `CustomAuraContainerPrivateMixin`, and binds `OnUpdate` to its native method.

`MixinUtil.lua:359–378`: initialization creates empty flags/phases; `SetDirtyPhases` stores the supplied array as `self.dirtyPhases` (line 377), replacing prior registration. `MixinUtil.lua:397–403` sets exactly the passed mask and updates dirty state; `405–411` clears a passed mask. `MixinUtil.lua:413–435` iterates registered phases in order, clears each matching phase flag before calling its handler, adds any returned downstream mask, updates dirty state, then asserts no flags remain.

The fixture `tests/on_update_modes.rs:151–192` creates the actual template and obtains its forbidden private table, but lines 164–168 explicitly replace native phases with `{{flag=1, handler=...}}`. It first clears existing flags, hides, calls `MarkDirty(1)`, and checks visible-once arming. The saved failure reaches inherited `ProcessDirtyFlags` and fails at the vendor `assertsafe` because remaining flags equal `18`. This matches the deliberate replacement of the complete phase registration by a single flag-1 handler; it does not demonstrate broken dispatcher sequencing. The phase values and meaning of remaining 18 should not be reverse-engineered from that aggregate: source registration and loop behavior are direct evidence of incompleteness.

`Blizzard_ManagedAuraContainer.lua:90–98`: native `OnUpdate` calls `ProcessDirtyFlags`; native `OnDirtyChanged(true)` sets `Enum.OnUpdateMode.RunWhenVisibleOnce`. In the fixture, `MarkDirty(1)` causes that real dirty-state transition. The existing dispatcher test assertion checks mode is Disabled from within the handler, and subsequent assertions check one call and Disabled after dispatch. However, because the fixture has replaced native handlers, its callback counter currently observes a fabricated phase callback rather than actual aura work.

## Smallest grounded fixture correction

Delete the fixture's `private:SetDirtyPhases(...)` replacement and its synthetic `AuraPhaseCalls` handler/counter assertions. Preserve the native registered phase table and use only already-observed contract state: hide the real container; invoke a real lifecycle producer; assert mode becomes `RunWhenVisibleOnce` while hidden; tick while hidden and assert it stays armed/no processing; show; tick; assert mode is Disabled and actual dirty state is clean; tick again and assert it remains Disabled/clean. This tests state transition and visible-once behavior, not implementation shape or handler count.

A real producer grounded in the native source is `ModeAuraContainer:UpdateAllAuras()` (`Blizzard_ManagedAuraContainer.lua:45–46`), which marks `AuraContainerDirtyMask.FullAuraRebuild`; that mask is source-defined at lines 20–27. Its full native phase chain is intended to consume the resulting work. Prefer this over hand-authored numeric mask `1`. To observe work, use an externally visible state change from a real container setup/operation, if one is available in this harness; do not wrap or replace individual native phase callbacks merely to count them. The present inspection did not establish a minimal initialized aura-group/frame fixture that yields a stable visible output, so the completion check should at minimum assert the public/private `IsDirty()` state clears after the visible dispatch. `ProcessDirtyFlags` itself enforces complete flag consumption.

The requested “visible once” semantic is observable directly through mode transitions and dirty-state clearing; callback implementation counts are unnecessary. Preserve `RunWhenVisibleOnce` while hidden, then verify Disabled after the eligible visible tick and no residual dirty state. Do not alter the production dirty-phase engine.

## Remaining unknowns

- Whether the broader test's stated intention requires proving aura-frame output mutation, beyond verifying real dirty processing; no minimal deterministic aura input/output fixture was established in this read-only pass.
- Whether `UpdateAllAuras()` has side effects requiring stubbing or setup under this harness; source shows it marks `FullAuraRebuild` and then refreshes/clears item enchantments, but runtime behavior was not executed.
- No test was rerun, so the proposed fixture correction remains an evidence-based diagnosis, not behavioral verification.
