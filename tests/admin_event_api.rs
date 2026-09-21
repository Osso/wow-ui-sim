//! Tests for the global FireEvent function (synchronous event dispatch).
//! Uses real WoW event names since RegisterEvent validates against the known list.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

fn env_with_intrinsic_event_frame() -> WowLuaEnv {
    let env = env();
    let root = tempfile::tempdir().unwrap();
    let toc = root.path().join("SynchronousEventProbe.toc");
    std::fs::write(&toc, "## Title: Synchronous event probe\nProbe.xml\n").unwrap();
    std::fs::write(
        root.path().join("Probe.xml"),
        r#"
        <Ui>
            <Frame name="SynchronousEventPre" virtual="true">
                <Scripts><OnEvent intrinsicOrder="precall">
                    SynchronousEventProbe(self, 'pre', event, ...)
                </OnEvent></Scripts>
            </Frame>
            <Frame name="SynchronousEventPost" virtual="true" inherits="SynchronousEventPre">
                <Scripts><OnEvent intrinsicOrder="postcall">
                    SynchronousEventProbe(self, 'post', event, ...)
                </OnEvent></Scripts>
            </Frame>
            <Frame name="SynchronousEventFrame" parent="UIParent" inherits="SynchronousEventPost"/>
        </Ui>
    "#,
    )
    .unwrap();
    let loaded = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env.exec(
        r#"
        assert(type(SynchronousEventFrame:GetScript('OnEvent', 0)) == 'function')
        assert(SynchronousEventFrame:GetScript('OnEvent', 1) == nil)
        assert(type(SynchronousEventFrame:GetScript('OnEvent', 2)) == 'function')
        assert(SynchronousEventFrame:GetScript('OnEvent') == nil)
    "#,
    )
    .expect("real XML installs both intrinsic bindings independently of the normal script");
    env
}

