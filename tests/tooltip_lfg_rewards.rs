#![cfg(all(
    feature = "client-retail",
    feature = "retail-12-1-5",
    feature = "optional-tests"
))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{LfgTooltipReward, LfgTooltipRewardGroup};
use wow_ui_sim::screen::ScreenKind;

const DUNGEON_ID: i32 = 1203;

fn original_group_finder_env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let discovered = wow_ui_sim::loader::discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    assert!(
        discovered
            .iter()
            .any(|(name, _)| name == "Blizzard_GroupFinder"),
        "Retail game-screen TOC selection must include Blizzard_GroupFinder"
    );

    let env = new_blizzard_addon_env(&ui);
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_GroupFinder"], &[]);
    assert!(
        loaded.iter().any(|name| name == "Blizzard_GroupFinder"),
        "selected Blizzard_GroupFinder closure must load"
    );
    env
}

fn seed_group(env: &WowLuaEnv, group: LfgTooltipRewardGroup) {
    let mut state = env.state().borrow_mut();
    assert!(
        state.lfd_dungeons.iter().any(|dungeon| {
            dungeon.dungeon_id == group.dungeon_id && dungeon.type_id == group.dungeon_type_id
        }),
        "host reward group must match a represented LFD dungeon"
    );
    state.lfg_tooltip_reward_groups.push(group);
    state.lfg_tooltip_rewards_ready = true;
}

fn reward_group(env: &WowLuaEnv) -> LfgTooltipRewardGroup {
    let dungeon_type_id = env
        .state()
        .borrow()
        .lfd_dungeons
        .iter()
        .find(|dungeon| dungeon.dungeon_id == DUNGEON_ID)
        .map(|dungeon| dungeon.type_id)
        .expect("fixture dungeon must exist in the default LFD catalog");
    LfgTooltipRewardGroup {
        dungeon_id: DUNGEON_ID,
        dungeon_type_id,
        ..LfgTooltipRewardGroup::default()
    }
}

fn assert_original_reward_consumer(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(C_AddOns.IsAddOnLoaded('Blizzard_GroupFinder'), 'selected GroupFinder must be loaded')
        local source = debug.getinfo(LFGDungeonReadyDialogReward_OnEnter, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_GroupFinder/Shared/LFGFrame.lua', 1, true), source)
        assert(GameTooltip.GetPrimaryTooltipInfo ~= nil, 'original TooltipDataHandler must be loaded')
        local rewardSource = debug.getinfo(GameTooltip.SetLFGDungeonReward, 'S').source:gsub('\\', '/')
        local shortageSource = debug.getinfo(GameTooltip.SetLFGDungeonShortageReward, 'S').source:gsub('\\', '/')
        assert(rewardSource:find('TooltipDataHandler.lua', 1, true), rewardSource)
        assert(shortageSource:find('TooltipDataHandler.lua', 1, true), shortageSource)
        "#,
    )
    .expect("selected original LFGFrame reward consumer should be present");
}

fn run_original_reward_hover(env: &WowLuaEnv, reward_type: &str, reward_index: i32, severity: i32) {
    env.exec(&format!(
        r#"
        local button = CreateFrame('Button', nil, UIParent)
        button.rewardType = '{reward_type}'
        button.rewardID = {reward_index}
        button.rewardArg = {severity}
        LFGDungeonReadyPopup.dungeonID = {DUNGEON_ID}
        LFGDungeonReadyDialogReward_OnEnter(button, {DUNGEON_ID})
        "#
    ))
    .expect("original LFGFrame reward hover should dispatch through GameTooltip");
}

fn assert_primary_identity(env: &WowLuaEnv, getter_name: &str, tooltip_type: &str, reward_id: u32) {
    env.exec(&format!(
        r#"
        local info = GameTooltip:GetPrimaryTooltipInfo()
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(data, 'selected reward must publish tooltip data')
        assert(info and info.getterName == '{getter_name}', 'TooltipDataHandler selected the wrong getter')
        assert(data.type == Enum.TooltipDataType.{tooltip_type}, 'reward tooltip type mismatch')
        assert(data.id == {reward_id}, 'reward tooltip identity mismatch')
        "#
    ))
    .expect("original reward accessor should publish the selected host reward identity");
}

#[test]
fn lfg_dungeon_reward_consumer_selects_distinct_item_entries() {
    let env = original_group_finder_env();
    assert_original_reward_consumer(&env);

    let mut group = reward_group(&env);
    group.rewards = vec![
        LfgTooltipReward {
            reward_id: 6948,
            quantity: 1,
            is_currency: false,
        },
        LfgTooltipReward {
            reward_id: 159,
            quantity: 2,
            is_currency: false,
        },
    ];
    seed_group(&env, group);

    run_original_reward_hover(&env, "reward", 1, 0);
    assert_primary_identity(&env, "GetLFGDungeonReward", "Item", 6948);
    run_original_reward_hover(&env, "reward", 2, 0);
    assert_primary_identity(&env, "GetLFGDungeonReward", "Item", 159);
}

#[test]
fn lfg_shortage_reward_consumer_selects_distinct_currency_tiers() {
    let env = original_group_finder_env();
    assert_original_reward_consumer(&env);

    let mut group = reward_group(&env);
    group.shortage_rewards[0] = vec![LfgTooltipReward {
        reward_id: 2245,
        quantity: 17,
        is_currency: true,
    }];
    group.shortage_rewards[2] = vec![LfgTooltipReward {
        reward_id: 1792,
        quantity: 23,
        is_currency: true,
    }];
    seed_group(&env, group);

    run_original_reward_hover(&env, "shortage", 1, 1);
    assert_primary_identity(&env, "GetLFGDungeonShortageReward", "Currency", 2245);
    run_original_reward_hover(&env, "shortage", 1, 3);
    assert_primary_identity(&env, "GetLFGDungeonShortageReward", "Currency", 1792);
}

#[test]
fn lfg_reward_getters_return_no_values_without_ready_reward_data() {
    let env = WowLuaEnv::new().expect("Retail Lua environment should initialize");
    env.exec(&format!(
        r#"
        assert(select('#', C_TooltipInfo.GetLFGDungeonReward({DUNGEON_ID}, 1)) == 0,
            'empty default reward input must not fabricate an ordinary reward')
        assert(select('#', C_TooltipInfo.GetLFGDungeonShortageReward({DUNGEON_ID}, 1, 1)) == 0,
            'empty default reward input must not fabricate a shortage reward')
        "#
    ))
    .expect("empty default LFG reward input should remain empty");
    let group = reward_group(&env);
    env.state()
        .borrow_mut()
        .lfg_tooltip_reward_groups
        .push(group);

    env.exec(&format!(
        r#"
        assert(select('#', C_TooltipInfo.GetLFGDungeonReward({DUNGEON_ID}, 1)) == 0,
            'ordinary reward miss must return zero values')
        assert(select('#', C_TooltipInfo.GetLFGDungeonShortageReward({DUNGEON_ID}, 1, 1)) == 0,
            'shortage reward miss must return zero values')
        assert(select('#', C_TooltipInfo.GetLFGDungeonShortageReward({DUNGEON_ID}, 4, 1)) == 0,
            'unsupported shortage tier must return zero values')
        "#
    ))
    .expect("missing or unsupported reward state must return no Lua values");
}
