//! Tests for _G frame access behavior.
//!
//! Frames are eagerly registered in _G via raw_set at creation time.

use super::*;

#[test]
fn test_create_frame_named_sets_global() {
    let (t, _) = load_test_lua(
        "test-g-named",
        r#"
        local f = CreateFrame("Frame", "GlobalTestFrame", UIParent)
        GLOBAL_LOOKUP_OK = (_G["GlobalTestFrame"] == f)
        BARE_LOOKUP_OK = (GlobalTestFrame == f)
    "#,
    );
    t.assert_lua_true("return GLOBAL_LOOKUP_OK", "named frame should be in _G");
    t.assert_lua_true(
        "return BARE_LOOKUP_OK",
        "named frame accessible as bare global",
    );
}

#[test]
fn test_create_frame_unnamed_not_in_globals() {
    let (t, _) = load_test_lua(
        "test-g-unnamed",
        r#"
        local f = CreateFrame("Frame", nil, UIParent)
        -- Unnamed frame should not pollute _G with any user-visible name
        UNNAMED_OK = true
        for k, v in pairs(_G) do
            if v == f and not tostring(k):find("^__") then
                UNNAMED_OK = false
            end
        end
    "#,
    );
    t.assert_lua_true(
        "return UNNAMED_OK",
        "unnamed frame should not appear in _G under user-visible keys",
    );
}

#[test]
fn test_create_frame_returns_functional_handle() {
    let (t, _) = load_test_lua(
        "test-g-handle",
        r#"
        local f = CreateFrame("Frame", "HandleTestFrame", UIParent)
        f:SetSize(123, 456)
        W = f:GetWidth()
        H = f:GetHeight()
        NAME = f:GetName()
    "#,
    );
    assert_eq!(t.env.eval::<f64>("return W").unwrap(), 123.0);
    assert_eq!(t.env.eval::<f64>("return H").unwrap(), 456.0);
    t.assert_lua_str("return NAME", "HandleTestFrame");
}

#[test]
fn test_global_overwritten_by_recreate() {
    let (t, _) = load_test_lua(
        "test-g-overwrite",
        r#"
        local f1 = CreateFrame("Frame", "OverwriteFrame", UIParent)
        f1:SetSize(100, 100)
        local f2 = CreateFrame("Frame", "OverwriteFrame", UIParent)
        f2:SetSize(200, 200)
        GLOBAL_IS_F2 = (_G["OverwriteFrame"] == f2)
        F2_WIDTH = f2:GetWidth()
    "#,
    );
    t.assert_lua_true("return GLOBAL_IS_F2", "_G should point to the second frame");
    assert_eq!(t.env.eval::<f64>("return F2_WIDTH").unwrap(), 200.0);
}

// Duplicate named CreateFrame keeps the replacement fresh instead of copying
// Lua table fields from the previous global binding — copying would let stale
// frame identity tokens dispatch replacement-frame methods through retired
// widgets. See `duplicate_named_frame_gets_fresh_identity_and_fields` in
// tests/globals_legacy.rs.
#[test]
fn test_recreated_named_parent_drops_lua_child_field() {
    let (t, _) = load_test_lua(
        "test-g-recreate-parentkey",
        r#"
        local parent1 = CreateFrame("Frame", "RecreatedParent", UIParent)
        local child = CreateFrame("Frame", "RecreatedParentChild", parent1)
        parent1.Child = child

        local parent2 = CreateFrame("Frame", "RecreatedParent", UIParent)

        OLD_FIELD_KEPT = (parent1.Child == child)
        CHILD_FIELD_FRESH = (parent2.Child == nil)
        GLOBAL_IS_REPLACEMENT = (_G["RecreatedParent"] == parent2)
        "#,
    );
    t.assert_lua_true(
        "return OLD_FIELD_KEPT",
        "original frame should keep its own Lua child field",
    );
    t.assert_lua_true(
        "return CHILD_FIELD_FRESH",
        "recreated named parent should not inherit Lua child field assignments",
    );
    t.assert_lua_true(
        "return GLOBAL_IS_REPLACEMENT",
        "global name should point at the replacement frame",
    );
}

