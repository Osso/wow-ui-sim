//! Tests for frame-level, security, render layer, window, and visual state methods.
use super::*;

#[test]
fn test_frame_level_methods_follow_parent_level_and_raise_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        ParentLevelFrame = CreateFrame("Frame", "ParentLevelFrame", UIParent)
        ChildLevelFrame = CreateFrame("Frame", "ChildLevelFrame", ParentLevelFrame)
        GrandchildLevelFrame = CreateFrame("Frame", "GrandchildLevelFrame", ChildLevelFrame)
        SiblingLevelFrame = CreateFrame("Frame", "SiblingLevelFrame", ParentLevelFrame)

        ParentLevelFrame:SetFrameLevel(10)

        assert(not ChildLevelFrame:IsUsingParentLevel(), "a default child tracks parent level (+1) but does NOT report IsUsingParentLevel (verified vs retail 12.0.5 via XmlFrameLevelProbe childBare)")
        assert(ChildLevelFrame:GetFrameLevel() == 11, "child should inherit parent level plus default offset")
        assert(GrandchildLevelFrame:GetFrameLevel() == 12, "grandchild should inherit recursively")
        assert(ParentLevelFrame:GetHighestFrameLevel() == 10, "default highest level should use self")
        assert(ParentLevelFrame:GetHighestFrameLevel(true) == 12, "highest level should include descendants when requested")

        ChildLevelFrame:SetUsingParentLevel(false)
        ChildLevelFrame:SetFrameLevel(30)

        assert(not ChildLevelFrame:IsUsingParentLevel(), "child should stop inheriting after SetUsingParentLevel(false)")
        assert(ChildLevelFrame:GetFrameLevel() == 30, "child should keep explicit fixed frame level")
        assert(ChildLevelFrame:GetHighestFrameLevel(true) == 31, "fixed child highest level should include descendant")
        assert(GrandchildLevelFrame:GetFrameLevel() == 31, "grandchild should inherit from fixed child level")

        ParentLevelFrame:SetFrameLevel(20)

        assert(ChildLevelFrame:GetFrameLevel() == 30, "fixed child level should survive parent level changes")
        assert(SiblingLevelFrame:GetFrameLevel() == 21, "sibling should continue inheriting updated parent level")
        assert(ParentLevelFrame:GetHighestFrameLevel(true) == 31, "highest level should reflect deepest descendant")

        ChildLevelFrame:Raise()
        assert(ChildLevelFrame:GetRaisedFrameLevel() == 0, "retail keeps simple raised frame level hidden from Lua")

        ChildLevelFrame:SetUsingParentLevel(true)

        assert(ChildLevelFrame:IsUsingParentLevel(), "child should resume inheriting after SetUsingParentLevel(true)")
        assert(ChildLevelFrame:GetFrameLevel() == 21, "child should snap back to parent-derived level")
        assert(ChildLevelFrame:GetHighestFrameLevel(true) == 22, "highest level should update after re-enabling inheritance")
        assert(GrandchildLevelFrame:GetFrameLevel() == 22, "grandchild should re-inherit from updated child level")

        FRAME_LEVEL_METHODS_OK = true
    "#,
    )
    .unwrap();

    let ok: bool = env.eval("return FRAME_LEVEL_METHODS_OK == true").unwrap();
    assert!(ok, "frame level method round-trip should succeed");
}

