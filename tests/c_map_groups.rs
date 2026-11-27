use wow_ui_sim::lua_api::WowLuaEnv;

const UNKNOWN_GROUP_PROBE: &str = r##"
    local members = { C_Map.GetMapGroupMembersInfo(12345) }
    local first = members[1]
    return select("#", C_Map.GetMapGroupID(2248)), select("#", unpack(members)),
        type(first), first and #first or -1
"##;

// Retail and Forever use the Rust C_Map producer for supplied map groups
// (eefcbd1eaa retired the Lua default); it returns nothing for an unknown
// group. R88f.F.3 leaves native empty-group arity UNKNOWN.
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
const UNKNOWN_GROUP_RESULT: (i32, i32, &str, i32) = (0, 0, "nil", -1);
#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
const UNKNOWN_GROUP_RESULT: (i32, i32, &str, i32) = (0, 1, "table", 0);

#[test]
fn test_get_map_group_members_info_defaults_empty() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let (group_returns, member_returns, members_type, members_count): (i32, i32, String, i32) =
        env.eval(UNKNOWN_GROUP_PROBE).unwrap();
    let actual = (group_returns, member_returns, members_type.as_str(), members_count);
    assert_eq!(actual, UNKNOWN_GROUP_RESULT);
}
