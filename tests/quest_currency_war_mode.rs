//! Real current-profile QuestUtils currency rewards and their independent inputs.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{CurrencyInfo, QuestRewardCurrency};

const QUEST_ID: i32 = 901001;
const CURRENCY_ID: i32 = 90001;
const REWARD_ICON: &str = "Interface\\Icons\\INV_Misc_QuestionMark";

fn probe(env: &WowLuaEnv, body: &str) {
    env.exec(body)
        .expect("public War Mode reward input behavior");
}

fn currency_reward_env() -> WowLuaEnv {
    let env = load_currency_reward_consumer();
    supply_currency_reward(&env);
    env
}

fn load_currency_reward_consumer() -> WowLuaEnv {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .expect("current profile Blizzard cache");
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_FrameXMLUtil"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_FrameXMLUtil"));
    let source: String = env
        .eval("return debug.getinfo(QuestUtils_AddQuestCurrencyRewardsToTooltip, 'S').source")
        .expect("loaded public QuestUtils closure provenance");
    eprintln!("Loaded public currency reward consumer: {source}");
    env
}

fn supply_currency_reward(env: &WowLuaEnv) {
    let mut quest = env.state().borrow().quest_log_entries.entries[0].clone();
    quest.quest_id = QUEST_ID;
    quest.title = "War Mode currency fixture".to_string();
    quest.is_world_quest = true;
    quest.currency_rewards = vec![build_currency_reward(CURRENCY_ID, 7, "Fixture Marks")];
    let mut sim = env.state().borrow_mut();
    sim.quest_log_entries.entries.push(quest);
    sim.currency_info.insert(
        CURRENCY_ID,
        CurrencyInfo {
            name: "Fixture Marks".to_string(),
            quality: 2,
            ..CurrencyInfo::default()
        },
    );
}

fn build_currency_reward(currency_id: i32, amount: i32, name: &str) -> QuestRewardCurrency {
    QuestRewardCurrency {
        currency_id,
        name: name.to_string(),
        texture: REWARD_ICON.to_string(),
        total_reward_amount: amount,
        base_reward_amount: None,
    }
}

