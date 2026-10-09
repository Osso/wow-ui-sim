//! Surface-level globals pinned by the `Blizzard_ActionBar` lane —
//! action-button keybind input dispatch.
//!
//! PLAN.md task: `ActionButtonDown`, `ActionButtonUp`,
//! `MultiActionButtonDown`, `MultiActionButtonUp`, `ExtraActionButtonKey`,
//! `TryUseActionButton`, `ActionBar_PageUp`, `ActionBar_PageDown` are
//! functions (depends-on: action-button input globals gap).
//!
//! All eight PLAN-named globals match the actual Blizzard source verbatim —
//! no spec/source mismatches in this batch. Each is declared via the
//! `function <name>(args)` syntax (not `<name> = function(args)`), which
//! under Lua 5.1 semantics binds to the global table at file-chunk
//! execution time:
//!
//! - `TryUseActionButton(self, checkingFromDown)` at
//!   `Shared/ActionButton.lua:109` — the shared "fire the secure click"
//!   helper. Calls `SecureActionButton_OnClick(self, "LeftButton",
//!   checkingFromDown, isKeyPress=true, isSecureAction=true)`, then if the
//!   click resolved to a real action, clears any "new action highlight"
//!   mark on `self.action` and refreshes the highlight visual via
//!   `self:UpdateHighlightMark()`. Always finishes with
//!   `self:UpdateState()` so the button's checked/visual state stays in
//!   sync. Both `ActionButtonDown`/`Up`, `MultiActionButtonDown`/`Up`, and
//!   `ExtraActionButtonKey` route through this single helper.
//!
//! - `ActionButtonDown(id)` at `Shared/ActionButton.lua:136` — the
//!   `BINDING_HEADER_ACTIONBAR` keybind down-handler. Early-returns when
//!   `CheckPetActionButtonEvent(id, true)` reports the player is in pet
//!   battle (the helper at line 123 routes to `PetBattleFrame_ButtonDown`
//!   instead). Otherwise resolves the bound button via
//!   `GetActionButtonForID(id)`, transitions `NORMAL`→`PUSHED` via
//!   `SetButtonState`, and dispatches `TryUseActionButton(button, true)`
//!   with `checkingFromDown=true`.
//!
//! - `ActionButtonUp(id)` at `Shared/ActionButton.lua:151` — the
//!   key-release counterpart. Same pet-battle short-circuit, then
//!   `PUSHED`→`NORMAL` transition + `TryUseActionButton(button, false)`.
//!   The down/up split exists because keybindings dispatch separately on
//!   key-press vs key-release; the secure click semantics differ between
//!   the two phases (only one of them actually fires the spell, depending
//!   on the `ActionButtonUseKeyDown` CVar).
//!
//! - `MultiActionButtonDown(barName, id)` at `Shared/MultiActionBars.lua:35`
//!   — keybind down-handler for the multi-bars (Bottom Right / Bottom
//!   Left / Right / Right2 / and the additional 3 bars added in newer
//!   patches). Reads `_G[barName].actionButtons[id]` to resolve the
//!   button and dispatches the same `NORMAL`→`PUSHED` + `TryUseActionButton`
//!   pair as `ActionButtonDown`. Two-arg surface (the bar name disambiguates
//!   which multi-bar the keypress belongs to; `id` is the button index 1-12
//!   within that bar).
//!
//! - `MultiActionButtonUp(barName, id)` at `Shared/MultiActionBars.lua:44`
//!   — key-release counterpart for the multi-bars. **Asymmetric with
//!   `ActionButtonUp`**: only fires `TryUseActionButton(button, false)` if
//!   the button was already in `PUSHED` state (i.e. had received a matching
//!   `MultiActionButtonDown`). The single-bar `ActionButtonUp` ALWAYS fires
//!   `TryUseActionButton` even from `NORMAL`, but the multi-bar variant
//!   gates on the down-state. This asymmetry exists because the multi-bars
//!   can be hidden mid-keypress via `MultiActionBar_HideAllGrids`, leaving
//!   a stranded button in `NORMAL` whose Up handler should not re-fire
//!   the action.
//!
//! - `ExtraActionButtonKey(id, isDown)` at `Shared/ExtraActionBar.lua:63`
//!   — keybind handler for the contextual extra-action bar (vehicle
//!   abilities, encounter-specific extra buttons). **Single function
//!   handles both down AND up phases** via the `isDown` parameter — the
//!   only entry point in the lane that fuses both phases into one global.
//!   Early-returns when `C_ActionBar.HasExtraActionBar()` is false (the
//!   bar isn't currently active). Resolves `_G["ExtraActionButton"..id]`
//!   and dispatches the same state-transition + TryUseActionButton pair
//!   as the regular Up handler, with the same Multi-bar-style PUSHED-gate
//!   on the up phase.
//!
//! - `ActionBar_PageUp()` at `Shared/ActionButton.lua:166` — the
//!   `BINDING_NAME_ACTIONPAGE2` (and similar) page-cycle handler. Walks
//!   the `VIEWABLE_ACTION_BAR_PAGES` table from `current_page + 1` to
//!   `NUM_ACTIONBAR_PAGES`, picks the first viewable page, and dispatches
//!   `C_ActionBar.SetActionBarPage(nextPage)`. Wraps to page 1 when no
//!   later page is viewable. Zero-arg surface.
//!
//! - `ActionBar_PageDown()` at `Shared/ActionButton.lua:181` — the
//!   page-cycle handler in the opposite direction. Walks
//!   `current_page - 1` down to 1, then wraps to the highest viewable
//!   page. Two-pass loop (descend, then wrap) — slightly larger control
//!   flow than `_PageUp` because the wrap target has to be found
//!   iteratively rather than always 1. Zero-arg surface.
//!
//! Why a single test with a table-driven loop is sufficient: every PLAN
//! global has the same contract — `type(name) == "function"` after the
//! startup-shape harness loads `Blizzard_ActionBar`. Splitting them into
//! per-global tests would buy nothing (all eight failure modes have the
//! same root cause — file chunk failed before reaching the declaration,
//! OR Blizzard re-shaped the surface).
//!
//! **Note on the gap dependency.** PLAN.md line 93 ("Implement action-
//! button input globals ...") is already checked off — the simulator's
//! `runtime_surface_bootstrap.lua` / Rust env-init publishes
//! `ActionButtonDown`, `ActionButtonUp`, `MultiActionButtonDown`,
//! `MultiActionButtonUp`, `ExtraActionButtonKey`, and `TryUseActionButton`
//! BEFORE any addon loads. The Blizzard source then re-declares these at
//! file scope (lines 109/136/151 in ActionButton.lua, 35/44 in
//! MultiActionBars.lua, 63 in ExtraActionBar.lua), overwriting the
//! simulator's pre-published versions. The post-load contract is therefore
//! a two-way pin: either the simulator's pre-publish OR the addon's
//! source declaration must result in a function. `ActionBar_PageUp` and
//! `ActionBar_PageDown` are the only two not on the gap line — they're
//! published exclusively by the addon's source (lines 166/181). A nil
//! reading on those two would prove the addon's `Shared/ActionButton.lua`
//! file chunk failed before reaching the declarations.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

