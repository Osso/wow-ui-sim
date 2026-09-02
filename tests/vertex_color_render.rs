//! Vertex colours (SetColorTexture, SetVertexColor, font colours) are sRGB
//! values. Atlas samples are decoded to linear by their texture format and the
//! render target re-encodes on write, so a vertex colour that skips the decode
//! comes out one sRGB encode too bright: a solid 0.5 landed at 188/255 where
//! 128 is right, and every tint and glyph with it.
#![cfg(feature = "gui")]

use crate::common;
#[path = "render_order_support.rs"]
mod render_order_support;

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::render::brightness_boost_divisor;
use wow_ui_sim::render::headless::render_to_image;

/// The value the shader must produce for a solid sRGB grey `v`: decode, apply
/// the brightness lift the process is configured with, encode again.
fn expected_grey(v: f32) -> u8 {
    let linear = if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    };
    let lifted = linear.powf(1.0 / brightness_boost_divisor());
    let encoded = if lifted <= 0.0031308 {
        lifted * 12.92
    } else {
        1.055 * lifted.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round() as u8
}

#[test]
fn solid_colour_textures_render_the_value_lua_set() {
    if common::try_create_gpu_device().is_none() {
        eprintln!("Skipping GPU vertex colour render test: no adapter available");
        return;
    }

    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let (width, height) = (160u32, 80u32);
    env.set_screen_size(width as f32, height as f32);
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "VertexColourHarness", UIParent)
        frame:SetAllPoints(UIParent)
        local half = frame:CreateTexture(nil, "ARTWORK")
        half:SetSize(60, 60)
        half:SetPoint("TOPLEFT", frame, "TOPLEFT", 10, -10)
        half:SetColorTexture(0.5, 0.5, 0.5, 1)
        local quarter = frame:CreateTexture(nil, "ARTWORK")
        quarter:SetSize(60, 60)
        quarter:SetPoint("TOPLEFT", frame, "TOPLEFT", 90, -10)
        quarter:SetColorTexture(0.25, 0.25, 0.25, 1)
        "#,
    )
    .expect("failed to build vertex colour harness");

    let mut tex_mgr = render_order_support::make_texture_manager();
    let batch = render_order_support::build_screenshot_like_batch(
        &env,
        width,
        height,
        Some("VertexColourHarness"),
    );
    let rendered = render_to_image(&batch, &mut tex_mgr, width, height, None);

    for (x, value) in [(40u32, 0.5f32), (120u32, 0.25f32)] {
        let pixel = rendered.get_pixel(x, 40).0;
        let expected = expected_grey(value);
        for channel in 0..3 {
            assert!(
                pixel[channel].abs_diff(expected) <= 2,
                "SetColorTexture({value}) rendered {pixel:?}, expected {expected} per channel \
                 (a value one sRGB encode too bright means the vertex colour skipped the decode)"
            );
        }
    }
}
