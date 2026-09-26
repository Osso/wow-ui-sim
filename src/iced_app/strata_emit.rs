//! Strata-level quad emission for headless and screenshot paths.

use iced::{Point, Rectangle, Size};
use rustc_hash::FxHashSet;

use crate::render::QuadBatch;
use crate::render::font::WowFontSystem;
use crate::render::glyph::GlyphAtlas;
use crate::render::texture::UI_SCALE;
use crate::widget::WidgetRegistry;
use crate::widget::WidgetType;

use super::frame_collect::{CollectedFrames, collect_hittable_frames, collect_subtree_ids};
use super::quad_builders::{
    FrameQuadEmit, build_texture_quads, emit_button_highlight, emit_frame_quads,
};
use super::statusbar::collect_statusbar_fills;
use super::tooltip::TooltipRenderData;

pub type MessageFrameMap =
    std::collections::HashMap<u64, crate::lua_api::message_frame::MessageFrameData>;
pub type TooltipDataMap = std::collections::HashMap<u64, TooltipRenderData>;
pub type QuestBlobMap = std::collections::HashMap<u64, crate::lua_api::state::QuestBlobState>;

fn uses_parent_alpha_fallback(frame: &crate::widget::Frame) -> bool {
    matches!(
        frame.parent_key.as_deref(),
        Some("NormalTexture" | "PushedTexture" | "HighlightTexture" | "DisabledTexture")
    )
}

fn chain_effective_alpha_from(
    start_id: Option<u64>,
    registry: &crate::widget::WidgetRegistry,
) -> f32 {
    let Some(mut current_id) = start_id else {
        return 1.0;
    };
    let mut alpha = 1.0;
    loop {
        let Some(frame) = registry.get(current_id) else {
            return 0.0;
        };
        if !frame.visible {
            return 0.0;
        }
        alpha *= frame.alpha;
        let Some(parent_id) = frame.parent_id else {
            return alpha;
        };
        current_id = parent_id;
    }
}

/// Emit quads for a single strata bucket (used by headless/screenshot paths).
///
/// Reads rect and effective_alpha fresh from the registry for each frame.
/// Button state textures use parent's effective_alpha as fallback.
pub(super) fn emit_single_strata(
    batch: &mut QuadBatch,
    text_ctx: &mut Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
    params: SingleStrataEmit<'_>,
) {
    let render_list = build_render_list(params.bucket, params.registry, params.screen_size);
    let statusbar_fills = collect_statusbar_fills(&render_list, params.registry);

    for entry in &render_list {
        emit_render_list_entry(batch, text_ctx, &statusbar_fills, params, *entry);
    }
}

fn emit_render_list_entry(
    batch: &mut QuadBatch,
    text_ctx: &mut Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
    statusbar_fills: &std::collections::HashMap<u64, super::statusbar::StatusBarFill>,
    params: SingleStrataEmit<'_>,
    entry: (u64, crate::LayoutRect, Option<crate::LayoutRect>, f32),
) {
    let (id, rect, clip_rect, eff_alpha) = entry;
    let Some(frame) = render_list_frame(params, id, rect, eff_alpha) else {
        return;
    };
    let bounds = layout_rect_to_screen_rect(rect);
    emit_frame_quads(
        batch,
        text_ctx,
        FrameQuadEmit {
            id,
            widget: frame,
            bounds,
            clip_bounds: clip_rect.map(layout_rect_to_screen_rect),
            bar_fill: statusbar_fills.get(&id),
            pressed_frame: params.pressed_frame,
            hovered_frame: params.hovered_frame,
            message_frames: params.message_frames,
            tooltip_data: params.tooltip_data,
            quest_blobs: params.quest_blobs,
            registry: params.registry,
            elapsed_secs: params.elapsed_secs,
            eff_alpha,
        },
    );
}

fn render_list_frame(
    params: SingleStrataEmit<'_>,
    id: u64,
    rect: crate::LayoutRect,
    eff_alpha: f32,
) -> Option<&crate::widget::Frame> {
    let frame = params.registry.get(id)?;
    if super::button_vis::should_skip_frame(
        frame,
        id,
        eff_alpha,
        params.visible_ids,
        params.registry,
        params.pressed_frame,
        params.hovered_frame,
    ) {
        return None;
    }
    renderable_frame_with_bounds(frame, rect)
}

