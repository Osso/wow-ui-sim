//! Runtime subsystems: text helpers, scroll scripts, C_Texture/atlas,
//! animation runtime, gamepad cursor, unit state, popup/tooltip frames.

use super::super::*;

#[test]
fn test_text_runtime_helpers_exist() {
    let env = WowLuaEnv::new().unwrap();
    let (
        font_object_ty,
        width_ty,
        text_to_fit_ty,
        hyperlink_ty,
        scroll_script_ty,
        frame_script_ty,
        width,
        hyperlinks_enabled,
    ): (String, String, String, String, String, String, f64, bool) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame")
            local fontString = frame:CreateFontString()
            fontString:SetText("Runtime helper text")
            fontString:SetFontObjectsToTry("GameFontHighlightLarge", "GameFontHighlightSmall")
            fontString:SetTextToFit("Runtime helper text")

            local messageFrame = CreateFrame("MessageFrame")
            messageFrame:SetHyperlinksEnabled(true)

            local scrollFrame = CreateFrame("ScrollFrame")
            scrollFrame:SetScript("OnVerticalScroll", function() end)
            frame:SetScript("OnKeyDown", function() end)

            return type(fontString:GetFontObject()),
                type(fontString.GetUnboundedStringWidth),
                type(fontString.SetTextToFit),
                type(messageFrame.SetHyperlinksEnabled),
                type(scrollFrame:GetScript("OnVerticalScroll")),
                type(frame:GetScript("OnKeyDown")),
                fontString:GetUnboundedStringWidth(),
                messageFrame:GetHyperlinksEnabled()
            "#,
        )
        .unwrap();

    assert_eq!(font_object_ty, "string");
    assert_eq!(width_ty, "function");
    assert_eq!(text_to_fit_ty, "function");
    assert_eq!(hyperlink_ty, "function");
    assert_eq!(scroll_script_ty, "function");
    assert_eq!(frame_script_ty, "function");
    assert!(width > 0.0);
    assert!(hyperlinks_enabled);
}
#[test]
fn test_scrollframe_scroll_scripts_fire_from_native_methods() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
            local scrollFrame = CreateFrame("ScrollFrame")
            scrollFrame:SetSize(100, 100)

            local scrollChild = CreateFrame("Frame")
            scrollChild:SetSize(100, 250)
            scrollFrame:SetScrollChild(scrollChild)

            local rangeCount, hRange, vRange = 0, nil, nil
            scrollFrame:SetScript("OnScrollRangeChanged", function(self, h, v)
                rangeCount = rangeCount + 1
                hRange = h
                vRange = v
            end)

            local scrollCount, scrollOffset = 0, nil
            scrollFrame:SetScript("OnVerticalScroll", function(self, offset)
                scrollCount = scrollCount + 1
                scrollOffset = offset
            end)

            scrollFrame:UpdateScrollChildRect()
            scrollFrame:SetVerticalScroll(40)

            function ScrollScriptSnapshot()
                return rangeCount, hRange, vRange, scrollCount, scrollOffset,
                    scrollFrame:GetVerticalScroll(), scrollFrame:GetVerticalScrollRange()
            end
        "#,
    )
    .unwrap();
    env.fire_on_update(0.016).unwrap();

    let (range_count, h_range, v_range, scroll_count, scroll_offset, current_scroll, current_range): (
        i32,
        f64,
        f64,
        i32,
        f64,
        f64,
        f64,
    ) = env
        .eval("return ScrollScriptSnapshot()")
        .unwrap();

    assert_eq!(range_count, 1);
    assert_eq!(h_range, 0.0);
    assert_eq!(v_range, 150.0);
    assert_eq!(scroll_count, 1);
    assert_eq!(scroll_offset, 40.0);
    assert_eq!(current_scroll, 40.0);
    assert_eq!(current_range, 150.0);
}

