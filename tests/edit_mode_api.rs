use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::saved_variables::{SavedVariablesManager, WtfConfig};

const EDIT_MODE_LAYOUT_ENV: &str = "WOW_SIM_EDIT_MODE_LAYOUT";

#[path = "edit_mode_api/cache.rs"]
mod cache;
#[path = "edit_mode_api/enums.rs"]
mod enums;

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_edit_mode_refreshes_cooldown_viewer_totems() {
    use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
    use wow_ui_sim::lua_api::globals::real::totems::Totem;
    use wow_ui_sim::screen::ScreenKind;
    use wow_ui_sim::startup::fire_startup_events_for_screen;

    let env = WowLuaEnv::new().expect("create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("Forever UI cache");
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    wow_ui_sim::xml::register_intrinsic_templates();
    for (name, toc_path) in discover_blizzard_addons_for_screen(&ui, ScreenKind::Game) {
        load_addon(&env.loader_env(), &toc_path).unwrap_or_else(|err| panic!("load {name}: {err}"));
    }
    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);

    let short = Totem {
        name: "Short totem".into(),
        start_time: 0.0,
        duration: 3600.0,
        icon: 135127,
        mod_rate: 1.0,
        spell_id: 5394,
    };
    let long = Totem {
        name: "Long totem".into(),
        duration: 7200.0,
        mod_rate: 1.25,
        ..short.clone()
    };
    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.totem_slots = vec![Some(short), None, Some(long)];
    }
    env.exec("CooldownViewer_MarkTotemCacheDirty()").unwrap();
    env.send_key_press("ESCAPE", None).expect("open Game Menu");
    env.exec(
        r#"
        assert(GameMenuFrame:IsShown())
        local editButton
        for _, button in ipairs(GameMenuFrame.buttons) do
            if button:GetText() == HUD_EDIT_MODE_MENU then
                editButton = button
                break
            end
        end
        assert(editButton, "Edit Mode button")
        editButton:Click()
        assert(EditModeManagerFrame:IsEditModeActive())
        "#,
    )
    .expect("enter Edit Mode through the Game Menu");
    let errors = env.state().borrow().lua_errors.clone();
    assert!(errors.is_empty(), "Edit Mode Lua errors: {errors:?}");

    env.exec(
        r#"
        assert(GetNumTotemSlots() == 3)
        local cache = CooldownViewerItemDataMixin:GetCurrentPlayerTotemCache()
        assert(cache[1] == nil and cache[2] == nil)
        local totem = assert(cache[3], "the longer-lived duplicate totem should be cached")
        assert(totem.name == "Long totem" and totem.spellID == 5394)
        assert(totem.duration == 7200 and totem.modRate == 1.25)
        "#,
    )
    .expect("the original Cooldown Viewer cache should consume supplied totem data");
}
