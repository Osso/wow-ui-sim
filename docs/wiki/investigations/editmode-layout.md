# EditMode Layout

Four frame position regressions after commit 73e6032 (which made `__index` check Lua fields before Rust methods, matching real WoW behavior).

## Root Cause

`EditModeSystemMixin:OnSystemLoad` replaces three core methods on all 43 registered system frames by writing into each frame's per-frame fenv table:

```lua
self.SetPoint  = self.SetPointOverride    -- expects 5 explicit args
self.SetScale  = self.SetScaleOverride    -- adjusts anchor offsets on scale change
self.ClearAllPoints = self.ClearAllPointsOverride
```

After 73e6032, `__index` checks the fenv table before Rust methods, so all `:SetPoint()` / `:SetScale()` calls now hit these Lua overrides.

## Regressions

| Frame | Root Cause |
|---|---|
| FocusFrame | `SetScaleOverride` adjusts existing anchor offsets when scale changes; offset shifted from 520 to 693.3 |
| ObjectiveTrackerFrame | `SetPointOverride` expects 5 explicit args; 3-arg form `SetPoint("TOPRIGHT", -offset, -topOffset)` mapped numbers as relativeTo/relativePoint |
| UIParentRightManagedFrameContainer | `GetActionBarToggles` returning `(1,1,1,1)` showed MultiBarLeft/Right, making `GetRightActionBarWidth()` return ~100px |
| PetFrame | Hidden frame (no pet); removed from test |

## Fix (`workarounds_editmode.rs`)

**`clear_edit_mode_overrides`**: After `UpdateSystems` runs, clear all three overrides from every system frame's fenv via `rawset(debug.getfenv(frame)[1], "SetPoint", nil)` etc.

**`GetActionBarToggles` fix**: Changed stub to return `(false, false, false, false)` — default for a character with no optional bars enabled. This hides MultiBarLeft/Right, making `GetRightActionBarWidth` return 0 naturally.

**`reposition_managed_frames`**: Re-run `UpdateManagedFrames()` after all fixes to trigger correct `VerticalLayoutMixin:Layout()`.

**`patch_update_systems` / `patch_init_anchors`**: Skip `isManagedFrame` frames — their position comes from the container layout, not preset anchors.

## Key Constraint

`rawset(frame, key, nil)` fails — FrameRef is userdata, not a table. Per-frame fields must be accessed through `debug.getfenv(frame)[1]`.

## Follow-up: Active Profile State

The runtime bootstrap fallback for `C_EditMode.GetLayouts()` used to return
`{ layouts = {}, activeLayout = 1 }` on every call, while
`C_EditMode.SetActiveLayout()` was a no-op. Blizzard's
`EditModeManagerFrameMixin:UpdateLayoutInfo()` prepends preset layouts before
saved account/character layouts, so a hard-coded `activeLayout = 1` selects the
first preset profile and presents the default UI.

The fallback now keeps an in-memory `C_EditMode` state model:
`SaveLayouts()` stores saved layout data, `SetActiveLayout()` updates the
selected layout index, and `GetLayouts()` returns a defensive copy.

Real EditMode layout data is not a Lua SavedVariables file. WoW writes it to
`WTF/Account/<account>/edit-mode-cache-account.txt`, while the active
per-specialization selection lives in
`WTF/Account/<account>/<realm>/<character>/edit-mode-cache-character.txt`.
Startup now imports those cache files before Blizzard addons load, decodes the
compact system/anchor/settings rows, and seeds `C_EditMode`. Blizzard addons
are also loaded through the saved-variable-aware loader so Blizzard-owned WTF
SavedVariables participate in full UI startup.

### Active selection correction — 2026-10-12

**Superseded rationale:** the earlier investigation treated character-cache
index2 as a saved-layout ordinal and required remapping after preset insertion,
claiming that otherwise `C_EditMode` selected Widescreen while the manager
activated Classic. That interpretation was wrong; retained here as the rationale
for the simulator remap, not the current selection contract.

