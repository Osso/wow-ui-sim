//! Regional name policy and the unmodified Forever naming consumer.

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_name_policy_defaults_to_disabled() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec("assert(RegionalUniqueNamesEnabled() == false); assert(select('#', RegionalUniqueNamesEnabled()) == 1)")
        .unwrap();
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_name_policy_changes_vendor_first_name_and_is_isolated() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    let other = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.state().borrow_mut().player.name = "Ada-Lovelace".to_owned();
    // Explicit separator fixture: this test covers name policy, not constants publication.
    env.exec("Constants.CharacterNameSeparatorConsts = { CHARACTERNAME_SURNAME_SEPARATOR = '-' }")
        .unwrap();
    let path = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .unwrap()
        .join("Blizzard_FrameXMLUtil/Camelot/NameUtil.lua");
    env.exec(&std::fs::read_to_string(path).unwrap()).unwrap();
    env.exec("assert(NameUtil.GetUnitFirstName('player') == 'Ada-Lovelace')")
        .unwrap();
    env.state()
        .borrow_mut()
        .player
        .regional_unique_names_enabled = true;
    env.exec("assert(RegionalUniqueNamesEnabled() == true); assert(NameUtil.GetUnitFirstName('player') == 'Ada')")
        .unwrap();
    other
        .exec("assert(RegionalUniqueNamesEnabled() == false)")
        .unwrap();
    env.state()
        .borrow_mut()
        .player
        .regional_unique_names_enabled = false;
    env.exec("assert(NameUtil.GetUnitFirstName('player') == 'Ada-Lovelace')")
        .unwrap();
}

#[test]
#[cfg(not(feature = "client-wowforever"))]
fn forever_name_policy_is_not_published_on_other_profiles() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec("assert(RegionalUniqueNamesEnabled == nil)")
        .unwrap();
}
