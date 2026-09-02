//! `SetAtlas` on a texture parented to a Button used to force the child's
//! shown state for every parentKey, not only the six state slots. The bag
//! bar's `SlotHighlightTexture` is declared hidden and receives its atlas from
//! `UpdateTextures` at load, so all six bag slots rendered a solid highlight
//! disc with no bag open.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn set_atlas_leaves_a_custom_parent_key_child_hidden() {
    let env = WowLuaEnv::new().expect("env");
    let (custom_shown, slot_shown): (bool, bool) = env
        .eval(
            r#"
            local button = CreateFrame("Button", nil, UIParent)
            button:SetSize(48, 48)
            local highlight = button:CreateTexture(nil, "OVERLAY")
            highlight:SetAllPoints()
            highlight:Hide()
            button.SlotHighlightTexture = highlight
            highlight:SetAtlas("bag-main-highlight")

            local normal = button:CreateTexture(nil, "ARTWORK")
            normal:Hide()
            button.NormalTexture = normal
            normal:SetAtlas("bag-main")
            return highlight:IsShown(), normal:IsShown()
            "#,
        )
        .expect("atlas probe");
    assert!(!custom_shown, "a hidden child with a custom parentKey stays hidden after SetAtlas");
    assert!(slot_shown, "the NormalTexture slot follows the button state (enabled, normal)");
}
