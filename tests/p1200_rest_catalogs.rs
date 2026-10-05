#![cfg(feature = "retail-12-0-0")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1200_rest_crafting_quality_missing() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(C_TradeSkillUI.GetRecipeItemQualityInfo(999999, 1) == nil)
        assert(not pcall(C_TradeSkillUI.GetRecipeQualityReagentLink, 999999, 1, 1))
    "#,
    )
    .unwrap();
    use wow_ui_sim::c_api::c_trade_skill_quality::CraftingQualityInfo;
    {
        let mut state = env.state().borrow_mut();
        state.recipe_quality_inputs.item_quality.insert(
            (164, 2),
            CraftingQualityInfo {
                quality: 2,
                icon: "Professions-Icon-Quality-Tier2".into(),
                icon_small: "Professions-Icon-Quality-Tier2-Small".into(),
                ..Default::default()
            },
        );
        state
            .recipe_quality_inputs
            .reagent_links
            .insert((164, 3, 2), "|Hitem:2840|h[Copper Bar]|h".into());
    }
    env.exec(r#"
        local q = C_TradeSkillUI.GetRecipeItemQualityInfo(164, 2)
        assert(q.quality == 2 and q.icon == 'Professions-Icon-Quality-Tier2')
        assert(q.iconSmall == 'Professions-Icon-Quality-Tier2-Small')
        assert(type(q.barBackgroundCap) == 'string' and type(q.iconQuestObjective) == 'string')
        q.icon = 'mutated'
        assert(C_TradeSkillUI.GetRecipeItemQualityInfo(164, 2).icon == 'Professions-Icon-Quality-Tier2')
        assert(C_TradeSkillUI.GetRecipeItemQualityInfo(165, 2) == nil)
        assert(C_TradeSkillUI.GetRecipeQualityReagentLink(164, 3, 2) == '|Hitem:2840|h[Copper Bar]|h')
        assert(not pcall(C_TradeSkillUI.GetRecipeQualityReagentLink, 164, 3, 3))
    "#).unwrap();
}

#[test]
fn p1200_rest_pvp_catalog_empty() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(C_PvP.AreTrainingGroundsEnabled() == false)
        local allowed, reason = C_PvP.CanPlayerUseTrainingGroundsUI()
        assert(allowed == false and type(reason) == 'string')
        assert(#C_PvP.GetTrainingGrounds() == 0)
        assert(C_PvP.GetBattlegroundInfo(1) == nil)
        assert(C_PvP.HasRandomTrainingGroundWinToday() == false)
        assert(C_PvP.HasMatchStarted() == false)
        assert(not pcall(C_PvP.JoinTrainingGround, 999999))
    "#,
    )
    .unwrap();
    {
        use wow_ui_sim::c_api::c_pvp::catalog::BattlegroundInfo;
        let mut state = env.state().borrow_mut();
        state.pvp_catalog.training_enabled = true;
        state.pvp_catalog.training_eligible = true;
        state.pvp_catalog.random_training_win_today = true;
        state.pvp_catalog.battlegrounds.push(BattlegroundInfo {
            name: "Training Arena".into(),
            lfg_dungeon_id: Some(1001),
            battleground_id: Some(22),
            max_players: 6,
            game_type: "Arena".into(),
            is_training_ground: true,
            can_enter: true,
            ..Default::default()
        });
        state.pvp_catalog.battlegrounds.push(BattlegroundInfo {
            name: "Non-training BG".into(),
            ..Default::default()
        });
    }
    env.exec(
        r#"
        assert(C_PvP.AreTrainingGroundsEnabled())
        local allowed,reason = C_PvP.CanPlayerUseTrainingGroundsUI()
        assert(allowed and reason == '')
        assert(C_PvP.HasRandomTrainingGroundWinToday())
        assert(C_PvP.GetBattlegroundInfo(1).battlegroundID == 22)
        local list = C_PvP.GetTrainingGrounds()
        assert(#list == 1 and list[1].lfgDungeonID == 1001 and list[1].maxPlayers == 6)
        list[1].name = 'mutated'
        assert(C_PvP.GetBattlegroundInfo(1).name == 'Training Arena')
        C_PvP.JoinTrainingGround(1001)
        assert(not C_PvP.HasMatchStarted())
        AcceptBattlefieldPort(1, true)
        assert(C_PvP.HasMatchStarted())
        LeaveBattlefield()
        assert(not C_PvP.HasMatchStarted())
    "#,
    )
    .unwrap();
    assert_eq!(env.state().borrow().battlefield_queue.index, 0);
    env.state().borrow_mut().pvp_catalog.match_completed = true;
    env.exec("assert(C_PvP.HasMatchStarted())").unwrap();
}

#[test]
fn p1200_rest_prey_missing() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(select('#', C_QuestLog.GetActivePreyQuest()) == 0)
        assert(C_UIWidgetManager.GetPreyHuntProgressWidgetVisualizationInfo(999999) == nil)
    "#,
    )
    .unwrap();
}
