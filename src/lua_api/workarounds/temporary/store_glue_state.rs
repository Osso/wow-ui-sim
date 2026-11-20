//! Temporary `C_StoreGlue` state surface.
//!
//! VAS/store glue flows are currently represented by small Lua-visible queues
//! and counters. Keep that compatibility fixture explicit until backed by a
//! real account/store subsystem.

use rilua::LuaApiMut;

const STORE_GLUE_STATE_LUA: &str = r#"
local modeledPurchaseInfo = ...
if type(C_StoreGlue) ~= "table" then
    C_StoreGlue = {}
end

local state = rawget(_G, "__wow_store_glue_state")
if type(state) ~= "table" then
    state = {
        disconnectOnLogout = false,
        vasProductReady = false,
        purchaseStateByGuid = {},
        requestedQueueGuids = {},
        requestCharacterQueueTimeCount = 0,
        updateVASPurchaseStatesCount = 0,
        lastRequestedQueueGuid = nil,
    }
    rawset(_G, "__wow_store_glue_state", state)
end

local function StoreGlueState()
    if type(state.purchaseStateByGuid) ~= "table" then
        state.purchaseStateByGuid = {}
    end
    if type(state.requestedQueueGuids) ~= "table" then
        state.requestedQueueGuids = {}
    end
    return state
end

C_StoreGlue._state = StoreGlueState()

if rawget(C_StoreGlue, "GetDisconnectOnLogout") == nil then
    function C_StoreGlue.GetDisconnectOnLogout()
        return StoreGlueState().disconnectOnLogout == true
    end
end

if rawget(C_StoreGlue, "GetVASProductReady") == nil then
    function C_StoreGlue.GetVASProductReady()
        return StoreGlueState().vasProductReady == true
    end
end

if not modeledPurchaseInfo and rawget(C_StoreGlue, "GetVASPurchaseStateInfo") == nil then
    function C_StoreGlue.GetVASPurchaseStateInfo(guid)
        local currentState = StoreGlueState()
        local record = currentState.purchaseStateByGuid[tostring(guid)] or currentState.purchaseStateByGuid[guid]
        if type(record) ~= "table" then
            return 0, 0, nil
        end
        return tonumber(record.purchaseState) or 0, tonumber(record.productID) or 0, record.result
    end
end

if rawget(C_StoreGlue, "RequestCharacterQueueTime") == nil then
    function C_StoreGlue.RequestCharacterQueueTime(guid)
        local currentState = StoreGlueState()
        table.insert(currentState.requestedQueueGuids, guid)
        currentState.requestCharacterQueueTimeCount = (tonumber(currentState.requestCharacterQueueTimeCount) or 0) + 1
        currentState.lastRequestedQueueGuid = guid
        return true
    end
end

if rawget(C_StoreGlue, "UpdateVASPurchaseStates") == nil then
    function C_StoreGlue.UpdateVASPurchaseStates()
        local currentState = StoreGlueState()
        currentState.updateVASPurchaseStatesCount = (tonumber(currentState.updateVASPurchaseStatesCount) or 0) + 1
        return true
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if crate::c_api::c_store_glue::enabled() {
        return Ok(());
    }
    let bootstrap = lua.load_bytes(STORE_GLUE_STATE_LUA.as_bytes(), "@store-glue-state")?;
    lua.call_function(
        &bootstrap,
        &[rilua::Val::Bool(crate::c_api::c_store_glue::enabled())],
    )?;
    Ok(())
}

#[cfg(all(
    test,
    not(any(feature = "client-retail", feature = "client-wowforever"))
))]
mod tests {
    use crate::lua_api::WowLuaEnv;

    const VAS_DEFAULTS_PROBE: &str = r#"
                if C_StoreGlue.GetDisconnectOnLogout() then
                    return "bad_disconnect_default"
                end
                if C_StoreGlue.GetVASProductReady() then
                    return "bad_ready_default"
                end
                C_StoreGlue._state.disconnectOnLogout = true
                C_StoreGlue._state.vasProductReady = true
                if not C_StoreGlue.GetDisconnectOnLogout() or not C_StoreGlue.GetVASProductReady() then
                    return "bad_boolean_state"
                end
                local purchaseState, productID, result = C_StoreGlue.GetVASPurchaseStateInfo("missing")
                if purchaseState ~= 0 or productID ~= 0 or result ~= nil then
                    return "bad_missing_purchase"
                end
                C_StoreGlue._state.purchaseStateByGuid["guid-1"] = {
                    purchaseState = 3,
                    productID = 99,
                    result = "ok",
                }
                purchaseState, productID, result = C_StoreGlue.GetVASPurchaseStateInfo("guid-1")
                if purchaseState ~= 3 or productID ~= 99 or result ~= "ok" then
                    return "bad_purchase"
                end
                if not C_StoreGlue.RequestCharacterQueueTime("guid-2") then
                    return "queue_request_failed"
                end
                if not C_StoreGlue.UpdateVASPurchaseStates() then
                    return "update_failed"
                end
                if C_StoreGlue._state.requestCharacterQueueTimeCount ~= 1 then
                    return "bad_queue_count"
                end
                if C_StoreGlue._state.lastRequestedQueueGuid ~= "guid-2" then
                    return "bad_last_guid"
                end
                if C_StoreGlue._state.updateVASPurchaseStatesCount ~= 1 then
                    return "bad_update_count"
                end
                return "ok"
                "#;

    #[test]
    fn installs_vas_defaults_and_request_counters() {
        if crate::c_api::c_store_glue::enabled() {
            return;
        }
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(VAS_DEFAULTS_PROBE)
            .expect("store glue probe should run");

        assert_eq!(result, "ok");
    }
}
