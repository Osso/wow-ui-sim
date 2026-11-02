//! Explicit ping interval contract; behavioral RED not yet observed.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use wow_ui_sim::c_api::PingCooldownInterval;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;

fn ping_env() -> WowLuaEnv {
    WowLuaEnv::new().expect("create ping environment")
}

// Host-owned explicit state fixture; never replaces the public Lua getter.
fn supply_interval(env: &WowLuaEnv, start_time_ms: u32, end_time_ms: u32) {
    env.state().borrow_mut().ping_burst.interval = PingCooldownInterval {
        start_time_ms,
        end_time_ms,
    };
}

fn assert_interval(env: &WowLuaEnv, start: u32, end: u32) {
    env.exec(&format!(
        r#"
        assert(select('#', C_Ping.GetCooldownInfo()) == 1, 'one required DTO')
        local info = C_Ping.GetCooldownInfo()
        assert(type(info) == 'table', 'required nonnil DTO')
        assert(type(info.startTimeMs) == 'number', 'numeric absolute start ms')
        assert(type(info.endTimeMs) == 'number', 'numeric absolute end ms')
        assert(info.startTimeMs == {start}, 'explicit start must survive projection')
        assert(info.endTimeMs == {end}, 'explicit end must survive projection')
        "#
    ))
    .expect("public getter projects exact interval");
}

#[test]
fn default_interval_is_nonnil_zero_and_accessible_to_tainted_zeroarg_caller() {
    let env = ping_env();
    assert_interval(&env, 0, 0);
    env.exec(
        r#"
        local function addonRead()
            assert(debug.getstacktaint() == 'PingCooldownDraft', 'actually insecure caller')
            assert(not issecure(), 'insecure context')
            assert(select('#', C_Ping.GetCooldownInfo()) == 1)
            local info = C_Ping.GetCooldownInfo()
            assert(type(info) == 'table')
            assert(type(info.startTimeMs) == 'number' and info.startTimeMs == 0)
            assert(type(info.endTimeMs) == 'number' and info.endTimeMs == 0)
            return true
        end
        debug.setobjecttaint(addonRead, 'PingCooldownDraft')
        assert(addonRead(), 'zeroarg getter must allow insecure access')
        "#,
    )
    .expect("tainted caller can read default interval without any secret argument");
}

#[test]
fn supplied_interval_is_returned_as_absolute_milliseconds() {
    let env = ping_env();
    supply_interval(&env, 12000, 15500);
    env.exec("GetTime = function() return 12 end")
        .expect("controlled consumer clock");
    assert_interval(&env, 12000, 15500);
}

#[test]
fn every_read_is_a_fresh_dto_and_mutation_does_not_change_state() {
    let env = ping_env();
    supply_interval(&env, 12000, 15500);
    env.exec(
        r#"
        local first = C_Ping.GetCooldownInfo()
        local second = C_Ping.GetCooldownInfo()
        assert(not rawequal(first, second), 'each read owns a fresh DTO')
        first.startTimeMs = 77
        first.endTimeMs = 88
        first.fixtureOnly = 'mutated'
        assert(second.startTimeMs == 12000 and second.endTimeMs == 15500)
        assert(second.fixtureOnly == nil)
        local third = C_Ping.GetCooldownInfo()
        assert(not rawequal(third, first) and not rawequal(third, second))
        assert(third.startTimeMs == 12000 and third.endTimeMs == 15500)
        assert(third.fixtureOnly == nil, 'caller edits cannot contaminate future reads')
        "#,
    )
    .expect("DTO identity and observable mutation independence");
    assert_interval(&env, 12000, 15500);
}

#[test]
fn live_state_replacement_and_explicit_reset_affect_the_next_read() {
    let env = ping_env();
    supply_interval(&env, 12000, 15500);
    env.exec("SavedPingInterval = C_Ping.GetCooldownInfo()")
        .expect("retain caller snapshot");
    supply_interval(&env, 22000, 24000);
    assert_interval(&env, 22000, 24000);
    env.state().borrow_mut().ping_burst.interval = PingCooldownInterval::default();
    assert_interval(&env, 0, 0);
    env.exec(
        "assert(SavedPingInterval.startTimeMs == 12000 and SavedPingInterval.endTimeMs == 15500)",
    )
    .expect("older DTO remains a snapshot after replacement and reset");
}

