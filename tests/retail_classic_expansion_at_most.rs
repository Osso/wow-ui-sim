#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn retail_classic_expansion_at_most_supports_lib_dual_spec_branch() {
    let env = WowLuaEnv::new().unwrap();
    let result: bool = env
        .eval(
            r#"
            assert(type(ClassicExpansionAtMost) == "function")
            local level = GetClassicExpansionLevel()
            local threshold = LE_EXPANSION_SHADOWLANDS
            local isAtMost = ClassicExpansionAtMost(threshold)
            local description = isAtMost and function(info)
                return info
            end
            assert(type(description) == (isAtMost and "function" or "boolean"))
            return isAtMost == (level <= threshold)
            "#,
        )
        .unwrap();
    assert!(result);
}
