//! Texture drawing utilities for iced canvas.

use iced::widget::canvas::{self, Frame};
use iced::widget::image::Handle as ImageHandle;
use iced::Rectangle;

/// UI scale factor (1.0 = pixel-perfect, no scaling).
pub const UI_SCALE: f32 = 0.75;

/// Draw a texture scaled to fit the target rectangle.
pub fn draw_scaled_texture(frame: &mut Frame, bounds: Rectangle, handle: &ImageHandle, alpha: f32) {
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        return;
    }

    // iced canvas Image doesn't support direct alpha, so we draw at full opacity
    // Alpha would need to be handled via pre-multiplied alpha in the image data
    let _ = alpha; // TODO: Handle alpha via shader or pre-multiplied texture

    frame.draw_image(bounds, canvas::Image::new(handle.clone()));
}

/// Draw a texture tiled to fill the target rectangle.
pub fn draw_tiled_texture(
    frame: &mut Frame,
    bounds: Rectangle,
    handle: &ImageHandle,
    tile_width: f32,
    tile_height: f32,
    alpha: f32,
) {
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        return;
    }

    let _ = alpha; // TODO: Handle alpha

    let mut y = bounds.y;
    while y < bounds.y + bounds.height {
        let mut x = bounds.x;
        let h = (bounds.y + bounds.height - y).min(tile_height);

        while x < bounds.x + bounds.width {
            let w = (bounds.x + bounds.width - x).min(tile_width);

            let tile_bounds = Rectangle {
                x,
                y,
                width: w,
                height: h,
            };
            frame.draw_image(tile_bounds, canvas::Image::new(handle.clone()));

            x += tile_width;
        }
        y += tile_height;
    }
}

/// Draw a texture with TexCoords (only uses a portion of the texture).
///
/// Note: Iced doesn't support drawing sub-regions of images directly.
/// The texture should be pre-cropped to the desired region before calling this.
/// For additive blending (highlights), we use semi-transparent overlay as approximation.
pub fn draw_texture_with_texcoords(
    frame: &mut Frame,
    bounds: Rectangle,
    handle: &ImageHandle,
    _tex_right: f32,
    _tex_bottom: f32,
    alpha: f32,
    additive: bool,
) {
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        return;
    }

    let _ = alpha;
    let _ = additive; // TODO: Iced doesn't expose blend modes

    // Draw the full texture scaled to bounds
    // For proper TexCoords support, the texture should be pre-cropped
    frame.draw_image(bounds, canvas::Image::new(handle.clone()));
}

/// Draw a texture using horizontal 3-slice (left cap, stretchable middle, right cap).
///
/// WoW button textures use this layout:
/// - `left_cap_ratio`: left cap as ratio of texture width (e.g., 0.09375 = 12/128)
/// - `right_cap_start`: where right cap starts as ratio (e.g., 0.53125)
/// - `tex_right`: right edge of used texture region (e.g., 0.625 = 80/128)
/// - `tex_bottom`: bottom edge of used texture region (e.g., 0.6875 = 22/32)
///
/// Note: For proper 3-slice rendering, we need pre-sliced textures.
/// This version draws the full texture scaled as a fallback.
pub fn draw_horizontal_slice_texture(
    frame: &mut Frame,
    bounds: Rectangle,
    handle: &ImageHandle,
    left_handle: Option<&ImageHandle>,
    middle_handle: Option<&ImageHandle>,
    right_handle: Option<&ImageHandle>,
    left_cap_width: f32,
    right_cap_width: f32,
    alpha: f32,
) {
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        return;
    }

    let _ = alpha;

    // If we have sliced handles, use them
    if let (Some(left), Some(middle), Some(right)) = (left_handle, middle_handle, right_handle) {
        let dst_left_cap = left_cap_width * UI_SCALE;
        let dst_right_cap = right_cap_width * UI_SCALE;
        let dst_middle = bounds.width - dst_left_cap - dst_right_cap;

        if dst_middle >= 0.0 {
            // Left cap
            let left_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y,
                width: dst_left_cap,
                height: bounds.height,
            };
            frame.draw_image(left_bounds, canvas::Image::new(left.clone()));

            // Middle (stretched)
            let middle_bounds = Rectangle {
                x: bounds.x + dst_left_cap,
                y: bounds.y,
                width: dst_middle,
                height: bounds.height,
            };
            frame.draw_image(middle_bounds, canvas::Image::new(middle.clone()));

            // Right cap
            let right_bounds = Rectangle {
                x: bounds.x + bounds.width - dst_right_cap,
                y: bounds.y,
                width: dst_right_cap,
                height: bounds.height,
            };
            frame.draw_image(right_bounds, canvas::Image::new(right.clone()));

            return;
        }
    }

    // Fallback: draw the full texture scaled
    frame.draw_image(bounds, canvas::Image::new(handle.clone()));
}

