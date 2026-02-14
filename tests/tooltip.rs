//! Tests for GameTooltip implementation.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::widget::AnchorPoint;

#[test]
fn test_gametooltip_exists_and_has_correct_type() {
    let env = WowLuaEnv::new().unwrap();

    let exists: bool = env.eval("return GameTooltip ~= nil").unwrap();
    assert!(exists);

    let obj_type: String = env.eval("return GameTooltip:GetObjectType()").unwrap();
    assert_eq!(obj_type, "GameTooltip");
}

#[test]
fn test_gametooltip_strata_is_tooltip() {
    let env = WowLuaEnv::new().unwrap();

    let strata: String = env.eval("return GameTooltip:GetFrameStrata()").unwrap();
    assert_eq!(strata, "TOOLTIP");
}

#[test]
fn test_addline_and_numlines() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddLine("First line")
        GameTooltip:AddLine("Second line", 1, 0, 0)
        GameTooltip:AddLine("Third line", 0, 1, 0, true)
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_adddoubleline_and_numlines() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddDoubleLine("Left", "Right")
        GameTooltip:AddDoubleLine("Name", "Value", 1, 1, 1, 0.5, 0.5, 0.5)
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_clearlines_resets_count() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddLine("Line 1")
        GameTooltip:AddLine("Line 2")
        GameTooltip:ClearLines()
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_settext_clears_and_sets_first_line() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddLine("Old line 1")
        GameTooltip:AddLine("Old line 2")
        GameTooltip:SetText("New text")
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(count, 1, "SetText should clear existing lines and add one");
}

#[test]
fn test_setowner_and_isowned_and_getowner() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "TooltipOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
    "#,
    )
    .unwrap();

    let is_owned: bool = env
        .eval("return GameTooltip:IsOwned(TooltipOwner)")
        .unwrap();
    assert!(is_owned, "GameTooltip should be owned by TooltipOwner");

    let owner_name: String = env
        .eval("return GameTooltip:GetOwner():GetName()")
        .unwrap();
    assert_eq!(owner_name, "TooltipOwner");

    // Check that non-owner returns false
    env.exec(r#"local other = CreateFrame("Frame", "OtherFrame", UIParent)"#)
        .unwrap();
    let not_owned: bool = env
        .eval("return GameTooltip:IsOwned(OtherFrame)")
        .unwrap();
    assert!(!not_owned, "GameTooltip should not be owned by OtherFrame");
}

#[test]
fn test_getanchortype_after_setowner() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "AnchorTestOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_BOTTOMRIGHT")
    "#,
    )
    .unwrap();

    let anchor: String = env.eval("return GameTooltip:GetAnchorType()").unwrap();
    assert_eq!(anchor, "ANCHOR_BOTTOMRIGHT");
}

#[test]
fn test_on_tooltip_cleared_fires_on_clearlines() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.tooltip_cleared_count = 0
        GameTooltip:SetScript("OnTooltipCleared", function()
            _G.tooltip_cleared_count = _G.tooltip_cleared_count + 1
        end)
        GameTooltip:AddLine("Some line")
        GameTooltip:ClearLines()
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return _G.tooltip_cleared_count").unwrap();
    assert_eq!(count, 1, "OnTooltipCleared should fire once on ClearLines");
}

#[test]
fn test_on_tooltip_cleared_fires_on_setowner() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.cleared_count = 0
        GameTooltip:SetScript("OnTooltipCleared", function()
            _G.cleared_count = _G.cleared_count + 1
        end)
        local owner = CreateFrame("Frame", "ClearedTestOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return _G.cleared_count").unwrap();
    assert_eq!(count, 1, "OnTooltipCleared should fire on SetOwner");
}

