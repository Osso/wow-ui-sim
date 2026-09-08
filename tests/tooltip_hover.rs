//! Full-environment tooltip tests requiring Blizzard addon loading.
//!
//! Tests that need the micro menu, OnEnter scripts, or the render pipeline
//! live here. Basic tooltip API tests stay in tooltip.rs.

mod tooltip_full_env_helpers;
mod tooltip_hover_helpers;

use tooltip_full_env_helpers::setup_full_env;
use tooltip_hover_helpers::{open_character_panel, refresh_buff_frame};
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn action_button_hover_preserves_spell_identity_for_tooltip_postcall() {
    let env = setup_full_env();
    let button_id = {
        let mut state = env.state().borrow_mut();
        state.action_bars.insert(5, 45524);
        let id = state.widgets.get_id_by_name("ActionButton5").unwrap();
        state.hovered_frame = Some(id);
        id
    };
    env.exec(
        r#"
        ActionButton5.action = 5
        ActionButton5:Show()
        ClickedStylePostcall = { calls = 0 }
        TooltipDataProcessor.AddTooltipPostCall(Enum.TooltipDataType.Spell, function(tooltip)
            if tooltip:IsForbidden() or tooltip.GetSpell == nil then return end
            local name, spellId = tooltip:GetSpell()
            if spellId == nil or issecretvalue(spellId) then return end
            ClickedStylePostcall.calls = ClickedStylePostcall.calls + 1
            ClickedStylePostcall.id = spellId
            ClickedStylePostcall.name = name
            tooltip:AddLine("Bound to test key")
        end)
        "#,
    )
    .unwrap();
    env.state().borrow_mut().lua_errors.clear();

    env.fire_script_handler(button_id, "OnEnter", vec![])
        .expect("actual ActionButton OnEnter should finish");

    env.exec(
        r#"
        assert(ClickedStylePostcall.calls > 0, "spell postcall never completed")
        assert(ClickedStylePostcall.id == 45524, "postcall lost action spell identity")
        assert(ClickedStylePostcall.name == C_Spell.GetSpellName(45524))
        assert(GameTooltip:GetPrimaryTooltipData().id == 45524)
        assert(GameTooltip:IsShown())
        local found = false
        for i = 1, GameTooltip:NumLines() do
            if GameTooltip:GetLeftLine(i):GetText() == "Bound to test key" then found = true end
        end
        assert(found, "postcall could not append its binding line")
        "#,
    )
    .expect("native TooltipDataHandler and GetSpell preserve action identity for addon postcalls");
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "hover must not record a swallowed postcall error"
    );
}

#[test]
fn test_tooltip_mixins_load_in_full_env() {
    let env = setup_full_env();

    let tooltip_data_handler_mixin_type: String =
        env.eval("return type(TooltipDataHandlerMixin)").unwrap();
    let game_tooltip_data_mixin_type: String =
        env.eval("return type(GameTooltipDataMixin)").unwrap();
    let status_bar_onload_type: String = env
        .eval("return type(GameTooltipStatusBar.OnLoad)")
        .unwrap();
    let tooltip_onload_type: String = env.eval("return type(GameTooltip.OnLoad)").unwrap();

    assert_eq!(tooltip_data_handler_mixin_type, "table");
    assert_eq!(game_tooltip_data_mixin_type, "table");
    assert_eq!(status_bar_onload_type, "function");
    assert_eq!(tooltip_onload_type, "function");
}

#[test]
fn test_game_tooltip_process_info_with_constructed_tooltip_info_populates_lines() {
    let env = setup_full_env();

    let processed: bool = env
        .eval(
            r#"
            GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
            local tooltipInfo = {
                getterName = "GetItemByID",
                getterArgs = { [1] = 6948, n = 1 },
            }
            return GameTooltip:ProcessInfo(tooltipInfo)
        "#,
        )
        .unwrap();

    assert!(
        processed,
        "GameTooltip:ProcessInfo should succeed for a valid item tooltipInfo"
    );

    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert!(
        num_lines > 0,
        "GameTooltip:ProcessInfo should populate tooltip lines for a valid tooltipInfo"
    );

    let first_line: String = env
        .eval("return GameTooltip:GetLeftLine(1):GetText()")
        .unwrap();
    assert_eq!(
        first_line, "Hearthstone",
        "GameTooltip:ProcessInfo should set the first tooltip line from the constructed tooltipInfo"
    );
}

