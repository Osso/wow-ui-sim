//! 9-slice rendering for WoW-style frame borders.

use crate::texture::TextureManager;
use iced::widget::canvas::{self, Image};
use iced::widget::image::Handle as ImageHandle;
use iced::Rectangle;
use std::collections::HashMap;

/// A 9-slice frame definition using the DialogFrame textures.
#[derive(Debug, Clone)]
pub struct NineSliceFrame {
    /// Base path for textures (e.g., "Interface/DialogFrame")
    pub base_path: String,
    /// Corner size (width/height of corner pieces)
    pub corner_size: f32,
    /// Edge thickness
    pub edge_size: f32,
}

impl Default for NineSliceFrame {
    fn default() -> Self {
        Self::dialog_frame()
    }
}

impl NineSliceFrame {
    /// Create a WoW DialogFrame style border.
    pub fn dialog_frame() -> Self {
        Self {
            base_path: "Interface/DialogFrame".to_string(),
            corner_size: 32.0,
            edge_size: 16.0,
        }
    }

    /// Get the texture paths needed for this frame.
    pub fn texture_paths(&self) -> Vec<String> {
        vec![
            format!("{}/DialogFrame-Corners", self.base_path),
            format!("{}/DialogFrame-Top", self.base_path),
            format!("{}/DialogFrame-Bot", self.base_path),
            format!("{}/DialogFrame-Left", self.base_path),
            format!("{}/DialogFrame-Right", self.base_path),
            format!("{}/UI-DialogBox-Background", self.base_path),
        ]
    }

    /// Get corner sub-region keys for the corners atlas.
    /// DialogFrame-Corners.PNG is 64x64 with 4 32x32 corners.
    pub fn corner_keys(&self) -> [(String, u32, u32, u32, u32); 4] {
        let corners_path = format!("{}/DialogFrame-Corners", self.base_path);
        [
            (format!("{}#TL", corners_path), 0, 0, 32, 32),      // Top-left
            (format!("{}#TR", corners_path), 32, 0, 32, 32),     // Top-right
            (format!("{}#BL", corners_path), 0, 32, 32, 32),     // Bottom-left
            (format!("{}#BR", corners_path), 32, 32, 32, 32),    // Bottom-right
        ]
    }
}

/// Preload all textures needed for a 9-slice frame.
pub fn preload_nine_slice_textures(
    tex_mgr: &mut TextureManager,
    handles: &mut HashMap<String, ImageHandle>,
    nine_slice: &NineSliceFrame,
) {
    // Load main textures
    for path in nine_slice.texture_paths() {
        if !handles.contains_key(&path) {
            if let Some(tex_data) = tex_mgr.load(&path) {
                let handle = ImageHandle::from_rgba(
                    tex_data.width,
                    tex_data.height,
                    tex_data.pixels.clone(),
                );
                handles.insert(path, handle);
            }
        }
    }

    // Load corner sub-regions
    for (key, x, y, w, h) in nine_slice.corner_keys() {
        if !handles.contains_key(&key) {
            let corners_path = format!("{}/DialogFrame-Corners", nine_slice.base_path);
            if let Some(tex_data) = tex_mgr.load_sub_region(&corners_path, x, y, w, h) {
                let handle = ImageHandle::from_rgba(
                    tex_data.width,
                    tex_data.height,
                    tex_data.pixels.clone(),
                );
                handles.insert(key, handle);
            }
        }
    }
}

