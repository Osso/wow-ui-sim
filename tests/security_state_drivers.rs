//! Tests for secure state drivers and secure frame helper globals.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn test_state_drivers_apply_visibility_and_attributes() {
    let env = env();
    let (shown, state_hidden, custom_state, numeric_state, nil_state): (
        bool,
        bool,
        String,
        i32,
        bool,
    ) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame")
            frame:Show()

            RegisterStateDriver(frame, "visibility", "hide")
            RegisterAttributeDriver(frame, "state-custom", "active")
            RegisterAttributeDriver(frame, "state-count", "17")
            RegisterAttributeDriver(frame, "state-empty", "nil")

            return frame:IsShown(),
                frame:GetAttribute("statehidden"),
                frame:GetAttribute("state-custom"),
                frame:GetAttribute("state-count"),
                frame:GetAttribute("state-empty") == nil
            "#,
        )
        .unwrap();
    assert!(
        !shown,
        "visibility drivers should immediately hide frames for the hide state"
    );
    assert!(
        state_hidden,
        "visibility drivers should mark hidden frames with statehidden"
    );
    assert!(
        custom_state == "active",
        "attribute drivers should resolve string values onto frame attributes"
    );
    assert_eq!(
        numeric_state, 17,
        "attribute drivers should coerce numeric strings to numbers"
    );
    assert!(
        nil_state,
        "attribute drivers should treat the literal nil token as an unset attribute"
    );
}

#[test]
fn test_state_driver_resolves_first_matching_clause_and_runs_state_snippet() {
    let env = env();
    let (shown, state_value, fade_value): (bool, String, bool) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame")
            frame:Show()
            frame:SetAttribute("_onstate-vis", [[
                if newstate == "show" then
                    self:Show()
                    self:SetAttribute("fade", false)
                elseif newstate == "hide" then
                    self:Hide()
                end
            ]])

            RegisterStateDriver(frame, "vis", "[petbattle]hide;hide;show")

            return frame:IsShown(),
                frame:GetAttribute("state-vis"),
                frame:GetAttribute("fade") == false
            "#,
        )
        .unwrap();

    assert!(
        !shown,
        "unconditional hide clauses should win over later fallback show clauses"
    );
    assert_eq!(state_value, "hide");
    assert!(
        !fade_value,
        "hide state should run the secure state snippet rather than the show branch"
    );
}

#[test]
fn test_state_attribute_set_runs_matching_state_snippet() {
    let env = env();
    let (shown, ran_state): (bool, String) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame")
            frame:Show()
            frame:SetAttribute("_onstate-vis", [[
                self:SetAttribute("ran", tostring(newstate))
                if newstate == "hide" then
                    self:Hide()
                end
            ]])

            frame:SetAttribute("state-vis", "hide")

            return frame:IsShown(),
                frame:GetAttribute("ran")
            "#,
        )
        .unwrap();

    assert!(
        !shown,
        "state attribute changes should execute matching _onstate snippets"
    );
    assert_eq!(ran_state, "hide");
}

#[test]
fn test_child_update_runs_matching_protected_child_snippet() {
    let env = env();
    let action: i32 = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "ChildUpdateParent")
            local child = CreateFrame("Button", "ChildUpdateChild", parent)
            A_Admin.SetFrameProtected("ChildUpdateChild", true)

            child:SetAttributeNoHandler(
                "_childupdate-page",
                "self:SetAttribute('action', (tonumber(message) or 1) * 12)"
            )
            parent:ChildUpdate("page", 3)

            return child:GetAttribute("action")
            "#,
        )
        .unwrap();

    assert_eq!(action, 36);
}

#[test]
fn test_state_driver_snippet_can_call_child_update() {
    let env = env();
    let action: i32 = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "StateDriverChildUpdateParent")
            local child = CreateFrame("Button", "StateDriverChildUpdateChild", parent)
            A_Admin.SetFrameProtected("StateDriverChildUpdateChild", true)

            parent:SetAttributeNoHandler("_onstate-page", [[
                local page = tonumber(newstate) or 1
                self:SetAttribute("actionpage", page)
                self:ChildUpdate("page", page)
            ]])
            child:SetAttributeNoHandler(
                "_childupdate-page",
                "self:SetAttribute('action', (tonumber(message) or 1) * 12)"
            )

            RegisterStateDriver(parent, "page", "3")

            return child:GetAttribute("action")
            "#,
        )
        .unwrap();

    assert_eq!(action, 36);
}

