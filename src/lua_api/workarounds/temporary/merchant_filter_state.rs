//! Temporary merchant filter state.
//!
//! The simulator does not model merchant UI filtering yet. Keep the legacy
//! filter helpers explicit here and preserve the selected filter until merchant
//! inventory state owns it. The merchant count is live but ignores this filter.
//! Native persistence is UNKNOWN; preserve this module's current Lua-local state.

const MERCHANT_FILTER_STATE_LUA: &str = r#"
local merchantFilter = 0

if GetMerchantFilter == nil then
  function GetMerchantFilter()
    return merchantFilter
  end
end

if SetMerchantFilter == nil then
  function SetMerchantFilter(filter)
    merchantFilter = filter or 0
  end
end

"#;

// VERIFIED: Retail 12.1.5.70688's MerchantFrame_OnShow calls this global and
// the getter exposes reset value 2. PTR applicability is INFERRED from the shared path.
const MERCHANT_FILTER_RESET_LUA: &str = r#"
if ResetSetMerchantFilter == nil then
  function ResetSetMerchantFilter()
    SetMerchantFilter(2)
  end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(MERCHANT_FILTER_STATE_LUA)?;
    // Keep the legacy reset global scoped to the Retail consumer and inferred PTR path.
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) {
        lua.exec(MERCHANT_FILTER_RESET_LUA)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn stores_merchant_filter() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if GetMerchantFilter() ~= 0 then return "initial" end
                SetMerchantFilter(3)
                if GetMerchantFilter() ~= 3 then return "round_trip" end
                SetMerchantFilter(nil)
                if GetMerchantFilter() ~= 0 then return "reset" end
                return "ok"
                "#,
            )
            .expect("merchant filter probe should run");

        assert_eq!(result, "ok");
    }
}
