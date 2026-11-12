use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn campaign_metadata_and_state_are_not_inferred_from_quest_progress() {
    let env = env();
    {
        let mut sim = env.state().borrow_mut();
        sim.quest_log_entries.campaign_ids.insert(42, 77);
        sim.quest_log_entries.completed_quest_ids.insert(42);
    }
    let result: String = env
        .eval(
            r#"
            if C_CampaignInfo.GetCampaignID(42) ~= 77 then
                return "supplied_quest_campaign_id"
            end
            if C_CampaignInfo.GetCampaignInfo(77) ~= nil then
                return "quest_association_must_not_create_campaign_metadata"
            end
            if C_CampaignInfo.GetState(77) ~= Enum.CampaignState.Invalid then
                return "quest_completion_must_not_infer_campaign_state"
            end
            if C_CampaignInfo.GetState(99999) ~= Enum.CampaignState.Invalid then
                return "missing_campaign_should_be_invalid"
            end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}
