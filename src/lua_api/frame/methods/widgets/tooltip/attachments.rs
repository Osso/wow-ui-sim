//! Retail/PTR texture and atlas attachments on existing tooltip rows.
//!
//! WM63 S8L and WM84: an attachment selects the last existing text row,
//! anchors independently to that row's left or right text and never creates a
//! row. Layout reads the stored settings to reserve width and overflow.

use super::super::shared::opt_string;
use super::super::texture::{apply_atlas_to_frame, apply_rect_tex_coords, resolve_texture_value};
use super::line_frames::sync_tooltip_line_frame;
use crate::lua_api::globals::create_frame::create_frame_instance;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_get};
use crate::lua_api::state::SimState;
use crate::lua_api::tooltip::{
    TOOLTIP_TEXTURE_ANCHOR_ALL, TOOLTIP_TEXTURE_REGION_RIGHT, TooltipAttachment, TooltipTexture,
};
use crate::lua_bridge::stack_val;
use crate::widget::{Anchor, Color, DrawLayer, WidgetType};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// WM63 S8L: the limit applies to the whole tooltip, not to one row.
const MAX_ATTACHMENTS: usize = 30;

/// Native draw order of attachments against tooltip text and each other is
/// UNKNOWN (WM84 section 4). Attachments use the simulator's default texture
/// layer; pooled textures draw in creation order within it.
const ATTACHMENT_DRAW_LAYER: DrawLayer = DrawLayer::Artwork;

enum AttachmentSource {
    Texture {
        path: Option<String>,
        file_data_id: Option<i64>,
    },
    Atlas(String),
}

struct AttachmentSettings {
    width: f32,
    height: f32,
    vertical_offset: f32,
    margins: [f32; 4],
    anchor: i32,
    region: i32,
    tex_coords: [f32; 4],
    color: Color,
    desaturation: f32,
}

pub(super) fn add_texture(state: &mut LuaState, tooltip_id: u64) -> LuaResult<()> {
    let value = stack_val(state, 2);
    let Some((path, file_data_id)) = resolve_texture_value(state, value) else {
        return Ok(());
    };
    if path.is_none() && file_data_id.is_none() {
        return Ok(());
    }
    attach(
        state,
        tooltip_id,
        AttachmentSource::Texture { path, file_data_id },
    )
}

pub(super) fn add_atlas(state: &mut LuaState, tooltip_id: u64) -> LuaResult<()> {
    match opt_string(state, 2).filter(|name| !name.trim().is_empty()) {
        Some(atlas) => attach(state, tooltip_id, AttachmentSource::Atlas(atlas)),
        None => Ok(()),
    }
}

/// Hide every pooled attachment texture after the rows holding them are gone.
pub(crate) fn release_tooltip_attachments(sim: &mut SimState, tooltip_id: u64) {
    let ids = sim
        .tooltips
        .get(&tooltip_id)
        .map(|td| td.attachment_texture_ids.clone())
        .unwrap_or_default();
    for id in ids {
        if let Some(frame) = sim.widgets.get_mut_visual(id) {
            frame.visible = false;
        }
        sim.widgets.mark_rect_dirty(id);
    }
}

fn attach(state: &mut LuaState, tooltip_id: u64, source: AttachmentSource) -> LuaResult<()> {
    let Some((row, slot)) = attachment_row(state, tooltip_id)? else {
        return Ok(());
    };
    let options = stack_val(state, 3);
    let settings = read_settings(state, options);
    let texture_id = pooled_texture(state, tooltip_id, slot)?;
    let target = selected_text_id(state, tooltip_id, row, settings.region)?;
    let mut sim = borrow_state_mut(state)?;
    let (width, height) = resolved_size(&sim, &source, &settings);
    let attachment = TooltipAttachment {
        texture_id,
        width,
        height,
        vertical_offset: settings.vertical_offset,
        margins: settings.margins,
        anchor: settings.anchor,
        region: settings.region,
    };
    apply_texture(&mut sim, &attachment, &source, &settings);
    anchor_texture(&mut sim, &attachment, target);
    if let Some(line) = sim
        .tooltips
        .get_mut(&tooltip_id)
        .and_then(|td| td.lines.get_mut(row))
    {
        match &mut line.texture {
            Some(TooltipTexture::Attachments(attachments)) => attachments.push(attachment),
            texture => *texture = Some(TooltipTexture::Attachments(vec![attachment])),
        }
    }
    sim.widgets.mark_rect_dirty(tooltip_id);
    Ok(())
}