#[test]
fn test_scroll_child_resize_refreshes_parent_scroll_range() {
    let env = WowLuaEnv::new().unwrap();
    let (range_count, v_range, current_range): (i32, f64, f64) = env
        .eval(
            r#"
            local scrollFrame = CreateFrame("ScrollFrame")
            scrollFrame:SetSize(100, 100)

            local scrollChild = CreateFrame("Frame")
            scrollChild:SetSize(100, 80)
            scrollFrame:SetScrollChild(scrollChild)
            scrollFrame:UpdateScrollChildRect()

            local rangeCount, vRange = 0, nil
            scrollFrame:SetScript("OnScrollRangeChanged", function(self, h, v)
                rangeCount = rangeCount + 1
                vRange = v
            end)

            scrollChild:SetHeight(250)

            return rangeCount, vRange, scrollFrame:GetVerticalScrollRange()
            "#,
        )
        .unwrap();

    assert_eq!(range_count, 1);
    assert_eq!(v_range, 150.0);
    assert_eq!(current_range, 150.0);
}
#[test]
fn test_c_texture_exposes_atlas_lookup() {
    let env = WowLuaEnv::new().unwrap();
    let (exists_ty, exists, info_ty, width, height, tile_h, tile_v): (
        String,
        bool,
        String,
        i64,
        i64,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            local info = C_Texture.GetAtlasInfo("UI-Frame-InnerTopLeft")
            return type(C_Texture.GetAtlasExists),
                C_Texture.GetAtlasExists("UI-Frame-InnerTopLeft"),
                type(info),
                info and info.width or 0,
                info and info.height or 0,
                info and info.tilesHorizontally or false,
                info and info.tilesVertically or false
            "#,
        )
        .unwrap();

    assert_eq!(exists_ty, "function");
    assert!(exists);
    assert_eq!(info_ty, "table");
    assert!(width > 0);
    assert!(height > 0);
    assert!(!tile_h);
    assert!(!tile_v);
}

#[test]
fn test_animation_runtime_exposes_core_configuration_methods() {
    let env = WowLuaEnv::new().unwrap();
    let (group_method_ty, animation_duration_ty, animation_order_ty, finished_script_ty): (String, String, String, String) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame")
            local group = frame:CreateAnimationGroup()
            local animation = group:CreateAnimation("Alpha")
            group:SetToFinalAlpha(true)
            animation:SetDuration(0.5)
            animation:SetOrder(2)
            group:SetScript("OnFinished", function() end)
            return type(group.SetToFinalAlpha), type(animation.SetDuration), type(animation.SetOrder), type(group:GetScript("OnFinished"))
            "#,
        )
        .unwrap();

    assert_eq!(group_method_ty, "function");
    assert_eq!(animation_duration_ty, "function");
    assert_eq!(animation_order_ty, "function");
    assert_eq!(finished_script_ty, "function");
}

#[test]
fn test_animation_script_handler_matrix_matches_retail_probe() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local handlers = {
                "OnLoad", "OnUpdate", "OnEvent", "OnPlay", "OnPause", "OnStop",
                "OnFinished", "OnLoop", "OnShow", "OnHide", "OnEnter", "OnLeave",
            }
            local animationHandlers = {
                OnLoad = true, OnUpdate = true, OnEvent = false, OnPlay = true,
                OnPause = true, OnStop = true, OnFinished = true, OnLoop = false,
                OnShow = false, OnHide = false, OnEnter = false, OnLeave = false,
            }
            local groupHandlers = {
                OnLoad = true, OnUpdate = true, OnEvent = false, OnPlay = true,
                OnPause = true, OnStop = true, OnFinished = true, OnLoop = true,
                OnShow = false, OnHide = false, OnEnter = false, OnLeave = false,
            }
            local frameHandlers = {
                OnLoad = true, OnUpdate = true, OnEvent = true, OnPlay = false,
                OnPause = false, OnStop = false, OnFinished = false, OnLoop = false,
                OnShow = true, OnHide = true, OnEnter = true, OnLeave = true,
            }

            local function checkObject(object, objectName, expected)
                for _, handler in ipairs(handlers) do
                    local supported = expected[handler]
                    local hasOk, hasValue = pcall(object.HasScript, object, handler)
                    local setOk = pcall(object.SetScript, object, handler, function() end)
                    if setOk then
                        pcall(object.SetScript, object, handler, nil)
                    end

                    if supported then
                        if not hasOk or hasValue ~= true then
                            return objectName .. ":" .. handler .. ":HasScript"
                        end
                        if not setOk then
                            return objectName .. ":" .. handler .. ":SetScript"
                        end
                    else
                        if not hasOk or hasValue ~= false then
                            return objectName .. ":" .. handler .. ":HasScript"
                        end
                        if setOk then
                            return objectName .. ":" .. handler .. ":SetScript unexpectedly accepted"
                        end
                    end
                end
                return nil
            end

            local frame = CreateFrame("Frame")
            local group = frame:CreateAnimationGroup()
            local failure = checkObject(frame, "Frame", frameHandlers)
                or checkObject(group, "AnimationGroup", groupHandlers)
            if failure then return failure end

            local animationTypes = {
                "Alpha", "Translation", "Scale", "Rotation", "LineTranslation",
                "LineScale", "Path", "FlipBook", "VertexColor",
            }
            for _, animationType in ipairs(animationTypes) do
                local createOk, animation = pcall(group.CreateAnimation, group, animationType)
                if not createOk or animation == nil then
                    return "create:" .. animationType
                end
                failure = checkObject(animation, animationType, animationHandlers)
                if failure then return failure end
            end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn hiding_parent_stops_active_child_animation_group() {
    let env = WowLuaEnv::new().unwrap();
    let (playing, paused, done): (bool, bool, bool) = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", nil, UIParent)
            local child = CreateFrame("Frame", nil, parent)
            local group = child:CreateAnimationGroup()
            local animation = group:CreateAnimation("Alpha")
            animation:SetDuration(10)
            group:Play()
            parent:Hide()
            return group:IsPlaying(), group:IsPaused(), group:IsDone()
            "#,
        )
        .unwrap();

    assert!(
        !playing,
        "hiding a parent should stop playing child animations"
    );
    assert!(!paused, "stopped child animations should not remain paused");
    assert!(done, "stopped child animations should be marked done");
}

