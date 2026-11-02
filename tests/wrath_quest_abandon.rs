//! Wrath abandonment through the unchanged full-startup selection/button/popup consumers.
#![cfg(feature = "client-wrath")]

#[path = "common/wrath_full_ui.rs"]
mod wrath_full_ui;

use sha2::{Digest, Sha256};
use std::ops::Deref;
use wow_ui_sim::c_api::quest_objectives::QuestObjectiveProgress;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::GossipQuestRow;

struct ConsumerEnv {
    env: WowLuaEnv,
    startup_errors: String,
}

impl Deref for ConsumerEnv {
    type Target = WowLuaEnv;

    fn deref(&self) -> &Self::Target {
        &self.env
    }
}

fn error_snapshot(env: &WowLuaEnv) -> String {
    let sim = env.state().borrow();
    format!("{:?}|{:?}", sim.lua_errors, sim.lua_error_counts)
}

fn assert_errors(env: &ConsumerEnv) {
    let errors: String = env
        .eval(r"return table.concat(AbandonErrors, '\n')")
        .unwrap();
    assert!(errors.is_empty(), "{errors}");
    assert_eq!(
        error_snapshot(env),
        env.startup_errors,
        "new consumer errors"
    );
}

fn exec(env: &ConsumerEnv, source: &str) {
    let result = env.exec(source);
    eprintln!("direct eval {source:?}: {result:?}");
    assert_errors(env);
    result.expect("original consumer and behavioral assertions");
}

fn assert_originals() {
    let toc = wow_ui_sim::client_profile::blizzard_ui_framexml_toc().unwrap();
    for (file, expected) in [
        (
            "UIParent.lua",
            "a48687367d05ba9965bdd7a43bdecb92666c8ba74873bd7d9da08b205b34e177",
        ),
        (
            "Constants.lua",
            "367f5024098edda7d8dbcc9f57cf2616c85da4a3a9eb9851d2d4fc0bea8b72fd",
        ),
        (
            "QuestLogFrame.lua",
            "6ac6ca92b3036417082b88cc2f8b5c9fc8889f51f82aa726fe0bcd44990bc5e2",
        ),
        (
            "QuestInfo.lua",
            "3bfe691acdb2a095a6633c3ce7bef4a13f53f464d20781b2e850e442f9b715cd",
        ),
        (
            "StaticPopup.lua",
            "1134b08e87d8577c0353895207eaec24e74e8f669775a49257a82431a4ac3042",
        ),
        (
            "StaticPopup.xml",
            "a575f35cc8e9e3a585c6ccd5db40f650dbd4f153e62be6736f1101c3f33dde7b",
        ),
        (
            "QuestLogFrame.xml",
            "530c396e36af4b23a725086d37b6cb68abaf1966db77ba26c4e3fed3ce76f154",
        ),
    ] {
        let path = toc.parent().unwrap().join(file);
        let bytes = std::fs::read(&path).unwrap();
        let hash = format!("{:x}", Sha256::digest(bytes));
        assert_eq!(hash, expected, "source drift: {}", path.display());
        eprintln!("original {} sha256={hash}", path.display());
    }
}

