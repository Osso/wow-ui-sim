//! Tests for Minimap, UnitPositionFrame, and FogOfWarFrame widget methods.
use super::*;

#[cfg(not(feature = "retail-12-0-7"))]
#[test]
fn test_minimap_texture_setters_persist_asset_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        MinimapTextureStateFrame = CreateFrame("Minimap", "MinimapTextureStateFrame", UIParent)
        MinimapTextureStateFrame:SetBlipTexture("Interface\\Minimap\\ObjectIcons")
        MinimapTextureStateFrame:SetMaskTexture("Interface\\Minimap\\UI-Minimap-Background")
        MinimapTextureStateFrame:SetIconTexture("Interface\\Minimap\\MiniMap-TrackingBorder")
        MinimapTextureStateFrame:SetPOIArrowTexture("Interface\\Minimap\\POIIcons")
        MinimapTextureStateFrame:SetCorpsePOIArrowTexture("Interface\\Minimap\\POIIcons-Corpse")
        MinimapTextureStateFrame:SetStaticPOIArrowTexture("Interface\\Minimap\\POIIcons-Static")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let minimap_id = state
        .widgets
        .get_id_by_name("MinimapTextureStateFrame")
        .expect("minimap should exist");
    let minimap = state
        .widgets
        .get(minimap_id)
        .expect("minimap frame should be readable");

    assert_eq!(
        minimap.minimap_blip_texture.as_deref(),
        Some("Interface\\Minimap\\ObjectIcons")
    );
    assert_eq!(
        minimap.minimap_mask_texture.as_deref(),
        Some("Interface\\Minimap\\UI-Minimap-Background")
    );
    assert_eq!(
        minimap.minimap_icon_texture.as_deref(),
        Some("Interface\\Minimap\\MiniMap-TrackingBorder")
    );
    assert_eq!(
        minimap.minimap_poi_arrow_texture.as_deref(),
        Some("Interface\\Minimap\\POIIcons")
    );
    assert_eq!(
        minimap.minimap_corpse_poi_arrow_texture.as_deref(),
        Some("Interface\\Minimap\\POIIcons-Corpse")
    );
    assert_eq!(
        minimap.minimap_static_poi_arrow_texture.as_deref(),
        Some("Interface\\Minimap\\POIIcons-Static")
    );
}

#[test]
fn test_minimap_player_texture_and_defaults_follow_runtime_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"MinimapDefaultStateFrame = CreateFrame("Minimap", "MinimapDefaultStateFrame", UIParent)"#).unwrap();
    #[cfg(not(feature = "retail-12-0-7"))]
    env.exec(r#"MinimapDefaultStateFrame:SetPlayerTexture("Interface\\Minimap\\MinimapArrow")"#)
        .unwrap();
    #[cfg(feature = "retail-12-0-7")]
    env.exec("assert(MinimapDefaultStateFrame.SetPlayerTexture == nil)")
        .unwrap();
    env.exec(
        r#"
        MinimapDefaultStateFrame:SetZoom(4)
        MinimapDefaultStateFrame:PingLocation(0.25, 0.75)
    "#,
    )
    .unwrap();

    {
        let state = env.state().borrow();
        let minimap_id = state
            .widgets
            .get_id_by_name("MinimapDefaultStateFrame")
            .expect("minimap should exist");
        let minimap = state
            .widgets
            .get(minimap_id)
            .expect("minimap frame should be readable");

        #[cfg(not(feature = "retail-12-0-7"))]
        assert_eq!(
            minimap.minimap_player_texture.as_deref(),
            Some("Interface\\Minimap\\MinimapArrow")
        );
        #[cfg(feature = "retail-12-0-7")]
        assert_eq!(minimap.minimap_player_texture, None);
        assert_eq!(minimap.minimap_ping_position, Some((0.25, 0.75)));
    }

    let zoom_before_reset: i32 = env
        .eval("return MinimapDefaultStateFrame:GetZoom()")
        .unwrap();
    assert_eq!(zoom_before_reset, 4);

    env.exec("MinimapDefaultStateFrame:SetToDefaults()")
        .unwrap();

    let zoom_after_reset: i32 = env
        .eval("return MinimapDefaultStateFrame:GetZoom()")
        .unwrap();
    assert_eq!(zoom_after_reset, 0);

    {
        let state = env.state().borrow();
        let minimap_id = state
            .widgets
            .get_id_by_name("MinimapDefaultStateFrame")
            .expect("minimap should exist");
        let minimap = state
            .widgets
            .get(minimap_id)
            .expect("minimap frame should be readable");

        assert_eq!(minimap.minimap_player_texture, None);
        assert_eq!(minimap.minimap_ping_position, None);
    }
}

