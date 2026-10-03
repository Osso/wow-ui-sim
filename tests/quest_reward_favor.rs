//! Row 293: bounded host-seeded favor behavior, not native parity proof.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_bool, wrap_host_secret_number, wrap_secret};
use wow_ui_sim::c_api::c_quest_info_system::QuestRewardFavor;
use wow_ui_sim::lua_api::WowLuaEnv;

fn favor_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("quest favor environment");
    env.exec(
        r#"
        function CheckFavor(quest, clamp, expected)
            local function check(...)
                assert(select('#', ...) == 1)
                local amount = ...
                assert(type(amount) == 'number' and amount == expected)
                assert(not issecretvalue(amount), 'INFERRED public output')
            end
            check(C_QuestInfoSystem.GetQuestLogRewardFavor(quest, clamp))
        end
        function RejectFavor(quest, clamp, fragment)
            local ok, err = pcall(C_QuestInfoSystem.GetQuestLogRewardFavor, quest, clamp)
            assert(not ok and type(err) == 'string')
            assert(string.find(err, fragment, 1, true), err)
        end
    "#,
    )
    .unwrap();
    env
}

fn seeded_favor_env() -> WowLuaEnv {
    let env = favor_env();
    {
        let mut state = env.state().borrow_mut();
        state.quest_favor.context_quest_id = Some(90001);
        state.quest_favor.rewards.extend([
            (
                90001,
                QuestRewardFavor {
                    amount: 120.0,
                    cycle_capped_amount: 50.0,
                },
            ),
            (
                90002,
                QuestRewardFavor {
                    amount: 40.0,
                    cycle_capped_amount: 17.0,
                },
            ),
        ]);
        // A different selected quest must not supply the INFERRED nil context.
        state.selected_quest_log_id = Some(90002);
    }
    env
}

fn install_favor_secrets(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).unwrap();
    let number = wrap_host_secret_number(lua.state_mut(), 90001.0);
    lua.state_mut().push(number);
    lua.set_global_val("FavorSecretQuest", number).unwrap();
    lua.state_mut().pop();
    for (name, flag) in [("FavorSecretTrue", true), ("FavorSecretFalse", false)] {
        let value = wrap_host_secret_bool(lua.state_mut(), flag);
        lua.state_mut().push(value);
        lua.set_global_val(name, value).unwrap();
        lua.state_mut().pop();
    }
    let nil = wrap_secret(lua.state_mut(), rilua::Val::Nil).unwrap();
    lua.state_mut().push(nil);
    lua.set_global_val("FavorSecretNil", nil).unwrap();
    lua.state_mut().pop();
}

#[test]
fn favor_empty_defaults_return_zero_without_context_or_records() {
    let env = favor_env();
    assert!(env.state().borrow().quest_favor.rewards.is_empty());
    assert_eq!(env.state().borrow().quest_favor.context_quest_id, None);
    env.exec(
        r#"
        CheckFavor(nil, nil, 0)
        CheckFavor(90001, false, 0)
        assert(C_QuestInfoSystem.GetQuestLogRewardFavor() == 0)
    "#,
    )
    .unwrap();
}

#[test]
fn favor_nil_quest_uses_only_explicit_context_and_default_clamp_is_false() {
    let env = seeded_favor_env();
    let amount: f64 = env
        .eval("return C_QuestInfoSystem.GetQuestLogRewardFavor()")
        .unwrap();
    assert_eq!(amount, 120.0);
    env.exec("CheckFavor(nil, nil, 120); CheckFavor(nil, false, 120); CheckFavor(nil, true, 50)")
        .unwrap();
    env.state().borrow_mut().quest_favor.context_quest_id = Some(90002);
    env.exec("CheckFavor(nil, nil, 40); CheckFavor(nil, true, 17)")
        .unwrap();
    env.state().borrow_mut().quest_favor.context_quest_id = None;
    env.exec("CheckFavor(nil, false, 0); CheckFavor(90001, false, 120)")
        .unwrap();
    env.state().borrow_mut().quest_favor.context_quest_id = Some(90003);
    env.exec("CheckFavor(nil, false, 0)").unwrap();
}

#[test]
fn favor_distinct_records_clamp_values_and_live_replacement_are_exact() {
    let env = seeded_favor_env();
    env.exec(
        "CheckFavor(90001, false, 120); CheckFavor(90001, true, 50); CheckFavor(90002, true, 17)",
    )
    .unwrap();
    env.state().borrow_mut().quest_favor.rewards.insert(
        90001,
        QuestRewardFavor {
            amount: 81.5,
            cycle_capped_amount: 2.5,
        },
    );
    env.exec("CheckFavor(90001, nil, 81.5); CheckFavor(90001, true, 2.5)")
        .unwrap();
    env.state().borrow_mut().quest_favor.rewards.remove(&90001);
    env.exec("CheckFavor(90001, false, 0); CheckFavor(nil, true, 0); CheckFavor(90002, false, 40)").unwrap();
}