#[test]
fn quest_currency_container_ranges_reach_original_reward_consumer() {
    let env = currency_reward_env();
    env.exec("EmbeddedItemTooltip_SetCurrencyByID = function() return true end").unwrap();
    for (currency, quantity, expected) in [
        (1553, 1, true), (1553, 25, true), (1553, 49, true),
        (1553, 50, true), (1553, 0, false), (1822, 0, true),
        (2003, 25, true), (2003, i32::MAX, true), (2003, 24, false),
        (999999, 25, false),
    ] {
        {
            let mut sim = env.state().borrow_mut();
            let quest = sim.quest_log_entries.entries.iter_mut().find(|q| q.quest_id == QUEST_ID).unwrap();
            quest.currency_rewards = vec![build_currency_reward(currency, quantity, "Fixture Marks")];
            sim.currency_info.insert(currency, CurrencyInfo { name: "Fixture Marks".into(), quality: 2, ..CurrencyInfo::default() });
        }
        let observed: bool = env.eval(r#"
            local _, container = QuestUtils_AddQuestCurrencyRewardsToTooltip(
                901001, GameTooltip, { Tooltip = GameTooltip }, nil)
            return container
        "#).unwrap();
        assert_eq!(observed, expected, "currency={currency} quantity={quantity}");
    }
}

fn reward_lines(env: &WowLuaEnv) -> Vec<String> {
    render_reward_lines(env, 1)
}

fn render_reward_lines(env: &WowLuaEnv, expected_rewards: i32) -> Vec<String> {
    probe(
        env,
        &format!(
            r#"
        GameTooltip:ClearLines()
        local added, container, info = QuestUtils_AddQuestCurrencyRewardsToTooltip(
            901001, GameTooltip, nil, nil)
        assert(added == {expected_rewards} and container == false and info == nil,
            'actual ordinary currency reward result')
        "#
        ),
    );
    let sim = env.state().borrow();
    let id = sim.widgets.get_id_by_name("GameTooltip").unwrap();
    sim.tooltips
        .get(&id)
        .unwrap()
        .lines
        .iter()
        .map(|line| line.left_text.clone())
        .collect()
}

#[test]
fn quest_currency_war_mode_desired_has_one_boolean_result() {
    let env = WowLuaEnv::new().unwrap();
    probe(
        &env,
        r#"
        local function desired(...)
            assert(select('#', ...) == 1, 'desired query returns one value')
            local value = ...
            assert(type(value) == 'boolean', 'War Mode desired is a real boolean')
            assert(value == false, 'normal modeled mode initially disabled')
        end
        desired(C_PvP.IsWarModeDesired())
        assert(C_PvP.GetWarModeRewardBonus() == 10, 'preserve existing simulator percentage')
        assert(C_PvP.GetWarModeRewardBonusDefault() == 10, 'separate default query unchanged')
    "#,
    );
}

#[test]
fn quest_currency_war_mode_eligibility_requires_numeric_identity() {
    let env = WowLuaEnv::new().unwrap();
    probe(
        &env,
        r#"
        local get = C_QuestLog.QuestCanHaveWarModeBonus
        for _, value in ipairs({'901001', false, {}, function() end}) do
            local ok, message = pcall(get, value)
            assert(not ok and type(message) == 'string' and string.find(message, 'number', 1, true),
                'quest eligibility must reject nonnumeric identity')
        end
        assert(not pcall(get), 'quest identity required')
        assert(not pcall(get, nil), 'nil quest identity rejected')
        assert(get(901001) == false, 'absent eligibility is modeled false, not native catalog truth')
    "#,
    );
}

#[test]
fn quest_currency_war_mode_faction_query_requires_numeric_identity() {
    let env = WowLuaEnv::new().unwrap();
    probe(
        &env,
        r#"
        local get = C_CurrencyInfo.GetFactionGrantedByCurrency
        for _, value in ipairs({'90001', false, {}, function() end}) do
            local ok, message = pcall(get, value)
            assert(not ok and type(message) == 'string' and string.find(message, 'number', 1, true),
                'faction query must reject nonnumeric identity')
        end
        assert(not pcall(get), 'currency identity required')
        assert(not pcall(get, nil), 'nil currency identity rejected')
        local function absent(...)
            assert(select('#', ...) == 1, 'faction query returns one value')
            assert((...) == nil, 'no supplied faction relation')
        end
        absent(get(90001))
    "#,
    );
}

#[test]
#[cfg(feature = "retail-12-1-0")]
fn quest_currency_war_mode_relation_queries_authenticate_tainted_secrets_first() {
    let env = WowLuaEnv::new().unwrap();
    probe(
        &env,
        r#"
        debug.settaintmode(true)
        local valid, invalid = secretwrap(901001), secretwrap('invalid')
        local getters = {C_QuestLog.QuestCanHaveWarModeBonus, C_CurrencyInfo.GetFactionGrantedByCurrency}
        local function addon()
            for _, get in ipairs(getters) do
                for _, value in ipairs({valid, invalid}) do
                    local ok, message = pcall(get, value)
                    assert(not ok and type(message) == 'string', 'tainted relation secret denied')
                    assert(string.find(message, 'untainted caller', 1, true),
                        'authentication precedes type and lookup')
                    assert(issecretvalue(value), 'wrapper preserved after denial')
                    assert(not pcall(secretunwrap, value), 'no declassification')
                    assert(debug.getstacktaint() == 'QuestCurrencyWarModeProbe', 'caller taint preserved')
                end
            end
        end
        debug.setobjecttaint(addon, 'QuestCurrencyWarModeProbe')
        addon()
        assert(debug.getstacktaint() == nil, 'caller restored')
    "#,
    );
}

#[test]
fn quest_currency_war_mode_actual_vendor_ordinary_reward_baseline() {
    let env = currency_reward_env();
    assert_eq!(
        reward_lines(&env),
        vec![format!("|T{REWARD_ICON}:16:16|t 7 Fixture Marks")]
    );
    assert!(env.state().borrow().lua_errors.is_empty());
}

fn enable_reward_inputs(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    sim.war_mode.desired = true;
    sim.war_mode.reward_bonus = 25.0;
    sim.quest_war_mode_eligibility.insert(QUEST_ID);
    sim.currency_war_mode_bonus
        .insert(CURRENCY_ID, (Some(true), None));
}

fn assert_reward_bonus(env: &WowLuaEnv, applies: bool) {
    let reward = format!("|T{REWARD_ICON}:16:16|t 7 Fixture Marks");
    let expected = if applies {
        vec![reward, "|cff1eff00+25% War Mode Bonus|r".to_string()]
    } else {
        vec![reward]
    };
    assert_eq!(reward_lines(env), expected);
}

fn set_reward_world_quest(env: &WowLuaEnv, is_world_quest: bool) {
    let mut sim = env.state().borrow_mut();
    let quest = sim
        .quest_log_entries
        .entries
        .iter_mut()
        .find(|entry| entry.quest_id == QUEST_ID)
        .unwrap();
    quest.is_world_quest = is_world_quest;
}

#[test]
fn quest_currency_war_mode_supplied_desired_is_live_independent_and_environment_owned() {
    let first = WowLuaEnv::new().unwrap();
    let second = WowLuaEnv::new().unwrap();
    first.state().borrow_mut().war_mode.desired = true;
    probe(
        &first,
        "assert(C_PvP.IsWarModeDesired() == true, 'supplied desired mode')",
    );
    second.state().borrow_mut().player.pvp_enabled = true;
    probe(
        &second,
        "assert(GetPVPDesired() == true); assert(C_PvP.IsWarModeDesired() == false, 'generic PvP not War Mode')",
    );
    first.state().borrow_mut().war_mode.desired = false;
    probe(
        &first,
        "assert(C_PvP.IsWarModeDesired() == false, 'live desired update')",
    );
    second.state().borrow_mut().war_mode.desired = true;
    probe(&second, "assert(C_PvP.IsWarModeDesired() == true)");
    probe(
        &first,
        "assert(C_PvP.IsWarModeDesired() == false, 'separate environment')",
    );
}

#[test]
fn quest_currency_war_mode_supplied_percentage_is_live_without_changing_default_query() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().war_mode.reward_bonus = 25.5;
    probe(
        &env,
        "assert(select('#', C_PvP.GetWarModeRewardBonus()) == 1); assert(C_PvP.GetWarModeRewardBonus() == 25.5, 'supplied percentage'); assert(C_PvP.GetWarModeRewardBonusDefault() == 10)",
    );
    env.state().borrow_mut().war_mode.reward_bonus = 0.0;
    probe(
        &env,
        "assert(C_PvP.GetWarModeRewardBonus() == 0, 'live zero percentage'); assert(C_PvP.GetWarModeRewardBonusDefault() == 10)",
    );
}

