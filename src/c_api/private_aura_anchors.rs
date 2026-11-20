//! Public private-aura anchor registration, independent of aura content and rendering.

mod input;
mod warning;

use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, create_table, frame_ref,
    registry_get, registry_set, table_set, table_set_num,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use std::collections::BTreeMap;

const ADDED_CALLBACK: &str = "__private_aura_anchor_added_callback";
const REMOVED_CALLBACK: &str = "__private_aura_anchor_removed_callback";

/// Parsed binding data retained for future AnchorPrivateAura application, not rendered here.
#[derive(Clone)]
pub struct AnchorBinding {
    pub point: crate::widget::AnchorPoint,
    pub relative_to: u64,
    pub relative_point: crate::widget::AnchorPoint,
    pub offset_x: f64,
    pub offset_y: f64,
}

#[derive(Clone)]
pub struct IconInfo {
    pub anchor: AnchorBinding,
    pub width: f64,
    pub height: f64,
    pub border_scale: Option<f64>,
}

/// Only Rust-owned metadata and canonical frame IDs; no unrooted Lua values.
#[derive(Clone)]
pub struct AnchorRecord {
    pub unit_token: String,
    pub aura_index: f64,
    pub parent: u64,
    pub show_cooldown_frame: bool,
    pub show_cooldown_edge: bool,
    pub show_countdown_numbers: bool,
    pub show_dispel_icon: bool,
    pub is_container: bool,
    pub icon: Option<IconInfo>,
    pub duration_anchor: Option<AnchorBinding>,
}

pub(crate) struct PrivateAuraAnchors {
    next_id: u64,
    records: BTreeMap<u64, AnchorRecord>,
    warning: warning::WarningPlacement,
}

impl Default for PrivateAuraAnchors {
    fn default() -> Self {
        Self {
            next_id: 1,
            records: BTreeMap::new(),
            warning: warning::WarningPlacement::default(),
        }
    }
}

/// Unconditional like the replaced private namespace owner, not aura-enumeration gated.
pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let public = super::ensure_namespace(state, "C_UnitAuras")?;
    table_set_rust_fn_static(state, public, "AddPrivateAuraAnchor", add)?;
    table_set_rust_fn_static(state, public, "RemovePrivateAuraAnchor", remove)?;
    let private = super::ensure_namespace(state, "C_UnitAurasPrivate")?;
    table_set_rust_fn_static(
        state,
        private,
        "SetPrivateAuraAnchorAddedCallback",
        set_added,
    )?;
    table_set_rust_fn_static(
        state,
        private,
        "SetPrivateAuraAnchorRemovedCallback",
        set_removed,
    )?;
    table_set_rust_fn_static(state, private, "GetPrivateAuraAnchors", list)?;
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    table_set_rust_fn_static(state, private, "AnchorPrivateAura", anchor_private_aura)?;
    warning::register(state, public, private)
}

