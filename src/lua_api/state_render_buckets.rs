use crate::widget::WidgetRegistry;
use rustc_hash::FxHashSet;

#[path = "state_render_buckets/trace.rs"]
mod trace;

pub(super) use trace::should_trace_strata_invalidations;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RegionEntry {
    depth: u32,
    id: u64,
}

pub(super) fn uses_parent_alpha_fallback(frame: &crate::widget::Frame) -> bool {
    matches!(
        frame.parent_key.as_deref(),
        Some("NormalTexture" | "PushedTexture" | "HighlightTexture" | "DisabledTexture")
    )
}

pub(super) fn is_region(wt: crate::widget::WidgetType) -> bool {
    matches!(
        wt,
        crate::widget::WidgetType::Texture
            | crate::widget::WidgetType::FontString
            | crate::widget::WidgetType::Line
    )
}

pub(super) fn is_strata_root_boundary(frame: &crate::widget::Frame) -> bool {
    matches!(frame.name.as_deref(), Some("UIParent" | "WorldFrame"))
}

/// DFS emit: parent frame, then its Texture regions (sorted by draw_layer),
/// then child frames (recursively), then its FontString regions.
///
/// FontStrings are deferred past child frames so that parent text renders on top
/// of child frame backgrounds. In WoW's flat render model, all regions at the
/// same frame_level are interleaved by draw_layer, so we approximate this by
/// splitting regions into textures before children and fontstrings after.
/// EditBox is the exception: its internal text/caret are emitted by the frame
/// itself, so child regions must come first or opaque backgrounds cover input.
pub(super) fn dfs_emit(
    id: u64,
    strata_idx: usize,
    widgets: &WidgetRegistry,
    visible: &FxHashSet<u64>,
    out: &mut Vec<u64>,
) {
    dfs_emit_with_region_mode(id, strata_idx, widgets, visible, out, false);
}

fn dfs_emit_with_region_mode(
    id: u64,
    strata_idx: usize,
    widgets: &WidgetRegistry,
    visible: &FxHashSet<u64>,
    out: &mut Vec<u64>,
    suppress_regions: bool,
) {
    let Some(frame) = widgets.get(id) else {
        return;
    };
    let tooltip_nineslice_id = tooltip_nineslice_child(frame, strata_idx, widgets, visible);
    emit_tooltip_nineslice(
        tooltip_nineslice_id,
        strata_idx,
        widgets,
        visible,
        out,
        suppress_regions,
    );

    let (regions, mut child_frames) =
        collect_regions_and_children(frame, strata_idx, widgets, visible, suppress_regions);
    if let Some(nineslice_id) = tooltip_nineslice_id {
        child_frames.retain(|&child_id| child_id != nineslice_id);
    }

    let region_placement = region_placement_for_frame(frame, suppress_regions);
    emit_frame_with_regions(
        id,
        regions,
        &mut child_frames,
        RegionEmitContext {
            strata_idx,
            widgets,
            visible,
            out,
            suppress_regions,
        },
        region_placement,
    );
}

fn emit_tooltip_nineslice(
    tooltip_nineslice_id: Option<u64>,
    strata_idx: usize,
    widgets: &WidgetRegistry,
    visible: &FxHashSet<u64>,
    out: &mut Vec<u64>,
    suppress_regions: bool,
) {
    if let Some(nineslice_id) = tooltip_nineslice_id {
        dfs_emit_with_region_mode(
            nineslice_id,
            strata_idx,
            widgets,
            visible,
            out,
            suppress_regions,
        );
    }
}

fn region_placement_for_frame(
    frame: &crate::widget::Frame,
    suppress_regions: bool,
) -> RegionPlacement {
    if !suppress_regions && frame.widget_type == crate::widget::WidgetType::EditBox {
        return RegionPlacement::BeforeFrame;
    }
    if !suppress_regions && frame.widget_type == crate::widget::WidgetType::GameTooltip {
        return RegionPlacement::TexturesBeforeFrame;
    }
    RegionPlacement::AfterFrame
}

#[derive(Clone, Copy)]
enum RegionPlacement {
    BeforeFrame,
    AfterFrame,
    TexturesBeforeFrame,
}

