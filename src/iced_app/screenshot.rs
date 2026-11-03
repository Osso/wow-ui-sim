//! IPC screenshot rendering for the running app.

use std::path::Path;

use crate::lua_server::Response as LuaResponse;
use crate::render::GlyphAtlas;
use crate::render::headless::{render_scene_to_image, scene_viewport};

use super::app::App;
const CROP_FORMAT_EXAMPLE: &str = "700x150+400+650";

impl App {
    /// Render a screenshot from the live app state and save to disk.
    pub(crate) fn render_screenshot(
        &self,
        output: &str,
        width: u32,
        height: u32,
        filter: Option<&str>,
        crop: Option<&str>,
        png: bool,
    ) -> LuaResponse {
        if width == 0 || height == 0 {
            return LuaResponse::Error("Screenshot dimensions must be positive".to_string());
        }
        let (batch, mut glyph_atlas) = self.build_screenshot_batch(width, height, filter);
        let glyph_data = glyph_atlas_data(&mut glyph_atlas);
        let texture_manager = self.env.borrow().state().borrow().texture_manager();
        let mut tex_mgr = texture_manager.borrow_mut();
        let img = render_scene_to_image(
            &batch,
            &mut tex_mgr,
            width,
            height,
            glyph_data,
            self.fixed_canvas_size(),
        );
        let img = match maybe_crop_image(img, crop) {
            Ok(img) => img,
            Err(e) => return LuaResponse::Error(e),
        };

        let output_path =
            match crate::screenshot_output::save_screenshot(&img, Path::new(output), png, 65.0) {
                Ok(path) => path,
                Err(e) => return LuaResponse::Error(format!("Failed to save screenshot: {e}")),
            };

        LuaResponse::Output(format_screenshot_saved_message(
            &img,
            &output_path,
            width,
            height,
            crop.is_some(),
        ))
    }

    pub(super) fn build_screenshot_batch(
        &self,
        width: u32,
        height: u32,
        filter: Option<&str>,
    ) -> (crate::render::QuadBatch, GlyphAtlas) {
        let scene_size = self.fixed_canvas_size();
        let layout = scene_size.unwrap_or(iced::Size::new(width as f32, height as f32));
        let raster_scale = scene_size.map_or(1.0, |size| {
            scene_viewport(width, height, scene_size).height as f32 / size.height
        });
        let mut glyph_atlas = GlyphAtlas::with_raster_scale(raster_scale);
        let mut batch = {
            let env = self.env.borrow();
            let mut fs = self.font_system.borrow_mut();
            let buckets = {
                let mut state = env.state().borrow_mut();
                super::tooltip::update_tooltip_sizes(&mut state, &mut fs);
                state.ensure_layout_rects();
                let _ = state.get_strata_buckets();
                state.strata_buckets.as_ref().unwrap().clone()
            };
            let state = env.state().borrow();
            let tooltip_data = super::tooltip::collect_tooltip_data(&state);
            super::build_quad_batch_for_registry_with_quest_blobs(
                super::RegistryQuadBatchParams::new(
                    &state.widgets,
                    (layout.width, layout.height),
                    &buckets,
                )
                .root_name(filter)
                .pressed_frame(self.pressed_frame)
                .hovered_frame(self.hovered_frame)
                .text_ctx(Some((&mut fs, &mut glyph_atlas)))
                .message_frames(Some(&state.message_frames))
                .tooltip_data(Some(&tooltip_data))
                .quest_blobs(Some(&state.quest_blobs)),
            )
        };
        batch.append(&self.build_overlay());
        (batch, glyph_atlas)
    }
}

fn glyph_atlas_data(glyph_atlas: &mut GlyphAtlas) -> Option<(&[u8], u32)> {
    if glyph_atlas.is_dirty() {
        let (data, size, _) = glyph_atlas.texture_data();
        Some((data, size))
    } else {
        None
    }
}

fn maybe_crop_image(img: image::RgbaImage, crop: Option<&str>) -> Result<image::RgbaImage, String> {
    match crop {
        Some(crop_str) => apply_crop(img, crop_str),
        None => Ok(img),
    }
}

fn format_screenshot_saved_message(
    img: &image::RgbaImage,
    output_path: &Path,
    width: u32,
    height: u32,
    cropped: bool,
) -> String {
    let size_label = if cropped {
        format!(
            "{}x{} (cropped from {}x{})",
            img.width(),
            img.height(),
            width,
            height
        )
    } else {
        format!("{}x{}", width, height)
    };
    format!(
        "Saved {} screenshot to {}",
        size_label,
        output_path.display()
    )
}

/// Parse a crop string in WxH+X+Y format (e.g., "700x150+400+650").
/// Returns (width, height, x, y) or None if the format is invalid.
fn parse_crop(s: &str) -> Option<(u32, u32, u32, u32)> {
    let (dims, rest) = s.split_once('+')?;
    let (x_str, y_str) = rest.split_once('+')?;
    let (w_str, h_str) = dims.split_once('x')?;
    let w = w_str.parse().ok()?;
    let h = h_str.parse().ok()?;
    let x = x_str.parse().ok()?;
    let y = y_str.parse().ok()?;
    Some((w, h, x, y))
}

/// Apply crop to an image, returning an error string on invalid input.
fn apply_crop(img: image::RgbaImage, crop_str: &str) -> Result<image::RgbaImage, String> {
    use image::GenericImageView;
    let (cw, ch, cx, cy) = parse_crop(crop_str).ok_or_else(|| {
        format!("Invalid crop format '{crop_str}', expected WxH+X+Y (e.g., {CROP_FORMAT_EXAMPLE})")
    })?;
    if cx + cw > img.width() || cy + ch > img.height() {
        return Err(format!(
            "Crop region {}x{}+{}+{} exceeds image bounds {}x{}",
            cw,
            ch,
            cx,
            cy,
            img.width(),
            img.height()
        ));
    }
    Ok(img.view(cx, cy, cw, ch).to_image())
}
