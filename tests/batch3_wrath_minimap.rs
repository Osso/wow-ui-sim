//! Original Wrath minimap consumer; no copied or substituted handlers.
#![cfg(feature = "client-wrath")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn original_minimap() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    let path = dirs::cache_dir()
        .unwrap()
        .join("wow-ui-sim/blizzard-ui/wrath/FrameXML/Minimap.lua");
    let path = path.to_str().unwrap();
    let source = std::fs::read_to_string(path).expect("original Wrath Minimap.lua required");
    env.exec_named(&source, &format!("@{path}")).unwrap();
    env.exec(
        r#"
        Minimap:SetSize(200, 200)
        MinimapZoomIn = CreateFrame("Button", "MinimapZoomIn", UIParent)
        MinimapZoomOut = CreateFrame("Button", "MinimapZoomOut", UIParent)
        MinimapPing = CreateFrame("Frame", "MinimapPing", Minimap)
        MinimapPing:Hide()
        MinimapPing:SetScript("OnEvent", MinimapPing_OnEvent)
        -- Register the real consumer without its unrelated player-texture setup.
        MinimapPing:RegisterEvent("MINIMAP_PING")
        MinimapPing:RegisterEvent("MINIMAP_UPDATE_ZOOM")
        Batch3ZoomEvents, Batch3Pings = 0, {}
        local observer = CreateFrame("Frame", nil, UIParent)
        observer:RegisterEvent("MINIMAP_UPDATE_ZOOM")
        observer:RegisterEvent("MINIMAP_PING")
        observer:SetScript("OnEvent", function(self, event, ...)
            if event == "MINIMAP_UPDATE_ZOOM" then
                Batch3ZoomEvents = Batch3ZoomEvents + 1
            else
                table.insert(Batch3Pings, {n = select('#', ...), ...})
            end
        end)
    "#,
    )
    .unwrap();
    env
}

#[test]
fn m3_signed_to_unsigned_before_upper_clamp() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(Minimap:GetZoomLevels() == 6)
        Minimap:SetZoom(-1)
        assert(Minimap:GetZoom() == 5, "Wrath -1 converts to unsigned then clamps to 5")
        Minimap:SetZoom(-0.5)
        assert(Minimap:GetZoom() == 0, "truncate toward zero before unsigned conversion")
        Minimap:SetZoom(2.9)
        assert(Minimap:GetZoom() == 2)
        Minimap:SetZoom(99)
        assert(Minimap:GetZoom() == 5)
    "#,
    )
    .unwrap();
}

#[test]
fn m3_original_zoom_buttons_refresh_both_endpoints_without_set_event() {
    let env = original_minimap();
    env.exec(
        r#"
        Minimap:SetZoom(0)
        for i = 1, 5 do Minimap_ZoomInClick() end
        assert(Minimap:GetZoom() == 5)
        -- 12340 IsEnabled returns 1/0 (FrameXML/UIPanelTemplates.xml:68-96).
        assert(MinimapZoomIn:IsEnabled() == 0 and MinimapZoomOut:IsEnabled() == 1)
        for i = 1, 5 do Minimap_ZoomOutClick() end
        assert(Minimap:GetZoom() == 0)
        assert(MinimapZoomIn:IsEnabled() == 1 and MinimapZoomOut:IsEnabled() == 0)
        Minimap:SetZoom(0)
        Minimap:SetZoom(3)
        Minimap:SetZoom(3)
        assert(Batch3ZoomEvents == 0, "changed and unchanged SetZoom emit no event")
    "#,
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn m3_original_zoom_event_handler_reads_state() {
    let env = original_minimap();
    for zoom in [0, 5] {
        env.exec(&format!("Minimap:SetZoom({zoom})")).unwrap();
        // Consumer-only control: this does NOT prove native startup/switch production.
        env.fire_event("MINIMAP_UPDATE_ZOOM").unwrap();
        env.exec(&format!(
            r#"
            assert(MinimapZoomIn:IsEnabled() == (({zoom} ~= 5) and 1 or 0))
            assert(MinimapZoomOut:IsEnabled() == (({zoom} ~= 0) and 1 or 0))
        "#
        ))
        .unwrap();
    }
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn m2_local_center_ping_reaches_original_consumer_twice_without_group() {
    let env = original_minimap();
    env.exec(
        r#"
        assert(UnitExists("player"), "fixture requires a player")
        assert(GetNumPartyMembers() == 0 and GetNumRaidMembers() == 0, "solo fixture")
        Minimap:PingLocation(0, 0)
        Minimap:PingLocation(0, 0)
    "#,
    )
    .unwrap();
    // Bounded dispatch opportunity; no manual MINIMAP_PING injection.
    env.fire_on_update(0.0).unwrap();
    env.exec(
        r#"
        assert(#Batch3Pings == 2, "each solo PingLocation must publish locally")
        for _, ping in ipairs(Batch3Pings) do
            assert(ping.n == 3 and ping[1] == "player", "exact unit,x,y tuple")
            assert(ping[2] == 0 and ping[3] == 0, "center maps to normalized origin")
        end
        assert(MinimapPing:IsShown(), "original Minimap_SetPing shows center ping")
        assert(MinimapPing:GetAlpha() == 1)
        local point, relative, relativePoint, x, y = MinimapPing:GetPoint(1)
        assert(point == "CENTER" and relative == Minimap and relativePoint == "CENTER")
        assert(x == 0 and y == 0)
        local px, py = Minimap:GetPingPosition()
        assert(px == 0 and py == 0)
        -- Original consumer owns its timer; no claim about native Wrath fade duration.
        assert(MinimapPing.timer == MINIMAPPING_TIMER)
    "#,
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn m2_nonzero_view_input_still_publishes_local_unit_event() {
    let env = original_minimap();
    env.exec("assert(UnitExists('player')); Minimap:PingLocation(17,-23)")
        .unwrap();
    env.fire_on_update(0.0).unwrap();
    env.exec(
        r#"
        assert(#Batch3Pings==1,'every player-present ping publishes locally')
        assert(Batch3Pings[1][1]=='player')
        -- Nonzero x/y are intentionally unasserted: transform/normalization missing.
    "#,
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn m2_missing_pair_uses_player_center_not_one_supplied_coordinate() {
    let env = original_minimap();
    env.exec(
        r#"
        assert(UnitExists("player"))
        Minimap:PingLocation(17)
    "#,
    )
    .unwrap();
    env.fire_on_update(0.0).unwrap();
    env.exec(
        r#"
        assert(#Batch3Pings == 1, "incomplete coordinate pair still publishes locally")
        local ping = Batch3Pings[1]
        assert(ping.n == 3 and ping[1] == "player")
        assert(ping[2] == 0 and ping[3] == 0, "missing pair chooses player position")
        local x, y = Minimap:GetPingPosition()
        assert(x == 0 and y == 0)
        assert(MinimapPing:IsShown())
    "#,
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