const ROOT: &str = "Blizzard_ActionBar";
const PLAN_NAMED_INPUT_GLOBALS: &[&str] = &[
    "ActionButtonDown",
    "ActionButtonUp",
    "MultiActionButtonDown",
    "MultiActionButtonUp",
    "ExtraActionButtonKey",
    "TryUseActionButton",
    "ActionBar_PageUp",
    "ActionBar_PageDown",
];
const PLAN_NAMED_HIGHLIGHT_GLOBALS: &[&str] = &[
    "MarkNewActionHighlight",
    "ClearNewActionHighlight",
    "GetNewActionHighlightMark",
    "ClearOnBarHighlightMarks",
    "GetOnBarHighlightMark",
    "UpdateOnBarHighlightMarksBySpell",
    "UpdateOnBarHighlightMarksByFlyout",
    "UpdateOnBarHighlightMarksByPetAction",
    "GetActionButtonForID",
];
const PLAN_NAMED_MULTIBAR_GLOBALS: &[&str] = &[
    "MultiActionBar_Update",
    "MultiActionBar_ShowAllGrids",
    "MultiActionBar_HideAllGrids",
    "MultiActionBar_GetBarForPage",
    "MultiBar1_IsVisible",
    "MultiBar2_IsVisible",
    "MultiBar3_IsVisible",
    "MultiBar4_IsVisible",
    "MultiBar5_IsVisible",
    "MultiBar6_IsVisible",
    "MultiBar7_IsVisible",
    "IsNormalActionBarState",
];
const PLAN_NAMED_EXTRA_BAR_GLOBALS: &[&str] = &[
    "ExtraActionBar_OnLoad",
    "ExtraActionBar_Update",
    "ExtraActionBar_ForceEmpty",
    "ExtraActionBar_ForceShowIfNeeded",
    "ExtraActionBar_CancelForceShow",
];
const PLAN_NAMED_BAR_HELPER_GLOBALS: &[&str] = &[
    "ArtifactBarGetNumArtifactTraitsPurchasableFromXP",
    "ReputationParagonWatchBar_OnEnter",
    "ReputationParagonWatchBar_OnLeave",
];

