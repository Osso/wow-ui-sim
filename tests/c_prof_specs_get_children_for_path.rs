#![cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const USAGE: &str = "Usage: local childIDs = C_ProfSpecs.GetChildrenForPath(pathID)";

fn env_with_path_rows() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create profession path environment");
    let mut sim = env.state().borrow_mut();
    sim.profession_spec_inputs
        .path_group_ids
        .extend([(1001, 10), (2001, 20), (4001, 30)]);
    sim.profession_spec_inputs
        .group_node_ids
        .extend([(10, vec![2001, 2002]), (20, vec![3001])]);
    drop(sim);
    env
}

fn invalid_argument_script() -> String {
    format!(
        r#"
        local usage = {USAGE:?}
        local invalid = {{
            function() C_ProfSpecs.GetChildrenForPath() end,
            function() C_ProfSpecs.GetChildrenForPath(nil) end,
            function() C_ProfSpecs.GetChildrenForPath("nope") end,
            function() C_ProfSpecs.GetChildrenForPath({{}}) end,
            function() C_ProfSpecs.GetChildrenForPath(0 / 0) end,
            function() C_ProfSpecs.GetChildrenForPath(math.huge) end,
            function() C_ProfSpecs.GetChildrenForPath(-math.huge) end,
            function() C_ProfSpecs.GetChildrenForPath(2147483647.5) end,
            function() C_ProfSpecs.GetChildrenForPath(-2147483648.5) end,
            function() C_ProfSpecs.GetChildrenForPath(2147483648) end,
            function() C_ProfSpecs.GetChildrenForPath(-2147483649) end,
        }}
        for _, call in ipairs(invalid) do
            local ok, err = pcall(call)
            if ok or not string.find(err, usage, 1, true) then return false end
        end
        local children = C_ProfSpecs.GetChildrenForPath("1001.9")
        return #children == 2 and children[1] == 2001 and children[2] == 2002
    "#
    )
}

#[test]
fn get_children_returns_one_ordered_table_for_numeric_paths() {
    let env = env_with_path_rows();
    assert!(env.eval::<bool>(r#"
        local count, children = select("#", C_ProfSpecs.GetChildrenForPath(1001.9)), C_ProfSpecs.GetChildrenForPath("1001.9")
        if count ~= 1 or type(children) ~= "table" then return false end
        if #children ~= 2 or children[1] ~= 2001 or children[2] ~= 2002 then return false end
        local nested = C_ProfSpecs.GetChildrenForPath(2001)
        return #nested == 1 and nested[1] == 3001
    "#).expect("GetChildrenForPath returns ordered child IDs"));
}

#[test]
fn get_children_returns_one_empty_table_for_absent_paths_and_leaves() {
    let env = env_with_path_rows();
    assert!(env.eval::<bool>(r#"
        local function empty_result(pathID)
            local count, children = select("#", C_ProfSpecs.GetChildrenForPath(pathID)), C_ProfSpecs.GetChildrenForPath(pathID)
            return count == 1 and type(children) == "table" and next(children) == nil
        end
        return empty_result(9999) and empty_result(4001) and empty_result(3001)
    "#).expect("unknown and leaf paths return empty tables"));
}

#[test]
fn get_children_uses_verified_int32_conversion_and_usage_errors() {
    let env = env_with_path_rows();
    assert!(
        env.eval::<bool>(&invalid_argument_script())
            .expect("verified argument contract")
    );
}