/// Draw a 9-slice frame.
pub fn draw_nine_slice(
    frame: &mut canvas::Frame,
    bounds: Rectangle,
    nine_slice: &NineSliceFrame,
    image_handles: &HashMap<String, ImageHandle>,
    alpha: f32,
) {
    let corner = nine_slice.corner_size;
    let edge = nine_slice.edge_size;

    // Draw center (tiled background)
    let center_path = format!("{}/UI-DialogBox-Background", nine_slice.base_path);
    if let Some(handle) = image_handles.get(&center_path) {
        let center_bounds = Rectangle {
            x: bounds.x + edge,
            y: bounds.y + edge,
            width: bounds.width - edge * 2.0,
            height: bounds.height - edge * 2.0,
        };
        draw_tiled(frame, center_bounds, handle, 64.0, 64.0, alpha);
    }

    // Draw edges (tiled)
    // Top edge
    let top_path = format!("{}/DialogFrame-Top", nine_slice.base_path);
    if let Some(handle) = image_handles.get(&top_path) {
        let top_bounds = Rectangle {
            x: bounds.x + corner,
            y: bounds.y,
            width: bounds.width - corner * 2.0,
            height: edge,
        };
        draw_tiled_horizontal(frame, top_bounds, handle, 32.0, alpha);
    }

    // Bottom edge
    let bottom_path = format!("{}/DialogFrame-Bot", nine_slice.base_path);
    if let Some(handle) = image_handles.get(&bottom_path) {
        let bottom_bounds = Rectangle {
            x: bounds.x + corner,
            y: bounds.y + bounds.height - edge,
            width: bounds.width - corner * 2.0,
            height: edge,
        };
        draw_tiled_horizontal(frame, bottom_bounds, handle, 32.0, alpha);
    }

    // Left edge
    let left_path = format!("{}/DialogFrame-Left", nine_slice.base_path);
    if let Some(handle) = image_handles.get(&left_path) {
        let left_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y + corner,
            width: edge,
            height: bounds.height - corner * 2.0,
        };
        draw_tiled_vertical(frame, left_bounds, handle, 32.0, alpha);
    }

    // Right edge
    let right_path = format!("{}/DialogFrame-Right", nine_slice.base_path);
    if let Some(handle) = image_handles.get(&right_path) {
        let right_bounds = Rectangle {
            x: bounds.x + bounds.width - edge,
            y: bounds.y + corner,
            width: edge,
            height: bounds.height - corner * 2.0,
        };
        draw_tiled_vertical(frame, right_bounds, handle, 32.0, alpha);
    }

    // Draw corners (fixed size)
    let corners_path = format!("{}/DialogFrame-Corners", nine_slice.base_path);

    // Top-left
    if let Some(handle) = image_handles.get(&format!("{}#TL", corners_path)) {
        let tl_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: corner,
            height: corner,
        };
        draw_image(frame, tl_bounds, handle, alpha);
    }

    // Top-right
    if let Some(handle) = image_handles.get(&format!("{}#TR", corners_path)) {
        let tr_bounds = Rectangle {
            x: bounds.x + bounds.width - corner,
            y: bounds.y,
            width: corner,
            height: corner,
        };
        draw_image(frame, tr_bounds, handle, alpha);
    }

    // Bottom-left
    if let Some(handle) = image_handles.get(&format!("{}#BL", corners_path)) {
        let bl_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y + bounds.height - corner,
            width: corner,
            height: corner,
        };
        draw_image(frame, bl_bounds, handle, alpha);
    }

    // Bottom-right
    if let Some(handle) = image_handles.get(&format!("{}#BR", corners_path)) {
        let br_bounds = Rectangle {
            x: bounds.x + bounds.width - corner,
            y: bounds.y + bounds.height - corner,
            width: corner,
            height: corner,
        };
        draw_image(frame, br_bounds, handle, alpha);
    }
}

/// Draw a single image at the given bounds.
fn draw_image(frame: &mut canvas::Frame, bounds: Rectangle, handle: &ImageHandle, alpha: f32) {
    let image = Image::new(handle.clone())
        .opacity(alpha)
        .filter_method(iced::widget::image::FilterMethod::Linear);
    frame.draw_image(bounds, image);
}

/// Draw a texture tiled horizontally.
fn draw_tiled_horizontal(
    frame: &mut canvas::Frame,
    bounds: Rectangle,
    handle: &ImageHandle,
    tile_width: f32,
    alpha: f32,
) {
    let mut x = bounds.x;
    while x < bounds.x + bounds.width {
        let width = (bounds.x + bounds.width - x).min(tile_width);
        let tile_bounds = Rectangle {
            x,
            y: bounds.y,
            width,
            height: bounds.height,
        };
        draw_image(frame, tile_bounds, handle, alpha);
        x += tile_width;
    }
}

/// Draw a texture tiled vertically.
fn draw_tiled_vertical(
    frame: &mut canvas::Frame,
    bounds: Rectangle,
    handle: &ImageHandle,
    tile_height: f32,
    alpha: f32,
) {
    let mut y = bounds.y;
    while y < bounds.y + bounds.height {
        let height = (bounds.y + bounds.height - y).min(tile_height);
        let tile_bounds = Rectangle {
            x: bounds.x,
            y,
            width: bounds.width,
            height,
        };
        draw_image(frame, tile_bounds, handle, alpha);
        y += tile_height;
    }
}

/// Draw a texture tiled in both directions.
fn draw_tiled(
    frame: &mut canvas::Frame,
    bounds: Rectangle,
    handle: &ImageHandle,
    tile_width: f32,
    tile_height: f32,
    alpha: f32,
) {
    let mut y = bounds.y;
    while y < bounds.y + bounds.height {
        let height = (bounds.y + bounds.height - y).min(tile_height);
        let mut x = bounds.x;
        while x < bounds.x + bounds.width {
            let width = (bounds.x + bounds.width - x).min(tile_width);
            let tile_bounds = Rectangle {
                x,
                y,
                width,
                height,
            };
            draw_image(frame, tile_bounds, handle, alpha);
            x += tile_width;
        }
        y += tile_height;
    }
}

/// Button state for texture selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonState {
    #[default]
    Normal,
    Hover,
    Pressed,
    Disabled,
}

/// Get the texture path for a WoW panel button in the given state.
pub fn button_texture_path(state: ButtonState) -> &'static str {
    match state {
        ButtonState::Normal => "Interface/Buttons/UI-Panel-Button-Up",
        ButtonState::Hover => "Interface/Buttons/UI-Panel-Button-Highlight",
        ButtonState::Pressed => "Interface/Buttons/UI-Panel-Button-Down",
        ButtonState::Disabled => "Interface/Buttons/UI-Panel-Button-Disabled",
    }
}

/// Draw a WoW-style button with the given state.
pub fn draw_button(
    frame: &mut canvas::Frame,
    bounds: Rectangle,
    state: ButtonState,
    image_handles: &HashMap<String, ImageHandle>,
    alpha: f32,
) {
    let path = button_texture_path(state);
    if let Some(handle) = image_handles.get(path) {
        draw_image(frame, bounds, handle, alpha);
    }
}
