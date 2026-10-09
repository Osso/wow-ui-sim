//! Temporary unit threat defaults.
//!
//! Threat state is not modeled yet beyond the Rust-owned `UnitThreatSituation`
//! nil default. Keep the remaining inert threat helpers explicit here until a
//! real threat table exists.

const UNIT_THREAT_DEFAULTS_LUA: &str = r#"
if UnitDetailedThreatSituation == nil then
  function UnitDetailedThreatSituation(_unit, _other_unit)
    return false, 0, 0, 0, 0
  end
end

if UnitThreatPercentageOfLead == nil then
  function UnitThreatPercentageOfLead(_unit, _other_unit)
    return 0
  end
end

if GetThreatStatusColor == nil then
  local colors = {
    [0] = { 0.69, 0.69, 0.69 },
    [1] = { 1.00, 1.00, 0.47 },
    [2] = { 1.00, 0.60, 0.00 },
    [3] = { 1.00, 0.00, 0.00 },
  }

  function GetThreatStatusColor(status)
    local color = colors[status]
    if color == nil then return nil end
    return color[1], color[2], color[3]
  end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(UNIT_THREAT_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_unit_threat_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                local isTanking, status, scaledPercent, rawPercent, threatValue =
                    UnitDetailedThreatSituation("player", "target")
                if isTanking ~= false or status ~= 0 or scaledPercent ~= 0 or rawPercent ~= 0 or threatValue ~= 0 then
                    return "detailed"
                end
                if UnitThreatPercentageOfLead("player", "target") ~= 0 then return "lead" end
                local r, g, b = GetThreatStatusColor(3)
                if r ~= 1 or g ~= 0 or b ~= 0 then return "color" end
                if GetThreatStatusColor(99) ~= nil then return "unknown-color" end
                return "ok"
                "#,
            )
            .expect("unit threat defaults probe should run");

        assert_eq!(result, "ok");
    }
}