**Source:** unchanged Blizzard `UpdateLayoutInfo()` prepends Modern/Classic
without offsetting `activeLayout`. Character-cache active2 is already in that
combined selection space and selects Classic. The simulator remap incorrectly
offset it to4/Widescreen. The user reports horizontal Micro Menu in the actual
WoW client; this report is not native acceptance of the simulator repair.

**RED:** genuine fresh file-import whole-startup regression at `653e829f1`
failed the selected-layout assertion; geometry was unreached. Low-level cache
proof does not replace this original-consumer boundary.

**Committed source:** `057b65d3d` removes the remap from simulator setup.
Cache import resolves explicit saved-layout names as
`Enum.EditModePresetLayoutsMeta.NumValues + savedordinal`. Supported existing
positive numeric custom ordinal/ID resolution moves into the override producer;
no extra provenance flag or fallback. Named/snapshot tests and a fresh override
full-consumer control were added. Preset-only compatibility tweaks must still
avoid rewriting saved layout settings.

**Bounded Retail GREEN:** `7d9d9f621` executed startup2 (native Classic/horizontal
and four explicit Widescreen/vertical override variants), cache10 and Enter3.
Corrected normal startup reports Classic329×40 with zero Lua errors; fmt/default
check PASS. [Packet](../../../data/patch-api/evidence/edit-mode-enter-2026-10-12/README.md)
retains initial fixture compile failure, genuine RED and invalid binding-lookup
probe failure separately. Owned current-artifact GUI query observed Classic2,
MicroMenu329×40 and native1920×1080 window; initial90s viewing app expired naturally.
User later accepted layout. Both keyboard fields initially failed with native
PointerRoot focus/zero Enter deliveries; native focus assignment on the same
artifact produced deliveries and user confirmation both worked. Automatic focus
setup/fresh-startup and independent3045 ownership/socket/focus/geometry/wiring/lifetime gate PASS ([packet](../../../data/patch-api/evidence/native-window-focus-2026-10-12/README.md)); user authorized stopping after each test, without automatic
expiry; main stopped the test app after the gate. This is a native-focus runtime configuration fix, not new Rust keyboard logic after `7d9d9f621`. Later PTR `f3145424c` startup2/cache10/Enter3 and normal zero-errors/fmt/PTR-check PASS ([packet](../../../data/patch-api/evidence/edit-mode-enter-ptr-2026-10-12/README.md)); same Enter bodies run through a fresh whole Game constructor, not Retail prefork/cache-process proof. Universal pixel/native/all-key/all-profile parity and whole handoff remain OPEN. See [selection contract](../../specs/edit-mode-layout-selection.md).

### PTR packet and current-suite submission — 2026-10-12

[PTR packet](../../../data/patch-api/evidence/edit-mode-enter-ptr-2026-10-12/README.md), committed at `41cbf5005`, owns the main-verified `f3145424c` runtime/check evidence. Retail `7d9d9f621` proof remains reused, not replayed. PTR runtime PASS is not PTR GUI, earlier-profile, prefork cache/process, causal-performance or broad-current-suite acceptance.

Later public native-focus gate3049 at `3fce068f0` passed its exact-stream retention/link/anchor/privacy scope, separately from private native gate3045 and user acceptance; the [native packet](../../../data/patch-api/evidence/native-window-focus-2026-10-12/README.md) now retains that gate's bounded result. Earlier pending labels and separate PTR execution epochs are not retroactive native parity proof.

