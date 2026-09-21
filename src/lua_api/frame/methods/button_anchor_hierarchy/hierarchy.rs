//! Hierarchy methods (parent/children/regions) and create-region methods.

#[cfg(feature = "forbidden-aspects")]
use crate::lua_api::frame::methods::forbidden_aspects;
use crate::lua_api::frame::methods::methods_helpers::{
    can_change_protected_state_for, emit_addon_action_blocked,
};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, clear_child_from_rilua_parent_key,
    create_string, extract_frame_id, frame_id_from_stack, frame_ref, table_get,
};
use crate::lua_bridge::{FromStack, IntoStack, stack_val};
use crate::widget::{AnchorPoint, Frame, WidgetRegistry};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

use super::font_strings::resolve_child_name;
use super::shared::{bind_named_child_global, opt_string};

// ── Hierarchy methods ─────────────────────────────────────────────────────────

pub(super) fn get_parent(state: &mut LuaState) -> LuaResult<u32> {
    use super::shared::frame_global_or_ref;
    let id = frame_id_from_stack(state, 1)?;
    let parent_id = {
        let sim = borrow_state(state)?;
        sim.widgets.get(id).and_then(|f| f.parent_id)
    };
    match parent_id {
        Some(pid) => {
            let val = frame_global_or_ref(state, pid)?;
            #[cfg(feature = "retail-12-1-0")]
            let val = crate::lua_api::script_object_transfer::project_parent_return(state, val)?;
            state.push(val);
            Ok(1)
        }
        None => {
            state.push(Val::Nil);
            Ok(1)
        }
    }
}

pub(super) fn set_parent(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let parent_val = stack_val(state, 2);
    let new_parent_id = extract_frame_id(state, parent_val);
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "SetParent");
        return Ok(0);
    }
    #[cfg(feature = "forbidden-aspects")]
    if let Some(parent_id) = new_parent_id {
        forbidden_aspects::ensure_forbidden_aspects_already_owned(
            state,
            id,
            parent_id,
            forbidden_aspects::INHERITANCE_PARENT,
            "SetParent",
        )?;
    }
    if super::animations::reparent_animation(state, id)? {
        return Ok(0);
    }
    let mut sim = borrow_state_mut(state)?;
    apply_parent_change(&mut sim, id, new_parent_id);
    Ok(0)
}

pub(super) fn apply_parent_change(
    sim: &mut crate::lua_api::SimState,
    id: u64,
    new_parent_id: Option<u64>,
) {
    let old_parent_id = sim.widgets.get(id).and_then(|frame| frame.parent_id);
    let retarget_all_points = should_retarget_parent_fill_anchors(&sim.widgets, id, old_parent_id);
    super::super::methods_hierarchy::reparent_widget(&mut sim.widgets, id, new_parent_id);
    if old_parent_id != new_parent_id {
        sim.invalidate_strata_buckets();
        sim.queue_hit_grid_eligibility_change(id);
    }
    if retarget_all_points {
        retarget_parent_fill_anchors(&mut sim.widgets, id, new_parent_id);
    }
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.default_parent = false;
    }
    sim.visible_on_update_cache = None;
    sim.widgets.mark_rect_dirty(id);
}

fn should_retarget_parent_fill_anchors(
    widgets: &WidgetRegistry,
    frame_id: u64,
    parent_id: Option<u64>,
) -> bool {
    let Some(parent_id) = parent_id else {
        return false;
    };
    let Some(frame) = widgets.get(frame_id) else {
        return false;
    };
    frame.xml_set_all_points || frame_fills_parent(frame, parent_id)
}

fn frame_fills_parent(frame: &Frame, parent_id: u64) -> bool {
    frame.anchors.len() == 2
        && has_parent_fill_anchor(frame, AnchorPoint::TopLeft, parent_id)
        && has_parent_fill_anchor(frame, AnchorPoint::BottomRight, parent_id)
}

fn has_parent_fill_anchor(frame: &Frame, point: AnchorPoint, parent_id: u64) -> bool {
    frame.anchors.iter().any(|anchor| {
        anchor.point == point
            && anchor.relative_point == point
            && anchor.relative_to_id == Some(parent_id as usize)
            && anchor.x_offset == 0.0
            && anchor.y_offset == 0.0
    })
}

