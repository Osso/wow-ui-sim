#![cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
//! Retail 12.1.5 C_PingSecure UI receiver selection, world-target misses and
//! radial-frame size (WM47 Q8/Q10, WM51-F4..F6). The PingManager consumer cases
//! are in ping_secure_sends.rs.

use wow_ui_sim::c_api::c_ping_secure::PingPendingTarget;
use wow_ui_sim::lua_api::WowLuaEnv;

/// API-level: receiver eligibility and draw order that PingAttributes.xml
/// frames alone cannot vary.
#[test]
fn get_target_ping_receiver_selects_the_topmost_eligible_frame() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local Ping = __secureenv.C_PingSecure
        local function Box(name, x, strata, level)
            local frame = CreateFrame('Frame', name)
            frame:SetSize(100, 100)
            frame:SetPoint('BOTTOMLEFT', WorldFrame, 'BOTTOMLEFT', x, 0)
            frame:SetFrameStrata(strata or 'MEDIUM')
            frame:SetFrameLevel(level or 1)
            return frame
        end
        local function At(x, y)
            assert(select('#', Ping.GetTargetPingReceiver(x, y)) == 1)
            local frame = Ping.GetTargetPingReceiver(x, y)
            return frame and frame:GetName() or 'none'
        end

        -- A non-top-level frame needs ping-receiver; any non-nil value counts and
        -- the name is matched case-insensitively.
        local plain = Box('Plain', 0)
        assert(At(50, 50) == 'none')
        plain:SetAttribute('ping-receiver', false)
        assert(At(50, 50) == 'Plain')
        plain:SetAttribute('ping-receiver', nil)
        assert(At(50, 50) == 'none')
        plain:SetAttribute('Ping-Receiver', 0)
        assert(At(50, 50) == 'Plain')
        -- It ignores pass-through and needs no mouse input.
        plain:SetAttribute('ping-top-level-pass-through', true)
        plain:EnableMouse(false)
        assert(At(50, 50) == 'Plain')
        -- Cursor Y counts up from the bottom; the hit rect applies its insets.
        assert(At(99, 99) == 'Plain' and At(50, 101) == 'none')
        plain:SetHitRectInsets(0, 0, 0, 20)
        assert(At(50, 10) == 'none' and At(50, 30) == 'Plain')
        plain:SetHitRectInsets(0, 0, 0, 0)
        -- Effective visibility and effective alpha.
        plain:Hide()
        assert(At(50, 50) == 'none')
        plain:Show()
        plain:SetAlpha(0)
        assert(At(50, 50) == 'none')
        plain:SetAlpha(1)
        local holder = Box('Holder', 200)
        local child = CreateFrame('Frame', 'Child', holder)
        child:SetAllPoints()
        child:SetAttribute('ping-receiver', true)
        assert(At(250, 50) == 'Child')
        holder:SetAlpha(0)
        assert(At(250, 50) == 'none')

        -- A top-level frame needs no attribute; set pass-through skips it and the
        -- walk continues.
        local under = Box('UnderTop', 400, 'LOW')
        under:SetAttribute('ping-receiver', true)
        local top = Box('Top', 400, 'HIGH')
        top:SetToplevel(true)
        assert(At(450, 50) == 'Top')
        top:SetAttribute('ping-top-level-pass-through', false)
        assert(At(450, 50) == 'UnderTop')

        -- Strata outranks level; equal keys keep the first frame in the list.
        Box('LowStrata', 600, 'LOW', 50):SetAttribute('ping-receiver', true)
        Box('HighStrata', 600, 'HIGH', 1):SetAttribute('ping-receiver', true)
        assert(At(650, 50) == 'HighStrata')
        Box('FirstTie', 800, 'MEDIUM', 5):SetAttribute('ping-receiver', true)
        local second = Box('SecondTie', 800, 'MEDIUM', 5)
        second:SetAttribute('ping-receiver', true)
        assert(At(850, 50) == 'FirstTie')
        second:SetFrameLevel(6)
        assert(At(850, 50) == 'SecondTie')

        local function addon()
            assert(select('#', Ping.GetTargetPingReceiver(50, 50)) == 0)
        end
        debug.setobjecttaint(addon, 'ReceiverFixture')
        addon()
        "#,
    )
    .expect("receiver selection follows eligibility and draw order");
}

/// API-level: PingManager discards the selection state of a world miss.
#[test]
fn world_target_selection_misses_and_keeps_the_pending_target() {
    let env = WowLuaEnv::new().unwrap();
    let pending = PingPendingTarget {
        target_guid: "Player-1-00000001".into(),
        position: [1.0, 2.0, 3.0],
        transport_guid: String::new(),
    };
    env.state().borrow_mut().ping_pending = pending.clone();
    env.exec(
        r#"
        local Ping = __secureenv.C_PingSecure
        assert(select('#', Ping.SetHitTestPingTarget(10, 20)) == 1)
        assert(Ping.SetHitTestPingTarget(10, 20) == Enum.PingSetTargetState.Failed)
        assert(Ping.SetHitTestPingTarget('10', '20', true) == Enum.PingSetTargetState.Failed)
        assert(not pcall(Ping.SetHitTestPingTarget, 10))
        local result = Ping.SetHitTestTargetAndSendPing()
        local count = 0
        for _ in pairs(result) do count = count + 1 end
        assert(count == 1 and result.result == Enum.PingResult.FailedGeneric)
        "#,
    )
    .expect("world selection misses");
    assert_eq!(env.state().borrow().ping_pending, pending);
}

/// API-level: WM51-F5/F6 map the native side (10/1024)/S x M back to exactly
/// 10 GetCursorPosition units for every screen size.
#[test]
fn radial_frame_is_ten_cursor_units_on_every_screen() {
    for (width, height) in [(1600.0, 1200.0), (2560.0, 1080.0)] {
        let env = WowLuaEnv::new().unwrap();
        {
            let mut state = env.state().borrow_mut();
            state.screen_width = width;
            state.screen_height = height;
        }
        env.exec(
            r#"
            local Ping = __secureenv.C_PingSecure
            Ping.SetPingRadialWheelCreatedCallback(function(frame) Radial = frame end)
            Ping.CreateFrame()
            local width, height = Radial:GetSize()
            assert(width == 10 and height == 10, width .. 'x' .. height)
            "#,
        )
        .unwrap_or_else(|error| panic!("{width}x{height}: {error:?}"));
    }
}
