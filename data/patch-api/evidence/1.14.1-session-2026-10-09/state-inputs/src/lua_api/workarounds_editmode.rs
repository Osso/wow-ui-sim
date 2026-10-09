//! EditMode layout workarounds.
//!
//! Patches EditModeManagerFrame to apply preset layout anchors to all 43
//! registered system frames. The real UpdateLayoutInfo crashes partway through
//! due to cascading dependencies, so we manually set up layoutInfo and replay
//! the native initial-anchor phase before per-system updates.

use super::WowLuaEnv;
use std::time::Instant;

const SETUP_LAYOUT_INFO_LUA: &str = include_str!("workarounds/editmode/setup_layout_info.lua");

const APPLY_SYSTEM_ANCHORS_LUA: &str =
    include_str!("workarounds/editmode/apply_system_anchors.lua");

const FIX_ACTION_BAR_NAN_SIZE_LUA: &str =
    include_str!("workarounds/editmode/fix_action_bar_nan_size.lua");

/// Initialize EditMode layout info and apply system anchors.
///
/// EDIT_MODE_LAYOUTS_UPDATED fires during startup but UpdateLayoutInfo
/// crashes partway through (cascading dependencies). This leaves
/// layoutInfo nil. Manually set it up from C_EditMode.GetLayouts() +
/// preset layouts, then replay native initial anchors before the custom
/// per-system pass and post-bootstrap action-bar managed positioning. Also ensures
/// accountSettings is initialized so CanEnterEditMode() returns true.
pub fn init_edit_mode_layout(env: &WowLuaEnv) {
    log_step(env, "setup_layout_info", || {
        setup_layout_info(env);
    });
    log_step(env, "apply_system_anchors", || {
        apply_system_anchors(env);
    });
    log_step(env, "finalize_action_bar_positions", || {
        finalize_action_bar_positions(env);
    });
    log_step(env, "invoke_anchor_changed_hooks", || {
        invoke_anchor_changed_hooks(env);
    });
    log_step(env, "fix_action_bar_nan_size", || {
        fix_action_bar_nan_size(env);
    });
    log_step(env, "fix_action_bar_scale", || {
        fix_action_bar_scale(env);
    });
}

/// Re-run the normal player-frame anchor path after startup events settle.
///
/// The player frame's edit-mode anchor path re-applies the cast bar anchor as
/// a side effect. Post-event startup leaves that path unapplied in some
/// headless/test flows, so explicitly replay it here instead of directly
/// attaching the cast bar.
pub fn reapply_player_frame_anchor(env: &WowLuaEnv) {
    log_step(env, "reapply_player_frame_anchor", || {
        refresh_player_frame_state(env);
    });
}

/// Re-run the player frame refresh after edit-mode anchoring settles.
///
/// `ApplySystemAnchor` can leave the player frame's health bar bound to the
/// wrong unit. Replaying the normal art update and then re-seeding the bar
/// restores the live health binding without forcing a full art swap.
fn refresh_player_frame_state(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if PlayerFrame and type(PlayerFrame_UpdateArt) == "function" then
            pcall(PlayerFrame_UpdateArt, PlayerFrame)
        end
        local healthBar = PlayerFrame and PlayerFrame_GetHealthBar and PlayerFrame_GetHealthBar()
        if healthBar and type(UnitFrameHealthBar_SetUnit) == "function" then
            pcall(UnitFrameHealthBar_SetUnit, healthBar, "player")
        end
        if PlayerFrame and type(UnitFrame_Update) == "function" then
            pcall(UnitFrame_Update, PlayerFrame)
        end
        if healthBar and type(UnitFrameHealthBar_Update) == "function" then
            pcall(UnitFrameHealthBar_Update, healthBar, "player")
        end
    "#,
    );
}

/// Populate layoutInfo from C_EditMode.GetLayouts() + preset layouts.
fn setup_layout_info(env: &WowLuaEnv) {
    let _ = env.exec(SETUP_LAYOUT_INFO_LUA);
}

/// Apply preset layout anchors and settings to all EditMode system frames.
fn apply_system_anchors(env: &WowLuaEnv) {
    let _ = env.exec(APPLY_SYSTEM_ANCHORS_LUA);
}

