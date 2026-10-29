//! Ordinary 3.3.5 acceptance through unchanged full FrameXML.
//! Laptop packet 20261018 items 1–2 governs events; giver identity is supplied.
//! Missing-player lifecycle is not modeled by the current always-present player.
#![cfg(feature = "client-wrath")]

#[path = "common/wrath_full_ui.rs"]
mod wrath_full_ui;

use sha2::{Digest, Sha256};
use wow_ui_sim::c_api::quest_objectives::{
    QuestObjective, QuestObjectiveProgress, commit_objective_progress,
};
use wow_ui_sim::event::EventArg;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::GossipQuestRow;

struct ConsumerEnv {
    env: WowLuaEnv,
    startup_errors: String,
}

fn error_snapshot(env: &WowLuaEnv) -> String {
    let sim = env.state().borrow();
    format!("{:?}|{:?}", sim.lua_errors, sim.lua_error_counts)
}

fn assert_errors(env: &ConsumerEnv) {
    let errors: String = env
        .env
        .eval(r"return table.concat(AcceptErrors, '\n')")
        .unwrap();
    assert!(errors.is_empty(), "original consumer errors: {errors}");
    assert_eq!(
        error_snapshot(&env.env),
        env.startup_errors,
        "new collected errors"
    );
}

fn exec(env: &ConsumerEnv, source: &str) {
    let result = env.env.exec(source);
    eprintln!("consumer {source:?}: {result:?}");
    assert_errors(env);
    result.expect("original consumer / behavioral assertion");
}

fn assert_originals() {
    let toc = wow_ui_sim::client_profile::blizzard_ui_framexml_toc().unwrap();
    for (file, expected) in [
        (
            "GossipFrame.lua",
            "ce4f5d9f2cd745a7ccc0ebefd4490dc8bffce20fff0098587dcc1fa65d421128",
        ),
        (
            "GossipFrame.xml",
            "978f95ccbae68943f1ac54a6c79d53926180346e514007d9fd61b15221dc9cbd",
        ),
        (
            "QuestFrame.lua",
            "4b2221da7f962535796ed500fbd1fc2adf95020e4564217ef457c1be7f9935f9",
        ),
        (
            "QuestFrame.xml",
            "1d5f50ab803551ad608abb22f90ae3959c37ff4e2fec80f78b05786b6bee0380",
        ),
        (
            "QuestInfo.lua",
            "3bfe691acdb2a095a6633c3ce7bef4a13f53f464d20781b2e850e442f9b715cd",
        ),
        (
            "QuestLogFrame.lua",
            "6ac6ca92b3036417082b88cc2f8b5c9fc8889f51f82aa726fe0bcd44990bc5e2",
        ),
        (
            "QuestLogFrame.xml",
            "530c396e36af4b23a725086d37b6cb68abaf1966db77ba26c4e3fed3ce76f154",
        ),
    ] {
        let path = toc.parent().unwrap().join(file);
        let hash = format!("{:x}", Sha256::digest(std::fs::read(&path).unwrap()));
        assert_eq!(hash, expected, "original source drift: {}", path.display());
    }
}

fn offer(id: u32, title: &str, level: i32) -> GossipQuestRow {
    GossipQuestRow {
        quest_id: id,
        title: title.into(),
        quest_level: level,
        ..Default::default()
    }
}

