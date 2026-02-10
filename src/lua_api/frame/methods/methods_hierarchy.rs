//! Hierarchy methods: GetParent, SetParent, GetNumChildren, GetChildren, GetRegions.

use super::FrameHandle;
use crate::widget::{FrameStrata, WidgetRegistry};
use mlua::{UserDataMethods, Value};
use std::rc::Rc;

/// Add hierarchy methods: parent access, children, regions.
pub fn add_hierarchy_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_parent_methods(methods);
    add_parent_key_methods(methods);
    add_children_frame_methods(methods);
    add_children_region_methods(methods);
}

/// GetParent, SetParent
fn add_parent_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("GetParent", |lua, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id)
            && let Some(parent_id) = frame.parent_id {
                let handle = FrameHandle {
                    id: parent_id,
                    state: Rc::clone(&this.state),
                };
                return Ok(Value::UserData(lua.create_userdata(handle)?));
            }
        Ok(Value::Nil)
    });

    methods.add_method("SetParent", |_, this, parent: Value| {
        let new_parent_id = match parent {
            Value::Nil => None,
            Value::UserData(ud) => ud.borrow::<FrameHandle>().ok().map(|h| h.id),
            _ => None,
        };
        let mut state = this.state.borrow_mut();
        reparent_widget(&mut state.widgets, this.id, new_parent_id);
        state.visible_on_update_cache = None;
        Ok(())
    });
}

/// Move a widget to a new parent, updating children lists and inheriting strata/level.
fn reparent_widget(widgets: &mut WidgetRegistry, child_id: u64, new_parent_id: Option<u64>) {
    // Remove from old parent's children list
    let old_parent_id = widgets.get(child_id).and_then(|f| f.parent_id);
    if let Some(old_pid) = old_parent_id
        && let Some(old_parent) = widgets.get_mut(old_pid) {
            old_parent.children.retain(|&id| id != child_id);
        }

    // Get parent's strata and level for inheritance
    let parent_props = new_parent_id.and_then(|pid| {
        widgets
            .get(pid)
            .map(|p| (p.frame_strata, p.frame_level))
    });

    if let Some(frame) = widgets.get_mut(child_id) {
        frame.parent_id = new_parent_id;
        // Inherit strata and level from parent (like wowless does)
        if let Some((parent_strata, parent_level)) = parent_props {
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

    // Add to new parent's children list
    if let Some(new_pid) = new_parent_id
        && let Some(new_parent) = widgets.get_mut(new_pid)
            && !new_parent.children.contains(&child_id) {
                new_parent.children.push(child_id);
            }
}

/// SetParentKey, GetParentKey
fn add_parent_key_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetParentKey", |_, this, key: String| {
        let mut state = this.state.borrow_mut();
        let parent_id = state.widgets.get(this.id).and_then(|f| f.parent_id);
        if let Some(pid) = parent_id
            && let Some(parent) = state.widgets.get_mut(pid) {
                parent.children_keys.insert(key, this.id);
            }
        Ok(())
    });

    methods.add_method("GetParentKey", |lua, this, ()| {
        let state = this.state.borrow();
        let parent_id = state.widgets.get(this.id).and_then(|f| f.parent_id);
        if let Some(pid) = parent_id
            && let Some(parent) = state.widgets.get(pid) {
                for (key, &cid) in &parent.children_keys {
                    if cid == this.id {
                        return Ok(Value::String(lua.create_string(key.as_bytes())?));
                    }
                }
            }
        Ok(Value::Nil)
    });
}

/// GetNumChildren, GetChildren
fn add_children_frame_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("GetNumChildren", |_, this, ()| {
        let state = this.state.borrow();
        let count = state
            .widgets
            .get(this.id)
            .map(|f| f.children.len())
            .unwrap_or(0);
        Ok(count as i32)
    });

    methods.add_method("GetChildren", |lua, this, ()| {
        let state = this.state.borrow();
        let mut result = mlua::MultiValue::new();
        if let Some(frame) = state.widgets.get(this.id) {
            let children = frame.children.clone();
            drop(state);

            for child_id in children {
                let handle = FrameHandle {
                    id: child_id,
                    state: Rc::clone(&this.state),
                };
                if let Ok(ud) = lua.create_userdata(handle) {
                    result.push_back(Value::UserData(ud));
                }
            }
        }
        Ok(result)
    });
}

/// GetNumRegions, GetRegions, GetAdditionalRegions
fn add_children_region_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("GetNumRegions", |_, this, ()| {
        use crate::widget::WidgetType;
        let state = this.state.borrow();
        let count = state.widgets.get(this.id).map(|f| {
            f.children.iter().filter(|&&cid| {
                state.widgets.get(cid).map(|c| {
                    matches!(c.widget_type, WidgetType::Texture | WidgetType::FontString)
                }).unwrap_or(false)
            }).count()
        }).unwrap_or(0);
        Ok(count as i32)
    });

    methods.add_method("GetRegions", |lua, this, ()| {
        use crate::widget::WidgetType;
        let state = this.state.borrow();
        let mut result = mlua::MultiValue::new();
        if let Some(frame) = state.widgets.get(this.id) {
            let children = frame.children.clone();
            drop(state);

            for child_id in children {
                let is_region = {
                    let state = this.state.borrow();
                    state.widgets.get(child_id).map(|f| {
                        matches!(f.widget_type, WidgetType::Texture | WidgetType::FontString)
                    }).unwrap_or(false)
                };
                if is_region {
                    let handle = FrameHandle {
                        id: child_id,
                        state: Rc::clone(&this.state),
                    };
                    if let Ok(ud) = lua.create_userdata(handle) {
                        result.push_back(Value::UserData(ud));
                    }
                }
            }
        }
        Ok(result)
    });

    methods.add_method("GetAdditionalRegions", |_, _this, ()| {
        Ok(mlua::MultiValue::new())
    });
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
        let Some(child) = widgets.get_mut(child_id) else { continue };
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
