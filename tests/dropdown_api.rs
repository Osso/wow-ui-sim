//! Tests for UIDropDownMenu system (dropdown_api.rs).

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// Global Constants
// ============================================================================

#[test]
fn test_dropdown_constants_exist() {
    let env = env();
    let max_buttons: i32 = env.eval("return UIDROPDOWNMENU_MAXBUTTONS").unwrap();
    assert_eq!(max_buttons, 1);
    let max_levels: i32 = env.eval("return UIDROPDOWNMENU_MAXLEVELS").unwrap();
    assert_eq!(max_levels, 3);
    let btn_height: i32 = env.eval("return UIDROPDOWNMENU_BUTTON_HEIGHT").unwrap();
    assert_eq!(btn_height, 16);
    let menu_level: i32 = env.eval("return UIDROPDOWNMENU_MENU_LEVEL").unwrap();
    assert_eq!(menu_level, 1);
    let show_time: i32 = env.eval("return UIDROPDOWNMENU_SHOW_TIME").unwrap();
    assert_eq!(show_time, 2);
}

#[test]
fn test_open_dropdownmenus_is_table() {
    let env = env();
    let is_table: bool = env
        .eval("return type(OPEN_DROPDOWNMENUS) == 'table'")
        .unwrap();
    assert!(is_table);
}

// ============================================================================
// DropDownList Frames
// ============================================================================

#[test]
fn test_dropdown_list_frames_exist() {
    let env = env();
    for level in 1..=3 {
        let exists: bool = env
            .eval(&format!("return DropDownList{} ~= nil", level))
            .unwrap();
        assert!(exists, "DropDownList{} should exist", level);
    }
}

#[test]
fn test_dropdown_list_is_hidden_initially() {
    let env = env();
    let visible: bool = env.eval("return DropDownList1:IsVisible()").unwrap();
    assert!(!visible, "DropDownList1 should be hidden initially");
}

#[test]
fn test_dropdown_list_buttons_exist() {
    let env = env();
    for level in 1..=3 {
        for btn in 1..=8 {
            let exists: bool = env
                .eval(&format!(
                    "return DropDownList{}Button{} ~= nil",
                    level, btn
                ))
                .unwrap();
            assert!(
                exists,
                "DropDownList{}Button{} should exist",
                level, btn
            );
        }
    }
}

#[test]
fn test_dropdown_list_button_text_frames_exist() {
    let env = env();
    let exists: bool = env
        .eval("return DropDownList1Button1NormalText ~= nil")
        .unwrap();
    assert!(exists, "Button NormalText child should exist");
}

#[test]
fn test_dropdown_list_num_buttons_initially_zero() {
    let env = env();
    let num: i32 = env.eval("return DropDownList1.numButtons").unwrap();
    assert_eq!(num, 0);
}

// ============================================================================
// UIDropDownMenu_CreateInfo
// ============================================================================

#[test]
fn test_create_info_returns_table() {
    let env = env();
    let is_table: bool = env
        .eval("return type(UIDropDownMenu_CreateInfo()) == 'table'")
        .unwrap();
    assert!(is_table);
}

// ============================================================================
// UIDropDownMenu_Initialize
// ============================================================================

#[test]
fn test_initialize_calls_init_function() {
    let env = env();
    env.exec(
        r#"
        INIT_CALLED = false
        INIT_LEVEL = nil
        local frame = CreateFrame("Frame", "TestDropDown", UIParent)
        UIDropDownMenu_Initialize(frame, function(self, level)
            INIT_CALLED = true
            INIT_LEVEL = level
        end)
    "#,
    )
    .unwrap();
    let called: bool = env.eval("return INIT_CALLED").unwrap();
    assert!(called, "Init function should be called during Initialize");
    let level: i32 = env.eval("return INIT_LEVEL").unwrap();
    assert_eq!(level, 1, "Default level should be 1");
}

#[test]
fn test_initialize_stores_init_function() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestDropDown2", UIParent)
        UIDropDownMenu_Initialize(frame, function(self, level) end)
    "#,
    )
    .unwrap();
    let has_init: bool = env
        .eval("return TestDropDown2.initialize ~= nil")
        .unwrap();
    assert!(has_init, "Init function should be stored on frame");
}

