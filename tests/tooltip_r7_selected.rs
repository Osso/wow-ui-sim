#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::c_api::{
    ItemInteractionTooltipInput, ItemInteractionTooltipLine, c_challenge_mode::KeystoneLocation,
    c_item_interaction::ItemInteractionInfo,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{CursorInfo, CursorItemOrigin};

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
    load_challenges_ui(&env);
    env
}

fn load_challenges_ui(env: &WowLuaEnv) {
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
    "##,
    )
    .unwrap();
}

fn register_keystone_event_probe(env: &WowLuaEnv) {
    env.exec(
        r##"
        R7KeystoneSlottedState = {}
        R7KeystoneEventProbe = CreateFrame("Frame")
        R7KeystoneEventProbe:RegisterEvent("CHALLENGE_MODE_KEYSTONE_SLOTTED")
        R7KeystoneEventProbe:SetScript("OnEvent", function(_, _, itemID)
            local data = C_TooltipInfo.GetSlottedKeystone()
            assert(C_ChallengeMode.HasSlottedKeystone())
            assert(data and data.id == itemID)
            table.insert(R7KeystoneSlottedState, {
                eventItemID = itemID,
                tooltipItemID = data.id,
                guid = data.guid,
            })
        end)
    "##,
    )
    .unwrap();
}