#[test]
fn action_bar_plan_named_input_globals_are_functions() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for name in PLAN_NAMED_INPUT_GLOBALS {
            let is_function = env
                .eval::<bool>(&format!(r#"return type({name}) == "function""#))
                .expect("file-scope global type probe must run cleanly");

            assert!(
                is_function,
                "Global `{name}` MUST be a function after the startup-shape harness loads \
                 `Blizzard_ActionBar`. Each entry in `PLAN_NAMED_INPUT_GLOBALS` is declared via \
                 `function <name>(...)` syntax in the lane's Lua files \
                 (Shared/ActionButton.lua:109/136/151/166/181, Shared/MultiActionBars.lua:35/44, \
                 Shared/ExtraActionBar.lua:63). A non-function reading proves either (a) the \
                 file chunk failed before reaching the declaration (load.rs's \
                 `action_bar_load_emits_no_lane_specific_lua_errors` would also fail in that case \
                 — multi-test failure mode), OR (b) Blizzard re-shaped the keybind dispatch \
                 surface (e.g. moved `ActionButtonDown` onto a SecureHandler-style frame method \
                 instead of a top-level global, which would break every existing `Bindings.xml` \
                 entry that names `ActionButtonDown` directly), OR (c) — for `ActionButtonDown`, \
                 `_Up`, `MultiActionButtonDown`, `_Up`, `ExtraActionButtonKey`, \
                 `TryUseActionButton` — the simulator's gap-fill pre-publish at PLAN.md:93 \
                 regressed AND the addon's source-side declaration didn't run; for \
                 `ActionBar_PageUp`/`_PageDown` (the only two NOT on the gap-fill line) a nil \
                 reading proves the addon's file chunk failed before the declaration line. \
                 Got `type({name}) == \"function\"` returned false."
            );
        }
    });
}

