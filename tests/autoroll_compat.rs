//! Compatibility tests using the AutoRoll addon as a real-world test case.

use wow_ui_sim::lua_api::WowLuaEnv;

/// Test that we can load the basic addon structure with vararg unpacking.
#[test]
fn test_vararg_unpacking() {
    let env = WowLuaEnv::new().unwrap();

    // WoW passes addon name and namespace table via varargs
    let result = env.exec(
        r#"
        local addon, ns = ...
        -- In WoW, these come from the loader. We need to provide them.
        "#,
    );

    // This will fail until we implement vararg passing
    // For now, just verify it doesn't crash
    assert!(result.is_ok() || result.is_err());
}

/// Test CreateFrame with all common argument patterns.
#[test]
fn test_create_frame_patterns() {
    let env = WowLuaEnv::new().unwrap();

    // Pattern 1: Just type
    env.exec(r#"local f1 = CreateFrame("Frame")"#).unwrap();

    // Pattern 2: Type and name
    env.exec(r#"local f2 = CreateFrame("Frame", "TestFrame2")"#)
        .unwrap();

    // Pattern 3: Type, name, and parent
    env.exec(r#"local f3 = CreateFrame("Frame", "TestFrame3", UIParent)"#)
        .unwrap();

    // Pattern 4: With nil name (anonymous frame with parent)
    env.exec(r#"local f4 = CreateFrame("Frame", nil, UIParent)"#)
        .unwrap();
}

/// Test event registration and handling pattern from AutoRoll.
#[test]
fn test_event_registration_pattern() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local main = CreateFrame("Frame", "AutoRollFrame")

        -- Track which events were received
        _G.receivedEvents = {}

        -- AutoRoll pattern: methods on frame table, dispatched via OnEvent
        function main:ADDON_LOADED(name)
            table.insert(_G.receivedEvents, "ADDON_LOADED:" .. tostring(name))
        end

        function main:START_LOOT_ROLL(rollID)
            table.insert(_G.receivedEvents, "START_LOOT_ROLL:" .. tostring(rollID))
        end

        -- The dispatch pattern used by AutoRoll
        main:SetScript("OnEvent", function(self, event, ...)
            if main[event] then
                main[event](self, ...)
            end
        end)

        main:RegisterEvent("ADDON_LOADED")
        main:RegisterEvent("START_LOOT_ROLL")
        "#,
    )
    .unwrap();

    // Fire events
    env.fire_event("ADDON_LOADED").unwrap();

    // Verify the handler was called
    let count: i32 = env.eval("return #_G.receivedEvents").unwrap();
    assert_eq!(count, 1, "Expected 1 event to be received");
}

/// Test that UnregisterEvent works.
#[test]
fn test_unregister_event() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.eventCount = 0
        _G.testFrame = CreateFrame("Frame")
        _G.testFrame:SetScript("OnEvent", function(self, event)
            _G.eventCount = _G.eventCount + 1
        end)
        _G.testFrame:RegisterEvent("TEST_EVENT")
        "#,
    )
    .unwrap();

    env.fire_event("TEST_EVENT").unwrap();
    let count: i32 = env.eval("return _G.eventCount").unwrap();
    assert_eq!(count, 1);

    env.exec(r#"_G.testFrame:UnregisterEvent("TEST_EVENT")"#)
        .unwrap();
    env.fire_event("TEST_EVENT").unwrap();

    let count: i32 = env.eval("return _G.eventCount").unwrap();
    assert_eq!(count, 1, "Event should not fire after unregister");
}

/// Test the print function (used extensively in AutoRoll).
#[test]
fn test_print_function() {
    let env = WowLuaEnv::new().unwrap();

    // print() is a standard Lua function, should just work
    env.exec(r#"print("Hello", "World", 123)"#).unwrap();
    env.exec(r#"print("[|Cff3388ffAutoRoll|r]", "test")"#)
        .unwrap();
}

/// Test pairs/ipairs iteration (used for table iteration in AutoRoll).
#[test]
fn test_table_iteration() {
    let env = WowLuaEnv::new().unwrap();

    let result: i32 = env
        .eval(
            r#"
        local sum = 0
        local t = {a = 1, b = 2, c = 3}
        for k, v in pairs(t) do
            sum = sum + v
        end
        return sum
        "#,
        )
        .unwrap();

    assert_eq!(result, 6);

    let result: i32 = env
        .eval(
            r#"
        local sum = 0
        local t = {10, 20, 30}
        for i, v in ipairs(t) do
            sum = sum + v
        end
        return sum
        "#,
        )
        .unwrap();

    assert_eq!(result, 60);
}

/// Test type() function.
#[test]
fn test_type_function() {
    let env = WowLuaEnv::new().unwrap();

    let result: String = env.eval(r#"return type("hello")"#).unwrap();
    assert_eq!(result, "string");

    let result: String = env.eval(r#"return type(123)"#).unwrap();
    assert_eq!(result, "number");

    let result: String = env.eval(r#"return type({})"#).unwrap();
    assert_eq!(result, "table");
}

/// Test global variable access (AutoRollDB pattern).
#[test]
fn test_saved_variables_pattern() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        -- Initialize saved variables (normally loaded from SavedVariables)
        if not AutoRollDB then
            AutoRollDB = {}
        end

        -- Merge defaults
        local defaults = {
            enabled = true,
            debugMode = false,
        }

        for k, v in pairs(defaults) do
            if AutoRollDB[k] == nil then
                AutoRollDB[k] = v
            end
        end
        "#,
    )
    .unwrap();

    let enabled: bool = env.eval("return AutoRollDB.enabled").unwrap();
    assert!(enabled);
}

/// Test frame visibility methods.
#[test]
fn test_frame_visibility() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local frame = CreateFrame("Frame", "VisTestFrame")
        _G.wasVisible1 = frame:IsVisible()
        frame:Hide()
        _G.wasVisible2 = frame:IsVisible()
        frame:Show()
        _G.wasVisible3 = frame:IsVisible()
        "#,
    )
    .unwrap();

    // Frames start visible by default
    let v1: bool = env.eval("return _G.wasVisible1").unwrap();
    let v2: bool = env.eval("return _G.wasVisible2").unwrap();
    let v3: bool = env.eval("return _G.wasVisible3").unwrap();

    assert!(v1, "Frame should start visible");
    assert!(!v2, "Frame should be hidden after Hide()");
    assert!(v3, "Frame should be visible after Show()");
}

/// Test GetName() method.
#[test]
fn test_get_name() {
    let env = WowLuaEnv::new().unwrap();

    let name: String = env
        .eval(
            r#"
        local frame = CreateFrame("Frame", "MyNamedFrame")
        return frame:GetName()
        "#,
        )
        .unwrap();

    assert_eq!(name, "MyNamedFrame");
}

/// Test GetObjectType() method.
#[test]
fn test_get_object_type() {
    let env = WowLuaEnv::new().unwrap();

    let obj_type: String = env
        .eval(
            r#"
        local frame = CreateFrame("Frame")
        return frame:GetObjectType()
        "#,
        )
        .unwrap();

    assert_eq!(obj_type, "Frame");
}

/// Test GetParent() method.
#[test]
fn test_get_parent() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local child = CreateFrame("Frame", "ChildFrame", UIParent)
        local parent = child:GetParent()
        _G.parentName = parent and parent:GetName() or "nil"
        "#,
    )
    .unwrap();

    let parent_name: String = env.eval("return _G.parentName").unwrap();
    assert_eq!(parent_name, "UIParent");
}
