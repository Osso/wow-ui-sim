//! IPC screenshot rendering for the running app.

use std::path::Path;

use crate::lua_server::Response as LuaResponse;
use crate::render::headless::render_to_image;
use crate::render::GlyphAtlas;

use super::app::App;
use super::render::build_quad_batch_for_registry;

impl App {
    /// Render a screenshot from the live app state and save to disk.
    pub(crate) fn render_screenshot(
        &self,
        output: &str,
        width: u32,
        height: u32,
        filter: Option<&str>,
        crop: Option<&str>,
    ) -> LuaResponse {
        let output_path = Path::new(output).with_extension("webp");

        let mut glyph_atlas = GlyphAtlas::new();
        let batch = {
            let env = self.env.borrow();
            let mut fs = self.font_system.borrow_mut();
            let buckets = {
                let mut state = env.state().borrow_mut();
                super::tooltip::update_tooltip_sizes(&mut state, &mut fs);
                let _ = state.get_strata_buckets();
                state.strata_buckets.as_ref().unwrap().clone()
            };
            let state = env.state().borrow();
            let tooltip_data = super::tooltip::collect_tooltip_data(&state);
            build_quad_batch_for_registry(
                &state.widgets,
                (width as f32, height as f32),
                filter,
                self.pressed_frame,
                self.hovered_frame,
                Some((&mut fs, &mut glyph_atlas)),
                Some(&state.message_frames),
                Some(&tooltip_data),
                &buckets,
            )
        };

        let glyph_data = if glyph_atlas.is_dirty() {
            let (data, size, _) = glyph_atlas.texture_data();
            Some((data, size))
        } else {
            None
        };

        let mut tex_mgr = self.texture_manager.borrow_mut();
        let img = render_to_image(&batch, &mut tex_mgr, width, height, glyph_data);

        let img = match crop {
            Some(crop_str) => match apply_crop(img, crop_str) {
                Ok(cropped) => cropped,
                Err(e) => return LuaResponse::Error(e),
            },
            None => img,
        };

        if let Err(e) = save_screenshot(&img, &output_path) {
            return LuaResponse::Error(format!("Failed to save screenshot: {}", e));
        }

        let size_label = if crop.is_some() {
            format!("{}x{} (cropped from {}x{})", img.width(), img.height(), width, height)
        } else {
            format!("{}x{}", width, height)
        };
        LuaResponse::Output(format!(
            "Saved {} screenshot to {}",
            size_label, output_path.display()
        ))
    }
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
        format!("Invalid crop format '{}', expected WxH+X+Y (e.g., 700x150+400+650)", crop_str)
    })?;
    if cx + cw > img.width() || cy + ch > img.height() {
        return Err(format!(
            "Crop region {}x{}+{}+{} exceeds image bounds {}x{}",
            cw, ch, cx, cy, img.width(), img.height()
        ));
    }
    Ok(img.view(cx, cy, cw, ch).to_image())
}

/// Save screenshot image as lossy WebP (quality 15). Extension is forced to .webp.
fn save_screenshot(img: &image::RgbaImage, output: &Path) -> Result<(), String> {
    let output = output.with_extension("webp");
    let encoder = webp::Encoder::from_rgba(img.as_raw(), img.width(), img.height());
    let mem = encoder.encode(15.0);
    std::fs::write(&output, &*mem).map_err(|e| e.to_string())
}
