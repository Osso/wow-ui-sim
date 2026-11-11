#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::common;
use wow_ui_sim::c_api::c_login::backgrounds::RaceBackgroundRow;
use wow_ui_sim::c_api::c_login::character_roster::CharacterSlot;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn supplied_background_env() -> common::LockedEnv {
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.set_screen_mode(ScreenKind::Login);
    let mut account = env
        .state()
        .borrow()
        .bnet_friends
        .iter()
        .flat_map(|friend| &friend.game_accounts)
        .find(|account| account.client_program == "WoW")
        .unwrap()
        .clone();
    account.class_id = 2;
    let mut slot = CharacterSlot::from(account);
    slot.record.as_mut().unwrap().race_id = 7;
    env.state().borrow_mut().glue_login.characters = vec![slot];
    env
}

#[test]
fn glue_background_uses_every_host_cascade_step_and_separate_override() {
    let env = supplied_background_env();
    assert_eq!(
        env.eval::<i32>("return GetSelectBackgroundModel(1)")
            .unwrap(),
        651555
    );
    {
        let mut sim = env.state().borrow_mut();
        sim.glue_login.backgrounds.classes.insert(2, 1101);
        sim.glue_login.backgrounds.races.insert(
            7,
            RaceBackgroundRow {
                select_screen_file_data_id: 2201,
                low_res_screen_file_data_id: 3301,
            },
        );
    }
    assert_eq!(
        env.eval::<i32>("return GetSelectBackgroundModel(1, 'ignored')")
            .unwrap(),
        1101
    );
    {
        let mut sim = env.state().borrow_mut();
        let record = sim.glue_login.characters[0].record.as_mut().unwrap();
        record.background_override = 4401;
        record.timerunning_season_id = Some(99);
    }
    assert_eq!(
        env.eval::<i32>("return GetSelectBackgroundModel('1')")
            .unwrap(),
        4401
    );
    {
        let mut sim = env.state().borrow_mut();
        sim.glue_login.characters[0]
            .record
            .as_mut()
            .unwrap()
            .background_override = 0;
        sim.glue_login.backgrounds.classes.insert(2, 0);
    }
    assert_eq!(
        env.eval::<i32>("return GetSelectBackgroundModel(1.9)")
            .unwrap(),
        2201
    );
    env.state()
        .borrow_mut()
        .glue_login
        .backgrounds
        .classes
        .clear();
    assert_eq!(
        env.eval::<i32>("return GetSelectBackgroundModel(1)")
            .unwrap(),
        2201
    );
    env.state()
        .borrow_mut()
        .glue_login
        .backgrounds
        .races
        .get_mut(&7)
        .unwrap()
        .select_screen_file_data_id = 0;
    assert_eq!(
        env.eval::<i32>("return GetSelectBackgroundModel(1)")
            .unwrap(),
        3301
    );
    env.state()
        .borrow_mut()
        .glue_login
        .backgrounds
        .races
        .get_mut(&7)
        .unwrap()
        .low_res_screen_file_data_id = 0;
    env.exec("assert(GetSelectBackgroundModel(1) == nil); assert(select('#', GetSelectBackgroundModel(1)) == 1)").unwrap();
    env.state()
        .borrow_mut()
        .glue_login
        .backgrounds
        .races
        .remove(&7);
    assert_eq!(
        env.eval::<i32>("return GetSelectBackgroundModel(1)")
            .unwrap(),
        651555
    );
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn glue_background_invalid_slots_use_only_key_zero_and_getter_uses_requested_slot() {
    let env = supplied_background_env();
    {
        let mut sim = env.state().borrow_mut();
        let mut fourth = sim.glue_login.characters[0].clone();
        fourth.character_id = "host-fourth".into();
        let record = fourth.record.as_mut().unwrap();
        record.account.wow_account_guid = fourth.character_id.clone();
        record.background_override = 5501;
        sim.glue_login.characters.extend([
            CharacterSlot::default(),
            CharacterSlot {
                character_id: "unresolved".into(),
                record: None,
            },
            fourth,
        ]);
        sim.glue_login.backgrounds.races.insert(
            0,
            RaceBackgroundRow {
                select_screen_file_data_id: 6601,
                low_res_screen_file_data_id: 7701,
            },
        );
    }
    env.exec(
        r#"
        SelectCharacter(1)
        assert(GetSelectBackgroundModel(4) == 5501)
        for _, index in ipairs({0, -1, 99, 0.5, 2, 3}) do
            assert(GetSelectBackgroundModel(index) == 7701)
            assert(select('#', GetSelectBackgroundModel(index)) == 1)
        end
        assert(GetCharacterSelection() == 1)
        local function bad(...)
            local ok, message = pcall(GetSelectBackgroundModel, ...)
            assert(not ok and message:find('Usage: GetSelectBackgroundModel(index)', 1, true))
        end
        bad(); bad(nil); bad(false); bad({}); bad('not-a-number')
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .glue_login
        .backgrounds
        .races
        .get_mut(&0)
        .unwrap()
        .low_res_screen_file_data_id = 0;
    env.exec("assert(GetSelectBackgroundModel(0) == nil)")
        .unwrap();
    env.state().borrow_mut().glue_login.characters.clear();
    env.exec("assert(GetSelectBackgroundModel(1) == nil)")
        .unwrap();
    env.state()
        .borrow_mut()
        .glue_login
        .backgrounds
        .races
        .remove(&0);
    assert_eq!(
        env.eval::<i32>("return GetSelectBackgroundModel(1)")
            .unwrap(),
        651555
    );
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn glue_background_setter_validates_without_faking_observable_model_state() {
    let env = supplied_background_env();
    env.state().borrow_mut().glue_login.characters[0]
        .record
        .as_mut()
        .unwrap()
        .background_override = 4401;
    env.exec(r#"
        SelectCharacter(1)
        backgroundEvents = 0
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('CHARACTER_LIST_UPDATE')
        listener:RegisterEvent('UPDATE_SELECTED_CHARACTER')
        listener:SetScript('OnEvent', function() backgroundEvents = backgroundEvents + 1 end)
        for _, value in ipairs({0, 651555, 651555.9, '651555', '', 'host-model.m2'}) do
            assert(select('#', SetCharSelectBackground(value, 'ignored')) == 0)
        end
        assert(backgroundEvents == 0)
        assert(GetCharacterSelection() == 1 and GetSelectBackgroundModel(1) == 4401)
        local function bad(...)
            local ok, message = pcall(SetCharSelectBackground, ...)
            assert(not ok and message:find('Usage: SetCharSelectBackground("filename" | file ID)', 1, true))
        end
        bad(); bad(nil); bad(false); bad({})
    "#).unwrap();
    env.set_screen_mode(ScreenKind::Game);
    env.exec("assert(GetSelectBackgroundModel == nil and SetCharSelectBackground == nil)")
        .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
