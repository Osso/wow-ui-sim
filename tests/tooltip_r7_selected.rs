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
        .enumerate()
        .map(|(index, text)| ItemInteractionTooltipLine {
            left_text: text.to_string(),
            left_color: (index == 1).then_some((0.12, 1.0, 0.0)),
            ..Default::default()
        })
        .collect()
}

fn load_item_interaction_env() -> WowLuaEnv {
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
    env
}

fn load_challenges_env() -> WowLuaEnv {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    for addon in ["Blizzard_GroupFinder", "Blizzard_ChallengesUI"] {
        let (loaded, reason): (bool, Option<String>) = env
            .eval(&format!("return C_AddOns.LoadAddOn('{addon}')"))
            .unwrap();
        assert!(loaded, "{addon} load failed: {reason:?}");
    }
    env.exec(
        r##"
        assert(PVEFrame, "selected GroupFinder path creates PVEFrame")
        assert(C_AddOns.IsAddOnLoaded("Blizzard_ChallengesUI"))
        assert(debug.getinfo(ChallengesKeystoneSlotMixin.OnEnter, "S").source
            :find("Blizzard_ChallengesUI.lua", 1, true))
        assert(debug.getinfo(GameTooltip.SetSlottedKeystone, "S").source
            :find("TooltipDataHandler.lua", 1, true))
        R7ChallengesKeystoneSlot = CreateFrame("Button", nil, UIParent)
        R7KeystoneSlottedItems = {}
        R7KeystoneEventProbe = CreateFrame("Frame")
        R7KeystoneEventProbe:RegisterEvent("CHALLENGE_MODE_KEYSTONE_SLOTTED")
        R7KeystoneEventProbe:SetScript("OnEvent", function(_, _, itemID)
            local data = C_TooltipInfo.GetSlottedKeystone()
            assert(C_ChallengeMode.HasSlottedKeystone())
            assert(data and data.id == itemID)
            table.insert(R7KeystoneSlottedItems, itemID)
        end)
    "##,
    )
    .unwrap();
    env
}

fn slot_keystone_from_bag(env: &WowLuaEnv, slot: u32, item_id: u32) -> String {
    let guid = add_bag_item(env, 0, slot, item_id);
    env.exec(&format!(
        "C_Container.PickupContainerItem(0, {slot}); assert(CursorHasItem()); C_ChallengeMode.SlotKeystone()"
    ))
    .unwrap();
    guid
}

fn assert_challenges_hover_hidden(env: &WowLuaEnv) {
    env.exec(
        r#"
        GameTooltip:Hide()
        ChallengesKeystoneSlotMixin.OnEnter(R7ChallengesKeystoneSlot)
        assert(not GameTooltip:IsShown())
    "#,
    )
    .unwrap();
}

fn assert_challenges_hover_item(env: &WowLuaEnv, item_id: u32, guid: &str, slot_index: usize) {
    env.exec(&format!(
        r#"
        local apiData = assert(C_TooltipInfo.GetSlottedKeystone(), "keystone API tooltip missing")
        assert(apiData.type == 0 and apiData.id == {item_id}, "API item identity")
        assert(apiData.guid == "{guid}", "API instance GUID")
        GameTooltip:Hide()
        ChallengesKeystoneSlotMixin.OnEnter(R7ChallengesKeystoneSlot)
        assert(GameTooltip:IsShown(), "original ChallengesUI hover shows the selected keystone")
        local data = assert(GameTooltip:GetPrimaryTooltipData(), "ChallengesUI tooltip missing")
        assert(data.type == 0 and data.id == {item_id}, "ChallengesUI item identity")
        assert(data.guid == "{guid}", "ChallengesUI instance GUID")
        assert(R7KeystoneSlottedItems[{slot_index}] == {item_id}, "slotted event payload")
    "#
    ))
    .unwrap();
}

