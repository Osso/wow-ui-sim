//! Retail consumer coverage through the selected game-startup addon set.
#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, new_blizzard_addon_env};
use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_startup_addons_for_screen, load_startup_addon,
    register_load_with_addons,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn retail_game_startup_env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    register_load_with_addons(&env.loader_env(), &discover_all_blizzard_addons(&ui));

    let startup = discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game);
    for addon in startup {
        let already_loaded = env
            .state()
            .borrow()
            .addons
            .iter()
            .any(|loaded| loaded.folder_name == addon.name && loaded.loaded);
        if !already_loaded {
            load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None)
                .unwrap_or_else(|error| panic!("load startup {}: {error}", addon.name));
        }
        if addon.name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
    }
    env.sync_string_metatable_to_global_string();
    env
}

#[test]
fn merchant_frame_on_show_resets_the_filter_without_return_values() {
    let env = retail_game_startup_env();
    {
        let mut state = env.state().borrow_mut();
        state.merchant_items = vec![6948, 2840];
        state.merchant_frame_open = true;
    }

    env.exec(
        r##"
        assert(type(MerchantFrame_OnShow) == "function", "Retail merchant consumer missing")
        SetMerchantFilter(3)
        MerchantFrame_OnShow(MerchantFrame)
        assert(GetMerchantFilter() == 2, "merchant filter was not reset")
        assert(select("#", ResetSetMerchantFilter()) == 0, "reset must return no values")
        assert(GetMerchantNumItems() == 2, "simulator merchant count remains unfiltered")
        "##,
    )
    .expect("original Retail MerchantFrame_OnShow should run");
}
