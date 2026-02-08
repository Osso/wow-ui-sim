//! Post-load Lua workarounds for Blizzard code that depends on
//! unimplemented engine features (AnimationGroups, EditMode, etc.).

use super::WowLuaEnv;

/// Apply workarounds that must run after startup events.
///
/// Some workarounds (like BagsBar anchoring) get undone by event handlers
/// (e.g. EDIT_MODE_LAYOUTS_UPDATED repositions managed frames).
pub fn apply_post_event(env: &WowLuaEnv) {
    fix_bags_bar_anchor(env);
}

/// Apply all post-load workarounds. Called after addon loading, before events.
pub fn apply(env: &WowLuaEnv) {
    let _ = env.exec("UpdateMicroButtons = function() end");
    // CompactUnitFrame helpers may not be defined if CompactUnitFrame.lua fails to load fully
    let _ = env.exec(
        "if not CompactUnitFrame_GetOptionDisplayOnlyDispellableDebuffs then \
         CompactUnitFrame_GetOptionDisplayOnlyDispellableDebuffs = function() return false end end",
    );
    patch_map_canvas_scroll(env);
    patch_gradual_animated_status_bar(env);
    patch_spell_alert_animations(env);
    patch_character_frame_subframes(env);
    setup_managed_frame_containers(env);
    init_objective_tracker(env);
    show_chat_frame(env);
    init_bag_bar(env);
    hide_super_tracked_frame(env);
}

/// SuperTrackedFrame shows a quest navigation arrow positioned by the engine's
/// 3D-to-screen projection (C_SuperTrack). Without the 3D world, OnUpdate never
/// repositions it, so the icon renders at default (0,0) in the top-left corner.
fn hide_super_tracked_frame(env: &WowLuaEnv) {
    let _ = env.exec("if SuperTrackedFrame then SuperTrackedFrame:Hide() end");
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

/// Stub AnimationGroup methods on ActionButtonSpellAlert frames.
///
/// ActionButtonSpellAlertManager uses local functions (ShowAlert/HideAlert)
/// that access alertFrame.ProcStartAnim, an AnimationGroup defined in XML
/// with parentKey. The simulator doesn't create AnimationGroups from templates,
/// so these are nil. Replace the manager methods with no-ops since spell alert
/// animations aren't needed in the simulator.
fn patch_spell_alert_animations(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if ActionButtonSpellAlertManager then
            function ActionButtonSpellAlertManager:ShowAlert() end
            function ActionButtonSpellAlertManager:HideAlert() end
        end
    "#,
    );
}

/// Initialize ObjectiveTrackerManager if EventRegistry dispatch didn't reach it.
/// The manager registers via EventRegistry:RegisterFrameEventAndCallback which
/// needs the full event dispatch chain. Call OnPlayerEnteringWorld directly,
/// then fire quest data callbacks so async title population works.
fn init_objective_tracker(env: &WowLuaEnv) {
    hide_empty_managed_frames(env);
    setup_tracker_frame(env);
    patch_tracker_animations(env);
    start_objective_tracker(env);
    populate_quest_titles(env);
    update_managed_frame_containers(env);
}

/// Hide managed frames that have no content in the simulator.
///
/// BossTargetFrameContainer and DurabilityFrame show during startup events
/// and add themselves to UIParentRightManagedFrameContainer via OnShow →
/// AddManagedFrame. Their height (305px + 75px) pushes ObjectiveTrackerFrame
/// out of the visible area. Hide them before OTF setup.
fn hide_empty_managed_frames(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        local frames = { "BossTargetFrameContainer", "DurabilityFrame" }
        for _, name in ipairs(frames) do
            local f = _G[name]
            if f and f.Hide then
                f:Hide()
                -- Prevent OnShow from re-showing during events
                f.ignoreInLayout = true
            end
        end
    "#,
    );
}

/// Call UpdateManagedFrames on both containers to ensure all visible
/// managed frames get laid out after ObjectiveTracker initialization.
fn update_managed_frame_containers(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if UIParentRightManagedFrameContainer
            and UIParentRightManagedFrameContainer.UpdateManagedFrames then
            UIParentRightManagedFrameContainer:UpdateManagedFrames()
        end
        if UIParentBottomManagedFrameContainer
            and UIParentBottomManagedFrameContainer.UpdateManagedFrames then
            UIParentBottomManagedFrameContainer:UpdateManagedFrames()
        end
    "#,
    );
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