fn renderable_frame_with_bounds(
    frame: &crate::widget::Frame,
    rect: crate::LayoutRect,
) -> Option<&crate::widget::Frame> {
    let is_fontstring = matches!(
        frame.widget_type,
        WidgetType::FontString | WidgetType::SimpleHTML
    );
    let is_line = matches!(frame.widget_type, WidgetType::Line);
    let has_height = rect.height > 0.0 || is_line;
    let has_width = rect.width > 0.0 || is_fontstring || is_line;
    (has_height && has_width).then_some(frame)
}

#[derive(Clone, Copy)]
pub(super) struct SingleStrataEmit<'a> {
    bucket: &'a [u64],
    registry: &'a crate::widget::WidgetRegistry,
    visible_ids: &'a Option<FxHashSet<u64>>,
    screen_size: (f32, f32),
    pressed_frame: Option<u64>,
    hovered_frame: Option<u64>,
    message_frames: Option<&'a MessageFrameMap>,
    tooltip_data: Option<&'a TooltipDataMap>,
    quest_blobs: Option<&'a QuestBlobMap>,
    elapsed_secs: f64,
}

/// Build the render list: visible frames with resolved rects and effective alpha.
pub(super) fn build_render_list(
    bucket: &[u64],
    registry: &crate::widget::WidgetRegistry,
    screen_size: (f32, f32),
) -> Vec<(u64, crate::LayoutRect, Option<crate::LayoutRect>, f32)> {
    let mut list = Vec::new();
    for &id in bucket {
        let Some(f) = registry.get(id) else { continue };
        if !has_renderable_rect(registry, f, id) {
            continue;
        }
        let rect = render_rect_for_frame(f, registry, id, screen_size);
        let clip_rect = resolve_clip_rect(id, registry);
        let eff_alpha = resolve_eff_alpha(f, registry);
        if eff_alpha <= 0.0 {
            continue;
        }
        if clip_rect.is_some_and(|clip| intersect_rects(rect, clip).is_none()) {
            continue;
        }
        list.push((id, rect, clip_rect, eff_alpha));
    }
    list
}

fn render_rect_for_frame(
    frame: &crate::widget::Frame,
    registry: &crate::widget::WidgetRegistry,
    id: u64,
    screen_size: (f32, f32),
) -> crate::LayoutRect {
    if matches!(frame.widget_type, WidgetType::Line)
        && let Some(rect) = line_endpoint_bounds(frame, registry)
    {
        return rect;
    }
    frame.layout_rect.unwrap_or_else(|| {
        crate::layout::compute_frame_rect(registry, id, screen_size.0, screen_size.1)
    })
}

fn line_endpoint_bounds(
    frame: &crate::widget::Frame,
    registry: &crate::widget::WidgetRegistry,
) -> Option<crate::LayoutRect> {
    let start = resolve_line_endpoint_position(frame.line_start.as_ref()?, registry)?;
    let end = resolve_line_endpoint_position(frame.line_end.as_ref()?, registry)?;
    let min_x = start.0.min(end.0);
    let min_y = start.1.min(end.1);
    let max_x = start.0.max(end.0);
    let max_y = start.1.max(end.1);
    Some(crate::LayoutRect {
        x: min_x,
        y: min_y,
        width: (max_x - min_x).max(1.0),
        height: (max_y - min_y).max(1.0),
    })
}

fn resolve_line_endpoint_position(
    anchor: &crate::widget::LineAnchor,
    registry: &crate::widget::WidgetRegistry,
) -> Option<(f32, f32)> {
    use super::layout::anchor_position;

    let target_id = anchor.target_id?;
    let rect = registry.get(target_id)?.layout_rect?;
    let (ax, ay) = anchor_position(anchor.point, rect.x, rect.y, rect.width, rect.height);
    let ui_scale = crate::render::texture::UI_SCALE;
    Some((
        (ax + anchor.x_offset) * ui_scale,
        (ay - anchor.y_offset) * ui_scale,
    ))
}

fn has_renderable_rect(
    registry: &crate::widget::WidgetRegistry,
    frame: &crate::widget::Frame,
    id: u64,
) -> bool {
    if !has_own_renderable_rect(registry, frame, id) {
        return false;
    }
    if has_parent_independent_geometry(frame) {
        return true;
    }
    let mut current = frame.parent_id;
    while let Some(parent_id) = current {
        let Some(parent) = registry.get(parent_id) else {
            return false;
        };
        if !has_own_renderable_rect(registry, parent, parent_id) {
            return false;
        }
        current = parent.parent_id;
    }
    true
}

