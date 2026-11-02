#![cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
//! Retail 12.1.5 C_PingSecure restrictions, callback slots and radial frame
//! creation (WM38, WM38F; PingManagerSecureDocumentation.lua).

use wow_ui_sim::lua_api::WowLuaEnv;

const SETTERS: &str = r#"{
    SetPendingPingOffScreenCallback = 'pendingPingOffScreen',
    SetPingCooldownStartedCallback = 'pingCooldownStarted',
    SetPingPinFrameAddedCallback = 'pingPinFrameAdded',
    SetPingPinFrameRemovedCallback = 'pingPinFrameRemoved',
    SetPingPinFrameScreenClampStateUpdatedCallback = 'pingPinFrameScreenClampStateUpdated',
    SetPingRadialWheelCreatedCallback = 'pingRadialWheelCreated',
    SetSendMacroPingCallback = 'sendMacroPing',
    SetTogglePingListenerCallback = 'togglePingListener',
}"#;

fn install_forbidden_listener(env: &WowLuaEnv) {
    env.exec(
        r#"
        Reports = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ADDON_ACTION_FORBIDDEN')
        listener:SetScript('OnEvent', function(_, _, addon, functionName, extra)
            assert(extra == nil)
            table.insert(Reports, addon .. '|' .. functionName)
        end)
        "#,
    )
    .expect("forbidden-action listener installs");
}

/// API-level: no pinned consumer calls the setters from tainted code.
#[test]
fn tainted_setters_are_refused_before_parsing_and_keep_their_slots() {
    let env = WowLuaEnv::new().unwrap();
    install_forbidden_listener(&env);
    env.exec(&format!(
        r#"
        local Ping = __secureenv.C_PingSecure
        local setters = {SETTERS}
        local originals = {{}}
        for name, key in pairs(setters) do
            originals[key] = function() end
            assert(select('#', Ping[name](originals[key])) == 0)
            assert(__wow_ping_secure_callbacks[key] == originals[key])
        end
        local expected = {{}}
        local function addon()
            -- Dotted access reports the member name only.
            Ping.SetPendingPingOffScreenCallback(function() end)
            table.insert(expected, 'RestrictionFixture|SetPendingPingOffScreenCallback()')
            for name in pairs(setters) do
                -- A local alias supplies its own debug name.
                local setter = Ping[name]
                assert(select('#', setter(function() end)) == 0)
                -- Refusal precedes argument parsing: malformed input is not an error.
                assert(select('#', setter(23)) == 0)
                table.insert(expected, 'RestrictionFixture|setter()')
                table.insert(expected, 'RestrictionFixture|setter()')
            end
        end
        debug.setobjecttaint(addon, 'RestrictionFixture')
        addon()
        for _, key in pairs(setters) do
            assert(__wow_ping_secure_callbacks[key] == originals[key])
        end
        assert(table.concat(Reports, ';') == table.concat(expected, ';'))
        "#
    ))
    .expect("tainted setters are refused with the live debug name");
}

/// API-level: secure callers must supply a function callback.
#[test]
fn secure_setters_require_a_function_callback() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(&format!(
        r#"
        local Ping = __secureenv.C_PingSecure
        for name, key in pairs({SETTERS}) do
            local original = function() end
            Ping[name](original)
            assert(not pcall(Ping[name]))
            assert(not pcall(Ping[name], nil))
            assert(not pcall(Ping[name], 23))
            assert(__wow_ping_secure_callbacks[key] == original)
            local replacement = function() end
            Ping[name](replacement)
            assert(__wow_ping_secure_callbacks[key] == replacement)
        end
        "#
    ))
    .expect("secure setters store functions and reject missing or other values");
}

#[test]
fn pending_clear_member_is_retired() {
    let env = WowLuaEnv::new().unwrap();
    let kind: String = env
        .eval("return type(__secureenv.C_PingSecure.ClearPendingPingOffScreenCallback)")
        .unwrap();
    assert_eq!(kind, "nil");
}

