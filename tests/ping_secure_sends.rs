#![cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
//! Retail 12.1.5 C_PingSecure sends, burst bookkeeping, pending hit-test
//! information and radial-frame placement (WM38, WM47;
//! PingManagerSecureDocumentation.lua:8-157).

use wow_ui_sim::c_api::c_ping_secure::PingPendingTarget;
use wow_ui_sim::lua_api::WowLuaEnv;

/// WM47 Q4: exactly 2^-22.
const POINT_THRESHOLD: f32 = 1.0 / 4_194_304.0;

fn supply_pending(env: &WowLuaEnv, target_guid: &str, position: [f32; 3]) {
    env.state().borrow_mut().ping_pending = PingPendingTarget {
        target_guid: target_guid.into(),
        position,
        transport_guid: "Transport-0-0-0-0-1-0".into(),
    };
}

fn pending(env: &WowLuaEnv) -> PingPendingTarget {
    env.state().borrow().ping_pending.clone()
}

/// Blizzard_PingManager.lua:248-268 sends through `SendPing` and shows each
/// failure with `DisplayError`, which EventImplementation.lua:167-168 routes to
/// UIErrorsFrame. `Step` records that message, or `ok` when there is none.
const CONSUMER_SETUP: &str = r#"
    Manager = __secureenv.PingManager
    assert(type(Manager) == 'table' and type(Manager.SendPing) == 'function')
    for _, name in ipairs({'PING_FAILED_GENERIC', 'PING_FAILED_SPAMMING',
            'PING_FAILED_OUT_OF_PING_AREA', 'PING_FAILED_UNSPECIFIED',
            'PING_FAILED_DISABLED_BY_SETTINGS', 'PING_FAILED_SQUELCHED',
            'PING_FAILED_DISABLED_BY_LEADER'}) do
        assert(type(_G[name]) == 'string' and _G[name] ~= '', name)
    end
    Messages, Outcomes = {}, {}
    hooksecurefunc(UIErrorsFrame, 'AddMessage', function(_, text)
        table.insert(Messages, text)
    end)
    function Step(label, uiTargetInfo)
        local before = #Messages
        Manager:SendPing(nil, uiTargetInfo)
        assert(#Messages <= before + 1)
        table.insert(Outcomes, label .. '=' .. (Messages[before + 1] or 'ok'))
    end
    function SelfPing(label)
        Step(label, {guid = UnitGUID('player')})
    end
    function Fresh(label)
        A_Admin.AdvanceTime(3)
        SelfPing(label)
    end
    function Record(label, action)
        local before = #Messages
        action()
        table.insert(Outcomes, label .. '=' .. (Messages[before + 1] or 'ok'))
    end
    function Expect(expected)
        local actual = table.concat(Outcomes, ';')
        Outcomes = {}
        assert(actual == table.concat(expected, ';'), actual)
    end
    A_Admin.SetManualClock(10)
"#;

#[cfg(feature = "retail-12-1-5")]
prefork_full_ui_case! {
fn ping_manager_reports_send_result_codes(env: &WowLuaEnv) {
    env.exec(CONSUMER_SETUP).expect("PingManager consumer setup");
    // WM47 Q5-Q7 and WM38: shared eligibility precedes these checks.
    env.exec(
        r#"
        Step('empty-guid', {guid = ''})
        Step('unknown-guid', {guid = 'Creature-0-0-0-0-12345-0000000000'})
        Step('item-zero', {itemID = 0})
        Step('spell-zero', {spellID = 0})
        Step('category-zero', {spellCategoryID = 0})
        Step('category-unsupported', {spellCategoryID = 5})
        SelfPing('self')
        Expect({
            'empty-guid=' .. PING_FAILED_UNSPECIFIED,
            'unknown-guid=' .. PING_FAILED_OUT_OF_PING_AREA,
            'item-zero=' .. PING_FAILED_UNSPECIFIED,
            'spell-zero=' .. PING_FAILED_UNSPECIFIED,
            'category-zero=' .. PING_FAILED_UNSPECIFIED,
            'category-unsupported=' .. PING_FAILED_GENERIC,
            'self=ok',
        })
        "#,
    )
    .expect("send result codes reach UIErrorsFrame through PingManager");
}
}

#[cfg(feature = "retail-12-1-5")]
prefork_full_ui_case! {
fn ping_manager_spam_cooldown_follows_the_burst_clock(env: &WowLuaEnv) {
    env.exec(CONSUMER_SETUP).expect("PingManager consumer setup");
    // WM47 Q1/Q2: the third send in the window crosses limit 2 and is still
    // submitted; the cooldown refuses until now reaches crossing + 3000 ms.
    // Expiry does not idle-reset, so the old window start then rolls the count.
    env.exec(
        r#"
        SelfPing('10.0')
        A_Admin.AdvanceTime(0.1) SelfPing('10.1')
        A_Admin.AdvanceTime(0.1) SelfPing('10.2')
        A_Admin.AdvanceTime(0.1) SelfPing('10.3')
        A_Admin.AdvanceTime(2.5) SelfPing('12.8')
        A_Admin.AdvanceTime(0.399) SelfPing('13.199')
        A_Admin.AdvanceTime(0.001) SelfPing('13.2')
        A_Admin.AdvanceTime(0.1) SelfPing('13.3')
        A_Admin.AdvanceTime(0.1) SelfPing('13.4')
        A_Admin.AdvanceTime(0.1) SelfPing('13.5')
        Expect({
            '10.0=ok', '10.1=ok', '10.2=ok',
            '10.3=' .. PING_FAILED_SPAMMING,
            '12.8=' .. PING_FAILED_SPAMMING,
            '13.199=' .. PING_FAILED_SPAMMING,
            '13.2=ok', '13.3=ok', '13.4=ok', '13.5=ok',
        })
        "#,
    )
    .expect("spam refusal follows the cooldown deadline");
}
}

#[cfg(feature = "retail-12-1-5")]
prefork_full_ui_case! {
fn ping_manager_idle_reset_counts_from_the_last_send(env: &WowLuaEnv) {
    env.exec(CONSUMER_SETUP).expect("PingManager consumer setup");
    // WM47 Q2: the idle reset needs 3000 ms since the last submission, not
    // since the window start; it then restarts the window at the current time.
    env.exec(
        r#"
        SelfPing('10.0')
        A_Admin.AdvanceTime(2.9) SelfPing('12.9')
        A_Admin.AdvanceTime(0.2) SelfPing('13.1')
        A_Admin.AdvanceTime(0.1) SelfPing('13.2')
        A_Admin.AdvanceTime(0.1) SelfPing('13.3')
        A_Admin.AdvanceTime(0.1) SelfPing('13.4')
        A_Admin.AdvanceTime(3.0) SelfPing('16.4')
        A_Admin.AdvanceTime(0.1) SelfPing('16.5')
        A_Admin.AdvanceTime(0.1) SelfPing('16.6')
        A_Admin.AdvanceTime(0.1) SelfPing('16.7')
        Expect({
            '10.0=ok', '12.9=ok', '13.1=ok', '13.2=ok', '13.3=ok', '13.4=ok',
            '16.4=ok', '16.5=ok', '16.6=ok',
            '16.7=' .. PING_FAILED_SPAMMING,
        })
        "#,
    )
    .expect("idle reset is measured from the last send");
}
}

#[cfg(feature = "retail-12-1-5")]
prefork_full_ui_case! {
fn ping_manager_hit_test_send_consumes_pending_target(env: &WowLuaEnv) {
    env.exec(CONSUMER_SETUP).expect("PingManager consumer setup");
    // WM38/WM47 Q4: a pending GUID takes priority over the position; the send
    // consumes the pending information; with neither, the result is 1.
    supply_pending(&env, "Player-1-00000001", [400.0, 0.0, 0.0]);
    env.exec("Step('pending-guid', {}) Step('consumed', {})").unwrap();
    assert_eq!(pending(env), PingPendingTarget::default());

    // Blizzard_PingManager.lua:274-276 cancels through ClearHitTestPingInfo.
    supply_pending(&env, "Player-1-00000001", [0.0; 3]);
    env.exec("Manager:CancelPendingPing() Step('cancelled', {})").unwrap();
    assert_eq!(pending(env), PingPendingTarget::default());

    // During a cooldown, only an attempted point send reaches eligibility (2);
    // a position below the threshold attempts nothing (1).
    env.exec("SelfPing('burst-1') SelfPing('burst-2') SelfPing('burst-3')").unwrap();
    let below = POINT_THRESHOLD / 2.0;
    supply_pending(&env, "", [below, -below, below]);
    env.exec("Step('below-threshold', {})").unwrap();
    assert_eq!(pending(env), PingPendingTarget::default());
    supply_pending(&env, "", [0.0, -POINT_THRESHOLD, 0.0]);
    env.exec("Step('at-threshold', {})").unwrap();
    assert_eq!(pending(env), PingPendingTarget::default());
    env.exec(
        r#"
        Expect({
            'pending-guid=ok',
            'consumed=' .. PING_FAILED_GENERIC,
            'cancelled=' .. PING_FAILED_GENERIC,
            'burst-1=ok', 'burst-2=ok', 'burst-3=' .. PING_FAILED_SPAMMING,
            'below-threshold=' .. PING_FAILED_GENERIC,
            'at-threshold=' .. PING_FAILED_SPAMMING,
        })
        "#,
    )
    .expect("hit-test sends consume the pending target");
}
}

#[cfg(feature = "retail-12-1-5")]
prefork_full_ui_case! {
fn radial_parent_is_anchored_to_world_frame(env: &WowLuaEnv) {
    // WM47 Q10: Blizzard_PingUI.lua:44-48 receives a parentless frame whose
    // CENTER is anchored to WorldFrame's BOTTOMLEFT, at MEDIUM strata, level 0.
    env.exec(
        r#"
        local parent = PingFrame.radialParent
        assert(parent:GetParent() == nil)
        assert(parent:GetNumPoints() == 1)
        local point, relativeTo, relativePoint, x, y = parent:GetPoint(1)
        assert(point == 'CENTER' and relativeTo == WorldFrame, tostring(point))
        assert(relativePoint == 'BOTTOMLEFT' and x == 0 and y == 0)
        assert(parent:GetFrameStrata() == 'MEDIUM' and parent:GetFrameLevel() == 0)
        "#,
    )
    .expect("the radial parent follows the native placement");
}
}

/// API-level: PingManager discards `type` unless it shows a ping spot, which
/// needs supplied texture kits.
#[test]
fn self_unit_ping_returns_contextual_or_supplied_type() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        A_Admin.SetManualClock(10)
        local Ping = __secureenv.C_PingSecure
        local function fields(result)
            local count = 0
            for _ in pairs(result) do count = count + 1 end
            return count, result.result, result.type
        end
        local guid = UnitGUID('player')
        -- WM47 Q3: a nil type selects contextual 4 only when the target can
        -- attack the player with reaction at most 1; self selects 5.
        local count, result, kind = fields(Ping.SendUnitPing(guid))
        assert(count == 2 and result == 0 and kind == 5)
        A_Admin.AdvanceTime(5)
        -- WM38: the supplied type narrows to a byte; 256 becomes 0.
        count, result, kind = fields(Ping.SendUnitPing(guid, Enum.PingSubjectType.Assist, true))
        assert(count == 2 and result == 0 and kind == 2)
        A_Admin.AdvanceTime(5)
        count, result, kind = fields(Ping.SendUnitPing(guid, 256))
        assert(count == 2 and result == 0 and kind == 0)
        -- An absent type leaves no field.
        count, result, kind = fields(Ping.SendUnitPing(''))
        assert(count == 1 and result == 7 and kind == nil)
        "#,
    )
    .expect("self unit pings publish result and type");
}

