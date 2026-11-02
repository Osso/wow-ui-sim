//! Original Mainline reward rows must see the existing supplied currency model.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use wow_ui_sim::lua_api::{WowLuaEnv, state::QuestRewardCurrency};

fn reward_env() -> WowLuaEnv {
    let cache = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, _) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &cache,
        &[
            "Blizzard_AccessibilityTemplates",
            "Blizzard_FrameXML",
            "Blizzard_UIPanels_Game",
            "Blizzard_ObjectiveTracker",
            "Blizzard_SplashFrame",
        ],
        &[],
    );
    env.state().borrow_mut().selected_quest_log_id = Some(80000);
    env.exec(
        "QuestInfoFrame.questLog = false; QuestInfoFrame.rewardsFrame = QuestInfoRewardsFrame",
    )
    .unwrap();
    env
}

#[test]
#[cfg(feature = "client-retail")]
fn original_reward_panel_reads_cached_and_detail_spell_rewards() {
    let env = reward_env();
    env.exec(r#"
        QuestInfoFrame.questLog = true
        local original = C_QuestInfoSystem.GetQuestRewardSpells
        C_QuestInfoSystem.GetQuestRewardSpells = function(...)
            captured = { n = select('#', original(...)), original(...) }
            omitted = { n = select('#', original()), original() }
            return original(...)
        end
        seenSpellIDs = {}
        C_QuestInfoSystem.GetQuestRewardSpellInfo = function(_, spellID)
            seenSpellIDs[#seenSpellIDs + 1] = spellID
            return nil
        end
    "#).unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.quest_reward_spell_cache.insert(80000, vec![5, 0, 5, 6, 7, 8]);
        sim.quest_detail_reward_spells = Some((80000, [11, 11, 12]));
    }
    env.exec(r#"
        QuestInfo_ShowRewards()
        assert(captured.n == 1 and #captured[1] == 3)
        assert(captured[1][1] == 5 and captured[1][2] == 6 and captured[1][3] == 7)
        assert(omitted.n == 1 and #omitted[1] == 3)
        assert(#seenSpellIDs == 3 and seenSpellIDs[1] == 5 and seenSpellIDs[3] == 7)
    "#).unwrap();
    env.state().borrow_mut().quest_reward_spell_cache.remove(&80000);
    env.exec(r#"
        seenSpellIDs = {}
        QuestInfo_ShowRewards()
        assert(captured.n == 1 and #captured[1] == 3)
        assert(captured[1][1] == 11 and captured[1][2] == 11 and captured[1][3] == 12)
        assert(omitted.n == 0, 'omission must not use detail fallback')
        assert(#seenSpellIDs == 3 and seenSpellIDs[1] == 11 and seenSpellIDs[2] == 11)
    "#).unwrap();
    env.state().borrow_mut().quest_reward_spell_cache.insert(80000, vec![]);
    env.exec(r#"
        seenSpellIDs = {}
        QuestInfo_ShowRewards()
        assert(captured.n == 1 and #captured[1] == 0 and #seenSpellIDs == 0)
    "#).unwrap();
}

#[test]
fn original_reward_panel_is_empty_without_supplied_rewards() {
    let env = reward_env();
    let before = env.state().borrow().lua_error_counts.clone();
    env.exec("QuestInfo_ShowRewards()").unwrap();
    let shown: bool = env.eval("return QuestInfoRewardsFrame:IsShown()").unwrap();
    assert!(!shown);
    assert_eq!(env.state().borrow().lua_error_counts, before);
}

#[test]
fn supplied_currency_reaches_original_reward_panel_name_count_and_context() {
    let env = reward_env();
    let before = env.state().borrow().lua_error_counts.clone();
    {
        let mut sim = env.state().borrow_mut();
        let entry = sim
            .quest_log_entries
            .entries
            .iter_mut()
            .find(|entry| entry.quest_id == 80000)
            .unwrap();
        entry.currency_rewards.push(QuestRewardCurrency {
            currency_id: 1166,
            name: "Supplied quest tokens".into(),
            texture: "Interface\\Icons\\INV_Misc_Coin_01".into(),
            total_reward_amount: 5,
            base_reward_amount: Some(5),
        });
    }
    env.exec("QuestInfo_ShowRewards()").unwrap();
    let name: Option<String> = env.eval("for _, row in ipairs(QuestInfoRewardsFrame.RewardButtons) do if row:IsShown() and row.objectType == 'currency' then return row.Name:GetText() end end").unwrap();
    assert_eq!(
        name.as_deref(),
        Some("Timewarped Badge"),
        "supplied currency must reach the original row and its native container display override"
    );
    env.exec(
        r#"
        local row = QuestInfoRewardsFrame.RewardButtons[1]
        assert(QuestInfoRewardsFrame:IsShown() and row.objectType == 'currency')
        assert(row.Count:GetText() == '5')
        assert(row.currencyInfo.currencyID == 1166 and row.currencyInfo.totalRewardAmount == 5)
        assert(row.currencyInfo.name == 'Supplied quest tokens', 'projection preserves supplied metadata before original container display override')
        local modern = C_QuestInfoSystem.GetQuestRewardCurrencies(80000)
        local legacy = C_QuestLog.GetQuestRewardCurrencies(80000)
        assert(#modern == 1 and #legacy == 1 and modern[1].name == legacy[1].name)
        assert(modern[1].totalRewardAmount == legacy[1].totalRewardAmount)
        assert(C_QuestInfoSystem.GetQuestRewardCurrencies()[1].currencyID == 1166)
    "#,
    )
    .unwrap();
    env.state().borrow_mut().selected_quest_log_id = None;
    let arity: i32 = env
        .eval("return select('#', C_QuestInfoSystem.GetQuestRewardCurrencies())")
        .unwrap();
    assert_eq!(
        arity, 0,
        "no selected context permits no result per cached signature"
    );
    assert_eq!(env.state().borrow().lua_error_counts, before);
}
