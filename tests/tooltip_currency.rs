use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
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
            local watched = C_TooltipInfo.GetCurrencyToken(1)

            if byId.type ~= Enum.TooltipDataType.Currency then
                return "currency_type"
            end
            if byId.id ~= 2245 then
                return "currency_id"
            end
            local currencyName = C_CurrencyInfo.GetCurrencyInfo(2245).name
            if currencyName == nil or currencyName == "" or byId.lines[1].leftText ~= currencyName then
                return "currency_name"
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
    if ACTIVE == ClientProfile::Retail {
        // R84 §Q4: the traced GetCurrencyByID producer does not publish a top-level quantity.
        let quantity_absent: bool = env
            .eval("return C_TooltipInfo.GetCurrencyByID(2245).quantity == nil")
            .unwrap();
        assert!(
            quantity_absent,
            "R84 §Q4: top-level quantity must be absent"
        );
        let override_quantity_absent: bool = env
            .eval("return C_TooltipInfo.GetCurrencyByID(2245, 99).quantity == nil")
            .unwrap();
        assert!(
            override_quantity_absent,
            "R84 §Q4: supplied amount must not become a top-level quantity"
        );
    }
}
