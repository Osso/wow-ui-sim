use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn pvp_no_state_defaults_do_not_override_state_backed_pvp_methods() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if C_PvP.IsInBrawl() ~= false then return "brawl" end
            if C_PvP.IsSoloShuffle() ~= false then return "solo-shuffle" end
            local spellID, startTime, duration = C_PvP.GetArenaCrowdControlInfo("player")
            if spellID ~= nil or startTime ~= 0 or duration ~= 0 then return "cc" end
            if C_PvP.GetLocklistMap(1) ~= 0 then return "locklist" end
            C_PvP.SetLocklistMap(566)
            if C_PvP.GetLocklistMap(1) ~= 566 then return "state-backed" end
            return "ok"
            "#,
        )
        .expect("PvP defaults and state-backed methods should be callable");

    assert_eq!(result, "ok");
}