#[test]
fn test_authored_definition_retires_bootstrap_placeholder_and_reparents_children() {
    let (t, _) = load_test_lua(
        "test-g-recreate-retires-old-frame",
        r#"
        local oldParent = HelpFrame
        oldParent:Show()
        local child = CreateFrame("Frame", "RecreatedVisibleParentChild", oldParent)
        oldParent.Child = child

        local newParent = CreateFrame("Frame", "HelpFrame", UIParent)

        OLD_PARENT_HIDDEN = not oldParent:IsShown()
        CHILD_PARENT_IS_NEW = child:GetParent() == newParent
        NEW_PARENT_CHILD_FIELD_FRESH = newParent.Child == nil
        GLOBAL_IS_NEW_PARENT = _G.HelpFrame == newParent
        "#,
    );
    t.assert_lua_true(
        "return OLD_PARENT_HIDDEN",
        "authored definition should retire the simulator bootstrap placeholder",
    );
    t.assert_lua_true(
        "return CHILD_PARENT_IS_NEW",
        "children of the stale frame should be reparented to the replacement",
    );
    t.assert_lua_true(
        "return NEW_PARENT_CHILD_FIELD_FRESH",
        "replacement frame should not inherit parentKey-style child fields",
    );
    t.assert_lua_true(
        "return GLOBAL_IS_NEW_PARENT",
        "global name should point to the replacement frame",
    );
}

#[test]
fn test_xml_named_frame_in_global() {
    let t = load_test_xml(
        "test-g-xml",
        r#"<Ui>
            <Frame name="XMLGlobalFrame" parent="UIParent">
                <Size x="100" y="50"/>
            </Frame>
        </Ui>"#,
    );
    t.assert_lua_true(
        "return XMLGlobalFrame ~= nil",
        "XML frame should be a global",
    );
    t.assert_lua_true(
        "return _G['XMLGlobalFrame'] == XMLGlobalFrame",
        "_G lookup should match bare name lookup",
    );
    assert_eq!(
        t.env
            .eval::<f64>("return XMLGlobalFrame:GetWidth()")
            .unwrap(),
        100.0,
    );
}

#[test]
fn test_xml_child_texture_in_global() {
    let t = load_test_xml(
        "test-g-childtex",
        r#"<Ui>
            <Frame name="TexGlobalParent" parent="UIParent">
                <Layers><Layer level="ARTWORK">
                    <Texture name="TexGlobalParent_Icon" parentKey="icon"/>
                </Layer></Layers>
            </Frame>
        </Ui>"#,
    );
    t.assert_lua_true(
        "return TexGlobalParent_Icon ~= nil",
        "named child texture should be a global",
    );
    t.assert_lua_true(
        "return TexGlobalParent.icon == TexGlobalParent_Icon",
        "parentKey lookup should match global lookup",
    );
}

#[test]
fn test_template_inherited_parent_key_assigns_lua_field_on_parent() {
    let t = load_test_xml(
        "test-g-template-parentkey",
        r#"<Ui>
            <Frame name="InheritedParentKeyTemplate" parentKey="PartyBackfill" virtual="true"/>
            <Frame name="InheritedParentKeyParent" parent="UIParent">
                <Frames>
                    <Frame name="$parentPartyBackfill" inherits="InheritedParentKeyTemplate"/>
                </Frames>
            </Frame>
        </Ui>"#,
    );
    t.assert_lua_true(
        "return InheritedParentKeyParent.PartyBackfill == InheritedParentKeyParentPartyBackfill",
        "template-inherited parentKey should assign a Lua field on the instantiated parent",
    );
}

#[test]
fn test_template_inherited_parent_key_climbs_one_level_for_nested_child() {
    let t = load_test_xml(
        "test-g-template-parentkey-parent",
        r#"<Ui>
            <Frame name="NestedParentKeyTemplate" parentKey="$parent.CloseButton" virtual="true"/>
            <Frame name="NestedParentKeyOuter" parent="UIParent">
                <Frames>
                    <Frame name="$parentInner" parentKey="Inner">
                        <Frames>
                            <Frame name="$parentGrandchild" inherits="NestedParentKeyTemplate"/>
                        </Frames>
                    </Frame>
                </Frames>
            </Frame>
        </Ui>"#,
    );
    t.assert_lua_true(
        "return NestedParentKeyOuter.CloseButton == NestedParentKeyOuterInnerGrandchild",
        "template-inherited $parent parentKey should assign on the nested child's grandparent",
    );
}

