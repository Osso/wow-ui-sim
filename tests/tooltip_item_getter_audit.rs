#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::c_api::{
    ItemInteractionTooltipInput, ItemInteractionTooltipLine,
    quest_party_progress::{
        QuestPartyProgressHeading, QuestPartyProgressParticipant, QuestPartyProgressRow,
        QuestPartyProgressSnapshot, QuestPartyProgressText,
    },
    quest_special_item::{QuestSpecialItemCacheRecord, QuestSpecialItemObject},
    recipe_rank_tooltip::RecipeRankTooltipRecord,
};
use wow_ui_sim::lua_api::state::{AppearanceSourceInfo, ArtifactInfo, BagItem, RelicSlotInfo};

const SOCKET_ASSERTIONS: &str = r#"
local sockets = C_ItemSocketInfo._state
sockets.newSockets = {{itemID=6948}, {itemID=159}}
sockets.existingSockets = {{itemID=1728}, {itemID=127830}}
for _, case in ipairs({
    {C_TooltipInfo.GetSocketGem, GameTooltip.SetSocketGem, sockets.newSockets},
    {C_TooltipInfo.GetExistingSocketGem, GameTooltip.SetExistingSocketGem, sockets.existingSockets},
}) do
    local previous
    for index, socket in ipairs(case[3]) do
        local direct = case[1](index)
        assert(direct and direct.id == socket.itemID, 'getter selects supplied socket item')
        GameTooltip:ClearLines()
        case[2](GameTooltip, index)
        local shown = GameTooltip:GetPrimaryTooltipData()
        assert(shown and shown.id == socket.itemID, 'setter publishes selected socket item')
        assert(not previous or previous ~= shown.id, 'socket entries must differ')
        previous = shown.id
    end
end
for _, itemID in ipairs({6948, 159}) do
    sockets.itemInfo = {itemID=itemID}
    local direct = C_TooltipInfo.GetSocketedItem()
    assert(direct and direct.id == itemID, 'getter selects supplied socketed item')
    GameTooltip:ClearLines()
    GameTooltip:SetSocketedItem()
    local shown = GameTooltip:GetPrimaryTooltipData()
    assert(shown and shown.id == itemID, 'setter publishes supplied socketed item')
end
"#;

const INTERACTION_ASSERTIONS: &str = r#"
local first = C_TooltipInfo.GetItemInteractionItem()
assert(first and first.lines[1].leftText == 'First supplied cleansing row')
GameTooltip:SetItemInteractionItem()
local shown = GameTooltip:GetPrimaryTooltipData()
assert(shown and shown.lines[1].leftText == 'First supplied cleansing row')
"#;

const SECOND_INTERACTION_ASSERTIONS: &str = r#"
local second = C_TooltipInfo.GetItemInteractionItem()
assert(second and second.lines[1].leftText == 'Second supplied cleansing row')
GameTooltip:SetItemInteractionItem()
local shown = GameTooltip:GetPrimaryTooltipData()
assert(shown and shown.lines[1].leftText == 'Second supplied cleansing row')
"#;

const WEEKLY_ASSERTIONS: &str = r#"
A_Admin.SetWeeklyRewardsReady(true)
A_Admin.SetWeeklyRewardEntry(41, 'item:1728')
A_Admin.SetWeeklyRewardEntry(42, 'item:127830')
for _, case in ipairs({{41,1728}, {42,127830}}) do
    local direct = C_TooltipInfo.GetWeeklyReward(case[1])
    assert(direct and direct.id == case[2], 'getter selects supplied reward')
    GameTooltip:SetWeeklyReward(case[1])
    local shown = GameTooltip:GetPrimaryTooltipData()
    assert(shown and shown.id == case[2], 'setter publishes supplied reward')
end
"#;

const QUEST_SPECIAL_ASSERTIONS: &str = r#"
for _, case in ipairs({{1,990701,6948}, {2,990702,7005}}) do
    local direct = C_TooltipInfo.GetQuestLogSpecialItem(case[1])
    assert(direct and direct.id == case[3], 'getter selects supplied quest item')
    GameTooltip:SetQuestLogSpecialItem(case[1])
    local shown = GameTooltip:GetPrimaryTooltipData()
    assert(shown and shown.id == case[3], 'setter publishes supplied quest item')
end
"#;

const QUEST_PARTY_ASSERTIONS: &str = r#"
for _, case in ipairs({{990701,'First supplied objective'}, {990702,'Second supplied objective'}}) do
    local direct = C_TooltipInfo.GetQuestPartyProgress(case[1])
    assert(direct and direct.lines[1].leftText == case[2], 'getter selects supplied quest snapshot')
    GameTooltip:SetQuestPartyProgress(case[1])
    local shown = GameTooltip:GetPrimaryTooltipData()
    assert(shown and shown.lines[1].leftText == case[2], 'setter publishes supplied quest snapshot')
end
"#;

