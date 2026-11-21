//! Temporary `C_TradeInfo` warning/no-op defaults.
//!
//! Trade offer risk and trade-money pickup state are not modeled yet. Keep the
//! inert startup compatibility surface explicit here until trade state exists.

const TRADE_INFO_DEFAULTS_LUA: &str = r#"
C_TradeInfo = C_TradeInfo or __wow_namespace()

local function installTradeInfoDefault(name, fn)
    if rawget(C_TradeInfo, name) == nil then
        C_TradeInfo[name] = fn
    end
end

installTradeInfoDefault("ShouldShowTradeOfferWarning", function()
    return false
end)

installTradeInfoDefault("PickupTradeMoney", function(_amount)
end)
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(TRADE_INFO_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    #[test]
    fn installs_trade_info_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let (should_warn, pickup_returns): (bool, i32) = env
            .eval(
                r##"
                return C_TradeInfo.ShouldShowTradeOfferWarning(),
                    select("#", C_TradeInfo.PickupTradeMoney(1000))
                "##,
            )
            .expect("trade info defaults should be callable");

        assert!(!should_warn);
        assert_eq!(pickup_returns, 0);
    }

    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    #[test]
    fn preserves_existing_trade_info_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_TradeInfo = C_TradeInfo or __wow_namespace()

            function C_TradeInfo.ShouldShowTradeOfferWarning()
                return true
            end
            function C_TradeInfo.PickupTradeMoney(_amount)
                return "picked"
            end
            "#,
        )
        .expect("fixture should install existing C_TradeInfo providers");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (bool, String) = env
            .eval(
                r#"
                return C_TradeInfo.ShouldShowTradeOfferWarning(),
                    C_TradeInfo.PickupTradeMoney(1000)
                "#,
            )
            .expect("existing C_TradeInfo providers should remain callable");

        assert_eq!(result, (true, "picked".to_string()));
    }
}
