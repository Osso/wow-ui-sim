//! Staged full-original consumer fixture; no quest API/callback replacement.
#[path = "wrath_full_ui.rs"]
mod wrath_full_ui;

use sha2::{Digest, Sha256};
use wow_ui_sim::event::EventArg;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::GossipQuestRow;
use wow_ui_sim::lua_api::state::QuestRewardItem;

pub const NPC_GUID: u64 = 0xF130_0030_0000_0001;
pub const TARGET: i32 = 990301;
pub const CONTROL: i32 = 991300;
pub const TITLE: &str = "Wrath northern recovery";

pub struct ConsumerEnv {
    pub env: WowLuaEnv,
    startup_errors: String,
}

fn error_snapshot(env: &WowLuaEnv) -> String {
    let sim = env.state().borrow();
    format!("{:?}|{:?}", sim.lua_errors, sim.lua_error_counts)
}

pub fn assert_errors(env: &ConsumerEnv) {
    let errors: String = env
        .env
        .eval(r"return table.concat(TurnInErrors, '\n')")
        .unwrap();
    assert!(errors.is_empty(), "original consumer errors: {errors}");
    let forbidden: i64 = env.env.eval("return TurnInForbiddenEvents or 0").unwrap();
    assert_eq!(forbidden, 0, "QUEST_TURNED_IN observed during this cycle");
    assert_eq!(
        error_snapshot(&env.env),
        env.startup_errors,
        "new collected errors"
    );
}

pub fn exec(env: &ConsumerEnv, source: &str) {
    let result = env.env.exec(source);
    eprintln!("consumer {source:?}: {result:?}");
    assert_errors(env);
    result.expect("original consumer / behavioral assertion");
}