const RECIPE_REAGENT_ASSERTIONS: &str = r#"
local previous
for index = 1, 2 do
    local direct = C_TooltipInfo.GetRecipeReagentItem(100005, index)
    assert(direct and direct.id, 'getter publishes supplied recipe reagent')
    GameTooltip:SetRecipeReagentItem(100005, index)
    local shown = GameTooltip:GetPrimaryTooltipData()
    assert(shown and shown.id == direct.id, 'setter publishes getter item')
    assert(not previous or previous ~= shown.id, 'reagent entries must differ')
    previous = shown.id
end
"#;

const RECIPE_RANK_ASSERTIONS: &str = r#"
for _, recipeID in ipairs({990701,990711}) do
    local direct = C_TooltipInfo.GetRecipeRankInfo(recipeID, 1)
    assert(direct and direct.id == recipeID and #direct.lines > 0,
        'getter publishes selected supplied recipe rank')
    GameTooltip:SetRecipeRankInfo(recipeID, 1)
    local shown = GameTooltip:GetPrimaryTooltipData()
    assert(shown and shown.id == recipeID and #shown.lines > 0,
        'setter publishes selected supplied recipe rank')
end
"#;

const APPEARANCE_ASSERTIONS: &str = r#"
for _, case in ipairs({{90001,1728}, {90002,127830}}) do
    local direct = C_TooltipInfo.GetItemByItemModifiedAppearanceID(case[1])
    assert(direct and direct.id == case[2], 'getter selects supplied appearance item')
    GameTooltip:SetItemByItemModifiedAppearanceID(case[1])
    local shown = GameTooltip:GetPrimaryTooltipData()
    assert(shown and shown.id == case[2], 'setter publishes supplied appearance item')
end
"#;

const RELIC_ASSERTIONS: &str = r#"
for index, expected in ipairs({132300,130162}) do
    local direct = C_TooltipInfo.GetSocketedRelic(index)
    assert(direct and direct.id == expected, 'getter selects supplied viewed relic')
    GameTooltip:SetSocketedRelic(index)
    local shown = GameTooltip:GetPrimaryTooltipData()
    assert(shown and shown.id == expected, 'setter publishes supplied viewed relic')
end
"#;

fn item_env() -> wow_ui_sim::lua_api::WowLuaEnv {
    tooltip_full_env_helpers::load_full_env_without_post_load()
}

fn seed_item_interaction(env: &wow_ui_sim::lua_api::WowLuaEnv, text: &str) {
    let mut state = env.state().borrow_mut();
    state.item_interaction_tooltip_input = Some(ItemInteractionTooltipInput {
        object_kind: Some(2),
        selected_item_id: Some(6948),
        cleansing_rows: vec![ItemInteractionTooltipLine {
            left_text: text.to_string(),
            ..Default::default()
        }],
        ..Default::default()
    });
}

fn seed_quest_item(
    env: &wow_ui_sim::lua_api::WowLuaEnv,
    log_index: i32,
    quest_id: i32,
    item_id: u32,
    slot: i32,
) {
    seed_quest_item_cache(env, log_index, quest_id, item_id, slot);
    seed_quest_bag_item(env, item_id, slot);
}

fn seed_quest_item_cache(
    env: &wow_ui_sim::lua_api::WowLuaEnv,
    log_index: i32,
    quest_id: i32,
    item_id: u32,
    slot: i32,
) {
    let mut state = env.state().borrow_mut();
    let inputs = &mut state.quest_log_entries.quest_special_item_inputs;
    inputs.quest_ids_by_log_index.insert(log_index, quest_id);
    inputs.cached_quest_records.insert(
        quest_id,
        QuestSpecialItemCacheRecord {
            special_item_id: Some(item_id),
            ..Default::default()
        },
    );
    inputs.selected_item_objects.insert(
        quest_id,
        QuestSpecialItemObject {
            item_id,
            bag: 0,
            slot,
        },
    );
}

fn seed_quest_bag_item(env: &wow_ui_sim::lua_api::WowLuaEnv, item_id: u32, slot: i32) {
    env.state().borrow_mut().bag_items.insert(
        (0, slot),
        BagItem {
            owned_identity: None,
            item_id,
            stack_count: 1,
            hyperlink: None,
            durability: None,
            applied_transmog: None,
        },
    );
}

fn seed_party_snapshot(env: &wow_ui_sim::lua_api::WowLuaEnv, quest_id: i32, text: &str) {
    let snapshot = QuestPartyProgressSnapshot {
        title: Some(QuestPartyProgressText {
            text: text.into(),
            color: (1.0, 0.0, 0.0),
        }),
        participants: vec![party_participant(text)],
    };
    env.state()
        .borrow_mut()
        .quest_log_entries
        .party_progress
        .insert(quest_id, snapshot);
}

fn party_participant(text: &str) -> QuestPartyProgressParticipant {
    let heading = QuestPartyProgressHeading {
        name: "Member".into(),
        guid: "Player-1".into(),
        color: (0.2, 0.8, 0.3),
    };
    let row = QuestPartyProgressRow {
        text: text.into(),
        color: (0.2, 0.8, 0.3),
        completed: false,
        num_required: Some(2),
        num_fulfilled: Some(1),
        progress_bar_percent: Some(50.0),
    };
    QuestPartyProgressParticipant {
        heading: Some(heading),
        exclude_when_ignoring_active_player: false,
        rows: vec![row],
    }
}