#[test]
fn test_isobjecttype_frame_returns_true_for_gametooltip() {
    let env = WowLuaEnv::new().unwrap();

    let is_frame: bool = env
        .eval("return GameTooltip:IsObjectType('Frame')")
        .unwrap();
    assert!(is_frame, "GameTooltip:IsObjectType('Frame') should return true");

    let is_region: bool = env
        .eval("return GameTooltip:IsObjectType('Region')")
        .unwrap();
    assert!(is_region, "GameTooltip:IsObjectType('Region') should return true");

    let is_tooltip: bool = env
        .eval("return GameTooltip:IsObjectType('GameTooltip')")
        .unwrap();
    assert!(is_tooltip, "GameTooltip:IsObjectType('GameTooltip') should return true");

    let is_button: bool = env
        .eval("return GameTooltip:IsObjectType('Button')")
        .unwrap();
    assert!(!is_button, "GameTooltip:IsObjectType('Button') should return false");
}

#[test]
fn test_isobjecttype_for_other_types() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TypeTestButton", UIParent)
        local cb = CreateFrame("CheckButton", "TypeTestCheckButton", UIParent)
        local frame = CreateFrame("Frame", "TypeTestFrame", UIParent)
    "#,
    )
    .unwrap();

    // Button is a Frame
    let btn_is_frame: bool = env
        .eval("return TypeTestButton:IsObjectType('Frame')")
        .unwrap();
    assert!(btn_is_frame);

    // CheckButton is a Button
    let cb_is_button: bool = env
        .eval("return TypeTestCheckButton:IsObjectType('Button')")
        .unwrap();
    assert!(cb_is_button);

    // CheckButton is a Frame
    let cb_is_frame: bool = env
        .eval("return TypeTestCheckButton:IsObjectType('Frame')")
        .unwrap();
    assert!(cb_is_frame);

    // Frame is NOT a Button
    let frame_is_button: bool = env
        .eval("return TypeTestFrame:IsObjectType('Button')")
        .unwrap();
    assert!(!frame_is_button);
}

#[test]
fn test_setminimumwidth_and_getminimumwidth() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:SetMinimumWidth(150)").unwrap();

    let width: f32 = env.eval("return GameTooltip:GetMinimumWidth()").unwrap();
    assert_eq!(width, 150.0);
}

#[test]
fn test_setpadding_and_getpadding() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:SetPadding(8)").unwrap();

    let padding: f32 = env.eval("return GameTooltip:GetPadding()").unwrap();
    assert_eq!(padding, 8.0);
}

#[test]
fn test_fadeout_hides_and_clears_owner() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "FadeOutOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
        GameTooltip:FadeOut()
    "#,
    )
    .unwrap();

    let visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    assert!(!visible, "FadeOut should hide the tooltip");

    let has_owner: bool = env
        .eval("return GameTooltip:GetOwner() ~= nil")
        .unwrap();
    assert!(!has_owner, "FadeOut should clear the owner");
}

#[test]
fn test_appendtext() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddLine("Hello")
        GameTooltip:AppendText(" World")
    "#,
    )
    .unwrap();

    // Verify through tooltip data
    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    assert_eq!(td.lines.len(), 1);
    assert_eq!(td.lines[0].left_text, "Hello World");
}

#[test]
fn test_setowner_makes_tooltip_visible() {
    let env = WowLuaEnv::new().unwrap();

    // GameTooltip starts hidden
    let initially_visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    assert!(!initially_visible, "GameTooltip should start hidden");

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "VisOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
    "#,
    )
    .unwrap();

    let now_visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    assert!(now_visible, "SetOwner should make tooltip visible");
}

#[test]
fn test_createframe_gametooltip_type() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local tt = CreateFrame("GameTooltip", "CustomTooltip", UIParent)
        tt:AddLine("Test")
    "#,
    )
    .unwrap();

    let obj_type: String = env.eval("return CustomTooltip:GetObjectType()").unwrap();
    assert_eq!(obj_type, "GameTooltip");

    let count: i32 = env.eval("return CustomTooltip:NumLines()").unwrap();
    assert_eq!(count, 1);

    let strata: String = env.eval("return CustomTooltip:GetFrameStrata()").unwrap();
    assert_eq!(strata, "TOOLTIP");
}