#[test]
fn quest_currency_war_mode_supplied_eligibility_is_not_world_quest_classification() {
    let env = currency_reward_env();
    let mut daily = env.state().borrow().quest_log_entries.entries[0].clone();
    daily.quest_id = 901002;
    daily.is_world_quest = false;
    env.state()
        .borrow_mut()
        .quest_log_entries
        .entries
        .push(daily);
    env.state()
        .borrow_mut()
        .quest_war_mode_eligibility
        .extend([901002, i32::MIN, i32::MAX]);
    probe(
        &env,
        r#"
        local get = C_QuestLog.QuestCanHaveWarModeBonus
        local function flag(expected, ...)
            assert(select('#', ...) == 1, 'one eligibility boolean')
            assert((...) == expected, 'supplied eligibility preserved')
        end
        flag(true, get(901002))
        flag(false, get(901001))
        assert(not C_QuestLog.IsWorldQuest(901002), 'classification stays separate')
        assert(C_QuestLog.IsWorldQuest(901001) and get(901001) == false, 'world quest not inferred eligible')
        assert(get(-2147483648) and get(2147483647), 'represented identities')
        for _, id in ipairs({901002.5, 4295868298, 2147483648, -2147483649, 0/0, math.huge, -math.huge}) do
            assert(get(id) == false, 'unrepresented IDs do not alias eligibility')
        end
    "#,
    );
    env.state()
        .borrow_mut()
        .quest_war_mode_eligibility
        .remove(&901002);
    env.state()
        .borrow_mut()
        .quest_war_mode_eligibility
        .insert(QUEST_ID);
    probe(
        &env,
        "assert(not C_QuestLog.QuestCanHaveWarModeBonus(901002)); assert(C_QuestLog.QuestCanHaveWarModeBonus(901001))",
    );
}

