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
    "#,
    )
    .unwrap();
}

fn assert_protected_input_method(method: &str, getter: &str) {
    let env = loaded_env();
    env.exec(&format!(
        r#"
        assert(type(GameEvent.HandleActionBlocked) == 'function')
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