#[test]
fn test_secret_and_protected_methods_reflect_frame_security_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        SecretValuesFrame = CreateFrame("Frame", "SecretValuesFrame", UIParent)
        ProtectedSecretFrame = CreateFrame("Frame", "ProtectedSecretFrame", UIParent)
        ForbiddenSecretFrame = CreateFrame("Frame", "ForbiddenSecretFrame", UIParent)

        assert(not SecretValuesFrame:HasAnySecretAspect(), "new frame should not have secret aspects by default")
        assert(not SecretValuesFrame:HasSecretValues(), "new frame should not have secret values by default")
        assert(not SecretValuesFrame:IsPreventingSecretValues(), "new frame should not prevent secret values by default")
        assert(not SecretValuesFrame:IsAnchoringSecret(), "new frame should not be anchoring secret by default")
        assert(not SecretValuesFrame:IsAnchoringRestricted(), "new frame should not be anchoring restricted by default")
        assert(not SecretValuesFrame:HasSecretAspect(Enum.SecretAspect.FrameLevel), "unrelated secret aspect should stay false")

        "#,
    )
    .unwrap();
    let id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("SecretValuesFrame")
        .unwrap();
    env.state()
        .borrow_mut()
        .widgets
        .get_mut(id)
        .unwrap()
        .prevent_secret_values = true;
    env.exec(
        r#"

        assert(SecretValuesFrame:IsPreventingSecretValues(), "SetPreventSecretValues(true) should persist")
        assert(SecretValuesFrame:HasSecretValues(), "preventing secret values should mark the frame as having secret values")
        assert(SecretValuesFrame:HasAnySecretAspect(), "secret-valued frame should report a secret aspect")
        assert(SecretValuesFrame:HasSecretAspect(Enum.SecretAspect.ObjectSecrets), "object secret aspect should be present")
        assert(SecretValuesFrame:IsAnchoringSecret(), "secret-valued frame should be anchoring secret")
        assert(not SecretValuesFrame:IsAnchoringRestricted(), "secret-valued frame should not become anchoring restricted")

        "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .widgets
        .get_mut(id)
        .unwrap()
        .prevent_secret_values = false;
    env.exec(
        r#"

        assert(not SecretValuesFrame:IsPreventingSecretValues(), "SetPreventSecretValues(false) should clear")
        assert(not SecretValuesFrame:HasSecretValues(), "clearing prevention should clear secret values")
        assert(not SecretValuesFrame:HasAnySecretAspect(), "clearing prevention should clear secret aspects")
        assert(not SecretValuesFrame:IsAnchoringSecret(), "clearing prevention should clear anchoring secret")

        A_Admin.SetFrameProtected("ProtectedSecretFrame", true)
        ForbiddenSecretFrame:SetForbidden(true)

        assert(ProtectedSecretFrame:IsAnchoringRestricted(), "protected frames should be anchoring restricted")
        assert(ProtectedSecretFrame:HasAnySecretAspect(), "protected frames should report a secret/security aspect")
        assert(ProtectedSecretFrame:HasSecretAspect(Enum.SecretAspect.ObjectSecurity), "protected frames should report object security aspect")
        assert(not ProtectedSecretFrame:HasSecretValues(), "protected frames should not imply secret values")
        assert(not ProtectedSecretFrame:IsAnchoringSecret(), "protected frames should not imply anchoring secret")

        assert(ForbiddenSecretFrame:IsAnchoringRestricted(), "forbidden frames should be anchoring restricted")
        assert(ForbiddenSecretFrame:HasAnySecretAspect(), "forbidden frames should report a secret/security aspect")
        assert(ForbiddenSecretFrame:HasSecretAspect(Enum.SecretAspect.ObjectSecurity), "forbidden frames should report object security aspect")

        SECRET_PROTECTED_STATE_OK = true
    "#,
    )
    .unwrap();

    let ok: bool = env
        .eval("return SECRET_PROTECTED_STATE_OK == true")
        .unwrap();
    assert!(
        ok,
        "secret/protected state should round-trip through frame methods"
    );
}

#[test]
fn test_flatten_render_methods_track_local_and_inherited_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        FlattenRootFrame = CreateFrame("Frame", "FlattenRootFrame", UIParent)
        FlattenParentFrame = CreateFrame("Frame", "FlattenParentFrame", FlattenRootFrame)
        FlattenChildFrame = CreateFrame("Frame", "FlattenChildFrame", FlattenParentFrame)

        assert(not FlattenRootFrame:GetFlattensRenderLayers(), "new frames should default flatten=false")
        assert(not FlattenRootFrame:GetEffectivelyFlattensRenderLayers(), "root should default effective flatten=false")
        assert(not FlattenChildFrame:GetFlattensRenderLayers(), "child local flatten should default false")
        assert(not FlattenChildFrame:GetEffectivelyFlattensRenderLayers(), "child effective flatten should default false")

        FlattenParentFrame:SetFlattensRenderLayers(true)

        assert(FlattenParentFrame:GetFlattensRenderLayers(), "local flatten flag should persist on the frame")
        assert(FlattenParentFrame:GetEffectivelyFlattensRenderLayers(), "frame should effectively flatten when local flag is enabled")
        assert(not FlattenChildFrame:GetFlattensRenderLayers(), "descendants should not inherit the local flatten flag itself")
        assert(FlattenChildFrame:GetEffectivelyFlattensRenderLayers(), "descendants should inherit effective flattening from ancestors")
        assert(not FlattenRootFrame:GetEffectivelyFlattensRenderLayers(), "ancestors should not inherit flattening upward")

        FlattenChildFrame:SetFlattensRenderLayers(true)
        FlattenParentFrame:SetFlattensRenderLayers(false)

        assert(FlattenChildFrame:GetFlattensRenderLayers(), "child local flatten flag should persist independently")
        assert(FlattenChildFrame:GetEffectivelyFlattensRenderLayers(), "child local flatten should keep effective flattening enabled")
        assert(not FlattenParentFrame:GetFlattensRenderLayers(), "parent local flatten flag should clear")
        assert(not FlattenParentFrame:GetEffectivelyFlattensRenderLayers(), "cleared parent should stop flattening effectively")

        FLATTEN_RENDER_METHODS_OK = true
    "#,
    )
    .unwrap();

    let ok: bool = env
        .eval("return FLATTEN_RENDER_METHODS_OK == true")
        .unwrap();
    assert!(
        ok,
        "flatten render layer methods should track local and inherited state"
    );
}

