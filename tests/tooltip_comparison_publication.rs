use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn tooltip_comparison_publication_matches_documented_profiles() {
    let env = WowLuaEnv::new().expect("Lua environment");
    let (namespace, compare, info, delta): (bool, bool, bool, bool) = env
        .eval(
            r#"
            local api = _G.C_TooltipComparison
            return type(api) == "table",
                   api ~= nil and type(api.CompareItem) == "function",
                   api ~= nil and type(api.GetItemComparisonInfo) == "function",
                   api ~= nil and type(api.GetItemComparisonDelta) == "function"
            "#,
        )
        .expect("namespace publication");
    let expected = matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    );
    assert_eq!(
        (namespace, compare, info, delta),
        (expected, expected, expected, expected)
    );
}
