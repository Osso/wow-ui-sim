#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::screen::ScreenKind;

// Preservation only: no modeled covenant reward catalog is available to seed.
#[test]
fn retail_original_renown_toast_handles_empty_rewards() {
    let ui = blizzard_ui_dir();
    let discovered = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    assert!(
        discovered
            .iter()
            .any(|(name, _)| name == "Blizzard_CovenantToasts")
    );
    let env = new_blizzard_addon_env(&ui);
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_CovenantToasts"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_CovenantToasts"));
    env.exec(r#"
        local setup = CovenantRenownToastMixin.SetupRewardVisuals
        local source = debug.getinfo(setup, "S").source:gsub("\\", "/")
        assert(source:find("Blizzard_CovenantToasts/Blizzard_CovenantRenownToast.lua", 1, true), source)
        local frame = CreateFrame("Frame")
        frame.RewardIcon = frame:CreateTexture()
        frame.RewardIconMouseOver = CreateFrame("Frame", nil, frame)
        frame.RewardIconRing = frame:CreateTexture()
        frame.RewardDescription = frame:CreateFontString()
        frame.RewardDescription:SetText("stale")
        setup(frame, 2507, 7)
        assert(not frame.RewardIcon:IsShown())
        assert(not frame.RewardIconMouseOver:IsShown())
        assert(not frame.RewardIconRing:IsShown())
        assert(frame.RewardDescription:GetText() == nil)
    "#).expect("original toast should clear its visuals for the existing empty result");
}