#[test]
fn test_window_display_methods_persist_window_and_dont_save_position() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        WindowOwnerFrame = CreateFrame("Frame", "WindowOwnerFrame", UIParent)
        local firstWindow = { name = "first" }
        local secondWindow = { name = "second" }

        assert(not WindowOwnerFrame:GetDontSavePosition(), "frames should default to saving their position")
        assert(WindowOwnerFrame:GetWindow() == nil, "frames should default to no associated window")

        WindowOwnerFrame:SetDontSavePosition(true)
        assert(WindowOwnerFrame:GetDontSavePosition(), "SetDontSavePosition(true) should persist")

        WindowOwnerFrame:SetWindow(firstWindow)
        assert(WindowOwnerFrame:GetWindow() == firstWindow, "SetWindow should persist the associated window object")

        WindowOwnerFrame:SetWindow(secondWindow)
        assert(WindowOwnerFrame:GetWindow() == secondWindow, "SetWindow should overwrite the previous window object")

        WindowOwnerFrame:SetWindow(nil)
        assert(WindowOwnerFrame:GetWindow() == nil, "SetWindow(nil) should clear the associated window")

        WindowOwnerFrame:SetDontSavePosition(false)
        assert(not WindowOwnerFrame:GetDontSavePosition(), "SetDontSavePosition(false) should clear the persisted flag")

        WINDOW_DISPLAY_METHODS_OK = true
    "#,
    )
    .unwrap();

    let ok: bool = env
        .eval("return WINDOW_DISPLAY_METHODS_OK == true")
        .unwrap();
    assert!(
        ok,
        "window display methods should persist associated window and dont-save-position state"
    );
}

#[test]
fn test_resize_and_user_placement_methods_persist_frame_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        ResizeStateFrame = CreateFrame("Frame", "ResizeStateFrame", UIParent)

        local minWidth, minHeight, maxWidth, maxHeight = ResizeStateFrame:GetResizeBounds()
        assert(minWidth == 0 and minHeight == 0, "frames should default to zero minimum resize bounds")
        assert(maxWidth == nil and maxHeight == nil, "frames should default to no maximum resize bounds")
        assert(not ResizeStateFrame:IsUserPlaced(), "frames should default to userPlaced=false")

        ResizeStateFrame:SetMinResize(120, 80)
        minWidth, minHeight, maxWidth, maxHeight = ResizeStateFrame:GetResizeBounds()
        assert(minWidth == 120 and minHeight == 80, "SetMinResize should persist minimum resize bounds")
        assert(maxWidth == nil and maxHeight == nil, "SetMinResize should not invent maximum bounds")

        ResizeStateFrame:SetMaxResize(480, 360)
        minWidth, minHeight, maxWidth, maxHeight = ResizeStateFrame:GetResizeBounds()
        assert(maxWidth == 480 and maxHeight == 360, "SetMaxResize should persist maximum resize bounds")
        assert(minWidth == 120 and minHeight == 80, "SetMaxResize should preserve existing minimum resize bounds")

        ResizeStateFrame:SetResizeBounds(160, 90, 640, 480)
        minWidth, minHeight, maxWidth, maxHeight = ResizeStateFrame:GetResizeBounds()
        assert(minWidth == 160 and minHeight == 90, "SetResizeBounds should overwrite minimum resize bounds")
        assert(maxWidth == 640 and maxHeight == 480, "SetResizeBounds should overwrite maximum resize bounds")

        ResizeStateFrame:SetResizeBounds(200, 110)
        minWidth, minHeight, maxWidth, maxHeight = ResizeStateFrame:GetResizeBounds()
        assert(minWidth == 200 and minHeight == 110, "two-argument SetResizeBounds should keep the new minimum resize bounds")
        assert(maxWidth == nil and maxHeight == nil, "two-argument SetResizeBounds should clear maximum resize bounds")

        ResizeStateFrame:SetUserPlaced(true)
        assert(ResizeStateFrame:IsUserPlaced(), "SetUserPlaced(true) should persist")

        ResizeStateFrame:SetUserPlaced(false)
        assert(not ResizeStateFrame:IsUserPlaced(), "SetUserPlaced(false) should clear the persisted flag")

        RESIZE_AND_USER_PLACEMENT_METHODS_OK = true
    "#,
    )
    .unwrap();

    let ok: bool = env
        .eval("return RESIZE_AND_USER_PLACEMENT_METHODS_OK == true")
        .unwrap();
    assert!(
        ok,
        "resize and user placement methods should persist frame state"
    );
}