/// Clear bootstrap layout guard and replay Blizzard's managed action-bar pass.
fn finalize_action_bar_positions(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if EditModeManagerFrame then
            EditModeManagerFrame.layoutApplyInProgress = false
            if EditModeManagerFrame.UpdateActionBarPositions then
                pcall(EditModeManagerFrame.UpdateActionBarPositions, EditModeManagerFrame)
            end
        end
    "#,
    );
}

/// Replay Blizzard's post-apply anchor-changed broadcast.
///
/// Blizzard's layout-apply pass ends with
/// `InvokeOnAnyEditModeSystemAnchorChanged(force)` so systems whose layout
/// depends on their final screen position re-run it. MicroMenu re-anchors
/// itself inside MicroMenuContainer based on which screen quadrant the
/// container's center is in; our workaround applies saved anchors after each
/// system's `UpdateSystem`, so without this broadcast the micro menu keeps
/// the quadrant computed from the container's pre-anchor position and ends
/// up one QueueStatusButton slot away from where Blizzard's layout puts it
/// on the first interactive re-layout.
pub(crate) fn invoke_anchor_changed_hooks(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if EditModeManagerFrame
            and EditModeManagerFrame.InvokeOnAnyEditModeSystemAnchorChanged then
            local force = true
            pcall(
                EditModeManagerFrame.InvokeOnAnyEditModeSystemAnchorChanged,
                EditModeManagerFrame,
                force
            )
        end
    "#,
    );
}

/// Fix MainActionBar NaN size after UpdateSystems.
///
/// Layout() produces NaN because the bar has no size yet when children try
/// to resolve anchors relative to it (chicken-and-egg). Compute the bar
/// size directly from the button grid; saved EditMode anchors are applied
/// separately and should not be repacked by Blizzard's automatic bar layout.
fn fix_action_bar_nan_size(env: &WowLuaEnv) {
    let _ = env.exec(FIX_ACTION_BAR_NAN_SIZE_LUA);
}

/// Force MainActionBar scale=1 after EditMode initialization.
fn fix_action_bar_scale(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if MainActionBar then MainActionBar:SetScale(1) end
    "#,
    );
}

fn log_with_timestamp(env: &WowLuaEnv, message: &str) {
    let start_time = env.state().borrow().start_time;
    eprintln!("{} {}", crate::logging::elapsed_prefix(start_time), message);
}

fn log_step(env: &WowLuaEnv, label: &str, apply_step: impl FnOnce()) {
    log_with_timestamp(env, &format!("[EditMode] starting {label}"));
    let started = Instant::now();
    apply_step();
    log_with_timestamp(
        env,
        &format!("[EditMode] finished {label} in {:.2?}", started.elapsed()),
    );
}

/// Patch EditModeManagerFrame after addon loading.
///
/// Before EDIT_MODE_LAYOUTS_UPDATED fires, layoutInfo is nil. Guard
/// GetActiveLayoutInfo with a fallback. Replace InitSystemAnchors with a
/// custom implementation that reads the active preset layout, applies
/// anchorInfo to all 43 registered system frames, then calls UpdateSystems
/// to apply settings (orientation, num rows, etc.) through the normal
/// Blizzard code path. Per-frame errors are caught by secureexecuterange.
///
/// Also wraps EnterEditMode/ExitEditMode with pcall protection so edit
/// mode can activate even when subsystems crash.
pub fn patch_edit_mode_manager(env: &WowLuaEnv) {
    patch_apply_system_anchor_nil_guard(env);
    fix_set_point_override_3arg(env);
    guard_action_bar_limits(env);
    patch_default_anchor(env);
    patch_enter_exit_edit_mode(env);
}

#[cfg(test)]
#[path = "workarounds_editmode_tests.rs"]
mod tests;

