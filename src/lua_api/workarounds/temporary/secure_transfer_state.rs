//! Temporary `C_SecureTransfer` state surface.
//!
//! Secure transfer workflows are not backed by simulator state yet. Keep the
//! small Lua-visible state table and action counters explicit instead of
//! hiding them in the generic runtime bootstrap.

const SECURE_TRANSFER_STATE_LUA: &str = r#"
local function SecureTransferNamespaceFallback(t, key)
    if type(__wow_record_nil_symbol_access) == "function" then
        __wow_record_nil_symbol_access("C_SecureTransfer", key, nil, nil)
    end
    return nil
end

if type(C_SecureTransfer) ~= "table" then
    C_SecureTransfer = {}
end

local mt = getmetatable(C_SecureTransfer)
if mt == nil then
    setmetatable(C_SecureTransfer, { __index = SecureTransferNamespaceFallback })
elseif mt.__index == nil then
    mt.__index = SecureTransferNamespaceFallback
end

local state = rawget(C_SecureTransfer, "_state")
if type(state) ~= "table" then
    state = {
        shouldShowTradeOfferWarning = false,
        tradePartner = nil,
        mailInfo = {
            target = "",
            sendMoney = 0,
        },
        housingPurchaseCost = 0,
        housingPurchaseQuantity = 0,
        housingVCPurchaseProductID = 0,
        acceptTradeCount = 0,
        sendMailCount = 0,
        completeHousingPurchaseCount = 0,
        completeHousingVCPurchaseCount = 0,
        cancelCount = 0,
        lastAction = nil,
    }
    C_SecureTransfer._state = state
end

if rawget(C_SecureTransfer, "GetMailInfo") == nil then
    function C_SecureTransfer.GetMailInfo()
        local mailInfo = C_SecureTransfer._state.mailInfo or {}
        return {
            target = tostring(mailInfo.target or ""),
            sendMoney = tonumber(mailInfo.sendMoney) or 0,
        }
    end
end
if rawget(C_SecureTransfer, "GetTradePartner") == nil then
    function C_SecureTransfer.GetTradePartner()
        return C_SecureTransfer._state.tradePartner
    end
end
if rawget(C_SecureTransfer, "ShouldShowTradeOfferWarning") == nil then
    function C_SecureTransfer.ShouldShowTradeOfferWarning()
        return C_SecureTransfer._state.shouldShowTradeOfferWarning == true
    end
end
if rawget(C_SecureTransfer, "GetHousingPurchaseCost") == nil then
    function C_SecureTransfer.GetHousingPurchaseCost()
        return tonumber(C_SecureTransfer._state.housingPurchaseCost) or 0
    end
end
if rawget(C_SecureTransfer, "GetHousingPurchaseQuantity") == nil then
    function C_SecureTransfer.GetHousingPurchaseQuantity()
        return tonumber(C_SecureTransfer._state.housingPurchaseQuantity) or 0
    end
end
if rawget(C_SecureTransfer, "GetHousingVCPurchaseProductID") == nil then
    function C_SecureTransfer.GetHousingVCPurchaseProductID()
        return tonumber(C_SecureTransfer._state.housingVCPurchaseProductID) or 0
    end
end
if rawget(C_SecureTransfer, "AcceptTrade") == nil then
    function C_SecureTransfer.AcceptTrade()
        C_SecureTransfer._state.acceptTradeCount =
            (tonumber(C_SecureTransfer._state.acceptTradeCount) or 0) + 1
        C_SecureTransfer._state.lastAction = "AcceptTrade"
    end
end
if rawget(C_SecureTransfer, "SendMail") == nil then
    function C_SecureTransfer.SendMail()
        C_SecureTransfer._state.sendMailCount =
            (tonumber(C_SecureTransfer._state.sendMailCount) or 0) + 1
        C_SecureTransfer._state.lastAction = "SendMail"
    end
end
if rawget(C_SecureTransfer, "CompleteHousingPurchase") == nil then
    function C_SecureTransfer.CompleteHousingPurchase()
        C_SecureTransfer._state.completeHousingPurchaseCount =
            (tonumber(C_SecureTransfer._state.completeHousingPurchaseCount) or 0) + 1
        C_SecureTransfer._state.lastAction = "CompleteHousingPurchase"
    end
end
if rawget(C_SecureTransfer, "CompleteHousingVCPurchase") == nil then
    function C_SecureTransfer.CompleteHousingVCPurchase()
        C_SecureTransfer._state.completeHousingVCPurchaseCount =
            (tonumber(C_SecureTransfer._state.completeHousingVCPurchaseCount) or 0) + 1
        C_SecureTransfer._state.lastAction = "CompleteHousingVCPurchase"
    end
end
if rawget(C_SecureTransfer, "Cancel") == nil then
    function C_SecureTransfer.Cancel()
        C_SecureTransfer._state.cancelCount =
            (tonumber(C_SecureTransfer._state.cancelCount) or 0) + 1
        C_SecureTransfer._state.lastAction = "Cancel"
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(SECURE_TRANSFER_STATE_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_state_queries_actions_and_leaves_unknown_members_nil() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                C_SecureTransfer._state.mailInfo = { target = "Alt", sendMoney = "123" }
                C_SecureTransfer._state.tradePartner = "Merchant"
                C_TradeInfo._state = C_TradeInfo._state or {}
                C_TradeInfo._state.shouldShowTradeOfferWarning = true
                C_SecureTransfer._state.housingPurchaseCost = "4500"
                C_SecureTransfer._state.housingPurchaseQuantity = "3"
                C_SecureTransfer._state.housingVCPurchaseProductID = "77"

                C_SecureTransfer.AcceptTrade()
                C_SecureTransfer.SendMail()
                C_SecureTransfer.CompleteHousingPurchase()
                C_SecureTransfer.CompleteHousingVCPurchase()
                C_SecureTransfer.Cancel()

                local mailInfo = C_SecureTransfer.GetMailInfo()
                if mailInfo.target ~= "Alt" or mailInfo.sendMoney ~= 123 then
                    return "bad_mail"
                end
                if C_SecureTransfer.GetTradePartner() ~= "Merchant" then
                    return "bad_partner"
                end
                if not C_SecureTransfer.ShouldShowTradeOfferWarning() then
                    return "bad_warning"
                end
                if C_SecureTransfer.GetHousingPurchaseCost() ~= 4500 then
                    return "bad_cost"
                end
                if C_SecureTransfer.GetHousingPurchaseQuantity() ~= 3 then
                    return "bad_quantity"
                end
                if C_SecureTransfer.GetHousingVCPurchaseProductID() ~= 77 then
                    return "bad_product"
                end
                if C_SecureTransfer._state.acceptTradeCount ~= 1 then
                    return "bad_accept_count"
                end
                if C_SecureTransfer._state.sendMailCount ~= 1 then
                    return "bad_mail_count"
                end
                if C_SecureTransfer._state.completeHousingPurchaseCount ~= 1 then
                    return "bad_housing_count"
                end
                if C_SecureTransfer._state.completeHousingVCPurchaseCount ~= 1 then
                    return "bad_vc_count"
                end
                if C_SecureTransfer._state.cancelCount ~= 1 then
                    return "bad_cancel_count"
                end
                if C_SecureTransfer._state.lastAction ~= "Cancel" then
                    return "bad_last_action"
                end
                if C_SecureTransfer.SomeUnimplementedMember ~= nil then
                    return "fabricated_member"
                end
                return "ok"
                "#,
            )
            .expect("secure transfer state probe should run");

        assert_eq!(result, "ok");
    }
}
