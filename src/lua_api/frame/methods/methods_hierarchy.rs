//! Hierarchy methods: GetParent, SetParent, GetNumChildren, GetChildren, GetRegions.

use crate::lua_api::frame::handle::{extract_frame_id, frame_lud, get_sim_state, lud_to_id};
use crate::widget::{FrameStrata, WidgetRegistry};
use mlua::{LightUserData, Lua, Value};

/// Add hierarchy methods: parent access, children, regions.
pub fn add_hierarchy_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_parent_methods(lua, methods)?;
    add_parent_key_methods(lua, methods)?;
    add_children_frame_methods(lua, methods)?;
    add_children_region_methods(lua, methods)?;
    Ok(())
}

/// GetParent, SetParent
fn add_parent_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("GetParent", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(id)
            && let Some(parent_id) = frame.parent_id
        {
            // Return the canonical LightUserData for the parent.
            return Ok(frame_lud(parent_id));
        }
        Ok(Value::Nil)
    })?)?;

    methods.set("SetParent", lua.create_function(|lua, (ud, parent): (LightUserData, Value)| {
        let id = lud_to_id(ud);
        let new_parent_id = extract_frame_id(&parent);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        reparent_widget(&mut state.widgets, id, new_parent_id);
        state.visible_on_update_cache = None;
        state.invalidate_layout(id);
        Ok(())
    })?)?;

    Ok(())
}

/// Move a widget to a new parent, updating children lists and inheriting strata/level.
fn reparent_widget(widgets: &mut WidgetRegistry, child_id: u64, new_parent_id: Option<u64>) {
    // Remove from old parent's children list
    let old_parent_id = widgets.get(child_id).and_then(|f| f.parent_id);
    if let Some(old_pid) = old_parent_id
        && let Some(old_parent) = widgets.get_mut_visual(old_pid) {
            old_parent.children.retain(|&id| id != child_id);
        }

    // Get parent's strata, level, effective_alpha, effective_scale for inheritance
    let parent_props = new_parent_id.and_then(|pid| {
        widgets
            .get(pid)
            .map(|p| (p.frame_strata, p.frame_level, p.effective_alpha, p.effective_scale))
    });

    if let Some(frame) = widgets.get_mut_visual(child_id) {
        frame.parent_id = new_parent_id;
        // Inherit strata and level from parent (like wowless does)
        if let Some((parent_strata, parent_level, _, _)) = parent_props {
            if !frame.has_fixed_frame_strata {
                frame.frame_strata = parent_strata;
            }
            if !frame.has_fixed_frame_level {
                frame.frame_level = parent_level + 1;
            }
        }
    }

    // Recursively propagate strata/level to all descendants (pool-acquired
    // frames keep stale levels from their original parent otherwise).
    propagate_strata_level(widgets, child_id);

    // Propagate effective_alpha and effective_scale from new parent.
    let parent_eff_alpha = parent_props.map(|(_, _, a, _)| a).unwrap_or(1.0);
    let parent_eff_scale = parent_props.map(|(_, _, _, s)| s).unwrap_or(1.0);
    widgets.propagate_effective_alpha(child_id, parent_eff_alpha);
    widgets.propagate_effective_scale(child_id, parent_eff_scale);

    // Add to new parent's children list
    if let Some(new_pid) = new_parent_id
        && let Some(new_parent) = widgets.get_mut_visual(new_pid)
            && !new_parent.children.contains(&child_id) {
                new_parent.children.push(child_id);
            }
}

