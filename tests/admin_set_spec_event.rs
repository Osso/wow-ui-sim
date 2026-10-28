//! Admin specialization events reach the selected Retail micro-menu consumer.
#![cfg(feature = "client-retail")]

use wow_ui_sim::loader::{
    StartupAddonLoadKind, discover_blizzard_startup_addons_for_screen, load_startup_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn load_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("fresh environment");
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir();
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    wow_ui_sim::xml::register_intrinsic_templates();
    if let Some(toc) = wow_ui_sim::client_profile::blizzard_ui_framexml_toc() {
        load_startup_addon(&env.loader_env(), &toc, StartupAddonLoadKind::Full, None)
            .expect("load FrameXML");
        env.fire_event_with_args("ADDON_LOADED", &[env.lua_string("FrameXML")])
            .expect("dispatch FrameXML load");
    }
    for addon in discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game) {
        load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None)
            .expect("load startup addon");
        if addon.kind == StartupAddonLoadKind::Full {
            env.fire_event_with_args("ADDON_LOADED", &[env.lua_string(&addon.name)])
                .expect("dispatch addon load");
        }
        if addon.name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
    }
    env.sync_string_metatable_to_global_string();
    env.sync_addon_names_to_lua();
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events(&env);
    wow_ui_sim::startup::process_pending_timers(&env);
    wow_ui_sim::startup::fire_one_on_update_tick(&env);
    env
}

#[test]
fn admin_set_spec_dispatches_to_observers_and_micro_menu() {
    let env = load_game_ui();
    env.exec(
        r#"
        assert(PlayerSpellsMicroButton:IsEventRegistered("PLAYER_SPECIALIZATION_CHANGED"))
        local source = debug.getinfo(PlayerSpellsMicroButton.GetAnyPvpTalentAlert, "S").source
        assert(string.find(source, "MainMenuBarMicroButtons.lua", 1, true), source)
        specEvents = {}
        local observer = CreateFrame("Frame")
        observer:RegisterEvent("PLAYER_SPECIALIZATION_CHANGED")
        observer:SetScript("OnEvent", function(_, event, ...)
            table.insert(specEvents, {event, select('#', ...), ..., GetSpecialization()})
        end)
        PlayerSpellsMicroButton.dirty = nil
        initialSpec = GetSpecialization()
        firstSpec = initialSpec == 1 and 2 or 1
        secondSpec = initialSpec == 3 and 2 or 3
        A_Admin.SetSpec(firstSpec)
        assert(#specEvents == 1 and specEvents[1][1] == "PLAYER_SPECIALIZATION_CHANGED")
        assert(specEvents[1][2] == 1 and specEvents[1][3] == "player" and specEvents[1][4] == firstSpec)
        assert(PlayerSpellsMicroButton.dirty, "original micro-menu consumer did not mark itself dirty")
        PlayerSpellsMicroButton.dirty = nil
        A_Admin.SetSpec(secondSpec)
        assert(#specEvents == 2 and specEvents[2][1] == "PLAYER_SPECIALIZATION_CHANGED")
        assert(specEvents[2][2] == 1 and specEvents[2][3] == "player" and specEvents[2][4] == secondSpec)
        assert(PlayerSpellsMicroButton.dirty, "second change did not mark micro-menu dirty")
        PlayerSpellsMicroButton.dirty = nil
        A_Admin.SetSpec(secondSpec)
        assert(#specEvents == 2, "same specialization must not fire an event")
        assert(not PlayerSpellsMicroButton.dirty, "same specialization must not refresh micro-menu")
        "#,
    )
    .expect("second spec event and same-spec no-op");
}

