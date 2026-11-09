# Frame-position baseline drift

The default `tests/frame_positions.rs` canvas (1600×1200) exposed obsolete raid-warning geometry and a renamed managed container. A live capture and causal replay also exposed simulator overrides that discarded custom ObjectiveTracker configuration. Those overrides are retired; the baseline now checks the supported geometry and native default-height relationship.

## Exact fixture evidence

The fixture loads all discovered Blizzard addons, runs normal startup plus post-event workarounds, waits two seconds, then runs three layout/timer ticks. It does not set a UI scale, SavedVariables, or Edit Mode fixture. Two fresh post-assertion diagnostics produced the same values in `/tmp/pi-frame-anchor-confirmed-{1,2}.stderr.log`.

`PrivateRaidBossEmoteFrameAnchor` was `(400, 182, 800×80)`. Current `Blizzard_RaidWarning/RaidWarning.xml` declares it `800×80`, top-anchored to `RaidWarningFrame`; that frame is `800×100`, top-anchored at `y=-182`. With no lowest warning message and hidden `DeadlyDebuffFrame`, `RaidWarningUtil.UpdateCenterScreenAnchors()` reanchors the private frame to `RaidWarningFrame.TOP`, yielding that rectangle.

`UIParentRightManagedFrameContainer` is absent. Current `RightManagedFrameContainer` occupies the equivalent managed-right role and was `(1335, 260, 260×847.5)`; `ObjectiveTrackerFrame` was parented to it. The old expected height `847` is within the test tolerance, but the old name is not present in current vendor source.

## Live capture and causal inputs

`FramePositionProbe` captured six samples with no recorded probe errors from retail `12.1.0.69587` (interface `120100`): login, world entry, delayed `0`/`2`/`5`, and manual. It confirms the private anchor is approximately `800×80`, `TOP`-anchored to `RaidWarningFrame.TOP` with zero offset; no lowest raid-warning message, hidden `DeadlyDebuffFrame`, and the old right-container name absent while `RightManagedFrameContainer` is present. Values are stable across samples, but the delayed `0`/`2`/`5` samples share one timestamp, so they are not three independently timed observations.

The custom `Ultrawide` layout, physical `3440×1440` display, UI `2293.333×960`, effective scale about `0.8`, and relevant non-Blizzard addons are valid causal replay inputs, not grounds to discard the capture. The simulator now replays the live physical display, scale, and raw saved `Ultrawide` EditMode cache through normal Blizzard startup; it does not assign any measured frame rectangle or anchor as setup output. Commit `b8d098059` supplies the distinct physical-display input model, and its display API coverage passes 9/9.

The initial replay exposed simulator-owned `ObjectiveTrackerFrame` geometry rather than a reason to reject the capture: a post-event repair supplied the wrong anchor, and startup retained height `836.5` through a duplicate headless clamp. Commit `1e79fca4f` first limited the post-event repair; commit `c6a452dfc` removes every ObjectiveTracker repair from `post_event_frame_layout.rs` and the duplicate startup clamp. Native Blizzard/EditMode layout now owns tracker geometry for default and custom layouts. The captured replay remains within the existing one-UI-unit tolerance; independent focused verification passed all 12 display/replay/probe cases. Its empty right-managed container is width `0` versus approximately `1` live, which is within that tolerance; this is not bitwise-exact frame parity.

## Boundary

The old baseline is a simulator expectation, not a matching real-client capture. Current-source consistency, repeated simulator runs, and the causal replay do not establish whole-UI parity: the capture had 78 non-Blizzard addons, while replay covers selected layout inputs and frames only. Nor does it establish byte-identical fixture parity: the simulator cache is `12.1.0.69497` and the capture is retail `12.1.0.69587`. RaidWarning can still reanchor for messages, debuffs, and Edit Mode; managed-container layout depends on visible managed frames. The earlier CLI cache-import probe used a different parent/layout and is not comparable.

The default-canvas expectations use the bounded contracts supported by this replay: an empty-warning raid anchor is 800×80 at RaidWarningFrame.TOP (400,182 on the 1600×1200 canvas), and the container is named RightManagedFrameContainer. ObjectiveTracker's default height is 847.5, from its parent height plus anchor offset in Blizzard's UpdateHeight, not the removed 836.5 clamp. Additional checks assert the empty-warning state, anchor relationship, default positioning, parent, and native height relationship. Test names and geometry/alpha tolerances are unchanged. This does not claim whole-client or byte-identical-build parity.

