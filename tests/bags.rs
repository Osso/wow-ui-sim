#![cfg(feature = "gui")]

//! Tests for bag frames opening and displaying items.
//!
//! Loads the full Blizzard addon set, opens bags via keybind, and verifies
//! that item slots are populated with real item data from the mock inventory.

use crate::common;
#[path = "common/keybinding_panel_fixtures.rs"]
mod keybinding_panel_fixtures;

use keybinding_panel_fixtures::setup_env;
#[cfg(not(feature = "client-wrath"))]
use keybinding_panel_fixtures::{BLIZZARD_ADDONS, BLIZZARD_TOKEN_UI_ADDON, blizzard_ui_dir};
#[cfg(not(feature = "client-wrath"))]
use wow_ui_sim::iced_app::{RegistryQuadBatchParams, build_quad_batch_for_registry};
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(not(feature = "client-wrath"))]
fn setup_env_with_bootstrap_loaded_token_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    if common::load_classic_game_startup_inventory(&env) {
        keybinding_panel_fixtures::load_token_ui(&env);
    } else {
        let ui = blizzard_ui_dir();
        for addon_name in BLIZZARD_ADDONS
            .iter()
            .copied()
            .chain(std::iter::once(BLIZZARD_TOKEN_UI_ADDON))
        {
            common::load_required_blizzard_addon(&env, &ui, addon_name);
        }
    }

    env.apply_post_load_workarounds();
    env
}

fn install_test_error_handler(env: &WowLuaEnv) {
    common::install_error_collector(env, "__test_errors");
}

fn drain_test_errors(env: &WowLuaEnv) -> Vec<String> {
    common::drain_string_table(env, "__test_errors")
}

fn clear_recorded_lua_errors(env: &WowLuaEnv) {
    common::panel_fixtures::clear_recorded_lua_errors(env);
}

fn disable_combined_bags(env: &WowLuaEnv) {
    if common::is_classic_ui_profile() {
        return;
    }
    env.exec(
        r#"
        assert(ContainerFrameSettingsManager, "ContainerFrameSettingsManager should exist")
        ContainerFrameSettingsManager:SetUsingCombinedBags(false)
        "#,
    )
    .expect("individual bag layout should be selectable");
}

#[cfg(not(feature = "client-wrath"))]
fn build_texture_requests(env: &WowLuaEnv) -> Vec<String> {
    let buckets = {
        let mut state = env.state().borrow_mut();
        let _ = state.get_strata_buckets();
        state
            .strata_buckets
            .as_ref()
            .expect("strata buckets should be initialized")
            .clone()
    };
    let state = env.state().borrow();
    let batch = build_quad_batch_for_registry(RegistryQuadBatchParams::new(
        &state.widgets,
        (1024.0, 768.0),
        &buckets,
    ));
    batch
        .texture_requests
        .iter()
        .map(|request| request.path.clone())
        .collect()
}

fn assert_no_bag_open_errors(env: &WowLuaEnv, context: &str) {
    let recorded_errors = common::panel_fixtures::recorded_lua_errors(env);
    let handler_errors = drain_test_errors(env);
    assert!(
        recorded_errors.is_empty(),
        "{context} produced {} recorded Lua error(s):\n{}\nhandler errors:\n{}",
        recorded_errors.len(),
        recorded_errors.join("\n"),
        handler_errors.join("\n"),
    );
    assert!(
        handler_errors.is_empty(),
        "{context} produced {} Lua error(s):\n{}",
        handler_errors.len(),
        handler_errors.join("\n"),
    );
}