/// Test that hovering a micro menu button shows the tooltip with text.
///
/// Uses the full Blizzard UI environment so OnEnter scripts run properly.
#[test]
fn test_micro_menu_hover_shows_tooltip() {
    let env = setup_full_env();

    // Find the CharacterMicroButton frame ID
    let btn_id = {
        let state = env.state().borrow();
        state
            .widgets
            .get_id_by_name("CharacterMicroButton")
            .expect("CharacterMicroButton should exist")
    };

    // Set hovered_frame so IsMouseMotionFocus() returns true
    env.state().borrow_mut().hovered_frame = Some(btn_id);

    // Fire OnEnter (this is what handle_mouse_move does)
    env.fire_script_handler(btn_id, "OnEnter", vec![]).unwrap();

    // Check tooltip state
    let visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();

    assert!(
        visible,
        "GameTooltip should be visible after micro menu hover"
    );
    assert!(
        num_lines > 0,
        "GameTooltip should have at least one line, got {}",
        num_lines
    );

    // Verify the tooltip text content
    {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        let td = state
            .tooltips
            .get(&gt_id)
            .expect("tooltip data should exist");
        assert!(!td.lines.is_empty(), "tooltip should have line data");
        eprintln!("Tooltip text: {:?}", td.lines[0].left_text);
    }

    // Propagate effective_alpha via get_strata_buckets, then verify
    {
        let mut state = env.state().borrow_mut();
        let _ = state.get_strata_buckets();
    }
    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    assert!(
        state
            .widgets
            .get(gt_id)
            .is_some_and(|f| f.effective_alpha > 0.0),
        "GameTooltip should be ancestor-visible (effective_alpha > 0)"
    );

    // Check frame dimensions (tooltip should not be 0x0)
    let frame = state.widgets.get(gt_id).unwrap();
    eprintln!(
        "Tooltip frame: visible={}, width={}, height={}",
        frame.visible, frame.width, frame.height
    );
}

#[test]
fn test_character_slot_hover_shows_inventory_tooltip() {
    let env = setup_full_env();
    open_character_panel(&env);

    let slot_id = {
        let state = env.state().borrow();
        state
            .widgets
            .get_id_by_name("CharacterHeadSlot")
            .expect("CharacterHeadSlot should exist after opening character panel")
    };

    env.state().borrow_mut().hovered_frame = Some(slot_id);
    env.fire_script_handler(slot_id, "OnEnter", vec![]).unwrap();

    let visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();

    assert!(
        visible,
        "GameTooltip should be visible after hovering CharacterHeadSlot"
    );
    assert!(
        num_lines >= 3,
        "Character slot hover should populate item tooltip lines, got {}",
        num_lines
    );

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state
        .tooltips
        .get(&gt_id)
        .expect("tooltip data should exist after character slot hover");
    assert_eq!(td.lines[0].left_text, "Entombed Seraph's Casque");
    assert_eq!(td.lines[1].left_text, "Item Level 571");
    assert_eq!(td.lines[2].left_text, "Head");
}

#[test]
fn test_buff_icon_hover_shows_aura_tooltip() {
    let env = setup_full_env();
    refresh_buff_frame(&env);

    let expected_name: String = env
        .eval(
            r#"
            for _, button in ipairs(BuffFrame.auraFrames) do
                if button:IsShown() and button.buttonInfo and button.buttonInfo.index then
                    button:OnEnter()
                    return C_TooltipInfo.GetUnitAura("player", button.buttonInfo.index, button:GetFilter()).lines[1].leftText
                end
            end
            error("No visible buff icon with tooltip data")
            "#,
        )
        .unwrap();

    let visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();

    assert!(
        visible,
        "GameTooltip should be visible after hovering a buff icon"
    );
    assert!(
        num_lines >= 1,
        "Buff icon hover should populate aura tooltip lines, got {}",
        num_lines
    );

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state
        .tooltips
        .get(&gt_id)
        .expect("tooltip data should exist after buff hover");
    assert_eq!(td.lines[0].left_text, expected_name);
}