fn retarget_parent_fill_anchors(
    widgets: &mut WidgetRegistry,
    frame_id: u64,
    new_parent_id: Option<u64>,
) {
    widgets.remove_all_anchor_dependents_for(frame_id);
    if let Some(frame) = widgets.get_mut_visual(frame_id) {
        frame.clear_all_points();
        frame.set_point(
            AnchorPoint::TopLeft,
            new_parent_id.map(|id| id as usize),
            AnchorPoint::TopLeft,
            0.0,
            0.0,
        );
        frame.set_point(
            AnchorPoint::BottomRight,
            new_parent_id.map(|id| id as usize),
            AnchorPoint::BottomRight,
            0.0,
            0.0,
        );
    }
    if let Some(parent_id) = new_parent_id {
        widgets.add_anchor_dependent(parent_id, frame_id);
    }
}

pub(super) fn get_num_children(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let count = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|f| {
                f.children
                    .iter()
                    .filter(|&&cid| is_frame_child(&sim, cid))
                    .count()
            })
            .unwrap_or(0) as i32
    };
    count.into_stack(state)
}

pub(super) fn get_children(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let children = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|f| {
                f.children
                    .iter()
                    .copied()
                    .filter(|&cid| !is_region_type(&sim, cid))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };
    let mut count = 0u32;
    for child_id in children {
        let is_child_frame = {
            let sim = borrow_state(state)?;
            is_frame_child(&sim, child_id)
        };
        if !is_child_frame {
            continue;
        }
        let val = frame_ref(state, child_id)?;
        state.push(val);
        count += 1;
    }
    Ok(count)
}

pub(super) fn get_num_regions(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let count = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|f| {
                f.children
                    .iter()
                    .filter(|&&cid| is_region_type(&sim, cid))
                    .count()
            })
            .unwrap_or(0) as i32
    };
    count.into_stack(state)
}

fn is_region_type(sim: &crate::lua_api::SimState, cid: u64) -> bool {
    use crate::widget::WidgetType;
    sim.widgets
        .get(cid)
        .map(|c| {
            matches!(
                c.widget_type,
                WidgetType::Texture | WidgetType::FontString | WidgetType::Line
            )
        })
        .unwrap_or(false)
}

fn is_frame_child(sim: &crate::lua_api::SimState, cid: u64) -> bool {
    sim.widgets
        .get(cid)
        .map(|child| !is_region_widget_type(child.widget_type))
        .unwrap_or(false)
}

fn is_region_widget_type(widget_type: crate::widget::WidgetType) -> bool {
    matches!(
        widget_type,
        crate::widget::WidgetType::Texture
            | crate::widget::WidgetType::FontString
            | crate::widget::WidgetType::Line
    )
}

pub(super) fn get_regions(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let children = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|f| f.children.clone())
            .unwrap_or_default()
    };
    let mut count = 0u32;
    for child_id in children {
        let is_region = {
            let sim = borrow_state(state)?;
            is_region_type(&sim, child_id)
        };
        if is_region {
            let val = frame_ref(state, child_id)?;
            state.push(val);
            count += 1;
        }
    }
    Ok(count)
}

pub(super) fn get_additional_regions(state: &mut LuaState) -> LuaResult<u32> {
    let _id = frame_id_from_stack(state, 1)?;
    Ok(0)
}

pub(super) fn get_parent_key(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let key = {
        let sim = borrow_state(state)?;
        sim.widgets.get(id).and_then(|child| {
            child.parent_key.clone().or_else(|| {
                child
                    .parent_id
                    .and_then(|parent_id| find_parent_key_for_child(&sim.widgets, parent_id, id))
            })
        })
    };
    match key {
        Some(k) => {
            let val = create_string(state, &k);
            state.push(val);
            Ok(1)
        }
        None => {
            state.push(Val::Nil);
            Ok(1)
        }
    }
}

pub(super) fn set_parent_key(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let key = String::from_stack(state, 2)?;
    let clear_existing = bool::from_stack(state, 3)?;
    let parent_id = {
        let sim = borrow_state(state)?;
        sim.widgets.get(id).and_then(|f| f.parent_id)
    };
    let Some(pid) = parent_id else {
        return Ok(0);
    };

    let aliases_to_clear = update_parent_key_state(state, pid, id, &key, clear_existing)?;

    for alias in aliases_to_clear {
        clear_child_from_rilua_parent_key(state, pid, &alias, id)?;
    }
    crate::lua_api::methods::sync_child_to_rilua(state, pid, &key, id)?;
    Ok(0)
}

