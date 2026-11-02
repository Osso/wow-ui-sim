//! Original HonorFrame consumers of three independently supplied reward caches.
#![cfg(feature = "client-retail")]

use wow_ui_sim::c_api::c_pvp::reward_cache::{PvpRewardCache, PvpRewardCurrency, PvpRewardItem};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::CurrencyInfo;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

pub(crate) fn load_honor_frame(env: &WowLuaEnv) {
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let path = ui.join("Blizzard_PVPUI/Blizzard_PVPUI_Mainline.toc");
    let toc = TocFile::from_file(&path).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(!toc.is_game_type_restricted());
    assert!(
        toc.files
            .iter()
            .any(|file| file.ends_with("Blizzard_PVPUI.lua"))
    );
    wow_ui_sim::loader::load_addon(&env.loader_env(), &path).unwrap();
    env.exec("assert(debug.getinfo(HonorFrameBonusFrame_Update, 'S').source:find('Blizzard_PVPUI.lua', 1, true))").unwrap();
}

fn cache(id: i64, honor: i64, experience: i64) -> PvpRewardCache {
    PvpRewardCache {
        honor,
        experience,
        items: vec![PvpRewardItem {
            id,
            name: format!("Reward {id}"),
            texture: 134400,
            quantity: 2,
        }],
        ..Default::default()
    }
}

fn seed_rewards(env: &WowLuaEnv) {
    let state = env.state();
    let mut sim = state.borrow_mut();
    sim.pvp_reward_caches.random_bg = cache(101, 11, 21);
    sim.pvp_reward_caches.arena_skirmish = cache(102, 12, 22);
    sim.pvp_reward_caches.random_epic_bg = cache(103, 13, 23);
}

fn assert_reward(env: &WowLuaEnv, button: &str, query: &str, id: i64, honor: i64, experience: i64) {
    env.exec(&format!(
        r#"
        HonorFrameBonusFrame_Update()
        local reward = HonorFrame.BonusFrame.{button}.Reward
        assert(reward:IsShown() and reward.itemID == {id})
        assert(reward.honor == {honor} and reward.experience == {experience})
        local h, x, items, currencies, shortage = C_PvP.{query}()
        assert(select('#', C_PvP.{query}()) == 5)
        assert(h == {honor} and x == {experience})
        assert(items[1].id == {id} and items[1].name == 'Reward {id}')
        assert(items[1].texture == 134400 and items[1].quantity == 2)
        assert(currencies == nil and shortage == nil)
    "#
    ))
    .unwrap();
}

fn assert_empty(env: &WowLuaEnv, button: &str, query: &str) {
    env.exec(&format!(
        r#"
        HonorFrameBonusFrame_Update()
        assert(not HonorFrame.BonusFrame.{button}.Reward:IsShown())
        assert(select('#', C_PvP.{query}()) == 5)
        local h, x, items, currencies, shortage = C_PvP.{query}()
        assert(h == 0 and x == 0 and items == nil and currencies == nil and shortage == nil)
    "#
    ))
    .unwrap();
}

prefork_full_ui_case! {
fn random_bg_rewards_drive_honor_frame(env: &WowLuaEnv) {
    load_honor_frame(env);
    seed_rewards(env);
    assert_reward(env, "RandomBGButton", "GetRandomBGRewards", 101, 11, 21);
    env.state().borrow_mut().pvp_reward_caches.random_bg = Default::default();
    assert_empty(env, "RandomBGButton", "GetRandomBGRewards");
    assert_reward(env, "Arena1Button", "GetArenaSkirmishRewards", 102, 12, 22);
}
}

prefork_full_ui_case! {
fn arena_skirmish_rewards_drive_honor_frame(env: &WowLuaEnv) {
    load_honor_frame(env);
    seed_rewards(env);
    assert_reward(env, "Arena1Button", "GetArenaSkirmishRewards", 102, 12, 22);
    env.state().borrow_mut().pvp_reward_caches.arena_skirmish = Default::default();
    assert_empty(env, "Arena1Button", "GetArenaSkirmishRewards");
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.currency_info.insert(1602, CurrencyInfo {
            currency_id: 1602,
            name: "Fixture conquest".into(),
            icon_file_id: 134400,
            ..Default::default()
        });
        sim.pvp_reward_caches.arena_skirmish.currencies.push(PvpRewardCurrency { id: 1602, quantity: 9 });
    }
    env.exec(r#"
        HonorFrameBonusFrame_Update()
        local _, _, items, currencies = C_PvP.GetArenaSkirmishRewards()
        assert(items == nil and currencies[1].id == 1602 and currencies[1].quantity == 9)
        assert(HonorFrame.BonusFrame.Arena1Button.Reward.conquestAmount == 9)
    "#).unwrap();
    assert_reward(env, "RandomEpicBGButton", "GetRandomEpicBGRewards", 103, 13, 23);
}
}

prefork_full_ui_case! {
fn random_epic_rewards_drive_honor_frame(env: &WowLuaEnv) {
    load_honor_frame(env);
    seed_rewards(env);
    assert_reward(env, "RandomEpicBGButton", "GetRandomEpicBGRewards", 103, 13, 23);
    env.state().borrow_mut().pvp_reward_caches.random_epic_bg = Default::default();
    assert_empty(env, "RandomEpicBGButton", "GetRandomEpicBGRewards");
    assert_reward(env, "RandomBGButton", "GetRandomBGRewards", 101, 11, 21);
}
}
