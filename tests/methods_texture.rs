//! Tests for texture-related Lua methods (methods_texture.rs).
//!
//! Covers: SetTexture, GetTexture, SetTexCoord, SetVertexColor, GetVertexColor,
//! SetColorTexture, SetAtlas, GetAtlas, SetBlendMode, GetBlendMode,
//! SetHorizTile, GetHorizTile, SetVertTile, GetVertTile, SetDrawLayer, GetDrawLayer,
//! SetDesaturated, IsDesaturated, mask textures, pixel grid, texel snapping,
//! and nine-slice stub methods.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// SetTexture / GetTexture
// ============================================================================

#[test]
fn test_set_get_texture_path() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TexPathFrame", UIParent)
        local tex = frame:CreateTexture("TexPathTex", "BACKGROUND")
        tex:SetTexture("Interface\\Buttons\\UI-Panel-Button-Up")
    "#,
    )
    .unwrap();

    let path: String = env.eval("return TexPathTex:GetTexture()").unwrap();
    assert_eq!(path, "Interface\\Buttons\\UI-Panel-Button-Up");
}

#[test]
fn test_set_texture_nil_clears() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TexNilFrame", UIParent)
        local tex = frame:CreateTexture("TexNilTex", "BACKGROUND")
        tex:SetTexture("Interface\\Buttons\\UI-Panel-Button-Up")
        tex:SetTexture(nil)
    "#,
    )
    .unwrap();

    let is_nil: bool = env.eval("return TexNilTex:GetTexture() == nil").unwrap();
    assert!(is_nil, "SetTexture(nil) should clear the texture path");
}

#[test]
fn test_get_texture_default_nil() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TexDefFrame", UIParent)
        local tex = frame:CreateTexture("TexDefTex", "BACKGROUND")
    "#,
    )
    .unwrap();

    let is_nil: bool = env.eval("return TexDefTex:GetTexture() == nil").unwrap();
    assert!(is_nil, "Texture path should be nil by default");
}

// ============================================================================
// SetVertexColor / GetVertexColor
// ============================================================================

#[test]
fn test_set_get_vertex_color() {
    let env = env();
    let (r, g, b, a): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "VCFrame", UIParent)
            local tex = frame:CreateTexture("VCTex", "BACKGROUND")
            tex:SetVertexColor(0.5, 0.6, 0.7, 0.8)
            return tex:GetVertexColor()
            "#,
        )
        .unwrap();
    assert!((r - 0.5).abs() < 0.001);
    assert!((g - 0.6).abs() < 0.001);
    assert!((b - 0.7).abs() < 0.001);
    assert!((a - 0.8).abs() < 0.001);
}

#[test]
fn test_vertex_color_default_alpha() {
    let env = env();
    let (r, g, b, a): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "VCDefFrame", UIParent)
            local tex = frame:CreateTexture("VCDefTex", "BACKGROUND")
            tex:SetVertexColor(0.1, 0.2, 0.3)
            return tex:GetVertexColor()
            "#,
        )
        .unwrap();
    assert!((r - 0.1).abs() < 0.001);
    assert!((g - 0.2).abs() < 0.001);
    assert!((b - 0.3).abs() < 0.001);
    assert!((a - 1.0).abs() < 0.001, "Alpha should default to 1.0");
}

#[test]
fn test_vertex_color_default_white() {
    let env = env();
    let (r, g, b, a): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "VCWhiteFrame", UIParent)
            local tex = frame:CreateTexture("VCWhiteTex", "BACKGROUND")
            return tex:GetVertexColor()
            "#,
        )
        .unwrap();
    assert_eq!(r, 1.0);
    assert_eq!(g, 1.0);
    assert_eq!(b, 1.0);
    assert_eq!(a, 1.0);
}

// ============================================================================
// SetColorTexture
// ============================================================================