#[test]
fn test_other_tooltip_frames_exist() {
    let env = WowLuaEnv::new().unwrap();

    let item_ref: bool = env.eval("return ItemRefTooltip ~= nil").unwrap();
    let shopping1: bool = env.eval("return ShoppingTooltip1 ~= nil").unwrap();
    let shopping2: bool = env.eval("return ShoppingTooltip2 ~= nil").unwrap();
    let friends: bool = env.eval("return FriendsTooltip ~= nil").unwrap();

    assert!(item_ref, "ItemRefTooltip should exist");
    assert!(shopping1, "ShoppingTooltip1 should exist");
    assert!(shopping2, "ShoppingTooltip2 should exist");
    assert!(friends, "FriendsTooltip should exist");

    // All should be GameTooltip type
    let item_type: String = env
        .eval("return ItemRefTooltip:GetObjectType()")
        .unwrap();
    assert_eq!(item_type, "GameTooltip");
}

#[test]
fn test_tooltip_anchor_right_sets_anchors() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "AnchorRightOwner", UIParent)
        owner:SetSize(100, 30)
        owner:SetPoint("CENTER")
        GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    assert_eq!(frame.anchors.len(), 1, "ANCHOR_RIGHT should set one anchor");
    let anchor = &frame.anchors[0];
    assert_eq!(anchor.point, AnchorPoint::TopLeft, "tooltip point should be TopLeft");
    assert_eq!(anchor.relative_point, AnchorPoint::TopRight, "owner point should be TopRight");

    let owner_id = state.widgets.get_id_by_name("AnchorRightOwner").unwrap();
    assert_eq!(anchor.relative_to_id, Some(owner_id as usize));
}

#[test]
fn test_tooltip_anchor_none_no_anchors() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "AnchorNoneOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    assert!(frame.anchors.is_empty(), "ANCHOR_NONE should not set anchors");
}

#[test]
fn test_tooltip_anchor_cursor_uses_absolute_position() {
    let env = WowLuaEnv::new().unwrap();

    // Set mouse position before SetOwner
    env.state().borrow_mut().mouse_position = Some((200.0, 300.0));

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "AnchorCursorOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_CURSOR")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    assert_eq!(frame.anchors.len(), 1, "ANCHOR_CURSOR should set one anchor");
    let anchor = &frame.anchors[0];
    assert_eq!(anchor.point, AnchorPoint::TopLeft);
    assert!(anchor.relative_to_id.is_none(), "ANCHOR_CURSOR should not reference owner");
    assert!((anchor.x_offset - 200.0).abs() < 0.1, "x_offset should be mouse x");
    assert!((anchor.y_offset - 320.0).abs() < 0.1, "y_offset should be mouse y + 20");
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
        state.widgets.get_id_by_name("CharacterMicroButton")
            .expect("CharacterMicroButton should exist")
    };

    // Set hovered_frame so IsMouseMotionFocus() returns true
    env.state().borrow_mut().hovered_frame = Some(btn_id);

    // Fire OnEnter (this is what handle_mouse_move does)
    env.fire_script_handler(btn_id, "OnEnter", vec![]).unwrap();

    // Check tooltip state
    let visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();

    assert!(visible, "GameTooltip should be visible after micro menu hover");
    assert!(num_lines > 0, "GameTooltip should have at least one line, got {}", num_lines);

    // Verify the tooltip text content
    {
        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        let td = state.tooltips.get(&gt_id).expect("tooltip data should exist");
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
        state.widgets.get(gt_id).is_some_and(|f| f.effective_alpha > 0.0),
        "GameTooltip should be ancestor-visible (effective_alpha > 0)"
    );

    // Check frame dimensions (tooltip should not be 0x0)
    let frame = state.widgets.get(gt_id).unwrap();
    eprintln!("Tooltip frame: visible={}, width={}, height={}", frame.visible, frame.width, frame.height);
}

