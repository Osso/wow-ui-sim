#![cfg(feature = "client-retail")]

#[test]
fn campaign_header_reflects_quest_progress() {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_ObjectAPI", "Blizzard_UIPanels_Game"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_UIPanels_Game"));
    {
        let mut sim = env.state().borrow_mut();
        let log = &mut sim.quest_log_entries;
        log.entries.clear();
        log.completed_quest_ids.clear();
        log.campaign_ids.extend([(91001, 101), (91002, 102)]);
        log.completed_quest_ids.insert(91001);
        let mut active = wow_ui_sim::lua_api::state::QuestLogState::seeded()
            .entries
            .remove(0);
        active.quest_id = 91002;
        active.is_complete = false;
        log.entries.push(active);
    }
    env.exec(r#"
        local function check(id, complete)
            local h = { UpdateTextureKit = function() end, UpdateComplete = function(self, v) self.result = v end,
                UpdateTitle = function() end, UpdateProgress = function() end,
                UpdateNextObjective = function() end, MarkDirty = function() end, Show = function() end }
            CampaignHeaderDisplayMixin.SetCampaign(h, id)
            assert(h.result == complete, 'campaign header completion for ' .. id)
        end
        check(101, true)
        check(102, false)
        assert(C_CampaignInfo.GetCampaignID(91001) == 101, 'quest campaign lookup')
        assert(C_CampaignInfo.GetCampaignInfo(999999) == nil, 'unknown campaign info')
        assert(C_CampaignInfo.GetState(999999) == Enum.CampaignState.Invalid, 'unknown state')
    "#).expect("selected Blizzard campaign header consumes quest progress");
}
