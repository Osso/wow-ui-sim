//! Inferred private warning placement: public requests target the separately registered frame.

use super::{AnchorBinding, input};
use crate::lua_api::frame::methods::button_anchor_hierarchy::apply_parent_change;
use crate::lua_api::frame::methods::core_state::visibility::dispatch_parent_visibility_change;
use crate::lua_api::frame::methods::methods_hierarchy::would_create_parent_cycle;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, clear_child_from_rilua_parent_key, create_table, table_get,
    table_set,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use crate::widget::WidgetType;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

#[derive(Clone)]
struct WarningRequest {
    parent: u64,
    binding: Option<AnchorBinding>,
}

/// Per-environment typed inputs; canonical frame tables are already pinned by the frame registry.
#[derive(Default)]
pub(super) struct WarningPlacement {
    request: Option<WarningRequest>,
    frame: Option<u64>,
}

pub(super) fn register(
    state: &mut LuaState,
    public: GcRef<Table>,
    private: GcRef<Table>,
) -> LuaResult<()> {
    let compatibility = create_table(state);
    table_set(state, Val::Table(private), "_state", compatibility);
    table_set_rust_fn_static(state, public, "SetPrivateWarningTextAnchor", set_anchor)?;
    table_set_rust_fn_static(state, private, "SetPrivateWarningTextFrame", set_frame)
}

fn read_simple_frame(state: &LuaState, value: Val, field: &str) -> LuaResult<u64> {
    let id = input::read_frame(state, value, field)?;
    validate_region(state, id)?;
    let sim = borrow_state(state)?;
    let frame = sim.widgets.get(id).expect("read_frame checked existence");
    if matches!(
        frame.widget_type,
        WidgetType::Texture | WidgetType::FontString | WidgetType::Line
    ) {
        return Err(runtime_error(format!(
            "private warning {field} must be a SimpleFrame"
        )));
    }
    Ok(id)
}

fn validate_region(state: &LuaState, id: u64) -> LuaResult<()> {
    let sim = borrow_state(state)?;
    // Animations and control points use backing Frame records, but are not ScriptRegions.
    let control_point = sim
        .widgets
        .get(id)
        .is_some_and(|frame| frame.object_type_name.as_deref() == Some("ControlPoint"));
    if sim.anim_frame_to_group.contains_key(&id)
        || sim.anim_frame_to_anim.contains_key(&id)
        || control_point
    {
        return Err(runtime_error(
            "private warning target must be a ScriptRegion",
        ));
    }
    Ok(())
}

fn set_anchor(state: &mut LuaState) -> LuaResult<u32> {
    let parent = read_simple_frame(state, stack_val(state, 1), "parent")?;
    let anchor = stack_val(state, 2);
    let request = WarningRequest {
        parent,
        binding: if anchor == Val::Nil {
            None
        } else {
            Some(input::read_binding(state, anchor, "anchor")?)
        },
    };
    if let Some(binding) = &request.binding {
        validate_region(state, binding.relative_to)?;
    }
    let frame = borrow_state(state)?.private_aura_anchors.warning.frame;
    if let Some(id) = frame {
        validate_placement(state, id, &request)?;
    }
    // Publish the latest request before callbacks, so reentrant registration observes it.
    borrow_state_mut(state)?
        .private_aura_anchors
        .warning
        .request = Some(request.clone());
    if let Some(id) = frame {
        apply_placement(state, id, &request)?;
    }
    Ok(0)
}

fn set_frame(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    let id = read_simple_frame(state, value, "warningTextFrame")?;
    let request = borrow_state(state)?
        .private_aura_anchors
        .warning
        .request
        .clone();
    if let Some(request) = &request {
        validate_placement(state, id, request)?;
    }
    let private = table_get(state, Val::Table(state.global), "C_UnitAurasPrivate");
    let compatibility = table_get(state, private, "_state");
    input::read_table(state, compatibility, "C_UnitAurasPrivate._state")?;
    // Preserve exact registered Lua frame identity, including secure-environment projections.
    table_set(state, compatibility, "warningTextFrame", value);
    borrow_state_mut(state)?.private_aura_anchors.warning.frame = Some(id);
    if let Some(request) = request {
        apply_placement(state, id, &request)?;
    }
    Ok(0)
}

fn validate_placement(state: &mut LuaState, id: u64, request: &WarningRequest) -> LuaResult<()> {
    {
        let sim = borrow_state(state)?;
        if sim.widgets.get(id).is_none() {
            return Err(runtime_error("private warning frame no longer exists"));
        }
        if would_create_parent_cycle(&sim.widgets, id, Some(request.parent)) {
            return Err(runtime_error(
                "private warning placement would create a parent cycle",
            ));
        }
        if let Some(binding) = &request.binding
            && sim
                .widgets
                .would_create_anchor_cycle(id, binding.relative_to)
        {
            return Err(runtime_error(
                "private warning placement would create an anchor cycle",
            ));
        }
    }
    #[cfg(feature = "forbidden-aspects")]
    {
        use crate::lua_api::frame::methods::forbidden_aspects::{
            INHERITANCE_LAYOUT, INHERITANCE_PARENT, ensure_forbidden_aspects_already_owned,
        };
        ensure_forbidden_aspects_already_owned(
            state,
            id,
            request.parent,
            INHERITANCE_PARENT,
            "SetPrivateWarningTextAnchor",
        )?;
        if let Some(binding) = &request.binding {
            ensure_forbidden_aspects_already_owned(
                state,
                id,
                binding.relative_to,
                INHERITANCE_LAYOUT,
                "SetPrivateWarningTextAnchor",
            )?;
        }
    }
    Ok(())
}

fn apply_placement(state: &mut LuaState, id: u64, request: &WarningRequest) -> LuaResult<()> {
    let (was_visible, old_parent_key) = {
        let sim = borrow_state(state)?;
        let frame = sim.widgets.get(id).expect("validated private frame");
        let key = if frame.parent_id != Some(request.parent) {
            frame.parent_id.zip(frame.parent_key.clone())
        } else {
            None
        };
        (sim.widgets.is_ancestor_visible(id), key)
    };
    if let Some((parent, key)) = old_parent_key {
        clear_child_from_rilua_parent_key(state, parent, &key, id)?;
    }
    let is_visible = {
        let mut sim = borrow_state_mut(state)?;
        apply_parent_change(&mut sim, id, Some(request.parent));
        sim.widgets.remove_all_anchor_dependents_for(id);
        let frame = sim
            .widgets
            .get_mut_for_anchor_edit(id)
            .expect("validated private frame");
        frame.clear_all_points();
        if let Some(binding) = &request.binding {
            frame.set_point(
                binding.point,
                Some(binding.relative_to as usize),
                binding.relative_point,
                binding.offset_x as f32,
                binding.offset_y as f32,
            );
            sim.widgets.add_anchor_dependent(binding.relative_to, id);
        }
        sim.widgets.mark_anchor_rect_dirty(id);
        sim.widgets.is_ancestor_visible(id)
    };
    if was_visible != is_visible {
        dispatch_parent_visibility_change(state, id, Some(request.parent), is_visible)?;
    }
    Ok(())
}