#[test]
fn test_button_frame_template_inset_parent_key_points_to_child() {
    let (t, _) = load_test_lua(
        "test-g-button-frame-inset",
        r#"
        local frame = CreateFrame("Frame", "ButtonFrameInsetAccess", UIParent, "ButtonFrameTemplate")
        local inset = frame.Inset
        local insetParent = inset and inset.GetParent and inset:GetParent() or nil
        HAS_INSET = inset ~= nil
        INSET_IS_PARENT = inset == frame
        INSET_PARENT_IS_FRAME = insetParent == frame
        INSET_NAME = inset and inset.GetName and inset:GetName() or "nil"
    "#,
    );
    let chain = crate::xml::get_template_chain("ButtonFrameTemplate");
    let last = chain
        .last()
        .expect("ButtonFrameTemplate should resolve to a non-empty template chain after env init");
    assert_eq!(
        last.name, "ButtonFrameTemplate",
        "ButtonFrameTemplate chain should keep the derived template as the final entry",
    );
    assert!(
        last.frame
            .all_frame_elements()
            .iter()
            .any(|(frame, tag)| *tag == "Frame" && frame.parent_key.as_deref() == Some("Inset")),
        "ButtonFrameTemplate template entry should contain the derived Inset child frame",
    );
    {
        let state = t.env.state().borrow();
        let frame_id = state
            .widgets
            .get_id_by_name("ButtonFrameInsetAccess")
            .expect("parent button frame should exist in the registry");
        let inset_id = state
            .widgets
            .get_id_by_name("ButtonFrameInsetAccessInset")
            .expect("template inset child should be created in the registry");
        let frame = state
            .widgets
            .get(frame_id)
            .expect("parent button frame should resolve by id");
        assert_eq!(
            frame.children_keys.get("Inset").copied(),
            Some(inset_id),
            "template inset child should be registered in the parent's children_keys map",
        );
    }
    t.assert_lua_true(
        "return HAS_INSET",
        "ButtonFrameTemplate should expose its Inset child through parentKey lookup",
    );
    t.assert_lua_true(
        "return not INSET_IS_PARENT",
        "ButtonFrameTemplate.Inset must not resolve to the parent frame itself",
    );
    t.assert_lua_true(
        "return INSET_PARENT_IS_FRAME",
        "ButtonFrameTemplate inset child should be parented to the frame",
    );
    t.assert_lua_str("return INSET_NAME", "ButtonFrameInsetAccessInset");
}

#[test]
fn test_xml_button_frame_template_inset_parent_key_points_to_child() {
    let t = load_test_xml(
        "test-g-xml-button-frame-inset",
        r#"<Ui>
            <Frame name="XMLButtonFrameInsetAccess" parent="UIParent" inherits="ButtonFrameTemplate"/>
        </Ui>"#,
    );
    t.assert_lua_true(
        "return XMLButtonFrameInsetAccess.Inset ~= nil",
        "XML ButtonFrameTemplate instance should expose its Inset child through parentKey lookup",
    );
    t.assert_lua_true(
        "return XMLButtonFrameInsetAccess.Inset ~= XMLButtonFrameInsetAccess",
        "XML ButtonFrameTemplate.Inset must not resolve to the parent frame itself",
    );
    t.assert_lua_true(
        "return XMLButtonFrameInsetAccess.Inset:GetParent() == XMLButtonFrameInsetAccess",
        "XML ButtonFrameTemplate inset child should be parented to the frame",
    );
    t.assert_lua_str(
        "return XMLButtonFrameInsetAccess.Inset:GetName()",
        "XMLButtonFrameInsetAccessInset",
    );
}

#[test]
fn test_button_child_globals_not_on_fresh_button() {
    let (t, _) = load_test_lua(
        "test-g-btn-no-children",
        r#"
        local btn = CreateFrame("Button", "BtnGlobalTest", UIParent)
        HAS_NORMAL = (_G["BtnGlobalTestNormalTexture"] ~= nil)
        HAS_TEXT = (_G["BtnGlobalTestText"] ~= nil)
    "#,
    );
    t.assert_lua_true(
        "return not HAS_NORMAL",
        "NormalTexture should NOT be a global on fresh button",
    );
    t.assert_lua_true(
        "return not HAS_TEXT",
        "Text should NOT be a global on fresh button",
    );
}

#[test]
fn test_preexisting_global_frames() {
    let env = WowLuaEnv::new().unwrap();
    assert!(
        env.eval::<bool>("return UIParent ~= nil").unwrap(),
        "UIParent"
    );
    assert!(
        env.eval::<bool>("return WorldFrame ~= nil").unwrap(),
        "WorldFrame"
    );
    assert!(
        env.eval::<bool>("return Minimap ~= nil").unwrap(),
        "Minimap"
    );
    assert!(
        env.eval::<bool>("return UIParent:GetName() == 'UIParent'")
            .unwrap()
    );
}