#[test]
fn test_minimap_blob_setters_persist_blob_style_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        MinimapBlobStateFrame = CreateFrame("Minimap", "MinimapBlobStateFrame", UIParent)

        MinimapBlobStateFrame:SetQuestBlobInsideTexture("Interface\\Minimap\\Quest-Inside")
        MinimapBlobStateFrame:SetQuestBlobInsideAlpha(0.25)
        MinimapBlobStateFrame:SetQuestBlobOutsideTexture("Interface\\Minimap\\Quest-Outside")
        MinimapBlobStateFrame:SetQuestBlobOutsideAlpha(0.5)
        MinimapBlobStateFrame:SetQuestBlobRingTexture("Interface\\Minimap\\Quest-Ring")
        MinimapBlobStateFrame:SetQuestBlobRingAlpha(0.75)
        MinimapBlobStateFrame:SetQuestBlobRingScalar(1.25)

        MinimapBlobStateFrame:SetTaskBlobInsideTexture("Interface\\Minimap\\Task-Inside")
        MinimapBlobStateFrame:SetTaskBlobInsideAlpha(0.3)
        MinimapBlobStateFrame:SetTaskBlobOutsideTexture("Interface\\Minimap\\Task-Outside")
        MinimapBlobStateFrame:SetTaskBlobOutsideAlpha(0.6)
        MinimapBlobStateFrame:SetTaskBlobRingTexture("Interface\\Minimap\\Task-Ring")
        MinimapBlobStateFrame:SetTaskBlobRingAlpha(0.9)
        MinimapBlobStateFrame:SetTaskBlobRingScalar(1.5)

        MinimapBlobStateFrame:SetArchBlobInsideTexture("Interface\\Minimap\\Arch-Inside")
        MinimapBlobStateFrame:SetArchBlobInsideAlpha(0.4)
        MinimapBlobStateFrame:SetArchBlobOutsideTexture("Interface\\Minimap\\Arch-Outside")
        MinimapBlobStateFrame:SetArchBlobOutsideAlpha(0.7)
        MinimapBlobStateFrame:SetArchBlobRingTexture("Interface\\Minimap\\Arch-Ring")
        MinimapBlobStateFrame:SetArchBlobRingAlpha(1.0)
        MinimapBlobStateFrame:SetArchBlobRingScalar(1.75)
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let minimap_id = state
        .widgets
        .get_id_by_name("MinimapBlobStateFrame")
        .expect("minimap should exist");
    let minimap = state
        .widgets
        .get(minimap_id)
        .expect("minimap frame should be readable");

    assert_eq!(
        minimap.quest_blob_inside.texture.as_deref(),
        Some("Interface\\Minimap\\Quest-Inside")
    );
    assert_eq!(minimap.quest_blob_inside.alpha, 0.25);
    assert_eq!(
        minimap.quest_blob_outside.texture.as_deref(),
        Some("Interface\\Minimap\\Quest-Outside")
    );
    assert_eq!(minimap.quest_blob_outside.alpha, 0.5);
    assert_eq!(
        minimap.quest_blob_ring.texture.as_deref(),
        Some("Interface\\Minimap\\Quest-Ring")
    );
    assert_eq!(minimap.quest_blob_ring.alpha, 0.75);
    assert_eq!(minimap.quest_blob_ring.scalar, 1.25);

    assert_eq!(
        minimap.task_blob_inside.texture.as_deref(),
        Some("Interface\\Minimap\\Task-Inside")
    );
    assert_eq!(minimap.task_blob_inside.alpha, 0.3);
    assert_eq!(
        minimap.task_blob_outside.texture.as_deref(),
        Some("Interface\\Minimap\\Task-Outside")
    );
    assert_eq!(minimap.task_blob_outside.alpha, 0.6);
    assert_eq!(
        minimap.task_blob_ring.texture.as_deref(),
        Some("Interface\\Minimap\\Task-Ring")
    );
    assert_eq!(minimap.task_blob_ring.alpha, 0.9);
    assert_eq!(minimap.task_blob_ring.scalar, 1.5);

    assert_eq!(
        minimap.arch_blob_inside.texture.as_deref(),
        Some("Interface\\Minimap\\Arch-Inside")
    );
    assert_eq!(minimap.arch_blob_inside.alpha, 0.4);
    assert_eq!(
        minimap.arch_blob_outside.texture.as_deref(),
        Some("Interface\\Minimap\\Arch-Outside")
    );
    assert_eq!(minimap.arch_blob_outside.alpha, 0.7);
    assert_eq!(
        minimap.arch_blob_ring.texture.as_deref(),
        Some("Interface\\Minimap\\Arch-Ring")
    );
    assert_eq!(minimap.arch_blob_ring.alpha, 1.0);
    assert_eq!(minimap.arch_blob_ring.scalar, 1.75);
}