## Profile reconciliation — 2026-11-09 design

The historical default table is Retail-specific. At master `3ca96ba58`, all inventory names still reproduce: PTR 5, Forever 15, Mists 25, Era 23 and Anniversary 23; Retail passes 28/28. These are simulator observations, not native captures. The lane report and source hashes live untracked in `~/.local/state/wow-ui-sim/handoff/frame-positions-profiles/`.

The [profile baseline contract](../../specs/frame-position-profile-baselines.md) preserves all case names and tolerances. Acceptance commit `de53ffa71` adds parent, native dimensions, unit scale and default-anchor checks before the existing minimap geometry assertions. PTR still rejects the old Retail rectangles after these causal inputs pass (`red-ptr.log`).

### Source-backed expectation rules

Paths below are under each profile's `AddOns` cache; each source is inspected in that profile, not borrowed from Retail. `[Family]` selects Mainline for Retail/PTR/Forever and Classic for Mists/Era/Anniversary; Forever's `[Game]` is Camelot. Classic flavor TOCs and their AllowLoadGameType clauses select the older definitions.

- **PTR minimap:** `Blizzard_Minimap/Mainline/Minimap.xml:3-16,25-62` gives cluster padding 20, a 215×226 container at TOP `(10,-30)`, a centered 198×198 map, and tracking width 17. `Mainline/MinimapZoneTextButton.xml:7-10` gives a centered 175×12 header at y=-6. `Mainline/MinimapIndicatorFrameTemplate.xml:5-6` fixes the indicator at 20×15; `Mainline/Minimap.lua:506-510` anchors its right edge two units past tracking's right edge. Relative to cluster center, the leftmost indicator is -105.5 and the rightmost container edge is 117.5: width = 223+20 = **243**. The visible tracking top is 3.5 and the container bottom is 256: height = **252.5**. With the preset TOPRIGHT anchor, the cluster is `(1357,0,243,252.5)` and map `(1389.5,44,198,198)`.
- **Forever minimap:** `Blizzard_Minimap/Mainline/Minimap.xml:3-16,47-62`, `Camelot/MinimapZoneTextButton.xml:9-11`, and `Camelot/MinimapIndicatorFrameTemplate.xml:5-9` give a header at y=0, container 215×226 at `(10,-30)`, map 198×198, and no indicator reaching beyond the container. `Camelot/Minimap.lua:31-38` uses the native frame atlas dimensions for the container. Width = 215+20 = **235**; height = **256**. Preset TOPRIGHT yields cluster `(1365,0,235,256)` and map `(1393.5,44,198,198)`.
- **Mists/Era/Anniversary minimap:** each `Blizzard_Minimap/Classic/Minimap.xml:18-29,64-92` declares a 176×28 top border, 32×32 toggle at RIGHT `(1,1)`, 140×140 container below the border with y=6, and a centered 140×140 map. Toggle extends one unit past the border right and three above its top; container bottom is 162. `ResizeLayoutMixin:Layout` takes direct-child extents, producing **177×165**. TOPRIGHT yields cluster `(1423,0,177,165)` and map `(1441.5,22,140,140)`. Their own Classic preset maps select header-above and size 5 (unit scale).
- **Classic party frames:** `Blizzard_CompactRaidFrames/Classic/Blizzard_CompactRaidFrameManager.lua:153-155` collapses the 200-wide manager to x=-182 at y=140. Each Classic preset map anchors Party TOPLEFT to its TOPRIGHT with `(0,12)`: `(18,128)`. `Blizzard_UnitFrame/Classic/PartyFrameTemplates.xml:70-71` declares 128×53 members; four members, three ten-unit gaps and two bottom padding give 128×244. Compact-party retains its existing 98×234 size contract; only its shared party anchor changes. Mists zero anchors remain failures, not new expectations.
- **Classic cast bar:** `Blizzard_UIPanels_Game/Classic/CastingBarFrame.xml:148-149` declares 195×13. The Classic preset CastBar block supplies CENTER at UIParent CENTER and unlocked state. The retained centered-startup contract becomes `(702.5,593.5,195,13)`; Mists' zero anchor must still fail. This checks the harness's current unlocked fixture, not native managed-cast-bar parity.
- **Classic raid warning:** the selected `Blizzard_FrameXML/Classic/RaidWarning.xml:29-65` declares RaidWarningFrame 512×70 below UIErrorsFrame, then RaidBossEmoteFrame 512×80 below RaidWarningFrame. On the unchanged UIErrors canvas, use `RaidBossEmoteFrame` at `(544,252,512,80)`, not the Retail private anchor or Retail `GetLowestMessage` method.
- **PTR/Forever dock:** `Blizzard_ChatFrameBase/Mainline/FloatingChatFrame.lua:2199-2202` anchors GeneralDockManager to ChatFrame1.Background, not ChatFrame1. The background at `:290-293` extends left by 2 and right by `7+scrollbarWidth`; stock scrollbar width is 8. From the separately checked 430×170 chat frame at `(35,980)`, the dock is `(33,951,447,26)`. Classic `Classic/FloatingChatFrame.lua:1734-1737` instead anchors directly to the chat frame with y=6, giving `(35,948,430,26)` for the same input. Era/Anniversary's shifted chat input remains independently failed.
- **PTR/Forever MicroMenu container:** each selected `Blizzard_MicroMenu/Shared/MicroMenuContainer.lua:20-60` sizes around MicroMenu alone, unlike Retail/Classic sources that add QueueStatusButton. PTR retains MicroMenu's existing `(1265,1154,329,40)` rectangle. Forever's `Blizzard_EditMode/Camelot/EditModePresetLayoutConstants.lua:38-43` sets BOTTOM at UIParent BOTTOM with x=116.5, y=6: `(752,1154,329,40)`. The same file `:5-10` anchors the unchanged 562×45 main bar BOTTOMRIGHT to menu BOTTOMLEFT at `(-4.5,-4)`, yielding `(185.5,1153,562,45)` and ActionButton1 x=185.5.
- **Unavailable frames are not ignored tests:** `Blizzard_EncounterWarnings.toc:2` allows only standard in each non-Retail profile's cache, excluding Camelot and Classic. Its three existing case names must assert that their globals are absent. Forever's `Blizzard_UnitFrame.toc:110-111` excludes Camelot from PaladinPowerBarFrame; Classic UnitFrame TOCs load PaladinPowerBar only for Cata/Mists, not Vanilla. Assert PaladinPowerBarFrame absence on Forever/Era/Anniversary. Mists' renamed PaladinPowerBar is a separate unresolved layout case, not evidence to accept the Retail name.

