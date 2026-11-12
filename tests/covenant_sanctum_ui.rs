use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn non_retail_covenant_sanctum_get_renown_rewards_returns_seeded_scenario() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local rewards = C_CovenantSanctumUI.GetRenownRewardsForLevel(1, 5)
            if #rewards ~= 1 then
                return "expected_one_seeded_reward"
            end

            local reward = rewards[1]
            if reward.name ~= "Path of Ascension" then
                return "expected_seeded_reward_name"
            end
            if reward.description ~= "Unlocks a new covenant activity." then
                return "expected_seeded_reward_description"
            end
            if reward.toastDescription ~= "Path of Ascension unlocked" then
                return "expected_seeded_toast_description"
            end
            if reward.icon ~= 4089529 then
                return "expected_seeded_reward_icon"
            end

            local unknownRewards = C_CovenantSanctumUI.GetRenownRewardsForLevel(9, 5)
            if #unknownRewards ~= 0 then
                return "unknown_covenant_should_have_no_rewards"
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "non-Retail profiles retain the prior simulator reward scenario"
    );
}
