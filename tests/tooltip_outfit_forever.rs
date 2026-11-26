#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn forever_outfit_getter_publishes_empty_names_for_missing_records() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local getter = C_TooltipInfo.GetOutfit
        assert(type(getter) == 'function', 'GetOutfit must be registered')
        assert(select('#', getter(0)) == 0, 'outfit ID zero returns no values')
        local ok, err = pcall(getter)
        assert(not ok and tostring(err):find(
            'Usage: local data = C_TooltipInfo.GetOutfit(outfitID)', 1, true),
            'missing outfit ID reports the documented usage')
        assert(not pcall(getter, 4294967295.9),
            'range validation occurs before truncation')
        for _, id in ipairs({42, 73}) do
            local input = tostring(id) .. '.8'
            local count = select('#', getter(input))
            local data = getter(input)
            assert(count == 1 and type(data) == 'table',
                'a nonzero missing-name outfit returns one TooltipData table')
            assert(data.type == 27 and data.id == id,
                'each TooltipData table retains its requested outfit ID')
            assert(data.lines[1].leftText == '',
                'a missing saved-name record produces an empty title')
        end
        "#,
    )
    .unwrap();
}
