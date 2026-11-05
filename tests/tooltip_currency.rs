use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn c_tooltip_info_currency_getters_expose_seeded_currency_lines() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local byId = C_TooltipInfo.GetCurrencyByID(2245)
            local overridden = C_TooltipInfo.GetCurrencyByID(2245, 99)
            local watched = C_TooltipInfo.GetCurrencyToken(1)

            if byId.type ~= Enum.TooltipDataType.Currency then
                return "currency_type"
            end
            if byId.id ~= 2245 then
                return "currency_id"
            end
            if byId.lines[1].leftText ~= "Valorstones" then
                return "currency_name"
            end
            -- WM97 DQ6: the supplied quantity is composition data, not an invented
            -- "Amount: N/cap" text line.
            if byId.quantity ~= 1847 then
                return "currency_quantity"
            end
            if overridden.quantity ~= 99 then
                return "currency_override"
            end
            for index, line in ipairs(byId.lines) do
                local text = line.leftText or ""
                if text:find("Amount:", 1, true) then
                    return "fabricated_amount_line"
                end
            end
            if watched.lines[1].leftText ~= "Valorstones" then
                return "currency_token"
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "Currency tooltip getters should expose seeded currency tooltip data"
    );
}