/// Position UIParentRightManagedFrameContainer and UIParentBottomManagedFrameContainer.
///
/// In WoW, `UIParent_ManageFramePositions()` calls through FramePositionDelegate
/// and EditModeUtil to position these containers. Since EditMode isn't implemented,
/// we position them directly with the default offsets.
fn setup_managed_frame_containers(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        -- Position UIParentRightManagedFrameContainer
        -- Offsets match EditModeUtil:GetRightContainerAnchor() defaults:
        -- TOPRIGHT, UIParent, TOPRIGHT, xOffset=-5, yOffset=-260
        if UIParentRightManagedFrameContainer then
            UIParentRightManagedFrameContainer:ClearAllPoints()
            UIParentRightManagedFrameContainer:SetPoint(
                "TOPRIGHT", UIParent, "TOPRIGHT", -5, -260
            )
            local minimapHeight = 0
            if MinimapCluster and MinimapCluster.GetHeight then
                minimapHeight = MinimapCluster:GetHeight()
            end
            UIParentRightManagedFrameContainer.fixedHeight =
                UIParent:GetHeight() - minimapHeight - 100
            UIParentRightManagedFrameContainer:Layout()
            if UIParentRightManagedFrameContainer.BottomManagedLayoutContainer then
                UIParentRightManagedFrameContainer.BottomManagedLayoutContainer:Layout()
            end
        end

        -- Position UIParentBottomManagedFrameContainer
        if UIParentBottomManagedFrameContainer then
            UIParentBottomManagedFrameContainer.fixedWidth = 573
            UIParentBottomManagedFrameContainer:ClearAllPoints()
            UIParentBottomManagedFrameContainer:SetPoint(
                "BOTTOM", UIParent, "BOTTOM", 0, 90
            )
            UIParentBottomManagedFrameContainer:Layout()
            if UIParentBottomManagedFrameContainer.BottomManagedLayoutContainer then
                UIParentBottomManagedFrameContainer.BottomManagedLayoutContainer:Layout()
            end
        end
    "#,
    );
}

/// Add ObjectiveTrackerFrame to its managed frame container and set its height.
///
/// The container system (UIParentRightManagedFrameContainer) handles positioning
/// via UIParentManagedFrameMixin's OnShow → AddManagedFrame → Layout chain.
/// After layout places OTF with an anchor, we compute height from the container's
/// height minus OTF's vertical offset within it.
fn setup_tracker_frame(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        local otf = ObjectiveTrackerFrame
        if not otf then return end
        -- Ensure layoutIndex is set (should come from XML KeyValue but may need fallback)
        if not otf.layoutIndex then otf.layoutIndex = 50 end
        -- AddManagedFrame checks IsInDefaultPosition() and skips frames not in
        -- default position. Since EditMode isn't initialized, the mixin's
        -- IsInDefaultPosition() returns false. Override so the container accepts it.
        otf.IsInDefaultPosition = function() return true end
        otf:Show()
        -- Explicitly add to the managed frame container. The OnShow handler
        -- may not fire correctly, so call AddManagedFrame directly.
        -- This reparents OTF into the container and calls Layout() to set anchors.
        local lp = otf.layoutParent
        if lp and lp.AddManagedFrame then
            pcall(lp.AddManagedFrame, lp, otf)
        end
        -- Compute height from container height minus OTF's vertical offset.
        -- UpdateHeight() does parentHeight + offsetY, but calling it triggers
        -- layout cycles. Compute it directly instead.
        local _, _, _, _, offsetY = otf:GetPoint(1)
        if offsetY and lp then
            local h = lp:GetHeight() + offsetY
            if h < 100 then h = 400 end
            otf:SetHeight(h)
        end
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

/// Show ChatFrame1 and set DEFAULT_CHAT_FRAME after addon loading.
///
/// The chat addons create ChatFrame1 via XML but it starts hidden.
/// Position it at bottom-left like in the real WoW client.
fn show_chat_frame(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if ChatFrame1 then
            ChatFrame1:Show()
            DEFAULT_CHAT_FRAME = ChatFrame1
            ChatFrame1:ClearAllPoints()
            ChatFrame1:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 32, 32)
            ChatFrame1:SetSize(430, 120)
        end
    "#,
    );
    start_fake_chat(env);
}

/// Start periodic fake chat messages across four channels, staggered by 5s.
///
/// Uses C_Timer.NewTicker to add pre-formatted messages directly via
/// AddMessage, bypassing the event system which needs many unimplemented
/// helpers (GetPlayerLink, ReplaceIconAndGroupExpressions, etc.).
fn start_fake_chat(env: &WowLuaEnv) {
    register_fake_chat_data(env);
    schedule_fake_chat_tickers(env);
}

/// Register message pools, name lists, and helper functions as globals
/// so the chat tickers can reference them.
fn register_fake_chat_data(env: &WowLuaEnv) {
    register_fake_chat_messages(env);
    register_fake_chat_names(env);
}

/// Populate `_FakeChat.msgs` with message pools for each channel.
fn register_fake_chat_messages(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not ChatFrame1 then return end
        _FakeChat = { msgs = {}, names = {}, idx = {} }
        _FakeChat.msgs.general = {
            "Anyone know where the portal trainer is?",
            "LFM Deadmines, need healer",
            "WTS [Copper Bar] x20, 5g each",
            "How do I get to Ironforge from here?",
            "Is the Darkmoon Faire up this week?",
            "Just hit level 60!",
            "What's the fastest way to level cooking?",
            "Any good guilds recruiting?",
        }
        _FakeChat.msgs.trade = {
            "WTS [Enchant Weapon - Crusader] your mats + 10g tip",
            "WTB [Large Brilliant Shard] x5, paying 3g each",
            "LF Blacksmith to craft [Arcanite Reaper], have mats",
            "WTS [Flask of the Titans] 45g, cheap!",
            "WTB [Righteous Orb] x2, PST with price",
            "Selling port to Dalaran, 1g",
        }
        _FakeChat.msgs.say = {
            "Anyone else lagging?", "Thanks for the group!",
            "Where did that quest NPC go?",
            "I think I took a wrong turn somewhere",
            "Wow, this place is huge", "Can someone help with this elite?",
        }
        _FakeChat.msgs.guild = {
            "Hey everyone!", "Anyone up for a dungeon run?",
            "Grats on the new gear!", "Guild bank has some free enchanting mats",
            "Raid signup is up on the calendar",
            "I just finished the attunement quest chain",
        }
    "#,
    );
}

