//! Texture / draw-layer widget methods.

mod atlas;
mod blocking_loads;
mod color;
mod coords;
mod draw_shadow;
#[cfg(feature = "retail-12-1-0")]
mod radial;
mod rotation_mask;

use crate::lua_bridge::table_set_rust_fn;
use rilua::LuaResult;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;

const TEXTURE_METHODS: &[(&'static str, rilua::vm::closure::RustFn)] = &[
    // Draw layer + shadow
    ("SetDrawLayer", draw_shadow::set_draw_layer),
    ("GetDrawLayer", draw_shadow::get_draw_layer),
    ("SetShadowOffset", draw_shadow::set_shadow_offset),
    ("GetShadowOffset", draw_shadow::get_shadow_offset),
    ("SetShadowColor", draw_shadow::set_shadow_color),
    ("GetShadowColor", draw_shadow::get_shadow_color),
    // Nine-slice
    ("SetTextureSliceMargins", coords::set_texture_slice_margins),
    ("GetTextureSliceMargins", coords::get_texture_slice_margins),
    ("SetTextureSliceMode", coords::set_texture_slice_mode),
    ("GetTextureSliceMode", coords::get_texture_slice_mode),
    ("ClearTextureSlice", coords::clear_texture_slice),
    // Atlas / texture source
    ("SetAtlas", atlas::set_atlas),
    ("GetAtlas", atlas::get_atlas),
    ("SetTexture", atlas::set_texture),
    ("GetTexture", atlas::get_texture),
    ("GetTextureFileID", atlas::get_texture_file_id),
    ("GetTextureFilePath", atlas::get_texture_file_path),
    // Desaturation
    ("SetDesaturated", color::set_desaturated),
    ("IsDesaturated", color::is_desaturated),
    ("SetDesaturation", color::set_desaturation),
    ("GetDesaturation", color::get_desaturation),
    // Color + blend
    ("SetColorTexture", color::set_color_texture),
    ("SetVertexColor", color::set_vertex_color),
    #[cfg(feature = "retail-12-0-0")]
    (
        "SetVertexColorFromBoolean",
        color::set_vertex_color_from_boolean,
    ),
    ("GetVertexColor", color::get_vertex_color),
    ("SetBlendMode", coords::set_blend_mode),
    ("GetBlendMode", coords::get_blend_mode),
    // Gradient + center color
    ("SetGradient", rotation_mask::set_gradient),
    ("SetCenterColor", color::set_center_color),
    // Rotation
    ("SetRotation", rotation_mask::set_rotation),
    ("SetRadians", rotation_mask::set_rotation),
    ("GetRotation", rotation_mask::get_rotation),
    // Mask
    ("SetMask", rotation_mask::set_mask),
    // Radial progress bar
    #[cfg(feature = "retail-12-1-0")]
    ("ClearRadialProgressBar", radial::clear_radial_progress_bar),
    #[cfg(feature = "retail-12-1-0")]
    (
        "SetRadialProgressBarPercent",
        radial::set_radial_progress_bar_percent,
    ),
    #[cfg(feature = "retail-12-1-0")]
    (
        "GetRadialProgressBarPercent",
        radial::get_radial_progress_bar_percent,
    ),
    #[cfg(feature = "retail-12-1-0")]
    (
        "SetRadialProgressBarStartOffset",
        radial::set_radial_progress_bar_start_offset,
    ),
    #[cfg(feature = "retail-12-1-0")]
    (
        "GetRadialProgressBarStartOffset",
        radial::get_radial_progress_bar_start_offset,
    ),
    #[cfg(feature = "retail-12-1-0")]
    (
        "SetRadialProgressBarEndOffset",
        radial::set_radial_progress_bar_end_offset,
    ),
    #[cfg(feature = "retail-12-1-0")]
    (
        "GetRadialProgressBarEndOffset",
        radial::get_radial_progress_bar_end_offset,
    ),
    #[cfg(feature = "retail-12-1-0")]
    (
        "SetRadialProgressBarFeather",
        radial::set_radial_progress_bar_feather,
    ),
    #[cfg(feature = "retail-12-1-0")]
    (
        "GetRadialProgressBarFeather",
        radial::get_radial_progress_bar_feather,
    ),
    #[cfg(feature = "retail-12-1-0")]
    (
        "SetRadialProgressBarReverse",
        radial::set_radial_progress_bar_reverse,
    ),
    #[cfg(feature = "retail-12-1-0")]
    (
        "GetRadialProgressBarReverse",
        radial::get_radial_progress_bar_reverse,
    ),
    #[cfg(feature = "retail-12-1-0")]
    (
        "SetVisualRadialProgressBarMode",
        radial::set_visual_radial_progress_bar_mode,
    ),
    // VectorGraphics compatibility methods. The simulator stores SVG metadata
    // on the Texture-backed object; rendering SVG paths is not modeled yet.
    #[cfg(feature = "retail-12-1-0")]
    ("ClearSVG", radial::clear_svg),
    #[cfg(feature = "retail-12-1-0")]
    ("GetSVGFileID", radial::get_svg_file_id),
    #[cfg(feature = "retail-12-1-0")]
    ("HasSVG", radial::has_svg),
    #[cfg(feature = "retail-12-1-0")]
    ("SetSVG", radial::set_svg),
    // Tex coords + thickness
    ("SetTexCoord", coords::set_tex_coord),
    ("GetTexCoord", coords::get_tex_coord),
    ("ResetTexCoord", coords::reset_tex_coord),
    ("SetThickness", coords::set_thickness),
    ("GetThickness", coords::get_thickness),
    // Tiling
    ("SetHorizTile", coords::set_horiz_tile),
    ("GetHorizTile", coords::get_horiz_tile),
    ("SetVertTile", coords::set_vert_tile),
    ("GetVertTile", coords::get_vert_tile),
    // Pixel snapping + security
    ("SetTexelSnappingBias", coords::set_texel_snapping_bias),
    ("GetTexelSnappingBias", coords::get_texel_snapping_bias),
    ("SetSnapToPixelGrid", coords::set_snap_to_pixel_grid),
    ("IsSnappingToPixelGrid", coords::is_snapping_to_pixel_grid),
    (
        "SetSecurityDisableSetText",
        draw_shadow::set_security_disable_set_text,
    ),
    // Visuals
    ("SetVisuals", rotation_mask::set_visuals),
    // Sprite sheet
    ("SetSpriteSheetCell", rotation_mask::set_sprite_sheet_cell),
    // Vertex offsets
    ("SetVertexOffset", coords::set_vertex_offset),
    ("GetVertexOffset", coords::get_vertex_offset),
    ("ClearVertexOffsets", coords::clear_vertex_offsets),
    // Blocking loads
    (
        "SetBlockingLoadsRequested",
        blocking_loads::set_blocking_loads_requested,
    ),
    (
        "IsBlockingLoadRequested",
        blocking_loads::is_blocking_load_requested,
    ),
];

pub(super) fn register_texture(state: &mut LuaState, metatable: GcRef<Table>) -> LuaResult<()> {
    for (name, func) in TEXTURE_METHODS {
        table_set_rust_fn(state, metatable, name, *func)?;
    }
    Ok(())
}