The private stage-qualified submission receipt records asynchronous `full-suite submit 41cbf5005d4fec8dba98538e425e6d8fd7f0f5c3`, exit0, unit `full-suite-1791847608.service`, invocation `ef206e594ed4485cbc0c1d4e9bbef291`. At submission, acceptance was **SUBMISSION ONLY**. Main subsequently independently read terminal JSON for source `41cbf5005`: integration exit0, prefork exit0, lib exit100; the broad suite **FAILS** on `setup_layout_info_remaps_active_saved_layout_after_prepending_presets` and `setup_layout_info_preserves_distinct_saved_action_bar_profile_settings`. Independent artifact gate3090 reconciles [retained terminal evidence](../../../data/test-perf/evidence/fullsuite-41cb-2026-10-12/README.md): 15,123 PASS / 2 FAIL / 19 reported skips. Evidence verification PASS does not mean suite PASS. Historical receipts remain unchanged; whole handoff OPEN.

### Test-only fixture repair — pending verification, 2026-10-12

Committed `b41b42fc1` corrects the fixtures to the existing combined-index contract: active4 selects the second custom layout; active3 selects the first custom layout, not layoutID9; native active2 remains the Classic control. Production native combined-index behavior is unchanged. Gate3095 found the new preset control reused cached manager layoutInfo; follow-up `ac170ecbf` resets that fixture before selecting Classic. At that historical checkpoint runtime was resource-BLOCKED, zero tests invoked; neither repaired test was credited PASS then. The terminal failure above belongs to source `41cbf5005`, not a post-repair run. Earlier Retail `7d9d9f621` and PTR `f3145424c` startup2/cache10/Enter3 proof and native user acceptance retain their bounded scopes; they do not make the current broad suite passing.

### Focused repaired-fixture execution — 2026-10-15