#[test]
fn item_interaction_cleansing_original_consumer_uses_distinct_supplied_rows() {
    let env = load_item_interaction_env();
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
        assert(data.lines[1].leftColor == nil, "omitted host color stays absent")
        assert(data.lines[2].leftText == "Fixture cleansing row A")
        assert(math.abs(data.lines[2].leftColor.g - 1) < 0.001)
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
        assert(data.lines[1].leftColor == nil, "omitted host color stays absent")
        assert(data.lines[2].leftText == "Fixture cleansing row B")
        assert(math.abs(data.lines[2].leftColor.g - 1) < 0.001)
    "#,
    )
    .unwrap();
}

#[test]
fn item_interaction_conversion_original_consumer_uses_distinct_output_ids() {
    let env = load_item_interaction_env();
    let selected_guid = add_bag_item(&env, 0, 3, 128_910);

    for output_item_id in [1728, 231_268] {
        set_interaction_input(&env, 4, &selected_guid, Some(output_item_id), Vec::new());
        env.exec(&format!(
            r#"
            ItemInteractionFrame.interactionType = Enum.UIItemInteractionType.ItemConversion
            ItemInteractionFrame.itemLocation = {{}}
            ItemInteractionItemConversionOutputSlotMixin.OnEnter(
                ItemInteractionFrame.ItemConversionFrame.ItemConversionOutputSlot)
            local data = assert(GameTooltip:GetPrimaryTooltipData(), "conversion tooltip missing")
            assert(data.type == 0 and data.id == {output_item_id}, "conversion output identity")
        "#
        ))
        .unwrap();
    }
}

#[test]
fn zero_conversion_output_preserves_the_unmodeled_native_gap() {
    // PRESERVATION only: native unresolved-item payload is UNKNOWN pending R88o.2 q1.
    let env = load_item_interaction_env();
    let selected_guid = add_bag_item(&env, 0, 3, 128_910);
    set_interaction_input(&env, 4, &selected_guid, Some(0), Vec::new());
    env.exec(r##"assert(select("#", C_TooltipInfo.GetItemInteractionItem()) == 0)"##)
        .expect("unverified unresolved-item content must not publish a fabricated tooltip");
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
            PossessButtonMixin.OnEnter(button)
            local data = assert(GameTooltip:GetPrimaryTooltipData(), "possession tooltip missing")
            assert(data.type == 1 and data.id == {spell_id}, "possession spell identity")
        "#
        ))
        .unwrap();
    }
}