/// SetParentKey, GetParentKey
fn add_parent_key_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetParentKey", lua.create_function(|lua, (ud, key): (LightUserData, String)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        let parent_id = state.widgets.get(id).and_then(|f| f.parent_id);
        if let Some(pid) = parent_id
            && let Some(parent) = state.widgets.get_mut_visual(pid) {
                parent.children_keys.insert(key, id);
            }
        Ok(())
    })?)?;

    methods.set("GetParentKey", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let parent_id = state.widgets.get(id).and_then(|f| f.parent_id);
        if let Some(pid) = parent_id
            && let Some(parent) = state.widgets.get(pid) {
                for (key, &cid) in &parent.children_keys {
                    if cid == id {
                        return Ok(Value::String(lua.create_string(key.as_bytes())?));
                    }
                }
            }
        Ok(Value::Nil)
    })?)?;

    Ok(())
}

/// GetNumChildren, GetChildren
fn add_children_frame_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("GetNumChildren", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let count = state
            .widgets
            .get(id)
            .map(|f| f.children.len())
            .unwrap_or(0);
        Ok(count as i32)
    })?)?;

    methods.set("GetChildren", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let mut result = mlua::MultiValue::new();
        if let Some(frame) = state.widgets.get(id) {
            let children = frame.children.clone();
            drop(state);
            for child_id in children {
                result.push_back(frame_lud(child_id));
            }
        }
        Ok(result)
    })?)?;

    Ok(())
}

/// GetNumRegions, GetRegions, GetAdditionalRegions
fn add_children_region_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("GetNumRegions", lua.create_function(|lua, ud: LightUserData| {
        use crate::widget::WidgetType;
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let count = state.widgets.get(id).map(|f| {
            f.children.iter().filter(|&&cid| {
                state.widgets.get(cid).map(|c| {
                    matches!(c.widget_type, WidgetType::Texture | WidgetType::FontString | WidgetType::Line)
                }).unwrap_or(false)
            }).count()
        }).unwrap_or(0);
        Ok(count as i32)
    })?)?;

    methods.set("GetRegions", lua.create_function(|lua, ud: LightUserData| {
        use crate::widget::WidgetType;
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let mut result = mlua::MultiValue::new();
        if let Some(frame) = state.widgets.get(id) {
            let children = frame.children.clone();
            drop(state);
            for child_id in children {
                let is_region = {
                    let state = state_rc.borrow();
                    state.widgets.get(child_id).map(|f| {
                        matches!(f.widget_type, WidgetType::Texture | WidgetType::FontString | WidgetType::Line)
                    }).unwrap_or(false)
                };
                if is_region {
                    result.push_back(frame_lud(child_id));
                }
            }
        }
        Ok(result)
    })?)?;

    methods.set("GetAdditionalRegions", lua.create_function(
        |_lua, _ud: LightUserData| Ok(mlua::MultiValue::new()),
    )?)?;

    Ok(())
}

/// Public wrapper for propagation, used by SetFrameLevel in methods_core.
pub fn propagate_strata_level_pub(widgets: &mut WidgetRegistry, root_id: u64) {
    propagate_strata_level(widgets, root_id);
}

/// BFS propagation of frame_strata and frame_level to all descendants.
/// Each child inherits parent_strata (unless has_fixed_frame_strata) and
/// parent_level + 1 (unless has_fixed_frame_level).
fn propagate_strata_level(widgets: &mut WidgetRegistry, root_id: u64) {
    let Some(root) = widgets.get(root_id) else { return };
    let root_strata = root.frame_strata;
    let root_level = root.frame_level;
    let mut queue: Vec<(u64, FrameStrata, i32)> = root
        .children
        .iter()
        .map(|&id| (id, root_strata, root_level))
        .collect();

    while let Some((child_id, parent_strata, parent_level)) = queue.pop() {
        let Some(child) = widgets.get_mut_visual(child_id) else { continue };
        if !child.has_fixed_frame_strata {
            child.frame_strata = parent_strata;
        }
        if !child.has_fixed_frame_level {
            child.frame_level = parent_level + 1;
        }
        let child_strata = child.frame_strata;
        let child_level = child.frame_level;
        let children = child.children.clone();
        for &grandchild_id in &children {
            queue.push((grandchild_id, child_strata, child_level));
        }
    }
}
