#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn character_create_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("character-create environment");
    env.set_screen_mode(ScreenKind::Game);
    env.exec(
        r##"
        local namespace = rawget(_G, "C_CharacterCreation")
        assert(namespace == nil
            or (rawget(namespace, "GetCurrentCameraZoom") == nil
                and rawget(namespace, "SetCameraZoomLevel") == nil
                and rawget(namespace, "SetModelDressState") == nil
                and rawget(namespace, "SetSelectedPreviewGearType") == nil))
        "##,
    )
    .expect("camera and preview APIs are not installed on Game");
    env.set_screen_mode(ScreenKind::CharacterCreate);
    env
}

#[test]
fn camera_zoom_level_clamps_and_getter_reads_stored_value() {
    let env = character_create_env();
    env.exec(
        r##"
        local getZoom = rawget(C_CharacterCreation, "GetCurrentCameraZoom")
        local setZoom = rawget(C_CharacterCreation, "SetCameraZoomLevel")
        assert(type(getZoom) == "function" and type(setZoom) == "function")
        assert(rawget(C_CharacterCreation, "GetCameraZoomLevel") == nil)

        local usage = "Usage: C_CharacterCreation.SetCameraZoomLevel(zoomLevel [, keepCustomZoom])"
        for _, call in ipairs({ function() setZoom() end, function() setZoom(nil) end }) do
            local ok, message = pcall(call)
            assert(not ok and message:find("bad argument #1", 1, true), tostring(message))
            assert(message:find(usage, 1, true), tostring(message))
        end

        assert(select("#", setZoom(-13.9)) == 0)
        assert(select("#", getZoom()) == 1 and getZoom() == 0)
        assert(select("#", setZoom(143.9)) == 0)
        assert(getZoom() == 100)
        assert(select("#", setZoom("42.8")) == 0)
        assert(getZoom() == 42)
        "##,
    )
    .expect("camera zoom input conversion, clamping, storage and getter");
    assert_eq!(env.state().borrow().character_create_camera_zoom, 42);
}

#[test]
fn camera_zoom_keep_custom_uses_optional_lua_truthiness() {
    let env = character_create_env();
    env.exec("C_CharacterCreation.SetCameraZoomLevel(42)")
        .expect("seed the camera zoom through its public setter");
    env.state()
        .borrow_mut()
        .character_create_camera_zoom_is_custom = true;

    env.exec(
        r##"
        local getZoom = C_CharacterCreation.GetCurrentCameraZoom
        local setZoom = C_CharacterCreation.SetCameraZoomLevel
        assert(select("#", setZoom(80, 0)) == 0 and getZoom() == 42)
        assert(select("#", setZoom(80, "")) == 0 and getZoom() == 42)
        assert(select("#", setZoom(80, {})) == 0 and getZoom() == 42)
        "##,
    )
    .expect("zero, empty strings and tables keep a custom zoom");
    assert!(env.state().borrow().character_create_camera_zoom_is_custom);

    env.exec("assert(select('#', C_CharacterCreation.SetCameraZoomLevel(80)) == 0)")
        .expect("missing keepCustomZoom is false");
    assert_eq!(env.state().borrow().character_create_camera_zoom, 80);
    assert!(!env.state().borrow().character_create_camera_zoom_is_custom);

    env.state()
        .borrow_mut()
        .character_create_camera_zoom_is_custom = true;
    env.exec("assert(select('#', C_CharacterCreation.SetCameraZoomLevel(85, nil)) == 0)")
        .expect("nil keepCustomZoom is false");
    assert_eq!(env.state().borrow().character_create_camera_zoom, 85);
    assert!(!env.state().borrow().character_create_camera_zoom_is_custom);

    env.state()
        .borrow_mut()
        .character_create_camera_zoom_is_custom = true;
    env.exec("assert(select('#', C_CharacterCreation.SetCameraZoomLevel(90, false)) == 0)")
        .expect("false keepCustomZoom replaces a custom zoom");
    assert_eq!(env.state().borrow().character_create_camera_zoom, 90);
    assert!(!env.state().borrow().character_create_camera_zoom_is_custom);

    env.state()
        .borrow_mut()
        .character_create_camera_zoom_is_custom = true;
    env.exec("assert(select('#', C_CharacterCreation.SetCameraZoomLevel(90, false)) == 0)")
        .expect("an unchanged zoom does not clear the custom flag");
    assert!(env.state().borrow().character_create_camera_zoom_is_custom);
}