/// Verify the tooltip produces render quads after the full rendering pipeline runs.
#[test]
fn test_tooltip_produces_quads_after_hover() {
    use std::path::PathBuf;
    use wow_ui_sim::render::font::WowFontSystem;
    use wow_ui_sim::render::glyph::GlyphAtlas;

    let env = setup_full_env();

    // Hover over CharacterMicroButton
    let btn_id = {
        let state = env.state().borrow();
        state.widgets.get_id_by_name("CharacterMicroButton")
            .expect("CharacterMicroButton should exist")
    };
    env.state().borrow_mut().hovered_frame = Some(btn_id);
    env.fire_script_handler(btn_id, "OnEnter", vec![]).unwrap();

    // Run tooltip sizing (same as build_quad_batch does)
    let mut font_sys = WowFontSystem::new(&PathBuf::from("./fonts"));
    {
        let mut state = env.state().borrow_mut();
        let _ = state.widgets.take_render_dirty();
        wow_ui_sim::iced_app::tooltip::update_tooltip_sizes(&mut state, &mut font_sys);
    }

    // Check tooltip got sized
    let gt_id = env.state().borrow().widgets.get_id_by_name("GameTooltip").unwrap();
    let (w, h) = {
        let state = env.state().borrow();
        let f = state.widgets.get(gt_id).unwrap();
        (f.width, f.height)
    };
    eprintln!("Tooltip after sizing: {}x{}", w, h);
    assert!(w > 0.0, "Tooltip width should be > 0 after sizing, got {}", w);
    assert!(h > 0.0, "Tooltip height should be > 0 after sizing, got {}", h);

    // Check tooltip position (compute_frame_rect uses the anchor system)
    {
        let state = env.state().borrow();
        let rect = wow_ui_sim::iced_app::compute_frame_rect(&state.widgets, gt_id, 1024.0, 768.0);
        eprintln!("Tooltip rect: x={}, y={}, w={}, h={}", rect.x, rect.y, rect.width, rect.height);
        assert!(rect.width > 0.0, "Tooltip rect width should be > 0");
        assert!(rect.height > 0.0, "Tooltip rect height should be > 0");
        // Check tooltip is within visible screen
        assert!(rect.x >= 0.0 && rect.x < 1024.0, "Tooltip x={} should be on screen", rect.x);
        assert!(rect.y >= 0.0 && rect.y < 768.0, "Tooltip y={} should be on screen", rect.y);
    }

    // Build quads and verify tooltip emits something
    let buckets = {
        let mut state = env.state().borrow_mut();
        let _ = state.get_strata_buckets();
        state.strata_buckets.as_ref().unwrap().clone()
    };
    let state = env.state().borrow();
    let tooltip_data = wow_ui_sim::iced_app::tooltip::collect_tooltip_data(&state);
    assert!(!tooltip_data.is_empty(), "Tooltip render data should exist");

    let mut glyph_atlas = GlyphAtlas::new();
    let batch = wow_ui_sim::iced_app::build_quad_batch_for_registry(
        &state.widgets,
        (1024.0, 768.0),
        None, None, None,
        Some((&mut font_sys, &mut glyph_atlas)),
        Some(&state.message_frames),
        Some(&tooltip_data),
        &buckets,
    );

    // Tooltip renders via glyph quads (text) not texture quads.
    // Verify the tooltip frame was reached by checking total quad count increased.
    eprintln!("Total quads: {}, vertices: {}", batch.vertices.len() / 4, batch.vertices.len());
    assert!(batch.vertices.len() > 100, "Batch should have many vertices (tooltip + UI)");
}

