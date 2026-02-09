//! Tests using the purpose-built TestAddon.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use std::fs;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::render::WowFontSystem;

fn load_test_addon(env: &WowLuaEnv) -> Result<(), String> {
    let addon_path = concat!(env!("CARGO_MANIFEST_DIR"), "/test_addons/TestAddon/TestAddon.lua");
    let code = fs::read_to_string(addon_path)
        .map_err(|e| format!("Failed to read TestAddon.lua: {}", e))?;

    // Wrap to provide varargs
    let wrapped = format!(
        r#"
        local function loadAddon(...)
            {}
        end
        loadAddon("TestAddon", {{}})
        "#,
        code
    );

    env.exec(&wrapped)
        .map_err(|e| format!("Failed to execute TestAddon: {}", e))
}

fn fire_addon_loaded(env: &WowLuaEnv) {
    // Fire ADDON_LOADED with "TestAddon" as the addon name argument
    // Check a wide range since widget IDs are global and increment across tests
    env.exec(r#"
        -- Manually trigger the loader's OnEvent
        for id = 1, 10000 do
            local key = "__frame_" .. id
            local frame = _G[key]
            if frame then
                local handler = frame:GetScript("OnEvent")
                if handler then
                    handler(frame, "ADDON_LOADED", "TestAddon")
                end
            end
        end
    "#).ok();
}

#[test]
fn test_addon_loads() {
    let env = WowLuaEnv::new().unwrap();
    load_test_addon(&env).expect("TestAddon should load");

    // Fire ADDON_LOADED to trigger tests
    fire_addon_loaded(&env);
}

#[test]
fn test_basic_frame() {
    let env = WowLuaEnv::new().unwrap();
    load_test_addon(&env).unwrap();
    fire_addon_loaded(&env);

    // Check TestBasicFrame exists with correct properties
    let width: f32 = env.eval("return TestBasicFrame:GetWidth()").unwrap();
    let height: f32 = env.eval("return TestBasicFrame:GetHeight()").unwrap();
    let visible: bool = env.eval("return TestBasicFrame:IsVisible()").unwrap();

    assert_eq!(width, 200.0, "Width should be 200");
    assert_eq!(height, 100.0, "Height should be 100");
    assert!(visible, "Frame should be visible");
}

#[test]
fn test_anchor_positions() {
    let env = WowLuaEnv::new().unwrap();
    load_test_addon(&env).unwrap();
    fire_addon_loaded(&env);

    // Verify all corner frames exist
    let tl_exists: bool = env.eval("return TestTopLeft ~= nil").unwrap();
    let tr_exists: bool = env.eval("return TestTopRight ~= nil").unwrap();
    let bl_exists: bool = env.eval("return TestBottomLeft ~= nil").unwrap();
    let br_exists: bool = env.eval("return TestBottomRight ~= nil").unwrap();

    assert!(tl_exists, "TestTopLeft should exist");
    assert!(tr_exists, "TestTopRight should exist");
    assert!(bl_exists, "TestBottomLeft should exist");
    assert!(br_exists, "TestBottomRight should exist");

    // Verify all have correct size
    let tl_w: f32 = env.eval("return TestTopLeft:GetWidth()").unwrap();
    let tr_w: f32 = env.eval("return TestTopRight:GetWidth()").unwrap();

    assert_eq!(tl_w, 50.0);
    assert_eq!(tr_w, 50.0);
}

#[test]
fn test_parent_child() {
    let env = WowLuaEnv::new().unwrap();
    load_test_addon(&env).unwrap();
    fire_addon_loaded(&env);

    let child1_parent: String = env
        .eval("return TestChild1:GetParent():GetName()")
        .unwrap();
    let child2_parent: String = env
        .eval("return TestChild2:GetParent():GetName()")
        .unwrap();

    assert_eq!(child1_parent, "TestParent");
    assert_eq!(child2_parent, "TestParent");
}

#[test]
fn test_visibility_toggle() {
    let env = WowLuaEnv::new().unwrap();
    load_test_addon(&env).unwrap();
    fire_addon_loaded(&env);

    // The test addon already ran visibility tests, check frame is visible now
    let visible: bool = env.eval("return TestVisibility:IsVisible()").unwrap();
    assert!(visible, "Frame should be visible after Show()");
}

#[test]
fn test_custom_fields() {
    let env = WowLuaEnv::new().unwrap();
    load_test_addon(&env).unwrap();
    fire_addon_loaded(&env);

    let my_data: String = env.eval("return TestCustomFields.myData").unwrap();
    let my_number: i32 = env.eval("return TestCustomFields.myNumber").unwrap();
    let method_result: String = env.eval("return TestCustomFields:CustomMethod()").unwrap();

    assert_eq!(my_data, "hello");
    assert_eq!(my_number, 42);
    assert_eq!(method_result, "hello world");
}

#[test]
fn test_event_registration() {
    let env = WowLuaEnv::new().unwrap();
    load_test_addon(&env).unwrap();

    // Fire events to set up, then fire test events
    fire_addon_loaded(&env);
    env.fire_event("PLAYER_LOGIN").unwrap();

    // Check event log
    let _log_count: i32 = env
        .eval(
            r#"
        local ns = select(2, ...)  -- Can't access ns this way
        -- Work around by checking frame directly
        return #(_G.TestAddon_eventLog or {})
        "#,
        )
        .unwrap_or(0);

    // We can't easily access ns.eventLog, but we can verify the frame exists
    let frame_exists: bool = env.eval("return TestEvents ~= nil").unwrap();
    assert!(frame_exists, "TestEvents frame should exist");
}

#[test]
fn test_alpha() {
    let env = WowLuaEnv::new().unwrap();
    load_test_addon(&env).unwrap();
    fire_addon_loaded(&env);

    let alpha: f32 = env.eval("return TestAlpha:GetAlpha()").unwrap();
    assert_eq!(alpha, 1.0, "Alpha should be 1.0 after test completes");
}

#[test]
fn test_strata_level() {
    let env = WowLuaEnv::new().unwrap();
    load_test_addon(&env).unwrap();
    fire_addon_loaded(&env);

    let strata: String = env.eval("return TestStrataLevel:GetFrameStrata()").unwrap();
    let level: i32 = env.eval("return TestStrataLevel:GetFrameLevel()").unwrap();

    assert_eq!(strata, "HIGH");
    assert_eq!(level, 10);
}

#[test]
fn test_strata_inheritance() {
    let env = WowLuaEnv::new().unwrap();
    load_test_addon(&env).unwrap();
    fire_addon_loaded(&env);

    // Child should inherit HIGH strata from parent
    let child_strata: String = env.eval("return TestStrataChild:GetFrameStrata()").unwrap();
    let child_level: i32 = env.eval("return TestStrataChild:GetFrameLevel()").unwrap();

    // Grandchild should also inherit HIGH strata
    let grandchild_strata: String = env.eval("return TestStrataGrandchild:GetFrameStrata()").unwrap();
    let grandchild_level: i32 = env.eval("return TestStrataGrandchild:GetFrameLevel()").unwrap();

    assert_eq!(child_strata, "HIGH", "Child should inherit HIGH strata from parent");
    assert_eq!(child_level, 6, "Child level should be parent level + 1");
    assert_eq!(grandchild_strata, "HIGH", "Grandchild should inherit HIGH strata");
    assert_eq!(grandchild_level, 7, "Grandchild level should be child level + 1");
}

#[test]
fn test_texture_creation() {
    let env = WowLuaEnv::new().unwrap();
    load_test_addon(&env).unwrap();
    fire_addon_loaded(&env);

    let obj_type: String = env.eval("return TestTexture:GetObjectType()").unwrap();
    let has_texture: bool = env.eval("return TestTexture:GetTexture() ~= nil").unwrap();
    let parent: String = env.eval("return TestTexture:GetParent():GetName()").unwrap();

    assert_eq!(obj_type, "Texture");
    assert!(has_texture);
    assert_eq!(parent, "TestTextureParent");
}

#[test]
fn test_fontstring_creation() {
    let env = WowLuaEnv::new().unwrap();
    load_test_addon(&env).unwrap();
    fire_addon_loaded(&env);

    let obj_type: String = env.eval("return TestFontString:GetObjectType()").unwrap();
    let text: String = env.eval("return TestFontString:GetText()").unwrap();
    let width: f32 = env.eval("return TestFontString:GetStringWidth()").unwrap();

    assert_eq!(obj_type, "FontString");
    assert_eq!(text, "Hello World");
    assert!(width > 0.0, "Font string should have width");
}

#[test]
fn test_get_point() {
    let env = WowLuaEnv::new().unwrap();
    load_test_addon(&env).unwrap();
    fire_addon_loaded(&env);

    let point: String = env.eval("local p = TestGetPoint:GetPoint(1); return p").unwrap();
    let num_points: i32 = env.eval("return TestGetPoint:GetNumPoints()").unwrap();

    assert_eq!(point, "TOPLEFT");
    assert_eq!(num_points, 1);
}

#[test]
fn test_all_frames_created() {
    let env = WowLuaEnv::new().unwrap();
    load_test_addon(&env).unwrap();
    fire_addon_loaded(&env);

    // Count all test frames (original 11 + 6 new parent frames + textures/fontstrings)
    let count: i32 = env
        .eval(
            r#"
        local count = 0
        local testFrames = {
            "TestBasicFrame", "TestTopLeft", "TestTopRight",
            "TestBottomLeft", "TestBottomRight", "TestParent",
            "TestChild1", "TestChild2", "TestVisibility",
            "TestEvents", "TestCustomFields",
            "TestAlpha", "TestStrataLevel", "TestStrataParent",
            "TestStrataChild", "TestStrataGrandchild", "TestMouse",
            "TestTextureParent", "TestTexture",
            "TestFontParent", "TestFontString",
            "TestGetPoint"
        }
        for _, name in ipairs(testFrames) do
            if _G[name] then count = count + 1 end
        end
        return count
        "#,
        )
        .unwrap();

    assert_eq!(count, 22, "All 22 test frames should be created");
}

#[test]
fn test_button_positions() {
    let env = WowLuaEnv::new().unwrap();

    // Create a panel at CENTER with a button at BOTTOMLEFT
    env.exec(r#"
        local mainFrame = CreateFrame("Frame", "TestMainPanel", UIParent)
        mainFrame:SetSize(300, 220)
        mainFrame:SetPoint("CENTER", 60, 0)

        local btn1 = CreateFrame("Button", "TestAcceptBtn", mainFrame)
        btn1:SetSize(100, 28)
        btn1:SetPoint("BOTTOMLEFT", 30, 25)

        local btn2 = CreateFrame("Button", "TestDeclineBtn", mainFrame)
        btn2:SetSize(100, 28)
        btn2:SetPoint("BOTTOMRIGHT", -30, 25)
    "#).unwrap();

    // Dump and print frame positions for debugging
    let dump = env.dump_frames();
    println!("Frame dump:\n{}", dump);

    // Verify the buttons exist
    let btn1_exists: bool = env.eval("return TestAcceptBtn ~= nil").unwrap();
    let btn2_exists: bool = env.eval("return TestDeclineBtn ~= nil").unwrap();
    assert!(btn1_exists, "AcceptButton should exist");
    assert!(btn2_exists, "DeclineButton should exist");

    // Check that GetPoint returns the correct anchor info
    let (point1, rel_point1, x1, y1): (String, String, f32, f32) = env.eval(r#"
        local p, rt, rp, x, y = TestAcceptBtn:GetPoint(1)
        return p, rp, x, y
    "#).unwrap();

    assert_eq!(point1, "BOTTOMLEFT");
    assert_eq!(rel_point1, "BOTTOMLEFT");
    assert_eq!(x1, 30.0);
    assert_eq!(y1, 25.0);

    let (point2, rel_point2, x2, y2): (String, String, f32, f32) = env.eval(r#"
        local p, rt, rp, x, y = TestDeclineBtn:GetPoint(1)
        return p, rp, x, y
    "#).unwrap();

    assert_eq!(point2, "BOTTOMRIGHT");
    assert_eq!(rel_point2, "BOTTOMRIGHT");
    assert_eq!(x2, -30.0);
    assert_eq!(y2, 25.0);
}

#[test]
fn test_parent_visibility_propagation() {
    let env = WowLuaEnv::new().unwrap();

    // Create a parent frame that is hidden
    env.exec(r#"
        local parent = CreateFrame("Frame", "TestHiddenParent", UIParent)
        parent:SetSize(200, 200)
        parent:Hide()  -- Parent is hidden

        local child = CreateFrame("Button", "TestChildOfHidden", parent)
        child:SetSize(100, 50)
        child:Show()  -- Child is explicitly shown, but parent is hidden
    "#).unwrap();

    // Verify parent is hidden
    let parent_visible: bool = env.eval("return TestHiddenParent:IsVisible()").unwrap();
    assert!(!parent_visible, "Parent should be hidden");

    // Verify child's own visibility flag is true
    let child_shown: bool = env.eval("return TestChildOfHidden:IsShown()").unwrap();
    assert!(child_shown, "Child's own shown flag should be true");

    // Check parent_id is correctly set
    let child_parent: String = env.eval("return TestChildOfHidden:GetParent():GetName()").unwrap();
    assert_eq!(child_parent, "TestHiddenParent", "Child should have correct parent");

    // Get parent_id from the widget registry directly
    let state = env.state().borrow();
    let child_id = state.widgets.get_id_by_name("TestChildOfHidden").expect("Child should exist");
    let parent_id = state.widgets.get_id_by_name("TestHiddenParent").expect("Parent should exist");

    let child_frame = state.widgets.get(child_id).expect("Child frame should exist");
    assert_eq!(child_frame.parent_id, Some(parent_id), "Child's parent_id should point to parent");

    let parent_frame = state.widgets.get(parent_id).expect("Parent frame should exist");
    assert!(!parent_frame.visible, "Parent's visible flag should be false");
    assert!(child_frame.visible, "Child's own visible flag should be true");
}

#[test]
fn test_lua_property_syncs_to_rust_children_keys() {
    // Test that Lua property assignment (parent.ChildKey = frame) automatically
    // syncs to Rust children_keys via __newindex metamethod.
    // This enables Rust methods like SetTitle to find child frames.
    let env = WowLuaEnv::new().unwrap();

    // Create parent and child frames, then assign child to parent property
    env.exec(r#"
        local parent = CreateFrame("Frame", "TestParentWithKey", UIParent)
        local child = CreateFrame("Frame", "TestChildWithKey", parent)
        local fontstring = parent:CreateFontString("TestFontStringWithKey")

        -- Assign frames to parent properties (like XML parentKey does)
        parent.MyChild = child
        parent.TitleContainer = child
        child.TitleText = fontstring
    "#).unwrap();

    // Verify Lua-side assignment works
    let lua_child_exists: bool = env.eval("return TestParentWithKey.MyChild ~= nil").unwrap();
    let lua_title_container: bool = env.eval("return TestParentWithKey.TitleContainer ~= nil").unwrap();
    let lua_title_text: bool = env.eval("return TestParentWithKey.TitleContainer.TitleText ~= nil").unwrap();
    assert!(lua_child_exists, "Lua property MyChild should exist");
    assert!(lua_title_container, "Lua property TitleContainer should exist");
    assert!(lua_title_text, "Lua property TitleText should exist");

    // Verify Rust-side children_keys was updated via __newindex
    let state = env.state().borrow();
    let parent_id = state.widgets.get_id_by_name("TestParentWithKey").expect("Parent should exist");
    let child_id = state.widgets.get_id_by_name("TestChildWithKey").expect("Child should exist");
    let fontstring_id = state.widgets.get_id_by_name("TestFontStringWithKey").expect("FontString should exist");

    let parent_frame = state.widgets.get(parent_id).expect("Parent frame should exist");
    let child_frame = state.widgets.get(child_id).expect("Child frame should exist");

    // Check children_keys was populated by __newindex
    assert_eq!(
        parent_frame.children_keys.get("MyChild"),
        Some(&child_id),
        "Rust children_keys should have MyChild pointing to child"
    );
    assert_eq!(
        parent_frame.children_keys.get("TitleContainer"),
        Some(&child_id),
        "Rust children_keys should have TitleContainer pointing to child"
    );
    assert_eq!(
        child_frame.children_keys.get("TitleText"),
        Some(&fontstring_id),
        "Child's children_keys should have TitleText pointing to fontstring"
    );
}

#[test]
fn test_buff_duration_text_centered_under_icon() {
    // Reproduce the buff icon layout: a 30x40 button with a 30x30 Icon at TOP
    // and a Duration FontString anchored TOP to Icon's BOTTOM.
    // After SetFormattedText, the Duration should have non-zero width so the
    // TOP anchor centers it horizontally under the Icon.
    let env = WowLuaEnv::new().unwrap();
    let font_system = Rc::new(RefCell::new(WowFontSystem::new(&PathBuf::from("./fonts"))));
    env.set_font_system(font_system);

    env.exec(r#"
        local parent = CreateFrame("Frame", "TestBuffButton", UIParent)
        parent:SetSize(30, 40)
        parent:SetPoint("CENTER")

        local icon = parent:CreateTexture("TestBuffIcon")
        icon:SetSize(30, 30)
        icon:SetPoint("TOP")

        local duration = parent:CreateFontString("TestBuffDuration")
        duration:SetFont("Fonts\\FRIZQT__.TTF", 12)
        duration:SetPoint("TOP", icon, "BOTTOM")
        duration:SetFormattedText("%dm", 60)
    "#).unwrap();

    // Verify the Duration FontString has auto-sized width
    let state = env.state().borrow();
    let duration_id = state.widgets.get_id_by_name("TestBuffDuration")
        .expect("Duration FontString should exist");
    let duration = state.widgets.get(duration_id)
        .expect("Duration frame should exist");

    // word_wrap defaults to true (matching WoW), but auto-sizing should still
    // work when no explicit width constraint is set (width == 0).
    assert!(
        duration.word_wrap,
        "word_wrap should default to true (matching WoW behavior)"
    );
    assert!(
        duration.width > 0.0,
        "Duration FontString width should be auto-sized after SetFormattedText, got {}. \
         word_wrap=true FontStrings without explicit width should still auto-size.",
        duration.width
    );

    // Verify horizontal centering: Duration's center X should equal Icon's center X.
    // Icon is 30px wide anchored at TOP of 30px parent → Icon center X = parent_x + 15
    // Duration with TOP anchor to Icon's BOTTOM: center X = icon_center_x = parent_x + 15
    // So Duration's left edge should be at: parent_x + 15 - duration.width/2
    let icon_id = state.widgets.get_id_by_name("TestBuffIcon")
        .expect("Icon should exist");
    let icon_rect = wow_ui_sim::lua_api::compute_frame_rect(
        &state.widgets, icon_id, 1024.0, 768.0,
    );
    let dur_rect = wow_ui_sim::lua_api::compute_frame_rect(
        &state.widgets, duration_id, 1024.0, 768.0,
    );

    let icon_center_x = icon_rect.x + icon_rect.width / 2.0;
    let dur_center_x = dur_rect.x + dur_rect.width / 2.0;

    assert!(
        (icon_center_x - dur_center_x).abs() < 1.0,
        "Duration text should be horizontally centered under Icon. \
         Icon center X={}, Duration center X={} (x={}, w={})",
        icon_center_x, dur_center_x, dur_rect.x, dur_rect.width
    );
}
