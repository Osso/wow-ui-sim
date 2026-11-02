//! Reward-focused integration tests for the `C_QuestLog` quest-log surface.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{QuestLogEntry, QuestRewardCurrency, QuestRewardItem};

fn env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("WowLuaEnv init");
    // Wrath's legacy reward read addresses the selected quest, not argument two.
    #[cfg(feature = "client-wrath")]
    env.exec("C_QuestLog.SetSelectedQuest(80000)").unwrap();
    env
}

fn update_seeded_quest(env: &WowLuaEnv, quest_id: i32, update: impl FnOnce(&mut QuestLogEntry)) {
    let mut st = env.state().borrow_mut();
    let entry = st
        .quest_log_entries
        .entries
        .iter_mut()
        .find(|entry| entry.quest_id == quest_id)
        .expect("seeded quest entry");
    update(entry);
}

// ── GetQuestLogRewardInfo ─────────────────────────────────────────────────────

fn seed_reward_items_for(env: &WowLuaEnv, quest_id: i32, items: Vec<QuestRewardItem>) {
    update_seeded_quest(env, quest_id, |entry| entry.reward_items = items);
}

fn seed_lost_expedition_one_reward(env: &WowLuaEnv) {
    seed_reward_items_for(
        env,
        80000,
        vec![QuestRewardItem {
            name: "Earthen Lockbox".into(),
            texture: "Interface\\Icons\\INV_Box_01".into(),
            count: 1,
            quality: 3,
            is_usable: true,
            item_id: None,
        }],
    );
}

#[test]
fn get_quest_log_reward_info_returns_seeded_item() {
    let env = env();
    seed_lost_expedition_one_reward(&env);

    let (name, texture, count, quality, is_usable): (String, String, i32, i32, bool) =
        env.eval("return GetQuestLogRewardInfo(1, 80000)").unwrap();

    assert_eq!(name, "Earthen Lockbox");
    assert_eq!(texture, "Interface\\Icons\\INV_Box_01");
    assert_eq!(count, 1);
    assert_eq!(quality, 3);
    assert!(is_usable);
}

