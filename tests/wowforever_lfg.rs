#![cfg(feature = "client-wowforever")]


#[test]
fn forever_lfg_constants_load_full_vendor_constants() {
    use crate::common::blizzard_addon_harness::{
        blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
    };
    // Constants.lua:288 calls GetMicroIconForRole (SharedXMLBase TextureUtil.lua;
    // 096d8a1f8c retired the bootstrap copy).
    let root = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&root);
    load_blizzard_addon_closure_into_env(&env, &root, &["Blizzard_SharedXMLBase"], &[]);
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