#[test]
fn challenges_ui_hover_uses_public_keystone_selection() {
    let env = load_challenges_env();
    env.exec("A_Admin.ClearBags(); C_ChallengeMode.ClearKeystone()")
        .expect("empty challenge selection fixture");
    assert_challenges_hover_hidden(&env);
    env.exec(
        r##"
        local function capture(...) return {n = select("#", ...), ...} end
        local result = capture(C_ChallengeMode.HasSlottedKeystone("ignored", 17))
        assert(result.n == 1 and type(result[1]) == "boolean" and not result[1])
    "##,
    )
    .unwrap();

    let first_guid = slot_keystone_from_bag(&env, 3, 180_653);
    assert_challenges_hover_item(&env, 180_653, &first_guid, 1);
    env.exec("C_ChallengeMode.ClearKeystone()")
        .expect("clear first selected keystone");
    env.exec(r#"assert(not C_ChallengeMode.HasSlottedKeystone())"#)
        .unwrap();
    assert_challenges_hover_hidden(&env);

    let second_guid = slot_keystone_from_bag(&env, 4, 158_923);
    assert_challenges_hover_item(&env, 158_923, &second_guid, 2);
    env.exec(
        r##"
        local function capture(...) return {n = select("#", ...), ...} end
        local result = capture(C_ChallengeMode.RemoveKeystone())
        assert(result.n == 1 and type(result[1]) == "boolean")
        assert(not C_ChallengeMode.HasSlottedKeystone())
    "##,
    )
    .unwrap();
    assert_challenges_hover_hidden(&env);
}

#[test]
fn has_slotted_keystone_tracks_guid_without_resolving_the_item() {
    let env = load_challenges_env();
    env.exec("A_Admin.ClearBags(); C_ChallengeMode.ClearKeystone()")
        .expect("unresolved item fixture starts empty");
    env.exec(
        r##"
        local function capture(...) return {n = select("#", ...), ...} end
        local result = capture(C_ChallengeMode.HasSlottedKeystone("ignored", 17))
        assert(result.n == 1 and type(result[1]) == "boolean" and not result[1])
    "##,
    )
    .unwrap();

    slot_keystone_from_bag(&env, 3, 180_653);
    env.exec(
        r##"
        local function capture(...) return {n = select("#", ...), ...} end
        local result = capture(C_ChallengeMode.HasSlottedKeystone("ignored", 17))
        assert(result.n == 1 and type(result[1]) == "boolean" and result[1])
        A_Admin.ClearBags()
        assert(C_ChallengeMode.HasSlottedKeystone())
        assert(select("#", C_TooltipInfo.GetSlottedKeystone()) == 0)
    "##,
    )
    .unwrap();
    env.exec("C_ChallengeMode.ClearKeystone()")
        .expect("existing selection clear should execute");
    env.exec("assert(not C_ChallengeMode.HasSlottedKeystone())")
        .expect("cleared selection should be absent");
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
    "##,
    )
    .unwrap();
    env.state().borrow_mut().possession_spell_id = Some(0);
    env.exec(r##"assert(select("#", C_TooltipInfo.GetPossession(1)) == 0)"##)
        .unwrap();
    env.state().borrow_mut().possession_spell_id = None;

    {
        let mut sim = env.state().borrow_mut();
        sim.item_interaction = Some(ItemInteractionInfo {
            interaction_type: 2,
            ..Default::default()
        });
        sim.item_interaction_tooltip_input = Some(ItemInteractionTooltipInput {
            selected_item_guid: Some("Item-0-1-1".to_string()),
            ..Default::default()
        });
    }
    env.exec(r##"assert(select("#", C_TooltipInfo.GetItemInteractionItem()) == 0)"##)
        .unwrap();
    env.state().borrow_mut().item_interaction_tooltip_input = None;
    env.exec(r##"assert(select("#", C_TooltipInfo.GetItemInteractionItem()) == 0)"##)
        .unwrap();
}

#[test]
fn unsupported_item_interaction_kind_requires_resolved_item() {
    let env = load_item_interaction_env();
    env.exec("A_Admin.ClearBags()")
        .expect("absent and unresolved fixtures start without bag items");
    {
        let mut sim = env.state().borrow_mut();
        sim.item_interaction = Some(ItemInteractionInfo {
            interaction_type: 1,
            ..Default::default()
        });
        sim.item_interaction_tooltip_input = Some(ItemInteractionTooltipInput::default());
    }
    env.exec(r##"assert(select("#", C_TooltipInfo.GetItemInteractionItem()) == 0)"##)
        .expect("unsupported kind with no selected object returns no values");

    {
        let mut sim = env.state().borrow_mut();
        sim.item_interaction_tooltip_input = Some(ItemInteractionTooltipInput {
            selected_item_guid: Some("Item-0-3-1728".to_string()),
            ..Default::default()
        });
    }
    env.exec(r##"assert(select("#", C_TooltipInfo.GetItemInteractionItem()) == 0)"##)
        .expect("unsupported kind with unresolved selected GUID returns no values");

    let selected_guid = add_bag_item(&env, 0, 3, 1728);
    env.state().borrow_mut().item_interaction_tooltip_input = Some(ItemInteractionTooltipInput {
        selected_item_guid: Some(selected_guid),
        ..Default::default()
    });
    env.exec(
        r##"
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
    .expect("existing unsupported-kind object preserves its exact native result shape");
}