// ============================================================================
// UIDropDownMenu_AddButton
// ============================================================================

#[test]
fn test_add_button_increments_num_buttons() {
    let env = env();
    env.exec(
        r#"
        local info = UIDropDownMenu_CreateInfo()
        info.text = "Option 1"
        UIDropDownMenu_AddButton(info, 1)
    "#,
    )
    .unwrap();
    let num: i32 = env.eval("return DropDownList1.numButtons").unwrap();
    assert_eq!(num, 1);
}

#[test]
fn test_add_button_sets_text() {
    let env = env();
    env.exec(
        r#"
        local info = UIDropDownMenu_CreateInfo()
        info.text = "Test Option"
        UIDropDownMenu_AddButton(info, 1)
    "#,
    )
    .unwrap();
    let text: String = env
        .eval("return DropDownList1Button1:GetText()")
        .unwrap();
    assert_eq!(text, "Test Option");
}

#[test]
fn test_add_button_copies_info_properties() {
    let env = env();
    env.exec(
        r#"
        local info = UIDropDownMenu_CreateInfo()
        info.text = "With Arrow"
        info.hasArrow = true
        info.value = 42
        UIDropDownMenu_AddButton(info, 1)
    "#,
    )
    .unwrap();
    let has_arrow: bool = env
        .eval("return DropDownList1Button1.hasArrow")
        .unwrap();
    assert!(has_arrow);
    let value: i32 = env.eval("return DropDownList1Button1.value").unwrap();
    assert_eq!(value, 42);
}

// ============================================================================
// UIDropDownMenu_SetWidth / SetText / GetText
// ============================================================================

#[test]
fn test_set_width() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestDropDownWidth", UIParent)
        UIDropDownMenu_SetWidth(frame, 200)
    "#,
    )
    .unwrap();
    let width: f64 = env.eval("return TestDropDownWidth:GetWidth()").unwrap();
    assert!((width - 200.0).abs() < 0.1);
}

#[test]
fn test_set_and_get_text() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestDropDownText", UIParent)
        UIDropDownMenu_SetText(frame, "Hello")
    "#,
    )
    .unwrap();
    let text: String = env
        .eval("return UIDropDownMenu_GetText(TestDropDownText)")
        .unwrap();
    assert_eq!(text, "Hello");
}

// ============================================================================
// UIDropDownMenu_SetSelectedID / GetSelectedID
// ============================================================================

#[test]
fn test_set_and_get_selected_id() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestSelID", UIParent)
        UIDropDownMenu_SetSelectedID(frame, 3)
    "#,
    )
    .unwrap();
    let id: i32 = env
        .eval("return UIDropDownMenu_GetSelectedID(TestSelID)")
        .unwrap();
    assert_eq!(id, 3);
}

// ============================================================================
// UIDropDownMenu_SetSelectedValue / GetSelectedValue
// ============================================================================

#[test]
fn test_set_and_get_selected_value() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestSelVal", UIParent)
        UIDropDownMenu_SetSelectedValue(frame, "myval")
    "#,
    )
    .unwrap();
    let val: String = env
        .eval("return UIDropDownMenu_GetSelectedValue(TestSelVal)")
        .unwrap();
    assert_eq!(val, "myval");
}

// ============================================================================
// ToggleDropDownMenu / CloseDropDownMenus
// ============================================================================

#[test]
fn test_toggle_dropdown_menu_shows_list() {
    let env = env();
    let before: bool = env.eval("return DropDownList1:IsVisible()").unwrap();
    assert!(!before);

    env.exec("ToggleDropDownMenu(1)").unwrap();
    let after: bool = env.eval("return DropDownList1:IsVisible()").unwrap();
    assert!(after, "DropDownList1 should be visible after toggle");
}

