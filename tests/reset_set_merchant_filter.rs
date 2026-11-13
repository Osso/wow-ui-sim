//! Retail consumer coverage through the selected game-startup addon set.
//! The callback later hits missing `CanAffordMerchantItem` at MerchantFrame.lua:290.
//! Assert that known gap; do not stub it to hide the unrelated failure.
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
        assert(CanAffordMerchantItem == nil, "known later consumer gap changed")
        SetMerchantFilter(3)
        local ok, err = pcall(MerchantFrame_OnShow, MerchantFrame)
        local errorText = tostring(err)
        assert(not ok, "merchant consumer should reach the known CanAffordMerchantItem gap")
        assert(errorText:find("MerchantFrame.lua:290", 1, true), errorText)
        assert(errorText:find("attempt to call a nil value", 1, true), errorText)
        assert(GetMerchantFilter() == 2, "merchant filter reset before the later consumer gap")
        assert(select("#", ResetSetMerchantFilter()) == 0, "reset must return no values")
        assert(GetMerchantNumItems() == 2, "simulator merchant count remains unfiltered")
        "##,
    )
    .expect("original Retail MerchantFrame_OnShow should reach the reset");
}