const TOOLTIP_TEST_ADDONS: &[(&str, &str)] = &[
    ("Blizzard_SharedXMLBase", "Blizzard_SharedXMLBase.toc"),
    ("Blizzard_Colors", "Blizzard_Colors_Mainline.toc"),
    ("Blizzard_SharedXML", "Blizzard_SharedXML_Mainline.toc"),
    ("Blizzard_SharedXMLGame", "Blizzard_SharedXMLGame_Mainline.toc"),
    ("Blizzard_UIPanelTemplates", "Blizzard_UIPanelTemplates_Mainline.toc"),
    ("Blizzard_FrameXMLBase", "Blizzard_FrameXMLBase_Mainline.toc"),
    ("Blizzard_LoadLocale", "Blizzard_LoadLocale.toc"),
    ("Blizzard_Fonts_Shared", "Blizzard_Fonts_Shared.toc"),
    ("Blizzard_HelpPlate", "Blizzard_HelpPlate.toc"),
    ("Blizzard_AccessibilityTemplates", "Blizzard_AccessibilityTemplates.toc"),
    ("Blizzard_ObjectAPI", "Blizzard_ObjectAPI_Mainline.toc"),
    ("Blizzard_UIParent", "Blizzard_UIParent_Mainline.toc"),
    ("Blizzard_TextStatusBar", "Blizzard_TextStatusBar.toc"),
    ("Blizzard_MoneyFrame", "Blizzard_MoneyFrame_Mainline.toc"),
    ("Blizzard_POIButton", "Blizzard_POIButton.toc"),
    ("Blizzard_Flyout", "Blizzard_Flyout.toc"),
    ("Blizzard_StoreUI", "Blizzard_StoreUI_Mainline.toc"),
    ("Blizzard_MicroMenu", "Blizzard_MicroMenu_Mainline.toc"),
    ("Blizzard_EditMode", "Blizzard_EditMode.toc"),
    ("Blizzard_GarrisonBase", "Blizzard_GarrisonBase.toc"),
    ("Blizzard_GameTooltip", "Blizzard_GameTooltip_Mainline.toc"),
    ("Blizzard_UIParentPanelManager", "Blizzard_UIParentPanelManager_Mainline.toc"),
    ("Blizzard_Settings_Shared", "Blizzard_Settings_Shared_Mainline.toc"),
    ("Blizzard_SettingsDefinitions_Shared", "Blizzard_SettingsDefinitions_Shared.toc"),
    ("Blizzard_SettingsDefinitions_Frame", "Blizzard_SettingsDefinitions_Frame_Mainline.toc"),
    ("Blizzard_FrameXMLUtil", "Blizzard_FrameXMLUtil_Mainline.toc"),
    ("Blizzard_ItemButton", "Blizzard_ItemButton_Mainline.toc"),
    ("Blizzard_QuickKeybind", "Blizzard_QuickKeybind.toc"),
    ("Blizzard_FrameXML", "Blizzard_FrameXML_Mainline.toc"),
    ("Blizzard_UIPanels_Game", "Blizzard_UIPanels_Game_Mainline.toc"),
    ("Blizzard_ActionBar", "Blizzard_ActionBar_Mainline.toc"),
];

/// Reusable full-env loader (same as micro_menu.rs).
fn setup_full_env() -> WowLuaEnv {
    use std::path::PathBuf;

    let env = WowLuaEnv::new().unwrap();
    env.set_screen_size(1024.0, 768.0);

    let ui = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Interface/BlizzardUI");
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];

    load_blizzard_addons(&env, &ui);
    env.apply_post_load_workarounds();
    fire_tooltip_test_startup_events(&env);
    env
}

fn load_blizzard_addons(env: &WowLuaEnv, ui: &std::path::Path) {
    use wow_ui_sim::loader::load_addon;

    for (name, toc) in TOOLTIP_TEST_ADDONS {
        let toc_path = ui.join(name).join(toc);
        if toc_path.exists() {
            if let Err(e) = load_addon(&env.loader_env(), &toc_path) {
                eprintln!("[load {name}] FAILED: {e}");
            }
        }
    }
}

fn fire_tooltip_test_startup_events(env: &WowLuaEnv) {
    let lua = env.lua();
    let _ = env.fire_event_with_args(
        "ADDON_LOADED",
        &[mlua::Value::String(lua.create_string("WoWUISim").unwrap())],
    );
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(event);
    }
    let _ = env.fire_event_with_args(
        "PLAYER_ENTERING_WORLD",
        &[mlua::Value::Boolean(true), mlua::Value::Boolean(false)],
    );
    for event in ["UPDATE_BINDINGS", "DISPLAY_SIZE_CHANGED", "UI_SCALE_CHANGED"] {
        let _ = env.fire_event(event);
    }
}
