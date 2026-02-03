//! Atlas data for WoW UI textures.
//!
//! Atlases are regions within larger texture sheets. This module provides
//! hardcoded atlas definitions for the frame pieces needed for UI rendering.

use std::collections::HashMap;
use std::sync::LazyLock;

/// Information about a texture atlas region.
#[derive(Debug, Clone)]
pub struct AtlasInfo {
    /// The texture file path (WoW-style path).
    pub file: &'static str,
    /// Width of the atlas region in pixels.
    pub width: u32,
    /// Height of the atlas region in pixels.
    pub height: u32,
    /// Left texture coordinate (0.0-1.0).
    pub left_tex_coord: f32,
    /// Right texture coordinate (0.0-1.0).
    pub right_tex_coord: f32,
    /// Top texture coordinate (0.0-1.0).
    pub top_tex_coord: f32,
    /// Bottom texture coordinate (0.0-1.0).
    pub bottom_tex_coord: f32,
    /// Whether this atlas tiles horizontally.
    pub tiles_horizontally: bool,
    /// Whether this atlas tiles vertically.
    pub tiles_vertically: bool,
}

/// Atlas database with all known atlas definitions.
pub static ATLAS_DB: LazyLock<HashMap<&'static str, AtlasInfo>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    // PortraitFrameTemplate pieces from UIFramePortrait.PNG (256x512)
    // These coordinates are approximations based on the texture layout
    let portrait_tex = "Interface\\FrameGeneral\\UIFramePortrait";

    // TopLeftCorner - Portrait corner with ring (large piece at bottom-left of texture)
    map.insert(
        "UI-Frame-PortraitMetal-CornerTopLeft",
        AtlasInfo {
            file: portrait_tex,
            width: 84,
            height: 84,
            left_tex_coord: 0.0,
            right_tex_coord: 84.0 / 256.0,
            top_tex_coord: 330.0 / 512.0,
            bottom_tex_coord: 414.0 / 512.0,
            tiles_horizontally: false,
            tiles_vertically: false,
        },
    );

    // TopRightCorner
    map.insert(
        "UI-Frame-Metal-CornerTopRight",
        AtlasInfo {
            file: portrait_tex,
            width: 32,
            height: 32,
            left_tex_coord: 224.0 / 256.0,
            right_tex_coord: 1.0,
            top_tex_coord: 64.0 / 512.0,
            bottom_tex_coord: 96.0 / 512.0,
            tiles_horizontally: false,
            tiles_vertically: false,
        },
    );

    // BottomLeftCorner
    map.insert(
        "UI-Frame-Metal-CornerBottomLeft",
        AtlasInfo {
            file: portrait_tex,
            width: 32,
            height: 32,
            left_tex_coord: 0.0,
            right_tex_coord: 32.0 / 256.0,
            top_tex_coord: 96.0 / 512.0,
            bottom_tex_coord: 128.0 / 512.0,
            tiles_horizontally: false,
            tiles_vertically: false,
        },
    );

    // BottomRightCorner
    map.insert(
        "UI-Frame-Metal-CornerBottomRight",
        AtlasInfo {
            file: portrait_tex,
            width: 32,
            height: 32,
            left_tex_coord: 224.0 / 256.0,
            right_tex_coord: 1.0,
            top_tex_coord: 96.0 / 512.0,
            bottom_tex_coord: 128.0 / 512.0,
            tiles_horizontally: false,
            tiles_vertically: false,
        },
    );

    // TopEdge (horizontal, tiles)
    map.insert(
        "_UI-Frame-Metal-EdgeTop",
        AtlasInfo {
            file: portrait_tex,
            width: 256,
            height: 8,
            left_tex_coord: 0.0,
            right_tex_coord: 1.0,
            top_tex_coord: 0.0,
            bottom_tex_coord: 8.0 / 512.0,
            tiles_horizontally: true,
            tiles_vertically: false,
        },
    );

    // BottomEdge (horizontal, tiles)
    map.insert(
        "_UI-Frame-Metal-EdgeBottom",
        AtlasInfo {
            file: portrait_tex,
            width: 256,
            height: 8,
            left_tex_coord: 0.0,
            right_tex_coord: 1.0,
            top_tex_coord: 8.0 / 512.0,
            bottom_tex_coord: 16.0 / 512.0,
            tiles_horizontally: true,
            tiles_vertically: false,
        },
    );

    // LeftEdge (vertical, tiles)
    map.insert(
        "!UI-Frame-Metal-EdgeLeft",
        AtlasInfo {
            file: portrait_tex,
            width: 8,
            height: 256,
            left_tex_coord: 0.0,
            right_tex_coord: 8.0 / 256.0,
            top_tex_coord: 128.0 / 512.0,
            bottom_tex_coord: 256.0 / 512.0,
            tiles_horizontally: false,
            tiles_vertically: true,
        },
    );

    // RightEdge (vertical, tiles)
    map.insert(
        "!UI-Frame-Metal-EdgeRight",
        AtlasInfo {
            file: portrait_tex,
            width: 8,
            height: 256,
            left_tex_coord: 248.0 / 256.0,
            right_tex_coord: 1.0,
            top_tex_coord: 128.0 / 512.0,
            bottom_tex_coord: 256.0 / 512.0,
            tiles_horizontally: false,
            tiles_vertically: true,
        },
    );

    // InsetFrameTemplate pieces - simple dark inset border
    let inset_tex = "Interface\\FrameGeneral\\UIFramePortrait";

    map.insert(
        "UI-Frame-InnerTopLeft",
        AtlasInfo {
            file: inset_tex,
            width: 16,
            height: 16,
            left_tex_coord: 0.0,
            right_tex_coord: 16.0 / 256.0,
            top_tex_coord: 256.0 / 512.0,
            bottom_tex_coord: 272.0 / 512.0,
            tiles_horizontally: false,
            tiles_vertically: false,
        },
    );

    map
});

/// Look up atlas information by name.
pub fn get_atlas_info(name: &str) -> Option<&'static AtlasInfo> {
    ATLAS_DB.get(name)
}