#[test]
fn test_set_color_texture() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "CTFrame", UIParent)
        local tex = frame:CreateTexture("CTTex", "BACKGROUND")
        tex:SetTexture("Interface\\Buttons\\something")
        tex:SetColorTexture(1, 0, 0, 0.5)
    "#,
    )
    .unwrap();

    // SetColorTexture should clear the file texture
    let is_nil: bool = env.eval("return CTTex:GetTexture() == nil").unwrap();
    assert!(is_nil, "SetColorTexture should clear file texture");

    // Verify via Rust state that color_texture is set
    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("CTTex").unwrap();
    let widget = state.widgets.get(id).unwrap();
    let color = widget.color_texture.unwrap();
    assert!((color.r - 1.0).abs() < 0.001);
    assert!((color.g - 0.0).abs() < 0.001);
    assert!((color.b - 0.0).abs() < 0.001);
    assert!((color.a - 0.5).abs() < 0.001);
}

#[test]
fn test_set_color_texture_default_alpha() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "CTDefFrame", UIParent)
        local tex = frame:CreateTexture("CTDefTex", "BACKGROUND")
        tex:SetColorTexture(0.2, 0.3, 0.4)
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("CTDefTex").unwrap();
    let widget = state.widgets.get(id).unwrap();
    let color = widget.color_texture.unwrap();
    assert!((color.a - 1.0).abs() < 0.001, "Alpha should default to 1.0");
}

// ============================================================================
// SetTexCoord / atlas-relative tex coords
// ============================================================================

#[test]
fn test_set_tex_coord_basic() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TCFrame", UIParent)
        local tex = frame:CreateTexture("TCTex", "BACKGROUND")
        tex:SetTexCoord(0.1, 0.9, 0.2, 0.8)
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("TCTex").unwrap();
    let widget = state.widgets.get(id).unwrap();
    let coords = widget.tex_coords.unwrap();
    assert!((coords.0 - 0.1).abs() < 0.001);
    assert!((coords.1 - 0.9).abs() < 0.001);
    assert!((coords.2 - 0.2).abs() < 0.001);
    assert!((coords.3 - 0.8).abs() < 0.001);
}

#[test]
fn test_set_tex_coord_with_atlas_remaps() {
    let env = env();
    // Manually set up atlas_tex_coords via Rust, then call SetTexCoord
    // The atlas sub-region is (0.25, 0.75, 0.1, 0.9)
    // SetTexCoord(0, 1, 0, 1) should produce the atlas coords themselves
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TCAtlasFrame", UIParent)
        local tex = frame:CreateTexture("TCAtlasTex", "BACKGROUND")
    "#,
    )
    .unwrap();

    // Set atlas_tex_coords directly in Rust state
    {
        let mut state = env.state().borrow_mut();
        let id = state.widgets.get_id_by_name("TCAtlasTex").unwrap();
        let widget = state.widgets.get_mut(id).unwrap();
        widget.atlas_tex_coords = Some((0.25, 0.75, 0.1, 0.9));
    }

    // Now call SetTexCoord(0, 1, 0, 1) - should remap to atlas sub-region
    env.exec("TCAtlasTex:SetTexCoord(0, 1, 0, 1)").unwrap();

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("TCAtlasTex").unwrap();
    let widget = state.widgets.get(id).unwrap();
    let coords = widget.tex_coords.unwrap();
    // 0.25 + 0 * 0.5 = 0.25, 0.25 + 1 * 0.5 = 0.75
    // 0.1 + 0 * 0.8 = 0.1, 0.1 + 1 * 0.8 = 0.9
    assert!((coords.0 - 0.25).abs() < 0.001);
    assert!((coords.1 - 0.75).abs() < 0.001);
    assert!((coords.2 - 0.1).abs() < 0.001);
    assert!((coords.3 - 0.9).abs() < 0.001);
}