#[test]
fn test_world_loot_object_tooltip_shows_world_loot_data() {
    let env = setup_full_env();

    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip.SetWorldLootObject, "GameTooltip:SetWorldLootObject should exist")
        local shown = GameTooltip:SetWorldLootObject("player")
        assert(shown == true, "SetWorldLootObject should report success for the supported unit")
        "#,
    )
    .expect("Failed to show the world loot object tooltip");

    let visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    let getter_name: String = env
        .eval(
            r#"
            local info = GameTooltip:GetPrimaryTooltipInfo()
            return info and info.getterName or ""
            "#,
        )
        .unwrap();

    assert!(
        visible,
        "GameTooltip should be visible after SetWorldLootObject"
    );
    assert!(
        num_lines >= 1,
        "GameTooltip should have world-loot tooltip lines, got {}",
        num_lines
    );
    assert_eq!(getter_name, "GetWorldLootObject");

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state
        .tooltips
        .get(&gt_id)
        .expect("tooltip data should exist after SetWorldLootObject");
    assert!(!td.lines[0].left_text.is_empty());
}

#[test]
fn test_tooltip_comparison_manager_get_comparison_item_data_uses_guid_lookup() {
    let env = setup_full_env();

    let has_real_tooltip: bool = env
        .eval(
            r#"
            assert(type(TooltipComparisonManager) == "table", "TooltipComparisonManager should be loaded")

            local guid = C_Item.GetItemGUID({ bagID = 0, slotIndex = 1 })
            local tooltipData = TooltipComparisonManager:GetComparisonItemData({ guid = guid })
            if not guid or guid == "" or not tooltipData or tooltipData.type ~= Enum.TooltipDataType.Item or not tooltipData.lines then
                return false
            end

            local nameLine = tooltipData.lines[1]
            return tooltipData.guid == guid
                and nameLine
                and nameLine.type == Enum.TooltipDataLineType.ItemName
                and nameLine.leftText == "Hearthstone"
            "#,
        )
        .unwrap();

    assert!(
        has_real_tooltip,
        "TooltipComparisonManager should resolve GUID-backed comparison items through C_TooltipInfo.GetItemByGUID",
    );
}

#[test]
fn test_tooltip_data_handler_set_owned_item_by_id_uses_owned_item_getter() {
    let env = setup_full_env();

    let has_real_tooltip: bool = env
        .eval(
            r#"
            assert(GameTooltip.SetOwnedItemByID, "GameTooltip:SetOwnedItemByID should exist")
            GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
            GameTooltip:SetOwnedItemByID(6948)

            local info = GameTooltip:GetPrimaryTooltipInfo()
            local tooltipData = GameTooltip:GetPrimaryTooltipData()
            if not info or not tooltipData or tooltipData.type ~= Enum.TooltipDataType.Item or not tooltipData.lines then
                return false
            end

            local nameLine = tooltipData.lines[1]
            return info.getterName == "GetOwnedItemByID"
                and nameLine
                and nameLine.type == Enum.TooltipDataLineType.ItemName
                and nameLine.leftText == "Hearthstone"
            "#,
        )
        .unwrap();

    assert!(
        has_real_tooltip,
        "TooltipDataHandler should populate GameTooltip:SetOwnedItemByID through C_TooltipInfo.GetOwnedItemByID",
    );
}

#[test]
fn test_tooltip_data_handler_set_inventory_item_uses_inventory_getter() {
    let env = setup_full_env();

    let has_real_tooltip = inventory_item_tooltip_uses_inventory_getter(&env);

    assert!(
        has_real_tooltip,
        "TooltipDataHandler should populate GameTooltip:SetInventoryItem through C_TooltipInfo.GetInventoryItem",
    );
}

fn inventory_item_tooltip_uses_inventory_getter(env: &WowLuaEnv) -> bool {
    env.eval(
        r#"
        assert(GameTooltip.SetInventoryItem, "GameTooltip:SetInventoryItem should exist")
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        local ok = GameTooltip:SetInventoryItem("player", 1)
        if not ok then
            return false
        end

        local info = GameTooltip:GetPrimaryTooltipInfo()
        local tooltipData = GameTooltip:GetPrimaryTooltipData()
        if not info or not tooltipData or tooltipData.type ~= Enum.TooltipDataType.Item or not tooltipData.lines then
            return false
        end

        local nameLine = tooltipData.lines[1]
        return info.getterName == "GetInventoryItem"
            and nameLine
            and nameLine.type == Enum.TooltipDataLineType.ItemName
            and nameLine.leftText == "Entombed Seraph's Casque"
        "#,
    )
    .unwrap()
}