fn seed_env(existing: u32, offers: Vec<GossipQuestRow>) -> ConsumerEnv {
    assert_originals();
    let env = wrath_full_ui::load_original_wrath_startup();
    let startup_errors = error_snapshot(&env);
    eprintln!("retained full-startup errors: {startup_errors}");
    env.exec("AcceptErrors = {}; seterrorhandler(function(e) AcceptErrors[#AcceptErrors+1] = tostring(e) end)").unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.quest_log_entries.entries = (0..existing)
            .map(|i| {
                sim.quest_log_entries
                    .snapshot_gossip_offer(&offer(
                        991000 + i,
                        &format!("Wrath existing supply {}", i + 1),
                        20,
                    ))
                    .unwrap()
            })
            .collect();
        sim.quest_log = (0..existing).map(|i| 991000 + i).collect();
        sim.pending_quest_offer = None;
        sim.selected_quest_log_id = None;
        sim.abandon_quest_id = None;
        sim.quest_log_entries.completed_quest_ids.clear();
        sim.quest_log_entries.completed_quest_ids.insert(998888);
        sim.gossip.active = true;
        sim.gossip.giver_guid = 12340;
        sim.gossip.text = "Supplies for the northern road".into();
        sim.gossip.options.clear();
        sim.gossip.num_options = 0;
        sim.gossip.active_quests.clear();
        sim.gossip.num_active_quests = 0;
        sim.gossip.num_available_quests = offers.len() as i32;
        sim.gossip.available_quests = offers;
    }
    let env = ConsumerEnv {
        env,
        startup_errors,
    };
    // Only original UI policy switches; no quest API or callback replacements.
    exec(
        &env,
        "AUTO_QUEST_WATCH = '0'; AUTO_QUEST_PROGRESS = '0'; QUEST_FADING_DISABLE = '1'",
    );
    dispatch_queued(&env);
    env
}

// Deliver ONLY actual queued production notifications, unchanged and in FIFO order.
// Synchronous production dispatch is already recorded by the same observers.
fn dispatch_queued(env: &ConsumerEnv) {
    for _ in 0..16 {
        let events = env.env.state().borrow_mut().events.drain();
        if events.is_empty() {
            return;
        }
        for event in events {
            eprintln!("producer notification {} {:?}", event.name, event.args);
            let args = event
                .args
                .iter()
                .map(|arg| match arg {
                    EventArg::Number(value) => rilua::Val::Num(*value),
                    EventArg::Boolean(value) => rilua::Val::Bool(*value),
                    EventArg::String(value) => env.env.lua_string(value),
                    EventArg::Nil => rilua::Val::Nil,
                })
                .collect::<Vec<_>>();
            env.env.fire_event_with_args(&event.name, &args).unwrap();
            assert_errors(env);
        }
    }
    panic!("producer notification loop did not quiesce within 16 batches");
}

fn open_detail(env: &ConsumerEnv, event: &str, button: &str, title: &str) {
    // External NPC interaction stimulus, NOT a fabricated lifecycle success event.
    env.env.fire_event(event).unwrap();
    assert_errors(env);
    exec(
        env,
        &format!("assert({button}:IsShown(), 'original offer button shown'); {button}:Click()"),
    );
    dispatch_queued(env);
    exec(
        env,
        &format!(
            "assert(QuestInfoTitleHeader:GetText() == '{title}', 'original detail renders supplied pending title')"
        ),
    );
    exec(
        env,
        r#"
        assert(QuestFrame:IsShown() and QuestFrameDetailPanel:IsShown(), 'original detail panel shown')
        -- Original detail disables Accept until its original fading OnUpdate completes.
        QuestInfoFadingFrame:GetScript('OnUpdate')(QuestInfoFadingFrame, 1)
        assert(QuestFrameAcceptButton:IsShown() and QuestFrameAcceptButton:IsEnabled(), 'original accept button usable')
        assert(not QuestGetAutoAccept() and not QuestFrame.autoQuest and not QuestFlagsPVP(), 'ordinary non-PvP offer')
        SelectQuestLogEntry(1)
        -- Keep the real log listener visible alongside detail; execute its original OnShow.
        QuestLogFrame:Show()
    "#,
    );
    dispatch_queued(env);
}