#[test]
fn test_set_tex_coord_with_atlas_partial() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TCPartialFrame", UIParent)
        local tex = frame:CreateTexture("TCPartialTex", "BACKGROUND")
    "#,
    )
    .unwrap();

    // Atlas region: left=0.0, right=1.0, top=0.0, bottom=1.0
    {
        let mut state = env.state().borrow_mut();
        let id = state.widgets.get_id_by_name("TCPartialTex").unwrap();
        let widget = state.widgets.get_mut(id).unwrap();
        widget.atlas_tex_coords = Some((0.0, 1.0, 0.0, 1.0));
    }

    // SetTexCoord(0.5, 1.0, 0.5, 1.0) - should produce (0.5, 1.0, 0.5, 1.0)
    env.exec("TCPartialTex:SetTexCoord(0.5, 1.0, 0.5, 1.0)")
        .unwrap();

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("TCPartialTex").unwrap();
    let widget = state.widgets.get(id).unwrap();
    let coords = widget.tex_coords.unwrap();
    assert!((coords.0 - 0.5).abs() < 0.001);
    assert!((coords.1 - 1.0).abs() < 0.001);
    assert!((coords.2 - 0.5).abs() < 0.001);
    assert!((coords.3 - 1.0).abs() < 0.001);
}

// ============================================================================
// SetHorizTile / GetHorizTile / SetVertTile / GetVertTile
// ============================================================================

#[test]
fn test_horiz_tile() {
    let env = env();
    let (before, after): (bool, bool) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "HTFrame", UIParent)
            local tex = frame:CreateTexture("HTTex", "BACKGROUND")
            local before = tex:GetHorizTile()
            tex:SetHorizTile(true)
            local after = tex:GetHorizTile()
            return before, after
            "#,
        )
        .unwrap();
    assert!(!before, "HorizTile should default to false");
    assert!(after, "HorizTile should be true after SetHorizTile(true)");
}

#[test]
fn test_vert_tile() {
    let env = env();
    let (before, after): (bool, bool) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "VTFrame", UIParent)
            local tex = frame:CreateTexture("VTTex", "BACKGROUND")
            local before = tex:GetVertTile()
            tex:SetVertTile(true)
            local after = tex:GetVertTile()
            return before, after
            "#,
        )
        .unwrap();
    assert!(!before, "VertTile should default to false");
    assert!(after, "VertTile should be true after SetVertTile(true)");
}

// ============================================================================
// SetBlendMode / GetBlendMode
// ============================================================================

#[test]
fn test_blend_mode_default() {
    let env = env();
    let mode: String = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "BMFrame", UIParent)
            local tex = frame:CreateTexture("BMTex", "BACKGROUND")
            return tex:GetBlendMode()
            "#,
        )
        .unwrap();
    assert_eq!(mode, "BLEND");
}

#[test]
fn test_set_blend_mode_no_error() {
    let env = env();
    // SetBlendMode is a stub - just verify it doesn't error
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "BMSetFrame", UIParent)
        local tex = frame:CreateTexture("BMSetTex", "BACKGROUND")
        tex:SetBlendMode("ADD")
        tex:SetBlendMode("ALPHAKEY")
        tex:SetBlendMode("DISABLE")
        tex:SetBlendMode("MOD")
    "#,
    )
    .unwrap();
}

// ============================================================================
// SetDesaturated / IsDesaturated
// ============================================================================

#[test]
fn test_desaturated_default() {
    let env = env();
    let desat: bool = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "DesatFrame", UIParent)
            local tex = frame:CreateTexture("DesatTex", "BACKGROUND")
            return tex:IsDesaturated()
            "#,
        )
        .unwrap();
    assert!(!desat, "IsDesaturated should default to false");
}

#[test]
fn test_set_desaturated_no_error() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "DesatSetFrame", UIParent)
        local tex = frame:CreateTexture("DesatSetTex", "BACKGROUND")
        tex:SetDesaturated(true)
        tex:SetDesaturated(false)
    "#,
    )
    .unwrap();
}

// ============================================================================
// SetAtlas / GetAtlas
// ============================================================================

#[test]
fn test_set_atlas_known() {
    let env = env();
    // "checkbox-minimal" is a known atlas in the WoW data
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "AtlasFrame", UIParent)
        local tex = frame:CreateTexture("AtlasTex", "BACKGROUND")
        tex:SetAtlas("checkbox-minimal")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("AtlasTex").unwrap();
    let widget = state.widgets.get(id).unwrap();
    assert_eq!(widget.atlas.as_deref(), Some("checkbox-minimal"));
    assert!(widget.texture.is_some(), "Known atlas should set texture path");
    assert!(
        widget.tex_coords.is_some(),
        "Known atlas should set tex_coords"
    );
    assert!(
        widget.atlas_tex_coords.is_some(),
        "Known atlas should set atlas_tex_coords"
    );
}

