//! Freeze only fresh, isolated graphs; never shared VM tables or metatables.
use crate::lua_api::WowLuaEnv;

#[test]
fn table_freeze_profile_returns_and_reads() {
    let env = WowLuaEnv::new().unwrap();
    let returns = if cfg!(feature = "retail-12-1-5") {
        1
    } else {
        0
    };
    env.exec(&format!(
        r#"
        assert(type(rawget(table, "freeze")) == "function", "freeze missing")
        assert(type(rawget(table, "isfrozen")) == "function", "isfrozen missing")
        local t = {{42, label = "unchanged"}}
        assert(table.isfrozen(t) == false)
        assert(select('#', table.isfrozen(t)) == 1)
        assert(select('#', table.freeze(t)) == {returns})
        assert(table.isfrozen(t) == true)
        local result = table.freeze(t)
        if {returns} == 1 then assert(rawequal(result, t)) else assert(result == nil) end
        assert(t[1] == 42 and t.label == "unchanged" and #t == 1)
        assert(table.concat(t) == "42")
        local count = 0
        for _ in pairs(t) do count = count + 1 end
        assert(count == 2)
        "#
    ))
    .unwrap();
}

#[test]
fn table_freeze_rejects_writes_and_mutators() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local writers = {
            function(t) t[1] = 9 end,
            function(t) t.added = 9 end,
            function(t) rawset(t, 1, 9) end,
            function(t) table.insert(t, 9) end,
            function(t) table.remove(t, 1) end,
            function(t) table.sort(t) end,
            function(t) wipe(t) end,
            function(t) tinsert(t, 9) end,
            function(t) tremove(t, 1) end,
        }
        if table.removeunordered then
            writers[#writers + 1] = function(t) table.removeunordered(t, 1) end
            writers[#writers + 1] = function(t) table.removevalue(t, 3) end
        end
        for index, write in ipairs(writers) do
            local t = {3, 1, 2}
            table.freeze(t)
            local ok, err = pcall(write, t)
            assert(not ok, "mutation succeeded: " .. index)
            assert(tostring(err):find("frozen"), tostring(err))
            assert(#t == 3 and t[1] == 3 and t[2] == 1 and t[3] == 2)
        end
        local mutable = {3, 1, 2}
        table.sort(mutable)
        table.insert(mutable, 4)
        assert(table.remove(mutable) == 4)
        assert(tinsert(mutable, 4) == nil and tremove(mutable) == 4)
        assert(wipe(mutable) == mutable and next(mutable) == nil)
        "#,
    )
    .unwrap();
}

#[test]
fn table_freeze_traverses_isolated_cycles_keys_and_metatables() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local child, key, inherited = {value = 7}, {}, {visible = 5}
        local private_meta = {__index = inherited}
        local root = setmetatable({child = child, [key] = child}, private_meta)
        root.self = root
        child.parent = root
        table.freeze(root)
        for _, t in ipairs({root, child, key, private_meta, inherited}) do
            assert(table.isfrozen(t) == true)
            assert(not pcall(rawset, t, "added", true))
        end
        assert(root.self == root and child.parent == root)
        assert(root[key] == child and root.child.value == 7 and root.visible == 5)
        table.freeze(root)
        collectgarbage("collect")
        assert(root.child.value == 7 and root.visible == 5)
        local unrelated = setmetatable({}, {})
        assert(not table.isfrozen(unrelated) and not table.isfrozen(getmetatable(unrelated)))
        unrelated.value = 8
        assert(unrelated.value == 8)
        "#,
    )
    .unwrap();
}

#[test]
fn table_freeze_rejects_non_table_arguments() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(rawget(table, "freeze")) == "function")
        assert(type(rawget(table, "isfrozen")) == "function")
        for _, call in ipairs({table.freeze, table.isfrozen}) do
            for _, value in ipairs({false, 1, "text", function() end, newproxy(true)}) do
                local ok, err = pcall(call, value)
                assert(not ok and tostring(err):find("table expected"), tostring(err))
            end
            assert(not pcall(call))
            assert(not pcall(call, nil))
        end
        "#,
    )
    .unwrap();
}