/// Pin the action-highlight-tracking globals — the surface that
/// `SpellBookFrame`, `Blizzard_PlayerSpells`, and `Blizzard_TalentUI`
/// consume to mark "new action" indicators on action bar buttons and to
/// flag the "this spell is already on a bar" hint while a player is
/// dragging from spellbook/talent panels.
///
/// PLAN.md task: `MarkNewActionHighlight`, `ClearNewActionHighlight`,
/// `GetNewActionHighlightMark`, `ClearOnBarHighlightMarks`,
/// `GetOnBarHighlightMark`, `UpdateOnBarHighlightMarksBySpell`,
/// `UpdateOnBarHighlightMarksByFlyout`,
/// `UpdateOnBarHighlightMarksByPetAction`, `GetActionButtonForID` are
/// functions (depends-on: action highlight tracking globals gap).
///
/// All 9 PLAN-named globals match the actual Blizzard source verbatim.
/// Each is declared via the `function <name>(...)` syntax (not `<name> =
/// function(...)`), which under Lua 5.1 semantics binds to the global
/// table at file-chunk execution time. Source map (all in
/// `Shared/ActionButton.lua`):
///
/// - `MarkNewActionHighlight(action)` at `lua:27` — single-line setter:
///   `ACTION_HIGHLIGHT_MARKS[action] = true`. The file-scope
///   `ACTION_HIGHLIGHT_MARKS = { }` declaration at lua:9 owns the
///   storage; this function is the public mutator.
///
/// - `ClearNewActionHighlight(action, preventIdenticalActionsFromClearing)`
///   at `lua:31` — clears the mark on `action` and (unless the second
///   arg is truthy) walks `ACTION_HIGHLIGHT_MARKS` to also clear marks
///   on every other action whose `(actionType, actionID)` from
///   `GetActionInfo` matches the cleared one. The cascade exists so
///   that putting the same spell on two bars only requires one click
///   to dismiss both highlights.
///
/// - `GetNewActionHighlightMark(action)` at `lua:61` — read accessor.
///   Trampolines through `securecallfunction(SecureGetNewActionHighlightMark, ...)`
///   (the local at lua:57) so taint from the caller's stack doesn't
///   leak into the table read. The comment at lua:55 flags
///   `ACTION_HIGHLIGHT_MARKS`/`ON_BAR_HIGHLIGHT_MARKS` keys as
///   "vulnerable to taint from talent and spellbook code" —
///   `securecallfunction` is the runtime defence.
///
/// - `ClearOnBarHighlightMarks()` at `lua:65` — re-binds
///   `ON_BAR_HIGHLIGHT_MARKS` to a fresh empty table (NOT
///   `wipe(ON_BAR_HIGHLIGHT_MARKS)` — re-binding is intentional so
///   stale references held by tainted callers don't survive the clear).
///   Zero-arg.
///
/// - `GetOnBarHighlightMark(action)` at `lua:73` — read accessor.
///   Mirror of `GetNewActionHighlightMark` but against
///   `ON_BAR_HIGHLIGHT_MARKS`; same `securecallfunction` taint
///   firewall, same single-arg shape.
///
/// - `UpdateOnBarHighlightMarksBySpell(spellID)` at `lua:85` — calls
///   `C_ActionBar.FindSpellActionButtons(spellID)` to resolve the bar
///   slots that currently hold the spell, then dispatches to the
///   file-local `UpdateOnBarHighlightMarks` (lua:77) which `tInvert`s
///   the result so `ON_BAR_HIGHLIGHT_MARKS[slot] = true` for each
///   matching slot. Falls through to `ClearOnBarHighlightMarks()` when
///   no buttons match.
///
/// - `UpdateOnBarHighlightMarksByFlyout(flyoutID)` at `lua:89` — same
///   shape as the spell variant but probes `FindFlyoutActionButtons`
///   for flyout-button slots (the master flyout button, e.g. for
///   mage portals or hunter pet specials).
///
/// - `UpdateOnBarHighlightMarksByPetAction(petAction)` at `lua:93` —
///   same shape, `FindPetActionButtons`. The three Update*By* variants
///   are deliberately thin trampolines into one shared
///   `UpdateOnBarHighlightMarks` helper — keeping the public surface
///   matched to the three different action-source kinds while letting
///   the storage re-write logic live in one place.
///
/// - `GetActionButtonForID(id)` at `lua:97` — the keybind dispatcher
///   uses this to map an action-button id (1-12) to a frame. Branches
///   on `OverrideActionBar:IsShown()`: when the override bar is up
///   (vehicle / possess / temporary shapeshift), returns
///   `_G["OverrideActionBarButton"..id]` (or nil when `id >
///   NUM_OVERRIDE_BUTTONS`); otherwise returns
///   `_G["ActionButton"..id]`. A regression on this function would
///   silently break every keybind on the override bar.
///
/// **Note on the gap dependency.** PLAN.md line 94 ("Implement action
/// highlight tracking globals ...") is checked off — the simulator's
/// `runtime_surface_bootstrap.lua` / Rust env-init pre-publishes ALL
/// 9 of these globals BEFORE any addon loads. The Blizzard source
/// then re-declares them at file scope (lines 27/31/61/65/73/85/89/93/97
/// in `Shared/ActionButton.lua`), overwriting the simulator's
/// pre-published versions. The post-load contract is therefore a
/// two-way pin: either the simulator's pre-publish OR the addon's
/// source declaration must result in a function. Unlike the input-
/// globals batch above, this batch has NO PLAN-only globals — every
/// entry is on PLAN.md:94, so a nil reading on any of them proves the
/// gap-fill regressed AND the addon's file-scope declaration didn't
/// run.
#[test]
fn action_bar_plan_named_highlight_tracking_globals_are_functions() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for name in PLAN_NAMED_HIGHLIGHT_GLOBALS {
            let is_function = env
                .eval::<bool>(&format!(r#"return type({name}) == "function""#))
                .expect("file-scope global type probe must run cleanly");

            assert!(
                is_function,
                "Global `{name}` MUST be a function after the startup-shape harness loads \
                 `Blizzard_ActionBar`. Each entry in `PLAN_NAMED_HIGHLIGHT_GLOBALS` is declared \
                 via `function <name>(...)` syntax in `Shared/ActionButton.lua` \
                 (lines 27/31/61/65/73/85/89/93/97). A non-function reading proves either (a) the \
                 file chunk failed before reaching the declaration (load.rs's \
                 `action_bar_load_emits_no_lane_specific_lua_errors` would also fail in that case \
                 — multi-test failure mode), OR (b) Blizzard re-shaped the highlight-tracking \
                 surface — e.g. moved the global setters/getters onto an `ActionBarHighlightMixin` \
                 frame mixin instead of top-level globals, which would break every \
                 SpellBookFrame / Blizzard_PlayerSpells / Blizzard_TalentUI consumer that names \
                 these globals directly), OR (c) the simulator's gap-fill pre-publish at \
                 PLAN.md:94 regressed AND the addon's source-side declaration didn't run. Every \
                 one of these 9 globals is on the gap-fill line, so a nil reading is unambiguous \
                 — both publishers failed. Storage globals `ACTION_HIGHLIGHT_MARKS` (lua:9) and \
                 `ON_BAR_HIGHLIGHT_MARKS` (lua:10) are NOT pinned by this test (separate \
                 surface_globals task) but a regression that wiped them would be observable here \
                 indirectly: `ClearOnBarHighlightMarks()` re-binds the global; if the file-scope \
                 declaration didn't run, the re-bind would create an out-of-band global that \
                 readers of the original file-scope name wouldn't see. Got `type({name}) == \
                 \"function\"` returned false."
            );
        }
    });
}

