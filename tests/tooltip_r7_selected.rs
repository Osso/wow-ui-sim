#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::c_api::{
    ItemInteractionTooltipInput, ItemInteractionTooltipLine,
    c_item_interaction::ItemInteractionInfo,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn add_bag_item(env: &WowLuaEnv, bag: u32, slot: u32, item_id: u32) -> String {
    env.exec(&format!("A_Admin.AddBagItem({bag}, {slot}, {item_id}, 1)"))
        .unwrap();
    env.eval(&format!(
        "return C_Item.GetItemGUID({{bagID={bag}, slotIndex={slot}}})"
    ))
    .unwrap()
}

fn set_interaction_input(
    env: &WowLuaEnv,
    interaction_type: i32,
    selected_item_guid: &str,
    conversion_output_item_id: Option<u32>,
    cleansing_rows: Vec<ItemInteractionTooltipLine>,
) {
    let mut sim = env.state().borrow_mut();
    sim.item_interaction = Some(ItemInteractionInfo {
        interaction_type,
        ..Default::default()
    });
    sim.item_interaction_tooltip_input = Some(ItemInteractionTooltipInput {
        selected_item_guid: Some(selected_item_guid.to_string()),
        conversion_output_item_id,
        cleansing_rows,
    });
}

fn cleansing_rows(title: &str, detail: &str) -> Vec<ItemInteractionTooltipLine> {
    [title, detail]
        .into_iter()
        .map(|text| ItemInteractionTooltipLine {
            left_text: text.to_string(),
            ..Default::default()
        })
        .collect()
}

#[test]
fn item_interaction_original_consumers_use_supplied_kind_payloads() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    let (loaded, reason): (bool, Option<String>) = env
        .eval(r#"return C_AddOns.LoadAddOn("Blizzard_ItemInteractionUI")"#)
        .unwrap();
    assert!(loaded, "Blizzard_ItemInteractionUI load failed: {reason:?}");
    env.exec(
        r#"
        assert(C_AddOns.IsAddOnLoaded("Blizzard_ItemInteractionUI"))
        assert(debug.getinfo(ItemInteractionMixin.SetInputItemSlotTooltip, "S").source
            :find("Blizzard_ItemInteractionUI.lua", 1, true))
        assert(debug.getinfo(ItemInteractionItemConversionOutputSlotMixin.OnEnter, "S").source
            :find("Blizzard_ItemInteractionUI.lua", 1, true))
    "#,
    )
    .unwrap();

    let selected_guid = add_bag_item(&env, 0, 3, 128_910);
    set_interaction_input(
        &env,
        2,
        &selected_guid,
        None,
        cleansing_rows("Fixture title A", "Fixture cleansing row A"),
    );
    env.exec(
        r#"
        ItemInteractionFrame.interactionType = Enum.UIItemInteractionType.CleanseCorruption
        ItemInteractionFrame:SetInputItemSlotTooltip(ItemInteractionFrame.ItemSlot, {})
        local data = assert(GameTooltip:GetPrimaryTooltipData(), "cleansing tooltip missing")
        assert(data.type == 20 and data.id == nil, "verified cleansing identity")
        assert(data.lines[1].leftText == "Fixture title A")
        assert(data.lines[2].leftText == "Fixture cleansing row A")
    "#,
    )
    .unwrap();

    set_interaction_input(
        &env,
        2,
        &selected_guid,
        None,
        cleansing_rows("Fixture title B", "Fixture cleansing row B"),
    );
    env.exec(
        r#"
        ItemInteractionFrame:SetInputItemSlotTooltip(ItemInteractionFrame.ItemSlot, {})
        local data = assert(GameTooltip:GetPrimaryTooltipData(), "second cleansing tooltip missing")
        assert(data.type == 20 and data.id == nil)
        assert(data.lines[1].leftText == "Fixture title B")
        assert(data.lines[2].leftText == "Fixture cleansing row B")
    "#,
    )
    .unwrap();

    for output_item_id in [1728, 231_268] {
        set_interaction_input(&env, 4, &selected_guid, Some(output_item_id), Vec::new());
        env.exec(&format!(
            r#"
            ItemInteractionFrame.interactionType = Enum.UIItemInteractionType.ItemConversion
            ItemInteractionFrame.itemLocation = {{}}
            ItemInteractionItemConversionOutputSlotMixin:OnEnter(
                ItemInteractionFrame.ItemConversionFrame.ItemConversionOutputSlot)
            local data = assert(GameTooltip:GetPrimaryTooltipData(), "conversion tooltip missing")
            assert(data.type == 0 and data.id == {output_item_id}, "conversion output identity")
        "#
        ))
        .unwrap();
    }
}

