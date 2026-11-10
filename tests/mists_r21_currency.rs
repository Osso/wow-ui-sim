#![cfg(feature = "client-mists")]

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::lua_api::WowLuaEnv;

const WATCH_FOUR: &str = r#"
    R21CurrencyRows = {}
    for i = 1, GetCurrencyListSize() do
        local _, header = GetCurrencyListInfo(i)
        if not header then
            SetCurrencyBackpack(i, 0)
            R21CurrencyRows[#R21CurrencyRows + 1] = i
        end
    end
    assert(#R21CurrencyRows >= 4, 'fixture must exceed UI limit')
    for i = 1, 4 do SetCurrencyBackpack(R21CurrencyRows[i], i) end
"#;

#[test]
fn mists_r21_currency_watch_flags_have_no_native_three_slot_cap() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(WATCH_FOUR).unwrap();
    env.exec(r#"
        for i = 1, 4 do
            local list = {GetCurrencyListInfo(R21CurrencyRows[i])}
            local watched = {GetBackpackCurrencyInfo(tostring(i))}
            assert(select('#', GetBackpackCurrencyInfo(i)) == 4)
            assert(watched[1] == list[1] and watched[2] == list[6])
            assert(watched[3] == list[7] and watched[4] == list[12])
            assert(list[5] == true, 'list publishes changed watch flag')
        end
        for _, index in ipairs({0, -1, 5, 999}) do
            assert(select('#', GetBackpackCurrencyInfo(index)) == 4, 'empty slot has four nils')
            local a, b, c, d = GetBackpackCurrencyInfo(index)
            assert(a == nil and b == nil and c == nil and d == nil)
        end
        SetCurrencyBackpack(R21CurrencyRows[2], 0)
        assert(select(4, GetBackpackCurrencyInfo(2)) == select(12, GetCurrencyListInfo(R21CurrencyRows[3])))
        assert(GetBackpackCurrencyInfo(4) == nil, 'removal rebuilds watched list')
        SetCurrencyBackpack(R21CurrencyRows[2], -2)
        assert(GetBackpackCurrencyInfo(4) ~= nil, 'nonzero numeric enable')
    "#).unwrap();
}

#[test]
fn mists_r21_currency_original_backpack_ui_observes_watch_changes() {
    with_blizzard_addon_startup_shape(
        &["Blizzard_UIPanels_Game", "Blizzard_TokenUI"],
        &[],
        |env, _| {
            env.exec(
                r#"
                local source = debug.getinfo(BackpackTokenFrame_Update, 'S').source:gsub('\\', '/')
                assert(source:find('Blizzard_TokenUI/Cata/Blizzard_TokenUI.lua', 1, true), source)
            "#,
            )
            .unwrap();
            env.exec(WATCH_FOUR).unwrap();
            env.exec(r#"
                BackpackTokenFrame_Update()
                assert(MAX_WATCHED_TOKENS == 3 and GetBackpackCurrencyInfo(4) ~= nil)
                for i = 1, 3 do
                    local button = _G['BackpackTokenFrameToken' .. i]
                    local name, count, icon, id = GetBackpackCurrencyInfo(i)
                    assert(name and button:IsShown() and button.itemID == id)
                    assert(button.icon:GetTexture() == icon)
                    assert(button.count:GetText() == (count <= 99999 and tostring(count) or '*'))
                end
                local oldFirst = BackpackTokenFrameToken1.itemID
                SetCurrencyBackpack(R21CurrencyRows[1], 0)
                BackpackTokenFrame_Update()
                assert(BackpackTokenFrameToken1.itemID ~= oldFirst, 'UI updates after unwatch')
                assert(BackpackTokenFrameToken3.itemID == select(12, GetCurrencyListInfo(R21CurrencyRows[4])))
            "#).unwrap();
        },
    );
}