fn seed_recipe_rank(env: &wow_ui_sim::lua_api::WowLuaEnv, recipe_id: i32, spell_id: u32) {
    env.state().borrow_mut().recipe_rank_tooltip_cache.insert(
        recipe_id,
        RecipeRankTooltipRecord {
            parent_recipe_id: 0,
            next_recipe_id: 0,
            spell_id,
            label: None,
            next_rank_source: None,
        },
    );
}

fn seed_appearance_sources(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::WowForever {
        env.state()
            .borrow_mut()
            .forever_appearance_item_ids
            .extend([(90001, 1728), (90002, 127830)]);
        return;
    }
    for (source_id, item_id) in [(90001, 1728), (90002, 127830)] {
        env.state().borrow_mut().transmog_appearance_sources.insert(
            source_id,
            AppearanceSourceInfo {
                category: 1,
                item_appearance_id: 0,
                can_have_illusion: false,
                icon: 0,
                is_collected: false,
                item_link: format!("item:{item_id}"),
                transmoglink: String::new(),
                source_type: None,
                item_subclass: 0,
                ignore_model_attachment_checks_for_illusion: false,
            },
        );
    }
}

fn artifact_info() -> ArtifactInfo {
    ArtifactInfo {
        item_id: 128910,
        alt_item_id: 0,
        name: "Strom'kar, the Warbreaker".into(),
        icon: String::new(),
        total_xp: 0,
        points_spent: 0,
        quality: 6,
        artifact_appearance_id: 0,
        appearance_mod_id: 0,
        item_appearance_id: 0,
        alt_item_appearance_id: 0,
        alt_on_top: false,
        tier: 0,
        maxed: false,
        disabled: false,
        category: 1,
    }
}

fn relic_slot(item_id: u32) -> RelicSlotInfo {
    RelicSlotInfo {
        slot_type: "Iron".into(),
        locked_reason: None,
        name: format!("Relic {item_id}"),
        icon: String::new(),
        link: format!("item:{item_id}"),
        socket: Some(wow_ui_sim::c_api::item_descriptor::ItemDescriptor {
            item_id,
            bonus_context: None,
            modifiers: Vec::new(),
        }),
    }
}

#[test]
fn selected_socket_getters_publish_distinct_supplied_items() {
    let env = item_env();
    env.exec(SOCKET_ASSERTIONS).unwrap();
}

#[test]
fn selected_item_interaction_getter_publishes_two_supplied_rows() {
    let env = item_env();
    seed_item_interaction(&env, "First supplied cleansing row");
    env.exec(INTERACTION_ASSERTIONS).unwrap();
    seed_item_interaction(&env, "Second supplied cleansing row");
    env.exec(SECOND_INTERACTION_ASSERTIONS).unwrap();
}

#[test]
fn selected_weekly_reward_getter_publishes_two_supplied_items() {
    let env = item_env();
    env.exec(WEEKLY_ASSERTIONS).unwrap();
}

#[test]
fn selected_quest_special_item_getter_publishes_two_supplied_items() {
    let env = item_env();
    seed_quest_item(&env, 1, 990701, 6948, 1);
    seed_quest_item(&env, 2, 990702, 7005, 2);
    env.exec(QUEST_SPECIAL_ASSERTIONS).unwrap();
}

#[test]
fn selected_quest_party_getter_publishes_two_supplied_snapshots() {
    let env = item_env();
    env.exec("A_Admin.SetPartySize(1)").unwrap();
    seed_party_snapshot(&env, 990701, "First supplied objective");
    seed_party_snapshot(&env, 990702, "Second supplied objective");
    env.exec(QUEST_PARTY_ASSERTIONS).unwrap();
}

#[test]
fn selected_recipe_reagent_getter_publishes_two_supplied_entries() {
    let env = item_env();
    env.exec(RECIPE_REAGENT_ASSERTIONS).unwrap();
}

#[test]
fn selected_recipe_rank_getter_publishes_two_supplied_recipes() {
    let env = item_env();
    seed_recipe_rank(&env, 990701, 19750);
    seed_recipe_rank(&env, 990711, 116);
    env.exec(RECIPE_RANK_ASSERTIONS).unwrap();
}

#[test]
fn selected_appearance_getter_publishes_two_supplied_items() {
    let env = item_env();
    seed_appearance_sources(&env);
    env.exec(APPEARANCE_ASSERTIONS).unwrap();
}

#[test]
fn selected_socketed_relic_getter_publishes_two_viewed_slots() {
    let env = item_env();
    let mut state = env.state().borrow_mut();
    state.viewed_artifact.info = Some(artifact_info());
    state
        .viewed_artifact
        .relic_slots
        .extend([relic_slot(132300), relic_slot(130162)]);
    drop(state);
    env.exec(RELIC_ASSERTIONS).unwrap();
}
