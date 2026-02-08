//! Post-load Lua workarounds for Blizzard code that depends on
//! unimplemented engine features (AnimationGroups, EditMode, etc.).

use super::WowLuaEnv;

/// Apply all post-load workarounds. Called after addon loading, before events.
pub fn apply(env: &WowLuaEnv) {
    let _ = env.exec("UpdateMicroButtons = function() end");
    patch_map_canvas_scroll(env);
    patch_gradual_animated_status_bar(env);
    patch_character_frame_subframes(env);
    init_objective_tracker(env);
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

/// Initialize ObjectiveTrackerManager if EventRegistry dispatch didn't reach it.
/// The manager registers via EventRegistry:RegisterFrameEventAndCallback which
/// needs the full event dispatch chain. Call OnPlayerEnteringWorld directly,
/// then fire quest data callbacks so async title population works.
fn init_objective_tracker(env: &WowLuaEnv) {
    setup_tracker_frame(env);
    patch_tracker_animations(env);
    start_objective_tracker(env);
    populate_quest_titles(env);
}

/// Stub AnimationGroup methods on ObjectiveTracker line/block templates.
///
/// The simulator doesn't create AnimationGroups from XML templates.
/// ObjectiveTrackerAnimLineMixin:SetState calls Play() on CheckAnim,
/// GlowAnim, FadeOutAnim etc. Patch SetState to skip animation calls.
fn patch_tracker_animations(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        local animStub = {
            Play = function() end,
            Stop = function() end,
            IsPlaying = function() return false end,
            IsDelaying = function() return false end,
            SetScript = function() end,
        }
        -- Patch SetState to inject animation stubs before calling Play()
        local iconStub = { Hide = function() end, Show = function() end }
        local function patchedSetState(self, state)
            self.CheckAnim = self.CheckAnim or animStub
            self.GlowAnim = self.GlowAnim or animStub
            self.FadeOutAnim = self.FadeOutAnim or animStub
            self.FadeInAnim = self.FadeInAnim or animStub
            self.Icon = self.Icon or iconStub
            if not self.Icon.Hide then self.Icon = iconStub end
            self.state = state
        end
        -- Apply to both the base mixin and all derived mixins
        if ObjectiveTrackerAnimLineMixin then
            ObjectiveTrackerAnimLineMixin.SetState = patchedSetState
        end
        if QuestObjectiveLineMixin then
            QuestObjectiveLineMixin.SetState = patchedSetState
        end
        -- Stub PlayAddAnimation on header mixin
        local function patchedPlayAdd(self)
            self.AddAnim = self.AddAnim or animStub
        end
        if ObjectiveTrackerAnimBlockHeaderMixin then
            ObjectiveTrackerAnimBlockHeaderMixin.PlayAddAnimation = patchedPlayAdd
        end
    "#,
    );
}

/// Position ObjectiveTrackerFrame on the right side and set its height.
///
/// In WoW, UpdateHeight() calculates `UIParent:GetHeight() + offsetY` where
/// offsetY comes from the frame's anchor. The EditMode system sets this up,
/// but the simulator doesn't implement EditMode. Set height directly.
fn setup_tracker_frame(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        local otf = ObjectiveTrackerFrame
        if not otf then return end
        -- Reparent to UIParent directly (normally managed by
        -- UIParentRightManagedFrameContainer layout system)
        otf:SetParent(UIParent)
        -- Position on the right side, below the minimap
        otf:ClearAllPoints()
        otf:SetPoint("TOPRIGHT", UIParent, "TOPRIGHT", -80, -200)
        -- Set height to fill most of the right side (UIParent.height - offset)
        local h = UIParent:GetHeight() - 200
        if h < 100 then h = 500 end
        otf:SetHeight(h)
        otf:Show()
    "#,
    );
}

/// Call OnPlayerEnteringWorld on the tracker manager.
///
/// The first call may fail at AdventureObjectiveTracker:InitModule because
/// POIButtonOwnerMixin:Init isn't applied by the simulator's template system.
/// A second call succeeds because the container's init guard prevents
/// re-initialization, and modules before AdventureObjectiveTracker (including
/// QuestObjectiveTracker) get their parentContainer set.
fn start_objective_tracker(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not ObjectiveTrackerManager
            or not ObjectiveTrackerManager.OnPlayerEnteringWorld then
            return
        end
        -- Call twice: first call may fail at AdventureObjectiveTracker Init,
        -- second call succeeds with container init guard already set.
        for i = 1, 2 do
            pcall(
                ObjectiveTrackerManager.OnPlayerEnteringWorld,
                ObjectiveTrackerManager, true, false
            )
        end
    "#,
    );
}

/// Fire quest data callbacks so QuestMixin async titles get populated,
/// then update the quest tracker module directly.
///
/// We can't rely on `ObjectiveTrackerManager:UpdateAll()` because the
/// container's `Update()` iterates all modules without pcall, and several
/// modules (MawBuffs, ScenarioObjectiveTracker, etc.) crash on missing
/// engine functions. Instead, call the quest module's `Update()` directly
/// with the container's available height.
fn populate_quest_titles(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if QuestEventListener and QuestEventListener.FireCallbacks then
            for _, qid in ipairs({80000, 80001, 80002}) do
                pcall(QuestEventListener.FireCallbacks, QuestEventListener, qid)
            end
        end
        -- Update quest module directly (bypass container loop which
        -- crashes on MawBuffs/ScenarioObjectiveTracker stubs)
        local qt = QuestObjectiveTracker
        if qt and qt.parentContainer then
            local c = qt.parentContainer
            local avail = c:GetAvailableHeight()
            pcall(qt.Update, qt, avail, false)
            -- Force module height and positioning (EndLayout may not
            -- run UpdateHeight due to state/animation issues)
            local h = qt.contentsHeight or 0
            if h > 0 then
                qt:SetHeight(h + (qt.bottomSpacing or 0))
                qt:ClearAllPoints()
                qt:SetPoint("TOP", c, "TOP", 0, -(c.topModulePadding or 0))
                qt:SetPoint("LEFT", c, "LEFT", qt.leftMargin or 0, 0)
                qt:Show()
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
