//! Tests for _G frame access behavior.
//!
//! Frames are eagerly registered in _G via raw_set at creation time.

use super::*;

#[test]
fn test_create_frame_named_sets_global() {
    let (t, _) = load_test_lua("test-g-named", r#"
        local f = CreateFrame("Frame", "GlobalTestFrame", UIParent)
        GLOBAL_LOOKUP_OK = (_G["GlobalTestFrame"] == f)
        BARE_LOOKUP_OK = (GlobalTestFrame == f)
    "#);
    t.assert_lua_true("return GLOBAL_LOOKUP_OK", "named frame should be in _G");
    t.assert_lua_true("return BARE_LOOKUP_OK", "named frame accessible as bare global");
}

#[test]
fn test_create_frame_unnamed_not_in_globals() {
    let (t, _) = load_test_lua("test-g-unnamed", r#"
        local f = CreateFrame("Frame", nil, UIParent)
        -- Unnamed frame should not pollute _G with any user-visible name
        UNNAMED_OK = true
        for k, v in pairs(_G) do
            if v == f and not tostring(k):find("^__") then
                UNNAMED_OK = false
            end
        end
    "#);
    t.assert_lua_true(
        "return UNNAMED_OK",
        "unnamed frame should not appear in _G under user-visible keys",
    );
}

#[test]
fn test_create_frame_returns_functional_handle() {
    let (t, _) = load_test_lua("test-g-handle", r#"
        local f = CreateFrame("Frame", "HandleTestFrame", UIParent)
        f:SetSize(123, 456)
        W = f:GetWidth()
        H = f:GetHeight()
        NAME = f:GetName()
    "#);
    assert_eq!(t.env.eval::<f64>("return W").unwrap(), 123.0);
    assert_eq!(t.env.eval::<f64>("return H").unwrap(), 456.0);
    t.assert_lua_str("return NAME", "HandleTestFrame");
}

#[test]
fn test_global_overwritten_by_recreate() {
    let (t, _) = load_test_lua("test-g-overwrite", r#"
        local f1 = CreateFrame("Frame", "OverwriteFrame", UIParent)
        f1:SetSize(100, 100)
        local f2 = CreateFrame("Frame", "OverwriteFrame", UIParent)
        f2:SetSize(200, 200)
        GLOBAL_IS_F2 = (_G["OverwriteFrame"] == f2)
        F2_WIDTH = f2:GetWidth()
    "#);
    t.assert_lua_true("return GLOBAL_IS_F2", "_G should point to the second frame");
    assert_eq!(t.env.eval::<f64>("return F2_WIDTH").unwrap(), 200.0);
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
    t.assert_lua_true("return XMLGlobalFrame ~= nil", "XML frame should be a global");
    t.assert_lua_true(
        "return _G['XMLGlobalFrame'] == XMLGlobalFrame",
        "_G lookup should match bare name lookup",
    );
    assert_eq!(
        t.env.eval::<f64>("return XMLGlobalFrame:GetWidth()").unwrap(),
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
fn test_button_child_globals() {
    let (t, _) = load_test_lua("test-g-btn-children", r#"
        local btn = CreateFrame("Button", "BtnGlobalTest", UIParent)
        HAS_NORMAL = (_G["BtnGlobalTestNormalTexture"] ~= nil)
        HAS_PUSHED = (_G["BtnGlobalTestPushedTexture"] ~= nil)
        HAS_HIGHLIGHT = (_G["BtnGlobalTestHighlightTexture"] ~= nil)
        HAS_DISABLED = (_G["BtnGlobalTestDisabledTexture"] ~= nil)
        HAS_TEXT = (_G["BtnGlobalTestText"] ~= nil)
    "#);
    t.assert_lua_true("return HAS_NORMAL", "NormalTexture should be a global");
    t.assert_lua_true("return HAS_PUSHED", "PushedTexture should be a global");
    t.assert_lua_true("return HAS_HIGHLIGHT", "HighlightTexture should be a global");
    t.assert_lua_true("return HAS_DISABLED", "DisabledTexture should be a global");
    t.assert_lua_true("return HAS_TEXT", "Text should be a global");
}

#[test]
fn test_preexisting_global_frames() {
    let env = WowLuaEnv::new().unwrap();
    assert!(env.eval::<bool>("return UIParent ~= nil").unwrap(), "UIParent");
    assert!(env.eval::<bool>("return WorldFrame ~= nil").unwrap(), "WorldFrame");
    assert!(env.eval::<bool>("return Minimap ~= nil").unwrap(), "Minimap");
    assert!(env.eval::<bool>("return UIParent:GetName() == 'UIParent'").unwrap());
}

#[test]
fn test_global_nil_for_nonexistent_frame() {
    let env = WowLuaEnv::new().unwrap();
    assert!(
        env.eval::<bool>("return _G['NoSuchFrameEver'] == nil").unwrap(),
        "nonexistent frame name should be nil in _G",
    );
    assert!(
        env.eval::<bool>("return type(NoSuchFrameEver) == 'nil'").unwrap(),
        "nonexistent bare name should be nil",
    );
}