fn has_own_renderable_rect(
    registry: &crate::widget::WidgetRegistry,
    frame: &crate::widget::Frame,
    id: u64,
) -> bool {
    !frame.anchors.is_empty()
        || has_line_endpoint_geometry(frame)
        || frame.name.as_deref() == Some("UIParent")
        || id == 1
        || is_statusbar_bar_child(registry, frame)
}

fn has_line_endpoint_geometry(frame: &crate::widget::Frame) -> bool {
    matches!(frame.widget_type, WidgetType::Line)
        && frame
            .line_start
            .as_ref()
            .is_some_and(|anchor| anchor.target_id.is_some())
        && frame
            .line_end
            .as_ref()
            .is_some_and(|anchor| anchor.target_id.is_some())
}

fn has_parent_independent_geometry(frame: &crate::widget::Frame) -> bool {
    has_parent_independent_line_geometry(frame) || has_parent_independent_anchor_geometry(frame)
}

fn has_parent_independent_line_geometry(frame: &crate::widget::Frame) -> bool {
    if !has_line_endpoint_geometry(frame) {
        return false;
    }
    let parent_id = frame.parent_id;
    let start_target = frame
        .line_start
        .as_ref()
        .and_then(|anchor| anchor.target_id);
    let end_target = frame.line_end.as_ref().and_then(|anchor| anchor.target_id);
    start_target != parent_id && end_target != parent_id
}

fn has_parent_independent_anchor_geometry(frame: &crate::widget::Frame) -> bool {
    !frame.anchors.is_empty()
        && frame.anchors.iter().all(|anchor| {
            anchor
                .relative_to_id
                .is_some_and(|target_id| Some(target_id as u64) != frame.parent_id)
        })
}

fn is_statusbar_bar_child(
    registry: &crate::widget::WidgetRegistry,
    frame: &crate::widget::Frame,
) -> bool {
    let Some(parent_id) = frame.parent_id else {
        return false;
    };
    registry
        .get(parent_id)
        .is_some_and(|parent| parent.statusbar_bar_id == Some(frame.id))
}

/// Resolve effective alpha for a frame, with button-state-texture parent fallback.
fn resolve_eff_alpha(f: &crate::widget::Frame, registry: &crate::widget::WidgetRegistry) -> f32 {
    if f.alpha > 0.0 && uses_parent_alpha_fallback(f) {
        return chain_effective_alpha_from(f.parent_id, registry) * f.alpha;
    }
    chain_effective_alpha_from(Some(f.id), registry)
}

fn resolve_clip_rect(
    id: u64,
    registry: &crate::widget::WidgetRegistry,
) -> Option<crate::LayoutRect> {
    let mut current_id = id;
    let mut clip_rect: Option<crate::LayoutRect> = None;
    while let Some(parent_id) = registry.get(current_id).and_then(|f| f.parent_id) {
        let Some(parent) = registry.get(parent_id) else {
            break;
        };
        let clips_scroll_child =
            matches!(parent.widget_type, crate::widget::WidgetType::ScrollFrame)
                && parent.scroll_child_id == Some(current_id);
        if (parent.clips_children || clips_scroll_child)
            && let Some(parent_rect) = parent.layout_rect
        {
            clip_rect = Some(match clip_rect {
                Some(existing) => intersect_rects(existing, parent_rect)?,
                None => parent_rect,
            });
        }
        current_id = parent_id;
    }
    clip_rect
}

fn intersect_rects(a: crate::LayoutRect, b: crate::LayoutRect) -> Option<crate::LayoutRect> {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);
    (right > left && bottom > top).then_some(crate::LayoutRect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    })
}

fn layout_rect_to_screen_rect(rect: crate::LayoutRect) -> Rectangle {
    Rectangle::new(
        Point::new(rect.x * UI_SCALE, rect.y * UI_SCALE),
        Size::new(rect.width * UI_SCALE, rect.height * UI_SCALE),
    )
}