/// Zero-based last row and pool slot, or `None` for an empty or full tooltip.
fn attachment_row(state: &LuaState, tooltip_id: u64) -> LuaResult<Option<(usize, usize)>> {
    let sim = borrow_state(state)?;
    let Some(td) = sim.tooltips.get(&tooltip_id) else {
        return Ok(None);
    };
    let count = td.attachment_count();
    Ok((!td.lines.is_empty() && count < MAX_ATTACHMENTS).then(|| (td.lines.len() - 1, count)))
}

fn pooled_texture(state: &mut LuaState, tooltip_id: u64, slot: usize) -> LuaResult<u64> {
    let existing = borrow_state(state)?
        .tooltips
        .get(&tooltip_id)
        .and_then(|td| td.attachment_texture_ids.get(slot).copied());
    if let Some(id) = existing {
        return Ok(id);
    }
    let id = create_frame_instance(
        state,
        WidgetType::Texture,
        "Texture",
        None,
        Some(tooltip_id),
        true,
        None,
    )?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.draw_layer = ATTACHMENT_DRAW_LAYER;
    }
    sim.invalidate_strata_buckets();
    sim.tooltips
        .entry(tooltip_id)
        .or_default()
        .attachment_texture_ids
        .push(id);
    Ok(id)
}

/// The selected row text object, without rewriting its current text. A left
/// line is created when the row has none yet. A right line that was never
/// created is missing; its attachment still reserves space but has no target.
fn selected_text_id(
    state: &mut LuaState,
    tooltip_id: u64,
    row: usize,
    region: i32,
) -> LuaResult<Option<u64>> {
    let right = region == TOOLTIP_TEXTURE_REGION_RIGHT;
    let existing = borrow_state(state)?
        .tooltips
        .get(&tooltip_id)
        .and_then(|td| {
            let ids = if right {
                &td.right_line_ids
            } else {
                &td.left_line_ids
            };
            ids.get(row).copied()
        })
        .filter(|&id| id != 0);
    if existing.is_some() || right {
        return Ok(existing);
    }
    sync_tooltip_line_frame(state, tooltip_id, false, row + 1)
}

fn read_settings(state: &mut LuaState, options: Val) -> AttachmentSettings {
    let margin = table_get(state, options, "margin");
    let coords = table_get(state, options, "texCoords");
    let color = table_get(state, options, "vertexColor");
    AttachmentSettings {
        width: number_setting(state, options, "width", 12.0),
        height: number_setting(state, options, "height", 12.0),
        vertical_offset: number_setting(state, options, "verticalOffset", 0.0),
        margins: [
            number_setting(state, margin, "left", 0.0),
            number_setting(state, margin, "right", 8.0),
            number_setting(state, margin, "top", 0.0),
            number_setting(state, margin, "bottom", 0.0),
        ],
        anchor: enum_setting(state, options, "anchor", TOOLTIP_TEXTURE_ANCHOR_ALL),
        region: enum_setting(state, options, "region", TOOLTIP_TEXTURE_REGION_RIGHT),
        tex_coords: [
            number_setting(state, coords, "left", 0.0),
            number_setting(state, coords, "right", 1.0),
            number_setting(state, coords, "top", 0.0),
            number_setting(state, coords, "bottom", 1.0),
        ],
        color: Color::new(
            unit_setting(state, color, "r"),
            unit_setting(state, color, "g"),
            unit_setting(state, color, "b"),
            unit_setting(state, color, "a"),
        ),
        desaturation: number_setting(state, options, "desaturation", 0.0).clamp(0.0, 1.0),
    }
}

/// WM84 section 2: double to float; nonnumeric or missing keeps the default.
fn number_setting(state: &mut LuaState, table: Val, key: &str, default: f32) -> f32 {
    match table_get(state, table, key) {
        Val::Num(value) => value as f32,
        _ => default,
    }
}

fn unit_setting(state: &mut LuaState, table: Val, key: &str) -> f32 {
    number_setting(state, table, key, 1.0).clamp(0.0, 1.0)
}