#[test]
fn test_fire_event_intrinsic_order_and_unit_filter() {
    let env = env_with_intrinsic_event_frame();
    env.exec(
        r#"
        local calls = {}
        local frame = SynchronousEventFrame
        frame:RegisterUnitEvent('UNIT_AURA', 'player')
        SynchronousEventProbe = function(self, label, event, ...)
            local unit, serial, empty, flag = ...
            assert(self == frame and event == 'UNIT_AURA')
            assert(select('#', ...) == 4 and unit == 'player' and serial == 17)
            assert(empty == nil and flag == false)
            calls[#calls + 1] = label
        end
        frame:SetScript('OnEvent', function(self, event, ...)
            SynchronousEventProbe(self, 'normal', event, ...)
        end)
        assert(frame:GetScript('OnEvent') == frame:GetScript('OnEvent', 1))
        A_Admin.FireEvent('UNIT_AURA', 'target', 17, nil, false)
        A_Admin.FireEvent('UNIT_AURA')
        assert(#calls == 0, 'unit filter must cover every script binding')
        FireEvent('UNIT_AURA', 'player', 17, nil, false)
        A_Admin.FireEvent('UNIT_AURA', 'player', 17, nil, false)
        assert(table.concat(calls, ',') == 'pre,normal,post,pre,normal,post')
    "#,
    )
    .expect("synchronous events preserve binding order, payloads and unit filtering");
}

#[test]
fn test_fire_event_frame_callbacks_keep_their_own_unit_filter() {
    let env = env_with_intrinsic_event_frame();
    env.exec(r#"
        local calls = {}
        local frame = SynchronousEventFrame
        frame:RegisterUnitEventCallback('UNIT_HEALTH', function(owner, unit)
            assert(owner == frame and unit == 'target')
            calls[#calls + 1] = 'callback'
        end, 'target')
        frame:RegisterUnitEvent('UNIT_HEALTH', 'player')
        SynchronousEventProbe = function(self, label, event, unit)
            assert(self == frame and event == 'UNIT_HEALTH' and unit == 'player')
            calls[#calls + 1] = label
        end
        assert(frame:GetScript('OnEvent') == nil)
        assert(type(frame:GetScript('OnEvent', 0)) == 'function')
        assert(type(frame:GetScript('OnEvent', 2)) == 'function')
        A_Admin.FireEvent('UNIT_HEALTH', 'target')
        assert(table.concat(calls, ',') == 'callback', 'frame filter must not suppress its separate callback')
        A_Admin.FireEvent('UNIT_HEALTH', 'player')
        assert(table.concat(calls, ',') == 'callback,pre,post')
    "#).expect("intrinsic-only frames and independently filtered callbacks both receive events");
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn test_fire_event_global_callbacks_precede_frame_bindings() {
    let env = env_with_intrinsic_event_frame();
    env.exec(
        r#"
        local calls = {}
        RegisterEventCallback('UNIT_HEALTH', function(owner, unit)
            assert(owner == nil and unit == 'player')
            calls[#calls + 1] = 'global'
        end)
        RegisterUnitEventCallback('UNIT_HEALTH', function(owner, unit)
            assert(owner == nil and unit == 'player')
            calls[#calls + 1] = 'global-unit'
        end, 'player')
        local frame = SynchronousEventFrame
        frame:RegisterUnitEventCallback('UNIT_HEALTH', function(owner, unit)
            assert(owner == frame and unit == 'player')
            calls[#calls + 1] = 'frame-unit'
        end, 'player')
        frame:RegisterUnitEvent('UNIT_HEALTH', 'player')
        SynchronousEventProbe = function(_, label) calls[#calls + 1] = label end
        frame:SetScript('OnEvent', function() calls[#calls + 1] = 'normal' end)
        A_Admin.FireEvent('UNIT_HEALTH', 'player')
        assert(table.concat(calls, ',') == 'global,global-unit,frame-unit,pre,normal,post')
    "#,
    )
    .expect("existing global and per-frame callback ordering precedes all script bindings");
}

#[test]
fn test_fire_event_reports_intrinsic_errors_and_continues_bindings() {
    let env = env_with_intrinsic_event_frame();
    env.exec(
        r#"
        local reported, calls = {}, {}
        seterrorhandler(function(message) reported[#reported + 1] = tostring(message) end)
        local frame = SynchronousEventFrame
        frame:RegisterEvent('PLAYER_LOGIN')
        SynchronousEventProbe = function(_, label)
            if label == 'pre' then error('synchronous-intrinsic-probe') end
            calls[#calls + 1] = label
        end
        frame:SetScript('OnEvent', function() calls[#calls + 1] = 'normal' end)
        A_Admin.FireEvent('PLAYER_LOGIN')
        assert(#reported == 1 and reported[1]:find('synchronous-intrinsic-probe', 1, true))
        assert(table.concat(calls, ',') == 'normal,post')
    "#,
    )
    .expect("intrinsic failures reach the error handler without skipping later bindings");
}

// ============================================================================
// FireEvent with string + number args
// ============================================================================

#[test]
fn test_fire_event_handler_receives_event_and_args() {
    let env = env();
    let (received, arg1, arg2): (bool, bool, bool) = env
        .eval(
            r#"
            local received = false
            local got_arg1, got_arg2
            local f = CreateFrame("Frame")
            f:RegisterEvent("PLAYER_ENTERING_WORLD")
            f:SetScript("OnEvent", function(self, event, a1, a2)
                if event == "PLAYER_ENTERING_WORLD" then
                    received = true
                    got_arg1 = a1
                    got_arg2 = a2
                end
            end)
            FireEvent("PLAYER_ENTERING_WORLD", true, false)
            return received, got_arg1, got_arg2
            "#,
        )
        .unwrap();
    assert!(received);
    assert!(arg1);
    assert!(!arg2);
}

#[test]
fn test_event_handler_survives_garbage_collection() {
    let env = env();
    let received: bool = env
        .eval(
            r#"
            local f = CreateFrame("Frame")
            f:RegisterEvent("PLAYER_ENTERING_WORLD")
            f:SetScript("OnEvent", function(self, event)
                if event == "PLAYER_ENTERING_WORLD" then
                    _G.__gc_event_received = true
                end
            end)
            collectgarbage("collect")
            FireEvent("PLAYER_ENTERING_WORLD")
            return _G.__gc_event_received == true
            "#,
        )
        .unwrap();
    assert!(received, "event handler closure should stay valid after GC");
}

#[test]
fn test_register_event_callback_uses_event_dispatch_index() {
    let env = env();
    let received: bool = env
        .eval(
            r#"
            local received = false
            local f = CreateFrame("Frame")
            f:RegisterEventCallback("MINIMAP_PING", function()
                received = true
            end)
            FireEvent("MINIMAP_PING")
            return received
            "#,
        )
        .unwrap();
    assert!(received);
}

#[test]
fn test_event_handler_with_traceback_survives_garbage_collection() {
    let env = env();
    let received: bool = env
        .eval(
            r#"
            local f = CreateFrame("Frame")
            f:RegisterEvent("PLAYER_ENTERING_WORLD")
            local function makeHandler()
                local marker = "alive"
                return function(self, event)
                    if event == "PLAYER_ENTERING_WORLD" and marker == "alive" then
                        _G.__protected_gc_event_received = true
                    end
                end
            end
            f:SetScript("OnEvent", makeHandler())
            collectgarbage("collect")
            FireEvent("PLAYER_ENTERING_WORLD")
            return _G.__protected_gc_event_received == true
            "#,
        )
        .unwrap();
    assert!(
        received,
        "protected event dispatch should keep closures valid after GC"
    );
}

#[test]
fn test_hooked_event_handler_survives_garbage_collection() {
    let env = env();
    let (base_called, hook_called): (bool, bool) = env
        .eval(
            r#"
            local f = CreateFrame("Frame")
            f:RegisterEvent("PLAYER_ENTERING_WORLD")
            f:SetScript("OnEvent", function(self, event)
                if event == "PLAYER_ENTERING_WORLD" then
                    _G.__hook_gc_base = true
                end
            end)
            f:HookScript("OnEvent", function(self, event)
                if event == "PLAYER_ENTERING_WORLD" then
                    _G.__hook_gc_hook = true
                end
            end)
            collectgarbage("collect")
            FireEvent("PLAYER_ENTERING_WORLD")
            return _G.__hook_gc_base == true, _G.__hook_gc_hook == true
            "#,
        )
        .unwrap();
    assert!(base_called, "base event handler should survive GC");
    assert!(hook_called, "hooked event handler should survive GC");
}

#[test]
fn test_frame_method_event_dispatch_survives_garbage_collection() {
    let env = env();
    let received: bool = env
        .eval(
            r#"
            do
                local f = CreateFrame("Frame")
                function f:PLAYER_ENTERING_WORLD()
                    _G.__method_dispatch_gc_received = true
                end
                f:SetScript("OnEvent", function(self, event, ...)
                    self[event](self, ...)
                end)
                f:RegisterEvent("PLAYER_ENTERING_WORLD")
            end
            collectgarbage("collect")
            FireEvent("PLAYER_ENTERING_WORLD")
            return _G.__method_dispatch_gc_received == true
            "#,
        )
        .unwrap();
    assert!(received, "frame table methods should stay valid after GC");
}

// ============================================================================
// Only registered frames receive the event
// ============================================================================

#[test]
fn test_fire_event_only_registered_frames_receive_it() {
    let env = env();
    let (got_a, got_b): (bool, bool) = env
        .eval(
            r#"
            local got_a = false
            local got_b = false
            local fa = CreateFrame("Frame")
            fa:RegisterEvent("PLAYER_LOGIN")
            fa:SetScript("OnEvent", function(self, event)
                if event == "PLAYER_LOGIN" then got_a = true end
            end)
            local fb = CreateFrame("Frame")
            fb:RegisterEvent("PLAYER_LOGOUT")
            fb:SetScript("OnEvent", function(self, event)
                if event == "PLAYER_LOGOUT" then got_b = true end
            end)
            FireEvent("PLAYER_LOGIN")
            return got_a, got_b
            "#,
        )
        .unwrap();
    assert!(got_a, "frame A should have received PLAYER_LOGIN");
    assert!(!got_b, "frame B should not have received PLAYER_LOGIN");
}

// ============================================================================
// FireEvent with no args
// ============================================================================

#[test]
fn test_fire_event_with_no_args() {
    let env = env();
    let received: bool = env
        .eval(
            r#"
            local received = false
            local f = CreateFrame("Frame")
            f:RegisterEvent("PLAYER_ALIVE")
            f:SetScript("OnEvent", function(self, event)
                if event == "PLAYER_ALIVE" then
                    received = true
                end
            end)
            FireEvent("PLAYER_ALIVE")
            return received
            "#,
        )
        .unwrap();
    assert!(received);
}

#[test]
fn test_fire_event_no_args_extra_params_are_nil() {
    let env = env();
    let (received, arg1_nil): (bool, bool) = env
        .eval(
            r#"
            local received = false
            local arg1_nil = false
            local f = CreateFrame("Frame")
            f:RegisterEvent("VARIABLES_LOADED")
            f:SetScript("OnEvent", function(self, event, a1)
                if event == "VARIABLES_LOADED" then
                    received = true
                    arg1_nil = (a1 == nil)
                end
            end)
            FireEvent("VARIABLES_LOADED")
            return received, arg1_nil
            "#,
        )
        .unwrap();
    assert!(received);
    assert!(
        arg1_nil,
        "first extra arg should be nil when FireEvent has no args"
    );
}

// ============================================================================
// Standard WoW events
// ============================================================================

#[test]
fn test_fire_event_player_entering_world() {
    let env = env();
    let received: bool = env
        .eval(
            r#"
            local received = false
            local f = CreateFrame("Frame")
            f:RegisterEvent("PLAYER_ENTERING_WORLD")
            f:SetScript("OnEvent", function(self, event)
                if event == "PLAYER_ENTERING_WORLD" then
                    received = true
                end
            end)
            FireEvent("PLAYER_ENTERING_WORLD")
            return received
            "#,
        )
        .unwrap();
    assert!(received);
}

#[test]
fn test_fire_event_boolean_args() {
    let env = env();
    let (received, is_initial, is_reload): (bool, bool, bool) = env
        .eval(
            r#"
            local received = false
            local got_initial, got_reload
            local f = CreateFrame("Frame")
            f:RegisterEvent("PLAYER_ENTERING_WORLD")
            f:SetScript("OnEvent", function(self, event, isInitial, isReloadUI)
                if event == "PLAYER_ENTERING_WORLD" then
                    received = true
                    got_initial = isInitial
                    got_reload = isReloadUI
                end
            end)
            FireEvent("PLAYER_ENTERING_WORLD", true, false)
            return received, got_initial, got_reload
            "#,
        )
        .unwrap();
    assert!(received);
    assert!(is_initial);
    assert!(!is_reload);
}

// ============================================================================
// Multiple frames all receive the same event
// ============================================================================

#[test]
fn test_fire_event_multiple_frames_all_receive() {
    let env = env();
    let (got1, got2): (bool, bool) = env
        .eval(
            r#"
            local got1 = false
            local got2 = false
            local f1 = CreateFrame("Frame")
            f1:RegisterEvent("ZONE_CHANGED")
            f1:SetScript("OnEvent", function(self, event)
                if event == "ZONE_CHANGED" then got1 = true end
            end)
            local f2 = CreateFrame("Frame")
            f2:RegisterEvent("ZONE_CHANGED")
            f2:SetScript("OnEvent", function(self, event)
                if event == "ZONE_CHANGED" then got2 = true end
            end)
            FireEvent("ZONE_CHANGED")
            return got1, got2
            "#,
        )
        .unwrap();
    assert!(got1, "frame 1 should receive ZONE_CHANGED");
    assert!(got2, "frame 2 should receive ZONE_CHANGED");
}

// ============================================================================
// String argument passthrough
// ============================================================================

#[test]
fn test_fire_event_string_arg() {
    let env = env();
    let (received, unit): (bool, String) = env
        .eval(
            r#"
            local received = false
            local got_unit
            local f = CreateFrame("Frame")
            f:RegisterEvent("UNIT_HEALTH")
            f:SetScript("OnEvent", function(self, event, unit)
                if event == "UNIT_HEALTH" then
                    received = true
                    got_unit = unit
                end
            end)
            FireEvent("UNIT_HEALTH", "player")
            return received, got_unit
            "#,
        )
        .unwrap();
    assert!(received);
    assert_eq!(unit, "player");
}

#[test]
fn test_fire_event_runs_matching_unit_event_callback() {
    let env = env();
    let (registered, fired_player, fired_target, owner_matches): (bool, bool, bool, bool) = env
        .eval(
            r#"
            local firedPlayer = false
            local firedTarget = false
            local ownerMatches = false
            local f = CreateFrame("Frame")
            local registered = f:RegisterUnitEventCallback("UNIT_HEALTH", function(owner, unit)
                ownerMatches = (owner == f)
                if unit == "player" then
                    firedPlayer = true
                elseif unit == "target" then
                    firedTarget = true
                end
            end, "player")
            FireEvent("UNIT_HEALTH", "target")
            FireEvent("UNIT_HEALTH", "player")
            return registered == true, firedPlayer, firedTarget, ownerMatches
            "#,
        )
        .unwrap();
    assert!(
        registered,
        "unit event callback should report successful registration"
    );
    assert!(fired_player, "matching unit should invoke the callback");
    assert!(
        !fired_target,
        "non-matching unit should not invoke the callback"
    );
    assert!(
        owner_matches,
        "callback owner should be the registering frame"
    );
}

#[test]
fn test_register_unit_event_reports_registered_unit() {
    let env = env();
    let (registered, unit, invalid_registered, invalid_unit_is_nil): (bool, String, bool, bool) =
        env.eval(
            r#"
            local f = CreateFrame("Frame")
            f:RegisterUnitEvent("UNIT_HEALTH", "player")
            local registered, unit = f:IsEventRegistered("UNIT_HEALTH")

            local invalid = CreateFrame("Frame")
            invalid:RegisterUnitEvent("UNIT_HEALTH", "not_a_unit")
            local invalidRegistered, invalidUnit = invalid:IsEventRegistered("UNIT_HEALTH")

            return registered, unit, invalidRegistered, invalidUnit == nil
            "#,
        )
        .unwrap();

    assert!(registered);
    assert_eq!(unit, "player");
    assert!(invalid_registered);
    assert!(invalid_unit_is_nil);
}
