//! A FontString shadow with `SetShadowOffset(2, -2)` sits two units right and
//! two units BELOW the glyphs in the client (WoW's y axis points up). The quad
//! builders work in screen pixels with y down; passing the offset through
//! unchanged drew the shadow above the text, so the bottom edge of every
//! string lost its shadow, and the offset was never scaled with the UI.
#![cfg(feature = "gui")]

use crate::common;

use std::cell::RefCell;
use std::rc::Rc;
use wow_ui_sim::iced_app::{RegistryQuadBatchParams, build_quad_batch_for_registry};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::render::headless::render_to_image;
use wow_ui_sim::render::{GlyphAtlas, QuadBatch, WowFontSystem};
use wow_ui_sim::texture::TextureManager;

/// Build the batch with a live glyph atlas, the way the screenshot command
/// does; the atlas pixels have to reach the renderer or text draws nothing.
fn batch_with_glyphs(env: &WowLuaEnv, width: u32, height: u32, root: &str) -> (QuadBatch, GlyphAtlas) {
    let font_system = Rc::new(RefCell::new(WowFontSystem::new()));
    env.set_font_system(Rc::clone(&font_system));
    let mut glyph_atlas = GlyphAtlas::new();
    let mut font_system = font_system.borrow_mut();
    let buckets = {
        let mut state = env.state().borrow_mut();
        state.ensure_layout_rects();
        let _ = state.get_strata_buckets();
        state.strata_buckets.as_ref().unwrap().clone()
    };
    let state = env.state().borrow();
    let batch = build_quad_batch_for_registry(
        RegistryQuadBatchParams::new(&state.widgets, (width as f32, height as f32), &buckets)
            .root_name(Some(root))
            .text_ctx(Some((&mut font_system, &mut glyph_atlas))),
    );
    drop(state);
    drop(font_system);
    (batch, glyph_atlas)
}

/// Rows of the image that hold white (glyph) and dark (shadow) pixels.
fn ink_rows(image: &image::RgbaImage) -> (Vec<u32>, Vec<u32>) {
    let mut white = Vec::new();
    let mut dark = Vec::new();
    for y in 0..image.height() {
        let mut has_white = false;
        let mut has_dark = false;
        for x in 0..image.width() {
            let p = image.get_pixel(x, y).0;
            if p[0] > 200 && p[1] > 200 && p[2] > 200 {
                has_white = true;
            }
            if p[0] < 100 && p[1] < 100 && p[2] < 100 {
                has_dark = true;
            }
        }
        if has_white {
            white.push(y);
        }
        if has_dark {
            dark.push(y);
        }
    }
    (white, dark)
}

#[test]
fn text_shadow_falls_below_the_glyphs() {
    if common::try_create_gpu_device().is_none() {
        eprintln!("Skipping GPU text shadow test: no adapter available");
        return;
    }
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let (width, height) = (240u32, 80u32);
    env.set_screen_size(width as f32, height as f32);
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "ShadowHarness", UIParent)
        frame:SetAllPoints(UIParent)
        local bg = frame:CreateTexture(nil, "BACKGROUND")
        bg:SetAllPoints()
        bg:SetColorTexture(0.5, 0.5, 0.5, 1)
        local text = frame:CreateFontString(nil, "OVERLAY")
        text:SetFont("Fonts\\FRIZQT__.TTF", 32, "")
        text:SetPoint("TOPLEFT", frame, "TOPLEFT", 10, -10)
        text:SetText("HxH")
        text:SetTextColor(1, 1, 1, 1)
        text:SetShadowColor(0, 0, 0, 1)
        text:SetShadowOffset(2, -2)
        "#,
    )
    .expect("failed to build shadow harness");

    let (batch, glyph_atlas) = batch_with_glyphs(&env, width, height, "ShadowHarness");
    let (glyph_pixels, glyph_size, _) = glyph_atlas.texture_data();
    let mut tex_mgr = TextureManager::new();
    let rendered = render_to_image(&batch, &mut tex_mgr, width, height, Some((glyph_pixels, glyph_size)));
    let (white, dark) = ink_rows(&rendered);
    let (white_top, white_bottom) = (*white.first().expect("glyph rows"), *white.last().expect("glyph rows"));
    let (dark_top, dark_bottom) = (*dark.first().expect("shadow rows"), *dark.last().expect("shadow rows"));
    assert!(
        dark_bottom > white_bottom,
        "the shadow must reach below the glyphs: glyph rows {white_top}..{white_bottom}, shadow rows {dark_top}..{dark_bottom}"
    );
    assert!(
        dark_top >= white_top,
        "the shadow must not rise above the glyphs: glyph rows {white_top}..{white_bottom}, shadow rows {dark_top}..{dark_bottom}"
    );
}
