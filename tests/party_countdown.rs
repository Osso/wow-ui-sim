//! Tests-only Retail 12.0.5 countdown contract; lifecycle/security policies are inferred.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::wrap_host_secret_number;
use wow_ui_sim::lua_api::WowLuaEnv;

const OBSERVE_COUNTDOWN: &str = r#"
    countdownEvents = {}
    countdownObserver = CreateFrame('Frame')
    countdownObserver:RegisterEvent('START_PLAYER_COUNTDOWN')
    countdownObserver:RegisterEvent('CANCEL_PLAYER_COUNTDOWN')
    countdownObserver:SetScript('OnEvent', function(_, event, ...)
        table.insert(countdownEvents, {
            event = event, count = select('#', ...), payload = {...},
            duringCall = countdownCallOpen,
            playerGUID = UnitGUID('player'), playerName = UnitName('player'),
        })
    end)
    local function pack(...)
        return {n = select('#', ...), ...}
    end
    function requestCountdown(seconds, expectedEvent)
        local before = #countdownEvents
        countdownCallOpen = true
        local result = pack(C_PartyInfo.DoCountdown(seconds))
        countdownCallOpen = false
        assert(result.n == 1, 'countdown returns exactly one value')
        assert(type(result[1]) == 'boolean' and result[1] == true,
            'accepted request returns success true')
        if expectedEvent == nil then
            assert(#countdownEvents == before, 'idle cancel must not emit an event')
            return
        end
        assert(#countdownEvents == before + 1, 'exactly one synchronous event per change')
        local observed = countdownEvents[before + 1]
        assert(observed.duringCall == true, 'event must run before DoCountdown returns')
        assert(observed.event == expectedEvent, 'wrong countdown event ordering')
        assert(type(observed.playerGUID) == 'string' and #observed.playerGUID > 0)
        assert(observed.payload[1] == observed.playerGUID, 'initiator is modeled player GUID')
        assert(type(observed.playerName) == 'string' and #observed.playerName > 0)
        if expectedEvent == 'START_PLAYER_COUNTDOWN' then
            assert(observed.count == 5, 'start payload arity')
            assert(type(observed.payload[2]) == 'number' and observed.payload[2] == seconds,
                'new request publishes full remaining seconds')
            assert(type(observed.payload[3]) == 'number' and observed.payload[3] == seconds,
                'new request publishes total seconds')
            assert(observed.payload[4] == false, 'local start must not fabricate group chat')
            assert(observed.payload[5] == observed.playerName, 'start uses modeled name')
        else
            assert(observed.count == 3, 'cancel payload arity')
            assert(observed.payload[2] == false, 'local cancel must not fabricate group chat')
            assert(observed.payload[3] == observed.playerName, 'cancel uses modeled name')
        end
    end
    function rejectCountdown(...)
        local before = #countdownEvents
        countdownCallOpen = true
        local ok, err = pcall(C_PartyInfo.DoCountdown, ...)
        countdownCallOpen = false
        assert(not ok, 'invalid or restricted request must explicitly reject')
        assert(type(err) == 'string' and #err > 0, 'rejection must report an error')
        assert(#countdownEvents == before, 'rejection must not publish countdown events')
        return err
    end
"#;

fn create_observed_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create countdown environment");
    env.exec(OBSERVE_COUNTDOWN)
        .expect("register real countdown event observer");
    env
}

fn install_secret_seconds(env: &WowLuaEnv, name: &str, seconds: f64) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let secret = wrap_host_secret_number(lua.state_mut(), seconds);
    lua.state_mut().push(secret);
    let inserted = lua.set_global_val(name, secret);
    lua.state_mut().pop();
    inserted.expect("install actual secret countdown duration");
}

#[test]
fn positive_request_returns_one_success_and_synchronous_modeled_identity_payload() {
    let env = create_observed_env();
    env.exec(
        r#"
        A_Admin.SetPlayerName('CountdownInitiator')
        requestCountdown(10, 'START_PLAYER_COUNTDOWN')
        assert(countdownEvents[1].payload[5] == 'CountdownInitiator')
        assert(#countdownEvents == 1)
        "#,
    )
    .expect("public start return, payload and before-return delivery");
}

#[test]
fn finite_fractional_seconds_and_inclusive_maximum_are_accepted() {
    let env = create_observed_env();
    env.exec(
        r#"
        local maximum = Constants.PartyCountdownConstants.MaxCountdownSeconds
        assert(maximum == 3600, 'cached countdown maximum')
        for _, seconds in ipairs({0.25, 1, 3599.5, maximum}) do
            requestCountdown(seconds, 'START_PLAYER_COUNTDOWN')
            requestCountdown(0, 'CANCEL_PLAYER_COUNTDOWN')
        end
        assert(#countdownEvents == 8)
        "#,
    )
    .expect("ordinary finite duration range is inclusive and does not round fractions");
}

#[test]
fn zero_on_fresh_or_cancelled_request_succeeds_without_an_event() {
    let env = create_observed_env();
    env.exec(
        r#"
        requestCountdown(0)
        requestCountdown(0)
        assert(#countdownEvents == 0)
        requestCountdown(6, 'START_PLAYER_COUNTDOWN')
        requestCountdown(0, 'CANCEL_PLAYER_COUNTDOWN')
        requestCountdown(0)
        assert(#countdownEvents == 2)
        "#,
    )
    .expect("idle cancellation is a successful no-op, including after cancellation");
}

#[test]
fn replacement_uses_latest_duration_in_both_call_orders_then_cancels_once() {
    for (first, replacement) in [(7, 23), (23, 7)] {
        let env = create_observed_env();
        env.exec(&format!(
            r#"
            requestCountdown({first}, 'START_PLAYER_COUNTDOWN')
            requestCountdown({replacement}, 'START_PLAYER_COUNTDOWN')
            requestCountdown(0, 'CANCEL_PLAYER_COUNTDOWN')
            requestCountdown(0)
            assert(#countdownEvents == 3, 'replacement does not add a cancel event')
            assert(countdownEvents[1].payload[3] == {first})
            assert(countdownEvents[2].payload[3] == {replacement})
            "#,
        ))
        .expect("positive replacement publishes latest request without duplicate cancellation");
    }
}

#[test]
fn start_cancel_start_and_cancel_start_cancel_both_publish_in_request_order() {
    for cancel_first in [false, true] {
        let env = create_observed_env();
        env.exec(&format!(
            r#"
            if {cancel_first} then requestCountdown(0) end
            requestCountdown(9, 'START_PLAYER_COUNTDOWN')
            requestCountdown(0, 'CANCEL_PLAYER_COUNTDOWN')
            requestCountdown(14, 'START_PLAYER_COUNTDOWN')
            requestCountdown(0, 'CANCEL_PLAYER_COUNTDOWN')
            assert(#countdownEvents == 4)
            "#,
        ))
        .expect("start and cancel ordering is independent of an earlier idle cancellation");
    }
}

#[test]
fn malformed_durations_neither_emit_events_nor_corrupt_active_cancellation() {
    let env = create_observed_env();
    env.exec(
        r#"
        local function rejectFreshAndActive(...)
            rejectCountdown(...)
            requestCountdown(0)
            requestCountdown(19, 'START_PLAYER_COUNTDOWN')
            rejectCountdown(...)
            requestCountdown(0, 'CANCEL_PLAYER_COUNTDOWN')
            requestCountdown(0)
        end
        rejectFreshAndActive()
        rejectFreshAndActive(nil)
        for _, seconds in ipairs({-0.25, -1, 3600.25, math.huge, -math.huge,
                0/0, '10', '0', true, false, {}, function() end, countdownObserver}) do
            rejectFreshAndActive(seconds)
        end
        "#,
    )
    .expect("missing, nonnumeric, out-of-range and nonfinite durations reject atomically");
}

#[test]
fn secret_start_and_cancel_reject_in_secure_and_tainted_callers_without_corruption() {
    let env = create_observed_env();
    install_secret_seconds(&env, "SecretStartSeconds", 12.0);
    install_secret_seconds(&env, "SecretCancelSeconds", 0.0);
    env.exec(
        r#"
        collectgarbage('collect')
        local function rejectSecrets()
            for _, seconds in ipairs({SecretStartSeconds, SecretCancelSeconds}) do
                rejectCountdown(seconds)
                requestCountdown(0)
                requestCountdown(27, 'START_PLAYER_COUNTDOWN')
                rejectCountdown(seconds)
                assert(issecretvalue(seconds), 'rejection must not declassify seconds')
                requestCountdown(0, 'CANCEL_PLAYER_COUNTDOWN')
                requestCountdown(0)
            end
        end
        assert(issecure())
        rejectSecrets()
        assert(issecure(), 'secure caller remains secure')
        local function addon()
            assert(debug.getstacktaint() == 'PartyCountdownFixture')
            rejectSecrets()
            assert(debug.getstacktaint() == 'PartyCountdownFixture',
                'rejection must preserve caller taint')
        end
        debug.setobjecttaint(addon, 'PartyCountdownFixture')
        addon()
        assert(issecure(), 'tainted callback must not taint its caller')
        "#,
    )
    .expect("conservative secret rejection preserves lifecycle and caller taint");
}

#[test]
fn countdown_mutations_and_idle_cancellation_are_environment_isolated() {
    let first = create_observed_env();
    let second = create_observed_env();
    first
        .exec("A_Admin.SetPlayerName('FirstInitiator'); requestCountdown(11, 'START_PLAYER_COUNTDOWN')")
        .unwrap();
    second
        .exec("A_Admin.SetPlayerName('SecondInitiator'); requestCountdown(0); assert(#countdownEvents == 0)")
        .unwrap();
    second
        .exec("requestCountdown(21, 'START_PLAYER_COUNTDOWN')")
        .unwrap();
    first
        .exec("assert(#countdownEvents == 1); requestCountdown(31, 'START_PLAYER_COUNTDOWN')")
        .unwrap();
    second
        .exec("assert(#countdownEvents == 1); requestCountdown(0, 'CANCEL_PLAYER_COUNTDOWN')")
        .unwrap();
    first
        .exec("assert(#countdownEvents == 2); requestCountdown(0, 'CANCEL_PLAYER_COUNTDOWN'); requestCountdown(0)")
        .unwrap();
    second
        .exec("requestCountdown(0); assert(#countdownEvents == 2)")
        .unwrap();
}

#[test]
fn lockdown_blocks_fresh_start_and_idle_cancel_independently_of_combat() {
    for combat in [false, true] {
        let env = create_observed_env();
        {
            let mut state = env.state().borrow_mut();
            state.player.in_combat = combat;
            state.chat_messaging_lockdown = true;
        }
        env.exec(
            r#"
            for _, seconds in ipairs({13, 0}) do
                local err = rejectCountdown(seconds)
                assert(string.find(err, 'chat messaging lockdown', 1, true), 'shared guard error')
            end
            assert(#countdownEvents == 0)
            "#,
        )
        .expect("lockdown rejects both fresh operations before events");
        env.state().borrow_mut().chat_messaging_lockdown = false;
        env.exec("requestCountdown(0); requestCountdown(13, 'START_PLAYER_COUNTDOWN')")
            .expect("unlocked fresh lifecycle recovers on either combat axis");
        assert_eq!(env.state().borrow().player.in_combat, combat);
    }
}

#[test]
fn lockdown_combat_matrix_preserves_active_request_and_guards_before_validation() {
    for (combat, lockdown) in [(false, false), (false, true), (true, false), (true, true)] {
        let env = create_observed_env();
        env.state().borrow_mut().player.in_combat = combat;
        env.exec("requestCountdown(17, 'START_PLAYER_COUNTDOWN')")
            .expect("seed active request through public API");
        env.state().borrow_mut().chat_messaging_lockdown = lockdown;
        if lockdown {
            env.exec(
                r#"
                local function assertLocked(...)
                    local err = rejectCountdown(...)
                    assert(string.find(err, 'chat messaging lockdown', 1, true),
                        'shared restriction guard must precede argument validation')
                end
                assertLocked(29)
                assertLocked(0)
                assertLocked()
                assertLocked('0')
                assertLocked(-1)
                assert(#countdownEvents == 1, 'blocked replacement/cancel preserve active request')
                "#,
            )
            .expect("locked operations reject before effects or malformed validation");
            env.state().borrow_mut().chat_messaging_lockdown = false;
        } else {
            env.exec("requestCountdown(29, 'START_PLAYER_COUNTDOWN')")
                .expect("unlocked replacement is allowed even in combat");
        }
        env.exec("requestCountdown(0, 'CANCEL_PLAYER_COUNTDOWN'); requestCountdown(0)")
            .expect("active request survives rejection and cancels exactly once after unlock");
        assert_eq!(env.state().borrow().player.in_combat, combat);
    }
}