#[test]
fn test_set_atlas_unknown() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "AtlasUnkFrame", UIParent)
        local tex = frame:CreateTexture("AtlasUnkTex", "BACKGROUND")
        tex:SetAtlas("nonexistent-atlas-name-12345")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("AtlasUnkTex").unwrap();
    let widget = state.widgets.get(id).unwrap();
    assert_eq!(
        widget.atlas.as_deref(),
        Some("nonexistent-atlas-name-12345"),
        "Unknown atlas should still store the name"
    );
    assert!(
        widget.texture.is_none(),
        "Unknown atlas should not set texture path"
    );
}

#[test]
fn test_get_atlas_default_nil() {
    let env = env();
    let is_nil: bool = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "AtlasNilFrame", UIParent)
            local tex = frame:CreateTexture("AtlasNilTex", "BACKGROUND")
            return tex:GetAtlas() == nil
            "#,
        )
        .unwrap();
    assert!(is_nil, "GetAtlas should return nil by default");
}

#[test]
fn test_get_atlas_returns_name() {
    let env = env();
    let name: String = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "AtlasGetFrame", UIParent)
            local tex = frame:CreateTexture("AtlasGetTex", "BACKGROUND")
            tex:SetAtlas("checkbox-minimal")
            return tex:GetAtlas()
            "#,
        )
        .unwrap();
    assert_eq!(name, "checkbox-minimal");
}

// ============================================================================
// SetAtlas - button parent propagation
// ============================================================================

#[test]
fn test_set_atlas_propagates_to_button_normal_texture() {
    let env = env();
    env.exec(
        r#"
        local btn = CreateFrame("Button", "AtlasBtnFrame", UIParent)
        btn:SetSize(30, 30)
        local normalTex = btn.NormalTexture
        normalTex:SetAtlas("checkbox-minimal")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let btn_id = state.widgets.get_id_by_name("AtlasBtnFrame").unwrap();
    let btn = state.widgets.get(btn_id).unwrap();
    assert!(
        btn.normal_texture.is_some(),
        "SetAtlas on NormalTexture child should propagate to parent button"
    );
    assert!(
        btn.normal_tex_coords.is_some(),
        "SetAtlas on NormalTexture child should set parent's normal_tex_coords"
    );
}

// ============================================================================
// Pixel grid and texel snapping stubs
// ============================================================================

#[test]
fn test_snap_to_pixel_grid() {
    let env = env();
    let snap: bool = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "SnapFrame", UIParent)
            local tex = frame:CreateTexture("SnapTex", "BACKGROUND")
            tex:SetSnapToPixelGrid(true)
            return tex:IsSnappingToPixelGrid()
            "#,
        )
        .unwrap();
    // Stub returns false always
    assert!(!snap);
}

#[test]
fn test_texel_snapping_bias() {
    let env = env();
    let bias: f64 = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "BiasFrame", UIParent)
            local tex = frame:CreateTexture("BiasTex", "BACKGROUND")
            tex:SetTexelSnappingBias(0.5)
            return tex:GetTexelSnappingBias()
            "#,
        )
        .unwrap();
    // Stub returns 0.0 always
    assert_eq!(bias, 0.0);
}

// ============================================================================
// Nine-slice stubs
// ============================================================================

#[test]
fn test_nine_slice_margins() {
    let env = env();
    let (l, r, t, b): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "NSFrame", UIParent)
            local tex = frame:CreateTexture("NSTex", "BACKGROUND")
            tex:SetTextureSliceMargins(10, 20, 30, 40)
            return tex:GetTextureSliceMargins()
            "#,
        )
        .unwrap();
    // Stubs return 0
    assert_eq!((l, r, t, b), (0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_nine_slice_mode() {
    let env = env();
    let mode: i32 = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "NSModeFrame", UIParent)
            local tex = frame:CreateTexture("NSModeTex", "BACKGROUND")
            tex:SetTextureSliceMode(1)
            return tex:GetTextureSliceMode()
            "#,
        )
        .unwrap();
    assert_eq!(mode, 0);
}