fn assert_originals() {
    let toc = wow_ui_sim::client_profile::blizzard_ui_framexml_toc().unwrap();
    let expected_root = std::path::PathBuf::from(std::env::var_os("HOME").unwrap())
        .join(".cache/wow-ui-sim/blizzard-ui/wrath/FrameXML");
    assert_eq!(toc.parent().unwrap(), expected_root);
    for (file, expected) in [
        (
            "UIErrorsFrame.lua",
            "d4d1865b12b06b2abeab1b1e3ec588dbfa8930185801ecc6e7cc9f7336ab0e5b",
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
            "QuestInfo.xml",
            "290ed9e781c98734b69829184b171b99348c8d34ff3972292a01b1db7736bccc",
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

fn active_row(id: i32, title: &str, complete: bool) -> GossipQuestRow {
    GossipQuestRow {
        quest_id: id as u32,
        title: title.into(),
        quest_level: 27,
        is_complete: Some(complete),
        ..Default::default()
    }
}

fn seed_existing_quests(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    let control = active_row(CONTROL, "Wrath unrelated patrol", true);
    let target = active_row(TARGET, TITLE, false);
    let rows = [control.clone(), target.clone()]
        .iter()
        .map(|row| sim.quest_log_entries.snapshot_gossip_offer(row).unwrap())
        .collect();
    sim.quest_log_entries.entries = rows;
    sim.quest_log_entries.objective_progress.clear();
    sim.quest_log_entries
        .objective_progress
        .insert(TARGET, Default::default());
    sim.quest_log_entries.background_materials.clear();
    sim.quest_log_entries
        .background_materials
        .insert(TARGET, "Parchment".into());
    sim.quest_log_entries.readiness = Default::default();
    sim.quest_log_entries.readiness.active_player_present = true;
    sim.gossip.giver_guid = NPC_GUID;
    sim.quest_log_entries.readiness.next_quest_id = 0;
    // Supplied simulator response: acceptance completes a cache-hit callback inline.
    // This is not a guarantee about the native server/cache schedule.
    sim.quest_log_entries
        .readiness
        .completion_callback_cache_hit = true;
    sim.quest_log_entries.completed_quest_ids.clear();
    sim.quest_log_entries.completed_quest_ids.insert(998888);
    sim.quest_log = vec![CONTROL as u32, TARGET as u32];
    sim.selected_quest_log_id = None;
    sim.pending_quest_offer = None;
    sim.abandon_quest_id = None;
    sim.gossip.active = true;
    sim.gossip.text = "Recovered supplies for the northern road".into();
    sim.gossip.options.clear();
    sim.gossip.num_options = 0;
    sim.gossip.available_quests.clear();
    sim.gossip.num_available_quests = 0;
    sim.gossip.active_quests = vec![control, target];
    sim.gossip.num_active_quests = 2;
}

pub fn seed_env() -> ConsumerEnv {
    assert_originals();
    let env = wrath_full_ui::load_original_wrath_startup();
    let startup_errors = error_snapshot(&env);
    eprintln!("retained full-startup errors: {startup_errors}");
    env.exec("TurnInErrors = {}; seterrorhandler(function(e) TurnInErrors[#TurnInErrors+1] = tostring(e) end)").unwrap();
    seed_existing_quests(&env);
    let consumer = ConsumerEnv {
        env,
        startup_errors,
    };
    exec(
        &consumer,
        "AUTO_QUEST_WATCH = '0'; AUTO_QUEST_PROGRESS = '0'; QUEST_FADING_DISABLE = '1'",
    );
    dispatch_queued(&consumer);
    consumer
}

// FIFO delivery of actual queued producer events only; no missing event/argument repair.
pub fn dispatch_queued(env: &ConsumerEnv) {
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
            let result = env.env.fire_event_with_args(&event.name, &args);
            assert_errors(env);
            result.unwrap();
        }
    }
    panic!("producer notifications did not quiesce within 16 batches");
}

pub fn observe(env: &ConsumerEnv) {
    exec(
        env,
        r#"
        TurnInEvents = {}; TurnInLogUpdates = 0
        TurnInForbiddenEvents = 0; TurnInPlayerUnits = 0
        local observer = CreateFrame('Frame')
        observer:RegisterAllEvents()
        observer:SetScript('OnEvent', function(_, event, ...)
            TurnInEvents[#TurnInEvents+1] = {name=event, argc=select('#', ...), args={...}}
            if event == 'QUEST_TURNED_IN' then
                TurnInForbiddenEvents = TurnInForbiddenEvents + 1
            elseif event == 'UNIT_QUEST_LOG_CHANGED' and select(1, ...) == 'player' then
                TurnInPlayerUnits = TurnInPlayerUnits + 1
            end
        end)
        QuestLogFrame:HookScript('OnEvent', function(_, event)
            if event == 'QUEST_LOG_UPDATE' then
                TurnInLogUpdates = TurnInLogUpdates + 1
                local _, count = GetNumQuestLogEntries()
                assert(QuestLogQuestCount:GetText() == string.format(QUEST_LOG_COUNT_TEMPLATE, count, 25), 'original log count rendered committed membership')
                if count == 1 then
                    assert(string.find(QuestLogScrollFrame.buttons[1]:GetText(), 'Wrath unrelated patrol', 1, true), 'original log renders surviving control')
                end
            end
        end)
        function TurnInCount(name)
            local count = 0
            for _, seen in ipairs(TurnInEvents) do
                if seen.name == name then count = count + 1 end
            end
            return count
        end
    "#,
    );
}

pub fn open_progress(env: &ConsumerEnv) {
    observe(env);
    env.env.fire_event("QUEST_GREETING").unwrap();
    assert_errors(env);
    exec(
        env,
        r#"
        assert(QuestTitleButton2:IsShown(), 'second active title displayed')
        assert(string.find(QuestTitleButton2:GetText(), 'Wrath northern recovery', 1, true), 'supplied active title')
        QuestTitleButton2:Click()
    "#,
    );
    dispatch_queued(env);
    exec(
        env,
        r#"
        assert(QuestFrame:IsShown() and QuestFrameProgressPanel:IsShown(), 'original progress panel')
        assert(QuestProgressTitleText:GetText() == 'Wrath northern recovery', 'dialog title')
        assert(QuestFrameCompleteButton:IsEnabled() == false or QuestFrameCompleteButton:IsEnabled() == 0, 'original incomplete UI disables button')
        assert(not QuestFrameRewardPanel:IsShown(), 'not yet rewards')
        assert(GetNumQuestItems() == 0 and GetQuestMoneyToGet() == 0, 'supplied no-items no-cost fixture')
    "#,
    );
    // Original QuestLog_Update returns while hidden. Keep the observed renderer
    // visible before later response notifications; never repair its output.
    exec(env, "QuestLogFrame:Show()");
}

pub fn supply_status(env: &ConsumerEnv, complete: bool, failed: bool) {
    let mut sim = env.env.state().borrow_mut();
    let row = sim
        .quest_log_entries
        .entries
        .iter_mut()
        .find(|row| row.quest_id == TARGET)
        .unwrap();
    row.is_complete = complete;
    row.is_failed = failed;
    sim.gossip
        .active_quests
        .iter_mut()
        .find(|row| row.quest_id == TARGET as u32)
        .unwrap()
        .is_complete = Some(complete && !failed);
}

pub fn force_complete_callback(env: &ConsumerEnv) {
    // Loaded XML callback even when disabled/hidden; no widget-suppression false proof.
    exec(
        env,
        "QuestFrameCompleteButton:GetScript('OnClick')(QuestFrameCompleteButton)",
    );
}

pub fn force_reward_callback(env: &ConsumerEnv) {
    exec(
        env,
        "QuestFrameCompleteQuestButton:GetScript('OnClick')(QuestFrameCompleteQuestButton)",
    );
}

pub fn offer_response(env: &ConsumerEnv) {
    // Proposed simulator server-offer stimulus, not a native API or callback shim.
    exec(
        env,
        "assert(A_Admin.OfferQuestReward(), 'simulator offer accepts pending presentation request')",
    );
    dispatch_queued(env);
    exec(
        env,
        r#"
        assert(QuestFrameRewardPanel:IsShown() and not QuestFrameProgressPanel:IsShown(), 'original reward panel consumes server offer')
        assert(QuestInfoTitleHeader:GetText() == 'Wrath northern recovery', 'reward uses dialog quest')
        assert(QuestFrameCompleteQuestButton:IsEnabled(), 'original claim usable')
        assert(TurnInCount('QUEST_COMPLETE') == 1, 'one server reward presentation')
    "#,
    );
}

pub fn open_rewards(env: &ConsumerEnv) {
    open_progress(env);
    supply_status(env, true, false);
    exec(
        env,
        r#"
        QuestLogFrame:Show()
        SelectQuestLogEntry(2)
        QuestFrameProgressPanel:Hide(); QuestFrameProgressPanel:Show()
        assert(QuestFrameCompleteButton:IsEnabled(), 'supplied readiness enables original UI')
    "#,
    );
    dispatch_queued(env);
    clear_trace(env);
    exec(env, "QuestFrameCompleteButton:Click()");
    assert_request_silent(env);
    offer_response(env);
    clear_trace(env);
}

pub fn clear_trace(env: &ConsumerEnv) {
    // Check the cumulative forbidden-event latch BEFORE discarding phase-local events.
    assert_errors(env);
    exec(env, "TurnInEvents = {}; TurnInLogUpdates = 0");
}

pub fn assert_active(env: &ConsumerEnv) {
    exec(
        env,
        r#"
        local n, count = GetNumQuestLogEntries()
        assert(n == 2 and count == 2, 'both quests remain active')
        assert(select(9, GetQuestLogTitle(1)) == 991300 and select(9, GetQuestLogTitle(2)) == 990301, 'concrete active IDs')
        assert(not IsQuestFlaggedCompleted(990301) and IsQuestFlaggedCompleted(998888), 'history unchanged')
    "#,
    );
}

pub fn assert_request_silent(env: &ConsumerEnv) {
    assert_active(env);
    exec(
        env,
        "assert(#TurnInEvents == 0, 'native request emits no lifecycle events')",
    );
    dispatch_queued(env);
    exec(
        env,
        "assert(#TurnInEvents == 0, 'native request queues no lifecycle events')",
    );
}

pub fn assert_no_server_request(env: &ConsumerEnv, progress: bool) {
    let call = if progress {
        "A_Admin.OfferQuestReward()"
    } else {
        "A_Admin.TurnInSelectedQuest()"
    };
    exec(
        env,
        &format!("assert(not {call}, 'simulator rejects response without native request')"),
    );
    assert_request_silent(env);
}

pub fn claim(env: &ConsumerEnv) {
    exec(env, "QuestFrameCompleteQuestButton:Click()");
    assert_request_silent(env);
}

// Step A: acceptance only. Callers inspect observable membership/history/receipts.
pub fn accept_response(env: &ConsumerEnv) {
    exec(
        env,
        "assert(A_Admin.TurnInSelectedQuest(), 'SIMULATOR POLICY: accept and complete supplied cache-hit callback')",
    );
}

// Step B: snapshot EACH acceptance, never reuse an earlier player's emission.
pub fn accept_response_with_immediate_unit(env: &ConsumerEnv) {
    exec(env, "TurnInPlayerUnitsBeforeAcceptance = TurnInPlayerUnits");
    accept_response(env);
    exec(
        env,
        "assert(TurnInPlayerUnits > TurnInPlayerUnitsBeforeAcceptance, 'this acceptance immediately emits player UNIT')",
    );
}

pub fn assert_log_update_deferred(env: &ConsumerEnv) {
    exec(
        env,
        "assert(TurnInCount('QUEST_LOG_UPDATE') == 0, 'LOG_UPDATE deferred until flush')",
    );
}

pub fn assert_finish_count(env: &ConsumerEnv, finishes: usize) {
    exec(
        env,
        &format!(
            r#"
        assert(TurnInCount('QUEST_FINISHED') == {finishes}, 'conditional finish count')
        for _, seen in ipairs(TurnInEvents) do
            if seen.name == 'QUEST_FINISHED' then
                assert(seen.argc == 0, 'argumentless finish')
            end
        end
    "#
        ),
    );
}

pub fn assert_trace(env: &ConsumerEnv, finishes: usize, player_units: usize) {
    exec(
        env,
        &format!(
            r#"
        local firstUnit, firstLog, playerUnits = nil, nil, 0
        for i, seen in ipairs(TurnInEvents) do
            assert(seen.name ~= 'QUEST_TURNED_IN', 'absent in build 12340')
            if seen.name == 'UNIT_QUEST_LOG_CHANGED' then
                assert(seen.argc == 1, 'one unit token per emission')
                if seen.args[1] == 'player' then
                    playerUnits = playerUnits + 1
                    firstUnit = firstUnit or i
                end
            elseif seen.name == 'QUEST_LOG_UPDATE' then
                assert(seen.argc == 0, 'argumentless deferred signal')
                firstLog = firstLog or i
            elseif seen.name == 'QUEST_FINISHED' then
                assert(seen.argc == 0, 'argumentless finish')
            end
        end
        assert(playerUnits >= {player_units}, 'required immediate player emissions; aliases allowed')
        assert(firstUnit and firstLog and firstUnit < firstLog, 'only UNIT before LOG_UPDATE flush')
        assert(TurnInCount('QUEST_LOG_UPDATE') == 1, 'coalesced per flush')
        assert(TurnInCount('QUEST_FINISHED') == {finishes}, 'conditional finish count')
        assert(TurnInLogUpdates == 1, 'original log listener refreshed once')
    "#
        ),
    );
}

pub fn assert_target_accepted(env: &ConsumerEnv) {
    exec(
        env,
        r#"
        local n, count = GetNumQuestLogEntries()
        assert(n == 1 and count == 1 and select(9, GetQuestLogTitle(1)) == 991300, 'dialog target removed, complete control preserved')
        assert(IsQuestFlaggedCompleted(990301) and IsQuestFlaggedCompleted(998888), 'simulator completion history')
        assert(not IsQuestFlaggedCompleted(991300), 'selection did not redirect acceptance')
    "#,
    );
}

pub fn assert_blocked_after_close(env: &ConsumerEnv) {
    exec(env, "CloseQuest()");
    dispatch_queued(env);
    exec(
        env,
        "assert(not QuestFrame:IsShown(), 'original finish listener hides frame')",
    );
    clear_trace(env);
    force_complete_callback(env);
    force_reward_callback(env);
    exec(
        env,
        r#"
        assert(select('#', CompleteQuest()) == 0)
        assert(select('#', GetQuestReward(999)) == 0, 'answered guard precedes invalid-choice validation')
        assert(not A_Admin.OfferQuestReward() and not A_Admin.TurnInSelectedQuest(), 'no new request after answer and close')
    "#,
    );
    dispatch_queued(env);
    exec(
        env,
        "assert(#TurnInEvents == 0, 'answered repeats silent after close')",
    );
}

pub fn supply_choices(env: &ConsumerEnv, reversed: bool) {
    let satchel = QuestRewardItem {
        name: "Northern recovery satchel".into(),
        texture: "Interface\\Icons\\INV_Misc_Bag_08".into(),
        count: 1,
        quality: 3,
        is_usable: true,
        item_id: None,
    };
    let seal = QuestRewardItem {
        name: "Patrol recovery seal".into(),
        texture: "Interface\\Icons\\INV_Jewelry_Talisman_07".into(),
        count: 3,
        quality: 2,
        is_usable: false,
        item_id: None,
    };
    let mut sim = env.env.state().borrow_mut();
    sim.quest_log_entries
        .entries
        .iter_mut()
        .find(|row| row.quest_id == TARGET)
        .unwrap()
        .reward_choices = if reversed {
        vec![seal, satchel]
    } else {
        vec![satchel, seal]
    };
    sim.quest_log_entries.turn_in_choices.insert(998888, 1);
}

pub fn assert_receipts(env: &ConsumerEnv, choice: Option<usize>) {
    let mut expected = std::collections::HashMap::from([(998888, 1)]);
    if let Some(choice) = choice {
        expected.insert(TARGET, choice);
    }
    assert_eq!(
        env.env.state().borrow().quest_log_entries.turn_in_choices,
        expected,
        "SIMULATOR POLICY: accepted choice receipt, not inventory grant"
    );
}

// Every caller creates a fresh dialog: a successful request consumes eligibility.
pub fn assert_bad_choice(argument: &str) {
    let env = seed_env();
    supply_choices(&env, false);
    open_rewards(&env);
    exec(
        &env,
        &format!(
            r#"
        assert(GetNumQuestChoices() == 2, 'two supplied choices')
        local ok, err = pcall(function() GetQuestReward({argument}) end)
        local body = 'Invalid reward choice in GetQuestReward([choice])'
        assert(not ok, 'invalid eligible choice must raise')
        assert(type(err) == 'string' and string.sub(err, -string.len(body)) == body, tostring(err))
    "#
        ),
    );
    assert_request_silent(&env);
    assert_no_server_request(&env, false);
    assert_receipts(&env, None);
    exec(
        &env,
        "assert(QuestFrameRewardPanel:IsShown()); QuestInfoItem2:Click()",
    );
    claim(&env);
    accept_response(&env);
    dispatch_queued(&env);
    assert_receipts(&env, Some(2));
}

pub fn assert_valid_choice(argument: &str, expected_choice: usize) {
    let env = seed_env();
    supply_choices(&env, false);
    open_rewards(&env);
    exec(&env, &format!("GetQuestReward({argument})"));
    assert_request_silent(&env);
    assert_receipts(&env, None);
    // Exactly one eligible native call; answered-guard precedence is tested separately.
    accept_response(&env);
    dispatch_queued(&env);
    assert_target_accepted(&env);
    // Receipt indices are one-based policy labels, not native wire indices.
    assert_receipts(&env, Some(expected_choice));
}
