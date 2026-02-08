//! IPC screenshot rendering for the running app.

use std::path::Path;

use crate::lua_server::Response as LuaResponse;
use crate::render::software::render_to_image;
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
    ) -> LuaResponse {
        let output_path = Path::new(output).with_extension("webp");

        let mut glyph_atlas = GlyphAtlas::new();
        let batch = {
            let env = self.env.borrow();
            let mut fs = self.font_system.borrow_mut();
            // Update tooltip sizes before rendering
            {
                let mut state = env.state().borrow_mut();
                super::tooltip::update_tooltip_sizes(&mut state, &mut fs);
            }
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

        if let Err(e) = save_screenshot(&img, &output_path) {
            return LuaResponse::Error(format!("Failed to save screenshot: {}", e));
        }

        LuaResponse::Output(format!(
            "Saved {}x{} screenshot to {}",
            width, height, output_path.display()
        ))
    }
}

/// Save screenshot image as lossy WebP (quality 15). Extension is forced to .webp.
fn save_screenshot(img: &image::RgbaImage, output: &Path) -> Result<(), String> {
    let output = output.with_extension("webp");
    let encoder = webp::Encoder::from_rgba(img.as_raw(), img.width(), img.height());
    let mem = encoder.encode(15.0);
    std::fs::write(&output, &*mem).map_err(|e| e.to_string())
}
