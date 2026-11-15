//! Original selected GroupFinder consumer over decoded replacement messages.
#![cfg(all(
    feature = "profile-retail",
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

use wow_ui_sim::c_api::lfg_reward_writer::{DecodedLfgRewardMessage, apply_decoded_message};

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

const LISTEN: &str = r#"
    WriterEvents = {}
    local frame = CreateFrame('Frame')
    frame:RegisterEvent('LFG_LOCK_INFO_RECEIVED')
    frame:RegisterEvent('LFG_UPDATE_RANDOM_INFO')
    frame:SetScript('OnEvent', function(_, event)
        local ordinary = C_TooltipInfo.GetLFGDungeonReward(1203, 1)
        local tier1 = C_TooltipInfo.GetLFGDungeonShortageReward(1203, 1, 1)
        local tier3 = C_TooltipInfo.GetLFGDungeonShortageReward(1203, 3, 1)
        local obsolete = C_TooltipInfo.GetLFGDungeonReward(WriterObsoleteDungeon, 1)
        table.insert(WriterEvents, {event=event, ordinary=ordinary and ordinary.id,
            tier1=tier1 and tier1.id, tier3=tier3 and tier3.id,
            quantity=(tier1 or tier3) and (tier1 or tier3).quantity,
            obsolete=obsolete and obsolete.id})
    end)
    WriterListener = frame
"#;

fn backed_group(
    env: &WowLuaEnv,
    item: u32,
    currency: u32,
    quantity: i32,
    tier: usize,
) -> LfgTooltipRewardGroup {
    let mut group = reward_group(env);
    group.rewards = vec![LfgTooltipReward {
        reward_id: item,
        quantity: 2,
        is_currency: false,
    }];
    group.shortage_rewards[tier] = vec![LfgTooltipReward {
        reward_id: currency,
        quantity,
        is_currency: true,
    }];
    group
}

fn obsolete_group(env: &WowLuaEnv) -> LfgTooltipRewardGroup {
    let mut group = reward_group(env);
    let sim = env.state().borrow();
    let dungeon = sim
        .lfd_dungeons
        .iter()
        .find(|d| d.dungeon_id != DUNGEON_ID)
        .unwrap();
    group.dungeon_id = dungeon.dungeon_id;
    group.dungeon_type_id = dungeon.type_id;
    group.rewards = vec![LfgTooltipReward {
        reward_id: 6948,
        quantity: 1,
        is_currency: false,
    }];
    group
}

fn assert_hover_pair(env: &WowLuaEnv, item: u32, currency: u32, quantity: i32, tier: i32) {
    run_original_reward_hover(env, "reward", 1, 0);
    assert_primary_identity(env, "GetLFGDungeonReward", "Item", item);
    run_original_reward_hover(env, "shortage", 1, tier);
    assert_primary_identity(env, "GetLFGDungeonShortageReward", "Currency", currency);
    env.exec(&format!(
        "assert(GameTooltip:GetPrimaryTooltipData().quantity == {quantity})"
    ))
    .unwrap();
}

fn assert_event_pair(
    env: &WowLuaEnv,
    count: usize,
    item: &str,
    tier1: &str,
    tier3: &str,
    qty: &str,
    obsolete: &str,
) {
    env.exec(&format!(
        r#"
        assert(#WriterEvents == {count}, 'exactly two events per update required')
        local first, second = WriterEvents[{count}-1], WriterEvents[{count}]
        assert(first.event == 'LFG_LOCK_INFO_RECEIVED')
        assert(second.event == 'LFG_UPDATE_RANDOM_INFO')
        for _, record in ipairs({{first, second}}) do
            assert(record.ordinary == {item} and record.tier1 == {tier1})
            assert(record.tier3 == {tier3} and record.quantity == {qty})
            assert(record.obsolete == {obsolete}, 'event callback saw stale groups')
        end
    "#
    ))
    .expect("callbacks must observe fully replaced state in native event order");
}

fn apply_first_update(env: &WowLuaEnv, obsolete: LfgTooltipRewardGroup) {
    let group = backed_group(env, 6948, 2245, 17, 0);
    apply_decoded_message(
        env,
        DecodedLfgRewardMessage {
            groups: vec![group, obsolete],
        },
    )
    .unwrap();
    assert!(env.state().borrow().lfg_tooltip_rewards_ready);
    assert_event_pair(env, 2, "6948", "2245", "nil", "17", "6948");
    assert_hover_pair(env, 6948, 2245, 17, 1);
    assert_eq!(env.state().borrow().lfg_tooltip_reward_groups.len(), 2);
}

fn apply_second_update(env: &WowLuaEnv) {
    let group = backed_group(env, 159, 1792, 23, 2);
    apply_decoded_message(
        env,
        DecodedLfgRewardMessage {
            groups: vec![group],
        },
    )
    .unwrap();
    assert!(env.state().borrow().lfg_tooltip_rewards_ready);
    assert_event_pair(env, 4, "159", "nil", "1792", "23", "nil");
    assert_hover_pair(env, 159, 1792, 23, 3);
    let sim = env.state().borrow();
    assert_eq!(sim.lfg_tooltip_reward_groups.len(), 1);
    assert_eq!(sim.lfg_tooltip_reward_groups[0].dungeon_id, DUNGEON_ID);
    assert!(sim.lfg_tooltip_reward_groups[0].shortage_rewards[0].is_empty());
    assert!(sim.lfg_tooltip_reward_groups[0].shortage_rewards[1].is_empty());
}

fn apply_empty_update(env: &WowLuaEnv) {
    apply_decoded_message(env, DecodedLfgRewardMessage { groups: vec![] }).unwrap();
    assert!(env.state().borrow().lfg_tooltip_rewards_ready);
    assert_event_pair(env, 6, "nil", "nil", "nil", "nil", "nil");
    assert!(env.state().borrow().lfg_tooltip_reward_groups.is_empty());
    env.exec(
        r#"
        assert(select('#', C_TooltipInfo.GetLFGDungeonReward(1203, 1)) == 0)
        for tier=1,3 do
            assert(select('#', C_TooltipInfo.GetLFGDungeonShortageReward(1203, tier, 1)) == 0)
        end
    "#,
    )
    .unwrap();
}

#[test]
fn decoded_updates_replace_groups_and_tiers_before_ordered_events_and_original_hover() {
    let env = original_group_finder_env();
    assert_original_reward_consumer(&env);
    assert!(!env.state().borrow().lfg_tooltip_rewards_ready);
    let obsolete = obsolete_group(&env);
    env.exec(&format!("WriterObsoleteDungeon = {}", obsolete.dungeon_id))
        .unwrap();
    env.exec(LISTEN).unwrap();
    apply_first_update(&env, obsolete);
    apply_second_update(&env);
    apply_empty_update(&env);
}