pub struct RegistryQuadBatchParams<'a, 'font> {
    pub registry: &'a WidgetRegistry,
    pub screen_size: (f32, f32),
    pub root_name: Option<&'a str>,
    pub pressed_frame: Option<u64>,
    pub hovered_frame: Option<u64>,
    pub text_ctx: Option<(&'font mut WowFontSystem, &'font mut GlyphAtlas)>,
    pub message_frames: Option<&'a MessageFrameMap>,
    pub tooltip_data: Option<&'a TooltipDataMap>,
    pub quest_blobs: Option<&'a QuestBlobMap>,
    pub strata_buckets: &'a [Vec<u64>],
}

impl<'a, 'font> RegistryQuadBatchParams<'a, 'font> {
    pub fn new(
        registry: &'a WidgetRegistry,
        screen_size: (f32, f32),
        strata_buckets: &'a [Vec<u64>],
    ) -> Self {
        Self {
            registry,
            screen_size,
            root_name: None,
            pressed_frame: None,
            hovered_frame: None,
            text_ctx: None,
            message_frames: None,
            tooltip_data: None,
            quest_blobs: None,
            strata_buckets,
        }
    }

    pub fn root_name(mut self, root_name: Option<&'a str>) -> Self {
        self.root_name = root_name;
        self
    }

    pub fn pressed_frame(mut self, pressed_frame: Option<u64>) -> Self {
        self.pressed_frame = pressed_frame;
        self
    }

    pub fn hovered_frame(mut self, hovered_frame: Option<u64>) -> Self {
        self.hovered_frame = hovered_frame;
        self
    }

    pub fn text_ctx(
        mut self,
        text_ctx: Option<(&'font mut WowFontSystem, &'font mut GlyphAtlas)>,
    ) -> Self {
        self.text_ctx = text_ctx;
        self
    }

    pub fn message_frames(mut self, message_frames: Option<&'a MessageFrameMap>) -> Self {
        self.message_frames = message_frames;
        self
    }

    pub fn tooltip_data(mut self, tooltip_data: Option<&'a TooltipDataMap>) -> Self {
        self.tooltip_data = tooltip_data;
        self
    }

    pub fn quest_blobs(mut self, quest_blobs: Option<&'a QuestBlobMap>) -> Self {
        self.quest_blobs = quest_blobs;
        self
    }
}

/// Build a QuadBatch from a WidgetRegistry without needing an App instance.
pub fn build_quad_batch_for_registry(params: RegistryQuadBatchParams<'_, '_>) -> QuadBatch {
    build_quad_batch_for_registry_with_quest_blobs(params)
}

pub fn build_quad_batch_for_registry_with_quest_blobs(
    params: RegistryQuadBatchParams<'_, '_>,
) -> QuadBatch {
    let mut text_ctx = params.text_ctx;
    let (batch, _) = build_quad_batch_with_cache(CachedQuadBatchParams {
        registry: params.registry,
        screen_size: params.screen_size,
        root_name: params.root_name,
        pressed_frame: params.pressed_frame,
        hovered_frame: params.hovered_frame,
        text_ctx: &mut text_ctx,
        message_frames: params.message_frames,
        tooltip_data: params.tooltip_data,
        quest_blobs: params.quest_blobs,
        strata_buckets: params.strata_buckets,
        elapsed_secs: 0.0,
    });
    batch
}

/// Scale hittable layout rects to screen coordinates, applying hit rect insets.
pub fn build_hittable_rects(
    collected: &CollectedFrames,
    registry: &crate::widget::WidgetRegistry,
) -> Vec<(u64, Rectangle, super::frame_collect::HitOrderKey)> {
    collected
        .hittable
        .iter()
        .map(|&(id, key, r)| {
            let (il, ir, it, ib) = registry
                .get(id)
                .map(super::frame_collect::scaled_hit_rect_insets)
                .unwrap_or((0.0, 0.0, 0.0, 0.0));
            (
                id,
                Rectangle::new(
                    Point::new((r.x + il) * UI_SCALE, (r.y + it) * UI_SCALE),
                    Size::new(
                        (r.width - il - ir).max(0.0) * UI_SCALE,
                        (r.height - it - ib).max(0.0) * UI_SCALE,
                    ),
                ),
                key,
            )
        })
        .collect()
}

