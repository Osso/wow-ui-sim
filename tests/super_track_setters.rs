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

// The source is inferred; the selected POIButton action and change/rejection
// effects use real host state and are independently specified by R88j.8.
#[test]
fn original_poi_button_click_accepts_a_supplied_area_row_and_keeps_it_on_miss() {
    use wow_ui_sim::c_api::c_area_poi_info::AreaPoiInfo;

    with_blizzard_addon_closure(&["Blizzard_POIButton"], &[], |env, _| {
        {
            let mut sim = env.state().borrow_mut();
            sim.area_pois.insert(
                4801,
                AreaPoiInfo {
                    area_poi_id: 4801,
                    name: "Supplied POI".into(),
                    ui_map_id: None,
                    position: (0.5, 0.5),
                    atlas_name: None,
                    description: None,
                    faction_id: None,
                    icon_widget_set: None,
                    linked_ui_map_id: None,
                    is_current_event: false,
                    should_glow: false,
                    is_suppressible: false,
                    is_locked: false,
                    seconds_left: None,
                },
            );
            sim.super_track.quest_id = Some(3001);
            sim.super_track.quest_tracking = true;
            sim.super_track.corpse = true;
        }
        env.exec(
            r#"
            local function click(areaPOI)
                local button = CreateFrame("Button", nil, UIParent, "POIButtonTemplate")
                button:SetAreaPOIID(areaPOI)
                button:GetScript("OnClick")(button, "LeftButton")
            end
            click(4801)
            local kind, id = C_SuperTrack.GetSuperTrackedMapPin()
            assert(kind == Enum.SuperTrackingMapPinType.AreaPOI and id == 4801)
            assert(not C_SuperTrack.IsSuperTrackingQuest())
            assert(C_SuperTrack.IsSuperTrackingCorpse())
            click(4802)
            local kindAfter, idAfter = C_SuperTrack.GetSuperTrackedMapPin()
            assert(kindAfter == kind and idAfter == id)
            "#,
        )
        .unwrap();
    });
}

// API-level: the ready-quest projection uses the simulator's existing
// resolved record input; it is not native request/cache identity evidence.
#[test]
fn super_track_map_pin_uses_only_the_supplied_resolved_quest_record() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.quest_log_entries
            .quest_special_item_inputs
            .cached_quest_records
            .insert(4801, Default::default());
    }
    env.exec(
        r#"
        C_SuperTrack.SetSuperTrackedMapPin("1.9", "4801.5")
        local kind, id = C_SuperTrack.GetSuperTrackedMapPin()
        assert(kind == 1 and id == 4801)
        for _, candidate in ipairs({{1, 4802}, {1, 0}, {2, 4801}, {3, 4801}, {4, 4801}}) do
            C_SuperTrack.SetSuperTrackedMapPin(candidate[1], candidate[2])
            local newKind, newID = C_SuperTrack.GetSuperTrackedMapPin()
            assert(newKind == kind and newID == id)
        end
        "#,
    )
    .unwrap();
}