#[test]
fn test_unregister_state_drivers_leaves_last_resolved_values_in_place() {
    let env = env();
    let (shown, custom_state): (bool, String) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame")
            frame:Show()

            RegisterStateDriver(frame, "visibility", "hide")
            RegisterAttributeDriver(frame, "state-custom", "active")
            UnregisterStateDriver(frame, "visibility")
            UnregisterAttributeDriver(frame, "state-custom")

            return frame:IsShown(), frame:GetAttribute("state-custom")
            "#,
        )
        .unwrap();
    assert!(
        !shown,
        "unregister should stop future updates, not force a visibility reset"
    );
    assert_eq!(
        custom_state, "active",
        "unregister should preserve the last resolved attribute value"
    );
}

#[test]
fn test_securecallmethod_returns_values() {
    let env = env();
    let result: i32 = env
        .eval(
            r#"
            local obj = { Add = function(self, a, b) return a + b end }
            return securecallmethod(obj, "Add", 3, 7)
            "#,
        )
        .unwrap();
    assert_eq!(result, 10);
}

#[test]
fn test_securecallmethod_swallows_errors() {
    let env = env();
    let result: bool = env
        .eval(
            r#"
            local obj = { Bad = function() error("boom") end }
            securecallmethod(obj, "Bad")
            return true
            "#,
        )
        .unwrap();
    assert!(result, "securecallmethod should swallow errors");
}

#[test]
fn test_securecallmethod_missing_method() {
    let env = env();
    let result: bool = env
        .eval(
            r#"
            local obj = {}
            local r = securecallmethod(obj, "Nope")
            return r == nil
            "#,
        )
        .unwrap();
    assert!(result, "missing method should return nil");
}

// ============================================================================
// CreateSecureDelegate
// ============================================================================

#[test]
fn test_create_secure_delegate_runs_with_secure_taint() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local function myFunc(a, b) return a + b, tostring(issecure()) end
            local delegate = CreateSecureDelegate(myFunc)
            local function addon() return delegate(40, 2) end
            debug.setobjecttaint(addon, 'DelegateProbe')
            local sum, secure = addon()
            return tostring(delegate ~= myFunc) .. ':' .. sum .. ':' .. secure
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "true:42:true",
        "delegates pass values through and execute securely for tainted callers"
    );
}

#[test]
fn test_create_secure_delegate_survives_nil() {
    let env = env();
    // Simulate what EnvironmentCleanup does, then restore
    let result: bool = env
        .eval(
            r#"
            local function myFunc() return 42 end
            -- EnvironmentCleanup nils it
            CreateSecureDelegate = nil
            assert(CreateSecureDelegate == nil, "should be nil after cleanup")
            return true
            "#,
        )
        .unwrap();
    assert!(result);

    // Restore it (as the loader does after EnvironmentCleanup)
    env.restore_post_cleanup_globals();

    let result: bool = env
        .eval(
            r#"
            local function myFunc() return 42 end
            local delegate = CreateSecureDelegate(myFunc)
            return delegate() == 42
            "#,
        )
        .unwrap();
    assert!(
        result,
        "CreateSecureDelegate should work after restore_post_cleanup_globals"
    );
}

#[test]
fn test_post_cleanup_restore_preserves_existing_globals() {
    let env = env();
    env.exec(
        r#"
        GetTime = function() return "blizzard-gettime" end
        ToggleCharacter = function() return "blizzard-toggle-character" end
        "#,
    )
    .unwrap();

    env.restore_post_cleanup_globals();

    let result: (String, String) = env
        .eval(
            r#"
            return GetTime(), ToggleCharacter()
            "#,
        )
        .unwrap();

    assert_eq!(
        result,
        (
            "blizzard-gettime".to_string(),
            "blizzard-toggle-character".to_string()
        ),
        "EnvironmentCleanup restore must not rerun base global registration over loaded Blizzard globals"
    );
}

#[test]
fn test_post_cleanup_restore_does_not_create_character_subframes() {
    let env = env();
    env.exec("CHARACTERFRAME_SUBFRAMES = nil").unwrap();

    env.restore_post_cleanup_globals();

    let exists: bool = env
        .eval("return CHARACTERFRAME_SUBFRAMES ~= nil")
        .expect("character subframe global probe should run");

    assert!(
        !exists,
        "EnvironmentCleanup restore must not synthesize CharacterFrame.lua globals"
    );
}