/// Build a QuadBatch by iterating visible-only strata buckets directly.
///
/// Also builds a hittable frame list as a side output for hit testing.
pub struct CachedQuadBatchParams<'a, 'text, 'font> {
    pub registry: &'a WidgetRegistry,
    pub screen_size: (f32, f32),
    pub root_name: Option<&'a str>,
    pub pressed_frame: Option<u64>,
    pub hovered_frame: Option<u64>,
    pub text_ctx: &'text mut Option<(&'font mut WowFontSystem, &'font mut GlyphAtlas)>,
    pub message_frames: Option<&'a MessageFrameMap>,
    pub tooltip_data: Option<&'a TooltipDataMap>,
    pub quest_blobs: Option<&'a QuestBlobMap>,
    pub strata_buckets: &'a [Vec<u64>],
    pub elapsed_secs: f64,
}

pub fn build_quad_batch_with_cache(
    mut params: CachedQuadBatchParams<'_, '_, '_>,
) -> (QuadBatch, CollectedFrames) {
    let mut batch = build_background_batch(params.screen_size);
    let visible_ids = params
        .root_name
        .map(|name| collect_subtree_ids(params.registry, name));
    let collected = collect_hittable_frames(params.registry, params.strata_buckets);

    emit_cached_strata(&mut batch, &mut params, &visible_ids);
    append_hover_highlight(&mut batch, &params, &visible_ids);
    (batch, collected)
}

fn build_background_batch(screen_size: (f32, f32)) -> QuadBatch {
    let mut batch = QuadBatch::with_capacity(1000);
    let size = Size::new(screen_size.0, screen_size.1);

    batch.push_tiled_path(
        Rectangle::new(Point::ORIGIN, size),
        256.0,
        256.0,
        "framegeneral/ui-background-marble",
        [0.55, 0.55, 0.55, 1.0],
    );

    batch
}

fn emit_cached_strata(
    batch: &mut QuadBatch,
    params: &mut CachedQuadBatchParams<'_, '_, '_>,
    visible_ids: &Option<FxHashSet<u64>>,
) {
    for bucket in params.strata_buckets {
        emit_single_strata(
            batch,
            params.text_ctx,
            SingleStrataEmit {
                bucket,
                registry: params.registry,
                visible_ids,
                screen_size: params.screen_size,
                pressed_frame: params.pressed_frame,
                hovered_frame: params.hovered_frame,
                message_frames: params.message_frames,
                tooltip_data: params.tooltip_data,
                quest_blobs: params.quest_blobs,
                elapsed_secs: params.elapsed_secs,
            },
        );
    }
}

fn append_hover_highlight(
    batch: &mut QuadBatch,
    params: &CachedQuadBatchParams<'_, '_, '_>,
    visible_ids: &Option<FxHashSet<u64>>,
) {
    let Some(hovered_id) = params.hovered_frame else {
        return;
    };
    if visible_ids
        .as_ref()
        .is_some_and(|ids| !ids.contains(&hovered_id))
    {
        return;
    }

    let Some(frame) = params.registry.get(hovered_id) else {
        return;
    };
    if !matches!(
        frame.widget_type,
        WidgetType::Button | WidgetType::CheckButton
    ) {
        return;
    }
    if params.pressed_frame == Some(hovered_id)
        || frame.button_state == 1
        || !super::button_vis::should_append_hover_highlight(params.registry, frame)
    {
        return;
    }

    append_hover_highlight_from_frame(batch, params.registry, frame);
}

fn append_hover_highlight_from_frame(
    batch: &mut QuadBatch,
    registry: &WidgetRegistry,
    frame: &crate::widget::Frame,
) {
    if let Some(bounds) = frame.layout_rect.map(layout_rect_to_screen_rect)
        && !frame.children_keys.contains_key("HighlightTexture")
    {
        emit_button_highlight(batch, bounds, frame, frame.alpha);
    }

    let Some(&highlight_id) = frame.children_keys.get("HighlightTexture") else {
        return;
    };
    let Some(highlight) = registry.get(highlight_id) else {
        return;
    };
    if let Some(bounds) = highlight.layout_rect.map(layout_rect_to_screen_rect) {
        build_texture_quads(batch, bounds, highlight, None, highlight.alpha);
    }
}

#[cfg(test)]
mod tests {
    use super::{build_render_list, resolve_eff_alpha};
    use crate::widget::{AnchorPoint, Frame, WidgetRegistry, WidgetType};

