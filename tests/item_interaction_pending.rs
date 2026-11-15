#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::c_api::bag_info::BagInfo;
use wow_ui_sim::c_api::{
    ItemInteractionTooltipInput, ItemInteractionTooltipLine,
    c_item_interaction::ItemInteractionInfo,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::EquippedItem;

const ITEM_A: u32 = 128_910;
const ITEM_B: u32 = 158_923;
const CURSOR_ITEM: u32 = 6_948;
const SET_PENDING_ITEM_USAGE: &str =
    "Usage: local success = C_ItemInteraction.SetPendingItem([item])";

fn pending_env(accepted_items: &[u32]) -> WowLuaEnv {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    seed_interaction_state(&env, accepted_items);
    load_original_item_interaction(&env);
    add_backed_items(&env);
    install_event_probe(&env);
    env
}

fn seed_interaction_state(env: &WowLuaEnv, accepted_items: &[u32]) {
    let mut sim = env.state().borrow_mut();
    sim.item_interaction = Some(ItemInteractionInfo {
        interaction_type: 2,
        ..Default::default()
    });
    sim.item_interaction_tooltip_input = Some(ItemInteractionTooltipInput {
        cleansing_rows: vec![ItemInteractionTooltipLine {
            left_text: "Selected pending item".to_string(),
            ..Default::default()
        }],
        pending_item_acceptance: accepted_items
            .iter()
            .map(|item_id| ((2, *item_id), true))
            .collect(),
        ..Default::default()
    });
}

fn load_original_item_interaction(env: &WowLuaEnv) {
    let (loaded, reason): (bool, Option<String>) = env
        .eval(r#"return C_AddOns.LoadAddOn("Blizzard_ItemInteractionUI")"#)
        .unwrap();
    assert!(loaded, "Blizzard_ItemInteractionUI load failed: {reason:?}");
    env.exec(
        r#"
        assert(debug.getinfo(ItemInteractionItemSlotMixin.OnReceiveDrag, "S").source
            :find("Blizzard_ItemInteractionUI.lua", 1, true))
        assert(debug.getinfo(ItemInteractionItemSlotMixin.OnDragStart, "S").source
            :find("Blizzard_ItemInteractionUI.lua", 1, true))
        assert(debug.getinfo(ItemInteractionMixin.SetDynamicFlyoutSettings, "S").source
            :find("Blizzard_ItemInteractionUI.lua", 1, true))
        assert(debug.getinfo(ItemInteractionMixin.SetInputItemSlotTooltip, "S").source
            :find("Blizzard_ItemInteractionUI.lua", 1, true))
        ItemInteractionFrame.interactionType = Enum.UIItemInteractionType.CleanseCorruption
        "#,
    )
    .unwrap();
}

fn add_backed_items(env: &WowLuaEnv) {
    env.exec(&format!(
        "A_Admin.ClearBags(); A_Admin.AddBagItem(0, 1, {ITEM_A}, 1); A_Admin.AddBagItem(0, 2, {ITEM_B}, 1); A_Admin.AddBagItem(0, 3, {CURSOR_ITEM}, 1); PendingLocationIDs = {{['0:1']={ITEM_A}, ['0:2']={ITEM_B}}}"
    ))
    .unwrap();
}

fn install_event_probe(env: &WowLuaEnv) {
    env.exec(
        r#"
        PendingSelectionEvents = {}
        PendingOrderEvents = {}
        PendingHideEvents = {}
        PendingEventProbe = CreateFrame("Frame")
        for _, event in ipairs({
            "ITEM_INTERACTION_ITEM_SELECTION_UPDATED", "ITEM_LOCK_CHANGED",
            "ITEM_LOCKED", "ITEM_UNLOCKED", "CURSOR_CHANGED",
            "PLAYER_INTERACTION_MANAGER_FRAME_HIDE",
        }) do
            PendingEventProbe:RegisterEvent(event)
        end
        PendingEventProbe:SetScript("OnEvent", function(_, event, payload)
            table.insert(PendingOrderEvents, event)
            if event == "PLAYER_INTERACTION_MANAGER_FRAME_HIDE" then
                table.insert(PendingHideEvents, payload)
            elseif event == "ITEM_INTERACTION_ITEM_SELECTION_UPDATED" then
                local itemID
                if payload and payload.bagID ~= nil and payload.slotIndex ~= nil then
                    local key = tostring(payload.bagID) .. ":" .. tostring(payload.slotIndex)
                    itemID = PendingLocationIDs[key]
                end
                table.insert(PendingSelectionEvents, itemID or false)
            end
        end)
        "#,
    )
    .unwrap();
}

fn select_from_original_cursor(env: &WowLuaEnv, slot: u32) {
    env.exec(&format!(
        "C_Container.PickupContainerItem(0, {slot}); PendingOrderEvents = {{}}"
    ))
    .unwrap();
    env.exec("ItemInteractionItemSlotMixin.OnReceiveDrag(ItemInteractionFrame.ItemSlot)")
        .unwrap();
}

fn select_from_original_location(env: &WowLuaEnv, location: &str) {
    env.exec(&format!(
        r#"
        ItemInteractionFrame:SetupEquipmentFlyout(true)
        ItemInteractionFrame:SetDynamicFlyoutSettings()
        local button = ItemInteractionFrame.ItemConversionFrame.ItemConversionInputSlot
        button.itemLocation = {location}
        PendingOrderEvents = {{}}
        local onClick = ItemInteractionFrame.flyoutSettings.onClickFunc
        assert(debug.getinfo(onClick, "S").source:find("Blizzard_ItemInteractionUI.lua", 1, true))
        onClick(button)
        "#
    ))
    .unwrap();
}

fn select_from_original_flyout(env: &WowLuaEnv, slot: u32) {
    select_from_original_location(
        env,
        &format!("ItemLocation:CreateFromBagAndSlot(0, {slot})"),
    );
}

fn stage_cursor_item(env: &WowLuaEnv) {
    env.exec(&format!(
        "A_Admin.AddBagItem(0, 3, {CURSOR_ITEM}, 1); C_Container.PickupContainerItem(0, 3)"
    ))
    .unwrap();
}

fn clear_from_original_slot(env: &WowLuaEnv) {
    env.exec("PendingOrderEvents = {}; ItemInteractionItemSlotMixin.OnDragStart(ItemInteractionFrame.ItemSlot)")
        .unwrap();
}

fn clear_event_history(env: &WowLuaEnv) {
    env.exec("PendingSelectionEvents = {}; PendingOrderEvents = {}; PendingHideEvents = {}")
        .unwrap();
}

fn assert_accept_order(env: &WowLuaEnv) {
    env.exec(
        r#"
        local positions = {}
        for index, event in ipairs(PendingOrderEvents) do
            positions[event] = positions[event] or index
        end
        local lockChanged = assert(positions["ITEM_LOCK_CHANGED"])
        local locked = assert(positions["ITEM_LOCKED"])
        local selected = assert(positions["ITEM_INTERACTION_ITEM_SELECTION_UPDATED"])
        local cursorChanged = assert(positions["CURSOR_CHANGED"])
        assert(lockChanged < locked and locked < selected and selected < cursorChanged)
        "#,
    )
    .unwrap();
}

fn assert_item_locked(env: &WowLuaEnv, slot: u32, locked: bool) {
    env.exec(&format!(
        "local location = ItemLocation:CreateFromBagAndSlot(0, {slot}); assert(C_Item.IsLocked(location) == {locked})"
    ))
    .unwrap();
}

fn assert_selected_tooltip(env: &WowLuaEnv, slot: u32) {
    env.exec(&format!(
        r#"
        GameTooltip:Hide()
        ItemInteractionFrame:SetInputItemSlotTooltip(
            ItemInteractionFrame.ItemSlot, ItemLocation:CreateFromBagAndSlot(0, {slot}))
        local data = assert(GameTooltip:GetPrimaryTooltipData(), "selected item tooltip missing")
        assert(data.type == 20 and data.lines[1].leftText == "Selected pending item")
        "#
    ))
    .unwrap();
}

fn assert_no_selected_tooltip(env: &WowLuaEnv) {
    env.exec(
        r##"
        assert(select("#", C_TooltipInfo.GetItemInteractionItem()) == 0)
        GameTooltip:Hide()
        ItemInteractionFrame:SetInputItemSlotTooltip(ItemInteractionFrame.ItemSlot, nil)
        assert(GameTooltip:GetPrimaryTooltipData() == nil)
        "##,
    )
    .unwrap();
}

fn set_identifiers(env: &WowLuaEnv, selection_id: i32, record_id: u32) {
    let mut state = env.state().borrow_mut();
    let input = state.item_interaction_tooltip_input.as_mut().unwrap();
    input.selection_id = selection_id;
    input.record_id = record_id;
}

fn assert_identifiers(env: &WowLuaEnv, selection_id: i32, record_id: u32) {
    let state = env.state().borrow();
    let input = state.item_interaction_tooltip_input.as_ref().unwrap();
    assert_eq!(
        (input.selection_id, input.record_id),
        (selection_id, record_id)
    );
}

fn assert_bad_location_raises(env: &WowLuaEnv) {
    env.exec(
        r##"
        local ok, err = pcall(C_ItemInteraction.SetPendingItem, {})
        assert(not ok and tostring(err):find(
            "Usage: local success = C_ItemInteraction.SetPendingItem([item])", 1, true))
        "##,
    )
    .unwrap();
}

fn allow_item_kind(env: &WowLuaEnv, kind: i32, item_id: u32) {
    let mut state = env.state().borrow_mut();
    state
        .item_interaction_tooltip_input
        .as_mut()
        .unwrap()
        .pending_item_acceptance
        .insert((kind, item_id), true);
}

fn assert_other_kind_rejected(env: &WowLuaEnv) {
    env.state()
        .borrow_mut()
        .item_interaction
        .as_mut()
        .unwrap()
        .interaction_type = 99;
    allow_item_kind(env, 99, ITEM_B);
    select_from_original_flyout(env, 2);
    assert_no_selected_tooltip(env);
}

#[test]
fn pending_item_accepts_distinct_backed_items_and_updates_tooltip() {
    let env = pending_env(&[ITEM_A, ITEM_B]);
    select_from_original_cursor(&env, 1);
    assert_accept_order(&env);
    assert_selected_tooltip(&env, 1);
    select_from_original_flyout(&env, 2);
    assert_item_locked(&env, 2, true);
    assert_selected_tooltip(&env, 2);
    clear_from_original_slot(&env);
    assert_item_locked(&env, 2, false);
    assert_no_selected_tooltip(&env);
    env.exec("assert(PendingSelectionEvents[1] == 128910 and PendingSelectionEvents[2] == 158923 and PendingSelectionEvents[3] == false)")
        .unwrap();
}

#[test]
fn pending_item_rejection_does_not_restore_old_selection() {
    let env = pending_env(&[ITEM_A]);
    select_from_original_flyout(&env, 1);
    stage_cursor_item(&env);
    select_from_original_flyout(&env, 2);
    assert_item_locked(&env, 1, false);
    assert_item_locked(&env, 2, false);
    assert_no_selected_tooltip(&env);
    assert_other_kind_rejected(&env);
    clear_from_original_slot(&env);
    env.exec(
        r#"
        assert(CursorHasItem())
        assert(PendingSelectionEvents[1] == 128910)
        assert(PendingSelectionEvents[2] == false)
        assert(PendingSelectionEvents[3] == false)
        assert(PendingSelectionEvents[4] == false)
        "#,
    )
    .unwrap();
    env.exec(
        r##"
        local location = ItemLocation:CreateFromBagAndSlot(0, 2)
        assert(select("#", C_ItemInteraction.SetPendingItem(location)) == 1)
        assert(C_ItemInteraction.SetPendingItem(location) == false)
        "##,
    )
    .unwrap();
}

fn set_interaction_kind(env: &WowLuaEnv, kind: i32) {
    env.state()
        .borrow_mut()
        .item_interaction
        .as_mut()
        .unwrap()
        .interaction_type = kind;
}

fn assert_unbacked_kind_requires_observation(env: &WowLuaEnv, kind: i32) {
    set_interaction_kind(env, kind);
    clear_event_history(env);
    select_from_original_flyout(env, 1);
    assert_item_locked(env, 1, false);
    assert_no_selected_tooltip(env);
    env.exec("assert(PendingSelectionEvents[1] == false)")
        .unwrap();
    allow_item_kind(env, kind, ITEM_A);
    allow_item_kind(env, kind, ITEM_B);
    select_from_original_flyout(env, 1);
    select_from_original_flyout(env, 2);
    assert_item_locked(env, 2, true);
    clear_from_original_slot(env);
    assert_item_locked(env, 2, false);
    env.exec("assert(PendingSelectionEvents[2] == 128910 and PendingSelectionEvents[3] == 158923 and PendingSelectionEvents[4] == false)")
        .unwrap();
}

#[test]
fn pending_item_unbacked_kinds_require_explicit_observations() {
    let env = pending_env(&[]);
    for kind in [2, 3, 4] {
        assert_unbacked_kind_requires_observation(&env, kind);
    }
}

#[test]
fn pending_item_same_object_fast_path_keeps_cursor_and_sends_no_event() {
    let env = pending_env(&[ITEM_A, ITEM_B]);
    select_from_original_flyout(&env, 1);
    clear_event_history(&env);
    stage_cursor_item(&env);
    select_from_original_flyout(&env, 1);
    env.exec(
        "assert(CursorHasItem() and #PendingSelectionEvents == 0 and #PendingOrderEvents == 0)",
    )
    .unwrap();
    env.exec("ClearCursor()").unwrap();
    select_from_original_flyout(&env, 2);
    assert_item_locked(&env, 2, true);
    stage_cursor_item(&env);
    clear_from_original_slot(&env);
    env.exec("assert(CursorHasItem() and PendingSelectionEvents[2] == false)")
        .unwrap();
}

#[test]
fn pending_item_clear_keeps_identifiers_and_preserves_cursor() {
    let env = pending_env(&[ITEM_A, ITEM_B]);
    set_identifiers(&env, 4, 7);
    select_from_original_flyout(&env, 1);
    select_from_original_flyout(&env, 2);
    assert_bad_location_raises(&env);
    assert_item_locked(&env, 2, true);
    stage_cursor_item(&env);
    clear_from_original_slot(&env);
    assert_item_locked(&env, 2, false);
    assert_identifiers(&env, 4, 7);
    assert_no_selected_tooltip(&env);
    env.exec(
        r##"
        assert(CursorHasItem() and PendingSelectionEvents[3] == false)
        assert(select("#", C_ItemInteraction.SetPendingItem()) == 1)
        assert(C_ItemInteraction.SetPendingItem() == false)
        assert(select("#", C_ItemInteraction.SetPendingItem(nil)) == 1)
        assert(C_ItemInteraction.SetPendingItem(nil) == false)
        assert(CursorHasItem())
        "##,
    )
    .unwrap();
}

#[test]
fn pending_item_nil_is_an_absent_replacement_and_preserves_cursor() {
    let env = pending_env(&[ITEM_A]);
    select_from_original_flyout(&env, 1);
    stage_cursor_item(&env);
    clear_event_history(&env);

    env.exec(
        r##"
        local function capture(...) return select("#", ...), ... end
        local count, success = capture(C_ItemInteraction.SetPendingItem(nil))
        assert(count == 1 and success == false)
        assert(not C_Item.IsLocked(ItemLocation:CreateFromBagAndSlot(0, 1)))
        assert(#PendingSelectionEvents == 1 and PendingSelectionEvents[1] == false)
        assert(CursorHasItem())
        local cursorType, cursorID = GetCursorInfo()
        assert(cursorType == "item" and cursorID == 6948)
        for _, event in ipairs(PendingOrderEvents) do
            assert(event ~= "CURSOR_CHANGED")
        end
        "##,
    )
    .unwrap();
}

#[test]
fn pending_item_initialize_frame_clears_without_releasing_lock() {
    let env = pending_env(&[ITEM_A, ITEM_B]);
    set_identifiers(&env, 18, 72);
    select_from_original_flyout(&env, 1);
    select_from_original_flyout(&env, 2);
    clear_event_history(&env);
    env.exec("C_ItemInteraction.InitializeFrame(); C_ItemInteraction.ClearPendingItem()")
        .expect("OnShow calls InitializeFrame then ClearPendingItem");
    assert_item_locked(&env, 2, true);
    assert_identifiers(&env, 18, 72);
    assert_no_selected_tooltip(&env);
    env.exec("assert(#PendingSelectionEvents == 1 and PendingSelectionEvents[1] == false)")
        .unwrap();
    assert_no_unlock_event(&env);
}

#[test]
fn pending_item_reset_clears_identifiers_without_selection_event() {
    let env = pending_env(&[ITEM_A, ITEM_B]);
    set_identifiers(&env, 23, 91);
    select_from_original_flyout(&env, 1);
    select_from_original_flyout(&env, 2);
    clear_event_history(&env);
    env.exec("UIPanelWindows.ItemInteractionFrame.showFailedFunc()")
        .expect("original UIPanelWindows Reset callback");
    assert_item_locked(&env, 2, true);
    assert_identifiers(&env, -1, 0);
    assert_no_selected_tooltip(&env);
    env.exec("assert(#PendingSelectionEvents == 0)").unwrap();
    assert_no_unlock_event(&env);
    clear_from_original_slot(&env);
    env.exec("assert(PendingSelectionEvents[1] == false)")
        .unwrap();
}

#[test]
fn pending_item_close_ui_leaves_selection_unchanged() {
    let env = pending_env(&[ITEM_A, ITEM_B]);
    set_identifiers(&env, 31, 41);
    select_from_original_flyout(&env, 1);
    select_from_original_flyout(&env, 2);
    env.state()
        .borrow_mut()
        .active_player_interactions
        .extend([43, 44]);
    clear_event_history(&env);
    env.exec("C_ItemInteraction.CloseUI()")
        .expect("original OnHide calls CloseUI");
    assert_item_locked(&env, 2, true);
    assert_selected_tooltip(&env, 2);
    assert_identifiers(&env, 31, 41);
    {
        let state = env.state().borrow();
        assert!(!state.active_player_interactions.contains(&44));
        assert!(state.active_player_interactions.contains(&43));
    }
    env.exec(
        "assert(#PendingSelectionEvents == 0 and #PendingHideEvents == 1 and PendingHideEvents[1] == 44)",
    )
    .unwrap();
    clear_from_original_slot(&env);
    assert_no_selected_tooltip(&env);
}

#[test]
fn pending_item_close_ui_inactive_selector_sends_no_hide() {
    let env = pending_env(&[]);
    env.state()
        .borrow_mut()
        .active_player_interactions
        .insert(43);
    clear_event_history(&env);
    env.exec("C_ItemInteraction.CloseUI()")
        .expect("inactive selector 44 does not request a hide notification");
    env.exec("assert(#PendingHideEvents == 0)").unwrap();
}

fn set_bag_slot_count(env: &WowLuaEnv, bag: i32, num_slots: i32) {
    let mut sim = env.state().borrow_mut();
    let info = sim.bag_info.entry(bag).or_insert(BagInfo {
        num_slots: 0,
        family: 0,
        name: None,
        inventory_slot: None,
        item_id: None,
        hyperlink: None,
        owned_identity: None,
    });
    info.num_slots = num_slots;
}

fn add_pending_bag_item(env: &WowLuaEnv, bag: i32, slot: i32, item_id: u32) {
    env.exec(&format!("A_Admin.AddBagItem({bag}, {slot}, {item_id}, 1)"))
        .unwrap();
}

fn seed_equipped_items(env: &WowLuaEnv, items: &[(i32, u32)]) {
    let mut sim = env.state().borrow_mut();
    sim.player.equipped_items.clear();
    for (slot, item_id) in items {
        sim.player.equipped_items.insert(
            *slot,
            EquippedItem {
                owned_identity: None,
                item_id: *item_id,
                enchant_id: 0,
                gem_ids: [0; 3],
                applied_transmog: None,
            },
        );
    }
}

fn assert_equipped_item_locked(env: &WowLuaEnv, slot: i32, locked: bool) {
    env.exec(&format!(
        "assert(C_Item.IsLocked(ItemLocation:CreateFromEquipmentSlot({slot})) == {locked})"
    ))
    .unwrap();
}

fn assert_bag_slot_locked(env: &WowLuaEnv, bag: i32, slot: i32, locked: bool) {
    env.exec(&format!(
        "assert(C_Item.IsLocked(ItemLocation:CreateFromBagAndSlot({bag}, {slot})) == {locked})"
    ))
    .unwrap();
}

#[test]
fn pending_item_accepts_numeric_string_location_fields() {
    let env = pending_env(&[ITEM_A, ITEM_B]);
    select_from_original_location(&env, "{bagID = '1', slotIndex = '1'}");
    assert_bag_slot_locked(&env, 1, 1, true);
    let cleared: bool = env
        .eval("return C_ItemInteraction.SetPendingItem(nil)")
        .unwrap();
    assert!(!cleared);

    set_bag_slot_count(&env, 16, 1);
    add_pending_bag_item(&env, 16, 1, ITEM_B);
    select_from_original_location(&env, "{bagID = '16', slotIndex = '1'}");
    assert_bag_slot_locked(&env, 16, 1, true);
}

#[test]
fn pending_item_truncates_location_fields_toward_zero() {
    let env = pending_env(&[ITEM_A, ITEM_B]);
    select_from_original_location(&env, "ItemLocation:CreateFromBagAndSlot(-0.9, 2.9)");
    assert_item_locked(&env, 2, true);
    assert_item_locked(&env, 1, false);
}

#[test]
fn pending_item_uses_bag_fields_when_equipment_field_is_not_numeric() {
    let env = pending_env(&[ITEM_A]);
    select_from_original_location(
        &env,
        "(function() local location = ItemLocation:CreateEmpty(); location.equipmentSlotIndex = 'not numeric'; location.bagID = 0; location.slotIndex = 1; return location end)()",
    );
    assert_item_locked(&env, 1, true);
}

#[test]
fn pending_item_prefers_numeric_equipment_field() {
    let env = pending_env(&[ITEM_A, ITEM_B]);
    seed_equipped_items(&env, &[(7, ITEM_B)]);
    select_from_original_location(
        &env,
        "(function() local location = ItemLocation:CreateFromEquipmentSlot('7.9'); location.bagID = 0; location.slotIndex = 1; return location end)()",
    );
    assert_equipped_item_locked(&env, 7, true);
    assert_item_locked(&env, 1, false);
    env.exec("assert(#PendingSelectionEvents == 1)").unwrap();
}

#[test]
fn pending_item_maps_negative_bag_ids_to_player_inventory() {
    let env = pending_env(&[ITEM_A, ITEM_B]);
    let items: Vec<_> = (1..=5)
        .map(|slot| (100 + slot, if slot % 2 == 0 { ITEM_B } else { ITEM_A }))
        .collect();
    seed_equipped_items(&env, &items);

    for bag in [-2, -3] {
        for slot in [1, 5] {
            select_from_original_location(
                &env,
                &format!("ItemLocation:CreateFromBagAndSlot({bag}, {slot})"),
            );
            assert_equipped_item_locked(&env, 100 + slot, true);
            let cleared: bool = env
                .eval("return C_ItemInteraction.SetPendingItem(nil)")
                .unwrap();
            assert!(!cleared);
        }
    }
}

#[test]
fn pending_item_maps_other_negative_bags_to_player_inventory() {
    let env = pending_env(&[ITEM_A, ITEM_B]);
    let items: Vec<_> = (1..=5)
        .map(|slot| (slot, if slot % 2 == 0 { ITEM_B } else { ITEM_A }))
        .collect();
    seed_equipped_items(&env, &items);

    for bag in [-4, i32::MIN + 1] {
        for slot in [1, 5] {
            select_from_original_location(
                &env,
                &format!("ItemLocation:CreateFromBagAndSlot({bag}, {slot})"),
            );
            assert_equipped_item_locked(&env, slot, true);
            let cleared: bool = env
                .eval("return C_ItemInteraction.SetPendingItem(nil)")
                .unwrap();
            assert!(!cleared);
        }
    }
}

#[test]
fn pending_item_maps_bags_six_through_eleven_to_slots_101_through_105() {
    let env = pending_env(&[ITEM_A, ITEM_B]);
    for bag in 6..=11 {
        set_bag_slot_count(&env, bag, 5);
        for slot in 1..=5 {
            let item_id = if (bag + slot) % 2 == 0 {
                ITEM_A
            } else {
                ITEM_B
            };
            add_pending_bag_item(&env, bag, slot + 100, item_id);
        }
    }

    for bag in 6..=11 {
        for slot in 1..=5 {
            select_from_original_location(
                &env,
                &format!("ItemLocation:CreateFromBagAndSlot({bag}, {slot})"),
            );
            assert_bag_slot_locked(&env, bag, slot + 100, true);
            let cleared: bool = env
                .eval("return C_ItemInteraction.SetPendingItem(nil)")
                .unwrap();
            assert!(!cleared);
        }
    }
}

#[test]
fn pending_item_enforces_bag_availability_and_slot_counts() {
    let env = pending_env(&[ITEM_A, ITEM_B]);
    add_pending_bag_item(&env, 16, 1, ITEM_A);
    assert_present_location_usage_error_without_side_effects(&env, "{bagID = 16, slotIndex = 1}");

    set_bag_slot_count(&env, 1, 1);
    add_pending_bag_item(&env, 1, 2, ITEM_B);
    assert_present_location_usage_error_without_side_effects(&env, "{bagID = 1, slotIndex = 2}");

    add_pending_bag_item(&env, 0, 17, ITEM_A);
    assert_present_location_usage_error_without_side_effects(&env, "{bagID = 0, slotIndex = 17}");
}

#[test]
fn pending_item_rejects_wrapped_or_unmapped_negative_bag_ids() {
    let env = pending_env(&[ITEM_A, ITEM_B]);
    add_pending_bag_item(&env, -1, 1, ITEM_A);
    add_pending_bag_item(&env, i32::MIN, 1, ITEM_B);
    for location in [
        "{bagID = -1, slotIndex = 1}",
        "{bagID = -2147483648, slotIndex = 1}",
    ] {
        assert_present_location_usage_error_without_side_effects(&env, location);
    }
}

fn prepare_invalid_location_case(env: &WowLuaEnv, location: &str) {
    select_from_original_flyout(env, 1);
    assert_item_locked(env, 1, true);
    assert_item_locked(env, 2, false);
    stage_cursor_item(env);
    env.exec(&format!(
        "InvalidPendingLocation = {location}; assert(InvalidPendingLocation ~= nil)"
    ))
    .unwrap();
    clear_event_history(env);
}

fn observe_present_location_effects(
    env: &WowLuaEnv,
) -> (bool, bool, bool, bool, bool, bool, bool, bool) {
    let script = format!(
        r##"
        local selectedLocation = ItemLocation:CreateFromBagAndSlot(0, 1)
        local otherLocation = ItemLocation:CreateFromBagAndSlot(0, 2)
        local cursorTypeBefore, cursorIDBefore = GetCursorInfo()
        local invalidButton = ItemInteractionFrame.ItemConversionFrame.ItemConversionInputSlot
        invalidButton.itemLocation = InvalidPendingLocation
        local ok, result = pcall(ItemInteractionFrame.flyoutSettings.onClickFunc, invalidButton)
        local cursorTypeAfter, cursorIDAfter = GetCursorInfo()
        local selectedCount = select("#", C_TooltipInfo.GetItemInteractionItem())
        local selectedData = C_TooltipInfo.GetItemInteractionItem()
        local hasSelectedItem = selectedCount == 1
            and selectedData.lines[1].leftText == "Selected pending item"
        return not ok,
            not ok and tostring(result):find({usage:?}, 1, true) ~= nil,
            #PendingSelectionEvents == 0,
            #PendingOrderEvents == 0,
            C_Item.IsLocked(selectedLocation),
            not C_Item.IsLocked(otherLocation),
            hasSelectedItem,
            CursorHasItem()
                and cursorTypeBefore == cursorTypeAfter
                and cursorIDBefore == cursorIDAfter
        "##,
        usage = SET_PENDING_ITEM_USAGE
    );
    env.eval(&script).unwrap()
}

fn assert_present_location_usage_error_without_side_effects(env: &WowLuaEnv, location: &str) {
    prepare_invalid_location_case(env, location);
    let (
        usage_error_raised,
        exact_usage_reason,
        no_selection_event,
        no_event_or_lock_notification,
        selected_item_stays_locked,
        other_item_stays_unlocked,
        selected_item_stays_published,
        cursor_stays_unchanged,
    ) = observe_present_location_effects(env);
    assert_eq!(
        (
            usage_error_raised,
            exact_usage_reason,
            no_selection_event,
            no_event_or_lock_notification,
            selected_item_stays_locked,
            other_item_stays_unlocked,
            selected_item_stays_published,
            cursor_stays_unchanged,
        ),
        (true, true, true, true, true, true, true, true)
    );
}

#[test]
fn empty_bag_item_location_raises_usage_without_side_effects() {
    let env = pending_env(&[ITEM_A, ITEM_B]);
    assert_present_location_usage_error_without_side_effects(
        &env,
        "ItemLocation:CreateFromBagAndSlot(0, 4)",
    );
}

#[test]
fn empty_equipment_item_location_raises_usage_without_side_effects() {
    let env = pending_env(&[ITEM_A, ITEM_B]);
    assert_present_location_usage_error_without_side_effects(
        &env,
        "ItemLocation:CreateFromEquipmentSlot(19)",
    );
}

#[test]
fn out_of_range_bag_item_location_raises_usage_without_side_effects() {
    let env = pending_env(&[ITEM_A, ITEM_B]);
    assert_present_location_usage_error_without_side_effects(
        &env,
        "ItemLocation:CreateFromBagAndSlot(17, 1)",
    );
}

#[test]
fn out_of_range_bag_slot_raises_usage_without_side_effects() {
    let env = pending_env(&[ITEM_A, ITEM_B]);
    assert_present_location_usage_error_without_side_effects(
        &env,
        "ItemLocation:CreateFromBagAndSlot(0, 99)",
    );
}

fn assert_no_unlock_event(env: &WowLuaEnv) {
    env.exec(
        r#"
        for _, event in ipairs(PendingOrderEvents) do
            assert(event ~= "ITEM_UNLOCKED", "direct clear released the selection lock")
        end
        "#,
    )
    .unwrap();
}