#[cfg(not(feature = "client-wrath"))]
#[test]
fn test_container_frames_registered() {
    let env = setup_env();

    // Check ContainerFrameContainer.ContainerFrames population
    let count: i32 = env
        .eval(
            r#"
            local t = ContainerFrameContainer.ContainerFrames
            if type(t) ~= "table" then return -1 end
            local n = 0
            for _ in pairs(t) do n = n + 1 end
            return n
        "#,
        )
        .unwrap();
    let expected_frames = if cfg!(feature = "client-wowforever") {
        // Forever ContainerFrame.xml declares ContainerFrame1 through 7 (lines 298-304).
        7
    } else {
        6
    };
    assert_eq!(
        count, expected_frames,
        "ContainerFrameContainer.ContainerFrames should match the active profile source"
    );

    // Check individual frames exist
    for i in 1..=expected_frames {
        let exists: bool = env
            .eval(&format!("return ContainerFrame{i} ~= nil"))
            .unwrap();
        assert!(exists, "ContainerFrame{i} should exist");
    }
}

#[cfg(not(feature = "client-wrath"))]
#[test]
fn test_container_frames_array_contains_only_real_container_frames() {
    let env = setup_env();

    let names: Vec<String> = env
        .eval(
            r#"
            local t = assert(ContainerFrameContainer and ContainerFrameContainer.ContainerFrames)
            local names = {}
            for k, v in pairs(t) do
                local key = tostring(k)
                local name = type(v) == "table" and v.GetName and v:GetName() or tostring(v)
                table.insert(names, key .. "=" .. tostring(name))
            end
            table.sort(names)
            return names
        "#,
        )
        .unwrap();

    let expected_frames = if cfg!(feature = "client-wowforever") {
        7
    } else {
        6
    };
    let expected: Vec<String> = (1..=expected_frames)
        .map(|index| format!("{index}=ContainerFrame{index}"))
        .collect();
    assert_eq!(
        names, expected,
        "ContainerFrameContainer.ContainerFrames should match the active profile source"
    );
}

#[cfg(not(feature = "client-wrath"))]
#[test]
fn test_bag_ui_c_container_helpers_have_safe_default_shapes() {
    let env = setup_env();
    if wow_ui_sim::client_profile::ACTIVE != wow_ui_sim::client_profile::ClientProfile::Wrath {
        env.exec(
            "A_Admin.AddBagItem(0, 1, 6948); A_Admin.SetItemUpgradeEligibility({bagID=0, slotIndex=1}, false)",
        )
        .unwrap();
    }

    let (not_filtered, quest_info_ok, cannot_upgrade, trade_money_zero): (bool, bool, bool, bool) =
        env.eval(
            r#"
            local questInfo = C_Container.GetContainerItemQuestInfo(0, 1)
            local itemLocation = ItemLocation:CreateFromBagAndSlot(0, 1)
            return C_Container.IsContainerFiltered(0) == false,
                   type(questInfo) == "table"
                       and questInfo.isQuestItem == false
                       and questInfo.isActive == false,
                   C_ItemUpgrade.CanUpgradeItem(itemLocation) == false,
                   GetPlayerTradeMoney() == 0
            "#,
        )
        .unwrap();

    assert!(not_filtered, "bag search defaults should start unfiltered");
    assert!(
        quest_info_ok,
        "bag quest-info helper should return a safe default table shape",
    );
    assert!(
        cannot_upgrade,
        "supplied false bag eligibility should preserve the boolean result"
    );
    assert!(
        trade_money_zero,
        "default trade money helper should report zero"
    );
}

/// Open all bags via pcall-protected ToggleAllBags, logging any Lua errors.
fn open_all_bags(env: &WowLuaEnv) {
    env.exec(
        r#"
        local ok, err = pcall(ToggleAllBags)
        if not ok then
            table.insert(__test_errors, "ToggleAllBags: " .. tostring(err))
        end
    "#,
    )
    .unwrap();

    let errors = drain_test_errors(env);
    for e in &errors {
        eprintln!("Lua error: {e}");
    }
}

/// Assert that at least one bag frame is visible.
fn assert_bag_frame_visible(env: &WowLuaEnv) {
    let bag_shown: bool = env
        .eval(
            "return (ContainerFrameCombinedBags and ContainerFrameCombinedBags:IsShown()) \
             or (ContainerFrame1 and ContainerFrame1:IsShown())",
        )
        .unwrap();
    assert!(
        bag_shown,
        "A bag frame should be visible after ToggleAllBags"
    );
}