/// INFERRED while R88bw is pending: the consumer ignores this placement call's return values.
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn anchor_private_aura(state: &mut LuaState) -> LuaResult<u32> {
    let aura_frame = input::read_frame(state, stack_val(state, 1), "auraFrame")?;
    let icon_frame = input::read_frame(state, stack_val(state, 2), "icon")?;
    let duration_frame = input::read_frame(state, stack_val(state, 3), "duration")?;
    let anchor_id = input::read_number(state, stack_val(state, 4), "anchorID")?;
    if anchor_id < 1.0 || anchor_id.fract() != 0.0 || anchor_id >= (1_u64 << 53) as f64 {
        return Err(runtime_error("private aura anchor anchorID is invalid"));
    }
    let (record, icon) = private_aura_placement(state, anchor_id as u64)?;
    let regions = AuraRegions {
        frame: aura_frame,
        icon: icon_frame,
        duration: duration_frame,
    };
    apply_private_aura_placement(state, regions, &record, icon)?;
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn private_aura_placement(state: &LuaState, anchor_id: u64) -> LuaResult<(AnchorRecord, i32)> {
    let sim = borrow_state(state)?;
    let record = sim
        .private_aura_anchors
        .records
        .get(&anchor_id)
        .cloned()
        .ok_or_else(|| runtime_error("private aura anchor ID is not registered"))?;
    let index = record.aura_index;
    if index < 1.0 || index.fract() != 0.0 || index > usize::MAX as f64 {
        return Err(runtime_error("private aura anchor auraIndex is invalid"));
    }
    let aura = sim
        .private_aura_data
        .private_auras_by_unit
        .get(&record.unit_token)
        .and_then(|auras| auras.get(index as usize - 1))
        .map(|entry| entry.aura.icon)
        .ok_or_else(|| runtime_error("private aura anchor aura data is unavailable"))?;
    Ok((record, aura))
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[derive(Clone, Copy)]
struct AuraRegions {
    frame: u64,
    icon: u64,
    duration: u64,
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn apply_private_aura_placement(
    state: &mut LuaState,
    regions: AuraRegions,
    record: &AnchorRecord,
    icon: i32,
) -> LuaResult<()> {
    let (was_visible, is_visible) = reparent_aura_frame(state, regions.frame, record.parent)?;
    apply_region_anchors(state, regions, record, icon)?;
    if was_visible != is_visible {
        crate::lua_api::frame::methods::core_state::visibility::dispatch_parent_visibility_change(
            state,
            regions.frame,
            Some(record.parent),
            is_visible,
        )?;
    }
    Ok(())
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn reparent_aura_frame(
    state: &mut LuaState,
    frame_id: u64,
    parent_id: u64,
) -> LuaResult<(bool, bool)> {
    use crate::lua_api::frame::methods::button_anchor_hierarchy::apply_parent_change;
    use crate::lua_api::methods::{borrow_state_mut, clear_child_from_rilua_parent_key};
    let sim = borrow_state_mut(state)?;
    if crate::lua_api::frame::methods::methods_hierarchy::would_create_parent_cycle(
        &sim.widgets,
        frame_id,
        Some(parent_id),
    ) {
        return Err(runtime_error("private aura anchor parent creates a cycle"));
    }
    let was_visible = sim.widgets.is_ancestor_visible(frame_id);
    let old_parent_key = sim.widgets.get(frame_id).and_then(|frame| {
        if frame.parent_id == Some(parent_id) {
            None
        } else {
            frame.parent_id.zip(frame.parent_key.clone())
        }
    });
    drop(sim);
    if let Some((old_parent, key)) = old_parent_key {
        clear_child_from_rilua_parent_key(state, old_parent, &key, frame_id)?;
    }
    let mut sim = borrow_state_mut(state)?;
    apply_parent_change(&mut sim, frame_id, Some(parent_id));
    sim.widgets.mark_anchor_rect_dirty(frame_id);
    Ok((was_visible, sim.widgets.is_ancestor_visible(frame_id)))
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn apply_region_anchors(
    state: &mut LuaState,
    regions: AuraRegions,
    record: &AnchorRecord,
    icon: i32,
) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    if let Some(icon_info) = &record.icon {
        apply_anchor_binding(&mut sim, regions.frame, &icon_info.anchor);
        apply_anchor_binding(&mut sim, regions.icon, &icon_info.anchor);
        set_aura_icon(&mut sim, regions.icon, icon_info, icon);
    }
    if let Some(binding) = &record.duration_anchor {
        apply_anchor_binding(&mut sim, regions.duration, binding);
    }
    sim.widgets.mark_anchor_rect_dirty(regions.frame);
    Ok(())
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_aura_icon(sim: &mut crate::lua_api::SimState, icon_id: u64, info: &IconInfo, file_id: i32) {
    if let Some(icon) = sim.widgets.get_mut_for_anchor_edit(icon_id) {
        icon.set_size(info.width as f32, info.height as f32);
        icon.texture = None;
        icon.texture_source_dimensions = None;
        icon.texture_file_data_id = Some(file_id as i64);
        icon.secret_texture = false;
        icon.color_texture = None;
        icon.atlas = None;
        icon.atlas_tex_coords = None;
    }
    sim.widgets.mark_visual_dirty(icon_id);
    sim.widgets.mark_anchor_rect_dirty(icon_id);
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn apply_anchor_binding(
    sim: &mut crate::lua_api::SimState,
    frame_id: u64,
    binding: &AnchorBinding,
) {
    sim.widgets.remove_all_anchor_dependents_for(frame_id);
    if let Some(frame) = sim.widgets.get_mut_for_anchor_edit(frame_id) {
        frame.clear_all_points();
        frame.set_point(
            binding.point,
            Some(binding.relative_to as usize),
            binding.relative_point,
            binding.offset_x as f32,
            binding.offset_y as f32,
        );
    }
    sim.widgets
        .add_anchor_dependent(binding.relative_to, frame_id);
}

fn set_callback(state: &mut LuaState, key: &'static str) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    input::reject_secret(state, value, "callback")?;
    if !matches!(value, Val::Nil | Val::Function(_)) {
        return Err(runtime_error(
            "private aura anchor callback must be a function or nil",
        ));
    }
    registry_set(state, key, value);
    Ok(0)
}

fn set_added(state: &mut LuaState) -> LuaResult<u32> {
    set_callback(state, ADDED_CALLBACK)
}

fn set_removed(state: &mut LuaState) -> LuaResult<u32> {
    set_callback(state, REMOVED_CALLBACK)
}

fn dispatch(state: &mut LuaState, key: &'static str, payload: Val) -> LuaResult<()> {
    state.push(payload);
    let callback = registry_get(state, key);
    let result = if matches!(callback, Val::Function(_)) {
        call_function_state(state, callback, &[payload]).map(|_| ())
    } else {
        Ok(())
    };
    state.pop();
    result
}

fn add(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    let record = input::read_record(state, value)?;
    let id = {
        let mut sim = borrow_state_mut(state)?;
        let anchors = &mut sim.private_aura_anchors;
        let id = anchors.next_id;
        // IDs must remain distinct when exposed as Lua numbers.
        if id >= (1_u64 << 53) {
            return Err(runtime_error("private aura anchor IDs exhausted"));
        }
        anchors.next_id += 1;
        anchors.records.insert(id, record.clone());
        id
    };
    let payload = publish_record(state, id, &record)?;
    dispatch(state, ADDED_CALLBACK, payload)?;
    state.push(Val::Num(id as f64));
    Ok(1)
}

fn remove(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    let id = input::read_number(state, value, "anchorID")?;
    // Unknown numeric IDs are inferred no-ops, never cast fractional IDs into live IDs.
    let removed = if id >= 1.0 && id < (1_u64 << 53) as f64 && id.fract() == 0.0 {
        borrow_state_mut(state)?
            .private_aura_anchors
            .records
            .remove(&(id as u64))
            .is_some()
    } else {
        false
    };
    if removed {
        dispatch(state, REMOVED_CALLBACK, Val::Num(id))?;
    }
    Ok(0)
}

fn list(state: &mut LuaState) -> LuaResult<u32> {
    let filter = stack_val(state, 1);
    let filter = if filter == Val::Nil {
        None
    } else {
        Some(input::read_string(state, filter, "unitToken")?)
    };
    let records: Vec<_> = borrow_state(state)?
        .private_aura_anchors
        .records
        .iter()
        .filter(|(_, record)| {
            filter
                .as_ref()
                .is_none_or(|unit| unit == &record.unit_token)
        })
        .map(|(id, record)| (*id, record.clone()))
        .collect();
    let result = create_table(state);
    // Root the growing list while allocating DTOs and resolving frame refs.
    state.push(result);
    if let Val::Table(reference) = result {
        for (index, (id, record)) in records.iter().enumerate() {
            let dto = publish_record(state, *id, record)?;
            table_set_num(state, reference, (index + 1) as f64, dto);
        }
    }
    Ok(1)
}

fn publish_record(state: &mut LuaState, id: u64, record: &AnchorRecord) -> LuaResult<Val> {
    let parent = frame_ref(state, record.parent)?;
    let dto = create_table(state);
    state.push(dto);
    let unit = create_string(state, &record.unit_token);
    table_set(state, dto, "anchorID", Val::Num(id as f64));
    table_set(state, dto, "unitToken", unit);
    table_set(state, dto, "auraIndex", Val::Num(record.aura_index));
    table_set(state, dto, "parent", parent);
    for (name, value) in [
        ("showCooldownFrame", record.show_cooldown_frame),
        ("showCooldownEdge", record.show_cooldown_edge),
        ("showCountdownNumbers", record.show_countdown_numbers),
        ("showDispelIcon", record.show_dispel_icon),
        ("isContainer", record.is_container),
    ] {
        table_set(state, dto, name, Val::Bool(value));
    }
    if let Some(icon) = &record.icon {
        table_set(state, dto, "iconWidth", Val::Num(icon.width));
        table_set(state, dto, "iconHeight", Val::Num(icon.height));
        if let Some(scale) = icon.border_scale {
            table_set(state, dto, "borderScale", Val::Num(scale));
        }
    }
    state.pop();
    Ok(dto)
}