/// Populate `_FakeChat.names`, index counters, and the `pick` helper.
fn register_fake_chat_names(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not _FakeChat then return end
        _FakeChat.names.general = {"Thunderfury", "Moonwhisper", "Stabbymcstab", "Healbot", "Tanklord"}
        _FakeChat.names.trade = {"Goldmaker", "Craftypants", "Auctioneer", "Bankalt"}
        _FakeChat.names.say = {"Legolas", "Arthasdklol", "Pwnstar", "Noobslayer"}
        _FakeChat.names.guild = {"Valorheart", "Shieldmaiden", "Firestorm", "Arcanewing"}
        _FakeChat.idx = {general = 1, trade = 1, say = 1, guild = 1}
        function _FakeChat:pick(channel)
            local list = self.msgs[channel]
            local i = self.idx[channel]
            self.idx[channel] = (i % #list) + 1
            return list[i], self.names[channel][math.random(#self.names[channel])]
        end
    "#,
    );
}

/// Schedule four staggered C_Timer tickers that post to ChatFrame1.
fn schedule_fake_chat_tickers(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not _FakeChat then return end
        local fc = _FakeChat
        local function post(channel, prefix, r, g, b)
            local msg, name = fc:pick(channel)
            ChatFrame1:AddMessage(prefix ..
                "|Hplayer:" .. name .. "|h[" .. name .. "]|h: " .. msg,
                r, g, b)
        end
        -- General (0s offset, light orange)
        C_Timer.After(0, function() C_Timer.NewTicker(40, function()
            post("general", "|Hchannel:General|h[1. General]|h ", 1.0, 0.75, 0.5)
        end) end)
        -- Trade (5s offset, light orange)
        C_Timer.After(5, function() C_Timer.NewTicker(40, function()
            post("trade", "|Hchannel:Trade|h[2. Trade]|h ", 1.0, 0.75, 0.5)
        end) end)
        -- Say (10s offset, white — uses "says:" format)
        C_Timer.After(10, function() C_Timer.NewTicker(40, function()
            local msg, name = fc:pick("say")
            ChatFrame1:AddMessage(
                "|Hplayer:" .. name .. "|h[" .. name .. "]|h says: " .. msg,
                1.0, 1.0, 1.0)
        end) end)
        -- Guild (15s offset, green)
        C_Timer.After(15, function() C_Timer.NewTicker(40, function()
            post("guild", "|Hchannel:Guild|h[Guild]|h ", 0.25, 1.0, 0.25)
        end) end)
    "#,
    );
}

/// Re-run bag button initialization that failed during OnLoad.
///
/// Blizzard_MainMenuBarBagButtons loads before both Blizzard_MicroMenu and
/// Blizzard_UIPanels_Game. This causes two problems:
///
/// 1. BagsBar's anchor `relativeTo="MicroButtonAndBagsBar"` resolves to nil
///    because MicroButtonAndBagsBar doesn't exist yet, placing BagsBar at
///    UIParent's TOPRIGHT (top of screen instead of bottom-right).
///
/// 2. `ContainerFrame_GetContainerNumSlots` and `PaperDollItemSlotButton_OnLoad`
///    don't exist when bag button OnLoad fires, so `UpdateTextures()` fails.
///
/// Fix both by re-anchoring BagsBar and re-calling UpdateTextures after all
/// addons are loaded.
fn init_bag_bar(env: &WowLuaEnv) {
    fix_bags_bar_anchor(env);
    update_bag_button_textures(env);
}

/// Re-anchor BagsBar to MicroButtonAndBagsBar now that it exists.
fn fix_bags_bar_anchor(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if BagsBar and MicroButtonAndBagsBar then
            BagsBar:ClearAllPoints()
            BagsBar:SetPoint("TOPRIGHT", MicroButtonAndBagsBar, "TOPRIGHT", 0, 0)
        end
    "#,
    );
}

/// Call UpdateTextures on each bag button now that ContainerFrame functions exist.
fn update_bag_button_textures(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not MainMenuBarBagManager or not MainMenuBarBagManager.allBagButtons then
            return
        end
        for _, btn in ipairs(MainMenuBarBagManager.allBagButtons) do
            if btn.UpdateTextures then
                pcall(btn.UpdateTextures, btn)
            end
        end
        if MainMenuBarBackpackButton and MainMenuBarBackpackButton.OnLoadInternal then
            pcall(MainMenuBarBackpackButton.OnLoadInternal, MainMenuBarBackpackButton)
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
