#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn retail_12_0_5_addon_can_freeze_its_own_table_without_freezing_children() {
    let env = WowLuaEnv::new().expect("Retail Lua environment");
    env.exec(
        r#"
        assert(type(table.freeze) == 'function', '12.0.5 must publish table.freeze')
        assert(type(table.isfrozen) == 'function', '12.0.5 must publish table.isfrozen')
        local function addon()
            assert(not issecure(), 'fixture must execute as an addon')
            local child = {value = 7}
            local owned = {3, 1, 2, child = child}
            assert(table.isfrozen(owned) == false)
            table.freeze(owned)
            assert(table.isfrozen(owned) == true)
            local writers = {
                function() owned[1] = 9 end,
                function() owned.added = 9 end,
                function() rawset(owned, 1, 9) end,
                function() table.insert(owned, 9) end,
                function() table.remove(owned, 1) end,
                function() table.sort(owned) end,
            }
            for index, write in ipairs(writers) do
                assert(not pcall(write), 'frozen addon mutation succeeded: ' .. index)
                assert(#owned == 3 and owned[1] == 3 and owned[2] == 1 and owned[3] == 2)
                assert(owned.added == nil and owned.child == child)
            end
            assert(table.concat(owned, ',') == '3,1,2')
            assert(table.isfrozen(child) == false)
            child.value = 8
            collectgarbage('collect')
            assert(table.isfrozen(owned) and owned.child.value == 8)
            assert(debug.getstacktaint() == 'Retail1205FreezeProbe')
        end
        debug.setobjecttaint(addon, 'Retail1205FreezeProbe')
        addon()
        assert(issecure(), 'addon return must restore outer secure context')
        "#,
    )
    .expect("addon-owned table is shallow read-only in Retail 12.0.5");
}