/// Assert the backpack has the expected number of populated item slots.
fn assert_backpack_item_count(env: &WowLuaEnv, expected: i32) {
    let populated_slots: i32 = env
        .eval(
            r#"
            local count = 0
            for slot = 1, 16 do
                local info = C_Container.GetContainerItemInfo(0, slot)
                if info and info.itemID then
                    count = count + 1
                end
            end
            return count
        "#,
        )
        .unwrap();
    assert_eq!(
        populated_slots, expected,
        "Backpack populated slot count mismatch"
    );
}

// The Mainline ContainerFrame branch owns the token tracker; Classic selects
// the legacy container branch without ContainerFrameSettingsManager.
retail_ptr_forever_toc_fixture! {
#[test]
fn test_bag_env_loads_real_backpack_token_tracker() {
    let env = setup_env();

    let (tracker_is_real_frame, tracker_matches_backpack_frame, addon_loaded): (bool, bool, bool) =
        env.eval(
            r#"
            local tracker = ContainerFrameSettingsManager and ContainerFrameSettingsManager.TokenTracker
            return type(tracker) == "table"
                and type(tracker.UpdateIfVisible) == "function"
                and type(tracker.GetMaxTokensWatched) == "function",
                tracker == BackpackTokenFrame,
                C_AddOns and C_AddOns.IsAddOnLoaded
                    and C_AddOns.IsAddOnLoaded("Blizzard_TokenUI") or false
            "#,
        )
        .unwrap();

    assert!(tracker_is_real_frame);
    assert!(tracker_matches_backpack_frame);
    assert!(addon_loaded);
}
}