#[test]
fn simultaneous_environments_do_not_share_interval_or_reset_state() {
    let first = ping_env();
    let second = ping_env();
    supply_interval(&first, 12000, 15500);
    assert_interval(&second, 0, 0);
    supply_interval(&second, 22000, 24000);
    assert_interval(&first, 12000, 15500);
    assert_interval(&second, 22000, 24000);
    first.state().borrow_mut().ping_burst.interval = PingCooldownInterval::default();
    assert_interval(&first, 0, 0);
    assert_interval(&second, 22000, 24000);
}

#[test]
fn actual_vendor_onenter_uses_absolute_end_ms_and_requests_three_point_five_seconds() {
    let env = ping_env();
    // Require the active profile cache; no alternate source.
    let source_path = default_blizzard_ui_addons_path()
        .expect("active profile cache already synced")
        .join("Blizzard_PingUI/Blizzard_PingUI.lua");
    let source =
        std::fs::read_to_string(&source_path).expect("cached active-profile PingUI source");
    env.exec_named(&source, &source_path.to_string_lossy())
        .expect("load actual unmodified vendor mixin definitions, not XML/startup");
    env.exec(
        r#"
        -- Only external effects are intercepted. Both vendor methods and getter remain real.
        GetTime = function() return 12 end
        PingDraftCursors = {}
        PingDraftTimers = {}
        SetCursor = function(cursor)
            PingDraftCursors[#PingDraftCursors + 1] = cursor
        end
        C_Timer.NewTimer = function(delay, callback)
            local timer = {delay = delay, callback = callback, cancelled = false}
            function timer:Cancel() self.cancelled = true end
            PingDraftTimers[#PingDraftTimers + 1] = timer
            return timer
        end
        PingDraftListener = setmetatable({}, {__index = PingListenerFrameMixin})
        "#,
    )
    .expect("bounded clock/cursor/timer fixture");

    // (0,0) is inactive at this controlled clock despite being a nonnil DTO.
    env.exec(
        r#"
        PingDraftListener:OnEnter()
        assert(#PingDraftCursors == 1 and PingDraftCursors[1] == 'PING_CURSOR')
        assert(#PingDraftTimers == 0 and PingDraftListener.cooldownTimer == nil)
        "#,
    )
    .expect("actual vendor interprets default as inactive");

    supply_interval(&env, 12000, 15500);
    env.exec(
        r#"
        PingDraftListener:OnEnter()
        assert(#PingDraftCursors == 2 and PingDraftCursors[2] == 'PING_ERROR_CURSOR')
        assert(#PingDraftTimers == 1)
        assert(PingDraftTimers[1].delay == 3.5, '15500ms / 1000 - 12s')
        assert(PingDraftListener.cooldownInfo.startTimeMs == 12000)
        assert(PingDraftListener.cooldownInfo.endTimeMs == 15500)
        assert(PingDraftListener.cooldownTimer == PingDraftTimers[1])
        -- Direct delivery proves vendor closure outputs only, not real timer scheduling.
        PingDraftTimers[1].callback()
        assert(#PingDraftCursors == 3 and PingDraftCursors[3] == 'PING_CURSOR')
        assert(PingDraftListener.cooldownTimer == nil)
        "#,
    )
    .expect("actual vendor millisecond interpretation and callback outputs");
    assert_interval(&env, 12000, 15500);

    // Exact end boundary belongs to consumer interpretation, not getter auto-expiry.
    env.exec(
        r#"
        GetTime = function() return 15.5 end
        PingDraftListener:OnEnter()
        assert(#PingDraftCursors == 4 and PingDraftCursors[4] == 'PING_CURSOR')
        assert(#PingDraftTimers == 1, 'no new timer at end boundary')
        "#,
    )
    .expect("vendor treats nowMs == endTimeMs as inactive");
    assert_interval(&env, 12000, 15500);
}
