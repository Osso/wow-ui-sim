# Frame-position baseline drift

At the `tests/frame_positions.rs` full-startup fixture (1600×1200), two checks disagree with the current retail cache. They are reproducible and source-consistent, but not validated against a matching real client; no assertion or simulator behavior changed.

## Exact fixture evidence

The fixture loads all discovered Blizzard addons, runs normal startup plus post-event workarounds, waits two seconds, then runs three layout/timer ticks. It does not set a UI scale, SavedVariables, or Edit Mode fixture. Two fresh post-assertion diagnostics produced the same values in `/tmp/pi-frame-anchor-confirmed-{1,2}.stderr.log`.

`PrivateRaidBossEmoteFrameAnchor` was `(400, 182, 800×80)`. Current `Blizzard_RaidWarning/RaidWarning.xml` declares it `800×80`, top-anchored to `RaidWarningFrame`; that frame is `800×100`, top-anchored at `y=-182`. With no lowest warning message and hidden `DeadlyDebuffFrame`, `RaidWarningUtil.UpdateCenterScreenAnchors()` reanchors the private frame to `RaidWarningFrame.TOP`, yielding that rectangle.

`UIParentRightManagedFrameContainer` is absent. Current `RightManagedFrameContainer` occupies the equivalent managed-right role and was `(1335, 260, 260×847.5)`; `ObjectiveTrackerFrame` was parented to it. The old expected height `847` is within the test tolerance, but the old name is not present in current vendor source.

## Boundary

The old baseline is a simulator expectation, not a tracked real-client capture. Current-source consistency and repeated simulator runs do not establish retail correctness: RaidWarning can reanchor for messages, debuffs, and Edit Mode; managed-container layout depends on Edit Mode anchors and visible managed frames. The earlier CLI cache-import probe used a different parent/layout and is not comparable. The obsolete-name post-event workaround is adjacent and unproven; this investigation does not attribute either failure to it.

Before changing assertions, capture a matching retail client at the same build, 1600×1200 resolution, scale, layout, warning/debuff state, and startup boundary.

## Sources

- [tests/frame_positions.rs](../../../tests/frame_positions.rs) — exact shared fixture and failing baseline assertions
- [post_event_frame_layout.rs](../../../src/lua_api/workarounds/temporary/post_event_frame_layout.rs) — adjacent old-name workaround
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_RaidWarning/RaidWarning.xml` — current size and XML anchors
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_RaidWarning/RaidWarningUtil.lua` — dynamic anchor choices
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_ManagedFrameSystem/Shared/ManagedFrameSystem.lua` — current right-container role

## See Also

- [[layout-lock-inventory]] — broad baseline lock coverage
- [[editmode-layout]] — managed-container and Edit Mode behavior
- [[prefork-test-harness]] — unrelated prefork migration verification