    #[test]
    fn button_state_texture_alpha_fallback_multiplies_own_alpha() {
        let mut registry = WidgetRegistry::new();

        let root = Frame::new(WidgetType::Frame, Some("UIParent".to_string()), None);
        let root_id = root.id;
        registry.register(root);

        let mut button = Frame::new(
            WidgetType::Button,
            Some("GameMenuButton".to_string()),
            Some(root_id),
        );
        button.alpha = 0.8;
        let button_id = button.id;
        registry.register(button);
        registry.add_child(root_id, button_id);

        let mut normal = Frame::new(WidgetType::Texture, None, Some(button_id));
        normal.visible = false;
        normal.alpha = 0.4;
        normal.parent_key = Some("NormalTexture".to_string());
        let normal_id = normal.id;
        registry.register(normal);
        registry.add_child(button_id, normal_id);

        registry.propagate_all_effective_alpha();

        let alpha = resolve_eff_alpha(registry.get(normal_id).unwrap(), &registry);
        assert!(
            (alpha - 0.32).abs() < f32::EPSILON,
            "expected hidden button texture alpha fallback to keep child alpha, got {alpha}"
        );
    }

    #[test]
    fn render_list_skips_unanchored_child_frames() {
        let mut registry = WidgetRegistry::new();

        let mut root = Frame::new(WidgetType::Frame, Some("UIParent".to_string()), None);
        root.layout_rect = Some(crate::LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 1024.0,
            height: 768.0,
        });
        let root_id = root.id;
        registry.register(root);

        let mut unanchored = Frame::new(WidgetType::EditBox, None, Some(root_id));
        unanchored.set_size(256.0, 32.0);
        let unanchored_id = unanchored.id;
        registry.register(unanchored);
        registry.add_child(root_id, unanchored_id);

        let render_list = build_render_list(&[unanchored_id], &registry, (1024.0, 768.0));

        assert!(
            render_list.is_empty(),
            "unanchored children have no valid WoW rect and should not render at parent origin"
        );
    }

    #[test]
    fn render_list_keeps_anchored_child_frames() {
        let mut registry = WidgetRegistry::new();

        let mut root = Frame::new(WidgetType::Frame, Some("UIParent".to_string()), None);
        root.layout_rect = Some(crate::LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 1024.0,
            height: 768.0,
        });
        let root_id = root.id;
        registry.register(root);

        let mut anchored = Frame::new(WidgetType::EditBox, None, Some(root_id));
        anchored.set_size(256.0, 32.0);
        anchored.set_point(
            AnchorPoint::TopLeft,
            Some(root_id as usize),
            AnchorPoint::TopLeft,
            20.0,
            -20.0,
        );
        let anchored_id = anchored.id;
        registry.register(anchored);
        registry.add_child(root_id, anchored_id);

        let render_list = build_render_list(&[anchored_id], &registry, (1024.0, 768.0));

        assert_eq!(render_list.len(), 1);
        assert_eq!(render_list[0].0, anchored_id);
    }

    #[test]
    fn render_list_skips_children_of_unanchored_frames() {
        let mut registry = WidgetRegistry::new();

        let mut root = Frame::new(WidgetType::Frame, Some("UIParent".to_string()), None);
        root.layout_rect = Some(crate::LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 1024.0,
            height: 768.0,
        });
        let root_id = root.id;
        registry.register(root);

        let mut unanchored = Frame::new(WidgetType::EditBox, None, Some(root_id));
        unanchored.set_size(256.0, 32.0);
        let unanchored_id = unanchored.id;
        registry.register(unanchored);
        registry.add_child(root_id, unanchored_id);

        let mut child_texture = Frame::new(WidgetType::Texture, None, Some(unanchored_id));
        child_texture.set_size(256.0, 32.0);
        child_texture.set_point(
            AnchorPoint::TopLeft,
            Some(unanchored_id as usize),
            AnchorPoint::TopLeft,
            0.0,
            0.0,
        );
        let child_id = child_texture.id;
        registry.register(child_texture);
        registry.add_child(unanchored_id, child_id);

        let render_list = build_render_list(&[child_id], &registry, (1024.0, 768.0));

        assert!(
            render_list.is_empty(),
            "children anchored to an unanchored frame should not render through a fallback parent-origin rect"
        );
    }
}

#[cfg(test)]
#[path = "strata_emit_endpoint_tests.rs"]
mod endpoint_tests;