/// Pin the multi-bar driver/visibility globals — the surface that
/// `EditMode`, `MainMenuBar`, and addon authors consume to query and
/// drive the state of the secondary action bars (BottomLeft / BottomRight
/// / Right / Right2 plus the three additional bars added in newer
/// patches).
///
/// PLAN.md task: `MultiActionBar_Update`, `MultiActionBar_ShowAllGrids`,
/// `MultiActionBar_HideAllGrids`, `MultiActionBar_GetBarForPage`,
/// `MultiBar1_IsVisible` … `MultiBar7_IsVisible`,
/// `IsNormalActionBarState` are functions.
///
/// All 12 PLAN-named globals match the actual Blizzard source verbatim
/// and live in a single file: `Shared/MultiActionBars.lua`. Each is
/// declared via `function <name>(...)` syntax binding to `_G` at file-
/// chunk execution time. Source map (line numbers in
/// `Shared/MultiActionBars.lua`):
///
/// - `IsNormalActionBarState()` at `lua:53` — single-line predicate:
///   `return MainActionBar:IsShown()`. The other multi-bar logic
///   gates on this — when the override bar is up (vehicle / possess),
///   `MainActionBar` hides and the multi-bars suppress their visibility
///   tracking via `UpdateMultiActionBar` at lua:78-93.
///
/// - `MultiActionBar_Update()` at `lua:95` — driver that walks the
///   table returned by `GetMultiActionBars()` (file-local at lua:58)
///   and dispatches `UpdateMultiActionBar(bar, isVisible, page)` for
///   each entry. The page argument feeds `VIEWABLE_ACTION_BAR_PAGES`
///   so `ActionBar_PageUp`/`PageDown` can skip pages bound to hidden
///   bars. Zero-arg.
///
/// - `MultiActionBar_ShowAllGrids(reason)` at `lua:104` — calls
///   `barEntry.bar:SetShowGrid(true, reason)` on every multi-bar.
///   The `reason` is one of the `ACTION_BUTTON_SHOW_GRID_REASON_*`
///   constants (CVAR / EVENT / SPELLCOLLECTION at
///   `Shared/ActionButton.lua:12-14`) so the bar can refcount overlapping
///   show-grid sources and only hide when ALL reasons clear.
///
/// - `MultiActionBar_HideAllGrids(reason)` at `lua:113` — counterpart
///   that calls `SetShowGrid(false, reason)` against the same reason
///   tag. The bar's own refcounted state at the SetShowGrid level
///   ensures pairing with the matching ShowAllGrids call.
///
/// - `MultiActionBar_GetBarForPage(page)` at `lua:131` — single
///   expression: `return (bars and bars[page]) and bars[page].bar or
///   nil`. The double-guard handles two failure modes — first the
///   addon-not-yet-loaded case (`GetMultiActionBars()` returns nil
///   before `MainActionBar` is available; see file-local at lua:60),
///   second the page-not-mapped case (only pages 3/4/5/6/13/14/15 map
///   to multi-bars; querying page 1 or 2 returns nil because they're
///   bound to `MainActionBar` itself).
///
/// - `MultiBar1_IsVisible()` … `MultiBar7_IsVisible()` at
///   `lua:140/144/148/152/156/160/164` — seven trampolines that each
///   delegate to the file-local `IsMultibarVisible(index)` (lua:17)
///   with their respective indices 1-7. The trampoline pattern exists
///   because `EditMode` and `MainMenuBar` register frame-show
///   callbacks that name a specific MultiBar by number, so each bar
///   needs its own globally-named accessor. Internally they all read
///   from `Settings.GetValue("PROXY_SHOW_ACTIONBAR_<N+1>")` (note the
///   off-by-one: `MultiBar1` reads `PROXY_SHOW_ACTIONBAR_2` because
///   `_1` is the main bar). Zero-arg surface.
///
/// **Note on the gap dependency.** This batch's PLAN line has NO
/// `(depends-on:)` suffix — none of these globals are pre-published by
/// the simulator's `runtime_surface_bootstrap.lua` / Rust env-init.
/// The contract is therefore one-way: a nil reading on any of these
/// globals proves the addon's `Shared/MultiActionBars.lua` file chunk
/// failed before reaching the declaration line. There is no fallback
/// publisher, unlike the input-globals and highlight-tracking batches
/// above. This makes the test a STRICTER tripwire than the prior two —
/// a partial-load regression that breaks the multi-bar Lua chunk would
/// be caught here even when other action-bar surface tests still pass.
#[test]
fn action_bar_plan_named_multibar_globals_are_functions() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for name in PLAN_NAMED_MULTIBAR_GLOBALS {
            let is_function = env
                .eval::<bool>(&format!(r#"return type({name}) == "function""#))
                .expect("file-scope global type probe must run cleanly");

            assert!(
                is_function,
                "Global `{name}` MUST be a function after the startup-shape harness loads \
                 `Blizzard_ActionBar`. Each entry in `PLAN_NAMED_MULTIBAR_GLOBALS` is declared \
                 via `function <name>(...)` syntax in `Shared/MultiActionBars.lua` \
                 (lines 53/95/104/113/131/140/144/148/152/156/160/164). Unlike the input-globals \
                 and highlight-tracking batches above, this batch has NO simulator-side gap-fill \
                 pre-publish (PLAN line carries no `depends-on:` suffix). The contract is \
                 therefore one-way: a nil reading proves the addon's file chunk failed before \
                 reaching the declaration — no fallback publisher exists. A non-function reading \
                 also flags any of: (a) the file chunk failed before this declaration line \
                 (load.rs's `action_bar_load_emits_no_lane_specific_lua_errors` would also fail \
                 in that case — multi-test failure mode), OR (b) Blizzard re-shaped the multi-bar \
                 surface — e.g. moved `MultiBar1_IsVisible` … `_IsVisible` onto a single \
                 `MultiActionBar_IsVisible(index)` accessor (which would break every \
                 EditMode/MainMenuBar consumer that names a specific MultiBar by number), OR \
                 (c) the file-local `IsMultibarVisible` (lua:17) was renamed and the trampolines \
                 still reference the old name — a parse-time success but a runtime failure on \
                 first call. Got `type({name}) == \"function\"` returned false."
            );
        }
    });
}