/// API-level singleton contract; the PingUI consumer case is below.
#[test]
fn create_frame_builds_one_unnamed_frame_and_notifies_once() {
    let env = WowLuaEnv::new().unwrap();
    install_forbidden_listener(&env);
    env.exec(
        r#"
        local Ping = __secureenv.C_PingSecure
        Created = {}
        Ping.SetPingRadialWheelCreatedCallback(function(...)
            table.insert(Created, {count = select('#', ...), frame = ...})
        end)
        local function addon()
            assert(select('#', Ping.CreateFrame()) == 0)
        end
        debug.setobjecttaint(addon, 'CreateFixture')
        addon()
        assert(#Created == 0)
        assert(table.concat(Reports, ';') == 'CreateFixture|CreateFrame()')

        assert(select('#', Ping.CreateFrame()) == 0)
        assert(#Created == 1 and Created[1].count == 1)
        local frame = Created[1].frame
        assert(frame:GetObjectType() == 'Frame' and frame:GetName() == nil)
        Ping.CreateFrame()
        assert(#Created == 1)
        -- An existing frame is not announced to a later callback.
        Ping.SetPingRadialWheelCreatedCallback(function() table.insert(Created, 'late') end)
        Ping.CreateFrame()
        assert(#Created == 1)
        "#,
    )
    .expect("CreateFrame creates and announces exactly one frame");
}

#[test]
fn create_frame_without_callback_misses_the_notification() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local Ping = __secureenv.C_PingSecure
        Ping.CreateFrame()
        Notified = 0
        Ping.SetPingRadialWheelCreatedCallback(function() Notified = Notified + 1 end)
        Ping.CreateFrame()
        assert(Notified == 0)
        "#,
    )
    .expect("a missed creation notification is not recovered");
}

/// API-level argument contract; the UIErrorsFrame consumer case is below.
#[test]
fn display_error_dispatches_nonempty_text_only() {
    let env = WowLuaEnv::new().unwrap();
    install_forbidden_listener(&env);
    env.exec(
        r#"
        local Ping = __secureenv.C_PingSecure
        Errors = {}
        local probe = CreateFrame('Frame')
        probe:RegisterEvent('PING_SYSTEM_ERROR')
        probe:SetScript('OnEvent', function(_, _, text, extra)
            assert(extra == nil)
            table.insert(Errors, text)
        end)
        assert(select('#', Ping.DisplayError('Ping failed')) == 0)
        Ping.DisplayError(42)
        Ping.DisplayError('')
        assert(not pcall(Ping.DisplayError))
        assert(not pcall(Ping.DisplayError, {}))
        local function addon() assert(select('#', Ping.DisplayError('tainted')) == 0) end
        debug.setobjecttaint(addon, 'DisplayFixture')
        addon()
        assert(table.concat(Errors, ';') == 'Ping failed;42')
        assert(table.concat(Reports, ';') == 'DisplayFixture|DisplayError()')
        "#,
    )
    .expect("DisplayError dispatches PING_SYSTEM_ERROR for nonempty text");
}

#[cfg(feature = "retail-12-1-5")]
prefork_full_ui_case! {
fn ping_frame_anchors_to_radial_parent_from_create_frame(env: &WowLuaEnv) {
    // Blizzard_PingUI.lua:24 installs the callback; :38-40 calls CreateFrame on
    // PLAYER_ENTERING_WORLD; :44-48 anchors PingFrame to the announced frame.
    env.exec(
        r#"
        local parent = PingFrame.radialParent
        assert(type(parent) == 'table' and parent:GetObjectType() == 'Frame')
        assert(parent:GetName() == nil)
        local point, relativeTo = PingFrame:GetPoint(1)
        assert(point == 'CENTER' and relativeTo == parent)
        Notified = 0
        __secureenv.C_PingSecure.SetPingRadialWheelCreatedCallback(function()
            Notified = Notified + 1
        end)
        __secureenv.C_PingSecure.CreateFrame()
        assert(Notified == 0)
        "#,
    )
    .expect("PingUI receives the singleton radial frame");
}
}

#[cfg(feature = "retail-12-1-5")]
prefork_full_ui_case! {
fn display_error_reaches_ui_errors_frame(env: &WowLuaEnv) {
    // EventRouting.lua:73 routes PING_SYSTEM_ERROR to EventImplementation.lua:167-168.
    env.exec(
        r#"
        Messages = {}
        hooksecurefunc(UIErrorsFrame, 'AddMessage', function(_, text)
            table.insert(Messages, text)
        end)
        __secureenv.C_PingSecure.DisplayError(PING_FAILED_GENERIC)
        __secureenv.C_PingSecure.DisplayError('')
        assert(#Messages == 1 and Messages[1] == PING_FAILED_GENERIC)
        "#,
    )
    .expect("PING_SYSTEM_ERROR reaches the original error frame consumer");
}
}
