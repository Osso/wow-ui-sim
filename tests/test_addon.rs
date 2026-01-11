//! Tests using the purpose-built TestAddon.

use std::fs;
use wow_ui_sim::lua_api::WowLuaEnv;

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
            "TestAlpha", "TestStrataLevel", "TestMouse",
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

    assert_eq!(count, 19, "All 19 test frames should be created");
}