fn update_parent_key_state(
    state: &mut LuaState,
    parent_id: u64,
    child_id: u64,
    key: &str,
    clear_existing: bool,
) -> LuaResult<Vec<String>> {
    let mut sim = borrow_state_mut(state)?;
    let existing_key = existing_parent_key_for_child(&sim.widgets, parent_id, child_id);
    let aliases_to_clear = if clear_existing {
        parent_keys_for_child(&sim.widgets, parent_id, child_id)
    } else {
        Vec::new()
    };

    if let Some(parent) = sim.widgets.get_mut_visual(parent_id) {
        if clear_existing {
            parent
                .children_keys
                .retain(|_, mapped_id| *mapped_id != child_id);
        }
        parent.children_keys.insert(key.to_string(), child_id);
    }
    if let Some(child) = sim.widgets.get_mut(child_id) {
        child.parent_key = if clear_existing {
            Some(key.to_string())
        } else {
            existing_key.or_else(|| Some(key.to_string()))
        };
    }
    Ok(aliases_to_clear)
}

fn existing_parent_key_for_child(
    widgets: &WidgetRegistry,
    parent_id: u64,
    child_id: u64,
) -> Option<String> {
    widgets
        .get(child_id)
        .and_then(|child| child.parent_key.clone())
        .or_else(|| find_parent_key_for_child(widgets, parent_id, child_id))
}

fn find_parent_key_for_child(
    widgets: &WidgetRegistry,
    parent_id: u64,
    child_id: u64,
) -> Option<String> {
    widgets.get(parent_id).and_then(|parent| {
        parent
            .children_keys
            .iter()
            .find_map(|(key, mapped_id)| (*mapped_id == child_id).then(|| key.clone()))
    })
}

fn parent_keys_for_child(widgets: &WidgetRegistry, parent_id: u64, child_id: u64) -> Vec<String> {
    widgets
        .get(parent_id)
        .map(|parent| {
            parent
                .children_keys
                .iter()
                .filter_map(|(key, mapped_id)| (*mapped_id == child_id).then(|| key.clone()))
                .collect()
        })
        .unwrap_or_default()
}

// ── Create methods ────────────────────────────────────────────────────────────

fn apply_draw_layer(frame: &mut crate::widget::Frame, layer: Option<String>) {
    use crate::widget::DrawLayer;
    if let Some(layer_str) = layer {
        if let Some(draw_layer) = DrawLayer::from_str(&layer_str) {
            frame.draw_layer = draw_layer;
        }
    }
}

fn register_child_with_strata(
    state: &mut LuaState,
    parent_id: u64,
    child_id: u64,
    widget: crate::widget::Frame,
) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let parent_props = sim
        .widgets
        .get(parent_id)
        .map(|p| (p.frame_strata, p.frame_level));
    sim.widgets.register(widget);
    sim.widgets.add_child(parent_id, child_id);
    sim.invalidate_strata_buckets();
    if let Some((parent_strata, parent_level)) = parent_props {
        if let Some(f) = sim.widgets.get_mut_visual(child_id) {
            f.frame_strata = parent_strata;
            f.frame_level = parent_level + 1;
        }
    }
    Ok(())
}

pub(super) fn create_texture(state: &mut LuaState) -> LuaResult<u32> {
    create_texture_like(state, None)
}

#[cfg(feature = "retail-12-1-0")]
pub(super) fn create_vector_graphics(state: &mut LuaState) -> LuaResult<u32> {
    create_texture_like(state, Some("VectorGraphics"))
}

fn create_texture_like(state: &mut LuaState, object_type_name: Option<&str>) -> LuaResult<u32> {
    use crate::widget::{Frame, WidgetType};
    let parent_id = frame_id_from_stack(state, 1)?;
    let name_raw: Option<String> = Option::<String>::from_stack(state, 2)?;
    let layer = opt_string(state, 3);
    let inherits = opt_string(state, 4);
    let sub_level = super::shared::opt_f32(state, 5).map(|n| n as i32);

    let name = resolve_child_name(state, name_raw, parent_id);
    let mut texture = Frame::new(WidgetType::Texture, name.clone(), Some(parent_id));
    if let Some(object_type_name) = object_type_name {
        texture.object_type_name = Some(object_type_name.to_string());
    }
    apply_draw_layer(&mut texture, layer);
    if let Some(inherits) = inherits.as_deref()
        && let Some((width, height)) = crate::xml::get_texture_template_size(&inherits)
    {
        texture.set_size(width, height);
    }
    if let Some(sub_level) = sub_level {
        texture.draw_sub_layer = sub_level;
    }

    let child_id = texture.id;
    register_child_with_strata(state, parent_id, child_id, texture)?;

    if let Some(ref n) = name {
        bind_named_child_global(state, n, child_id)?;
    }
    let val = frame_ref(state, child_id)?;
    apply_texture_template_effects(state, val, inherits.as_deref())?;
    state.push(val);
    Ok(1)
}