/// Draw a texture using 9-slice scaling (corners fixed, edges stretch).
pub fn draw_nine_slice_texture(
    frame: &mut Frame,
    bounds: Rectangle,
    handles: &NineSliceHandles,
    corner_size: f32,
    edge_size: f32,
    alpha: f32,
) {
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        return;
    }

    let _ = alpha;

    // If target is smaller than corners, just scale the whole center
    if bounds.width < corner_size * 2.0 || bounds.height < corner_size * 2.0 {
        if let Some(center) = &handles.center {
            frame.draw_image(bounds, canvas::Image::new(center.clone()));
        }
        return;
    }

    let inner_width = bounds.width - corner_size * 2.0;
    let inner_height = bounds.height - corner_size * 2.0;

    // Draw center (tiled or stretched)
    if let Some(center) = &handles.center {
        let center_bounds = Rectangle {
            x: bounds.x + edge_size,
            y: bounds.y + edge_size,
            width: bounds.width - edge_size * 2.0,
            height: bounds.height - edge_size * 2.0,
        };
        frame.draw_image(center_bounds, canvas::Image::new(center.clone()));
    }

    // Draw corners
    if let Some(tl) = &handles.top_left {
        let tl_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: corner_size,
            height: corner_size,
        };
        frame.draw_image(tl_bounds, canvas::Image::new(tl.clone()));
    }

    if let Some(tr) = &handles.top_right {
        let tr_bounds = Rectangle {
            x: bounds.x + bounds.width - corner_size,
            y: bounds.y,
            width: corner_size,
            height: corner_size,
        };
        frame.draw_image(tr_bounds, canvas::Image::new(tr.clone()));
    }

    if let Some(bl) = &handles.bottom_left {
        let bl_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y + bounds.height - corner_size,
            width: corner_size,
            height: corner_size,
        };
        frame.draw_image(bl_bounds, canvas::Image::new(bl.clone()));
    }

    if let Some(br) = &handles.bottom_right {
        let br_bounds = Rectangle {
            x: bounds.x + bounds.width - corner_size,
            y: bounds.y + bounds.height - corner_size,
            width: corner_size,
            height: corner_size,
        };
        frame.draw_image(br_bounds, canvas::Image::new(br.clone()));
    }

    // Draw edges
    if let Some(top) = &handles.top {
        let top_bounds = Rectangle {
            x: bounds.x + corner_size,
            y: bounds.y,
            width: inner_width,
            height: edge_size,
        };
        frame.draw_image(top_bounds, canvas::Image::new(top.clone()));
    }

    if let Some(bottom) = &handles.bottom {
        let bottom_bounds = Rectangle {
            x: bounds.x + corner_size,
            y: bounds.y + bounds.height - edge_size,
            width: inner_width,
            height: edge_size,
        };
        frame.draw_image(bottom_bounds, canvas::Image::new(bottom.clone()));
    }

    if let Some(left) = &handles.left {
        let left_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y + corner_size,
            width: edge_size,
            height: inner_height,
        };
        frame.draw_image(left_bounds, canvas::Image::new(left.clone()));
    }

    if let Some(right) = &handles.right {
        let right_bounds = Rectangle {
            x: bounds.x + bounds.width - edge_size,
            y: bounds.y + corner_size,
            width: edge_size,
            height: inner_height,
        };
        frame.draw_image(right_bounds, canvas::Image::new(right.clone()));
    }
}

/// Handles for 9-slice texture rendering.
#[derive(Default, Clone)]
pub struct NineSliceHandles {
    pub top_left: Option<ImageHandle>,
    pub top: Option<ImageHandle>,
    pub top_right: Option<ImageHandle>,
    pub left: Option<ImageHandle>,
    pub center: Option<ImageHandle>,
    pub right: Option<ImageHandle>,
    pub bottom_left: Option<ImageHandle>,
    pub bottom: Option<ImageHandle>,
    pub bottom_right: Option<ImageHandle>,
}