#[test]
fn test_tooltip_data_handler_set_recipe_result_item_uses_recipe_getters() {
    let env = setup_full_env();

    let recipe_result_ok: bool = env
        .eval(
            r#"
            assert(GameTooltip.SetRecipeResultItem, "GameTooltip:SetRecipeResultItem should exist")
            GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
            GameTooltip:SetRecipeResultItem(100005, {}, nil, nil, nil)

            local info = GameTooltip:GetPrimaryTooltipInfo()
            local tooltipData = GameTooltip:GetPrimaryTooltipData()
            if not info or not tooltipData or tooltipData.type ~= Enum.TooltipDataType.Item or not tooltipData.lines then
                return false
            end

            local nameLine = tooltipData.lines[1]
            return info.getterName == "GetRecipeResultItem"
                and nameLine
                and nameLine.type == Enum.TooltipDataLineType.ItemName
                and nameLine.leftText == "Ordained Forge Maul"
            "#,
        )
        .unwrap();

    let order_result_ok: bool = env
        .eval(
            r#"
            assert(GameTooltip.SetRecipeResultItemForOrder, "GameTooltip:SetRecipeResultItemForOrder should exist")
            GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
            GameTooltip:SetRecipeResultItemForOrder(100005, {}, 1, nil, nil)

            local info = GameTooltip:GetPrimaryTooltipInfo()
            local tooltipData = GameTooltip:GetPrimaryTooltipData()
            if not info or not tooltipData or tooltipData.type ~= Enum.TooltipDataType.Item or not tooltipData.lines then
                return false
            end

            local nameLine = tooltipData.lines[1]
            return info.getterName == "GetRecipeResultItemForOrder"
                and nameLine
                and nameLine.leftText == "Ordained Forge Maul"
            "#,
        )
        .unwrap();

    assert!(
        recipe_result_ok && order_result_ok,
        "TooltipDataHandler should populate both recipe result item accessors through C_TooltipInfo",
    );
}

#[test]
fn test_tooltip_data_handler_set_minimap_mouseover_uses_minimap_getter() {
    let env = setup_full_env();

    let has_real_tooltip: bool = env
        .eval(
            r#"
            assert(GameTooltip.SetMinimapMouseover, "GameTooltip:SetMinimapMouseover should exist")
            GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
            GameTooltip:SetMinimapMouseover()

            local info = GameTooltip:GetPrimaryTooltipInfo()
            local tooltipData = GameTooltip:GetPrimaryTooltipData()
            if not info or not tooltipData or tooltipData.type ~= Enum.TooltipDataType.MinimapMouseover or not tooltipData.lines then
                return false
            end

            local zoneLine = tooltipData.lines[1]
            local subZoneLine = tooltipData.lines[2]
            return info.getterName == "GetMinimapMouseover"
                and zoneLine
                and zoneLine.leftText == "Stormwind City"
                and subZoneLine
                and subZoneLine.leftText == "Trade District"
            "#,
        )
        .unwrap();

    assert!(
        has_real_tooltip,
        "TooltipDataHandler should populate GameTooltip:SetMinimapMouseover through C_TooltipInfo.GetMinimapMouseover",
    );
}

#[test]
fn test_tooltip_data_handler_set_upgrade_item_uses_upgrade_getter() {
    let env = setup_full_env();

    let has_real_tooltip: bool = env
        .eval(
            r#"
            C_ItemUpgrade.SetItemUpgradeFromLocation({ bagID = 0, slotIndex = 1 })
            assert(GameTooltip.SetUpgradeItem, "GameTooltip:SetUpgradeItem should exist")
            GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
            GameTooltip:SetUpgradeItem()

            local info = GameTooltip:GetPrimaryTooltipInfo()
            local tooltipData = GameTooltip:GetPrimaryTooltipData()
            if not info or not tooltipData or tooltipData.type ~= Enum.TooltipDataType.Item or not tooltipData.lines then
                return false
            end

            local nameLine = tooltipData.lines[1]
            local itemLevelLine = tooltipData.lines[2]
            return info.getterName == "GetUpgradeItem"
                and nameLine
                and nameLine.leftText == "Hearthstone"
                and itemLevelLine
                and itemLevelLine.leftText == "Item Level 1"
            "#,
        )
        .unwrap();

    assert!(
        has_real_tooltip,
        "TooltipDataHandler should populate GameTooltip:SetUpgradeItem through C_TooltipInfo.GetUpgradeItem",
    );
}
