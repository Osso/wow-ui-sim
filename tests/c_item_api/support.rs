use wow_ui_sim::lua_api::WowLuaEnv;

pub(crate) fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

/// Retail and Forever own the `ItemButton` intrinsic in the original
/// Blizzard_ItemButton ItemButtonTemplate.xml; eb2ee8488b retired the
/// CreateFrame surface fallback there.
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn item_button_env() -> WowLuaEnv {
    use crate::common::blizzard_addon_harness::{
        blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
    };
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_ItemButton"], &[]);
    env
}

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
pub(crate) fn item_button_env() -> WowLuaEnv {
    env()
}