#[test]
fn preview_only_setters_validate_without_events_or_cvar_updates() {
    let env = character_create_env();
    let events_before = env
        .state()
        .borrow()
        .events
        .pending()
        .iter()
        .map(|event| event.name.clone())
        .collect::<Vec<_>>();

    env.exec(
        r##"
        local setDress = rawget(C_CharacterCreation, "SetModelDressState")
        local setGear = rawget(C_CharacterCreation, "SetSelectedPreviewGearType")
        assert(type(setDress) == "function" and type(setGear) == "function")
        assert(rawget(C_CharacterCreation, "GetModelDressState") == nil)
        assert(rawget(C_CharacterCreation, "GetSelectedPreviewGearType") == nil)

        local function expect_usage(call, usage)
            local ok, message = pcall(call)
            assert(not ok and message:find("bad argument #1", 1, true), tostring(message))
            assert(message:find(usage, 1, true), tostring(message))
        end
        expect_usage(function() setDress() end,
            "Usage: C_CharacterCreation.SetModelDressState(dressedState)")
        expect_usage(function() setDress(nil) end,
            "Usage: C_CharacterCreation.SetModelDressState(dressedState)")
        expect_usage(function() setGear() end,
            "Usage: C_CharacterCreation.SetSelectedPreviewGearType(gearType)")
        expect_usage(function() setGear(nil) end,
            "Usage: C_CharacterCreation.SetSelectedPreviewGearType(gearType)")

        local scale = GetCVar("uiScale")
        local useScale = GetCVar("useUiScale")
        cvarUpdateCount = 0
        local eventProbe = CreateFrame("Frame")
        eventProbe:RegisterEvent("CVAR_UPDATE")
        eventProbe:SetScript("OnEvent", function() cvarUpdateCount = cvarUpdateCount + 1 end)

        assert(select("#", setDress(false)) == 0)
        assert(select("#", setDress(0)) == 0)
        assert(select("#", setDress("")) == 0)
        assert(select("#", setDress({})) == 0)
        assert(select("#", setGear(0)) == 0)
        assert(select("#", setGear(1)) == 0)
        assert(select("#", setGear(256)) == 0)
        assert(select("#", setGear(257)) == 0)
        assert(select("#", setGear(-256)) == 0)
        assert(select("#", setGear(-255)) == 0)
        assert(select("#", setGear(1.9)) == 0)
        assert(select("#", setGear("257")) == 0)
        assert(select("#", setGear(false)) == 0)
        assert(select("#", setGear({})) == 0)
        assert(select("#", C_CharacterCreation.SetCameraZoomLevel(55, {})) == 0)

        local ok, message = pcall(setGear, 2)
        assert(not ok and message:find("bad argument #1", 1, true), tostring(message))
        assert(message:find(
            "Usage: C_CharacterCreation.SetSelectedPreviewGearType(gearType)",
            1,
            true
        ), tostring(message))
        local negative_ok, negative_message = pcall(setGear, -1)
        assert(not negative_ok and negative_message:find(
            "Usage: C_CharacterCreation.SetSelectedPreviewGearType(gearType)",
            1,
            true
        ), tostring(negative_message))
        assert(cvarUpdateCount == 0)
        assert(GetCVar("uiScale") == scale and GetCVar("useUiScale") == useScale)
        "##,
    )
    .expect("model-only setters validate inputs without changing observable CVar or event state");

    let events_after = env
        .state()
        .borrow()
        .events
        .pending()
        .iter()
        .map(|event| event.name.clone())
        .collect::<Vec<_>>();
    assert_eq!(events_after, events_before);
}
