//! R88n.6: ordinary descriptor and argument-result contracts.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn tooltip_comparison_validates_in_argument_order_and_returns_zero_on_misses() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(
        r##"
        local api = assert(C_TooltipComparison)
        local tip = CreateFrame("GameTooltip", nil, UIParent)
        local frame = CreateFrame("Button", nil, UIParent)
        local texture = frame:CreateTexture()
        local missed = {hyperlink = "item:2147483647"}
        local usableGuid = {guid = "Item-0-999-211993", hyperlink = "item:211993"}
        local function check(index, name, reason, callback, ...)
            local ok, err = pcall(callback, ...)
            assert(not ok and type(err) == "string", "expected argument error")
            assert(err:find("bad argument #" .. index, 1, true), err)
            assert(err:find(name, 1, true) and err:find(reason, 1, true), err)
        end
        local infoUsage = "Usage: local info = C_TooltipComparison.GetItemComparisonInfo(comparisonItem)"
        local deltaUsage = "Usage: local lines = C_TooltipComparison.GetItemComparisonDelta(comparisonItem, equippedItem [, pairedItem, addPairedStats])"
        local cmpUsage = "Usage: C_TooltipComparison.CompareItem(comparisonItem, tooltip [, anchorFrame])"

        check(1, "CompareItem", cmpUsage, api.CompareItem)
        check(1, "CompareItem", cmpUsage, api.CompareItem, {}, {})
        check(2, "CompareItem", cmpUsage, api.CompareItem, missed, nil)
        check(2, "CompareItem", "Wrong object type for function - " .. cmpUsage, api.CompareItem, missed, frame)
        check(2, "CompareItem", "Attempt to find 'this' in non-table object (used '.' instead of ':' ?) - " .. cmpUsage, api.CompareItem, missed, 12)
        check(3, "CompareItem", "Wrong object type for function - " .. cmpUsage, api.CompareItem, missed, tip, texture)
        assert(select("#", api.CompareItem(missed, tip)) == 0)
        assert(select("#", api.CompareItem(missed, tip, nil, "extra")) == 0)
        assert(select("#", api.CompareItem(usableGuid, tip, frame, "extra")) == 0)

        check(1, "GetItemComparisonInfo", infoUsage, api.GetItemComparisonInfo, nil)
        for _, invalid in ipairs({false, 211993, "item:211993", {}, {guid = "Item-0-1-0"}, {hyperlink = "spell:1"}}) do
            check(1, "GetItemComparisonInfo", infoUsage, api.GetItemComparisonInfo, invalid)
        end
        assert(select("#", api.GetItemComparisonInfo(usableGuid, "extra")) == 0, "convert GUID, then miss")
        assert(select("#", api.GetItemComparisonInfo(missed)) == 0)
        assert(select("#", api.GetItemComparisonInfo({guid = "garbage", hyperlink = "item:2147483647", overrideItemLevel = "bad"})) == 0)

        check(1,"GetItemComparisonDelta", deltaUsage, api.GetItemComparisonDelta, nil, nil, {})
        check(2,"GetItemComparisonDelta", deltaUsage, api.GetItemComparisonDelta, missed, nil, {})
        check(3,"GetItemComparisonDelta", deltaUsage, api.GetItemComparisonDelta, missed, missed, 0)
        assert(select("#", api.GetItemComparisonDelta(missed, missed)) == 0)
        assert(select("#", api.GetItemComparisonDelta(missed, missed, nil, {}, "ignored extra")) == 0)

        assert(Enum.TooltipComparisonMethod.Single == 0)
        assert(Enum.TooltipComparisonMethod.WithBothHands == 1)
        assert(Enum.TooltipComparisonMethod.WithBagMainHandItem == 2)
        assert(Enum.TooltipComparisonMethod.WithBagOffHandItem == 3)
        "##,
    )
    .expect("ordinary R88n.6 argument and arity contract");
}