[Owning selection spec](../../specs/edit-mode-layout-selection.md#focused-fixture-execution--2026-10-15) records actual sourcefb4b31b8f Retail GUI-feature execution of both repaired fixtures. This closes only their bounded execution gap, not historical source41cb broad FAIL or full-suite/startup/native acceptance. Independent saved-runtime reconciliation **PASS**; the owning spec links the report. No production remap, vendor, user-cache or orientation change is implied.

Follow-up: selecting the saved layout still was not enough for the visible UI.
The startup fast path for action bars seeded Widescreen `systemInfo` and replayed
settings, but skipped `ApplySystemAnchor`, leaving bars at the temporary
`TOPLEFT, 0, 0` anchor from `InitSystemAnchors()`. The fast path now applies the
saved anchor before refreshing bar layout/art.

Follow-up: the main action bar side art is an EditMode setting
(`Enum.EditModeActionBarSetting.HideBarArt`), not independent renderer state.
Sparse saved-layout rows can omit default-valued settings, so startup now merges
missing defaults from Blizzard's modern preset map before seeding systems. The
action-bar fast path applies those fields directly instead of calling Blizzard
handlers that iterate `self.actionButtons` before the simulator has a real Lua
button array there.

Follow-up: the action-bar fast path now replays the safe runtime side effects
for saved profile settings once a real Lua `actionButtons` array exists:
visibility updates, show-grid state, icon count, icon scale, padding, page
number visibility, and button art refresh. This keeps Widescreen action-bar
options from being merely preserved in `systemInfo` without changing the visible
bar state.

Follow-up: saved action-bar slider values are compact raw values. The fast path
must read values through Blizzard's `GetSettingValue()` after `UpdateSettingMap()`
so settings like `IconSize` use the converted display percentage; raw enum
comparisons such as orientation and visibility still use raw values.

Follow-up: account-level EditMode settings are not per-system settings, so
preserving `EditModeManagerFrame.accountSettings` is not enough. Startup now
calls Blizzard's `InitializeAccountSettings()` after rebuilding layout info so
saved grid, snap, advanced-options, and hidden-frame toggles are applied through
the normal account-setting control path.

Follow-up: account caches can be sparse when newer account options were added
after the profile was saved. The bootstrap `C_EditMode.__LoadCache()` path now
merges saved account values over the default account-setting list so omitted
newer toggles, such as damage meter and external defensives, still have default
values before `InitializeAccountSettings()` reads them.

Follow-up: startup no longer forces the active profile's cast bar to lock under
the player frame. Saved profiles can intentionally set
`Enum.EditModeCastBarSetting.LockToPlayerFrame = 0` and
`Enum.EditModeUnitFrameSetting.CastBarUnderneath = 0`; those values must remain
profile data, while missing defaults are still filled from the preset map.

Follow-up: some EditMode systems are intentionally seeded instead of sent through
Blizzard's full `UpdateSystem()` during startup because those paths do expensive
roster work or can apply scale before the frame is ready. Seeding still needs to
replay saved settings, so the startup path now calls `UpdateSystemSetting()` for
those systems after loading their saved `systemInfo`. Unit-frame
`BuffsOnTop` is deferred when a seeded frame has no `UpdateAuras()` method,
which keeps ScriptErrors clean without dropping the saved setting from the
profile data.

Follow-up: `--no-saved-vars` must not skip the EditMode cache. EditMode profile
data lives in `edit-mode-cache-account.txt` and
`edit-mode-cache-character.txt`, not Lua SavedVariables files. Skipping that
cache made the simulator fall back to no active profile, so profile-hidden
systems such as the status tracking / XP bar could become visible even though
the real client hides them. Startup now uses a separate EditMode-cache-only WTF
loader when Lua SavedVariables are disabled.

Follow-up: Mists Classic 5.5.4 (`Gethe/wow-ui-source@0fa05055`) added a real
`Blizzard_EditMode` addon for the Mists profile. The existing `C_EditMode`
layout/cache fallback was enough for startup, but ticking the loaded tree exposed
a legacy mirror-timer constant gap: `Blizzard_UIParent/Classic/WorldFrame.lua`
iterates `for i=1, MIRRORTIMER_NUMTIMERS` while processing hidden mirror timers.
`GetMirrorTimerInfo()` and `GetMirrorTimerProgress()` were already modeled in
Rust, so the matching constant now lives in the same mirror-timer registration
surface instead of a Mists-only bootstrap shim.

The same patch also made `Blizzard_UIParentPanelManager` loadable for Mists
through the `_Classic.toc` variant. The simulator's Mists profile must not
exclude that addon anymore, and it must prefer `_Classic.toc` for this addon so
stale cached `_Mists.toc` files from older pins cannot win TOC selection.
The Blizzard UI cache manifest includes the Classic panel-manager TOC,
`UIPanelWindows.lua`, and Mists overrides now; without those files
`GameMenuFrame` opens as a raw frame but `ShowUIPanel(GameMenuFrame)` has no
panel registration to dispatch through.

Mists 5.5.4 EditMode added a second startup hazard for action bars. Blizzard's
`EditModeSystemMixin:ApplySystemAnchor()` intentionally parks default bottom/right
action bars at `TOPLEFT UIParent` while `layoutApplyInProgress` is true, then
expects `EditModeManagerFrame:UpdateActionBarPositions()` to move them. The sim's
manual `apply_system_anchors.lua` path can abort before its cleanup tail, leaving
`MainActionBar`, action buttons, and the micro-menu at the temporary top-left
anchor. `workarounds_editmode.rs` now runs a separate
`finalize_action_bar_positions` step after `apply_system_anchors`: it clears
`layoutApplyInProgress` and calls Blizzard's managed action-bar positioning pass.

The Mists TargetFrame EditMode refresh also requires Classic global
`UnitIsCivilian`. The simulator registers it in the modeled unit-query surface
and currently returns false because civilian NPC flags are not modeled.

ObjectiveTracker phantom duplicate was the legacy-plus-modern tracker path, but
only when bundled addons create the legacy quest watch surface. Clean no-addons
startup has `WatchFrame` nil; bundled-addon startup showed visible `WatchFrame`
quest text overlapping modern `ObjectiveTrackerFrame` quest text. Mists
`post_load.lua` now hides `WatchFrame` when `ObjectiveTrackerFrame` exists and
wraps `WatchFrame_Update()` so legacy quest-log code cannot resurrect it.

Regression coverage lives in `tests/edit_mode_api.rs`.

## PTR preset and startup fixture reconciliation — 2026-11-04

The pinned PTR original preset manager inserts Modern, Classic and Gamepad;
Retail inserts Modern and Classic. The [selection contract](../../specs/edit-mode-layout-selection.md)
records the exact source lines. Five cache fixtures assumed Retail's two presets.
They now use the published preset count or the original manager's copied preset
list, while retaining saved-row identity, geometry and native Classic controls.

The cast-bar lock fixture also assumed an idle bar already had a managed parent.
PTR `Blizzard_UIPanels_Game/Mainline/CastingBarFrame.xml:468–482` starts it hidden
under UIParent. `Shared/CastingBarFrame.lua:1385–1389` and
`Blizzard_ManagedFrameSystem/Shared/ManagedFrameSystem.lua:17–18,46–76` move it
through the original OnShow path. The fixture now shows the unlocked bar before
asserting managed parenting, then retains the locked PlayerFrame attachment check.
The existing Mists fade/hidden-parent assertions are unchanged.

The seventh failure was not a preset mismatch: default `lua-errors` startup
exited1 because original `Blizzard_FrameXMLUtil/Mainline/DifficultyUtil.lua:87`
called the missing `C_QuestLog.GetQuestDifficultyLevel`. Its existing supplied-row
producer was Forever-only. The [query contract](../../specs/forever-quest-difficulty.md#declared-profile-publication--2026-11-04)
now records the declared Retail/PTR/Forever publication set and named emulation
limits. No Blizzard Lua was changed.

At source `9c2eea67f`, PTR integration `edit_mode` passes46/46, shared getter
controls pass2/2 and the full PTR library passes2200 with one existing opt-in
GPU test ignored. Retail integration `edit_mode` passes56/56 and getter controls
pass2/2. The Retail filter does not run the cast-bar prefork selector; broader
final verification belongs to the integration supervisor. These are bounded
simulator proofs, not native parity.

## Files Modified

- `src/lua_api/workarounds_editmode.rs`
- `src/lua_api/globals/c_stubs_api.rs`
- `tests/frame_positions.rs`, `tests/action_bar.rs`
- `src/lua_api/env_init/runtime_surface_bootstrap.lua`
- `src/saved_variables.rs`
- `src/bin/wow_sim/main.rs`
- `src/bin/wow_sim/addon_loading.rs`
- `tests/edit_mode_api.rs`
- `src/lua_api/globals/instance_info.rs`
- `tests/instance_info.rs`
- `tests/blizzard_ui_parent_loads.rs`
- `src/loader/mod.rs`
- `src/blizzard_ui_sync/profile_cache.rs`
- `tests/mists_interface_options_panel.rs`

## Sources

- [editmode-layout-fixes.md](../../editmode-layout-fixes.md) — historical investigation
- [edit-mode-layout-selection.md](../../specs/edit-mode-layout-selection.md) — current selection contract and proof limits
- [runtime_surface_bootstrap.lua](../../../src/lua_api/env_init/runtime_surface_bootstrap.lua) — `C_EditMode` fallback state
- [saved_variables.rs](../../../src/saved_variables.rs) — WTF cache import bridge
- [instance_info.rs](../../../src/lua_api/globals/instance_info.rs) — mirror-timer API and `MIRRORTIMER_NUMTIMERS`
- [loader/mod.rs](../../../src/loader/mod.rs) — Mists panel-manager TOC selection
- [profile_cache.rs](../../../src/blizzard_ui_sync/profile_cache.rs) — Mists Blizzard UI cache requirements

## See Also

- [[method-dispatch-refactor]] — the `__index` ordering change that exposed these bugs