### Edges and exclusions

No simulator layout primitive or Blizzard code changes are justified by these baseline failures. Preserve the Retail table, original count, original alpha/geometry tolerances and failed-name reporting. Select explicit profile contracts; never generate expectations from simulator output. Missing anchors, placeholder ObjectiveTracker state, and unresolved managed-container, aura, bag, chat or status-bar behavior stay failed and are enumerated in the lane report. Unknown native-C inputs must be questions for main, not guessed coordinates. All profile source caches and Cargo dependency files remain unchanged.

## Sources

- [tests/frame_positions.rs](../../../tests/frame_positions.rs) — exact shared fixture and failing baseline assertions
- [FramePositionProbe](../../addons/FramePositionProbe/README.md) — read-only capture protocol
- [display-metrics.md](../../specs/display-metrics.md) — physical-display and base-canvas replay contract and proof boundary
- `docs/local/private/probes/FramePositionProbe-2026-09-04T211127Z.lua` (gitignored) — live retail capture; SHA-256 `c12586fbaa0c9203dad07089df827975c45cdc5fb3562606be365ef306c19b72`
- [post_event_frame_layout.rs](../../../src/lua_api/workarounds/temporary/post_event_frame_layout.rs) — remaining unrelated repairs after retiring ObjectiveTracker overrides
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_RaidWarning/RaidWarning.xml` — current size and XML anchors
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_RaidWarning/RaidWarningUtil.lua` — dynamic anchor choices
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_ManagedFrameSystem/Shared/ManagedFrameSystem.lua` — current right-container role

## See Also

- [[layout-lock-inventory]] — broad baseline lock coverage
- [[editmode-layout]] — managed-container and Edit Mode behavior
- [[display-metrics]] — physical display, UI canvas, and replay input contract
- [[layout-system]] — base-canvas layout coordinates
- [[prefork-test-harness]] — unrelated prefork migration verification
