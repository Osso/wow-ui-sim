//! Nine-slice quad rendering for panel borders and frames.

use iced::Rectangle;
use super::quad::{BlendMode, QuadBatch};

/// Texture indices for 9-slice rendering.
#[derive(Debug, Clone, Copy, Default)]
pub struct NineSliceTextures {
    pub top_left: Option<i32>,
    pub top: Option<i32>,
    pub top_right: Option<i32>,
    pub left: Option<i32>,
    pub center: Option<i32>,
    pub right: Option<i32>,
    pub bottom_left: Option<i32>,
    pub bottom: Option<i32>,
    pub bottom_right: Option<i32>,
}

impl QuadBatch {
    /// Push a 9-slice texture (corners fixed, edges stretched, center stretched).
    pub fn push_nine_slice(
        &mut self,
        bounds: Rectangle,
        corner_size: f32,
        edge_size: f32,
        textures: &NineSliceTextures,
        color: [f32; 4],
    ) {
        if bounds.width < corner_size * 2.0 || bounds.height < corner_size * 2.0 {
            if let Some(center) = textures.center {
                self.push_textured(bounds, center, color, BlendMode::Alpha);
            }
            return;
        }

        let inner_width = bounds.width - corner_size * 2.0;
        let inner_height = bounds.height - corner_size * 2.0;
        let full_uv = Rectangle::new(iced::Point::ORIGIN, iced::Size::new(1.0, 1.0));

        push_center(self, bounds, edge_size, textures, color, full_uv);
        push_corners(self, bounds, corner_size, textures, color, full_uv);
        push_edges(self, bounds, corner_size, edge_size, inner_width, inner_height, textures, color, full_uv);
    }
}

fn push_center(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    edge_size: f32,
    textures: &NineSliceTextures,
    color: [f32; 4],
    full_uv: Rectangle,
) {
    if let Some(tex) = textures.center {
        let center_bounds = Rectangle::new(
            iced::Point::new(bounds.x + edge_size, bounds.y + edge_size),
            iced::Size::new(bounds.width - edge_size * 2.0, bounds.height - edge_size * 2.0),
        );
        batch.push_quad(center_bounds, full_uv, color, tex, BlendMode::Alpha);
    }
}

fn push_corners(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    corner_size: f32,
    textures: &NineSliceTextures,
    color: [f32; 4],
    full_uv: Rectangle,
) {
    if let Some(tex) = textures.top_left {
        let corner = Rectangle::new(
            iced::Point::new(bounds.x, bounds.y),
            iced::Size::new(corner_size, corner_size),
        );
        batch.push_quad(corner, full_uv, color, tex, BlendMode::Alpha);
    }
    if let Some(tex) = textures.top_right {
        let corner = Rectangle::new(
            iced::Point::new(bounds.x + bounds.width - corner_size, bounds.y),
            iced::Size::new(corner_size, corner_size),
        );
        batch.push_quad(corner, full_uv, color, tex, BlendMode::Alpha);
    }
    if let Some(tex) = textures.bottom_left {
        let corner = Rectangle::new(
            iced::Point::new(bounds.x, bounds.y + bounds.height - corner_size),
            iced::Size::new(corner_size, corner_size),
        );
        batch.push_quad(corner, full_uv, color, tex, BlendMode::Alpha);
    }
    if let Some(tex) = textures.bottom_right {
        let corner = Rectangle::new(
            iced::Point::new(
                bounds.x + bounds.width - corner_size,
                bounds.y + bounds.height - corner_size,
            ),
            iced::Size::new(corner_size, corner_size),
        );
        batch.push_quad(corner, full_uv, color, tex, BlendMode::Alpha);
    }
}

#[allow(clippy::too_many_arguments)]
fn push_edges(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    corner_size: f32,
    edge_size: f32,
    inner_width: f32,
    inner_height: f32,
    textures: &NineSliceTextures,
    color: [f32; 4],
    full_uv: Rectangle,
) {
    if let Some(tex) = textures.top {
        let edge = Rectangle::new(
            iced::Point::new(bounds.x + corner_size, bounds.y),
            iced::Size::new(inner_width, edge_size),
        );
        batch.push_quad(edge, full_uv, color, tex, BlendMode::Alpha);
    }
    if let Some(tex) = textures.bottom {
        let edge = Rectangle::new(
            iced::Point::new(bounds.x + corner_size, bounds.y + bounds.height - edge_size),
            iced::Size::new(inner_width, edge_size),
        );
        batch.push_quad(edge, full_uv, color, tex, BlendMode::Alpha);
    }
    if let Some(tex) = textures.left {
        let edge = Rectangle::new(
            iced::Point::new(bounds.x, bounds.y + corner_size),
            iced::Size::new(edge_size, inner_height),
        );
        batch.push_quad(edge, full_uv, color, tex, BlendMode::Alpha);
    }
    if let Some(tex) = textures.right {
        let edge = Rectangle::new(
            iced::Point::new(bounds.x + bounds.width - edge_size, bounds.y + corner_size),
            iced::Size::new(edge_size, inner_height),
        );
        batch.push_quad(edge, full_uv, color, tex, BlendMode::Alpha);
    }
}