/// Guard ApplySystemAnchor against nil systemInfo.
///
/// `EditModePlayerFrameSystemMixin:ApplySystemAnchor` calls
/// `PlayerCastingBarFrame:ApplySystemAnchor()` as a side effect, but
/// PlayerCastingBarFrame (and 13 other frames) have nil systemInfo because
/// the preset layout doesn't include entries for all system types.
/// The original Blizzard code at EditModeSystemTemplates.lua:375 accesses
/// `self.systemInfo.anchorInfo` without a nil check.
fn patch_apply_system_anchor_nil_guard(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not EditModeSystemMixin then return end
        local orig = EditModeSystemMixin.ApplySystemAnchor
        function EditModeSystemMixin:ApplySystemAnchor()
            if not self.systemInfo then return end
            return orig(self)
        end
    "#,
    );
}

const SYNC_EDIT_MODE_SET_POINT_OVERRIDES_LUA: &str =
    include_str!("workarounds/editmode/sync_set_point_overrides.lua");

/// Fix SetPointOverride to handle the 3-arg SetPoint form and keep Rust layout in sync.
///
/// Blizzard's `SetPointOverride(point, relativeTo, relativePoint, offsetX, offsetY)`
/// always forwards all 5 args to `SetPointBase`. But `VerticalLayoutMixin` and other
/// code calls the 3-arg form: `SetPoint("TOPRIGHT", x, y)`, where x,y are offsets
/// relative to the parent. In that case `relativeTo` receives a number (the x offset)
/// and `relativePoint` receives a number (the y offset), which is wrong.
///
/// OnSystemLoad already ran during addon loading, copying the original
/// SetPointOverride into each frame's fields table. That Lua override updates
/// EditMode's anchor bookkeeping, but it can bypass the simulator's Rust
/// anchor state that drives rendering. We replace it on each registered frame
/// and mirror the final Lua point through the real Rust SetPoint method.
fn fix_set_point_override_3arg(env: &WowLuaEnv) {
    let _ = env.exec(SYNC_EDIT_MODE_SET_POINT_OVERRIDES_LUA);
}

/// Guard action bar positioning methods against nil frame positions.
///
/// `GetRightActionBarTopLimit` calls `MinimapCluster:GetBottom() - 10`, which
/// returns nil when MinimapCluster hasn't been laid out yet. Similarly,
/// `GetRightActionBarBottomLimit` calls `MicroButtonAndBagsBar:GetTop() + 24`.
/// During the real UpdateLayoutInfo (EDIT_MODE_LAYOUTS_UPDATED event), layout
/// hasn't run yet, so these frame positions are nil. Fall back to UIParent
/// boundaries when the frame position isn't available.
fn guard_action_bar_limits(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not EditModeManagerFrameMixin then return end
        function EditModeManagerFrameMixin:GetRightActionBarTopLimit()
            if MinimapCluster and MinimapCluster.IsInDefaultPosition
                    and MinimapCluster:IsInDefaultPosition() then
                local bottom = MinimapCluster:GetBottom()
                if bottom then return bottom - 10 end
            end
            return UIParent:GetTop()
        end
        function EditModeManagerFrameMixin:GetRightActionBarBottomLimit()
            if MicroButtonAndBagsBar then
                local top = MicroButtonAndBagsBar:GetTop()
                if top then return top + 24 end
            end
            return 0
        end
    "#,
    );
}

/// Provide a fallback GetDefaultAnchor for frames that query it.
fn patch_default_anchor(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not EditModeManagerFrame then return end
        function EditModeManagerFrame:GetDefaultAnchor(frame)
            return {
                point = "TOPRIGHT",
                relativeTo = UIParent,
                relativePoint = "TOPRIGHT",
                offsetX = -205,
                offsetY = -13,
            }
        end
    "#,
    );
}

const PATCH_ENTER_EXIT_EDIT_MODE_LUA: &str =
    include_str!("workarounds/editmode/patch_enter_exit_edit_mode.lua");

/// Wrap EnterEditMode/ExitEditMode with pcall protection.
///
/// EnterEditMode calls crash-prone functions: ShowSystemSelections
/// iterates 43 frames, AccountSettings does 30+ Setup/Refresh calls.
/// Wrapping each step with pcall lets edit mode activate even when
/// individual subsystems fail.
fn patch_enter_exit_edit_mode(env: &WowLuaEnv) {
    let _ = env.exec(PATCH_ENTER_EXIT_EDIT_MODE_LUA);
}
