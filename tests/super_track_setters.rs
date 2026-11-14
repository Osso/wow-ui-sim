//! Retail R88j.8: selected original consumers and API-only reader contracts.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn original_poi_button_click_reselects_the_supplied_map_pin() {
    with_blizzard_addon_closure(&["Blizzard_POIButton"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_POIButton"));
        {
            let mut sim = env.state().borrow_mut();
            sim.super_track.map_pin = Some((0, 4801));
            sim.super_track.map_pin_saved = (0, 4801);
            // Keep selection presence but clear the flag: the equal pair
            // skips validation and does not repair this mismatch.
        }
        env.exec(
            r#"
            local button = CreateFrame("Button", nil, UIParent, "POIButtonTemplate")
            assert(button.OnClick == POIButtonMixin.OnClick)
            button:SetAreaPOIID(4801)
            button:Click("LeftButton")
            assert(not C_SuperTrack.IsSuperTrackingMapPin())
            "#,
        )
        .unwrap();
        let sim = env.state().borrow();
        assert_eq!(sim.super_track.map_pin, Some((0, 4801)));
        assert!(
            sim.lua_errors
                .iter()
                .all(|error| !error.contains("POIButton.lua:504")),
            "original callback failed: {:?}",
            sim.lua_errors
        );
    });
}

#[test]
fn original_poi_button_click_clears_tracking_for_empty_vignette() {
    with_blizzard_addon_closure(&["Blizzard_POIButton"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_POIButton"));
        {
            let mut sim = env.state().borrow_mut();
            sim.super_track.vignette = Some("Vignette-fixture-1".into());
            sim.super_track.vignette_tracking = true;
        }
        env.exec(
            r#"
            assert(C_SuperTrack.IsSuperTrackingAnything())
            local button = CreateFrame("Button", nil, UIParent, "POIButtonTemplate")
            assert(button.OnClick == POIButtonMixin.OnClick)
            button:SetVignette("")
            button:Click("LeftButton")
            assert(not C_SuperTrack.IsSuperTrackingAnything())
            "#,
        )
        .unwrap();
        let state = env.state().borrow();
        assert_eq!(state.super_track.vignette.as_deref(), Some(""));
        assert!(!state.super_track.vignette_tracking);
    });
}

// API-level VERIFIED: no original consumer exercises malformed setter calls.
#[test]
fn super_track_setters_use_r88j_8_argument_details() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local api = C_SuperTrack
        local map = "Usage: C_SuperTrack.SetSuperTrackedMapPin(type, typeID)"
        local quest = "Usage: C_SuperTrack.SetSuperTrackedQuestID(questID)"
        local vignette = "Usage: C_SuperTrack.SetSuperTrackedVignette(vignetteGUID)"
        local range = "outside of expected range -2147483648 to 2147483647 - "
        local function failsWith(f, text, ...)
            local ok, err = pcall(f, ...)
            assert(not ok and tostring(err):find(text, 1, true), tostring(err))
        end
        failsWith(api.SetSuperTrackedMapPin, map, nil, 1)
        failsWith(api.SetSuperTrackedMapPin, map, 1, nil)
        failsWith(api.SetSuperTrackedMapPin, map, 5, 1)
        failsWith(api.SetSuperTrackedMapPin, "integer overflow attempting to store 2147483648.000000", 2147483648, 1)
        failsWith(api.SetSuperTrackedMapPin, range .. map, 1, false)
        failsWith(api.SetSuperTrackedMapPin, range .. map, 1, 2147483647.1)
        api.SetSuperTrackedMapPin(-0.9, "1.9", "extra")
        api.SetSuperTrackedMapPin(4.9, 1, "extra")
        api.SetSuperTrackedMapPin({}, 1, "extra")
        failsWith(api.SetSuperTrackedQuestID, quest)
        failsWith(api.SetSuperTrackedQuestID, range .. quest, 2147483647.1)
        failsWith(api.SetSuperTrackedQuestID, range .. quest, {})
        api.SetSuperTrackedQuestID("51.9", "extra")
        assert(api.GetSuperTrackedQuestID() == 51)
        failsWith(api.SetSuperTrackedVignette, vignette, nil)
        failsWith(api.SetSuperTrackedVignette, vignette, {})
        api.SetSuperTrackedVignette(1, "extra")
        "#,
    )
    .unwrap();
}