#[test]
fn quest_currency_war_mode_supplied_faction_projection_updates_removes_and_preserves_identity() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().currency_faction_grants.extend([
        (90001, 47),
        (90002, 89),
        (i32::MIN, 71),
        (i32::MAX, 89),
    ]);
    probe(
        &env,
        r#"
        local get = C_CurrencyInfo.GetFactionGrantedByCurrency
        local function faction(expected, ...)
            assert(select('#', ...) == 1, 'one nullable faction value')
            assert((...) == expected, 'supplied faction preserved')
        end
        faction(47, get(90001))
        faction(89, get(90002))
        assert(get(-2147483648) == 71 and get(2147483647) == 89, 'represented identities')
        originalFaction = get(90001)
        for _, id in ipairs({90001.5, 4295057297, 2147483648, -2147483649, 0/0, math.huge, -math.huge}) do
            assert(get(id) == nil, 'unrepresented IDs do not alias faction')
        end
    "#,
    );
    env.state()
        .borrow_mut()
        .currency_faction_grants
        .insert(90001, 99);
    probe(
        &env,
        "assert(C_CurrencyInfo.GetFactionGrantedByCurrency(90001) == 99); assert(originalFaction == 47); assert(C_CurrencyInfo.GetFactionGrantedByCurrency(90002) == 89)",
    );
    env.state()
        .borrow_mut()
        .currency_faction_grants
        .remove(&90001);
    probe(
        &env,
        "assert(C_CurrencyInfo.GetFactionGrantedByCurrency(90001) == nil); assert(C_CurrencyInfo.GetFactionGrantedByCurrency(90002) == 89)",
    );
}

#[test]
#[cfg(feature = "retail-12-1-0")]
fn quest_currency_war_mode_supplied_untainted_secrets_preserve_identity_and_caller() {
    let env = WowLuaEnv::new().unwrap();
    env.state()
        .borrow_mut()
        .quest_war_mode_eligibility
        .insert(QUEST_ID);
    env.state()
        .borrow_mut()
        .currency_faction_grants
        .insert(CURRENCY_ID, 47);
    probe(
        &env,
        r#"
        debug.settaintmode(true)
        local quest, currency = secretwrap(901001), secretwrap(90001)
        assert(C_QuestLog.QuestCanHaveWarModeBonus(quest) == true, 'wrapped quest identity')
        assert(C_CurrencyInfo.GetFactionGrantedByCurrency(currency) == 47, 'wrapped currency identity')
        for _, get in ipairs({C_QuestLog.QuestCanHaveWarModeBonus, C_CurrencyInfo.GetFactionGrantedByCurrency}) do
            local ok, message = pcall(get, secretwrap('invalid'))
            assert(not ok and type(message) == 'string' and string.find(message, 'number', 1, true), 'unwrapped type rejected')
        end
        assert(issecretvalue(quest) and issecretvalue(currency), 'wrappers remain secret')
        assert(secretunwrap(quest) == 901001 and secretunwrap(currency) == 90001, 'wrapped IDs unchanged')
        assert(debug.getstacktaint() == nil, 'untainted caller preserved')
    "#,
    );
}

