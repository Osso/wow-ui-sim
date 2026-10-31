#![cfg(feature = "client-retail")]

use wow_ui_sim::loader::{discover_blizzard_addon_closure_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

#[test]
fn loot_specialization_original_popup_selection() {
    let env = WowLuaEnv::new().expect("environment");
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir();
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    wow_ui_sim::xml::register_intrinsic_templates();
    let addons = discover_blizzard_addon_closure_for_screen(
        &ui,
        ScreenKind::Game,
        &["Blizzard_UnitPopupShared"],
    );
    assert!(
        addons
            .iter()
            .any(|(name, _)| name == "Blizzard_UnitPopupShared")
    );
    for (name, toc) in addons {
        let result =
            load_addon(&env.loader_env(), &toc).unwrap_or_else(|error| panic!("{name}: {error}"));
        assert!(result.warnings.is_empty(), "{name}: {:?}", result.warnings);
    }
    env.exec(
        r#"
        local default = UnitPopupLootSpecializationDefaultButtonMixin
        local chosen = UnitPopupLootSpecialization2ButtonMixin
        local source = debug.getinfo(chosen.OnClick, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_UnitPopupShared/UnitPopupSharedButtonMixins.lua', 1, true))
        assert(GetLootSpecialization() == 0 and HasLootSpecializations() == true)
        assert(default:IsChecked({}) and not chosen:IsChecked({}))
        local id = chosen:GetSpecID()
        assert(id > 0)
        local events = 0
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('PLAYER_LOOT_SPEC_UPDATED')
        listener:SetScript('OnEvent', function(_, _, ...)
            assert(select('#', ...) == 0)
            assert(GetLootSpecialization() == id)
            events = events + 1
        end)
        assert(chosen:OnClick({}) == MenuResponse.Close)
        assert(GetLootSpecialization() == 0 and events == 0)
        _G.loot_spec_id = id
        _G.loot_spec_events = function() return events end
    "#,
    )
    .expect("popup request");
    env.fire_on_update(0.0).unwrap();
    env.exec(r#"
        assert(GetLootSpecialization() == loot_spec_id)
        assert(UnitPopupLootSpecialization2ButtonMixin:IsChecked({}))
        assert(loot_spec_events() == 1)
    "#).expect("popup response");
}

#[test]
fn loot_specialization_rate_limit_drops_sixth_request() {
    let env = WowLuaEnv::new().expect("environment");
    env.exec(r#"
        events = 0
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('PLAYER_LOOT_SPEC_UPDATED')
        listener:SetScript('OnEvent', function() events = events + 1 end)
        for i = 1, 5 do SetLootSpecialization(0) end
        SetLootSpecialization(65)
        assert(GetLootSpecialization() == 0 and events == 0)
    "#).unwrap();
    env.fire_on_update(0.0).unwrap();
    env.exec("assert(GetLootSpecialization() == 0 and events == 5)").unwrap();
}

#[test]
fn loot_specialization_usage_error() {
    let env = WowLuaEnv::new().expect("environment");
    let error = env.exec("SetLootSpecialization({})").unwrap_err().to_string();
    assert!(error.contains("Usage: SetLootSpecialization(specializationID)"), "{error}");
}