fn register_child_widget(
    state: &mut LuaState,
    parent_id: u64,
    child_id: u64,
    widget: crate::widget::Frame,
) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    sim.widgets.register(widget);
    sim.widgets.add_child(parent_id, child_id);
    sim.invalidate_strata_buckets();
    Ok(())
}

pub(super) fn create_mask_texture(state: &mut LuaState) -> LuaResult<u32> {
    use crate::widget::{Frame, WidgetType};
    let parent_id = frame_id_from_stack(state, 1)?;
    let name_raw: Option<String> = Option::<String>::from_stack(state, 2)?;
    let name = resolve_child_name(state, name_raw, parent_id);
    let mut texture = Frame::new(WidgetType::Texture, name.clone(), Some(parent_id));
    texture.is_mask = true;
    texture.object_type_name = Some("MaskTexture".to_string());
    let child_id = texture.id;
    register_child_widget(state, parent_id, child_id, texture)?;
    if let Some(ref n) = name {
        bind_named_child_global(state, n, child_id)?;
    }
    let val = frame_ref(state, child_id)?;
    state.push(val);
    Ok(1)
}

pub(super) fn add_mask_texture(state: &mut LuaState) -> LuaResult<u32> {
    let texture_id = frame_id_from_stack(state, 1)?;
    let Some(mask_id) = extract_frame_id(state, stack_val(state, 2)) else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    let is_mask = sim.widgets.get(mask_id).map(|f| f.is_mask).unwrap_or(false);
    if !is_mask {
        return Ok(0);
    }
    if let Some(texture) = sim.widgets.get_mut_visual(texture_id) {
        add_mask_texture_id(texture, mask_id);
    }
    Ok(0)
}

fn add_mask_texture_id(texture: &mut crate::widget::Frame, mask_id: u64) {
    if texture
        .mask_textures
        .iter()
        .any(|existing| *existing == mask_id)
    {
        return;
    }
    texture.mask_textures.push(mask_id);
}

pub(super) fn remove_mask_texture(state: &mut LuaState) -> LuaResult<u32> {
    let texture_id = frame_id_from_stack(state, 1)?;
    let Some(mask_id) = extract_frame_id(state, stack_val(state, 2)) else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    if let Some(texture) = sim.widgets.get_mut_visual(texture_id) {
        texture.mask_textures.retain(|id| *id != mask_id);
    }
    Ok(0)
}

pub(super) fn get_num_mask_textures(state: &mut LuaState) -> LuaResult<u32> {
    let texture_id = frame_id_from_stack(state, 1)?;
    let count = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(texture_id)
            .map(|f| f.mask_textures.len())
            .unwrap_or(0)
    };
    state.push(Val::Num(count as f64));
    Ok(1)
}

pub(super) fn get_mask_texture(state: &mut LuaState) -> LuaResult<u32> {
    let texture_id = frame_id_from_stack(state, 1)?;
    let index = i64::from_stack(state, 2).unwrap_or(1);
    let mask_id = {
        let sim = borrow_state(state)?;
        if index <= 0 {
            None
        } else {
            sim.widgets
                .get(texture_id)
                .and_then(|f| f.mask_textures.get((index - 1) as usize).copied())
        }
    };
    if let Some(mask_id) = mask_id {
        let mask_ref = frame_ref(state, mask_id)?;
        state.push(mask_ref);
    } else {
        state.push(Val::Nil);
    }
    Ok(1)
}

pub(super) fn create_line(state: &mut LuaState) -> LuaResult<u32> {
    use crate::widget::{Frame, WidgetType};
    let parent_id = frame_id_from_stack(state, 1)?;
    let name_raw: Option<String> = Option::<String>::from_stack(state, 2)?;
    let layer = opt_string(state, 3);
    let inherits = opt_string(state, 4);
    let name = resolve_child_name(state, name_raw, parent_id);
    let mut line = Frame::new(WidgetType::Line, name.clone(), Some(parent_id));
    apply_draw_layer(&mut line, layer);
    let child_id = line.id;
    register_child_widget(state, parent_id, child_id, line)?;
    let val = frame_ref(state, child_id)?;
    apply_line_template_effects(state, child_id, val, inherits.as_deref())?;
    state.push(val);
    Ok(1)
}