#[test]
fn get_quest_log_reward_info_returns_nil_for_unknown_quest() {
    let env = env();
    let result: Option<bool> = env
        .eval("local n = GetQuestLogRewardInfo(1, 999999); return n ~= nil or nil")
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn get_quest_log_reward_info_returns_nil_for_index_past_end() {
    let env = env();
    seed_lost_expedition_one_reward(&env);
    let result: Option<bool> = env
        .eval("local n = GetQuestLogRewardInfo(2, 80000); return n ~= nil or nil")
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn get_quest_log_reward_info_returns_nil_for_zero_index() {
    let env = env();
    seed_lost_expedition_one_reward(&env);
    let result: Option<bool> = env
        .eval("local n = GetQuestLogRewardInfo(0, 80000); return n ~= nil or nil")
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn get_quest_log_reward_info_returns_nil_when_no_rewards_seeded() {
    let env = env();
    let result: Option<bool> = env
        .eval("local n = GetQuestLogRewardInfo(1, 80000); return n ~= nil or nil")
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn get_quest_log_reward_info_indexes_each_reward_independently() {
    let env = env();
    seed_reward_items_for(
        &env,
        80000,
        vec![
            QuestRewardItem {
                name: "First Reward".into(),
                texture: "Interface\\Icons\\INV_Misc_01".into(),
                count: 1,
                quality: 2,
                is_usable: true,
                item_id: None,
            },
            QuestRewardItem {
                name: "Second Reward".into(),
                texture: "Interface\\Icons\\INV_Misc_02".into(),
                count: 5,
                quality: 4,
                is_usable: false,
                item_id: None,
            },
        ],
    );

    let first: (String, String, i32, i32, bool) =
        env.eval("return GetQuestLogRewardInfo(1, 80000)").unwrap();
    assert_eq!(first.0, "First Reward");
    assert_eq!(first.3, 2);
    assert!(first.4);

    let second: (String, String, i32, i32, bool) =
        env.eval("return GetQuestLogRewardInfo(2, 80000)").unwrap();
    assert_eq!(second.0, "Second Reward");
    assert_eq!(second.2, 5);
    assert_eq!(second.3, 4);
    assert!(!second.4);
}

#[test]
fn get_quest_log_reward_info_is_not_usable_field_round_trips_false() {
    let env = env();
    seed_reward_items_for(
        &env,
        80000,
        vec![QuestRewardItem {
            name: "Plate Helm".into(),
            texture: "Interface\\Icons\\INV_Helm_01".into(),
            count: 1,
            quality: 4,
            is_usable: false,
            item_id: None,
        }],
    );
    let is_usable: bool = env
        .eval("local _, _, _, _, u = GetQuestLogRewardInfo(1, 80000); return u")
        .unwrap();
    assert!(!is_usable);
}

// ── C_QuestLog.GetQuestRewardCurrencies ───────────────────────────────────────

fn seed_currency_rewards_for(env: &WowLuaEnv, quest_id: i32, currencies: Vec<QuestRewardCurrency>) {
    update_seeded_quest(env, quest_id, |entry| entry.currency_rewards = currencies);
}

#[test]
fn get_quest_reward_currencies_returns_empty_table_when_unseeded() {
    let env = env();
    let count: i32 = env
        .eval("return #C_QuestLog.GetQuestRewardCurrencies(80000)")
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn get_quest_reward_currencies_returns_empty_table_for_unknown_quest() {
    let env = env();
    let count: i32 = env
        .eval("return #C_QuestLog.GetQuestRewardCurrencies(999999)")
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn get_quest_reward_currencies_returns_seeded_entry_fields() {
    let env = env();
    seed_currency_rewards_for(
        &env,
        80000,
        vec![QuestRewardCurrency {
            currency_id: 2245,
            name: "Flightstones".into(),
            texture: "Interface\\Icons\\Currency_Flightstones".into(),
            total_reward_amount: 35,
            base_reward_amount: Some(25),
        }],
    );
    let (currency_id, name, texture, total, base): (i32, String, String, i32, i32) = env
        .eval(
            r#"
            local list = C_QuestLog.GetQuestRewardCurrencies(80000)
            local row = list[1]
            return row.currencyID, row.name, row.texture,
                   row.totalRewardAmount, row.baseRewardAmount
            "#,
        )
        .unwrap();
    assert_eq!(currency_id, 2245);
    assert_eq!(name, "Flightstones");
    assert_eq!(texture, "Interface\\Icons\\Currency_Flightstones");
    assert_eq!(total, 35);
    assert_eq!(base, 25);
}

#[test]
fn get_quest_reward_currencies_omits_base_reward_when_none() {
    let env = env();
    seed_currency_rewards_for(
        &env,
        80000,
        vec![QuestRewardCurrency {
            currency_id: 1828,
            name: "Soul Ash".into(),
            texture: "Interface\\Icons\\inv_misc_soulash".into(),
            total_reward_amount: 90,
            base_reward_amount: None,
        }],
    );
    let has_base: Option<bool> = env
        .eval(
            r#"
            local row = C_QuestLog.GetQuestRewardCurrencies(80000)[1]
            return row.baseRewardAmount ~= nil or nil
            "#,
        )
        .unwrap();
    assert!(has_base.is_none());
}

#[test]
fn get_quest_reward_currencies_preserves_order() {
    let env = env();
    seed_currency_rewards_for(
        &env,
        80000,
        vec![
            QuestRewardCurrency {
                currency_id: 100,
                name: "First".into(),
                texture: "tex1".into(),
                total_reward_amount: 1,
                base_reward_amount: None,
            },
            QuestRewardCurrency {
                currency_id: 200,
                name: "Second".into(),
                texture: "tex2".into(),
                total_reward_amount: 2,
                base_reward_amount: None,
            },
            QuestRewardCurrency {
                currency_id: 300,
                name: "Third".into(),
                texture: "tex3".into(),
                total_reward_amount: 3,
                base_reward_amount: None,
            },
        ],
    );
    let (count, first_id, second_id, third_id): (i32, i32, i32, i32) = env
        .eval(
            r#"
            local list = C_QuestLog.GetQuestRewardCurrencies(80000)
            return #list, list[1].currencyID, list[2].currencyID, list[3].currencyID
            "#,
        )
        .unwrap();
    assert_eq!(count, 3);
    assert_eq!(first_id, 100);
    assert_eq!(second_id, 200);
    assert_eq!(third_id, 300);
}

#[test]
fn get_quest_reward_currencies_supports_ipairs_iteration() {
    let env = env();
    seed_currency_rewards_for(
        &env,
        80000,
        vec![
            QuestRewardCurrency {
                currency_id: 100,
                name: "Alpha".into(),
                texture: "tex1".into(),
                total_reward_amount: 7,
                base_reward_amount: None,
            },
            QuestRewardCurrency {
                currency_id: 200,
                name: "Beta".into(),
                texture: "tex2".into(),
                total_reward_amount: 9,
                base_reward_amount: None,
            },
        ],
    );
    let total_amount: i32 = env
        .eval(
            r#"
            local sum = 0
            for _, row in ipairs(C_QuestLog.GetQuestRewardCurrencies(80000)) do
                sum = sum + row.totalRewardAmount
            end
            return sum
            "#,
        )
        .unwrap();
    assert_eq!(total_amount, 16);
}

#[test]
fn get_quest_reward_currencies_returns_table_value() {
    let env = env();
    let returned_type: String = env
        .eval("return type(C_QuestLog.GetQuestRewardCurrencies(80000))")
        .unwrap();
    assert_eq!(returned_type, "table");
}