struct RegionEmitContext<'a> {
    strata_idx: usize,
    widgets: &'a WidgetRegistry,
    visible: &'a FxHashSet<u64>,
    out: &'a mut Vec<u64>,
    suppress_regions: bool,
}

fn emit_frame_with_regions(
    id: u64,
    regions: Vec<RegionEntry>,
    child_frames: &mut [u64],
    ctx: RegionEmitContext<'_>,
    placement: RegionPlacement,
) {
    let mut deferred_font_regions =
        emit_regions_around_frame(id, regions, ctx.widgets, ctx.out, placement);
    emit_child_frames_and_hoisted(
        child_frames,
        ctx.strata_idx,
        ctx.widgets,
        ctx.visible,
        ctx.out,
        ctx.suppress_regions,
        &mut deferred_font_regions,
    );
    sort_regions(&mut deferred_font_regions, ctx.widgets);
    ctx.out
        .extend(deferred_font_regions.into_iter().map(|entry| entry.id));
}

fn emit_regions_around_frame(
    id: u64,
    regions: Vec<RegionEntry>,
    widgets: &WidgetRegistry,
    out: &mut Vec<u64>,
    placement: RegionPlacement,
) -> Vec<RegionEntry> {
    match placement {
        RegionPlacement::BeforeFrame => {
            emit_all_regions(regions, widgets, out);
            out.push(id);
            Vec::new()
        }
        RegionPlacement::AfterFrame => {
            out.push(id);
            emit_immediate_regions(regions, widgets, out)
        }
        RegionPlacement::TexturesBeforeFrame => {
            emit_textures_before_frame(id, regions, widgets, out)
        }
    }
}

fn emit_textures_before_frame(
    id: u64,
    mut regions: Vec<RegionEntry>,
    widgets: &WidgetRegistry,
    out: &mut Vec<u64>,
) -> Vec<RegionEntry> {
    sort_regions(&mut regions, widgets);
    let (immediate_regions, deferred_regions) = split_font_regions(regions, widgets);
    out.extend(immediate_regions.into_iter().map(|entry| entry.id));
    out.push(id);
    deferred_regions
}

fn tooltip_nineslice_child(
    frame: &crate::widget::Frame,
    strata_idx: usize,
    widgets: &WidgetRegistry,
    visible: &FxHashSet<u64>,
) -> Option<u64> {
    if frame.widget_type != crate::widget::WidgetType::GameTooltip {
        return None;
    }
    let nineslice_id = *frame.children_keys.get("NineSlice")?;
    if !visible.contains(&nineslice_id) {
        return None;
    }
    let child = widgets.get(nineslice_id)?;
    (child.frame_strata.as_index() == strata_idx).then_some(nineslice_id)
}

fn collect_regions_and_children(
    frame: &crate::widget::Frame,
    strata_idx: usize,
    widgets: &WidgetRegistry,
    visible: &FxHashSet<u64>,
    suppress_regions: bool,
) -> (Vec<RegionEntry>, Vec<u64>) {
    let mut regions = Vec::new();
    let mut child_frames = Vec::new();
    if suppress_regions {
        collect_frame_children_only(frame, strata_idx, widgets, visible, &mut child_frames);
    } else {
        collect_frame_regions(
            frame,
            strata_idx,
            widgets,
            visible,
            0,
            &mut regions,
            &mut child_frames,
        );
    }
    (regions, child_frames)
}

fn emit_immediate_regions(
    mut regions: Vec<RegionEntry>,
    widgets: &WidgetRegistry,
    out: &mut Vec<u64>,
) -> Vec<RegionEntry> {
    sort_regions(&mut regions, widgets);
    let (immediate_regions, deferred_regions) = split_font_regions(regions, widgets);
    out.extend(immediate_regions.into_iter().map(|entry| entry.id));
    deferred_regions
}

fn emit_all_regions(mut regions: Vec<RegionEntry>, widgets: &WidgetRegistry, out: &mut Vec<u64>) {
    sort_regions(&mut regions, widgets);
    out.extend(regions.into_iter().map(|entry| entry.id));
}