/// WM84 section 2: truncation toward zero; values outside `0..=max` keep the
/// initialized zero. Doubles outside the signed 32-bit range enter a native
/// integer-error path whose effect is UNKNOWN; they also keep zero here.
fn enum_setting(state: &mut LuaState, table: Val, key: &str, max: i32) -> i32 {
    let Val::Num(value) = table_get(state, table, key) else {
        return 0;
    };
    if !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&value) {
        return 0;
    }
    let converted = value.trunc() as i32;
    if (0..=max).contains(&converted) {
        converted
    } else {
        0
    }
}

/// WM63 S8L: a zero dimension uses the resolved texture extent.
fn resolved_size(
    sim: &SimState,
    source: &AttachmentSource,
    settings: &AttachmentSettings,
) -> (f32, f32) {
    if settings.width != 0.0 && settings.height != 0.0 {
        return (settings.width, settings.height);
    }
    let (width, height) = texture_extent(sim, source).unwrap_or((0.0, 0.0));
    (
        if settings.width == 0.0 {
            width
        } else {
            settings.width
        },
        if settings.height == 0.0 {
            height
        } else {
            settings.height
        },
    )
}

fn texture_extent(sim: &SimState, source: &AttachmentSource) -> Option<(f32, f32)> {
    match source {
        AttachmentSource::Atlas(name) => crate::atlas::get_render_atlas_info(name)
            .map(|lookup| (lookup.width() as f32, lookup.height() as f32)),
        AttachmentSource::Texture {
            path: Some(path), ..
        } => sim
            .texture_manager()
            .borrow_mut()
            .get_or_load_texture_size(path)
            .map(|(width, height)| (width as f32, height as f32)),
        AttachmentSource::Texture { path: None, .. } => None,
    }
}

fn apply_texture(
    sim: &mut SimState,
    attachment: &TooltipAttachment,
    source: &AttachmentSource,
    settings: &AttachmentSettings,
) {
    let id = attachment.texture_id;
    if let AttachmentSource::Atlas(name) = source
        && let Some(lookup) = crate::atlas::get_render_atlas_info(name)
    {
        let info = lookup.info;
        let slot = (
            info.left_tex_coord,
            info.right_tex_coord,
            info.top_tex_coord,
            info.bottom_tex_coord,
        );
        apply_atlas_to_frame(&mut sim.widgets, id, name, &lookup, slot, false);
    }
    let Some(frame) = sim.widgets.get_mut_visual(id) else {
        return;
    };
    match source {
        AttachmentSource::Texture { path, file_data_id } => {
            frame.texture = path.clone();
            frame.texture_file_data_id = *file_data_id;
            frame.atlas = None;
            frame.atlas_tex_coords = None;
        }
        AttachmentSource::Atlas(name) if frame.atlas.as_deref() != Some(name.as_str()) => {
            frame.texture = None;
            frame.texture_file_data_id = None;
            frame.atlas = Some(name.clone());
            frame.atlas_tex_coords = None;
        }
        AttachmentSource::Atlas(_) => {}
    }
    frame.color_texture = None;
    frame.set_size(attachment.width, attachment.height);
    // S8L: explicit coordinates are passed to the texture. Their composition
    // with atlas coordinates is UNKNOWN; this uses the texture's SetTexCoord.
    apply_rect_tex_coords(frame, settings.tex_coords);
    frame.vertex_color = Some(settings.color);
    frame.desaturated = settings.desaturation > 0.0;
}

/// WM63 S8L anchor table against the selected text object. Renderer clipping
/// of attachments is UNKNOWN; none is applied.
fn anchor_texture(sim: &mut SimState, attachment: &TooltipAttachment, target: Option<u64>) {
    let id = attachment.texture_id;
    sim.widgets.remove_all_anchor_dependents_for(id);
    let anchors = target.map_or_else(Vec::new, |target| {
        attachment
            .anchor_points()
            .into_iter()
            .map(|(point, relative_point, x_offset)| Anchor {
                point,
                relative_to: None,
                relative_to_id: Some(target as usize),
                relative_point,
                x_offset,
                y_offset: attachment.vertical_offset,
            })
            .collect()
    });
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.anchors = anchors;
        // Placement against a missing text object is UNKNOWN; it is not drawn.
        frame.visible = target.is_some();
    }
    if let Some(target) = target {
        sim.widgets.add_anchor_dependent(target, id);
    }
    sim.widgets.mark_rect_dirty(id);
}
