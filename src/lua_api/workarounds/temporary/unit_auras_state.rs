//! Temporary `C_UnitAuras` compatibility surface.
//!
//! Aura lookup and blocked-aura/provider-switch state are Rust-backed in
//! `globals::auras`. Private warning placement is owned by
//! `c_api::private_aura_anchors`.

const UNIT_AURAS_STATE_LUA: &str = r#"
if type(C_UnitAuras) ~= "table" then
    C_UnitAuras = {}
end

if type(GetBuildInfo) == "function" and select(4, GetBuildInfo()) >= 120100 then
    if type(C_UnitAuras._groupBuffVisualAlerts) ~= "table" then
        C_UnitAuras._groupBuffVisualAlerts = {}
    end
    if rawget(C_UnitAuras, "GetGroupBuffVisualAlerts") == nil then
        function C_UnitAuras.GetGroupBuffVisualAlerts()
            return C_UnitAuras._groupBuffVisualAlerts
        end
    end
    if rawget(C_UnitAuras, "SetGroupBuffVisualAlerts") == nil then
        function C_UnitAuras.SetGroupBuffVisualAlerts(alerts)
            C_UnitAuras._groupBuffVisualAlerts = type(alerts) == "table" and alerts or {}
        end
    end

    if type(C_UnitAuras._hiddenGroupBuffs) ~= "table" then
        C_UnitAuras._hiddenGroupBuffs = {}
    end
    if rawget(C_UnitAuras, "GetHiddenGroupBuffs") == nil then
        function C_UnitAuras.GetHiddenGroupBuffs()
            return C_UnitAuras._hiddenGroupBuffs
        end
    end
    if rawget(C_UnitAuras, "SetHiddenGroupBuffs") == nil then
        function C_UnitAuras.SetHiddenGroupBuffs(hiddenBuffs)
            C_UnitAuras._hiddenGroupBuffs = type(hiddenBuffs) == "table" and hiddenBuffs or {}
        end
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(UNIT_AURAS_STATE_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_private_warning_text_anchor_without_replacing_aura_state() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if type(C_UnitAuras.SetPrivateWarningTextAnchor) ~= "function" then
                    return "missing_warning_anchor"
                end
                if type(C_UnitAuras.AddBlockedAura) ~= "function" then
                    return "missing_blocked_aura"
                end
                if type(C_UnitAuras._blockedAuras) ~= "table" then
                    return "missing_blocked_state"
                end
                if C_UnitAuras._providerSwitched ~= false then
                    return "bad_provider_state"
                end
                local parent = CreateFrame("Frame")
                C_UnitAuras.SetPrivateWarningTextAnchor(parent, {
                    point = "TOP", relativeTo = parent, relativePoint = "TOP",
                    offsetX = 1, offsetY = 2,
                })
                return "ok"
                "#,
            )
            .expect("unit aura compatibility probe should run");

        assert_eq!(result, "ok");
    }
}
