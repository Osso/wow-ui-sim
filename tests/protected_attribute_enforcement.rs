//! Protected-frame attribute write enforcement.
//!
//! Real WoW rejects `SetAttribute` / `SetAttributeNoHandler` on protected
//! frames when the caller is tainted (insecure). Secure callers can write
//! freely; insecure callers silently drop the write.
//!
//! These tests simulate insecure context by stamping the running chunk with
//! an addon taint via `debug.setstacktaint`.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

fn mark_frame_protected(env: &WowLuaEnv, frame_global: &str) {
    let name = frame_global.to_string();
    let state = env.state().borrow();
    let id = state
        .widgets
        .get_id_by_name(&name)
        .unwrap_or_else(|| panic!("frame {name} should exist"));
    drop(state);
    let mut state = env.state().borrow_mut();
    state
        .widgets
        .get_mut(id)
        .expect("widget exists")
        .is_protected = true;
}

#[test]
fn get_attribute_wildcard_lookup_preserves_explicit_false() {
    let env = env();
    let (
        direct_false,
        wildcard_false,
        nil_prefix_wildcard_false,
        direct_true,
        wildcard_true,
        wildcard_string,
    ): (
        bool,
        bool,
        bool,
        bool,
        bool,
        String,
    ) = env
        .eval(
            r#"
        local frame = CreateFrame("Button", "AttributeWildcardFalseProbe", UIParent)
        frame:SetAttribute("harmtype1", false)
        frame:SetAttribute("*type1", false)
        frame:SetAttribute("helptype1", true)
        frame:SetAttribute("*type2", "spell")

        return frame:GetAttribute("harmtype1") == false,
            frame:GetAttribute("harm", "type", "1") == false,
            frame:GetAttribute(nil, "type", "1") == false,
            frame:GetAttribute("helptype1") == true,
            frame:GetAttribute("help", "type", "1") == true,
            frame:GetAttribute("help", "type", "2")
    "#,
        )
        .unwrap();

    assert!(direct_false);
    assert!(wildcard_false);
    assert!(nil_prefix_wildcard_false);
    assert!(direct_true);
    assert!(wildcard_true);
    assert_eq!(wildcard_string, "spell");
}

#[test]
fn get_attribute_reports_live_client_arity_errors() {
    let env = env();
    let (one_nil_ok, two_nil_name_ok, two_prefix_name_ok, three_nil_suffix_ok): (
        bool,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
        local frame = CreateFrame("Button", "AttributeArityProbe", UIParent)
        return pcall(function() return frame:GetAttribute(nil) end),
            pcall(function() return frame:GetAttribute(nil, "type") end),
            pcall(function() return frame:GetAttribute("help", "type") end),
            pcall(function() return frame:GetAttribute("help", "type", nil) end)
    "#,
        )
        .unwrap();

    assert!(!one_nil_ok, "nil direct name should raise an arity error");
    assert!(
        !two_nil_name_ok,
        "nil prefix without explicit suffix should raise an arity error"
    );
    assert!(two_prefix_name_ok, "two string args are valid wildcard lookup");
    assert!(
        three_nil_suffix_ok,
        "explicit nil suffix is valid wildcard lookup"
    );
}