#[test]
fn test_flipbook_animation_runtime_exposes_configuration_surface() {
    let env = WowLuaEnv::new().unwrap();
    let (
        set_frames_ty,
        get_frames_ty,
        get_columns_ty,
        get_width_ty,
        frames,
        columns,
        width,
        height,
    ): (String, String, String, String, i64, i64, f64, f64) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame")
            local group = frame:CreateAnimationGroup()
            local animation = group:CreateAnimation("FlipBook")
            animation:SetFlipBookRows(3)
            animation:SetFlipBookColumns(4)
            animation:SetFlipBookFrames(12)
            animation:SetFlipBookFrameWidth(64)
            animation:SetFlipBookFrameHeight(32)
            return type(animation.SetFlipBookFrames), type(animation.GetFlipBookFrames),
                type(animation.GetFlipBookColumns), type(animation.GetFlipBookFrameWidth),
                animation:GetFlipBookFrames(), animation:GetFlipBookColumns(),
                animation:GetFlipBookFrameWidth(), animation:GetFlipBookFrameHeight()
            "#,
        )
        .unwrap();

    assert_eq!(set_frames_ty, "function");
    assert_eq!(get_frames_ty, "function");
    assert_eq!(get_columns_ty, "function");
    assert_eq!(get_width_ty, "function");
    assert_eq!(frames, 12);
    assert_eq!(columns, 4);
    assert_eq!(width, 64.0);
    assert_eq!(height, 32.0);
}
#[test]
fn test_gamepad_cursor_bootstrap_functions_exist() {
    let env = WowLuaEnv::new().unwrap();
    let (auto_ty, auto_value, set_ty): (String, bool, String) = env
        .eval(
            r#"
            return type(CanAutoSetGamePadCursorControl),
                CanAutoSetGamePadCursorControl(true),
                type(SetGamePadCursorControl)
            "#,
        )
        .unwrap();

    assert_eq!(auto_ty, "function");
    assert!(!auto_value);
    assert_eq!(set_ty, "function");
}

#[test]
fn test_unit_state_bootstrap_functions_exist() {
    let env = WowLuaEnv::new().unwrap();
    let (ghost_ty, ghost_value, dead_ty, dead_value): (String, bool, String, bool) = env
        .eval(
            r#"
            return type(UnitIsGhost), UnitIsGhost("player"), type(UnitIsDead), UnitIsDead("player")
            "#,
        )
        .unwrap();

    assert_eq!(ghost_ty, "function");
    assert!(!ghost_value);
    assert_eq!(dead_ty, "function");
    assert!(!dead_value);
}

// Retail and Forever take UISpecialFrames from the original UIParent; the
// placeholder tables are compiled out there (c1b07efb79).
#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
#[test]
fn test_ui_special_frames_table() {
    let env = WowLuaEnv::new().unwrap();
    let ty: String = env.eval("return type(UISpecialFrames)").unwrap();
    assert_eq!(ty, "table");
}

// SOUNDKIT: from Blizzard_SharedXML/SoundKitConstants.lua
// Tested via Lua addon tests (run-tests).

#[test]
fn test_game_tooltip_methods() {
    let env = WowLuaEnv::new().unwrap();
    for m in &["SetOwner", "Show", "Hide", "FadeOut"] {
        let expr = format!("return type(GameTooltip.{})", m);
        let ty: String = env.eval(&expr).unwrap();
        assert_eq!(ty, "function", "GameTooltip.{} should be function", m);
    }
}

// Retail and Forever take StaticPopup from the original Blizzard_StaticPopup
// addons; the bare-environment defaults are a no-op there (1cf0851a44, 1a2ba976fe).
#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
#[test]
fn test_static_popup() {
    let env = WowLuaEnv::new().unwrap();
    let ty: String = env.eval("return type(StaticPopup_Show)").unwrap();
    assert_eq!(ty, "function");
    let ty2: String = env.eval("return type(StaticPopupDialogs)").unwrap();
    assert_eq!(ty2, "table");
}

// ContinuableContainer, ItemButtonUtil, ScrollUtil, CreateScrollBoxLinearView,
// MainMenuBarBackpackButton: all from Blizzard addon Lua/XML.
// Tested via Lua addon tests (run-tests).
