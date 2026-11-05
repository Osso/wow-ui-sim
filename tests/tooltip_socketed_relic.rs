//! Original artifact-title hover reads the viewed artifact's socketed relic.
//!
//! Native behaviour: WM97 DQ4 and WM104 (Retail 12.1.5). Catalog and relic rows
//! are the pinned 70391 exports.

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::c_api::item_descriptor::{ItemBonusContext, ItemDescriptor};
use wow_ui_sim::lua_api::state::{ArtifactInfo, RelicSlotInfo};

/// Strom'kar: ItemSparse.ArtifactID 36 with two nonzero socket markers.
const ARTIFACT_ITEM: i32 = 128_910;
/// GemProperties 2603: Type 8192 (Life), effect 11 argument 708.
const LIFE_RELIC: u32 = 132_300;
/// GemProperties 2550: Type 14 has no native token, effect 11 argument 0.
const UNKNOWN_TYPE_RELIC: u32 = 130_162;
/// Bonus list 708 is one Type 1 row with Value_0 2.
const LEVEL_BONUS_LIST: u32 = 708;

fn seeded_artifact() -> ArtifactInfo {
    ArtifactInfo {
        item_id: ARTIFACT_ITEM,
        alt_item_id: 0,
        name: "Strom'kar, the Warbreaker".to_string(),
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

fn socket(item_id: u32, level_bonus: Option<u32>) -> RelicSlotInfo {
    RelicSlotInfo {
        slot_type: "Iron".to_string(),
        locked_reason: None,
        name: format!("Relic {item_id}"),
        icon: String::new(),
        link: format!("item:{item_id}"),
        socket: Some(ItemDescriptor {
            item_id,
            bonus_context: level_bonus.map(|list_id| ItemBonusContext {
                creation_context: 0,
                bonus_list_ids: vec![list_id],
            }),
            modifiers: Vec::new(),
        }),
    }
}

fn load_artifact_ui(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let addons = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let toc = wow_ui_sim::loader::find_toc_file(&addons.join("Blizzard_ArtifactUI")).unwrap();
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
}

crate::retail_relic_record_fixture! {
#[test]
fn tooltip_socketed_relic_original_hover_selects_viewed_slot() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    load_artifact_ui(&env);
    {
        let mut state = env.state().borrow_mut();
        state.viewed_artifact.info = Some(seeded_artifact());
        state
            .viewed_artifact
            .relic_slots
            .push(socket(LIFE_RELIC, Some(LEVEL_BONUS_LIST)));
        state
            .viewed_artifact
            .relic_slots
            .push(socket(UNKNOWN_TYPE_RELIC, None));
    }
    env.exec(
        r#"
        assert(ArtifactTitleTemplateMixin and ArtifactPerksMixin, 'original artifact consumers must load')
        local perks = CreateFrame('Frame', nil, UIParent)
        Mixin(perks, ArtifactPerksMixin)
        local title = CreateFrame('Frame', nil, perks)
        Mixin(title, ArtifactTitleTemplateMixin)
        local relic = CreateFrame('Frame', nil, title)

        -- Two socket markers: the third internal slot does not exist.
        GameTooltip:ClearHandlerInfo()
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        GameTooltip:SetSocketedRelic(3)
        assert(GameTooltip:GetPrimaryTooltipData() == nil, 'out-of-range slot must not select a relic')
        GameTooltip:SetSocketedRelic(0)
        assert(GameTooltip:GetPrimaryTooltipData() == nil, 'zero slot must not select the first relic')

        local function hover(index)
            local _, _, slotType, link = C_ArtifactUI.GetRelicInfo(index)
            relic.relicSlotIndex = index
            relic.relicType = slotType
            relic.relicLink = link
            title:OnRelicSlotMouseEnter(relic)
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(info and info.getterName == 'GetSocketedRelic', 'socketed-relic accessor missing')
            return data
        end

        local function lineTexts(data)
            local texts = {}
            for _, line in ipairs(data.lines) do
                texts[#texts + 1] = line.leftText
            end
            return texts
        end

        local function hasLine(data, expected)
            for _, text in ipairs(lineTexts(data)) do
                if text == expected then
                    return true
                end
            end
            return false
        end

        local function hasIncreaseNumber(data)
            for _, text in ipairs(lineTexts(data)) do
                if type(text) == 'string' and text:find('Item Levels', 1, true) then
                    return true
                end
            end
            return false
        end

        local lifeData = hover(1)
        assert(lifeData.id == 132300, 'must use the first viewed socket descriptor')
        assert(hasLine(lifeData, 'Life Artifact Relic'), 'Life relic type line missing')
        -- 70391 bonus list 708 is a Type 1 row only, so the published number stays 0 (WM104).
        assert(not hasIncreaseNumber(lifeData), 'Type 1 rows must not publish an item-level number')
        -- The socket descriptor carries bonus list 708: level 44 catalog + 2.
        assert(hasLine(lifeData, 'Item Level 46'), 'socket bonus list must reach the item level')
        local catalog = C_TooltipInfo.GetItemByID(132300)
        assert(catalog.lines[2].leftText == 'Item Level 44', 'catalog level is the unbonused baseline')
        assert(lifeData.lines[1].leftText == catalog.lines[1].leftText, 'must retain the selected relic title')
        assert(GameTooltip:GetLeftLine(1):GetText() == lifeData.lines[1].leftText)

        local unknownData = hover(2)
        assert(unknownData.id == 130162, 'must use the second viewed socket descriptor')
        assert(hasLine(unknownData, 'Unknown Artifact Relic'), 'unrecognized exact type must use the Unknown token')
        assert(not hasIncreaseNumber(unknownData), 'zero effect argument contributes no number')
        "#,
    )
    .unwrap();
}
}

crate::socketed_relic_getter_fixture! {
#[test]
fn tooltip_socketed_relic_without_descriptor_is_a_miss() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    load_artifact_ui(&env);
    {
        let mut state = env.state().borrow_mut();
        state.viewed_artifact.info = Some(seeded_artifact());
        let mut slot = socket(LIFE_RELIC, None);
        slot.socket = None;
        state.viewed_artifact.relic_slots.push(slot);
    }
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        GameTooltip:SetSocketedRelic(1)
        assert(GameTooltip:GetPrimaryTooltipData() == nil, 'descriptor-free socket must not invent a catalog item')
        "#,
    )
    .unwrap();

    // A viewed artifact without an artifact predicate cannot expose sockets.
    {
        let mut state = env.state().borrow_mut();
        state.viewed_artifact.info = None;
        state
            .viewed_artifact
            .relic_slots
            .push(socket(LIFE_RELIC, None));
    }
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        GameTooltip:SetSocketedRelic(1)
        assert(GameTooltip:GetPrimaryTooltipData() == nil, 'missing viewed artifact must not select a relic')
        "#,
    )
    .unwrap();
}
}