/// API-level: tainted callers are refused before the pending state changes.
#[test]
fn clear_hit_test_ping_info_zeroes_the_pending_target() {
    let env = WowLuaEnv::new().unwrap();
    supply_pending(&env, "Player-1-00000001", [1.0, 2.0, 3.0]);
    env.exec(
        r#"
        local function addon()
            assert(select('#', __secureenv.C_PingSecure.ClearHitTestPingInfo()) == 0)
            assert(select('#', __secureenv.C_PingSecure.SendUnitPing(UnitGUID('player'))) == 0)
        end
        debug.setobjecttaint(addon, 'ClearFixture')
        addon()
        "#,
    )
    .unwrap();
    assert_eq!(pending(&env).position, [1.0, 2.0, 3.0]);
    let returned: i32 = env
        .eval("return select('#', __secureenv.C_PingSecure.ClearHitTestPingInfo())")
        .unwrap();
    assert_eq!(returned, 0);
    assert_eq!(pending(&env), PingPendingTarget::default());
}

#[cfg(feature = "retail-12-1-5")]
prefork_full_ui_case! {
fn ping_listener_receives_cooldown_info_before_the_crossing_send_returns(env: &WowLuaEnv) {
    env.exec(CONSUMER_SETUP).expect("PingManager consumer setup");
    // WM51-F1: the crossing stores {now, now + 3000} where C_Ping.GetCooldownInfo
    // reads it, then synchronously calls Blizzard_PingUI.lua:77-81 before the send
    // returns; expiry clears the pair.
    env.exec(
        r#"
        Seen = {}
        hooksecurefunc(PingListenerFrame, 'PingCooldownStarted', function(_, info, extra)
            local stored = C_Ping.GetCooldownInfo()
            table.insert(Seen, table.concat({info.startTimeMs, info.endTimeMs,
                stored.startTimeMs, stored.endTimeMs, #Outcomes, tostring(extra)}, ','))
        end)
        SelfPing('10.0')
        A_Admin.AdvanceTime(0.1) SelfPing('10.1')
        A_Admin.AdvanceTime(0.1) SelfPing('10.2')
        assert(table.concat(Seen, ';') == '10200,13200,10200,13200,2,nil', table.concat(Seen, ';'))
        assert(PingListenerFrame.cooldownInfo.endTimeMs == 13200)
        A_Admin.AdvanceTime(3) SelfPing('13.2')
        local info = C_Ping.GetCooldownInfo()
        assert(info.startTimeMs == 0 and info.endTimeMs == 0)
        assert(#Seen == 1)
        Expect({'10.0=ok', '10.1=ok', '10.2=ok', '13.2=ok'})
        "#,
    )
    .expect("the cooldown-started callback receives the stored pair");
}
}

#[cfg(feature = "retail-12-1-5")]
prefork_full_ui_case! {
fn ping_manager_reports_eligibility_gates_in_order(env: &WowLuaEnv) {
    env.exec(CONSUMER_SETUP).expect("PingManager consumer setup");
    // WM51-F2: enablePings (4) precedes the cooldown (2); squelch (6) and the
    // leader restriction (3) apply only in a group.
    env.exec(
        r#"
        SetCVar('enablePings', '0') Fresh('settings-off')
        SetCVar('enablePings', '1') Fresh('on-1') SelfPing('on-2') SelfPing('on-3')
        SelfPing('cooldown')
        SetCVar('enablePings', '0') SelfPing('settings-off-in-cooldown')
        SetCVar('enablePings', '1')
        "#,
    )
    .unwrap();
    // The full-UI fixture starts in a party; leave it to send solo first.
    env.state().borrow_mut().social_restrictions.is_squelched = true;
    env.exec(
        "A_Admin.SetPartySize(0) Fresh('solo-squelched') A_Admin.SetPartySize(2) \
         Fresh('group-squelched')",
    )
    .unwrap();
    env.state().borrow_mut().social_restrictions.is_squelched = false;
    env.exec(
        r#"
        C_PartyInfo.SetRestrictPings(Enum.RestrictPingsTo.Lead)
        Fresh('lead-as-leader')
        A_Admin.SetPartyLeader(1) Fresh('lead-as-member')
        C_PartyInfo.SetRestrictPings(Enum.RestrictPingsTo.Assist)
        Fresh('assist-as-member')
        "#,
    )
    .unwrap();
    env.state().borrow_mut().everyone_assistant = true;
    env.exec("Fresh('assist-as-assistant')").unwrap();
    env.state().borrow_mut().everyone_assistant = false;
    env.exec(
        r#"
        C_PartyInfo.SetRestrictPings(Enum.RestrictPingsTo.TankHealer)
        assert(UnitSetRole('player', 'DAMAGER')) Fresh('damager')
        assert(UnitSetRole('player', 'TANK')) Fresh('tank')
        assert(UnitSetRole('player', 'NONE')) Fresh('unassigned')
        C_PartyInfo.SetRestrictPings(Enum.RestrictPingsTo.Lead)
        A_Admin.SetPartySize(0) Fresh('solo-lead')
        Expect({
            'settings-off=' .. PING_FAILED_DISABLED_BY_SETTINGS,
            'on-1=ok', 'on-2=ok', 'on-3=ok',
            'cooldown=' .. PING_FAILED_SPAMMING,
            'settings-off-in-cooldown=' .. PING_FAILED_DISABLED_BY_SETTINGS,
            'solo-squelched=ok',
            'group-squelched=' .. PING_FAILED_SQUELCHED,
            'lead-as-leader=ok',
            'lead-as-member=' .. PING_FAILED_DISABLED_BY_LEADER,
            'assist-as-member=' .. PING_FAILED_DISABLED_BY_LEADER,
            'assist-as-assistant=ok',
            'damager=' .. PING_FAILED_DISABLED_BY_LEADER,
            'tank=ok', 'unassigned=ok', 'solo-lead=ok',
        })
        "#,
    )
    .expect("eligibility gates report in native order");
}
}

#[cfg(feature = "retail-12-1-5")]
prefork_full_ui_case! {
fn ping_manager_world_targets_miss_and_keep_the_pending_target(env: &WowLuaEnv) {
    env.exec(CONSUMER_SETUP).expect("PingManager consumer setup");
    // WM51-F4: the simulator has no world geometry, so every world query misses.
    // Selection fails before storing anything; the immediate contextual send
    // reports result 1 without eligibility, even during a cooldown.
    supply_pending(&env, "Player-1-00000001", [400.0, 0.0, 0.0]);
    env.exec(
        r#"
        assert(__secureenv.C_PingSecure.GetTargetPingReceiver(2, 2) == nil)
        local target = Manager:DeterminePingTarget(2, 2)
        assert(target.targetState == Enum.PingSetTargetState.Failed and not target.hasUITarget)
        SelfPing('burst-1') SelfPing('burst-2') SelfPing('burst-3') SelfPing('burst-4')
        Record('contextual', function() Manager:SendContextualWorldPing() end)
        Expect({
            'burst-1=ok', 'burst-2=ok', 'burst-3=ok',
            'burst-4=' .. PING_FAILED_SPAMMING,
            'contextual=' .. PING_FAILED_GENERIC,
        })
        "#,
    )
    .expect("world targeting misses without consuming the pending target");
    assert_eq!(pending(env).target_guid, "Player-1-00000001");
    assert_eq!(pending(env).position, [400.0, 0.0, 0.0]);
}
}

#[cfg(feature = "retail-12-1-5")]
prefork_full_ui_case! {
fn ping_manager_targets_pingable_ui_frames(env: &WowLuaEnv) {
    env.exec(CONSUMER_SETUP).expect("PingManager consumer setup");
    // WM47 Q8/WM51-F5: PingAttributes.xml templates and PingableType.lua mixins
    // drive Blizzard_PingManager.lua:110-185 through GetTargetPingReceiver.
    env.exec(
        r#"
        local function Place(frame, strata)
            frame:SetFrameStrata(strata)
            frame:SetSize(80, 80)
            frame:SetPoint('BOTTOMLEFT', WorldFrame, 'BOTTOMLEFT', 300, 300)
        end
        local unitFrame = CreateFrame('Frame', 'PingUnitFixture', nil, 'PingableUnitFrameTemplate')
        unitFrame.unit = 'player'
        Place(unitFrame, 'DIALOG')
        local target = Manager:DeterminePingTarget(340, 340)
        assert(target.targetState == Enum.PingSetTargetState.Ok)
        assert(target.hasUITarget and target.allowRadialWheel)
        assert(target.uiTargetInfo.guid == UnitGUID('player'))
        Record('unit-frame', function() Manager:DeterminePingTargetAndSend(340, 340) end)

        local blocker = CreateFrame('Frame', 'PingBlockerFixture')
        blocker:SetToplevel(true)
        Place(blocker, 'FULLSCREEN_DIALOG')
        Record('blocked', function() Manager:DeterminePingTargetAndSend(340, 340) end)
        blocker:Hide()

        local passer = CreateFrame('Frame', 'PingPassFixture', nil,
            'PingTopLevelPassThroughAttributeTemplate')
        passer:SetToplevel(true)
        Place(passer, 'FULLSCREEN_DIALOG')
        Record('pass-through', function() Manager:DeterminePingTargetAndSend(340, 340) end)
        Expect({'unit-frame=ok', 'blocked=' .. PING_FAILED_GENERIC, 'pass-through=ok'})
        "#,
    )
    .expect("PingManager targets pingable UI frames");
}
}