/// Pin the ExtraActionBar lifecycle/visibility globals — the surface
/// that drives the contextual extra-action bar (vehicle abilities,
/// encounter-specific extra buttons, world-event handout buttons like
/// the "Use Item" prompt during scripted sequences).
///
/// PLAN.md task: `ExtraActionBar_OnLoad`, `ExtraActionBar_Update`,
/// `ExtraActionBar_ForceEmpty`, `ExtraActionBar_ForceShowIfNeeded`,
/// `ExtraActionBar_CancelForceShow` are functions.
///
/// All 5 PLAN-named globals match the actual Blizzard source verbatim
/// and live in a single file: `Shared/ExtraActionBar.lua`. Each is
/// declared via `function <name>(...)` syntax binding to `_G` at file-
/// chunk execution time. Source map (line numbers in
/// `Shared/ExtraActionBar.lua`):
///
/// - `ExtraActionBar_OnLoad(self)` at `lua:5` — the XML `<OnLoad>`
///   handler bound at `Shared/ExtraActionBar.xml:126`. Two-line body:
///   `self:SetFrameLevel(self:GetFrameLevel() + 2)` (so the bar
///   renders above its parent's contemporaries) and `self:SetAlpha(0.0)`
///   (default-hidden — the alpha drives the intro/outro animations
///   later when `_Update` toggles visibility). Single-arg surface.
///
/// - `ExtraActionBar_Update()` at `lua:10` — the public visibility
///   driver. Three-way branch on `C_ActionBar.HasExtraActionBar()`:
///   when true, plays the `intro` animation and registers the bar with
///   `ExtraAbilityContainer:AddFrame(bar, ExtraActionButtonPriority)`;
///   when false-and-shown, either calls `_ForceEmpty` (if
///   `KeybindFrames_InQuickKeybindMode`) or plays the `outro` animation;
///   when false-and-hidden, removes from the ability container. Zero-arg.
///
/// - `ExtraActionBar_ForceEmpty()` at `lua:32` — strips the visible
///   button to a transparent shell: `bar.button.style:Hide()` plus
///   `bar.button.icon:SetAlpha(0)`. Used during quick-keybind mode
///   (so the player can re-bind the slot without the live extra
///   action's icon getting in the way) and from `_ForceShowIfNeeded`
///   to ensure the bar shows blank when forced visible without a
///   real action. Zero-arg.
///
/// - `ExtraActionBar_ForceShowIfNeeded()` at `lua:38` — the
///   force-show entry point used by glue-side flows that need the
///   bar visible regardless of `HasExtraActionBar()`. When the bar is
///   currently hidden, fires `_ForceEmpty` + `bar.button:Show()` +
///   `bar:Show()` + `UpdateUsable` + `ExtraAbilityContainer:AddFrame`
///   + `intro` animation. The if-not-shown guard prevents double-show
///   on repeat calls. Zero-arg.
///
/// - `ExtraActionBar_CancelForceShow()` at `lua:51` — the counterpart
///   that hides the bar IF it was forced visible AND no real extra
///   action exists. Gated by `not C_ActionBar.HasExtraActionBar() and
///   bar:IsShown()` — so a real extra action surviving past the
///   force-show period stays visible. Restores the visual state
///   (`bar.button.style:Show()` + `bar.button.icon:SetAlpha(1)`)
///   before hiding, so the next `_Update` that re-shows the bar with
///   a real action doesn't paint over a stripped style. Zero-arg.
///
/// **Note on the gap dependency.** This batch's PLAN line has NO
/// `(depends-on:)` suffix — none of these globals are pre-published
/// by the simulator's `runtime_surface_bootstrap.lua` / Rust env-init.
/// The contract is therefore one-way: a nil reading on any of them
/// proves the addon's `Shared/ExtraActionBar.lua` file chunk failed
/// before reaching the declaration line. There is no fallback
/// publisher, matching the multi-bar batch above. Note the asymmetry
/// with the input-globals batch: `ExtraActionButtonKey` (lua:63 in
/// the same file) IS on the gap-fill line at PLAN.md:93, but the
/// five lifecycle globals here are addon-only.
#[test]
fn action_bar_plan_named_extra_bar_globals_are_functions() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for name in PLAN_NAMED_EXTRA_BAR_GLOBALS {
            let is_function = env
                .eval::<bool>(&format!(r#"return type({name}) == "function""#))
                .expect("file-scope global type probe must run cleanly");

            assert!(
                is_function,
                "Global `{name}` MUST be a function after the startup-shape harness loads \
                 `Blizzard_ActionBar`. Each entry in `PLAN_NAMED_EXTRA_BAR_GLOBALS` is declared \
                 via `function <name>(...)` syntax in `Shared/ExtraActionBar.lua` \
                 (lines 5/10/32/38/51). The PLAN line carries no `(depends-on:)` suffix, so the \
                 simulator does NOT pre-publish these in `runtime_surface_bootstrap.lua` / Rust \
                 env-init — a nil reading is unambiguous: the addon's file chunk failed before \
                 the declaration. No fallback publisher exists. Note the asymmetry within this \
                 same source file: `ExtraActionButtonKey` (lua:63) IS gap-filled by PLAN.md:93, \
                 but the five lifecycle globals here are addon-only — a partial-load regression \
                 that breaks the file chunk between lua:5 and lua:51 would leave \
                 `ExtraActionButtonKey` working (via the gap-fill pre-publish) while ALL of these \
                 five fail simultaneously, surfacing the partial-load mode that no other test in \
                 this lane catches. A non-function reading also flags any of: (a) the file chunk \
                 failed before this declaration line (load.rs's \
                 `action_bar_load_emits_no_lane_specific_lua_errors` would also fail in that \
                 case — multi-test failure mode), OR (b) Blizzard re-shaped the lifecycle \
                 surface — e.g. moved `_Update` / `_ForceEmpty` onto an `ExtraActionBarMixin` \
                 frame mixin instead of top-level globals, which would break the XML `<OnLoad>` \
                 binding at `Shared/ExtraActionBar.xml:126` plus every world-event consumer that \
                 calls `ExtraActionBar_ForceShowIfNeeded` / `_CancelForceShow` directly. Got \
                 `type({name}) == \"function\"` returned false."
            );
        }
    });
}