#[test]
fn test_toggle_dropdown_menu_hides_on_second_toggle() {
    let env = env();
    env.exec("ToggleDropDownMenu(1)").unwrap();
    env.exec("ToggleDropDownMenu(1)").unwrap();
    let visible: bool = env.eval("return DropDownList1:IsVisible()").unwrap();
    assert!(!visible, "DropDownList1 should be hidden after double toggle");
}

#[test]
fn test_close_dropdown_menus() {
    let env = env();
    env.exec("ToggleDropDownMenu(1)").unwrap();
    env.exec("ToggleDropDownMenu(2)").unwrap();

    let l1_vis: bool = env.eval("return DropDownList1:IsVisible()").unwrap();
    let l2_vis: bool = env.eval("return DropDownList2:IsVisible()").unwrap();
    assert!(l1_vis);
    assert!(l2_vis);

    env.exec("CloseDropDownMenus()").unwrap();
    let l1_after: bool = env.eval("return DropDownList1:IsVisible()").unwrap();
    let l2_after: bool = env.eval("return DropDownList2:IsVisible()").unwrap();
    assert!(!l1_after);
    assert!(!l2_after);
}

#[test]
fn test_close_dropdown_menus_from_level() {
    let env = env();
    env.exec("ToggleDropDownMenu(1)").unwrap();
    env.exec("ToggleDropDownMenu(2)").unwrap();

    // Close only level 2+
    env.exec("CloseDropDownMenus(2)").unwrap();
    let l1_vis: bool = env.eval("return DropDownList1:IsVisible()").unwrap();
    let l2_vis: bool = env.eval("return DropDownList2:IsVisible()").unwrap();
    assert!(l1_vis, "Level 1 should still be visible");
    assert!(!l2_vis, "Level 2 should be closed");
}

// ============================================================================
// UIDropDownMenu_EnableDropDown / DisableDropDown
// ============================================================================

#[test]
fn test_enable_disable_dropdown() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestEnableDD", UIParent)
        UIDropDownMenu_DisableDropDown(frame)
    "#,
    )
    .unwrap();
    // The attribute is stored internally — we just verify no errors occur
    env.exec("UIDropDownMenu_EnableDropDown(TestEnableDD)")
        .unwrap();
}

// ============================================================================
// UIDropDownMenu_AddSeparator / AddSpace
// ============================================================================

#[test]
fn test_add_separator_adds_button() {
    let env = env();
    // Reset numButtons first by using level 2
    let before: i32 = env.eval("return DropDownList2.numButtons").unwrap();
    env.exec("UIDropDownMenu_AddSeparator(2)").unwrap();
    let after: i32 = env.eval("return DropDownList2.numButtons").unwrap();
    assert_eq!(after, before + 1);
}

#[test]
fn test_add_space_adds_button() {
    let env = env();
    let before: i32 = env.eval("return DropDownList3.numButtons").unwrap();
    env.exec("UIDropDownMenu_AddSpace(3)").unwrap();
    let after: i32 = env.eval("return DropDownList3.numButtons").unwrap();
    assert_eq!(after, before + 1);
}

// ============================================================================
// UIDropDownMenu_GetCurrentDropDown / IsOpen
// ============================================================================

#[test]
fn test_get_current_dropdown_nil_initially() {
    let env = env();
    let is_nil: bool = env
        .eval("return UIDropDownMenu_GetCurrentDropDown() == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_is_open_false_when_closed() {
    let env = env();
    env.exec(r#"local frame = CreateFrame("Frame", "TestIsOpenDD", UIParent)"#)
        .unwrap();
    let open: bool = env
        .eval("return UIDropDownMenu_IsOpen(TestIsOpenDD)")
        .unwrap();
    assert!(!open);
}

// ============================================================================
// No-op functions don't error
// ============================================================================

#[test]
fn test_noop_functions_dont_error() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestNoopDD", UIParent)
        UIDropDownMenu_Refresh(frame)
        UIDropDownMenu_JustifyText(frame, "LEFT")
        UIDropDownMenu_HandleGlobalMouseEvent("LeftButton", "GLOBAL_MOUSE_DOWN")
    "#,
    )
    .unwrap();
}
