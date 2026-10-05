use super::{env, setup_texture};

// ============================================================================
// Mask textures
// ============================================================================

#[test]
fn test_add_and_remove_mask_texture() {
    let env = env();
    let (added, removed): (i32, i32) = env
        .eval(
            r#"
        local f = CreateFrame("Frame", nil, UIParent)
        local tex = f:CreateTexture(nil, "BACKGROUND")
        local mask = f:CreateMaskTexture(nil, "BACKGROUND")
        tex:AddMaskTexture(mask)
        local a = tex:GetNumMaskTextures()
        tex:RemoveMaskTexture(mask)
        return a, tex:GetNumMaskTextures()
    "#,
        )
        .unwrap();
    assert_eq!(added, 1);
    assert_eq!(removed, 0);
}

#[test]
fn test_add_mask_texture_no_duplicates() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
        local f = CreateFrame("Frame", nil, UIParent)
        local tex = f:CreateTexture(nil, "BACKGROUND")
        local mask = f:CreateMaskTexture(nil, "BACKGROUND")
        tex:AddMaskTexture(mask); tex:AddMaskTexture(mask)
        return tex:GetNumMaskTextures()
    "#,
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_named_mask_texture_publishes_global() {
    let env = env();
    let published: bool = env
        .eval(
            r#"
        local f = CreateFrame("Frame", nil, UIParent)
        local mask = f:CreateMaskTexture("NamedMaskTextureGlobal", "BACKGROUND")
        return NamedMaskTextureGlobal == mask
    "#,
        )
        .unwrap();

    assert!(
        published,
        "named MaskTexture regions should publish globals"
    );
}

#[test]
fn test_get_mask_texture_nil() {
    let env = env();
    let (_, tex) = setup_texture(&env, "MaskNil");
    let is_nil: bool = env
        .eval(&format!("return {tex}:GetMaskTexture(1) == nil"))
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_set_mask_creates_mask_texture() {
    let env = env();
    let (_, tex) = setup_texture(&env, "SetMask");
    let (mask_count, mask_path): (i32, String) = env
        .eval(&format!(
            r#"
        {tex}:SetMask("Interface\\CharacterFrame\\TempPortraitAlphaMask")
        local mask = {tex}:GetMaskTexture(1)
        return {tex}:GetNumMaskTextures(), mask and mask:GetTexture() or ""
    "#
        ))
        .unwrap();

    assert_eq!(mask_count, 1);
    assert_eq!(
        mask_path,
        "Interface\\CharacterFrame\\TempPortraitAlphaMask"
    );
}

// ============================================================================
// SetPortraitToTexture
// ============================================================================

#[cfg(not(feature = "retail-12-0-0"))]
#[test]
fn test_set_portrait_to_texture_applies_circle_mask() {
    let env = env();
    let mask_count: i32 = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "PortraitMaskFrame", UIParent)
            local tex = frame:CreateTexture("PortraitMaskTex", "BORDER")
            SetPortraitToTexture(tex, "Interface\\Icons\\Ability_Mount_RidingHorse")
            return tex:GetNumMaskTextures()
            "#,
        )
        .unwrap();
    assert_eq!(
        mask_count, 1,
        "SetPortraitToTexture should apply a circular mask"
    );
}

#[cfg(not(feature = "retail-12-0-0"))]
#[test]
fn test_set_portrait_to_texture_no_double_mask() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "PortraitNoDoubleFrame", UIParent)
            local tex = frame:CreateTexture("PortraitNoDoubleTex", "BORDER")
            SetPortraitToTexture(tex, "Interface\\Icons\\Ability_Mount_RidingHorse")
            SetPortraitToTexture(tex, "Interface\\Icons\\INV_Misc_QuestionMark")
            return tex:GetNumMaskTextures()
            "#,
        )
        .unwrap();
    assert_eq!(
        count, 1,
        "calling SetPortraitToTexture twice should not add a second mask"
    );
}

// ============================================================================
// SetDrawLayer / GetDrawLayer
// ============================================================================

#[test]
fn test_draw_layer_default() {
    let env = env();
    let (_, tex) = setup_texture(&env, "DL");
    let (layer, sublayer): (String, i32) =
        env.eval(&format!("return {tex}:GetDrawLayer()")).unwrap();
    assert_eq!(layer, "BACKGROUND");
    assert_eq!(sublayer, 0);
}

#[test]
fn test_set_draw_layer_no_error() {
    let env = env();
    let (_, tex) = setup_texture(&env, "DLSet");
    env.exec(&format!(
        r#"{tex}:SetDrawLayer("OVERLAY", 2); {tex}:SetDrawLayer("BORDER")"#
    ))
    .unwrap();
}

// ============================================================================
// SetGradient / SetCenterColor stubs
// ============================================================================

#[test]
fn test_set_gradient_no_error() {
    let env = env();
    let (_, tex) = setup_texture(&env, "Grad");
    env.exec(&format!(
        r#"{tex}:SetGradient("HORIZONTAL", {{r=1, g=0, b=0, a=1}}, {{r=0, g=0, b=1, a=1}})"#
    ))
    .unwrap();
}

#[test]
fn test_set_center_color_no_error() {
    let env = env();
    let (_, tex) = setup_texture(&env, "Center");
    env.exec(&format!("{tex}:SetCenterColor(1, 0, 0, 1)"))
        .unwrap();
}

// ============================================================================
// SetAtlas with useAtlasSize
// ============================================================================

#[test]
fn test_set_atlas_use_atlas_size() {
    let env = env();
    let (_, tex) = setup_texture(&env, "AtlasSize");
    env.exec(&format!(r#"{tex}:SetAtlas("checkbox-minimal", true)"#))
        .unwrap();

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name(&tex).unwrap();
    let widget = state.widgets.get(id).unwrap();
    assert!(
        widget.width > 0.0 || widget.height > 0.0,
        "useAtlasSize=true should set non-zero dimensions from atlas"
    );
}