fn observe_accept(env: &ConsumerEnv, target: u32, index: u32, title: &str, level: i32) {
    exec(
        env,
        &format!(
            r#"
        AcceptExpectedID = {target}; AcceptExpectedIndex = {index}
        AcceptExpectedTitle = '{title}'; AcceptExpectedLevel = {level}
        AcceptEvents = {{}}; AcceptLogUpdates = 0
        QuestLogQuestCount:SetText('stale-before-accept')
        if AcceptExpectedIndex <= #QuestLogScrollFrame.buttons then
            QuestLogScrollFrame.buttons[AcceptExpectedIndex]:SetText('stale-before-accept')
        end
        local observer = CreateFrame('Frame')
        for _, name in ipairs({{'QUEST_ACCEPTED', 'QUEST_LOG_UPDATE', 'UNIT_QUEST_LOG_CHANGED', 'QUEST_FINISHED', 'UI_ERROR_MESSAGE'}}) do observer:RegisterEvent(name) end
        observer:SetScript('OnEvent', function(_, event, ...)
            local n, q = GetNumQuestLogEntries()
            local row = {{GetQuestLogTitle(AcceptExpectedIndex)}}
            AcceptEvents[#AcceptEvents+1] = {{name=event, argc=select('#', ...), args={{...}}, count=n, quests=q, row=row}}
        end)
        QuestLogFrame:HookScript('OnEvent', function(_, event)
            if event == 'QUEST_LOG_UPDATE' then
                AcceptLogUpdates = AcceptLogUpdates + 1
                assert(QuestLogQuestCount:GetText() == string.format(QUEST_LOG_COUNT_TEMPLATE, {index}, 25), 'original log listener rendered new count')
                if AcceptExpectedIndex <= #QuestLogScrollFrame.buttons then
                    local button = QuestLogScrollFrame.buttons[AcceptExpectedIndex]
                    assert(string.find(button:GetText(), AcceptExpectedTitle, 1, true), 'original log listener rendered accepted title')
                end
            end
        end)
    "#
        ),
    );
}

fn preserved_state(env: &ConsumerEnv) -> String {
    let sim = env.env.state().borrow();
    format!(
        "{:?}|{:?}|{}|{}",
        sim.quest_log_entries.completed_quest_ids,
        sim.quest_log_entries
            .entries
            .iter()
            .filter(|row| row.quest_id >= 991000)
            .collect::<Vec<_>>(),
        sim.player.money,
        sim.player.xp
    )
}

fn assert_committed(env: &ConsumerEnv, existing: u32, id: u32, title: &str, level: i32) {
    let sim = env.env.state().borrow();
    let expected = (0..existing)
        .map(|i| 991000 + i)
        .chain([id])
        .collect::<Vec<_>>();
    assert_eq!(
        sim.quest_log, expected,
        "light membership committed exactly once"
    );
    assert_eq!(
        sim.quest_log_entries
            .entries
            .iter()
            .map(|row| row.quest_id as u32)
            .collect::<Vec<_>>(),
        expected
    );
    let row = sim.quest_log_entries.entries.last().unwrap();
    assert_eq!(
        (row.title.as_str(), row.level),
        (title, level),
        "pending metadata retained"
    );
    assert!(sim.pending_quest_offer.is_none(), "pending offer consumed");
}

fn assert_delivered(env: &ConsumerEnv) {
    // Search by name: queued/coalesced log refresh has no established total order.
    // QUEST_FINISHED is local dialog closure; its state snapshot need not be committed.
    exec(
        env,
        r#"
        local counts = {}
        for _, seen in ipairs(AcceptEvents) do
            counts[seen.name] = (counts[seen.name] or 0) + 1
            if seen.name == 'QUEST_ACCEPTED' then
                assert(seen.argc == 1 and seen.args[1] == AcceptExpectedIndex, 'one displayed log index; no quest ID')
            elseif seen.name == 'UNIT_QUEST_LOG_CHANGED' then
                assert(seen.argc == 1 and seen.args[1] == 'player', 'changed unit payload')
            elseif seen.name == 'QUEST_LOG_UPDATE' or seen.name == 'QUEST_FINISHED' then
                assert(seen.argc == 0, 'argumentless refresh or dialog closure')
            else
                error('unexpected acceptance event: '..seen.name)
            end
        end
        assert(counts.QUEST_ACCEPTED == 1, 'one acceptance for the new quest')
        assert(counts.UNIT_QUEST_LOG_CHANGED == 1, 'player log changed')
        assert((counts.QUEST_LOG_UPDATE or 0) >= 1, 'queued/coalesced refresh delivered')
        assert(counts.QUEST_FINISHED == 1, 'nonzero-giver fixture closes dialog')
        for _, seen in ipairs(AcceptEvents) do
            if seen.name ~= 'QUEST_FINISHED' then
                assert(seen.count == AcceptExpectedIndex and seen.quests == AcceptExpectedIndex, 'committed log visible to log notifications')
                local row = seen.row
                assert(row[1] == AcceptExpectedTitle and row[2] == AcceptExpectedLevel and row[9] == AcceptExpectedID, 'real Wrath title fields at dispatch')
                assert(not row[3] and not row[4] and not row[5] and not row[6] and not row[7] and not row[8] and not row[10], 'ordinary nonheader/incomplete/nondaily fields')
            end
        end
        assert(AcceptLogUpdates >= 1, 'original QuestLogFrame received queued refresh')
        assert(not QuestFrame:IsShown(), 'original finished listener closed QuestFrame')
    "#,
    );
}

fn exercise_accept(existing: u32, offers: Vec<GossipQuestRow>, event: &str, button: &str) {
    let selected = offers[1].clone();
    let env = seed_env(existing, offers);
    let before = preserved_state(&env);
    open_detail(&env, event, button, &selected.title);
    {
        let sim = env.env.state().borrow();
        let pending = sim
            .pending_quest_offer
            .as_ref()
            .expect("original selection created pending offer");
        assert_eq!(
            (pending.quest_id, pending.title.as_str(), pending.level),
            (
                selected.quest_id as i32,
                selected.title.as_str(),
                selected.quest_level
            )
        );
    }
    observe_accept(
        &env,
        selected.quest_id,
        existing + 1,
        &selected.title,
        selected.quest_level,
    );
    exec(&env, "QuestFrameAcceptButton:Click()");
    exec(
        &env,
        "for _, event in ipairs(AcceptEvents) do assert(event.name ~= 'QUEST_LOG_UPDATE', 'log update is queued, not inline') end",
    );
    dispatch_queued(&env);
    assert_committed(
        &env,
        existing,
        selected.quest_id,
        &selected.title,
        selected.quest_level,
    );
    assert_delivered(&env);
    assert_eq!(
        preserved_state(&env),
        before,
        "unrelated rows/history/economy preserved"
    );
    // Hidden widgets can suppress Click. Force the SAME loaded original callback to test repeat/no-pending.
    exec(
        &env,
        "AcceptEvents = {}; QuestFrameAcceptButton:GetScript('OnClick')(QuestFrameAcceptButton, 'LeftButton')",
    );
    dispatch_queued(&env);
    assert_committed(
        &env,
        existing,
        selected.quest_id,
        &selected.title,
        selected.quest_level,
    );
    exec(
        &env,
        "assert(#AcceptEvents == 0, 'repeat accept has no success notifications')",
    );
    assert_eq!(preserved_state(&env), before);
}

fn lifecycle_accept() -> ConsumerEnv {
    let env = seed_env(0, vec![offer(990301, "Wrath northern recovery", 23)]);
    open_detail(
        &env,
        "GOSSIP_SHOW",
        "GossipTitleButton1",
        "Wrath northern recovery",
    );
    exec(&env, "QuestFrameAcceptButton:Click()");
    dispatch_queued(&env);
    assert_committed(&env, 0, 990301, "Wrath northern recovery", 23);
    env
}

#[test]
fn original_combined_lifecycle_accept_progress_turn_in() {
    let env = lifecycle_accept();
    for (text, finished, count) in [
        ("Wolves slain: 1/2", false, 1),
        ("Wolves slain: 2/2", true, 2),
    ] {
        let mut sim = env.env.state().borrow_mut();
        assert!(commit_objective_progress(
            &mut sim.quest_log_entries,
            990301,
            QuestObjectiveProgress {
                objectives: vec![QuestObjective {
                    text: text.into(),
                    objective_type: "monster".into(),
                    finished,
                    num_fulfilled: count,
                    num_required: 2,
                    criteria_id: None
                }],
                ..Default::default()
            }
        ));
        drop(sim);
        exec(
            &env,
            &format!(
                "SelectQuestLogEntry(1); QuestLogFrame:Show(); QuestLog_UpdateQuestDetails(false); assert(QuestInfoObjective1:GetText() == '{}', 'original objective progress')",
                if finished {
                    format!("{} (' .. COMPLETE .. ')", text)
                } else {
                    text.into()
                }
            ),
        );
    }
    {
        let mut sim = env.env.state().borrow_mut();
        sim.quest_log_entries.readiness.active_player_present = true;
        sim.quest_log_entries
            .readiness
            .completion_callback_cache_hit = true;
        let mut active = offer(990301, "Wrath northern recovery", 23);
        active.is_complete = Some(false);
        sim.gossip.active = true;
        sim.gossip.giver_guid = 12340;
        sim.gossip.available_quests.clear();
        sim.gossip.num_available_quests = 0;
        sim.gossip.active_quests = vec![active];
        sim.gossip.num_active_quests = 1;
    }
    exec(&env, "HideUIPanel(QuestLogFrame)");
    env.env.fire_event("QUEST_GREETING").unwrap();
    exec(&env, "QuestTitleButton1:Click()");
    dispatch_queued(&env);
    {
        let mut sim = env.env.state().borrow_mut();
        sim.quest_log_entries.entries[0].is_complete = true;
        sim.gossip.active_quests[0].is_complete = Some(true);
    }
    exec(
        &env,
        "assert(QuestFrameProgressPanel:IsShown(), 'progress panel'); QuestFrameProgressPanel:Hide(); QuestFrameProgressPanel:Show(); assert(QuestFrameCompleteButton:IsEnabled(), 'complete enabled'); QuestFrameCompleteButton:Click(); assert(A_Admin.OfferQuestReward(), 'server offer')",
    );
    dispatch_queued(&env);
    exec(
        &env,
        "assert(QuestFrameRewardPanel:IsShown()); QuestFrameCompleteQuestButton:Click(); assert(A_Admin.TurnInSelectedQuest())",
    );
    dispatch_queued(&env);
    let sim = env.env.state().borrow();
    assert!(!sim.quest_log.contains(&990301));
    assert!(sim.quest_log_entries.completed_quest_ids.contains(&990301));
}

#[test]
fn original_combined_lifecycle_accept_then_abandon() {
    let env = lifecycle_accept();
    exec(
        &env,
        "ShowUIPanel(QuestLogFrame); QuestLog_SetSelection(1); assert(QuestLogFrameAbandonButton:IsVisible(), 'abandon visible'); QuestLogFrameAbandonButton:Click(); local popup = StaticPopup_FindVisible('ABANDON_QUEST'); assert(popup and popup:IsShown(), 'abandon popup'); popup.button1:Click()",
    );
    dispatch_queued(&env);
    let sim = env.env.state().borrow();
    assert!(!sim.quest_log.contains(&990301));
    assert!(!sim.quest_log_entries.completed_quest_ids.contains(&990301));
}

#[test]
fn original_gossip_second_offer_accepts_at_log_index_two() {
    exercise_accept(
        1,
        vec![
            offer(990101, "Wrath northern dispatch", 23),
            offer(990202, "Wrath eastern supplies", 27),
        ],
        "GOSSIP_SHOW",
        "GossipTitleButton2",
    );
}

#[test]
fn original_greeting_reversed_second_offer_accepts_at_log_index_three() {
    exercise_accept(
        2,
        vec![
            offer(990202, "Wrath eastern supplies", 27),
            offer(990101, "Wrath northern dispatch", 23),
        ],
        "QUEST_GREETING",
        "QuestTitleButton2",
    );
}

#[test]
fn original_accept_callback_without_pending_offer_is_silent() {
    let env = seed_env(1, vec![]);
    let before = preserved_state(&env);
    let membership = env.env.state().borrow().quest_log.clone();
    let rich_before = format!("{:?}", env.env.state().borrow().quest_log_entries);
    exec(
        &env,
        r#"
        AcceptNoOfferEvents = {}
        local observer = CreateFrame('Frame')
        for _, name in ipairs({'QUEST_ACCEPTED', 'QUEST_LOG_UPDATE', 'UNIT_QUEST_LOG_CHANGED', 'QUEST_FINISHED', 'UI_ERROR_MESSAGE'}) do observer:RegisterEvent(name) end
        observer:SetScript('OnEvent', function(_, event) AcceptNoOfferEvents[#AcceptNoOfferEvents+1] = event end)
        QuestFrameAcceptButton:GetScript('OnClick')(QuestFrameAcceptButton, 'LeftButton')
    "#,
    );
    dispatch_queued(&env);
    assert_eq!(env.env.state().borrow().quest_log, membership);
    assert_eq!(
        format!("{:?}", env.env.state().borrow().quest_log_entries),
        rich_before
    );
    assert_eq!(preserved_state(&env), before);
    assert!(env.env.state().borrow().pending_quest_offer.is_none());
    exec(
        &env,
        "assert(#AcceptNoOfferEvents == 0, 'no pending offer is not success')",
    );
}

#[test]
fn ordinary_accept_allows_twenty_fifth_but_rejects_twenty_sixth() {
    for existing in [24, 25] {
        let env = seed_env(existing, vec![offer(990101, "Wrath northern dispatch", 23)]);
        let before = preserved_state(&env);
        open_detail(
            &env,
            "GOSSIP_SHOW",
            "GossipTitleButton1",
            "Wrath northern dispatch",
        );
        observe_accept(&env, 990101, existing + 1, "Wrath northern dispatch", 23);
        exec(&env, "QuestFrameAcceptButton:Click()");
        exec(
            &env,
            "for _, event in ipairs(AcceptEvents) do assert(event.name ~= 'QUEST_LOG_UPDATE', 'log update is queued, not inline') end",
        );
        dispatch_queued(&env);
        if existing == 24 {
            assert_committed(&env, existing, 990101, "Wrath northern dispatch", 23);
            assert_delivered(&env);
        } else {
            let sim = env.env.state().borrow();
            assert_eq!(
                sim.quest_log,
                (0..25).map(|i| 991000 + i).collect::<Vec<_>>()
            );
            assert_eq!(sim.quest_log_entries.entries.len(), 25);
            drop(sim);
            exec(
                &env,
                r#"
                local entries, quests = GetNumQuestLogEntries()
                assert(entries == 25 and quests == 25, 'full log unchanged')
                local errors, finished = 0, 0
                assert(ERR_QUEST_LOG_FULL == 'Your quest log is full.', 'original Wrath GlobalStrings.lua:3425')
                for _, event in ipairs(AcceptEvents) do
                    if event.name == 'UI_ERROR_MESSAGE' then
                        errors = errors + 1
                        assert(event.argc == 1 and type(event.args[1]) == 'string', 'one localized error string; no error code')
                        assert(event.args[1] == ERR_QUEST_LOG_FULL, 'server-full stand-in uses original localized text')
                    elseif event.name == 'QUEST_FINISHED' then
                        finished = finished + 1
                        assert(event.argc == 0, 'argumentless local dialog closure')
                    else
                        error('full log must not emit acceptance/log/unit events: '..event.name)
                    end
                end
                assert(errors == 1 and finished == 1, 'server rejection plus nonzero-giver local closure')
                assert(not QuestFrame:IsShown(), 'full log still closes original dialog')
            "#,
            );
        }
        assert_eq!(
            preserved_state(&env),
            before,
            "capacity boundary preserves history/unrelated rows/economy"
        );
    }
}