#[test]
fn quest_currency_war_mode_actual_vendor_bonus_and_each_independent_gate() {
    let env = currency_reward_env();
    enable_reward_inputs(&env);
    assert_reward_bonus(&env, true);
    env.state().borrow_mut().war_mode.desired = false;
    assert_reward_bonus(&env, false);
    env.state().borrow_mut().war_mode.desired = true;
    env.state()
        .borrow_mut()
        .quest_war_mode_eligibility
        .remove(&QUEST_ID);
    assert_reward_bonus(&env, false);
    env.state()
        .borrow_mut()
        .quest_war_mode_eligibility
        .insert(QUEST_ID);
    set_reward_world_quest(&env, false);
    assert_reward_bonus(&env, false);
    set_reward_world_quest(&env, true);
    for flags in [(None, None), (Some(false), Some(true)), (None, Some(false))] {
        env.state()
            .borrow_mut()
            .currency_war_mode_bonus
            .insert(CURRENCY_ID, flags);
        assert_reward_bonus(&env, false);
    }
    env.state()
        .borrow_mut()
        .currency_war_mode_bonus
        .insert(CURRENCY_ID, (Some(true), Some(false)));
    assert_reward_bonus(&env, true);
    env.state()
        .borrow_mut()
        .currency_faction_grants
        .insert(CURRENCY_ID, 47);
    assert_reward_bonus(&env, false);
    env.state()
        .borrow_mut()
        .currency_faction_grants
        .remove(&CURRENCY_ID);
    assert_reward_bonus(&env, true);
    env.state().borrow_mut().war_mode.reward_bonus = 40.0;
    assert_eq!(
        reward_lines(&env),
        vec![
            format!("|T{REWARD_ICON}:16:16|t 7 Fixture Marks"),
            "|cff1eff00+40% War Mode Bonus|r".to_string(),
        ]
    );
    assert!(env.state().borrow().lua_errors.is_empty());
}

fn supply_duplicate_rewards(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    let quest = sim
        .quest_log_entries
        .entries
        .iter_mut()
        .find(|entry| entry.quest_id == QUEST_ID)
        .unwrap();
    quest.currency_rewards = vec![
        build_currency_reward(CURRENCY_ID, 7, "Fixture Marks"),
        build_currency_reward(90002, 5, "Other Marks"),
        build_currency_reward(CURRENCY_ID, 11, "Fixture Marks"),
    ];
    sim.currency_info.insert(
        90002,
        CurrencyInfo {
            name: "Other Marks".to_string(),
            quality: 2,
            ..CurrencyInfo::default()
        },
    );
    sim.currency_war_mode_bonus
        .insert(CURRENCY_ID, (Some(true), Some(true)));
    sim.currency_war_mode_bonus
        .insert(90002, (Some(false), None));
}

fn assert_duplicate_bonus_count(env: &WowLuaEnv, bonus_count: usize) {
    let mut lines = render_reward_lines(env, 3);
    let bonus = "|cff1eff00+25% War Mode Bonus|r";
    assert_eq!(
        lines.iter().filter(|line| line.as_str() == bonus).count(),
        bonus_count
    );
    lines.retain(|line| line != bonus);
    lines.sort();
    let mut expected = vec![
        format!("|T{REWARD_ICON}:16:16|t 7 Fixture Marks"),
        format!("|T{REWARD_ICON}:16:16|t 11 Fixture Marks"),
        format!("|T{REWARD_ICON}:16:16|t 5 Other Marks"),
    ];
    expected.sort();
    assert_eq!(lines, expected, "ordinary reward rows preserved");
}

#[test]
fn quest_currency_war_mode_actual_vendor_duplicate_first_instances_are_per_currency() {
    let env = currency_reward_env();
    enable_reward_inputs(&env);
    supply_duplicate_rewards(&env);
    assert_duplicate_bonus_count(&env, 1);
    assert_duplicate_bonus_count(&env, 1);
    env.state()
        .borrow_mut()
        .currency_war_mode_bonus
        .insert(CURRENCY_ID, (Some(true), Some(false)));
    assert_duplicate_bonus_count(&env, 2);
    env.state()
        .borrow_mut()
        .currency_war_mode_bonus
        .insert(90002, (Some(true), Some(true)));
    assert_duplicate_bonus_count(&env, 3);
    env.state()
        .borrow_mut()
        .currency_war_mode_bonus
        .insert(CURRENCY_ID, (Some(true), Some(true)));
    assert_duplicate_bonus_count(&env, 2);
    env.state()
        .borrow_mut()
        .currency_faction_grants
        .insert(CURRENCY_ID, 47);
    assert_duplicate_bonus_count(&env, 1);
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn quest_currency_war_mode_actual_vendor_inputs_are_environment_local() {
    let first = currency_reward_env();
    enable_reward_inputs(&first);
    let second = currency_reward_env();
    assert_reward_bonus(&first, true);
    assert_reward_bonus(&second, false);
    first.state().borrow_mut().war_mode.desired = false;
    enable_reward_inputs(&second);
    assert_reward_bonus(&first, false);
    assert_reward_bonus(&second, true);
}
