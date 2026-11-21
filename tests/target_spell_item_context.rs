#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn load_stock_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .expect("profile Blizzard UI cache should exist");
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    crate::common::load_profile_framexml(&env);
    for (name, toc) in wow_ui_sim::loader::discover_blizzard_addons(&ui) {
        wow_ui_sim::loader::load_addon(&env.loader_env(), &toc)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
    }
    env
}

#[test]
fn item_context_uses_supplied_pending_target_spell_metadata() {
    let env = load_stock_ui();
    env.exec(
        r#"
        A_Admin.SetPendingTargetSpell(1719, true, false, false)
        assert(ItemButtonUtil.GetItemContext() == ItemButtonUtil.ItemContextEnum.Enchanting)

        A_Admin.SetPendingTargetSpell(20154, false, true, false)
        assert(ItemButtonUtil.GetItemContext() == ItemButtonUtil.ItemContextEnum.JumpUpgradeTrack)

        A_Admin.SetPendingTargetSpell(324739, false, false, true)
        assert(ItemButtonUtil.GetItemContext() == ItemButtonUtil.ItemContextEnum.ReplaceBonusTree)
        "#,
    )
    .expect("stock item-context consumer should read supplied target-spell metadata");
}
