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
        assert(GetLootSpecialization() == id)
        assert(chosen:IsChecked({}) and not default:IsChecked({}))
        assert(events == 1)
        listener:UnregisterEvent('PLAYER_LOOT_SPEC_UPDATED')
        default:OnClick({})
        assert(GetLootSpecialization() == 0 and default:IsChecked({}) and not chosen:IsChecked({}))
    "#,
    )
    .expect("original loot specialization popup selection");
}
