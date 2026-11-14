//! Tests of the explicitly inferred simulator comparison projection, not native formatting.
#![cfg(feature = "client-retail")]

use wow_ui_sim::c_api::item_tooltip::{ItemTooltipInput, ItemTooltipStat};
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn supplied_tooltip_stats_follow_explicit_operands_and_optional_pair() {
    let env = WowLuaEnv::new().expect("Lua environment");
    for (id, level, amount) in [(211993, 571, 41), (221125, 437, 13), (211993, 500, 19)] {
        env.state().borrow_mut().item_tooltip_inputs.insert(
            (id, level),
            ItemTooltipInput {
                stats: vec![ItemTooltipStat {
                    label: "Stamina".to_owned(),
                    amount,
                }],
                ..Default::default()
            },
        );
    }
    env.exec(
        r##"
        A_Admin.EquipItem(1, 221125)
        local api = C_TooltipComparison
        local a = {hyperlink = "item:211993"}
        local b = {hyperlink = "item:221125"}
        local info = assert(api.GetItemComparisonInfo(a, "extra"))
        assert(select("#", api.GetItemComparisonInfo(a)) == 1)
        assert(info.method == Enum.TooltipComparisonMethod.Single)
        assert(info.item.hyperlink:find("item:221125", 1, true))
        assert(type(info.additionalItems) == "table" and #info.additionalItems == 0)
        local noLiveGuid = {guid = "Item-0-999-211993", hyperlink = a.hyperlink}
        assert(select("#", api.GetItemComparisonInfo(noLiveGuid)) == 0, "usable GUID takes precedence even if absent")
        local fallback = {guid = "invalid", hyperlink = a.hyperlink}
        assert(api.GetItemComparisonInfo(fallback).item.hyperlink == info.item.hyperlink)

        local function line(...)
            assert(select("#", api.GetItemComparisonDelta(...)) == 1)
            local rows = api.GetItemComparisonDelta(...)
            assert(#rows == 1 and type(rows[1]) == "string")
            return rows[1]
        end
        local forward, reverse = line(a,b), line(b,a)
        assert(forward ~= reverse, "Delta uses explicit equipped operand, not inventory")
        assert(line(a,b,nil,0) == forward and line(a,b,nil,"") == forward)
        local subtract = line(a,b,b,false)
        assert(line(a,b,b,nil) == subtract)
        local add = line(a,b,b,true)
        assert(add ~= subtract)
        for _, truthy in ipairs({0,"",{}}) do
            assert(line(a,b,b,truthy) == add)
        end
        assert(line({hyperlink=a.hyperlink, overrideItemLevel="500.9"}, b)
           == line({hyperlink=a.hyperlink, overrideItemLevel=500}, b))
        assert(line({hyperlink=a.hyperlink, overrideItemLevel=500}, b) ~= forward)
        for _, bad in ipairs({math.huge, 2147483648, {}, "invalid"}) do
            assert(line({hyperlink=a.hyperlink, overrideItemLevel=bad}, b) == forward)
        end
        assert(select("#", api.GetItemComparisonDelta(a,a)) == 1)
        assert(type(api.GetItemComparisonDelta(a,a)) == "table" and #api.GetItemComparisonDelta(a,a) == 0)
        "##,
    )
    .expect("simulator inference uses existing equipment and host-provided stats only");
}