#[test]
fn test_clear_texture_slice_no_error() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "NSClearFrame", UIParent)
        local tex = frame:CreateTexture("NSClearTex", "BACKGROUND")
        tex:ClearTextureSlice()
    "#,
    )
    .unwrap();
}

// ============================================================================
// Mask textures
// ============================================================================

#[test]
fn test_add_and_remove_mask_texture() {
    let env = env();
    let (added, removed): (i32, i32) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "MaskFrame", UIParent)
            local tex = frame:CreateTexture("MaskTex", "BACKGROUND")
            local mask = frame:CreateMaskTexture("MaskMask", "BACKGROUND")
            tex:AddMaskTexture(mask)
            local added = tex:GetNumMaskTextures()
            tex:RemoveMaskTexture(mask)
            return added, tex:GetNumMaskTextures()
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
            local frame = CreateFrame("Frame", "MaskNoDupFrame", UIParent)
            local tex = frame:CreateTexture("MaskNoDupTex", "BACKGROUND")
            local mask = frame:CreateMaskTexture("MaskNoDupMask", "BACKGROUND")
            tex:AddMaskTexture(mask)
            tex:AddMaskTexture(mask)
            return tex:GetNumMaskTextures()
            "#,
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_get_mask_texture_nil() {
    let env = env();
    let is_nil: bool = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "MaskNilFrame", UIParent)
            local tex = frame:CreateTexture("MaskNilTex", "BACKGROUND")
            return tex:GetMaskTexture(1) == nil
            "#,
        )
        .unwrap();
    assert!(is_nil);
}

// ============================================================================
// SetDrawLayer / GetDrawLayer
// ============================================================================

#[test]
fn test_draw_layer_default() {
    let env = env();
    let (layer, sublayer): (String, i32) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "DLFrame", UIParent)
            local tex = frame:CreateTexture("DLTex", "BACKGROUND")
            return tex:GetDrawLayer()
            "#,
        )
        .unwrap();
    assert_eq!(layer, "ARTWORK");
    assert_eq!(sublayer, 0);
}

#[test]
fn test_set_draw_layer_no_error() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "DLSetFrame", UIParent)
        local tex = frame:CreateTexture("DLSetTex", "BACKGROUND")
        tex:SetDrawLayer("OVERLAY", 2)
        tex:SetDrawLayer("BORDER")
    "#,
    )
    .unwrap();
}

// ============================================================================
// SetGradient / SetCenterColor stubs
// ============================================================================

#[test]
fn test_set_gradient_no_error() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "GradFrame", UIParent)
        local tex = frame:CreateTexture("GradTex", "BACKGROUND")
        tex:SetGradient("HORIZONTAL", {r=1, g=0, b=0, a=1}, {r=0, g=0, b=1, a=1})
    "#,
    )
    .unwrap();
}

#[test]
fn test_set_center_color_no_error() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "CenterFrame", UIParent)
        local tex = frame:CreateTexture("CenterTex", "BACKGROUND")
        tex:SetCenterColor(1, 0, 0, 1)
    "#,
    )
    .unwrap();
}

// ============================================================================
// SetAtlas with useAtlasSize
// ============================================================================

#[test]
fn test_set_atlas_use_atlas_size() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "AtlasSizeFrame", UIParent)
        local tex = frame:CreateTexture("AtlasSizeTex", "BACKGROUND")
        tex:SetAtlas("checkbox-minimal", true)
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("AtlasSizeTex").unwrap();
    let widget = state.widgets.get(id).unwrap();
    // With useAtlasSize=true, dimensions should be set from atlas info
    assert!(
        widget.width > 0.0 || widget.height > 0.0,
        "useAtlasSize=true should set non-zero dimensions from atlas"
    );
}
