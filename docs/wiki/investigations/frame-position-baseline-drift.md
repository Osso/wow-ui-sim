# Frame-position baseline drift

At the `tests/frame_positions.rs` full-startup fixture (1600×1200), two checks disagree with the current retail cache. They are reproducible and source-consistent, but not validated against a matching real client; no assertion or simulator behavior changed.

## Exact fixture evidence

The fixture loads all discovered Blizzard addons, runs normal startup plus post-event workarounds, waits two seconds, then runs three layout/timer ticks. It does not set a UI scale, SavedVariables, or Edit Mode fixture. Two fresh post-assertion diagnostics produced the same values in `/tmp/pi-frame-anchor-confirmed-{1,2}.stderr.log`.

`PrivateRaidBossEmoteFrameAnchor` was `(400, 182, 800×80)`. Current `Blizzard_RaidWarning/RaidWarning.xml` declares it `800×80`, top-anchored to `RaidWarningFrame`; that frame is `800×100`, top-anchored at `y=-182`. With no lowest warning message and hidden `DeadlyDebuffFrame`, `RaidWarningUtil.UpdateCenterScreenAnchors()` reanchors the private frame to `RaidWarningFrame.TOP`, yielding that rectangle.

`UIParentRightManagedFrameContainer` is absent. Current `RightManagedFrameContainer` occupies the equivalent managed-right role and was `(1335, 260, 260×847.5)`; `ObjectiveTrackerFrame` was parented to it. The old expected height `847` is within the test tolerance, but the old name is not present in current vendor source.

## Live capture and causal inputs

`FramePositionProbe` captured six samples with no recorded probe errors from retail `12.1.0.69587` (interface `120100`): login, world entry, delayed `0`/`2`/`5`, and manual. It confirms the private anchor is approximately `800×80`, `TOP`-anchored to `RaidWarningFrame.TOP` with zero offset; no lowest raid-warning message, hidden `DeadlyDebuffFrame`, and the old right-container name absent while `RightManagedFrameContainer` is present. Values are stable across samples, but the delayed `0`/`2`/`5` samples share one timestamp, so they are not three independently timed observations.

The custom `Ultrawide` layout, physical `3440×1440` display, UI `2293.333×960`, effective scale about `0.8`, and relevant non-Blizzard addons are valid causal replay inputs, not grounds to discard the capture. The simulator now replays the live physical display, scale, and raw saved `Ultrawide` EditMode cache through normal Blizzard startup; it does not assign any measured frame rectangle or anchor as setup output. Commit `b8d098059` supplies the distinct physical-display input model, and its display API coverage passes 9/9.

The initial replay exposed two simulator defects rather than a reason to reject the capture: `ObjectiveTrackerFrame` used the wrong anchor, then retained height `836.5` from a duplicate headless override. Commit `1e79fca4f` makes the post-event tracker repair default-only, reads the relevant layout state through its getter, and removes the duplicate unconditional `SetHeight`. The replay passes 1/1 against the existing one-UI-unit tolerance. Its empty right-managed container is width `0` versus approximately `1` live, which is within that tolerance; this is not bitwise-exact frame parity.

## Boundary

The old baseline is a simulator expectation, not a matching real-client capture. Current-source consistency, repeated simulator runs, and the causal replay do not establish whole-UI parity: the capture had 78 non-Blizzard addons, while replay covers selected layout inputs and frames only. Nor does it establish byte-identical fixture parity: the simulator cache is `12.1.0.69497` and the capture is retail `12.1.0.69587`. RaidWarning can still reanchor for messages, debuffs, and Edit Mode; managed-container layout depends on visible managed frames. The earlier CLI cache-import probe used a different parent/layout and is not comparable.

The capture remains valid evidence and the default `tests/frame_positions.rs` expectations remain unchanged pending an independent audit. Before changing them, reconcile the legacy assertions against this causal replay and explain any remaining source/build difference. A default-fixture mismatch alone is not a reason to discard the capture.

## Sources

- [tests/frame_positions.rs](../../../tests/frame_positions.rs) — exact shared fixture and failing baseline assertions
- [FramePositionProbe](../../addons/FramePositionProbe/README.md) — read-only capture protocol
- [display-metrics.md](../../specs/display-metrics.md) — physical-display and base-canvas replay contract and proof boundary
- `docs/local/private/probes/FramePositionProbe-2026-09-04T211127Z.lua` (gitignored) — non-parity retail capture; SHA-256 `c12586fbaa0c9203dad07089df827975c45cdc5fb3562606be365ef306c19b72`
- [post_event_frame_layout.rs](../../../src/lua_api/workarounds/temporary/post_event_frame_layout.rs) — adjacent old-name workaround
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_RaidWarning/RaidWarning.xml` — current size and XML anchors
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_RaidWarning/RaidWarningUtil.lua` — dynamic anchor choices
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_ManagedFrameSystem/Shared/ManagedFrameSystem.lua` — current right-container role

## See Also

- [[layout-lock-inventory]] — broad baseline lock coverage
- [[editmode-layout]] — managed-container and Edit Mode behavior
- [[display-metrics]] — physical display, UI canvas, and replay input contract
- [[layout-system]] — base-canvas layout coordinates
- [[prefork-test-harness]] — unrelated prefork migration verification
