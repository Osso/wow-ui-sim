use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

fn load_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    let ui = default_blizzard_ui_addons_path().expect("Blizzard UI cache should be synced");
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    wow_ui_sim::xml::register_intrinsic_templates();
    for (name, toc_path) in discover_blizzard_addons_for_screen(&ui, ScreenKind::Game) {
        load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|error| panic!("[load {name}] FAILED: {error}"));
    }
    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn dropdown_cached_templates_preserve_original_mixin_behavior() {
    let env = load_game_ui();
    env.exec(r#"
        local selection = CreateFrame("DropdownButton", nil, UIParent, "WowStyle1DropdownTemplate")
        selection:UpdateToMenuSelections(nil, {
            { text = "selected", IsSelectionIgnored = function() return false end },
        })
        assert(selection.Text:GetText() == "selected")

        local filter = CreateFrame("DropdownButton", nil, UIParent, "WowStyle1FilterDropdownTemplate")
        filter:SetIsDefaultCallback(function() return false end)
        filter:RegisterMenu(MenuUtil.CreateRootMenuDescription())
        assert(filter.ResetButton:IsShown())
        filter:SetIsDefaultCallback(function() return true end)
        filter:OnMenuAssigned()
        assert(not filter.ResetButton:IsShown())

        local arrow = CreateFrame("DropdownButton", nil, UIParent, "WowStyle1ArrowDropdownTemplate")
        arrow:SetEnabled(false)
        arrow:OnButtonStateChanged()
        assert(arrow.Arrow:GetAtlas() == "common-dropdown-a-button-disabled")
    "#).expect("cached dropdown templates should preserve observable mixin behavior");
}
