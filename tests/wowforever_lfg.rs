#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn forever_lfg_constants_load_full_vendor_constants() {
    let env = WowLuaEnv::new().unwrap();
    let root = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let source = std::fs::read_to_string(root.join("Blizzard_FrameXMLBase/Constants.lua")).unwrap();
    env.exec("MAX_WORLD_PVP_QUEUES = nil; LFG_CATEGORY_NAMES = nil")
        .unwrap();
    env.exec(source.trim_start_matches('\u{feff}')).unwrap();
    env.exec(
        r#"
        assert(LE_LFG_CATEGORY_LAIR == 8)
        assert(NUM_LE_LFG_CATEGORYS == 8)
        assert(type(LAIR) == "string")
        assert(LFG_CATEGORY_NAMES[8] == LAIR)
        assert(LFG_CATEGORY_NAMES[LE_LFG_CATEGORY_LFD] == LOOKING_FOR_DUNGEON)
        assert(MAX_WORLD_PVP_QUEUES == 2)
    "#,
    )
    .unwrap();
}
