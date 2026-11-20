//! Retail and Forever quest and guild-bank item tooltip consumers.
#![cfg(any(feature = "profile-retail", feature = "client-wowforever"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::{
    WowLuaEnv,
    state::{BagItem, QuestRewardItem},
};

fn tooltip_env() -> Option<WowLuaEnv> {
    matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever)
        .then(tooltip_full_env_helpers::setup_full_env)
}

fn reward_item(item_id: u32, name: &str) -> QuestRewardItem {
    QuestRewardItem {
        name: name.to_owned(),
        texture: "Interface\\Icons\\INV_Misc_QuestionMark".to_owned(),
        count: 1,
        quality: 1,
        is_usable: false,
        item_id: Some(item_id),
    }
}

fn seed_quest_items(env: &WowLuaEnv) -> (i32, i32) {
    use wow_ui_sim::c_api::quest_readiness::QuestDialogPhase;

    let mut sim = env.state().borrow_mut();
    let dialog_id = 81234;
    let log_id = 81235;
    let mut dialog = sim.quest_log_entries.entries[0].clone();
    dialog.quest_id = dialog_id;
    dialog.reward_items = vec![reward_item(6948, "Dialog reward")];
    let mut log = sim.quest_log_entries.entries[1].clone();
    log.quest_id = log_id;
    log.reward_items = vec![reward_item(229181, "Quest-log reward")];
    sim.quest_log_entries.entries = vec![dialog, log];
    sim.quest_log = vec![dialog_id as u32, log_id as u32];
    sim.selected_quest_log_id = Some(dialog_id as u32);
    sim.gossip.active = true;
    let mut active_quest = sim
        .gossip
        .active_quests
        .first()
        .cloned()
        .unwrap_or_default();
    active_quest.quest_id = dialog_id as u32;
    sim.gossip.active_quests = vec![active_quest];
    sim.quest_log_entries.readiness.dialog = Some((dialog_id, QuestDialogPhase::Rewards));
    (dialog_id, log_id)
}

fn assert_quest_reward_hover(env: &WowLuaEnv, quest_log: bool, item_id: u32) {
    env.exec(&format!(
        r#"
        QuestInfoFrame.questLog = {quest_log}
        local button = QuestInfoRewardsFrame.RewardButtons[1]
        button:SetID(1)
        button.type = 'reward'
        button.objectType = 'item'
        button:Show()
        button:OnEnter()
        local info = GameTooltip:GetPrimaryTooltipInfo()
        local data = GameTooltip:GetPrimaryTooltipData()
        local expected = C_TooltipInfo.GetItemByID({item_id})
        assert(info and info.getterName == '{getter}', 'QuestInfo reward hover selects the matching item getter')
        assert(data and data.type == Enum.TooltipDataType.Item, 'quest reward tooltip is an item')
        assert(data.lines[1].leftText == expected.lines[1].leftText, 'quest reward tooltip uses its supplied item')
        "#,
        getter = if quest_log { "GetQuestLogItem" } else { "GetQuestItem" },
    ))
    .unwrap();
}

#[test]
fn quest_info_reward_hover_uses_dialog_and_quest_log_items() {
    let Some(env) = tooltip_env() else {
        return;
    };
    let (dialog_id, log_id) = seed_quest_items(&env);
    assert_quest_reward_hover(&env, false, 6948);
    env.state().borrow_mut().selected_quest_log_id = Some(log_id as u32);
    assert_quest_reward_hover(&env, true, 229181);
    assert_ne!(dialog_id, log_id);
}

#[test]
fn guild_bank_item_hover_uses_the_selected_slot_item() {
    let Some(env) = tooltip_env() else {
        return;
    };
    env.state().borrow_mut().guild_bank_items.insert(
        (1, 17),
        BagItem {
            owned_identity: None,
            item_id: 6948,
            stack_count: 1,
            hyperlink: None,
            durability: None,
            applied_transmog: None,
        },
    );
    env.exec(
        r###"
        local loaded, reason = C_AddOns.LoadAddOn('Blizzard_GuildBankUI')
        assert(loaded, 'GuildBankUI must load before its item consumer runs: ' .. tostring(reason))
        local button = CreateFrame('Button', nil, UIParent)
        button:SetID(17)
        GuildBankItemButtonMixin:OnEnter(button)
        local info = GameTooltip:GetPrimaryTooltipInfo()
        local data = GameTooltip:GetPrimaryTooltipData()
        local expected = C_TooltipInfo.GetItemByID(6948)
        assert(info and info.getterName == 'GetGuildBankItem', 'guild-bank item hover selects GetGuildBankItem')
        assert(data and data.type == Enum.TooltipDataType.Item, 'guild-bank tooltip is an item')
        assert(data.lines[1].leftText == expected.lines[1].leftText, 'guild-bank tooltip uses the selected slot item')
        "###,
    )
    .unwrap();
}
