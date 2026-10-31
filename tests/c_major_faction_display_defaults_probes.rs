use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn major_faction_display_defaults_do_not_replace_state_backed_data() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if C_MajorFactions.GetMajorFactionData(999999) ~= nil then return "state-backed-data" end
            if #C_MajorFactions.GetRenownRewardsForLevel(2507, 1) ~= 0 then return "rewards" end
            if C_MajorFactions.ShouldUseJourneyRewardTrack(2507) ~= false then return "journey" end
            if C_MajorFactions.GetRenownNPCFactionID(2507) ~= 0 then return "npc" end
            return "ok"
            "#,
        )
        .expect("major faction display defaults should be callable");

    assert_eq!(result, "ok");
}
