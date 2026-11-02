//! Regional name policy and the unmodified Forever naming consumer.

#[cfg(feature = "client-wowforever")]
fn load_forever_name_util(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    // Explicit separator fixture: these tests do not establish native separator publication.
    env.exec("Constants.CharacterNameSeparatorConsts = { CHARACTERNAME_SURNAME_SEPARATOR = '-' }")
        .unwrap();
    let addons = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    for source in [
        "Blizzard_SharedXMLBase/Compat.lua",
        "Blizzard_FrameXMLUtil/Camelot/NameUtil.lua",
    ] {
        env.exec(&std::fs::read_to_string(addons.join(source)).unwrap())
            .unwrap();
    }
}

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
    load_forever_name_util(&env);
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
#[cfg(feature = "client-wowforever")]
fn forever_supplied_surname_reaches_original_name_consumers() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    let other = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.player.name = "Ada".into();
        state.player.surname = Some("Lovelace".into());
    }
    load_forever_name_util(&env);
    env.exec(
        r#"
        for _, query in ipairs({UnitName, UnitNameUnmodified, UnitFullName}) do
            local first, surname = query('player')
            assert(first == 'Ada' and surname == 'Lovelace')
            assert(select('#', query('player')) == 2)
            local absent, second = query('target')
            assert(absent == nil and second == nil)
            assert(select('#', query('target')) == 2)
            local upperFirst, upperSurname = query('PLAYER')
            assert(upperFirst == first and upperSurname == surname)
        end
        assert(NameUtil.FormatUnitNameForDisplay('player') == 'Ada-Lovelace')
        assert(NameUtil.GetUnmodifiedUnitFullName('player') == 'Ada-Lovelace')
        assert(NameUtil.GetUnitFirstName('player') == 'Ada')
        assert(NameUtil.IsPlayerMe('Ada', 'Lovelace'))
        assert(not NameUtil.IsPlayerMe('Ada', 'Other'))
        assert(not NameUtil.IsPlayerMe('Ada', nil))
        "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .player
        .regional_unique_names_enabled = true;
    env.exec(
        "local first, surname = UnitName('player'); \
         assert(first == 'Ada' and surname == 'Lovelace'); \
         assert(select('#', UnitName('player')) == 2)",
    )
    .unwrap();
    other
        .exec("local _, surname = UnitName('player'); assert(surname == nil); assert(select('#', UnitName('player')) == 2)")
        .unwrap();

    env.state().borrow_mut().player.surname = None;
    env.exec(
        r#"
        local first, surname = UnitName('player')
        assert(first == 'Ada' and surname == nil)
        assert(select('#', UnitName('player')) == 2)
        assert(select('#', UnitNameUnmodified('player')) == 2)
        assert(NameUtil.FormatUnitNameForDisplay('player') == 'Ada')
        assert(NameUtil.GetUnmodifiedUnitFullName('player') == 'Ada')
        assert(NameUtil.IsPlayerMe('Ada', nil))
        "#,
    )
    .unwrap();
}

#[test]
#[cfg(not(any(feature = "client-wowforever", feature = "client-ptr")))]
fn forever_name_policy_is_not_published_on_other_profiles() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec("assert(RegionalUniqueNamesEnabled == nil)")
        .unwrap();
}
