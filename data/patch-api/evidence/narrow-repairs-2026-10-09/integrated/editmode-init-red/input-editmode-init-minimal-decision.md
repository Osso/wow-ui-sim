# EditMode `InitSystemAnchors` fixture decision

**Scope:** read-only, bounded review of the three failing `apply_system_anchors` library tests, using the provided falsification notes plus the current canonical checkout and unchanged retail cache. No tests, edits to the repo, delegation, network, or vendor changes were made. The three failures in `/tmp/lib-sentinel-falsification.md` stop at the same `env.exec(APPLY_SYSTEM_ANCHORS_LUA)` precondition; none reached its named assertions.

## Literal contract and boundary

The unchanged cached vendor file `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_EditMode/Shared/EditModeManager.lua` calls `InitSystemAnchors()` in `UpdateLayoutInfo` before `UpdateSystems` (lines 952–954). Its initializer at lines 1382–1394 resets each registered, non-exempt system frame to `TOPLEFT UIParent, 0, 0`, using each frame's `ClearAllPoints`/`SetPoint`; managed frames already in default position are exempt. `GetActiveLayoutSystemInfo` at lines 1441–1450 performs an exact `(system, systemIndex)` lookup; this function does not itself define nil-to-minus-one retry.

The simulator wrapper begins by setting `layoutApplyInProgress` and calling this same initializer (`src/lua_api/workarounds/editmode/apply_system_anchors.lua:5–8`), before its replay/final-anchor work. This is a simulator startup workaround, not a vendor patch. Its existence, comments, and direct base-anchor helpers show deliberate avoidance of costly/unsafe Blizzard update paths in selected startup cases, but do **not** establish that the whole wrapper is an unsupported performance override. The initial-anchor phase matches the cached vendor ordering and the literal spec `docs/specs/edit-mode-initial-anchors.md:1–10` (initialize before callbacks; QueueStatus consumer; retain final saved QueueStatus anchor). Selected per-system bypasses are implementation workarounds and are not independently authorized by that spec. No optimization proposal follows from these failures; do not alter vendor code.

## Exact minimum fixture repair

Add the same small, semantically accurate manager method to each of the three fixtures, before `APPLY_SYSTEM_ANCHORS_LUA` runs:

```lua
function EditModeManagerFrame:InitSystemAnchors()
    for _, systemFrame in ipairs(self.registeredSystemFrames or {}) do
        if not ((systemFrame.isBottomManagedFrame or systemFrame.isRightManagedFrame)
            and systemFrame:IsInDefaultPosition()) then
            systemFrame:ClearAllPoints()
            systemFrame:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
        end
    end
end
```

This is fixture setup, not a production guard/no-op. It preserves `systemFrame.systemInfo`, settings maps, and other fixture state; it changes only the temporary anchor state just as the cached method does. For these three fixtures, none provides managed-frame markers or `IsInDefaultPosition`, so the method needs only each frame's existing fake geometry API. Exact existing per-test fixture anchors:

- `cast_and_player.rs:278–407`: manager at 370–374 lacks the method. The player fixture has `ClearAllPointsBase`/`SetPointBase`, but the production helper's normal base APIs are only present here for the final saved anchor; initializer calls the regular frame methods. Add fixture `ClearAllPoints` and `SetPoint` methods that mutate temporary anchor fields, leaving `systemInfo.anchorInfo` intact; keep the final-anchor base methods/count separate. This is necessary to represent the vendor initializer without conflating temporary reset with saved-anchor application.
- `singletons.rs:123–214`: manager at 157–161 lacks it; frame currently lacks `ClearAllPoints`/`SetPoint`. Add those methods and minimal current-anchor state to that fixture, then add the initializer loop above. Do not manufacture or mutate its saved `systemInfo`/settings.
- `unit_frames.rs:291–467`: manager at 395–403 lacks it; each compact frame can receive `ClearAllPoints`/`SetPoint` methods and separate current-anchor fields, then initializer loop. The only existing anchor observations for this test are callback counters, not final geometry.

The helper should live in each fixture (or a narrowly shared test helper only if one already exists; none was established here). Avoid a no-op `InitSystemAnchors`: it would silence the failure while omitting the actual required initialization semantics.

## What the current tests actually assert

| Test | Observable outcome data present? | Unsupported/internal-shape assertion | Grounded disposition |
|---|---|---|---|
| `cast_and_player.rs:278–407` | Saved player frame size maps to display value 150, so expected frame scale 1.5; saved `BOTTOM` anchor is in fixture, but asserted only `point` and `relativePoint`, not `relativeTo`/offset nor final coordinates. `setPointBaseCalls == 1` is a mechanism count. | Manager stub's `ApplySystemAnchor` would call cast bar's `ApplySystemAnchor`; the test requires zero calls. Yet this fixture's `PlayerCastingBarFrame` only defines `UpdateSystemSettingBarSize` to throw, not `ApplySystemAnchor`, so that named error text is not exercised by the shown method. The production player-frame path intentionally calls `apply_anchor_info_directly`, not `ApplySystemAnchor` (`apply_system_anchors.lua:711–720`); that path and its no cast-bar callback are local implementation behavior, not a general native contract. | Preserve/check the visible scale and final saved geometry as results. The spec grounds initial-before-final ordering only for the QueueStatus dependency, not a player-frame-specific callback prohibition. Do not assert exact base SetPoint count or zero `ApplySystemAnchor` as WoW contract. Whether to keep a regression test for simulator-specific no-side-effect behavior is not decided by the provided contracts. The fixture presently does not demonstrate the named cast-bar side-effect counterexample. |
| `singletons.rs:123–214` | If `-1` row is selected, setting-map and setting-handler result are observed. The fixture provides no saved anchor to inspect, and the manager returns a fabricated row only for `-1`. | Exact request list `"nil,-1"` is internal lookup sequence; nil→minus-one retry is not required in `edit-mode-initial-anchors.md`, and cached vendor `GetActiveLayoutSystemInfo` does exact index matching. | Delete/replace the invented nil→minus-one-fallback test contract unless another literal project contract or consumer evidence is supplied. Do not infer that production fallback should be removed: the simulator wrapper deliberately has this lookup at `lookup_system_info` (`apply_system_anchors.lua`, around 89–101), while the current spec does not govern it. The test does not establish a real singleton geometry/settings result independent of its fabricated `-1` fixture row. |
| `unit_frames.rs:291–467` | Fixture records handler input strings plus counts for fake refresh hooks, but stores no resulting frame layout/state/geometry. | Entire assertion is an exact summary of synthetic method calls/counts (`10:true,...|1|2...`); it explicitly errors if `UpdateSystemSetting` is used instead. This proves only the chosen batching/callback implementation shape, not correct compact-frame outcome or native callback count. | Delete/replace the exact batch-count assertion as evidence for behavior; replacement requires an observable final layout/settings/refresh result contract and representative backing fixture. No such contract is in the cited spec. Do not conclude production batching is wrong or right from this fixture. |

## Decision boundary

Fixture initializer repair is precise and sufficient to get past the common setup precondition; it is not authorization to alter production behavior or assert the subsequent tests pass. The three test bodies still require separate assessment against actual results after fixture repair. No result for those bodies is available here because tests were explicitly forbidden. Recommendations about deleting/replacing assertions are contract-evidence findings, not automatic tasks. The full suite is independently running; this read-only decision does not inspect or interfere with it.