/// Pin the bar-helper globals — a mixed-publisher batch that exercises
/// the cross-addon surface boundary. Two of the three globals are
/// declared by a SIBLING addon (`Blizzard_UIPanels_Game`), not by
/// `Blizzard_ActionBar` itself, even though they are CONSUMED by
/// `Blizzard_ActionBar`'s `Shared/ReputationBar.lua`. A regression in
/// either addon's file-chunk load — or in the TOC dependency wiring
/// that ensures `Blizzard_UIPanels_Game` loads BEFORE `Blizzard_ActionBar`
/// — would break this contract.
///
/// PLAN.md task: `ArtifactBarGetNumArtifactTraitsPurchasableFromXP`,
/// `ReputationParagonWatchBar_OnEnter`, `ReputationParagonWatchBar_OnLeave`
/// are functions.
///
/// Source map and publisher attribution:
///
/// - `ArtifactBarGetNumArtifactTraitsPurchasableFromXP(pointsSpent,
///   artifactXP, artifactTier)` at
///   `Blizzard_ActionBar/Mainline/ArtifactBar.lua:96` — declared BY
///   `Blizzard_ActionBar` itself, but ONLY in the Mainline TOC flavor.
///   Pure helper that loops calling `C_ArtifactUI.GetCostForPointAtRank`
///   until either `artifactXP` runs below the next-rank cost or
///   `xpForNextPoint` reaches 0 (the `C_ArtifactUI` cap signal). Returns
///   the triple `(numPoints, remainingXP, xpForNextPoint)` so the caller
///   can render the partial-fill UI. The internal consumer is
///   `ArtifactBarMixin:UpdateBar` at lua:17 — the public global exists
///   so the legacy artifact UI in `Blizzard_ArtifactUI` can drive its own
///   tier-spend math through the same helper without re-implementing it.
///
/// - `ReputationParagonWatchBar_OnEnter(self)` at
///   `Blizzard_UIPanels_Game/Mainline/ReputationFrame.lua:686` — declared
///   BY THE SIBLING ADDON. Hover handler that gates on
///   `C_Reputation.IsFactionParagonForCurrentPlayer(self.factionID)`,
///   then anchors `EmbeddedItemTooltip` to the watch bar and dispatches
///   `ReputationParagonFrame_SetupParagonTooltip(self)` (a sibling
///   helper at `ReputationFrame.lua:660`). Stores
///   `self.UpdateTooltip = ReputationParagonFrame_SetupParagonTooltip`
///   so the tooltip auto-refreshes on `OnUpdate` ticks while the cursor
///   is held over the bar.
///
/// - `ReputationParagonWatchBar_OnLeave(self)` at
///   `Blizzard_UIPanels_Game/Mainline/ReputationFrame.lua:697` — also
///   declared BY THE SIBLING ADDON. Counterpart that calls
///   `EmbeddedItemTooltip_Hide(EmbeddedItemTooltip)` and clears
///   `self.UpdateTooltip = nil` so the auto-refresh stops.
///
/// **Why this is a cross-addon surface.** `Blizzard_ActionBar/Shared/
/// ReputationBar.lua:187` and `:196` are inside
/// `ReputationStatusBarMixin:OnEnter` and `:OnLeave` — they CALL the
/// two `ReputationParagon*` globals but do NOT declare them. The TOC
/// for `Blizzard_ActionBar` lists `Blizzard_UIPanels_Game` as a
/// dependency, AND `Blizzard_UIPanels_Game` is in
/// `panel_fixtures.rs`'s `PANEL_ADDONS` baseline preload (the harness
/// loads the panel addons OUTSIDE the closure-walked `loaded` slice
/// passed to the test callback). Both wiring paths must hold for the
/// `ReputationParagonWatchBar_*` globals to be visible at the moment
/// `Blizzard_ActionBar` finishes loading.
///
/// **Note on the gap dependency.** The PLAN line carries no
/// `(depends-on:)` suffix, so NONE of these globals are pre-published
/// by the simulator's `runtime_surface_bootstrap.lua` / Rust env-init.
/// The contract is therefore one-way: a nil reading proves either (a)
/// the publishing addon's file chunk failed (Blizzard_ActionBar for the
/// Artifact helper, Blizzard_UIPanels_Game for the two ReputationParagon
/// handlers), OR (b) the TOC dependency wiring regressed —
/// `Blizzard_UIPanels_Game` loaded AFTER `Blizzard_ActionBar` instead of
/// before, leaving the `ReputationParagonWatchBar_*` globals nil at the
/// moment the harness probes them, OR (c) the panel-baseline preload
/// in `panel_fixtures.rs` dropped `Blizzard_UIPanels_Game` (which would
/// also break every other test that depends on the panel-addons
/// baseline — multi-test failure mode), OR (d) Blizzard moved the
/// `Mainline/ArtifactBar.lua` chunk into a TOC flavor branch the
/// harness doesn't pick up (e.g. moved to a Classic/Cataclysm-only
/// path while the test still loads Mainline).
#[test]
fn action_bar_plan_named_bar_helper_globals_are_functions() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for name in PLAN_NAMED_BAR_HELPER_GLOBALS {
            let is_function = env
                .eval::<bool>(&format!(r#"return type({name}) == "function""#))
                .expect("file-scope global type probe must run cleanly");

            assert!(
                is_function,
                "Global `{name}` MUST be a function after the startup-shape harness loads \
                 `Blizzard_ActionBar`. This batch is a MIXED-PUBLISHER set: \
                 `ArtifactBarGetNumArtifactTraitsPurchasableFromXP` is declared by \
                 `Blizzard_ActionBar` itself at `Mainline/ArtifactBar.lua:96` (Mainline-only — \
                 gated by the addon's TOC flavor selection), while \
                 `ReputationParagonWatchBar_OnEnter` and `_OnLeave` are declared by the SIBLING \
                 addon `Blizzard_UIPanels_Game` at `Mainline/ReputationFrame.lua:686` and `:697` \
                 — `Blizzard_ActionBar/Shared/ReputationBar.lua:187` and `:196` consume them from \
                 `ReputationStatusBarMixin:OnEnter` / `:OnLeave` but do NOT declare them. The PLAN \
                 line has no `(depends-on:)` suffix, so NONE of these are pre-published by the \
                 simulator's `runtime_surface_bootstrap.lua` / Rust env-init — the contract is \
                 one-way. A nil reading proves either (a) the publishing addon's file chunk \
                 failed (Blizzard_ActionBar's `Mainline/ArtifactBar.lua` for the Artifact helper, \
                 Blizzard_UIPanels_Game's `Mainline/ReputationFrame.lua` for the two \
                 ReputationParagon handlers — load.rs's \
                 `action_bar_load_emits_no_lane_specific_lua_errors` would also fail for the \
                 ActionBar publisher), OR (b) the TOC dependency wiring regressed and \
                 `Blizzard_UIPanels_Game` loaded AFTER `Blizzard_ActionBar` instead of before — \
                 leaving the cross-addon globals nil at probe time, OR (c) the panel-baseline \
                 preload in `tests/common/panel_fixtures.rs` dropped `Blizzard_UIPanels_Game` \
                 from `PANEL_ADDONS` (would also break every other panel-baseline-dependent test \
                 — multi-test failure mode), OR (d) the addon's TOC flavor selection no longer \
                 picks up `Mainline/ArtifactBar.lua` (e.g. Blizzard moved the Artifact UI to a \
                 different TOC flavor), OR (e) Blizzard re-shaped one of the surfaces — moved \
                 `ArtifactBarGetNumArtifactTraitsPurchasableFromXP` onto `C_ArtifactUI` as a \
                 method (which would break legacy `Blizzard_ArtifactUI` consumers), or moved the \
                 ReputationParagonWatchBar handlers onto a `ReputationParagonWatchBarMixin` frame \
                 mixin (which would break ActionBar's direct global call sites at \
                 `Shared/ReputationBar.lua:187/196`). Got `type({name}) == \"function\"` returned \
                 false."
            );
        }
    });
}
