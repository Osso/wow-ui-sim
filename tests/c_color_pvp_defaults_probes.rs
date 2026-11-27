use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn pvp_no_state_defaults_do_not_override_state_backed_pvp_methods() {
    let env = env();
    // Retail 12.0.5+/Forever model the crowd-control spell: no supplied spell
    // returns nothing. Other profiles keep the `nil, 0, 0` fallback.
    let cc_check = if cfg!(any(feature = "retail-12-0-5", feature = "client-wowforever")) {
        r##"if select("#", C_PvP.GetArenaCrowdControlInfo("player")) ~= 0 then return "cc" end"##
    } else {
        r#"local spellID, startTime, duration = C_PvP.GetArenaCrowdControlInfo("player")
            if spellID ~= nil or startTime ~= 0 or duration ~= 0 then return "cc" end"#
    };
    let result: String = env
        .eval(&format!(
            r#"
            if C_PvP.IsInBrawl() ~= false then return "brawl" end
            if C_PvP.IsSoloShuffle() ~= false then return "solo-shuffle" end
            {cc_check}
            if C_PvP.GetLocklistMap(1) ~= 0 then return "locklist" end
            C_PvP.SetLocklistMap(566)
            if C_PvP.GetLocklistMap(1) ~= 566 then return "state-backed" end
            return "ok"
            "#
        ))
        .expect("PvP defaults and state-backed methods should be callable");

    assert_eq!(result, "ok");
}
