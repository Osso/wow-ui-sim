use wow_ui_sim::lua_api::WowLuaEnv;

fn assert_skill_header_publication(env: &WowLuaEnv) {
    let published: (bool, bool) = env
        .eval(
            "return type(CollapseSkillHeader) == 'function', type(ExpandSkillHeader) == 'function'",
        )
        .unwrap();
    if cfg!(feature = "retail-12-0-0") {
        assert_eq!(published, (false, false));
        let raw_absent: bool = env
            .eval("return rawget(_G, 'CollapseSkillHeader') == nil and rawget(_G, 'ExpandSkillHeader') == nil")
            .unwrap();
        assert!(raw_absent);
    } else {
        assert_eq!(published, (true, true));
        env.exec("CollapseSkillHeader(1); ExpandSkillHeader(1)")
            .unwrap();
    }
}

#[test]
fn patch_4_0_1_skill_headers_factory_surface() {
    let env = WowLuaEnv::new().unwrap();
    assert_skill_header_publication(&env);
}

prefork_full_ui_case! {
fn patch_4_0_1_skill_headers_cached_surface(env: &WowLuaEnv) {
    assert_skill_header_publication(env);
}
}