fn emit_child_frames_and_hoisted(
    child_frames: &mut [u64],
    strata_idx: usize,
    widgets: &WidgetRegistry,
    visible: &FxHashSet<u64>,
    out: &mut Vec<u64>,
    suppress_regions: bool,
    deferred_regions: &mut Vec<RegionEntry>,
) {
    sort_child_frames(child_frames, widgets);
    for &child_id in child_frames.iter() {
        let child = widgets.get(child_id);
        let child_suppress = child.is_some_and(|frame| {
            is_regionless_transparent_wrapper(frame, strata_idx, widgets, visible)
        });
        if !suppress_regions && child_suppress {
            let mut hoisted_regions = Vec::new();
            collect_transparent_wrapper_regions(
                child_id,
                strata_idx,
                widgets,
                visible,
                1,
                &mut hoisted_regions,
            );
            let deferred_hoisted = emit_immediate_regions(hoisted_regions, widgets, out);
            deferred_regions.extend(deferred_hoisted);
        }
        dfs_emit_with_region_mode(child_id, strata_idx, widgets, visible, out, child_suppress);
    }
}

fn collect_frame_regions(
    frame: &crate::widget::Frame,
    strata_idx: usize,
    widgets: &WidgetRegistry,
    visible: &FxHashSet<u64>,
    depth: u32,
    regions: &mut Vec<RegionEntry>,
    child_frames: &mut Vec<u64>,
) {
    collect_frame_regions_inner(
        frame,
        strata_idx,
        widgets,
        visible,
        depth,
        regions,
        child_frames,
    );
}

fn collect_frame_regions_inner(
    frame: &crate::widget::Frame,
    strata_idx: usize,
    widgets: &WidgetRegistry,
    visible: &FxHashSet<u64>,
    depth: u32,
    regions: &mut Vec<RegionEntry>,
    child_frames: &mut Vec<u64>,
) {
    for &child_id in &frame.children {
        collect_child_for_emit(
            child_id,
            strata_idx,
            widgets,
            visible,
            depth,
            regions,
            child_frames,
        );
    }
}

fn collect_child_for_emit(
    child_id: u64,
    strata_idx: usize,
    widgets: &WidgetRegistry,
    visible: &FxHashSet<u64>,
    depth: u32,
    regions: &mut Vec<RegionEntry>,
    child_frames: &mut Vec<u64>,
) {
    if !visible.contains(&child_id) {
        return;
    }
    let Some(child) = widgets.get(child_id) else {
        return;
    };
    if is_region(child.widget_type) {
        regions.push(RegionEntry {
            depth,
            id: child_id,
        });
        return;
    }
    if child.frame_strata.as_index() == strata_idx {
        child_frames.push(child_id);
    }
}

fn collect_transparent_wrapper_regions(
    id: u64,
    strata_idx: usize,
    widgets: &WidgetRegistry,
    visible: &FxHashSet<u64>,
    depth: u32,
    regions: &mut Vec<RegionEntry>,
) {
    let Some(frame) = widgets.get(id) else {
        return;
    };
    for &child_id in &frame.children {
        if !visible.contains(&child_id) {
            continue;
        }
        let Some(child) = widgets.get(child_id) else {
            continue;
        };
        if is_region(child.widget_type) {
            regions.push(RegionEntry {
                depth,
                id: child_id,
            });
            continue;
        }
        if child.frame_strata.as_index() == strata_idx && is_transparent_wrapper(child) {
            collect_transparent_wrapper_regions(
                child_id,
                strata_idx,
                widgets,
                visible,
                depth + 1,
                regions,
            );
        }
    }
}

fn collect_frame_children_only(
    frame: &crate::widget::Frame,
    strata_idx: usize,
    widgets: &WidgetRegistry,
    visible: &FxHashSet<u64>,
    child_frames: &mut Vec<u64>,
) {
    for &child_id in &frame.children {
        if !visible.contains(&child_id) {
            continue;
        }
        let Some(child) = widgets.get(child_id) else {
            continue;
        };
        if is_region(child.widget_type) {
            continue;
        }
        if child.frame_strata.as_index() == strata_idx {
            child_frames.push(child_id);
        }
    }
}