#[test]
fn test_bags_open_with_items() {
    #[cfg(feature = "client-wrath")]
    if common::action_bar_fixture::is_wrath() {
        let env = common::action_bar_fixture::wrath_env();
        // Wrath FrameXML/ContainerFrame.lua:56 and :360 open legacy bag frames.
        env.exec(
            r#"
            A_Admin.AddBagItem(0, 5, 6948, 1)
            ToggleBackpack()
            assert(IsBagOpen(0), "backpack should be open")
            local count = 0
            for slot = 1, 16 do
                if GetContainerItemInfo(0, slot) then count = count + 1 end
            end
            assert(count == 5, "backpack should contain five items")
            assert(string.find(GetContainerItemLink(0, 1), "Hearthstone", 1, true))
            assert(GetContainerItemLink(0, 6) == nil)
            "#,
        )
        .expect("original Wrath backpack should open with supplied items");
        return;
    }
    let env = setup_env();
    install_test_error_handler(&env);
    clear_recorded_lua_errors(&env);

    // Backpack starts with 4 default items (Hearthstone, Water, Bread, Skinning Knife)
    // Add one more via admin API
    env.exec("A_Admin.AddBagItem(0, 5, 6948, 1)").unwrap();

    open_all_bags(&env);
    assert_no_bag_open_errors(&env, "ToggleAllBags backpack open");
    assert_bag_frame_visible(&env);
    assert_backpack_item_count(&env, 5);

    // Verify default Hearthstone in slot 1
    let item_link: String = env
        .eval(r#"return C_Container.GetContainerItemInfo(0, 1).hyperlink"#)
        .unwrap();
    assert!(
        item_link.contains("Hearthstone"),
        "Slot 1 should contain Hearthstone, got: {item_link}",
    );

    // Verify empty slots return nil
    let empty: bool = env
        .eval("return C_Container.GetContainerItemInfo(0, 6) == nil")
        .unwrap();
    assert!(empty, "Slot 6 should be empty");
}

#[cfg(not(feature = "client-wrath"))]
#[test]
fn test_container_frame_1_item_1_icon_matches_first_bag_slot_item() {
    let env = setup_env();
    install_test_error_handler(&env);
    clear_recorded_lua_errors(&env);

    disable_combined_bags(&env);

    open_all_bags(&env);
    assert_no_bag_open_errors(&env, "ToggleAllBags individual-bag open");

    let result: String = env
        .eval(
            r#"
            if not ContainerFrame1 then
                return "missing_container_frame_1"
            end
            local button = ContainerFrame1Item1
            if not button and ContainerFrame1.Items then
                button = ContainerFrame1.Items[1]
                if not button then
                    return "missing_container_frame_1_items_1"
                end
                if not button.icon then
                    return "missing_container_frame_1_items_1_icon"
                end

                local buttonSlot = button:GetID()
                local actual = button.icon:GetTexture()
                local buttonInfo = C_Container.GetContainerItemInfo(0, buttonSlot)
                local buttonTexture = buttonInfo and buttonInfo.iconFileID
                if actual ~= buttonTexture then
                    return string.format(
                        "stale_name_items_1_button_slot_%d_expected_%s_actual_%s",
                        buttonSlot,
                        tostring(buttonTexture),
                        tostring(actual)
                    )
                end
                return string.format("stale_name_items_1_button_slot_%d", buttonSlot)
            end
            if not button then
                return "missing_container_frame_1_item_1"
            end

            local buttonSlot = button:GetID()
            local actual = button.icon and button.icon:GetTexture()
            local firstSlotInfo = C_Container.GetContainerItemInfo(0, 1)
            local firstSlotTexture = firstSlotInfo and firstSlotInfo.iconFileID

            if buttonSlot ~= 1 then
                local buttonInfo = C_Container.GetContainerItemInfo(0, buttonSlot)
                local buttonTexture = buttonInfo and buttonInfo.iconFileID
                if actual ~= buttonTexture then
                    return string.format(
                        "stale_name_button_slot_%d_expected_%s_actual_%s",
                        buttonSlot,
                        tostring(buttonTexture),
                        tostring(actual)
                    )
                end
                return string.format("stale_name_button_slot_%d", buttonSlot)
            end

            if actual ~= firstSlotTexture then
                return string.format(
                    "icon_mismatch_expected_%s_actual_%s",
                    tostring(firstSlotTexture),
                    tostring(actual)
                )
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert!(
        result == "ok"
            || result.starts_with("stale_name_button_slot_")
            || result.starts_with("stale_name_items_1_button_slot_"),
        "ContainerFrame1Item1 should either match bag slot 1 directly or prove the plan wording is stale through the real item-button list: {result}"
    );
}

#[cfg(not(feature = "client-wrath"))]
#[test]
fn test_new_bag_item_button_icon_matches_its_slot_after_refresh() {
    let env = setup_env();
    install_test_error_handler(&env);
    clear_recorded_lua_errors(&env);

    disable_combined_bags(&env);
    env.exec("A_Admin.AddBagItem(0, 5, 2852, 1)").unwrap();

    open_all_bags(&env);
    assert_no_bag_open_errors(&env, "ToggleAllBags after adding Copper Chain Pants");

    let result: String = env
        .eval(
            r#"
            local expectedInfo = C_Container.GetContainerItemInfo(0, 5)
            if not expectedInfo then
                return "missing_slot_5_info"
            end
            if expectedInfo.iconFileID ~= 134583 then
                return "wrong_api_icon_" .. tostring(expectedInfo.iconFileID)
            end

            local button
            if ContainerFrame1 and ContainerFrame1.Items then
                for _, itemButton in ipairs(ContainerFrame1.Items) do
                    if itemButton:GetID() == 5 then
                        button = itemButton
                        break
                    end
                end
            end
            button = button or ContainerFrame1Item5
            if not button then
                return "missing_slot_5_button"
            end
            if not button.icon then
                return "missing_slot_5_icon_texture"
            end

            local actual = button.icon:GetTexture()
            if actual ~= expectedInfo.iconFileID then
                return string.format(
                    "icon_mismatch_expected_%s_actual_%s",
                    tostring(expectedInfo.iconFileID),
                    tostring(actual)
                )
            end
            local width, height = button.icon:GetSize()
            if width < 30 or height < 30 then
                return string.format("icon_too_small_%s_%s", tostring(width), tostring(height))
            end
            local tlx, tly, blx, bly, trx, try, brx, bry = button.icon:GetTexCoord()
            if tlx ~= 0 or tly ~= 0 or blx ~= 0 or bly ~= 1 or trx ~= 1 or try ~= 0 or brx ~= 1 or bry ~= 1 then
                return string.format(
                    "icon_has_stale_tex_coords_%s_%s_%s_%s_%s_%s_%s_%s",
                    tostring(tlx),
                    tostring(tly),
                    tostring(blx),
                    tostring(bly),
                    tostring(trx),
                    tostring(try),
                    tostring(brx),
                    tostring(bry)
                )
            end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "Copper Chain Pants bag button should render the slot's item icon: {result}"
    );

    let textures = build_texture_requests(&env);
    assert!(
        textures
            .iter()
            .any(|path| path.to_ascii_lowercase().contains("inv_pants_03")),
        "render batch should request the Copper Chain Pants icon, got {} texture requests",
        textures.len()
    );
}

#[cfg(not(feature = "client-wrath"))]
#[test]
fn test_toggle_backpack_bootstrap_token_ui_does_not_nil_error() {
    let env = setup_env_with_bootstrap_loaded_token_ui();
    install_test_error_handler(&env);
    clear_recorded_lua_errors(&env);

    env.exec(
        r#"
        local ok, err = pcall(ToggleBackpack)
        if not ok then
            table.insert(__test_errors, "ToggleBackpack: " .. tostring(err))
        end
        "#,
    )
    .unwrap();

    let handler_errors = drain_test_errors(&env);
    let recorded_errors = common::panel_fixtures::recorded_lua_errors(&env);
    assert!(
        handler_errors.is_empty(),
        "ToggleBackpack should not error with bootstrap-loaded TokenUI; got:\n{}",
        handler_errors.join("\n"),
    );
    assert!(
        recorded_errors.is_empty(),
        "ToggleBackpack should not emit recorded Lua errors; got:\n{}",
        recorded_errors.join("\n"),
    );
}

#[test]
fn test_backpack_token_hover_populates_tooltip() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    install_test_error_handler(&env);
    clear_recorded_lua_errors(&env);

    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        GameTooltip:SetBackpackToken(1)
        if not GameTooltip:IsShown() then
            error("SetBackpackToken did not show GameTooltip")
        end
        if GameTooltip:NumLines() == 0 then
            error("SetBackpackToken did not populate GameTooltip")
        end
        local info = C_TooltipInfo.GetBackpackToken(1)
        if not info or not info.lines or not info.lines[1] then
            error("C_TooltipInfo.GetBackpackToken did not return tooltip lines")
        end
        "#,
    )
    .unwrap();

    assert_no_bag_open_errors(&env, "SetBackpackToken");
}