fn apply_line_template_effects(
    state: &mut LuaState,
    line_id: u64,
    line: Val,
    inherits: Option<&str>,
) -> LuaResult<()> {
    let Some(inherits) = inherits.filter(|value| !value.trim().is_empty()) else {
        return Ok(());
    };
    if !crate::xml::get_template_chain(inherits).is_empty() {
        crate::lua_api::globals::create_frame::apply_runtime_template_chain(
            state,
            line_id,
            Some(inherits),
            true,
        )?;
        return Ok(());
    }
    apply_texture_template_effects(state, line, Some(inherits))
}

fn apply_texture_template_effects(
    state: &mut LuaState,
    widget: Val,
    inherits: Option<&str>,
) -> LuaResult<()> {
    let Some(inherits) = inherits.filter(|value| !value.trim().is_empty()) else {
        return Ok(());
    };
    let texture = crate::xml::TextureXml {
        inherits: Some(inherits.to_string()),
        ..Default::default()
    };
    apply_texture_template_mixins(state, widget, &texture)?;
    apply_texture_template_key_values(state, widget, &texture);
    Ok(())
}

fn apply_texture_template_mixins(
    state: &mut LuaState,
    widget: Val,
    texture: &crate::xml::TextureXml,
) -> LuaResult<()> {
    let globals = Val::Table(state.global);
    let mixin_func = table_get(state, globals, "Mixin");
    if !matches!(mixin_func, Val::Function(_)) {
        return Ok(());
    }
    for mixin_name in crate::xml::collect_texture_mixins(texture) {
        let mixin_table = table_get(state, globals, &mixin_name);
        if matches!(mixin_table, Val::Table(_)) {
            call_function_state(state, mixin_func, &[widget, mixin_table])?;
        }
    }
    Ok(())
}

fn apply_texture_template_key_values(
    state: &mut LuaState,
    widget: Val,
    texture: &crate::xml::TextureXml,
) {
    let Val::Table(table_ref) = widget else {
        return;
    };
    for key_value in crate::xml::collect_texture_key_values(texture) {
        let key = create_string(state, &key_value.key);
        let value =
            texture_template_key_value(state, &key_value.value, key_value.value_type.as_deref());
        if let Some(table) = state.gc.tables.get_mut(table_ref) {
            let _ = table.raw_set(key, value, &state.gc.string_arena);
        }
        state.gc.barrier_back(table_ref);
    }
}

fn texture_template_key_value(state: &mut LuaState, value: &str, value_type: Option<&str>) -> Val {
    match value_type {
        Some("number") => value.parse::<f64>().map(Val::Num).unwrap_or(Val::Nil),
        Some("boolean") => Val::Bool(value.eq_ignore_ascii_case("true")),
        Some("global") => resolve_global_path(state, value),
        None if value.parse::<f64>().is_ok() => Val::Num(value.parse().unwrap()),
        _ => create_string(state, value),
    }
}

fn resolve_global_path(state: &mut LuaState, path: &str) -> Val {
    let mut current = Val::Table(state.global);
    for part in path.split('.') {
        if part.is_empty() {
            return Val::Nil;
        }
        current = table_get(state, current, part);
        if matches!(current, Val::Nil) {
            break;
        }
    }
    current
}

pub(super) fn attach_texture(state: &mut LuaState) -> LuaResult<u32> {
    use crate::widget::{Frame, WidgetType};
    let parent_id = frame_id_from_stack(state, 1)?;
    let texture = Frame::new(WidgetType::Texture, None, Some(parent_id));
    let child_id = texture.id;
    register_child_widget(state, parent_id, child_id, texture)?;
    let val = frame_ref(state, child_id)?;
    state.push(val);
    Ok(1)
}

pub(super) fn attach_font_string(state: &mut LuaState) -> LuaResult<u32> {
    use crate::widget::{Frame, WidgetType};
    let parent_id = frame_id_from_stack(state, 1)?;
    let fontstring = Frame::new(WidgetType::FontString, None, Some(parent_id));
    let child_id = fontstring.id;
    register_child_widget(state, parent_id, child_id, fontstring)?;
    let val = frame_ref(state, child_id)?;
    state.push(val);
    Ok(1)
}