fn is_transparent_wrapper(frame: &crate::widget::Frame) -> bool {
    matches!(
        frame.widget_type,
        crate::widget::WidgetType::Frame | crate::widget::WidgetType::ScrollFrame
    ) && !is_strata_root_boundary(frame)
}

fn is_regionless_transparent_wrapper(
    frame: &crate::widget::Frame,
    strata_idx: usize,
    widgets: &WidgetRegistry,
    visible: &FxHashSet<u64>,
) -> bool {
    is_transparent_wrapper(frame)
        && !has_visible_same_strata_region_child(frame, strata_idx, widgets, visible)
}

fn has_visible_same_strata_region_child(
    frame: &crate::widget::Frame,
    strata_idx: usize,
    widgets: &WidgetRegistry,
    visible: &FxHashSet<u64>,
) -> bool {
    frame.children.iter().copied().any(|child_id| {
        visible.contains(&child_id)
            && widgets.get(child_id).is_some_and(|child| {
                is_region(child.widget_type) && child.frame_strata.as_index() == strata_idx
            })
    })
}

pub(super) fn collect_same_strata_subtree_ids(
    id: u64,
    strata_idx: usize,
    widgets: &WidgetRegistry,
    out: &mut FxHashSet<u64>,
) {
    if !out.insert(id) {
        return;
    }
    let Some(frame) = widgets.get(id) else {
        return;
    };
    for &child_id in &frame.children {
        let Some(child) = widgets.get(child_id) else {
            continue;
        };
        if is_region(child.widget_type) {
            out.insert(child_id);
            continue;
        }
        if child.frame_strata.as_index() == strata_idx {
            collect_same_strata_subtree_ids(child_id, strata_idx, widgets, out);
        }
    }
}

pub(super) fn same_strata_subtree_segment_end(
    bucket: &[u64],
    start: usize,
    subtree_ids: &FxHashSet<u64>,
) -> usize {
    let mut end = start + 1;
    while end < bucket.len() && subtree_ids.contains(&bucket[end]) {
        end += 1;
    }
    end
}

fn sort_regions(regions: &mut [RegionEntry], widgets: &WidgetRegistry) {
    regions.sort_by(|a, b| {
        let (frame_a, frame_b) = match (widgets.get(a.id), widgets.get(b.id)) {
            (Some(frame_a), Some(frame_b)) => (frame_a, frame_b),
            _ => return a.id.cmp(&b.id),
        };
        match region_sort_key(*a, frame_a).cmp(&region_sort_key(*b, frame_b)) {
            std::cmp::Ordering::Equal if frame_a.parent_id == frame_b.parent_id => {
                (frame_a.region_order, a.id).cmp(&(frame_b.region_order, b.id))
            }
            std::cmp::Ordering::Equal => a.id.cmp(&b.id),
            structural_order => structural_order,
        }
    });
}

fn region_sort_key(entry: RegionEntry, frame: &crate::widget::Frame) -> (u32, i32, i32, u8) {
    let text_region = matches!(
        frame.widget_type,
        crate::widget::WidgetType::FontString | crate::widget::WidgetType::SimpleHTML
    ) as u8;
    (
        entry.depth,
        frame.draw_layer as i32,
        frame.draw_sub_layer,
        text_region,
    )
}

fn split_font_regions(
    regions: Vec<RegionEntry>,
    widgets: &WidgetRegistry,
) -> (Vec<RegionEntry>, Vec<RegionEntry>) {
    regions.into_iter().partition(|entry| {
        widgets
            .get(entry.id)
            .is_none_or(|frame| frame.widget_type != crate::widget::WidgetType::FontString)
    })
}

fn sort_child_frames(frames: &mut [u64], widgets: &WidgetRegistry) {
    frames.sort_by(|&a, &b| match (widgets.get(a), widgets.get(b)) {
        (Some(frame_a), Some(frame_b)) => (frame_a.frame_level, frame_a.raise_order, a).cmp(&(
            frame_b.frame_level,
            frame_b.raise_order,
            b,
        )),
        _ => a.cmp(&b),
    });
}

#[cfg(test)]
#[path = "state_render_buckets_tests.rs"]
mod tests;
