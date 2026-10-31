use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

fn loaded_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )));
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    wow_ui_sim::xml::register_intrinsic_templates();
    for (name, toc) in discover_blizzard_addons_for_screen(&ui, ScreenKind::Game) {
        load_addon(&env.loader_env(), &toc).unwrap_or_else(|err| panic!("load {name}: {err}"));
    }
    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn protected_method_set_id_routes_blocked_event_and_allows_trusted_calls() {
    let env = loaded_env();
    env.exec(
        r#"
        assert(type(GameEvent.HandleActionBlocked) == 'function')
        local routed
        hooksecurefunc(GameEvent, 'HandleActionBlocked', function(_, event, owner)
            if event == 'ADDON_ACTION_BLOCKED' then routed = owner end
        end)
        local frame = CreateFrame('Frame', 'ProtectedMethodId', UIParent)
        local events = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ADDON_ACTION_BLOCKED')
        listener:SetScript('OnEvent', function(_, _, owner, action)
            events[#events + 1] = { owner, action }
        end)
        A_Admin.SetFrameProtected('ProtectedMethodId', true)
        A_Admin.SetInCombat(false)
        debug.setstacktaint('ProtectedMethodAddon')
        frame:SetID(11)
        assert(frame:GetID() == 11, 'out-of-combat insecure SetID must work')
        debug.setstacktaint(nil)
        A_Admin.SetInCombat(true)
        frame:SetID(22)
        assert(frame:GetID() == 22, 'secure combat SetID must work')
        debug.setstacktaint('ProtectedMethodAddon')
        frame:SetID(33)
        debug.setstacktaint(nil)
        assert(frame:GetID() == 22, 'insecure combat SetID must be refused')
        assert(#events == 1, 'blocked action must fire once')
        assert(events[1][1] == 'ProtectedMethodAddon')
        assert(events[1][2] == 'ProtectedMethodId:SetID()')
        assert(routed == 'ProtectedMethodAddon', 'Blizzard handler must receive the event')
    "#,
    )
    .unwrap();
}

fn assert_protected_input_method(method: &str, getter: &str) {
    let env = loaded_env();
    env.exec(&format!(
        r#"
        assert(type(GameEvent.HandleActionBlocked) == 'function')
        local routed
        hooksecurefunc(GameEvent, 'HandleActionBlocked', function(_, event, owner)
            if event == 'ADDON_ACTION_BLOCKED' then routed = owner end
        end)
        local frame = CreateFrame('Frame', 'ProtectedInputMethod', UIParent)
        local events = {{}}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ADDON_ACTION_BLOCKED')
        listener:SetScript('OnEvent', function(_, _, owner, action)
            events[#events + 1] = {{ owner, action }}
        end)
        A_Admin.SetFrameProtected('ProtectedInputMethod', true)
        A_Admin.SetInCombat(false)
        debug.setstacktaint('ProtectedMethodAddon')
        frame:{method}(true)
        assert(frame:{getter}() == true, '{method} must work outside combat')
        debug.setstacktaint(nil)
        A_Admin.SetInCombat(true)
        frame:{method}(false)
        assert(frame:{getter}() == false, '{method} must work for secure callers')
        debug.setstacktaint('ProtectedMethodAddon')
        frame:{method}(true)
        debug.setstacktaint(nil)
        assert(frame:{getter}() == false, '{method} must be refused in combat')
        assert(#events == 1, '{method} must emit one blocked event')
        assert(events[1][1] == 'ProtectedMethodAddon')
        assert(events[1][2] == 'ProtectedInputMethod:{method}()')
        assert(routed == 'ProtectedMethodAddon', 'Blizzard handler must receive the event')
    "#
    ))
    .unwrap();
}

macro_rules! protected_input_test {
    ($test:ident, $method:literal, $getter:literal) => {
        #[test]
        fn $test() {
            assert_protected_input_method($method, $getter);
        }
    };
}

protected_input_test!(protected_input_mouse, "EnableMouse", "IsMouseEnabled");
protected_input_test!(
    protected_input_wheel,
    "EnableMouseWheel",
    "IsMouseWheelEnabled"
);
protected_input_test!(
    protected_input_keyboard,
    "EnableKeyboard",
    "IsKeyboardEnabled"
);
protected_input_test!(
    protected_input_motion,
    "EnableMouseMotion",
    "IsMouseMotionEnabled"
);
protected_input_test!(
    protected_input_motion_setter,
    "SetMouseMotionEnabled",
    "IsMouseMotionEnabled"
);
protected_input_test!(
    protected_input_click_setter,
    "SetMouseClickEnabled",
    "IsMouseClickEnabled"
);

protected_input_test!(
    protected_input_propagate_clicks,
    "SetPropagateMouseClicks",
    "CanPropagateMouseClicks"
);
protected_input_test!(
    protected_input_propagate_motion,
    "SetPropagateMouseMotion",
    "CanPropagateMouseMotion"
);
protected_input_test!(
    protected_input_parent_scale,
    "SetIgnoreParentScale",
    "IsIgnoringParentScale"
);
protected_input_test!(
    protected_input_collapses,
    "SetCollapsesLayout",
    "CollapsesLayout"
);
protected_input_test!(
    protected_input_clamped,
    "SetClampedToScreen",
    "IsClampedToScreen"
);
protected_input_test!(
    protected_input_parent_level,
    "SetUsingParentLevel",
    "IsUsingParentLevel"
);

protected_input_test!(
    protected_input_gamepad_button,
    "EnableGamePadButton",
    "IsGamePadButtonEnabled"
);
protected_input_test!(
    protected_input_gamepad_stick,
    "EnableGamePadStick",
    "IsGamePadStickEnabled"
);
protected_input_test!(
    protected_input_round_layout,
    "SetRoundLayoutToNearestPixel",
    "GetRoundLayoutToNearestPixel"
);
protected_input_test!(
    protected_input_hyperlinks,
    "SetHyperlinksEnabled",
    "GetHyperlinksEnabled"
);

fn assert_protected_transition(
    kind: &str,
    method: &str,
    setup: &str,
    out: &str,
    secure: &str,
    blocked: &str,
    out_check: &str,
    secure_check: &str,
) {
    let env = loaded_env();
    env.exec(&format!(
        r#"
        local frame = CreateFrame('{kind}', 'ProtectedTransition', UIParent)
        {setup}
        local events = {{}}
        local routed
        hooksecurefunc(GameEvent, 'HandleActionBlocked', function(_, event, owner)
            if event == 'ADDON_ACTION_BLOCKED' then routed = owner end
        end)
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ADDON_ACTION_BLOCKED')
        listener:SetScript('OnEvent', function(_, _, owner, action)
            events[#events + 1] = {{ owner, action }}
        end)
        A_Admin.SetFrameProtected('ProtectedTransition', true)
        A_Admin.SetInCombat(false)
        debug.setstacktaint('ProtectedMethodAddon')
        {out}
        assert({out_check}, '{method} must work outside combat')
        debug.setstacktaint(nil)
        A_Admin.SetInCombat(true)
        {secure}
        assert({secure_check}, '{method} must work for secure callers')
        debug.setstacktaint('ProtectedMethodAddon')
        {blocked}
        debug.setstacktaint(nil)
        assert({secure_check}, '{method} must be refused in combat')
        assert(#events == 1, '{method} must emit one blocked event')
        assert(events[1][1] == 'ProtectedMethodAddon')
        assert(events[1][2] == 'ProtectedTransition:{method}()')
        assert(routed == 'ProtectedMethodAddon', 'Blizzard handler must receive the event')
    "#
    ))
    .unwrap();
}

#[test]
fn protected_transition_button_enabled() {
    assert_protected_transition(
        "Button",
        "SetEnabled",
        "",
        "frame:SetEnabled(false)",
        "frame:SetEnabled(true)",
        "frame:SetEnabled(false)",
        "not frame:IsEnabled()",
        "frame:IsEnabled()",
    );
}

#[test]
fn protected_transition_scroll_horizontal() {
    assert_protected_transition(
        "ScrollFrame",
        "SetHorizontalScroll",
        "frame:SetSize(100,100); local child = CreateFrame('Frame', nil, frame); child:SetSize(500,500); frame:SetScrollChild(child)",
        "frame:SetHorizontalScroll(10)",
        "frame:SetHorizontalScroll(20)",
        "frame:SetHorizontalScroll(30)",
        "frame:GetHorizontalScroll() == 10",
        "frame:GetHorizontalScroll() == 20",
    );
}

#[test]
fn protected_transition_scroll_vertical() {
    assert_protected_transition(
        "ScrollFrame",
        "SetVerticalScroll",
        "frame:SetSize(100,100); local child = CreateFrame('Frame', nil, frame); child:SetSize(500,500); frame:SetScrollChild(child)",
        "frame:SetVerticalScroll(10)",
        "frame:SetVerticalScroll(20)",
        "frame:SetVerticalScroll(30)",
        "frame:GetVerticalScroll() == 10",
        "frame:GetVerticalScroll() == 20",
    );
}

#[test]
fn protected_transition_cooldown() {
    assert_protected_transition(
        "Cooldown",
        "SetCooldown",
        "",
        "frame:SetCooldown(10, 11)",
        "frame:SetCooldown(20, 21)",
        "frame:SetCooldown(30, 31)",
        "select(2, frame:GetCooldownTimes()) == 11000",
        "select(2, frame:GetCooldownTimes()) == 21000",
    );
}

#[test]
fn protected_transition_rolesets() {
    assert_protected_transition(
        "Frame",
        "SetRolesets",
        "",
        "frame:SetRolesets('alpha')",
        "frame:SetRolesets('beta')",
        "frame:SetRolesets('gamma')",
        "frame:GetRolesetNames()[1] == 'alpha'",
        "frame:GetRolesetNames()[1] == 'beta'",
    );
}

#[test]
fn protected_transition_passthrough() {
    assert_protected_transition(
        "Frame",
        "SetPassThroughButtons",
        "",
        "frame:SetPassThroughButtons('LeftButton')",
        "frame:SetPassThroughButtons('RightButton')",
        "frame:SetPassThroughButtons('MiddleButton')",
        "frame:ShouldButtonPassThrough('LeftButton')",
        "frame:ShouldButtonPassThrough('RightButton') and not frame:ShouldButtonPassThrough('MiddleButton')",
    );
}

#[test]
fn protected_transition_button_enable() {
    assert_protected_transition(
        "Button",
        "Enable",
        "",
        "frame:Disable(); frame:Enable()",
        "frame:Disable()",
        "frame:Enable()",
        "frame:IsEnabled()",
        "not frame:IsEnabled()",
    );
}

#[test]
fn protected_transition_button_disable() {
    assert_protected_transition(
        "Button",
        "Disable",
        "",
        "frame:Disable()",
        "frame:Enable()",
        "frame:Disable()",
        "not frame:IsEnabled()",
        "frame:IsEnabled()",
    );
}

#[test]
fn protected_transition_scroll_child() {
    assert_protected_transition(
        "ScrollFrame",
        "SetScrollChild",
        "local a = CreateFrame('Frame',nil,frame); local b = CreateFrame('Frame',nil,frame); local c = CreateFrame('Frame',nil,frame)",
        "frame:SetScrollChild(a)",
        "frame:SetScrollChild(b)",
        "frame:SetScrollChild(c)",
        "frame:GetScrollChild() == a",
        "frame:GetScrollChild() == b",
    );
}

#[test]
fn protected_transition_clamp_insets() {
    assert_protected_transition(
        "Frame",
        "SetClampRectInsets",
        "",
        "frame:SetClampRectInsets(11,0,0,0)",
        "frame:SetClampRectInsets(22,0,0,0)",
        "frame:SetClampRectInsets(33,0,0,0)",
        "frame:GetClampRectInsets() == 11",
        "frame:GetClampRectInsets() == 22",
    );
}

#[test]
fn protected_transition_hit_insets() {
    assert_protected_transition(
        "Frame",
        "SetHitRectInsets",
        "",
        "frame:SetHitRectInsets(11,0,0,0)",
        "frame:SetHitRectInsets(22,0,0,0)",
        "frame:SetHitRectInsets(33,0,0,0)",
        "frame:GetHitRectInsets() == 11",
        "frame:GetHitRectInsets() == 22",
    );
}

#[test]
fn protected_transition_cooldown_unix() {
    assert_protected_transition(
        "Cooldown",
        "SetCooldownUNIX",
        "",
        "frame:SetCooldownUNIX(10,11)",
        "frame:SetCooldownUNIX(20,21)",
        "frame:SetCooldownUNIX(30,31)",
        "select(2,frame:GetCooldownTimes()) == 11000",
        "select(2,frame:GetCooldownTimes()) == 21000",
    );
}

#[test]
fn protected_transition_cooldown_duration() {
    assert_protected_transition(
        "Cooldown",
        "SetCooldownDuration",
        "",
        "frame:SetCooldownDuration(11)",
        "frame:SetCooldownDuration(21)",
        "frame:SetCooldownDuration(31)",
        "select(2,frame:GetCooldownTimes()) == 11000",
        "select(2,frame:GetCooldownTimes()) == 21000",
    );
}

#[test]
fn protected_transition_cooldown_expiration() {
    assert_protected_transition(
        "Cooldown",
        "SetCooldownFromExpirationTime",
        "",
        "frame:SetCooldownFromExpirationTime(40,11)",
        "frame:SetCooldownFromExpirationTime(50,21)",
        "frame:SetCooldownFromExpirationTime(60,31)",
        "select(2,frame:GetCooldownTimes()) == 11000",
        "select(2,frame:GetCooldownTimes()) == 21000",
    );
}

#[test]
fn protected_transition_cooldown_clear() {
    assert_protected_transition(
        "Cooldown",
        "Clear",
        "frame:SetCooldown(10,11)",
        "frame:Clear()",
        "frame:SetCooldown(20,21)",
        "frame:Clear()",
        "select(2,frame:GetCooldownTimes()) == 0",
        "select(2,frame:GetCooldownTimes()) == 21000",
    );
}

#[test]
fn protected_transition_roleset_add() {
    assert_protected_transition(
        "Frame",
        "AddRoleset",
        "",
        "frame:AddRoleset('alpha')",
        "frame:SetRolesets('beta'); frame:AddRoleset('gamma')",
        "frame:AddRoleset('delta')",
        "frame:GetRolesetNames()[1] == 'alpha'",
        "table.concat(frame:GetRolesetNames(), ',') == 'beta,gamma'",
    );
}

#[test]
fn protected_transition_roleset_remove() {
    assert_protected_transition(
        "Frame",
        "RemoveRoleset",
        "frame:SetRolesets('alpha,beta')",
        "frame:RemoveRoleset('alpha')",
        "frame:SetRolesets('alpha,beta'); frame:RemoveRoleset('beta')",
        "frame:RemoveRoleset('alpha')",
        "frame:GetRolesetNames()[1] == 'beta'",
        "frame:GetRolesetNames()[1] == 'alpha'",
    );
}

#[test]
fn protected_transition_clear_attribute() {
    assert_protected_transition(
        "Frame",
        "ClearAttribute",
        "frame:SetAttribute('x',11)",
        "frame:ClearAttribute('x')",
        "frame:SetAttribute('x',22)",
        "frame:ClearAttribute('x')",
        "frame:GetAttribute('x') == nil",
        "frame:GetAttribute('x') == 22",
    );
}

#[test]
fn protected_transition_clear_attributes() {
    assert_protected_transition(
        "Frame",
        "ClearAttributes",
        "frame:SetAttribute('x',11)",
        "frame:ClearAttributes()",
        "frame:SetAttribute('x',22)",
        "frame:ClearAttributes()",
        "frame:GetAttribute('x') == nil",
        "frame:GetAttribute('x') == 22",
    );
}

#[test]
fn protected_transition_clear_points_offset() {
    assert_protected_transition(
        "Frame",
        "ClearPointsOffset",
        "frame:SetPoint('CENTER', UIParent, 'CENTER', 11, 12)",
        "frame:ClearPointsOffset()",
        "frame:SetPoint('CENTER', UIParent, 'CENTER', 22, 23)",
        "frame:ClearPointsOffset()",
        "select(4, frame:GetPoint(1)) == 0 and select(5, frame:GetPoint(1)) == 0",
        "select(4, frame:GetPoint(1)) == 22 and select(5, frame:GetPoint(1)) == 23",
    );
}

#[test]
fn clear_attribute_reports_existence_and_dispatches_change() {
    let env = loaded_env();
    env.exec(
        r#"
        local frame = CreateFrame('Frame')
        frame:SetAttribute('x', 11)
        local calls = 0
        frame:SetScript('OnAttributeChanged', function(_, name, value)
            assert(name == 'x' and value == nil)
            calls = calls + 1
        end)
        assert(frame:ClearAttribute('x') == true)
        assert(frame:GetAttribute('x') == nil and calls == 1)
        assert(frame:ClearAttribute('x') == false and calls == 1)
    "#,
    )
    .unwrap();
}

#[test]
fn clear_points_offset_resets_all_points_without_removing_them() {
    let env = loaded_env();
    env.exec(r#"
        local frame = CreateFrame('Frame')
        frame:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 11, 12)
        frame:SetPoint('BOTTOMRIGHT', UIParent, 'BOTTOMRIGHT', 21, 22)
        frame:ClearPointsOffset()
        assert(frame:GetNumPoints() == 2)
        local point, relative, relative_point, x, y = frame:GetPoint(1)
        assert(point == 'TOPLEFT' and relative == UIParent and relative_point == 'TOPLEFT' and x == 0 and y == 0)
        point, relative, relative_point, x, y = frame:GetPoint(2)
        assert(point == 'BOTTOMRIGHT' and relative == UIParent and relative_point == 'BOTTOMRIGHT' and x == 0 and y == 0)
    "#).unwrap();
}

#[test]
fn protected_transition_start_moving() {
    assert_protected_transition(
        "Frame",
        "StartMoving",
        "frame:SetMovable(true)",
        "frame:StartMoving()",
        "frame:StopMovingOrSizing()",
        "frame:StartMoving()",
        "frame:IsDragging()",
        "not frame:IsDragging()",
    );
}

#[test]
fn protected_transition_start_sizing() {
    assert_protected_transition(
        "Frame",
        "StartSizing",
        "frame:SetResizable(true)",
        "frame:StartSizing('BOTTOMRIGHT')",
        "frame:StopMovingOrSizing()",
        "frame:StartSizing('BOTTOMRIGHT')",
        "frame:IsDragging()",
        "not frame:IsDragging()",
    );
}

#[test]
fn protected_transition_stop_moving() {
    assert_protected_transition(
        "Frame",
        "StopMovingOrSizing",
        "frame:SetMovable(true)",
        "frame:StartMoving(); frame:StopMovingOrSizing()",
        "frame:StartMoving()",
        "frame:StopMovingOrSizing()",
        "not frame:IsDragging()",
        "frame:IsDragging()",
    );
}

#[test]
fn protected_transition_cooldown_duration_object() {
    assert_protected_transition(
        "Cooldown",
        "SetCooldownFromDurationObject",
        "local a=C_DurationUtil.CreateDuration(); a:SetTimeFromStart(10,11,1); local b=C_DurationUtil.CreateDuration(); b:SetTimeFromStart(20,21,1); local c=C_DurationUtil.CreateDuration(); c:SetTimeFromStart(30,31,1)",
        "frame:SetCooldownFromDurationObject(a)",
        "frame:SetCooldownFromDurationObject(b)",
        "frame:SetCooldownFromDurationObject(c)",
        "select(2,frame:GetCooldownTimes()) == 11000",
        "select(2,frame:GetCooldownTimes()) == 21000",
    );
}

#[test]
fn protected_transition_defaults() {
    assert_protected_transition(
        "Frame",
        "SetToDefaults",
        "frame:SetSize(11,12)",
        "frame:SetToDefaults()",
        "frame:SetSize(22,23)",
        "frame:SetToDefaults()",
        "frame:GetWidth() == 0",
        "frame:GetWidth() == 22",
    );
}

fn assert_protected_registration(
    method: &str,
    first: &str,
    second: &str,
    third: &str,
    clicks: bool,
) {
    let env = loaded_env();
    env.exec(
        r#"
        RegistrationFrame = CreateFrame('Button', 'ProtectedRegistration', UIParent)
        RegistrationEvents = {}
        RegistrationRouted = nil
        hooksecurefunc(GameEvent, 'HandleActionBlocked', function(_, event, owner)
            if event == 'ADDON_ACTION_BLOCKED' then RegistrationRouted = owner end
        end)
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ADDON_ACTION_BLOCKED')
        listener:SetScript('OnEvent', function(_, _, owner, action)
            RegistrationEvents[#RegistrationEvents + 1] = {owner, action}
        end)
        A_Admin.SetFrameProtected('ProtectedRegistration', true)
        A_Admin.SetInCombat(false)
    "#,
    )
    .unwrap();
    let assert_registered = |expected: &str| {
        let sim = env.state().borrow();
        let id = sim.widgets.get_id_by_name("ProtectedRegistration").unwrap();
        let frame = sim.widgets.get(id).unwrap();
        let buttons = if clicks {
            &frame.registered_click_buttons
        } else {
            &frame.registered_mouse_buttons
        };
        assert_eq!(buttons.len(), 1);
        assert!(buttons.contains(expected));
    };
    env.exec(&format!("debug.setstacktaint('ProtectedMethodAddon'); RegistrationFrame:{method}('{first}'); debug.setstacktaint(nil)")).unwrap();
    assert_registered(first);
    env.exec(&format!(
        "A_Admin.SetInCombat(true); RegistrationFrame:{method}('{second}')"
    ))
    .unwrap();
    assert_registered(second);
    env.exec(&format!("debug.setstacktaint('ProtectedMethodAddon'); RegistrationFrame:{method}('{third}'); debug.setstacktaint(nil)")).unwrap();
    assert_registered(second);
    env.exec(&format!("assert(#RegistrationEvents == 1 and RegistrationEvents[1][1] == 'ProtectedMethodAddon' and RegistrationEvents[1][2] == 'ProtectedRegistration:{method}()' and RegistrationRouted == 'ProtectedMethodAddon')")).unwrap();
}

#[test]
fn protected_registration_clicks() {
    assert_protected_registration(
        "RegisterForClicks",
        "LeftButtonUp",
        "RightButtonUp",
        "MiddleButtonUp",
        true,
    );
}

#[test]
fn protected_registration_mouse() {
    assert_protected_registration(
        "RegisterForMouse",
        "LeftButton",
        "RightButton",
        "MiddleButton",
        false,
    );
}

#[test]
fn protected_transition_raise() {
    assert_protected_transition(
        "Frame",
        "Raise",
        "frame:SetToplevel(true); frame:Show(); local previous = frame:GetRaisedFrameLevel()",
        "frame:Raise()",
        "frame:Raise(); previous = frame:GetRaisedFrameLevel()",
        "frame:Raise()",
        "frame:GetRaisedFrameLevel() > previous",
        "frame:GetRaisedFrameLevel() == previous",
    );
}
