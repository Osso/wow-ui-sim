//! Observable behavior of the PTR-native table extensions; security is not covered.

use crate::lua_api::WowLuaEnv;

fn assert_lua(script: &str) {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.eval::<()>(script)
        .expect("table extension assertions should pass");
}

#[test]
fn indexof_returns_first_contiguous_match() {
    assert_lua(
        r#"
        local values = {"a", "b", "a"}
        assert(table.indexof(values, "a") == 1)
        assert(table.indexof(values, "b") == 2)
        assert(table.indexof(values, "missing") == nil)
        assert(table.indexof({}, "a") == nil)
        assert(table.indexof({[1] = "a", [3] = "b"}, "b") == nil)
        assert(#values == 3 and values[2] == "b")
        "#,
    );
}

#[test]
fn contains_searches_array_and_hash_values() {
    assert_lua(
        r#"
        local object = {}
        local values = {"a", named = false, object = object}
        assert(table.contains(values, "a"))
        assert(table.contains(values, false))
        assert(table.contains(values, object))
        assert(not table.contains(values, "named"))
        assert(not table.contains(values, {}))
        assert(not table.contains(values, nil))
        assert(not table.contains({}, "a"))
        "#,
    );
}

#[test]
fn count_returns_exactly_one_total() {
    assert_lua(
        r#"
        local values = {"first", "second", [4] = false, rank = "leader"}
        assert(select('#', table.count(values)) == 1)
        assert(table.count(values) == 4)
        assert(select('#', table.count({})) == 1)
        assert(table.count({}) == 0)
        values.rank = nil
        assert(table.count(values) == 3)
        "#,
    );
}

#[test]
fn isempty_tracks_entry_insertion_and_removal() {
    assert_lua(
        r#"
        local values = {}
        assert(table.isempty(values))
        values[4] = false
        assert(not table.isempty(values))
        values[4] = nil
        values.name = "value"
        assert(not table.isempty(values))
        values.name = nil
        assert(table.isempty(values))
        "#,
    );
}

#[test]
fn removeunordered_swaps_last_value_and_returns_removed_value() {
    assert_lua(
        r#"
        local values = {"a", "b", "c", label = "kept"}
        assert(table.removeunordered(values, 2) == "b")
        assert(#values == 2 and values[1] == "a" and values[2] == "c")
        assert(values[3] == nil and values.label == "kept")
        assert(table.removeunordered(values) == "c")
        assert(#values == 1 and values[1] == "a")
        assert(table.removeunordered(values, 0) == nil)
        assert(table.removeunordered(values, 2) == nil)
        assert(#values == 1 and values[1] == "a")
        assert(table.removeunordered({}) == nil)
        "#,
    );
}

#[test]
fn removevalue_removes_all_array_matches_and_preserves_order() {
    assert_lua(
        r#"
        local values = {"a", "b", "a", "c", "a", label = "a"}
        assert(table.removevalue(values, "a") == 3)
        assert(#values == 2 and values[1] == "b" and values[2] == "c")
        assert(values[3] == nil and values[4] == nil and values[5] == nil)
        assert(values.label == "a")
        assert(table.removevalue(values, "missing") == 0)
        assert(#values == 2)
        assert(table.removevalue({}, "a") == 0)
        "#,
    );
}

#[test]
fn keys_returns_dense_array_of_all_keys_without_order_assumptions() {
    assert_lua(
        r#"
        local object = {}
        local source = {[3] = "number", name = false, [object] = "object"}
        local keys = table.keys(source)
        assert(keys ~= source and #keys == 3)
        local expected = {[3] = true, name = true, [object] = true}
        for index = 1, 3 do
            local key = keys[index]
            assert(expected[key] == true)
            expected[key] = nil
        end
        assert(next(expected) == nil and keys[4] == nil)
        keys[1] = "changed"
        assert(source[3] == "number" and source.name == false)
        assert(source[object] == "object")
        assert(next(table.keys({})) == nil)
        "#,
    );
}

#[test]
fn values_returns_dense_array_preserving_duplicate_values() {
    assert_lua(
        r#"
        local object = {}
        local source = {[3] = "same", first = "same", flag = false, obj = object}
        local values = table.values(source)
        assert(values ~= source and #values == 4)
        local remaining = {["same"] = 2, [false] = 1, [object] = 1}
        for index = 1, 4 do
            local value = values[index]
            assert(remaining[value] and remaining[value] > 0)
            remaining[value] = remaining[value] - 1
        end
        assert(remaining.same == 0 and remaining[false] == 0)
        assert(remaining[object] == 0 and values[5] == nil)
        values[1] = "changed"
        assert(source[3] == "same" and source.first == "same")
        assert(source.flag == false and source.obj == object)
        assert(next(table.values({})) == nil)
        "#,
    );
}