crate::forever_catalog_relic_fixture! {
#[test]
fn tooltip_socketed_relic_forever_keeps_the_catalog_item() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    load_artifact_ui(&env);
    // Forever has no viewed-artifact state in play, so the shared miss path runs.
    env.exec(
        r#"
        GameTooltip:ClearHandlerInfo()
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        GameTooltip:SetSocketedRelic(1)
        assert(GameTooltip:GetPrimaryTooltipData() == nil, 'Forever without a viewed artifact must miss')
        "#,
    )
    .unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.viewed_artifact.info = Some(seeded_artifact());
        state
            .viewed_artifact
            .relic_slots
            .push(socket(LIFE_RELIC, Some(LEVEL_BONUS_LIST)));
    }
    // A supplied socket resolves the shared catalog item; Retail 70391 relic and
    // bonus records must not reach this profile.
    env.exec(
        r#"
        local perks = CreateFrame('Frame', nil, UIParent)
        Mixin(perks, ArtifactPerksMixin)
        local title = CreateFrame('Frame', nil, perks)
        Mixin(title, ArtifactTitleTemplateMixin)
        local relic = CreateFrame('Frame', nil, title)
        local _, _, slotType, link = C_ArtifactUI.GetRelicInfo(1)
        relic.relicSlotIndex = 1
        relic.relicType = slotType
        relic.relicLink = link
        title:OnRelicSlotMouseEnter(relic)
        local info = GameTooltip:GetPrimaryTooltipInfo()
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(info and info.getterName == 'GetSocketedRelic', 'socketed-relic accessor missing')
        assert(data and data.id == 132300, 'must resolve the socket descriptor item ID')
        assert(data.lines[2].leftText == 'Item Level 44', 'Forever keeps the catalog level; Retail bonus 708 must not apply')
        for _, line in ipairs(data.lines) do
            local text = line.leftText
            assert(not (type(text) == 'string' and text:find('Artifact Relic', 1, true)),
                'Retail relic type records must not reach Forever')
            assert(not (type(text) == 'string' and text:find('Item Levels', 1, true)),
                'Retail item-level contribution records must not reach Forever')
        end
        "#,
    )
    .unwrap();
}
}