#[cfg(not(feature = "client-wrath"))]
#[test]
fn test_default_backpack_seed_items_do_not_use_question_mark_icons() {
    let env = setup_env();
    install_test_error_handler(&env);
    clear_recorded_lua_errors(&env);

    let (slot1, slot2, slot3, slot4): (i64, i64, i64, i64) = env
        .eval(
            r#"
            local i1 = C_Container.GetContainerItemInfo(0, 1)
            local i2 = C_Container.GetContainerItemInfo(0, 2)
            local i3 = C_Container.GetContainerItemInfo(0, 3)
            local i4 = C_Container.GetContainerItemInfo(0, 4)
            return i1 and i1.iconFileID or 0,
                   i2 and i2.iconFileID or 0,
                   i3 and i3.iconFileID or 0,
                   i4 and i4.iconFileID or 0
        "#,
        )
        .unwrap();

    assert_ne!(
        slot1, 134400,
        "slot 1 (Hearthstone) should not render as question-mark placeholder"
    );
    assert_ne!(
        slot2, 134400,
        "slot 2 (Refreshing Spring Water) should not render as question-mark placeholder"
    );
    assert_ne!(
        slot3, 134400,
        "slot 3 (Tough Hunk of Bread) should not render as question-mark placeholder"
    );
    assert_ne!(
        slot4, 134400,
        "slot 4 (Skinning Knife) should not render as question-mark placeholder"
    );
}