#[test]
fn test_misc_visual_state_methods_persist_and_desaturate_hierarchy() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        MiscStateRootFrame = CreateFrame("Frame", "MiscStateRootFrame", UIParent)
        MiscStateChildTexture = MiscStateRootFrame:CreateTexture("MiscStateChildTexture", "ARTWORK")
        MiscStateGrandchildFrame = CreateFrame("Frame", "MiscStateGrandchildFrame", MiscStateRootFrame)
        MiscStateGrandchildTexture = MiscStateGrandchildFrame:CreateTexture("MiscStateGrandchildTexture", "ARTWORK")

        assert(not MiscStateRootFrame:IsHighlightLocked(), "highlight lock should default false")
        assert(not MiscStateRootFrame:IsIgnoringChildrenForBounds(), "ignore children for bounds should default false")
        assert(not MiscStateChildTexture:IsDesaturated(), "child texture should default not desaturated")
        assert(not MiscStateGrandchildTexture:IsDesaturated(), "grandchild texture should default not desaturated")

        MiscStateRootFrame:SetHighlightLocked(true)
        MiscStateRootFrame:SetIgnoringChildrenForBounds(true)
        MiscStateRootFrame:DesaturateHierarchy(1, true)

        assert(MiscStateRootFrame:IsHighlightLocked(), "highlight lock should persist")
        assert(MiscStateRootFrame:IsIgnoringChildrenForBounds(), "ignore children for bounds should persist")
        assert(MiscStateChildTexture:IsDesaturated(), "desaturate hierarchy should affect direct child textures")
        assert(MiscStateGrandchildTexture:IsDesaturated(), "desaturate hierarchy should affect descendant textures")

        MiscStateRootFrame:SetHighlightLocked(false)
        MiscStateRootFrame:SetIgnoringChildrenForBounds(false)
        MiscStateRootFrame:DesaturateHierarchy(0)

        assert(not MiscStateRootFrame:IsHighlightLocked(), "highlight lock should clear")
        assert(not MiscStateRootFrame:IsIgnoringChildrenForBounds(), "ignore children for bounds should clear")
        assert(not MiscStateChildTexture:IsDesaturated(), "desaturate hierarchy should clear child textures")
        assert(not MiscStateGrandchildTexture:IsDesaturated(), "desaturate hierarchy should clear descendant textures")

        MISC_VISUAL_STATE_METHODS_OK = true
    "#,
    )
    .unwrap();

    {
        let state = env.state().borrow();
        let root_id = state
            .widgets
            .get_id_by_name("MiscStateRootFrame")
            .expect("root frame should exist");
        let root = state
            .widgets
            .get(root_id)
            .expect("root frame should be readable");
        assert!(
            !root.desaturated,
            "excludeRoot=true should leave the root frame undessaturated"
        );
    }

    let ok: bool = env
        .eval("return MISC_VISUAL_STATE_METHODS_OK == true")
        .unwrap();
    assert!(
        ok,
        "misc visual state methods should persist booleans and desaturate descendants"
    );
}