fn seed_env() -> ConsumerEnv {
    assert_originals();
    let env = wrath_full_ui::load_original_wrath_startup();
    let startup_errors = error_snapshot(&env);
    eprintln!("retained full-startup errors: {startup_errors}");
    env.exec("AbandonErrors = {}; seterrorhandler(function(e) AbandonErrors[#AbandonErrors+1] = tostring(e) end)").unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.quest_log_entries.entries = [
            (990101, "Wrath abandon first"),
            (990202, "Wrath abandon second"),
        ]
        .into_iter()
        .map(|(quest_id, title)| {
            sim.quest_log_entries
                .snapshot_gossip_offer(&GossipQuestRow {
                    quest_id,
                    title: title.into(),
                    quest_level: 23,
                    ..Default::default()
                })
                .unwrap()
        })
        .collect();
        sim.quest_log = vec![990101, 990202];
        sim.pending_quest_offer = None;
        sim.selected_quest_log_id = None;
        sim.abandon_quest_id = None;
        for id in [990101, 990202] {
            sim.quest_log_entries
                .objective_progress
                .insert(id, QuestObjectiveProgress::default());
            sim.quest_log_entries
                .background_materials
                .insert(id, format!("material-{id}"));
            sim.quest_log_entries.readiness.auto_complete.insert(id);
            sim.quest_log_entries.readiness.notified.insert(id);
            sim.quest_log_entries
                .readiness
                .popups
                .push((id, "OFFER".into()));
        }
    }
    let env = ConsumerEnv {
        env,
        startup_errors,
    };
    exec(
        &env,
        r#"
        assert(QuestLogFrameAbandonButton and StaticPopup1, 'original startup widgets')
        AbandonEvents = {}
        local observer = CreateFrame('Frame')
        for _, event in ipairs({'QUEST_LOG_UPDATE', 'QUEST_REMOVED', 'QUEST_ACCEPTED', 'QUEST_TURNED_IN'}) do observer:RegisterEvent(event) end
        observer:SetScript('OnEvent', function(_, event, ...)
            AbandonEvents[#AbandonEvents+1] = {event, GetNumQuestLogEntries(), GetQuestLogTitle(1), select('#', ...)}
        end)
    "#,
    );
    env
}

// Deliver only notifications actually queued by the producer, through registered
// original listeners. This is not a native event-timing claim.
fn dispatch_queued_abandon_events(env: &ConsumerEnv) -> usize {
    let events = env.state().borrow_mut().events.drain();
    let count = events.len();
    for event in events {
        assert_eq!(event.name, "QUEST_LOG_UPDATE");
        assert!(event.args.is_empty());
        env.fire_event(&event.name).unwrap();
        assert_errors(env);
    }
    count
}

fn snapshot(env: &WowLuaEnv) -> String {
    let sim = env.state().borrow();
    format!(
        "{:?}|{:?}|{:?}|{:?}|{}|{}",
        sim.quest_log,
        sim.quest_log_entries,
        sim.pending_quest_offer,
        sim.events.pending(),
        sim.player.money,
        sim.player.xp
    )
}

#[test]
fn original_difficulty_color_uses_player_level_green_bands() {
    let env = seed_env();
    for (level, range) in [
        (9, 4),
        (10, 5),
        (19, 5),
        (20, 6),
        (29, 6),
        (30, 7),
        (39, 7),
        (40, 8),
        (45, 8),
        (55, 8),
        (60, 8),
        (80, 8),
        (95, 8),
        (1000, 8),
    ] {
        env.state().borrow_mut().player.level = level;
        exec(
            &env,
            &format!(
                r#"
            local function assertColor(color, r, g, b)
                assert(color.r == r and color.g == g and color.b == b, 'original difficulty color at player level {level}')
            end
            assertColor(GetQuestDifficultyColor({green}), 0.25, 0.75, 0.25)
            assertColor(GetQuestDifficultyColor({gray}), 0.50, 0.50, 0.50)
            assertColor(GetQuestDifficultyColor({red}), 1.00, 0.10, 0.10)
            assertColor(GetQuestDifficultyColor({orange}), 1.00, 0.50, 0.25)
            assertColor(GetQuestDifficultyColor({yellow}), 1.00, 1.00, 0.00)
            assert(select('#', GetQuestGreenRange()) == 1)
            assert(GetQuestGreenRange() == {range}, 'live player-derived range')
            assert(GetQuestGreenRange('player') == {range})
            assert(GetQuestGreenRange('target', false, {{}}) == {range}, 'arguments ignored')
        "#,
                green = level - range,
                gray = level - range - 1,
                red = level + 5,
                orange = level + 3,
                yellow = level - 2
            ),
        );
    }
}