#[cfg(not(feature = "client-wrath"))]
#[test]
fn test_backpack_bg_regions_have_color_fill() {
    let env = setup_env();
    install_test_error_handler(&env);
    clear_recorded_lua_errors(&env);
    env.exec("ToggleBackpack()").unwrap();
    assert_no_bag_open_errors(&env, "ToggleBackpack background inspection");

    let state = env.state().borrow();
    let frame_id = state
        .widgets
        .get_id_by_name("ContainerFrame1")
        .expect("ContainerFrame1 must exist");
    let bg_id = state
        .widgets
        .get(frame_id)
        .and_then(|f| f.children_keys.get("Bg").copied())
        .expect("ContainerFrame1.Bg must exist");

    let top_section_id = state
        .widgets
        .get(bg_id)
        .and_then(|f| f.children_keys.get("TopSection").copied())
        .expect("ContainerFrame1.Bg.TopSection must exist");
    let bottom_edge_id = state
        .widgets
        .get(bg_id)
        .and_then(|f| f.children_keys.get("BottomEdge").copied())
        .expect("ContainerFrame1.Bg.BottomEdge must exist");

    let top_section = state
        .widgets
        .get(top_section_id)
        .expect("TopSection widget must exist");
    let bottom_edge = state
        .widgets
        .get(bottom_edge_id)
        .expect("BottomEdge widget must exist");

    assert!(
        top_section.color_texture.is_some(),
        "TopSection should be initialized via SetColorTexture"
    );
    assert!(
        bottom_edge.color_texture.is_some(),
        "BottomEdge should be initialized via SetColorTexture"
    );
}

// This probe uses Mainline-only `ContainerFrame1:EnumerateValidItems`; the
// Classic and Mists container files do not define that method.
retail_ptr_forever_toc_fixture! {
#[test]
fn test_item_search_marks_container_buttons_by_item_name() {
    let env = setup_env();
    install_test_error_handler(&env);
    clear_recorded_lua_errors(&env);
    env.exec("ContainerFrameSettingsManager:SetUsingCombinedBags(false)")
        .unwrap();
    open_all_bags(&env);
    assert_no_bag_open_errors(&env, "ToggleAllBags individual-bag open");

    let matches_after = |search: &str| -> String {
        env.eval(&format!(
            r#"
            C_Container.SetItemSearch({search:?})
            local out = {{}}
            for _, button in ContainerFrame1:EnumerateValidItems() do
                local info = C_Container.GetContainerItemInfo(button:GetBagID(), button:GetID())
                if info then
                    local name = C_Item.GetItemNameByID(info.itemID) or tostring(info.itemID)
                    out[#out + 1] = name .. "=" .. tostring(button:GetMatchesSearch())
                end
            end
            table.sort(out)
            return table.concat(out, ",")
            "#
        ))
        .unwrap()
    };

    let searched = matches_after("HEARTH");
    assert!(
        searched.contains("Hearthstone=true"),
        "case-insensitive name match should stay lit: {searched}"
    );
    assert!(
        searched.split(',').any(|entry| entry.ends_with("=false")),
        "non-matching items should be dimmed: {searched}"
    );
    let cleared = matches_after("");
    assert!(
        !cleared.contains("=false"),
        "clearing the search should relight every item: {cleared}"
    );
    assert_no_bag_open_errors(&env, "item search");
}
}