#[test]
fn possession_original_action_bar_hover_uses_the_supplied_spell() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.exec(
        r#"
        assert(C_AddOns.IsAddOnLoaded("Blizzard_ActionBar"))
        assert(debug.getinfo(PossessButtonMixin.OnEnter, "S").source
            :find("PossessActionBar.lua", 1, true))
    "#,
    )
    .unwrap();

    for spell_id in [116, 133] {
        env.state().borrow_mut().possession_spell_id = Some(spell_id);
        env.exec(&format!(
            r#"
            local button = CreateFrame("Button", nil, UIParent)
            button:SetID(1)
            PossessButtonMixin:OnEnter(button)
            local data = assert(GameTooltip:GetPrimaryTooltipData(), "possession tooltip missing")
            assert(data.type == 1 and data.id == {spell_id}, "possession spell identity")
        "#
        ))
        .unwrap();
    }
}

#[test]
fn slotted_keystone_original_challenges_hover_uses_the_supplied_guid() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    let (loaded, reason): (bool, Option<String>) = env
        .eval(r#"return C_AddOns.LoadAddOn("Blizzard_ChallengesUI")"#)
        .unwrap();
    assert!(loaded, "Blizzard_ChallengesUI load failed: {reason:?}");
    env.exec(
        r#"
        assert(C_AddOns.IsAddOnLoaded("Blizzard_ChallengesUI"))
        assert(debug.getinfo(ChallengesKeystoneSlotMixin.OnEnter, "S").source
            :find("Blizzard_ChallengesUI.lua", 1, true))
        C_ChallengeMode = C_ChallengeMode or {}
        C_ChallengeMode.HasSlottedKeystone = function() return true end
    "#,
    )
    .unwrap();

    for (slot, item_id) in [(3, 1728), (4, 231_268)] {
        let guid = add_bag_item(&env, 0, slot, item_id);
        env.state().borrow_mut().slotted_keystone_guid = Some(guid.clone());
        env.exec(&format!(
            r#"
            local button = CreateFrame("Button", nil, UIParent)
            ChallengesKeystoneSlotMixin:OnEnter(button)
            local data = assert(GameTooltip:GetPrimaryTooltipData(), "keystone tooltip missing")
            assert(data.type == 0 and data.id == {item_id}, "slotted item identity")
            assert(data.guid == "{guid}", "selected instance GUID")
        "#
        ))
        .unwrap();
    }
}

#[test]
fn round7_selected_getter_misses_keep_the_verified_result_arities() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r##"
        assert(select("#", C_TooltipInfo.GetItemInteractionItem()) == 0)
        assert(select("#", C_TooltipInfo.GetPossession(1)) == 0)
        assert(select("#", C_TooltipInfo.GetPossession(2)) == 0)
        assert(select("#", C_TooltipInfo.GetSlottedKeystone()) == 0)
    "#)
    .unwrap();

    {
        let mut sim = env.state().borrow_mut();
        sim.item_interaction = Some(ItemInteractionInfo {
            interaction_type: 1,
            ..Default::default()
        });
        sim.item_interaction_tooltip_input = Some(ItemInteractionTooltipInput::default());
    }
    env.exec(r#"
        local count, data = select("#", C_TooltipInfo.GetItemInteractionItem()),
            C_TooltipInfo.GetItemInteractionItem()
        assert(count == 1 and type(data) == "table")
        assert(type(data.lines) == "table" and next(data.lines) == nil)
        assert(data.dataInstanceID == nil)
        local fields = 0
        for key in pairs(data) do
            fields = fields + 1
            assert(key == "lines", "unsupported-kind result has only lines")
        end
        assert(fields == 1, "unsupported-kind result has exactly one field")
    "##,
    )
    .unwrap();
}