#[test]
fn original_standalone_detail_button_cancel_accepts_marked_target() {
    let env = seed_env();
    exec(
        &env,
        r#"
        QuestLog_SetSelection(1)
        assert(not QuestLogFrame:IsShown(), 'standalone original detail route')
        assert(QuestLogDetailFrame:IsShown())
        assert(QuestLogControlPanel:GetParent() == QuestLogDetailFrame)
        assert(QuestLogFrameAbandonButton:IsVisible(), 'real original control is visible')
    "#,
    );
    let before = snapshot(&env);
    exec(
        &env,
        r#"
        QuestLogFrameAbandonButton:Click()
        local popup = StaticPopup_FindVisible('ABANDON_QUEST')
        assert(popup and popup:IsVisible())
        assert(string.find(popup.text:GetText(), 'Wrath abandon first', 1, true))
        popup.button2:Click()
        assert(not popup:IsShown())
    "#,
    );
    assert_eq!(
        snapshot(&env),
        before,
        "visible original cancel preserves state"
    );
    let history = env
        .state()
        .borrow()
        .quest_log_entries
        .completed_quest_ids
        .clone();
    let economy = {
        let sim = env.state().borrow();
        (sim.player.money, sim.player.xp)
    };
    exec(
        &env,
        r#"
        QuestLogFrameAbandonButton:Click()
        SelectQuestLogEntry(2)
        StaticPopup_FindVisible('ABANDON_QUEST').button1:Click()
        assert(not QuestLogDetailFrame:IsShown(), 'original confirmation hides detail')
    "#,
    );
    assert_eq!(dispatch_queued_abandon_events(&env), 1);
    {
        let sim = env.state().borrow();
        assert_eq!(sim.quest_log, vec![990202]);
        assert_eq!(sim.quest_log_entries.entries.len(), 1);
        assert_eq!(sim.quest_log_entries.entries[0].quest_id, 990202);
        assert_eq!(sim.quest_log_entries.completed_quest_ids, history);
        assert_eq!((sim.player.money, sim.player.xp), economy);
        assert_eq!(sim.abandon_quest_id, Some(990101));
    }
    exec(
        &env,
        "assert(#AbandonEvents == 1 and AbandonEvents[1][3] == 'Wrath abandon second'); AbandonQuest()",
    );
    assert_eq!(dispatch_queued_abandon_events(&env), 0);
    assert_eq!(env.state().borrow().quest_log, vec![990202]);
}

