//! Bag button workarounds for load-order issues.
//!
//! Blizzard_MainMenuBarBagButtons loads before Blizzard_UIPanels_Game and
//! Blizzard_FrameXMLUtil, so several functions needed by bag button OnLoad
//! don't exist yet. These workarounds re-run initialization after all addons
//! are loaded.

use super::WowLuaEnv;

/// Re-run bag button initialization and fix BagsBar anchor.
pub fn init_bag_bar(env: &WowLuaEnv) {
    fix_bags_bar_anchor(env);
    update_bag_button_textures(env);
}

/// `Blizzard_TokenUI` is an on-demand addon that creates `BackpackTokenFrame`.
/// `ContainerFrameSettingsManager:SetTokenTrackerOwner()` crashes if
/// `self.TokenTracker` is nil.  Create a stub frame to avoid the nil index.
pub fn init_bag_token_tracker(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if ContainerFrameSettingsManager and not ContainerFrameSettingsManager.TokenTracker then
            local f = CreateFrame("Frame", "BackpackTokenFrame", UIParent)
            f.ShouldShow = function() return false end
            f.MarkDirty = function() end
            f.CleanDirty = function() end
            f.SetIsCombinedInventory = function() end
            ContainerFrameSettingsManager.TokenTracker = f
        end
    "#,
    );
}

/// Re-anchor BagsBar to MicroButtonAndBagsBar now that it exists.
pub fn fix_bags_bar_anchor(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if BagsBar and MicroButtonAndBagsBar then
            BagsBar:ClearAllPoints()
            BagsBar:SetPoint("TOPRIGHT", MicroButtonAndBagsBar, "TOPRIGHT", 0, 0)
        end
    "#,
    );
}

/// Fix ItemContextOverlay showing on bag buttons after startup events.
///
/// `ItemButton`'s `PostOnShow` calls `UpdateItemContextMatching()` which references
/// `ItemButtonUtil` (from `Blizzard_FrameXMLUtil`). But bag buttons load before
/// `Blizzard_FrameXMLUtil`, so `PostOnShow` errors out and `itemContextMatchResult`
/// stays nil. When `PLAYER_ENTERING_WORLD` later triggers `SetMatchesSearch` →
/// `GetItemContextOverlayMode`, `nil ~= DoesNotApply` evaluates to true, showing
/// a black 80% opacity overlay on each bag icon. Re-run `UpdateItemContextMatching`
/// after events so `itemContextMatchResult` is properly set to `DoesNotApply`.
pub fn fix_bag_item_context_overlay(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        local dna = ItemButtonUtil and ItemButtonUtil.ItemContextMatchResult
            and ItemButtonUtil.ItemContextMatchResult.DoesNotApply
        if not dna then return end
        local function fixBtn(btn)
            if not btn then return end
            btn.itemContextMatchResult = dna
            if btn.UpdateItemContextOverlay then
                pcall(btn.UpdateItemContextOverlay, btn)
            end
        end
        if MainMenuBarBagManager and MainMenuBarBagManager.allBagButtons then
            for _, btn in ipairs(MainMenuBarBagManager.allBagButtons) do
                fixBtn(btn)
            end
        end
        fixBtn(MainMenuBarBackpackButton)
    "#,
    );
}

/// Re-run bag button initialization now that PaperDollItemSlotButton_OnLoad exists.
///
/// During initial OnLoad, `PaperDollItemSlotButton_OnLoad` was a no-op stub
/// (Blizzard_UIPanels_Game hadn't loaded yet). Re-run it on each bag button
/// to set the correct slot ID, backgroundTextureName, etc., then update
/// textures via `PaperDollItemSlotButton_Update` and `UpdateTextures`.
fn update_bag_button_textures(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not MainMenuBarBagManager or not MainMenuBarBagManager.allBagButtons then
            return
        end
        for _, btn in ipairs(MainMenuBarBagManager.allBagButtons) do
            -- Backpack has its own OnLoadInternal that doesn't call
            -- PaperDollItemSlotButton_OnLoad (name pattern doesn't match,
            -- producing wrong slot ID and head-slot icon texture).
            local isBackpack = btn:IsBackpack()
            if not isBackpack and PaperDollItemSlotButton_OnLoad then
                pcall(PaperDollItemSlotButton_OnLoad, btn)
            end
            if not isBackpack and PaperDollItemSlotButton_Update then
                pcall(PaperDollItemSlotButton_Update, btn)
            end
            if btn.UpdateTextures then
                pcall(btn.UpdateTextures, btn)
            end
        end
    "#,
    );
}