#[test]
fn secure_caller_can_set_attribute_on_protected_frame() {
    let env = env();
    env.exec(
        r#"
        Protected = CreateFrame("Frame", "ProtectedSecureWrite", UIParent)
    "#,
    )
    .unwrap();
    mark_frame_protected(&env, "ProtectedSecureWrite");

    env.exec(
        r#"
        Protected:SetAttribute("foo", "bar")
    "#,
    )
    .unwrap();

    let got: String = env
        .eval(r#"return tostring(Protected:GetAttribute("foo"))"#)
        .unwrap();
    assert_eq!(
        got, "bar",
        "secure caller's write must land on the protected frame"
    );
}

#[test]
fn insecure_caller_cannot_set_attribute_on_protected_frame() {
    let env = env();
    env.exec(
        r#"
        Protected = CreateFrame("Frame", "ProtectedInsecureWrite", UIParent)
        A_Admin.SetInCombat(true)
    "#,
    )
    .unwrap();
    mark_frame_protected(&env, "ProtectedInsecureWrite");

    env.exec(
        r#"
        debug.setstacktaint("evil-addon")
        Protected:SetAttribute("foo", "attempted")
        debug.setstacktaint(nil)
        A_Admin.SetInCombat(false)
    "#,
    )
    .unwrap();

    let got: String = env
        .eval(r#"return tostring(Protected:GetAttribute("foo"))"#)
        .unwrap();
    assert_eq!(
        got, "nil",
        "insecure write on a protected frame must be dropped silently"
    );
}

#[test]
fn insecure_caller_can_set_attribute_on_protected_frame_out_of_combat() {
    let env = env();
    env.exec(
        r#"
        Protected = CreateFrame("Frame", "ProtectedOutOfCombatWrite", UIParent)
        A_Admin.SetInCombat(false)
    "#,
    )
    .unwrap();
    mark_frame_protected(&env, "ProtectedOutOfCombatWrite");

    env.exec(
        r#"
        debug.setstacktaint("evil-addon")
        Protected:SetAttribute("foo", "allowed")
        debug.setstacktaint(nil)
    "#,
    )
    .unwrap();

    let got: String = env
        .eval(r#"return tostring(Protected:GetAttribute("foo"))"#)
        .unwrap();
    assert_eq!(
        got, "allowed",
        "insecure write on a protected frame should be allowed out of combat"
    );
}

#[test]
fn insecure_caller_can_set_attribute_on_unprotected_frame() {
    let env = env();
    env.exec(
        r#"
        Unprotected = CreateFrame("Frame", "UnprotectedInsecureWrite", UIParent)
        debug.setstacktaint("evil-addon")
        Unprotected:SetAttribute("foo", "allowed")
        debug.setstacktaint(nil)
    "#,
    )
    .unwrap();

    let got: String = env
        .eval(r#"return tostring(Unprotected:GetAttribute("foo"))"#)
        .unwrap();
    assert_eq!(
        got, "allowed",
        "insecure write on an unprotected frame must succeed"
    );
}

#[test]
fn insecure_caller_cannot_set_attribute_no_handler_on_protected_frame() {
    let env = env();
    env.exec(
        r#"
        Protected = CreateFrame("Frame", "ProtectedNoHandlerBlock", UIParent)
        A_Admin.SetInCombat(true)
    "#,
    )
    .unwrap();
    mark_frame_protected(&env, "ProtectedNoHandlerBlock");

    env.exec(
        r#"
        debug.setstacktaint("evil-addon")
        Protected:SetAttributeNoHandler("foo", "attempted")
        debug.setstacktaint(nil)
        A_Admin.SetInCombat(false)
    "#,
    )
    .unwrap();

    let got: String = env
        .eval(r#"return tostring(Protected:GetAttribute("foo"))"#)
        .unwrap();
    assert_eq!(
        got, "nil",
        "insecure SetAttributeNoHandler on a protected frame must be dropped silently"
    );
}

#[test]
fn unchanged_scalar_attribute_refires_on_attribute_changed() {
    let env = env();
    let (scalar_count, false_count, false_values, false_visible_during_handler, stored_false):
        (i32, i32, bool, bool, bool) = env.eval(r#"
            local frame = CreateFrame("Frame", "UnchangedAttributeRefire", UIParent)
            local scalarCount, falseCount = 0, 0
            local falseValues, falseVisibleDuringHandler = true, true
            frame:SetScript("OnAttributeChanged", function(self, name, value)
                if name == "showgrid" then scalarCount = scalarCount + 1 end
                if name == "flag" then
                    falseCount = falseCount + 1
                    falseValues = falseValues and value == false
                    falseVisibleDuringHandler = falseVisibleDuringHandler and self:GetAttribute("flag") == nil
                end
            end)
            frame:SetAttribute("showgrid", 1)
            frame:SetAttribute("showgrid", 1)
            frame:SetAttribute("flag", false)
            frame:SetAttribute("flag", false)
            return scalarCount, falseCount, falseValues, falseVisibleDuringHandler, frame:GetAttribute("flag") == false
        "#).unwrap();
    assert_eq!(scalar_count, 2);
    assert_eq!(false_count, 2);
    assert!(false_values);
    assert!(false_visible_during_handler);
    assert!(stored_false);
}

fn assert_attribute_handler_error_preserves_runtime(install_handler: &str) {
    let env = env();
    let source = r#"
        local frame = CreateFrame("Frame")
        local other = CreateFrame("Frame")
        local handled, subsequent = 0, 0
        local function onAttributeChanged(self, name, value)
            if value == "boom" then error("nested attribute failure") end
            handled = handled + 1
        end
        INSTALL_HANDLER
        other:SetScript("OnAttributeChanged", function()
            subsequent = subsequent + 1
        end)

        local step = 0
        local function nextStep()
            step = step + 1
            return step
        end
        local function caller()
            local marker = "caller intact"
            local retained = { label = "upvalue intact", nextStep = nextStep }
            frame:SetAttribute("first", "boom")
            local firstStep = retained.nextStep()
            local match = string.match("ok:37", "ok:(%d+)")
            local allocated = { marker, match, retained.nextStep() }
            frame:SetAttribute("second", "ok")
            other:SetAttribute("later", true)
            return allocated[1], retained.label, firstStep, allocated[2], allocated[3]
        end
        local marker, label, firstStep, match, secondStep = caller()
        return marker, label, firstStep, match, secondStep, handled, subsequent
    "#
    .replace("INSTALL_HANDLER", install_handler);
    let actual: (String, String, i32, String, i32, i32, i32) = env
        .eval(&source)
        .expect("a failed attribute callback must not corrupt the active Lua caller");
    assert_eq!(
        actual,
        (
            "caller intact".into(),
            "upvalue intact".into(),
            1,
            "37".into(),
            2,
            1,
            1,
        )
    );
    let errors = env.state().borrow().lua_errors.clone();
    assert_eq!(
        errors.len(),
        1,
        "the failed callback should be recorded once"
    );
    assert!(
        errors[0].contains("nested attribute failure"),
        "the recorded error should identify the failed callback: {errors:?}"
    );
}

#[test]
fn attribute_handler_error_preserves_caller_and_subsequent_handlers() {
    assert_attribute_handler_error_preserves_runtime(
        "frame:SetScript('OnAttributeChanged', onAttributeChanged)",
    );
}

#[test]
fn attribute_handler_error_preserves_caller_and_subsequent_handlers_direct_method() {
    assert_attribute_handler_error_preserves_runtime(
        "frame.OnAttributeChanged = onAttributeChanged",
    );
}

#[test]
fn set_attribute_dispatches_direct_on_attribute_changed_method() {
    let env = env();
    let got: String = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "DirectAttributeChangedMethod", UIParent)

            function frame:OnAttributeChanged(name, value)
                self.result = name .. "=" .. tostring(value)
            end

            frame:SetAttribute("open-to-category", 7)

            return frame.result or "missing"
        "#,
        )
        .unwrap();

    assert_eq!(
        got, "open-to-category=7",
        "SetAttribute should dispatch direct OnAttributeChanged methods"
    );
}
