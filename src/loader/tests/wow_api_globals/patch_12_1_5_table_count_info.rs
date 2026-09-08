//! Bounded table counting and required creation-hint contracts.

use crate::lua_api::WowLuaEnv;

#[test]
fn table_create_requires_array_hint() {
    let env = WowLuaEnv::new().unwrap();
    let succeeds: bool = env.eval("return pcall(table.create)").unwrap();
    assert!(!succeeds);
}

#[test]
fn table_create_valid_hints_return_empty_tables() {
    let env = WowLuaEnv::new().unwrap();
    env.eval::<()>(
        r#"
        for _, hints in ipairs({{4}, {4, 2}}) do
            local value = table.create(unpack(hints))
            assert(type(value) == 'table')
            assert(next(value) == nil)
        end
    "#,
    )
    .unwrap();
}

#[cfg(feature = "retail-12-1-5")]
#[test]
fn table_getcountinfo_counts_mixed_and_empty_tables() {
    let env = WowLuaEnv::new().unwrap();
    env.eval::<()>(
        r#"
        local values = {[1]='a', [3]=false, [-1]='negative', [1.5]='fractional', named='x'}
        assert(type(table.getcountinfo) == 'function')
        assert(select('#', table.getcountinfo(values)) == 3)
        local total, positive, maximum = table.getcountinfo(values)
        assert(total == 5 and positive == 2 and maximum == 3)
        assert(select('#', table.getcountinfo({})) == 3)
        total, positive, maximum = table.getcountinfo({})
        assert(total == 0 and positive == 0 and maximum == 0)
    "#,
    )
    .unwrap();
}

#[cfg(feature = "retail-12-1-5")]
#[test]
fn table_count_retains_single_return() {
    let env = WowLuaEnv::new().unwrap();
    env.eval::<()>(
        r#"
        local values = {[1]='a', [3]=false, [-1]='negative', [1.5]='fractional', named='x'}
        assert(select('#', table.count(values)) == 1)
        assert(table.count(values) == 5)
        assert(table.count({}) == 0)
    "#,
    )
    .unwrap();
}

#[cfg(not(feature = "retail-12-1-5"))]
#[test]
fn table_getcountinfo_is_absent_before_ptr() {
    let env = WowLuaEnv::new().unwrap();
    let kind: String = env.eval("return type(table.getcountinfo)").unwrap();
    assert_eq!(kind, "nil");
}