#[test]
fn test_unit_position_frame_methods_persist_runtime_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        UnitPositionStateFrame = CreateFrame("UnitPositionFrame", "UnitPositionStateFrame", UIParent)
        UnitPositionStateFrame:SetUiMapID(2274)
        UnitPositionStateFrame:AddUnit("player", "Interface\\Buttons\\WHITE8X8", 24, 26, 0.1, 0.2, 0.3, 0.4, 7, true)
        UnitPositionStateFrame:AddUnit("party1", "Interface\\Buttons\\WHITE8X8", 18, 19, 0.5, 0.6, 0.7, 0.8, 3, false)
        UnitPositionStateFrame:SetUnitColor("player", 0.9, 0.8, 0.7, 0.6)
        UnitPositionStateFrame:SetPlayerPingTexture(Enum.PingTextureType.Center, "Interface\\Minimap\\UI-Minimap-Ping-Center", 32, 32)
        UnitPositionStateFrame:SetPlayerPingTexture(Enum.PingTextureType.Rotation, "Interface\\Minimap\\UI-Minimap-Ping-Rotate", 70, 70)
        UnitPositionStateFrame:SetPlayerPingScale(0.65)
        UnitPositionStateFrame:StartPlayerPing(2, 0.25)
        UnitPositionStateFrame:FinalizeUnits()
    "#,
    )
    .unwrap();

    let ping_scale: f64 = env
        .eval("return UnitPositionStateFrame:GetPlayerPingScale()")
        .unwrap();
    assert_eq!(ping_scale, 0.65);
    let ui_map_id: i32 = env
        .eval("return UnitPositionStateFrame:GetUiMapID()")
        .unwrap();
    assert_eq!(ui_map_id, 2274);

    {
        let state = env.state().borrow();
        let frame_id = state
            .widgets
            .get_id_by_name("UnitPositionStateFrame")
            .expect("unit position frame should exist");
        let unit_state = state
            .unit_position_frames
            .get(&frame_id)
            .expect("unit position state should exist");
        assert_eq!(unit_state.ui_map_id, Some(2274));
        assert_eq!(unit_state.units.len(), 2);

        let player = &unit_state.units[0];
        assert_eq!(player.unit, "player");
        assert_eq!(
            player.asset.as_deref(),
            Some("Interface\\Buttons\\WHITE8X8")
        );
        assert_eq!(player.width, Some(24.0));
        assert_eq!(player.height, Some(26.0));
        assert_eq!(player.color, Some((0.9, 0.8, 0.7, 0.6)));
        assert_eq!(player.sublevel, Some(7));
        assert_eq!(player.show_facing, Some(true));

        let party = &unit_state.units[1];
        assert_eq!(party.unit, "party1");
        assert_eq!(party.asset.as_deref(), Some("Interface\\Buttons\\WHITE8X8"));
        assert_eq!(party.width, Some(18.0));
        assert_eq!(party.height, Some(19.0));
        assert_eq!(party.color, Some((0.5, 0.6, 0.7, 0.8)));
        assert_eq!(party.sublevel, Some(3));
        assert_eq!(party.show_facing, Some(false));

        assert_eq!(
            unit_state.unit_colors.get("player"),
            Some(&(0.9, 0.8, 0.7, 0.6))
        );
        let center_ping = unit_state
            .player_ping_textures
            .get(&0)
            .expect("center ping texture should exist");
        assert_eq!(
            center_ping.asset.as_deref(),
            Some("Interface\\Minimap\\UI-Minimap-Ping-Center")
        );
        assert_eq!(center_ping.width, 32.0);
        assert_eq!(center_ping.height, 32.0);
        let rotation_ping = unit_state
            .player_ping_textures
            .get(&2)
            .expect("rotation ping texture should exist");
        assert_eq!(
            rotation_ping.asset.as_deref(),
            Some("Interface\\Minimap\\UI-Minimap-Ping-Rotate")
        );
        assert_eq!(rotation_ping.width, 70.0);
        assert_eq!(rotation_ping.height, 70.0);
        assert_eq!(unit_state.player_ping_scale, 0.65);
        assert!(unit_state.player_ping_active);
        assert_eq!(unit_state.player_ping_duration, Some(2.0));
        assert_eq!(unit_state.player_ping_fade_duration, Some(0.25));
        assert!(unit_state.is_finalized);
    }

    {
        let frame_id = {
            let state = env.state().borrow();
            state
                .widgets
                .get_id_by_name("UnitPositionStateFrame")
                .expect("unit position frame should exist")
        };
        let mut state = env.state().borrow_mut();
        let unit_state = state
            .unit_position_frames
            .get_mut(&frame_id)
            .expect("unit position state should exist");
        unit_state.mouse_over_units = vec!["party1".to_string(), "player".to_string()];
    }

    let hovered_units: (String, String) = env
        .eval(
            r#"
            local first, second = UnitPositionStateFrame:GetMouseOverUnits()
            return first, second
        "#,
        )
        .unwrap();
    assert_eq!(hovered_units.0, "party1");
    assert_eq!(hovered_units.1, "player");

    env.exec(
        r#"
        UnitPositionStateFrame:StopPlayerPing()
        UnitPositionStateFrame:ClearUnits()
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let frame_id = state
        .widgets
        .get_id_by_name("UnitPositionStateFrame")
        .expect("unit position frame should exist");
    let unit_state = state
        .unit_position_frames
        .get(&frame_id)
        .expect("unit position state should still exist");
    assert!(unit_state.units.is_empty());
    assert!(unit_state.unit_colors.is_empty());
    assert!(!unit_state.player_ping_active);
    assert!(!unit_state.is_finalized);
}

