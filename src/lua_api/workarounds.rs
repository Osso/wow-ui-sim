//! Post-load Lua workarounds for Blizzard code that depends on
//! unimplemented engine features (AnimationGroups, EditMode, etc.).

use super::WowLuaEnv;

/// Apply all post-load workarounds. Called after addon loading, before events.
pub fn apply(env: &WowLuaEnv) {
    let _ = env.exec("UpdateMicroButtons = function() end");
    patch_map_canvas_scroll(env);
    patch_gradual_animated_status_bar(env);
    patch_character_frame_subframes(env);
}

/// MapCanvasScrollControllerMixin:IsZoomingOut/In compare targetScale with
/// GetCanvasScale(), but OnUpdate fires before CalculateScaleExtents sets
/// targetScale. Initialize it on the WorldMapFrame scroll container.
fn patch_map_canvas_scroll(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if WorldMapFrame and WorldMapFrame.ScrollContainer then
            local sc = WorldMapFrame.ScrollContainer
            sc.targetScale = sc.targetScale or 1
            sc.currentScale = sc.currentScale or 1
            sc.zoomLevels = sc.zoomLevels or {{ scale = 1 }}
        end
    "#,
    );
}

/// GradualAnimatedStatusBarTemplate XML defines an AnimationGroup with
/// parentKey="LevelUpMaxAlphaAnimation", but the simulator doesn't create
/// AnimationGroups from templates. Patch existing instances and the mixin.
fn patch_gradual_animated_status_bar(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        local stub = { IsPlaying = function() return false end }

        if StatusTrackingBarManager and StatusTrackingBarManager.barContainers then
            for _, container in ipairs(StatusTrackingBarManager.barContainers) do
                for _, bar in pairs(container.bars or {}) do
                    if bar.StatusBar then
                        if not bar.StatusBar.LevelUpMaxAlphaAnimation then
                            bar.StatusBar.LevelUpMaxAlphaAnimation = stub
                        end
                    end
                end
            end
        end

        if GradualAnimatedStatusBarMixin then
            function GradualAnimatedStatusBarMixin:IsAnimating()
                return self.targetValue and self:GetValue() < self.targetValue
                    or self.gainFinishedAnimation and self.gainFinishedAnimation:IsPlaying()
                    or self.LevelUpMaxAlphaAnimation and self.LevelUpMaxAlphaAnimation:IsPlaying()
                    or self.overrideLevelUpMaxAlphaAnimation and self.overrideLevelUpMaxAlphaAnimation:IsPlaying()
            end
        end
    "#,
    );
}

/// CHARACTERFRAME_SUBFRAMES lists PaperDollFrame, ReputationFrame, TokenFrame.
/// TokenFrame is in Blizzard_TokenUI (not always loaded). Create stub frames
/// for any missing subframes so ShowSubFrame doesn't crash.
fn patch_character_frame_subframes(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if CHARACTERFRAME_SUBFRAMES then
            for _, name in ipairs(CHARACTERFRAME_SUBFRAMES) do
                if not _G[name] then
                    _G[name] = CreateFrame("Frame", name, CharacterFrame)
                    _G[name]:Hide()
                end
            end
        end
    "#,
    );
}
