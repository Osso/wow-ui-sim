//! Retail API contract for `C_ProfSpecs.GetChildrenForPath` (R88as).
#![cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]

use common::panel_fixtures::setup_env;
use wow_ui_sim::lua_api::WowLuaEnv;

fn env_with_path_inputs() -> WowLuaEnv {
    let env = setup_env();
    let mut sim = env.state().borrow_mut();
    sim.profession_spec_inputs
        .path_group_ids
        .extend([(1001, 10), (2001, 20), (3001, 30)]);
    sim.profession_spec_inputs.group_node_ids.extend([
        (10, vec![24, 22]),
        (20, vec![34]),
        (30, vec![]),
    ]);
    drop(sim);
    env
}

#[test]
fn returns_one_ordered_table_for_backed_paths_and_empty_tables_for_misses() {
    env_with_path_inputs()
        .exec(
            r#"
            local api = C_ProfSpecs.GetChildrenForPath
            assert(select('#', api(1001)) == 1)
            local first = api(1001)
            assert(type(first) == 'table' and #first == 2)
            assert(first[1] == 24 and first[2] == 22, 'native row order is preserved')
            local second = api('2001')
            assert(#second == 1 and second[1] == 34)
            local fraction = api(' 1001.9 ')
            assert(fraction[1] == 24, 'numeric strings truncate toward zero')
            local leaf = api(3001)
            local absent = api(9999)
            assert(type(leaf) == 'table' and next(leaf) == nil)
            assert(type(absent) == 'table' and next(absent) == nil)
            "#,
        )
        .expect("path children should preserve supplied groups and row order");
}

#[test]
fn rejects_invalid_path_ids_with_the_documented_usage_error() {
    env_with_path_inputs()
        .exec(
            r#"
            local api = C_ProfSpecs.GetChildrenForPath
            local usage = 'Usage: local childIDs = C_ProfSpecs.GetChildrenForPath(pathID)'
            local range = 'outside of expected range -2147483648 to 2147483647 - ' .. usage
            local ok, err = pcall(api)
            assert(not ok and err:find(usage, 1, true), tostring(err))
            ok, err = pcall(api, nil)
            assert(not ok and err:find(usage, 1, true), tostring(err))
            for _, bad in ipairs({'abc', {}, 2147483648, -2147483649, 0/0, math.huge}) do
                ok, err = pcall(api, bad)
                assert(not ok and err:find(usage, 1, true), tostring(err))
                assert(err:find(range, 1, true), tostring(err))
            end
            assert(type(api(-2147483648)) == 'table')
            assert(type(api(2147483647)) == 'table')
            "#,
        )
        .expect("invalid path IDs should raise the native argument error");
}