#[test]
fn test_fog_of_war_frame_get_ui_map_id_round_trips_setter_state() {
    let env = WowLuaEnv::new().unwrap();
    let ui_map_id: i32 = env
        .eval(
            r#"
            local fog = CreateFrame("FogOfWarFrame", "FogOfWarStateFrame", UIParent)
            fog:SetUiMapID(2274)
            return fog:GetUiMapID()
        "#,
        )
        .unwrap();
    assert_eq!(ui_map_id, 2274);
}

#[test]
fn test_fog_of_war_frame_atlas_and_mask_scalar_round_trip() {
    let env = WowLuaEnv::new().unwrap();
    let (background_atlas, mask_atlas, mask_scalar): (String, String, f64) = env
        .eval(
            r#"
            local fog = CreateFrame("FogOfWarFrame", "FogOfWarAtlasFrame", UIParent)
            fog:SetFogOfWarBackgroundAtlas("worldmap-wardisplay-background")
            fog:SetFogOfWarMaskAtlas("worldmap-wardisplay-mask")
            fog:SetMaskScalar(0.75)
            return fog:GetFogOfWarBackgroundAtlas(), fog:GetFogOfWarMaskAtlas(), fog:GetMaskScalar()
        "#,
        )
        .unwrap();
    assert_eq!(background_atlas, "worldmap-wardisplay-background");
    assert_eq!(mask_atlas, "worldmap-wardisplay-mask");
    assert!((mask_scalar - 0.75).abs() < f64::EPSILON);
}

#[test]
fn test_fog_of_war_frame_allows_clearing_optional_atlases() {
    let env = WowLuaEnv::new().unwrap();
    let (background_atlas, mask_atlas, mask_scalar): (Option<String>, Option<String>, f64) = env
        .eval(
            r#"
            local fog = CreateFrame("FogOfWarFrame", "FogOfWarClearedAtlasFrame", UIParent)
            fog:SetFogOfWarBackgroundAtlas("worldmap-wardisplay-background")
            fog:SetFogOfWarMaskAtlas("worldmap-wardisplay-mask")
            fog:SetMaskScalar(0.75)
            fog:SetFogOfWarBackgroundAtlas(nil)
            fog:SetFogOfWarMaskAtlas(nil)
            fog:SetMaskScalar(nil)
            return fog:GetFogOfWarBackgroundAtlas(), fog:GetFogOfWarMaskAtlas(), fog:GetMaskScalar()
        "#,
        )
        .unwrap();
    assert_eq!(background_atlas, None);
    assert_eq!(mask_atlas, None);
    assert!((mask_scalar - 1.0).abs() < f64::EPSILON);
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn test_minimap_mask_texture_remains_live_after_retired_texture_setters() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        MinimapTextureStateFrame = CreateFrame("Minimap", "MinimapTextureStateFrame", UIParent)
        MinimapTextureStateFrame:SetMaskTexture("Interface\\Minimap\\UI-Minimap-Background")
        for _, method in ipairs({"SetBlipTexture", "SetIconTexture", "SetPOIArrowTexture",
            "SetCorpsePOIArrowTexture", "SetStaticPOIArrowTexture", "SetPlayerTexture"}) do
            assert(MinimapTextureStateFrame[method] == nil)
        end
    "#,
    )
    .unwrap();
    let state = env.state().borrow();
    let id = state
        .widgets
        .get_id_by_name("MinimapTextureStateFrame")
        .unwrap();
    let minimap = state.widgets.get(id).unwrap();
    assert_eq!(
        minimap.minimap_mask_texture.as_deref(),
        Some("Interface\\Minimap\\UI-Minimap-Background")
    );
}
