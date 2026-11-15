#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::c_api::{
    ItemInteractionTooltipInput, ItemInteractionTooltipLine,
    c_item_interaction::ItemInteractionInfo,
};
use wow_ui_sim::lua_api::WowLuaEnv;

const ITEM_A: u32 = 128_910;
const ITEM_B: u32 = 158_923;
const CURSOR_ITEM: u32 = 6_948;

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
        assert(debug.getinfo(ItemInteractionMixin.OnShow, "S").source
            :find("Blizzard_ItemInteractionUI.lua", 1, true))
        assert(debug.getinfo(ItemInteractionMixin.OnHide, "S").source
            :find("Blizzard_ItemInteractionUI.lua", 1, true))
        ItemInteractionFrame.interactionType = Enum.UIItemInteractionType.CleanseCorruption
        "#,
    )
    .unwrap();
}

fn add_backed_items(env: &WowLuaEnv) {
    env.exec(&format!(
        "A_Admin.ClearBags(); A_Admin.AddBagItem(0, 1, {ITEM_A}, 1); A_Admin.AddBagItem(0, 2, {ITEM_B}, 1); A_Admin.AddBagItem(0, 3, {CURSOR_ITEM}, 1)"
    ))
    .unwrap();
}

fn install_event_probe(env: &WowLuaEnv) {
    env.exec(
        r#"
        PendingSelectionEvents = {}
        PendingOrderEvents = {}
        PendingEventProbe = CreateFrame("Frame")
        for _, event in ipairs({
            "ITEM_INTERACTION_ITEM_SELECTION_UPDATED", "ITEM_LOCK_CHANGED",
            "ITEM_LOCKED", "ITEM_UNLOCKED", "CURSOR_CHANGED",
        }) do
            PendingEventProbe:RegisterEvent(event)
        end
        PendingEventProbe:SetScript("OnEvent", function(_, event, location)
            table.insert(PendingOrderEvents, event)
            if event == "ITEM_INTERACTION_ITEM_SELECTION_UPDATED" then
                table.insert(PendingSelectionEvents,
                    location and C_Item.GetItemID(location) or false)
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

fn select_from_original_flyout(env: &WowLuaEnv, slot: u32) {
    env.exec(&format!(
        r#"
        ItemInteractionFrame:SetupEquipmentFlyout(true)
        ItemInteractionFrame:SetDynamicFlyoutSettings()
        local button = ItemInteractionFrame.ItemConversionFrame.ItemConversionInputSlot
        button:SetItemLocation(ItemLocation:CreateFromBagAndSlot(0, {slot}))
        PendingOrderEvents = {{}}
        ItemInteractionFrame.flyoutSettings.onClickFunc(button)
        "#
    ))
    .unwrap();
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
    env.exec("PendingSelectionEvents = {}; PendingOrderEvents = {}")
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
    env.exec(
        r##"
        local location = ItemLocation:CreateFromBagAndSlot(0, 1)
        assert(select("#", C_ItemInteraction.SetPendingItem(location)) == 1)
        assert(C_ItemInteraction.SetPendingItem(location) == true)
        "##,
    )
    .unwrap();
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
fn pending_item_initialize_frame_clears_without_releasing_lock() {
    let env = pending_env(&[ITEM_A, ITEM_B]);
    set_identifiers(&env, 18, 72);
    select_from_original_flyout(&env, 1);
    select_from_original_flyout(&env, 2);
    clear_event_history(&env);
    env.exec("ItemInteractionFrame:OnShow()")
        .expect("original OnShow invokes InitializeFrame and ClearPendingItem");
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
    env.exec("ItemInteractionFrame:OnHide()")
        .expect("original OnHide invokes CloseUI");
    assert_item_locked(&env, 2, true);
    assert_selected_tooltip(&env, 2);
    assert_identifiers(&env, 31, 41);
    let state = env.state().borrow();
    assert!(!state.active_player_interactions.contains(&44));
    assert!(state.active_player_interactions.contains(&43));
    env.exec("assert(#PendingSelectionEvents == 0)").unwrap();
    clear_from_original_slot(&env);
    assert_no_selected_tooltip(&env);
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