#[test]
fn test_world_frame_object_type() {
    let env = WowLuaEnv::new().unwrap();
    // WorldFrame reports its type as "Frame" (WoW quirk), but IsObjectType("Frame") is false
    assert!(
        env.eval::<bool>("return WorldFrame:GetObjectType() == 'Frame'")
            .unwrap(),
        "WorldFrame:GetObjectType() should return 'Frame'",
    );
    // WorldFrame:IsObjectType("WorldFrame") should return true
    assert!(
        env.eval::<bool>("return WorldFrame:IsObjectType('WorldFrame')")
            .unwrap(),
        "WorldFrame:IsObjectType('WorldFrame') should return true",
    );
    // WorldFrame:IsObjectType("Frame") should return false (special case, unlike other frames)
    assert!(
        !env.eval::<bool>("return WorldFrame:IsObjectType('Frame')")
            .unwrap(),
        "WorldFrame:IsObjectType('Frame') should return false",
    );
    // WorldFrame:IsObjectType("Region") should return true (it is a Region)
    assert!(
        env.eval::<bool>("return WorldFrame:IsObjectType('Region')")
            .unwrap(),
        "WorldFrame:IsObjectType('Region') should return true",
    );
}

#[test]
fn test_create_frame_has_owner_addon() {
    let (t, _) = load_test_lua(
        "test-owner-addon",
        r#"
        local f = CreateFrame("Frame", "OwnerTestFrame", UIParent)
    "#,
    );
    let state = t.env.state().borrow();
    let id = state.widgets.get_id_by_name("OwnerTestFrame").unwrap();
    let frame = state.widgets.get(id).unwrap();
    assert!(
        frame.owner_addon.is_some(),
        "frame should have owner_addon set"
    );
    let addon = &state.addons[frame.owner_addon.unwrap() as usize];
    assert_eq!(addon.folder_name, "TestAddon");
}

#[test]
fn test_child_inherits_owner_from_parent() {
    let (t, _) = load_test_lua(
        "test-owner-inherit",
        r#"
        local parent = CreateFrame("Frame", "OwnerParent", UIParent)
        local child = CreateFrame("Frame", "OwnerChild", OwnerParent)
    "#,
    );
    let state = t.env.state().borrow();
    let parent_id = state.widgets.get_id_by_name("OwnerParent").unwrap();
    let child_id = state.widgets.get_id_by_name("OwnerChild").unwrap();
    let parent = state.widgets.get(parent_id).unwrap();
    let child = state.widgets.get(child_id).unwrap();
    assert_eq!(
        parent.owner_addon, child.owner_addon,
        "child should inherit parent's owner"
    );
}

#[test]
fn test_builtin_frames_have_owner() {
    let env = WowLuaEnv::new().unwrap();
    let state = env.state().borrow();
    let ui_parent_id = state.widgets.get_id_by_name("UIParent").unwrap();
    let frame = state.widgets.get(ui_parent_id).unwrap();
    assert!(
        frame.owner_addon.is_some(),
        "UIParent should have owner_addon"
    );
    let addon = &state.addons[frame.owner_addon.unwrap() as usize];
    assert_eq!(addon.folder_name, "__BuiltIn");
}

#[test]
fn test_get_source_location_uses_owner_addon_folder() {
    let (t, _) = load_test_lua(
        "test-source-location-addon",
        r#"
        local f = CreateFrame("Frame", "SourceLocationFrame", UIParent)
        SOURCE = f:GetSourceLocation()
    "#,
    );

    let source: String = t.env.eval("return SOURCE").unwrap();
    assert_eq!(source, "Interface/AddOns/TestAddon");
}

#[test]
fn test_get_source_location_builtin_frames_use_builtin_bucket() {
    let env = WowLuaEnv::new().unwrap();
    let source: String = env.eval("return UIParent:GetSourceLocation()").unwrap();
    assert_eq!(source, "Interface/FrameXML");
}

#[test]
fn test_global_nil_for_nonexistent_frame() {
    let env = WowLuaEnv::new().unwrap();
    assert!(
        env.eval::<bool>("return _G['NoSuchFrameEver'] == nil")
            .unwrap(),
        "nonexistent frame name should be nil in _G",
    );
    assert!(
        env.eval::<bool>("return type(NoSuchFrameEver) == 'nil'")
            .unwrap(),
        "nonexistent bare name should be nil",
    );
}