// INFERRED fixture: this bag cursor stands in for native item/source GUID fields.
fn slot_keystone_from_bag(env: &WowLuaEnv, slot: u32, item_id: u32) -> String {
    let guid = add_bag_item(env, 0, slot, item_id);
    {
        let mut sim = env.state().borrow_mut();
        sim.keystone.item_modifiers.insert(
            KeystoneLocation::Bag(0, slot as i32),
            (item_id, vec![(17, 2)]),
        );
        sim.keystone.current_map_selector = Some(960);
    }
    env.exec(&format!(
        r##"
        C_Container.PickupContainerItem(0, {slot})
        assert(CursorHasItem())
        local count = select("#", C_ChallengeMode.SlotKeystone())
        assert(count == 0, "SlotKeystone returns no Lua values")
        assert(not CursorHasItem(), "SlotKeystone clears the cursor")
        assert(C_Item.GetItemGUID({{bagID = 0, slotIndex = {slot}}}) == nil,
            "cursor clear does not restore the source item")
        "##
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

fn assert_challenges_hover_item(env: &WowLuaEnv, item_id: u32, guid: &str) {
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
fn zero_conversion_output_publishes_verified_unresolved_item_shape() {
    let env = load_item_interaction_env();
    let selected_guid = add_bag_item(&env, 0, 3, 128_910);
    set_interaction_input(&env, 4, &selected_guid, Some(0), Vec::new());
    env.exec(
        r##"
        local function capture(...) return {n = select("#", ...), ...} end
        local function assertPayload(expectedText)
            local result = capture(C_TooltipInfo.GetItemInteractionItem())
            assert(
                result.n == 1 and type(result[1]) == "table",
                "zero conversion output must publish one TooltipData table"
            )
            local data = result[1]
            assert(type(data.dataInstanceID) == "number", "result must carry dataInstanceID")
            assert(data.id == nil and data.type == nil, "result omits top-level id and type")
            local dataFields = 0
            for key in pairs(data) do
                dataFields = dataFields + 1
                assert(key == "lines" or key == "dataInstanceID")
            end
            assert(dataFields == 2 and type(data.lines) == "table", "result has only lines and dataInstanceID")

            if expectedText == nil then
                assert(next(data.lines) == nil, "absent localization appends no line")
                return
            end

            local line = assert(data.lines[1])
            assert(line.type == 41 and line.leftText == expectedText, "retrieving line uses type 41 and supplied localization")
            assert(line.rightText == nil and line.rightColor == nil, "retrieving line has no right-side fields")
            local lineFields = 0
            for key in pairs(line) do
                lineFields = lineFields + 1
                assert(key == "type" or key == "leftText" or key == "leftColor")
            end
            assert(lineFields == 3, "retrieving line has only type, leftText, and leftColor")

            local color = assert(line.leftColor)
            assert(type(rawget(color, "r")) == "number", "error color publishes red")
            assert(type(rawget(color, "g")) == "number", "error color publishes green")
            assert(type(rawget(color, "b")) == "number", "error color publishes blue")
            assert(rawget(color, "a") == nil, "error color omits alpha")
            local r, g, b = color:GetRGB()
            local er, eg, eb = 1, 0, 1
            if ERROR_COLOR ~= nil then er, eg, eb = ERROR_COLOR:GetRGB() end
            assert(math.abs(r - er) < 0.001)
            assert(math.abs(g - eg) < 0.001)
            assert(math.abs(b - eb) < 0.001)
        end

        local retrievingText = RETRIEVING_ITEM_INFO
        if retrievingText ~= nil then assertPayload(retrievingText) end
        RETRIEVING_ITEM_INFO = nil
        assertPayload(nil)
        RETRIEVING_ITEM_INFO = retrievingText
        if retrievingText == nil then assertPayload(nil) end
    "##,
    )
    .expect("zero conversion output publishes only the q1 unresolved-item fields");
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

// INFERRED model coverage: selection uses the simulator's bag-cursor projection.
#[test]
fn challenges_ui_hover_uses_public_keystone_selection() {
    for (slot, item_id) in [(3, 180_653), (4, 158_923)] {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec("A_Admin.ClearBags(); C_ChallengeMode.ClearKeystone()")
            .expect("empty challenge selection fixture");
        env.exec(
            r##"
            local function capture(...) return {n = select("#", ...), ...} end
            local has = capture(C_ChallengeMode.HasSlottedKeystone("ignored"))
            local removed = capture(C_ChallengeMode.RemoveKeystone())
            assert(has.n == 1 and has[1] == false)
            assert(removed.n == 1 and removed[1] == false)
        "##,
        )
        .unwrap();

        let guid = slot_keystone_from_bag(&env, slot, item_id);
        load_challenges_ui(&env);
        assert_challenges_hover_item(&env, item_id, &guid);
        env.exec(
            r##"
            local function capture(...) return {n = select("#", ...), ...} end
            local removed = capture(C_ChallengeMode.RemoveKeystone())
            assert(removed.n == 1 and removed[1] == true)
            assert(not C_ChallengeMode.HasSlottedKeystone())
            assert(select("#", C_TooltipInfo.GetSlottedKeystone()) == 0)
        "##,
        )
        .unwrap();
        assert_challenges_hover_hidden(&env);
    }
}

// INFERRED model coverage: selection uses the simulator's bag-cursor projection.
#[test]
fn slot_keystone_notifies_after_selection_state() {
    for (slot, item_id) in [(3, 180_653), (4, 158_923)] {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        register_keystone_event_probe(&env);
        let guid = slot_keystone_from_bag(&env, slot, item_id);
        env.exec(&format!(
            r##"
            assert(#R7KeystoneSlottedState == 1)
            local selected = R7KeystoneSlottedState[1]
            assert(selected.eventItemID == {item_id})
            assert(selected.tooltipItemID == {item_id})
            assert(selected.guid == "{guid}")
            local removed = C_ChallengeMode.RemoveKeystone()
            assert(removed == true and not C_ChallengeMode.HasSlottedKeystone())
            assert(select("#", C_TooltipInfo.GetSlottedKeystone()) == 0)
            assert(select("#", C_ChallengeMode.SlotKeystone()) == 0)
            assert(#R7KeystoneSlottedState == 1)
            "##
        ))
        .unwrap();
    }
}

// SUPPLEMENTARY, INFERRED model coverage: this constructs the simulator's bag cursor directly.
#[test]
fn slot_keystone_leaves_unresolved_cursor_untouched() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    register_keystone_event_probe(&env);
    assert!(env.state().borrow().item_info(999_999).is_none());
    env.state().borrow_mut().cursor_item = Some(CursorInfo::Item {
        owned_identity: None,
        item_id: 999_999,
        stack_count: 1,
        origin: CursorItemOrigin::Bag { bag: 0, slot: 3 },
        applied_transmog: None,
    });
    env.exec(
        r##"
        assert(select("#", C_ChallengeMode.SlotKeystone()) == 0)
        assert(CursorHasItem())
        local kind, itemID = GetCursorInfo()
        assert(kind == "item" and itemID == 999999)
        assert(not C_ChallengeMode.HasSlottedKeystone())
        assert(select("#", C_TooltipInfo.GetSlottedKeystone()) == 0)
        assert(#R7KeystoneSlottedState == 0)
        "##,
    )
    .unwrap();
}

// SUPPLEMENTARY, INFERRED model coverage: this is rejection-only and uses the context proxy.
#[test]
fn slot_keystone_rejects_unmatched_or_missing_inputs_before_publication() {
    let cases = [
        (158_923, Some(2), Some(961), true, true),
        (158_923, Some(999_999), Some(960), true, false),
        (158_923, None, Some(960), true, false),
        (158_923, Some(2), None, true, false),
        (158_923, Some(2), Some(960), false, false),
        (6_948, Some(2), Some(960), true, false),
    ];

    for (index, (item_id, modifier_id, map_id, context_present, seed_lock)) in
        cases.into_iter().enumerate()
    {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        register_keystone_event_probe(&env);
        let slot = 3;
        add_bag_item(&env, 0, slot, item_id);
        env.exec(&format!("C_Container.PickupContainerItem(0, {slot})"))
            .unwrap();
        {
            let mut sim = env.state().borrow_mut();
            let location = KeystoneLocation::Bag(0, slot as i32);
            if let Some(modifier_id) = modifier_id {
                sim.keystone
                    .item_modifiers
                    .insert(location, (item_id, vec![(17, modifier_id)]));
            }
            sim.keystone.current_map_selector = map_id;
            sim.player.present = context_present;
            if seed_lock {
                sim.keystone.reason_one_locks.insert(location);
            }
        }

        env.exec(
            r##"
            assert(select("#", C_ChallengeMode.SlotKeystone()) == 0)
            assert(not CursorHasItem())
            assert(not C_ChallengeMode.HasSlottedKeystone())
            assert(select("#", C_TooltipInfo.GetSlottedKeystone()) == 0)
            assert(#R7KeystoneSlottedState == 0)
            "##,
        )
        .unwrap();

        if index == 0 {
            env.exec(&format!(
                r#"
                A_Admin.AddBagItem(0, {slot}, 6948, 1)
                assert(not C_Item.IsLocked(ItemLocation:CreateFromBagAndSlot(0, {slot})))
                "#
            ))
            .unwrap();
        }
    }
}

// SUPPLEMENTARY, INFERRED model coverage: one positive case cannot reject fixed staged output.
#[test]
fn remove_keystone_stages_item_for_nonzero_guid_category() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    slot_keystone_from_bag(&env, 3, 158_923);
    env.state().borrow_mut().keystone.cursor_stage_permission = Some(true);
    env.exec(
        r##"
        assert(C_ChallengeMode.RemoveKeystone())
        assert(not C_ChallengeMode.HasSlottedKeystone())
        local kind, cursorItemID = GetCursorInfo()
        assert(kind == "item" and cursorItemID == 158923)
        "##,
    )
    .unwrap();
}

// INFERRED model coverage: both GUID strings, staged count and bag origin are simulator projections.
#[test]
fn remove_keystone_zero_category_clears_cursor_without_staging() {
    for (selected_guid, should_stage) in [("0-0-0-158923", false), ("Item-0-3-158923", true)] {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r##"
            A_Admin.AddBagItem(0, 3, 158923, 1)
            A_Admin.AddBagItem(0, 4, 6948, 1)
            C_Container.PickupContainerItem(0, 4)
            "##,
        )
        .unwrap();
        {
            let mut sim = env.state().borrow_mut();
            sim.keystone.selected_guid = Some(selected_guid.to_string());
            sim.keystone.selected_item_id = Some(158_923);
            sim.keystone.selected_location = Some(KeystoneLocation::Bag(0, 3));
            sim.keystone.cursor_stage_permission = Some(true);
            sim.player.present = true;
        }
        env.exec(
            r##"
            local kind, itemID = GetCursorInfo()
            assert(kind == "item" and itemID == 6948)
            assert(C_ChallengeMode.RemoveKeystone())
            assert(not C_ChallengeMode.HasSlottedKeystone())
            "##,
        )
        .unwrap();

        if should_stage {
            env.exec(
                r##"
                local kind, itemID = GetCursorInfo()
                assert(kind == "item" and itemID == 158923)
                "##,
            )
            .unwrap();
            let staged_cursor = env.state().borrow().cursor_item.clone();
            assert!(matches!(
                staged_cursor,
                Some(CursorInfo::Item {
                    item_id: 158_923,
                    stack_count: 1,
                    origin: CursorItemOrigin::Bag { bag: 0, slot: 3 },
                    ..
                })
            ));
        } else {
            env.exec("assert(GetCursorInfo() == nil)").unwrap();
            assert!(env.state().borrow().cursor_item.is_none());
        }
    }
}

// SUPPLEMENTARY, INFERRED model coverage: all combinations deny through the context/permission proxies.
#[test]
fn remove_keystone_stage_gate_denial_preserves_cursor_and_clears_selection() {
    for (permission, context_present) in [(None, true), (Some(false), true), (Some(true), false)] {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r##"
            A_Admin.AddBagItem(0, 4, 6948, 1)
            C_Container.PickupContainerItem(0, 4)
            "##,
        )
        .unwrap();
        {
            let mut sim = env.state().borrow_mut();
            sim.keystone.selected_guid = Some("Item-0-3-158923".to_string());
            sim.keystone.cursor_stage_permission = permission;
            sim.player.present = context_present;
        }
        env.exec(
            r##"
            assert(C_ChallengeMode.RemoveKeystone())
            assert(not C_ChallengeMode.HasSlottedKeystone())
            local kind, cursorItemID = GetCursorInfo()
            assert(kind == "item" and cursorItemID == 6948)
            "##,
        )
        .unwrap();
    }
}

// SUPPLEMENTARY: the empty-selection path does not exercise nonempty staging.
#[test]
fn remove_keystone_without_selection_preserves_cursor() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.exec(
        r##"
        A_Admin.AddBagItem(0, 4, 6948, 1)
        C_Container.PickupContainerItem(0, 4)
        "##,
    )
    .unwrap();
    env.state().borrow_mut().keystone.cursor_stage_permission = Some(true);
    env.exec(
        r##"
        assert(not C_ChallengeMode.RemoveKeystone())
        assert(not C_ChallengeMode.HasSlottedKeystone())
        local kind, cursorItemID = GetCursorInfo()
        assert(kind == "item" and cursorItemID == 6948)
        "##,
    )
    .unwrap();
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

    env.state().borrow_mut().keystone.selected_guid = Some("Item-0-3-999999".to_string());
    env.exec(
        r##"
        local function capture(...) return {n = select("#", ...), ...} end
        local result = capture(C_ChallengeMode.HasSlottedKeystone("ignored", 17))
        assert(result.n == 1 and type(result[1]) == "boolean" and result[1])
        assert(select("#", C_TooltipInfo.GetSlottedKeystone()) == 0)
        local removed = capture(C_ChallengeMode.RemoveKeystone())
        assert(removed.n == 1 and removed[1] == true)
        assert(not C_ChallengeMode.HasSlottedKeystone())
    "##,
    )
    .unwrap();
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
        assert(
            count == 1 and type(data) == "table",
            "resolved unsupported kind returns one TooltipData table"
        )
        assert(
            type(data.lines) == "table" and next(data.lines) == nil,
            "unsupported-kind lines are empty"
        )
        assert(data.dataInstanceID == nil, "unsupported-kind result has no dataInstanceID")
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