#[test]
fn favor_zero_and_u32_endpoint_keys_are_explicit_hits() {
    let env = seeded_favor_env();
    env.state().borrow_mut().quest_favor.rewards.extend([
        (
            0,
            QuestRewardFavor {
                amount: 0.0,
                cycle_capped_amount: 0.0,
            },
        ),
        (
            u32::MAX,
            QuestRewardFavor {
                amount: 9.0,
                cycle_capped_amount: 3.0,
            },
        ),
    ]);
    env.exec(
        "CheckFavor(0, true, 0); CheckFavor(4294967295, false, 9); CheckFavor(4294967295, true, 3)",
    )
    .unwrap();
}

#[test]
fn favor_wrong_public_inputs_error_before_state_lookup() {
    favor_env()
        .exec(
            r#"
        for _, value in ipairs({true, false, '90001', {}, function() end,
            coroutine.create(function() end), CreateFrame('Frame'), -1, 0/0,
            math.huge, -math.huge, 90001.5, 4294967296}) do
            RejectFavor(value, false, 'questID')
        end
        for _, value in ipairs({0, 1, 'true', {}, function() end, CreateFrame('Frame')}) do
            RejectFavor(nil, value, 'clampFavorToCycleCap')
        end
    "#,
        )
        .unwrap();
}

#[test]
fn favor_queries_are_read_only_and_environment_inputs_are_isolated() {
    let first = seeded_favor_env();
    let second = seeded_favor_env();
    let before = first.state().borrow().quest_favor.clone();
    first.exec("CheckFavor(nil, true, 50); CheckFavor(90002, false, 40); CheckFavor(90003, nil, 0)").unwrap();
    assert_eq!(first.state().borrow().quest_favor, before);
    first.state().borrow_mut().quest_favor.rewards.clear();
    second
        .exec("CheckFavor(nil, false, 120); CheckFavor(90002, true, 17)")
        .unwrap();
}

#[test]
fn favor_untainted_secrets_are_authenticated_without_declassifying_inputs() {
    let env = seeded_favor_env();
    install_favor_secrets(&env);
    env.exec(
        r#"
        local quest, flag, nilValue = FavorSecretQuest, FavorSecretTrue, FavorSecretNil
        for iteration = 1, 3 do
            collectgarbage('collect')
            assert(issecure())
            CheckFavor(FavorSecretQuest, false, 120)
            CheckFavor(90001, FavorSecretTrue, 50)
            CheckFavor(FavorSecretQuest, FavorSecretTrue, 50)
            CheckFavor(FavorSecretNil, FavorSecretFalse, 120)
            CheckFavor(nil, FavorSecretNil, 120)
            assert(C_QuestInfoSystem.GetQuestLogRewardFavor(90001) == 120)
            assert(C_QuestInfoSystem.GetQuestLogRewardFavor(90001, false, FavorSecretTrue) == 120)
            assert(issecretvalue(quest) and issecretvalue(flag) and issecretvalue(nilValue))
            assert(rawequal(quest, FavorSecretQuest) and rawequal(flag, FavorSecretTrue))
            assert(issecure())
        end
    "#,
    )
    .unwrap();
}

#[test]
fn favor_tainted_secret_rejection_precedes_validation_and_lookup() {
    let env = seeded_favor_env();
    install_favor_secrets(&env);
    let before = env.state().borrow().quest_favor.clone();
    env.exec(
        r#"
        local function probe()
            assert(debug.getstacktaint() == 'FavorProbe')
            CheckFavor(90001, true, 50)
            CheckFavor(nil, false, 120)
            RejectFavor(FavorSecretQuest, false, 'untainted caller')
            RejectFavor(90001, FavorSecretTrue, 'untainted caller')
            RejectFavor(FavorSecretQuest, FavorSecretFalse, 'untainted caller')
            RejectFavor(FavorSecretNil, false, 'untainted caller')
            RejectFavor(nil, FavorSecretNil, 'untainted caller')
            assert(C_QuestInfoSystem.GetQuestLogRewardFavor(90001, false, {}) == 120)
            RejectFavor(false, FavorSecretTrue, 'untainted caller')
            RejectFavor(90003, FavorSecretTrue, 'untainted caller')
            RejectFavor(FavorSecretQuest, {}, 'untainted caller')
            local ok, err = pcall(C_QuestInfoSystem.GetQuestLogRewardFavor, false, false, FavorSecretTrue)
            assert(not ok and string.find(err, 'untainted caller', 1, true))
            assert(issecretvalue(FavorSecretQuest) and issecretvalue(FavorSecretTrue))
            assert(debug.getstacktaint() == 'FavorProbe')
        end
        debug.setobjecttaint(probe, 'FavorProbe')
        probe()
        assert(issecure())
        CheckFavor(FavorSecretQuest, FavorSecretFalse, 120)
    "#,
    ).unwrap();
    assert_eq!(env.state().borrow().quest_favor, before);
}