#[test]
fn original_button_cancel_then_accept_marked_target() {
    let env = seed_env();
    exec(
        &env,
        r#"
        SelectQuestLogEntry(1)
        ShowUIPanel(QuestLogFrame)
        assert(QuestLogFrame:IsShown(), 'original log refresh must not return early')
        local firstTitle = QuestLogScrollFrame.buttons[1]:GetText()
        local secondTitle = QuestLogScrollFrame.buttons[2]:GetText()
        assert(firstTitle == '  Wrath abandon first', firstTitle or '<nil first title>')
        assert(secondTitle == '  Wrath abandon second', secondTitle or '<nil second title>')
        "#,
    );
    let before = snapshot(&env);
    let clicked = env.exec("QuestLogFrameAbandonButton:Click()");
    eprintln!("original button result: {clicked:?}");
    assert_eq!(snapshot(&env), before, "marking must not delete or notify");
    assert_errors(&env);
    clicked.unwrap();
    exec(
        &env,
        r#"
        local popup = StaticPopup_FindVisible('ABANDON_QUEST')
        assert(popup and popup:IsShown(), 'original popup shown')
        assert(string.find(popup.text:GetText(), 'Wrath abandon first', 1, true), 'marked supplied title')
        popup.button2:Click()
        assert(not popup:IsShown(), 'real cancel button dismisses')
    "#,
    );
    assert_eq!(snapshot(&env), before, "cancel preserves all state");
    exec(
        &env,
        "QuestLogFrameAbandonButton:Click(); SelectQuestLogEntry(2)",
    );
    let history = env
        .state()
        .borrow()
        .quest_log_entries
        .completed_quest_ids
        .clone();
    let economy = {
        let sim = env.state().borrow();
        (sim.player.money, sim.player.xp)
    };
    exec(
        &env,
        "assert(GetAbandonQuestName() == 'Wrath abandon first'); StaticPopup_FindVisible('ABANDON_QUEST').button1:Click()",
    );
    assert_eq!(dispatch_queued_abandon_events(&env), 1);
    {
        let sim = env.state().borrow();
        assert_eq!(sim.quest_log, vec![990202]);
        assert_eq!(
            sim.quest_log_entries
                .entries
                .iter()
                .map(|row| row.quest_id)
                .collect::<Vec<_>>(),
            vec![990202]
        );
        assert_eq!(sim.quest_log_entries.completed_quest_ids, history);
        assert_eq!((sim.player.money, sim.player.xp), economy);
        assert!(
            !sim.quest_log_entries
                .objective_progress
                .contains_key(&990101)
        );
        assert!(
            !sim.quest_log_entries
                .background_materials
                .contains_key(&990101)
        );
        assert!(
            !sim.quest_log_entries
                .readiness
                .auto_complete
                .contains(&990101)
        );
        assert!(!sim.quest_log_entries.readiness.notified.contains(&990101));
        assert!(
            !sim.quest_log_entries
                .readiness
                .popups
                .iter()
                .any(|(id, _)| *id == 990101)
        );
        assert!(
            sim.quest_log_entries
                .objective_progress
                .contains_key(&990202)
        );
        assert!(
            sim.quest_log_entries
                .background_materials
                .contains_key(&990202)
        );
        assert!(
            sim.quest_log_entries
                .readiness
                .auto_complete
                .contains(&990202)
        );
    }
    exec(
        &env,
        r#"
        assert(#AbandonEvents == 1 and AbandonEvents[1][1] == 'QUEST_LOG_UPDATE', 'Wrath log notification, no modern removal event')
        assert(AbandonEvents[1][2] == 1 and AbandonEvents[1][3] == 'Wrath abandon second', 'listener sees committed log')
        assert(AbandonEvents[1][4] == 0, 'argumentless log update')
        assert(QuestLogFrame:IsShown(), 'original log remains visible after confirmation')
        assert(QuestLogScrollFrame.buttons[1]:GetText() == '  Wrath abandon second', 'original log listener renders surviving row')
        assert(not QuestLogScrollFrame.buttons[2]:IsShown(), 'original log listener hides removed row')
        StaticPopup1Button1:Click()
        AbandonQuest()
        assert(#AbandonEvents == 1, 'repeat does not notify')
    "#,
    );
    assert_eq!(dispatch_queued_abandon_events(&env), 0);
    assert_eq!(env.state().borrow().quest_log, vec![990202]);
}

#[test]
fn original_popup_removals_deliver_unit_changes_and_coalesce_log_refresh() {
    let env = seed_env();
    exec(
        &env,
        r#"
        ResponseEvents = {}
        local observer = CreateFrame('Frame')
        observer:RegisterEvent('UNIT_QUEST_LOG_CHANGED')
        observer:RegisterEvent('QUEST_LOG_UPDATE')
        observer:SetScript('OnEvent', function(_, event, ...)
            ResponseEvents[#ResponseEvents+1] = {event, select('#', ...), ..., GetNumQuestLogEntries()}
        end)
        SelectQuestLogEntry(1); SetAbandonQuest(); QuestLogControlPanel_UpdateState()
        QuestLogFrameAbandonButton:Click()
        StaticPopup_FindVisible('ABANDON_QUEST').button1:Click()
        assert(#ResponseEvents == 1 and ResponseEvents[1][1] == 'UNIT_QUEST_LOG_CHANGED', 'unit signal delivered before confirmation returns')
        assert(ResponseEvents[1][2] == 1 and ResponseEvents[1][3] == 'player' and ResponseEvents[1][4] == 1, 'unit listener sees player and committed membership')
        SelectQuestLogEntry(1); SetAbandonQuest(); QuestLogControlPanel_UpdateState()
        QuestLogFrameAbandonButton:Click()
        StaticPopup_FindVisible('ABANDON_QUEST').button1:Click()
        assert(#ResponseEvents == 2 and ResponseEvents[2][4] == 0, 'second direct unit signal precedes queued refresh')
    "#,
    );
    assert_eq!(dispatch_queued_abandon_events(&env), 1, "coalesced refresh");
    exec(
        &env,
        "assert(#ResponseEvents == 3 and ResponseEvents[3][1] == 'QUEST_LOG_UPDATE' and ResponseEvents[3][2] == 0)",
    );
}

#[test]
fn original_selection_renders_supplied_item_reward() {
    let env = seed_env();
    env.state().borrow_mut().quest_log_entries.entries[0]
        .reward_items
        .push(wow_ui_sim::lua_api::state::QuestRewardItem {
            name: "Wrath supplied reward".into(),
            texture: "Interface/Icons/INV_Box_01".into(),
            count: 3,
            quality: 2,
            is_usable: true,
            item_id: None,
        });
    exec(&env, "QuestLog_SetSelection(1)");
    exec(
        &env,
        r#"
        assert(GetQuestLogRewardSpell() == nil, 'absent spell is nil, never numeric zero')
        assert(GetQuestLogRewardArenaPoints() == 0 and GetQuestLogRewardTalents() == 0, 'numeric empty rewards')
        assert(GetNumQuestLogRewardFactions() == 0, 'absent faction rewards')
        assert(QuestInfoRewardsFrame:IsShown() and QuestInfoItem1:IsShown())
        assert(QuestInfoItem1Name:GetText() == 'Wrath supplied reward')
        assert(GetNumQuestLogRewards() == 1)
        QuestLog_SetSelection(2)
        assert(GetNumQuestLogRewards() == 0)
        assert(not QuestInfoRewardsFrame:IsShown(), 'empty row hides rewards')
    "#,
    );
}

#[test]
fn original_daily_quest_counter_counts_slots_not_history() {
    let env = seed_env();
    {
        let mut sim = env.state().borrow_mut();
        sim.quest_log_entries.daily_quest_ids[0] = 710101;
        sim.quest_log_entries.daily_quest_ids[7] = 710101;
        sim.quest_log_entries.daily_quest_ids[24] = 710103;
        sim.quest_log_entries.completed_quest_ids =
            [710001, 710002, 710003, 710004].into_iter().collect();
    }
    exec(
        &env,
        r#"
        QuestLog_UpdateQuestCount(2)
        assert(select('#', GetDailyQuestsCompleted()) == 1 and select('#', GetMaxDailyQuests()) == 1)
        assert(GetDailyQuestsCompleted() == 3 and GetMaxDailyQuests() == 25)
        assert(QuestLogDailyQuestCount:IsShown())
        assert(QuestLogDailyQuestCount:GetText() == format(QUEST_LOG_DAILY_COUNT_TEMPLATE, 3, 25))
    "#,
    );
    env.state()
        .borrow_mut()
        .quest_log_entries
        .daily_quest_ids
        .fill(0);
    exec(
        &env,
        r#"
        QuestLog_UpdateQuestCount(2)
        assert(GetDailyQuestsCompleted() == 0 and GetMaxDailyQuests() == 25)
        assert(not QuestLogDailyQuestCount:IsShown())
    "#,
    );
    assert_eq!(
        env.state()
            .borrow()
            .quest_log_entries
            .completed_quest_ids
            .len(),
        4
    );
}

#[test]
fn original_selection_renders_supplied_spell_and_points() {
    let env = seed_env();
    {
        let mut sim = env.state().borrow_mut();
        let row = &mut sim.quest_log_entries.entries[0];
        row.reward_spell = Some(wow_ui_sim::lua_api::state::QuestRewardSpell {
            texture: "Interface/Icons/Spell_Holy_MagicalSentry".into(),
            name: "Wrath supplied spell".into(),
            is_trade_skill: false,
            is_learned: true,
        });
        row.reward_arena_points = 17;
        row.reward_talents = 2;
    }
    exec(&env, "QuestLog_SetSelection(1)");
    exec(
        &env,
        r#"
        assert(QuestInfoItem1Name:GetText() == 'Wrath supplied spell')
        assert(QuestInfoArenaPointsFramePoints:GetText() == '17')
        assert(QuestInfoTalentFramePoints:GetText() == '2')
        local texture, name, tradeSkill, learned = GetQuestLogRewardSpell()
        assert(texture == 'Interface/Icons/Spell_Holy_MagicalSentry' and name == 'Wrath supplied spell')
        assert(tradeSkill == false and learned == true)
        QuestLog_SetSelection(2)
        assert(GetQuestLogRewardSpell() == nil)
        assert(GetQuestLogRewardArenaPoints() == 0 and GetQuestLogRewardTalents() == 0)
        assert(not QuestInfoRewardsFrame:IsShown())
    "#,
    );
}

#[test]
fn original_reward_processing_prepares_shared_faction_output() {
    let env = seed_env();
    exec(
        &env,
        "assert(select('#', GetNumQuestLogRewardFactions()) == 1); assert(GetNumQuestLogRewardFactions() == 0)",
    );
    env.state().borrow_mut().quest_log_entries.entries[0].reward_factions =
        vec![(72, 500), (47, -100)];
    exec(&env, "QuestLog_SetSelection(1)");
    exec(
        &env,
        r#"
        assert(GetNumQuestLogRewardFactions() == 2, 'original QuestInfo_ShowRewards prepares the shared output')
        local id, amount = GetQuestLogRewardFactionInfo(1)
        assert(id == 72 and amount == 500)
        id, amount = GetQuestLogRewardFactionInfo(2)
        assert(id == 47 and amount == -100)
        SelectQuestLogEntry(2)
        assert(GetNumQuestLogRewardFactions() == 2, 'reader does not auto-refresh selection')
        CloseQuestFrame()
        assert(GetNumQuestLogRewardFactions() == 2, 'dialog clear preserves prepared output')
        local id, amount = GetQuestLogRewardFactionInfo(0)
        assert(id == 0 and amount == 0)
        id, amount = GetQuestLogRewardFactionInfo(99)
        assert(id == 0 and amount == 0)
    "#,
    );
    let events = env.state().borrow().events.pending().len();
    exec(
        &env,
        "ProcessQuestLogRewardFactions(); assert(GetNumQuestLogRewardFactions() == 0)",
    );
    assert_eq!(
        env.state().borrow().events.pending().len(),
        events,
        "processing emits no event"
    );
}

#[test]
fn original_popup_confirmation_retains_marked_identity() {
    let env = seed_env();
    exec(
        &env,
        "SelectQuestLogEntry(1); SetAbandonQuest(); QuestLogControlPanel_UpdateState(); QuestLogFrameAbandonButton:Click(); SelectQuestLogEntry(2)",
    );
    {
        let mut sim = env.state().borrow_mut();
        sim.quest_log_entries.entries.reverse();
        sim.quest_log.reverse();
    }
    exec(
        &env,
        "assert(GetQuestLogSelection() == 1, 'selection resolves the reordered current log')",
    );
    exec(
        &env,
        "StaticPopup_FindVisible('ABANDON_QUEST').button1:Click()",
    );
    assert_eq!(env.state().borrow().quest_log, vec![990202]);
    assert_eq!(
        env.state().borrow().abandon_quest_id,
        Some(990101),
        "confirmation retains the marked quest ID, not the current selection"
    );
    let before = snapshot(&env);
    exec(&env, "AbandonQuest()");
    assert_eq!(snapshot(&env), before, "missing marked row is a no-op");
    assert_eq!(env.state().borrow().abandon_quest_id, Some(990101));
}

#[test]
fn invalid_and_stale_targets_do_not_remove_other_rows() {
    let env = seed_env();
    let before = snapshot(&env);
    exec(
        &env,
        "SelectQuestLogEntry(1); SetAbandonQuest(); SelectQuestLogEntry(99); SetAbandonQuest(); AbandonQuest()",
    );
    assert!(env.state().borrow().abandon_quest_id.is_none());
    assert_eq!(snapshot(&env), before);
    env.state().borrow_mut().abandon_quest_id = Some(999999);
    exec(&env, "AbandonQuest()");
    assert_eq!(env.state().borrow().abandon_quest_id, Some(999999));
    assert_eq!(snapshot(&env), before);
    exec(
        &env,
        "SelectQuestLogEntry(2); SetAbandonQuest(); AbandonQuest(); AbandonQuest()",
    );
    assert_eq!(env.state().borrow().quest_log, vec![990101]);
    assert_eq!(dispatch_queued_abandon_events(&env), 1);
    exec(
        &env,
        "assert(#AbandonEvents == 1 and AbandonEvents[1][1] == 'QUEST_LOG_UPDATE')",
    );
}

#[test]
fn confirmation_clears_only_matching_pending_offer_and_selection() {
    let env = seed_env();
    {
        let mut sim = env.state().borrow_mut();
        sim.pending_quest_offer = Some(sim.quest_log_entries.entries[1].clone());
    }
    exec(
        &env,
        "SelectQuestLogEntry(1); SetAbandonQuest(); AbandonQuest()",
    );
    {
        let sim = env.state().borrow();
        assert_eq!(sim.pending_quest_offer.as_ref().unwrap().quest_id, 990202);
        assert!(sim.selected_quest_log_id.is_none());
    }
    exec(&env, "SelectQuestLogEntry(1); SetAbandonQuest()");
    assert!(env.state().borrow().pending_quest_offer.is_some());
    exec(&env, "AbandonQuest()");
    let sim = env.state().borrow();
    assert!(sim.pending_quest_offer.is_none());
    assert!(sim.quest_log.is_empty());
    assert!(sim.quest_log_entries.entries.is_empty());
    assert!(sim.selected_quest_log_id.is_none());
}

#[test]
fn original_selection_zero_is_nondestructive() {
    let env = seed_env();
    let before = snapshot(&env);
    exec(&env, "QuestLog_SetSelection(0)");
    assert_eq!(snapshot(&env), before);
    assert!(env.state().borrow().abandon_quest_id.is_none());
}

#[test]
fn original_selection_renders_supplied_suggested_group_size() {
    let env = seed_env();
    env.state().borrow_mut().quest_log_entries.entries[0].suggested_group_size = Some(5);
    exec(&env, "QuestLog_SetSelection(1)");
    exec(
        &env,
        r#"
        assert(select('#', GetQuestLogGroupNum()) == 1, 'one numeric return')
        assert(GetQuestLogGroupNum() == 5)
        assert(QuestInfoGroupSize:IsShown())
        assert(QuestInfoGroupSize:GetText() == format(QUEST_SUGGESTED_GROUP_NUM, 5))
        QuestLog_SetSelection(2)
        assert(GetQuestLogGroupNum() == 0)
        assert(not QuestInfoGroupSize:IsShown())
        SelectQuestLogEntry(0)
        assert(GetQuestLogGroupNum() == 0, 'absent selection')
    "#,
    );
    env.state().borrow_mut().selected_quest_log_id = Some(999999);
    exec(
        &env,
        "assert(GetQuestLogGroupNum() == 0, 'absent rich row')",
    );
}

#[test]
fn original_nonzero_selection_is_nondestructive() {
    let env = seed_env();
    let before = snapshot(&env);
    let selected = env.exec("QuestLog_SetSelection(1)");
    eprintln!("original full selection result: {selected:?}");
    assert_eq!(snapshot(&env), before, "ordinary selection must not delete");
    assert_errors(&env);
    selected.unwrap();
}
