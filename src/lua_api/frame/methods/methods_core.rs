//! Core frame methods: GetName, SetSize, Show/Hide, strata/level, mouse, scale, rect.

use super::methods_helpers::{calculate_frame_height, calculate_frame_width};
use crate::lua_api::frame::handle::{frame_lud, get_sim_state, lud_to_id};
use crate::lua_api::layout::compute_frame_rect;
use crate::lua_api::SimState;
use mlua::{LightUserData, Lua, Value};

/// Read screen dimensions from SimState.
fn screen_dims(state: &SimState) -> (f32, f32) {
    (state.screen_width, state.screen_height)
}

/// Add core frame methods to the shared methods table.
pub fn add_core_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_identity_methods(lua, methods)?;
    add_size_methods(lua, methods)?;
    add_rect_methods(lua, methods)?;
    add_visibility_methods(lua, methods)?;
    add_strata_level_methods(lua, methods)?;
    add_mouse_input_methods(lua, methods)?;
    add_scale_methods(lua, methods)?;
    add_region_query_methods(lua, methods)?;
    Ok(())
}

/// Identity methods: GetName, GetObjectType
fn add_identity_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("GetName", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).and_then(|f| f.name.clone()))
    })?)?;

    methods.set("GetDebugName", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(id) {
            if let Some(ref name) = frame.name {
                return Ok(name.clone());
            }
            if let Some(pid) = frame.parent_id
                && let Some(parent) = state.widgets.get(pid) {
                    for (key, &cid) in &parent.children_keys {
                        if cid == id {
                            let parent_name = parent.name.as_deref().unwrap_or("?");
                            return Ok(format!("{}.{}", parent_name, key));
                        }
                    }
                }
            return Ok(format!("[{}]", frame.widget_type.as_str()));
        }
        Ok("[Unknown]".to_string())
    })?)?;

    methods.set("GetObjectType", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let obj_type = state
            .widgets
            .get(id)
            .map(|f| f.widget_type.as_str())
            .unwrap_or("Frame");
        Ok(obj_type.to_string())
    })?)?;

    methods.set("IsObjectType", lua.create_function(|lua, (ud, type_name): (LightUserData, String)| {
        use crate::widget::WidgetType;
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let wt = state
            .widgets
            .get(id)
            .map(|f| f.widget_type)
            .unwrap_or(WidgetType::Frame);
        Ok(widget_type_is_a(wt, &type_name))
    })?)?;

    Ok(())
}

/// Size methods: GetWidth, GetHeight, GetSize, SetWidth, SetHeight, SetSize
fn add_size_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_size_getters(lua, methods)?;
    add_size_setters(lua, methods)?;
    Ok(())
}

/// Size getter methods: GetWidth, GetHeight, GetSize
fn add_size_getters(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("GetWidth", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        state.resolve_rect_if_dirty(id);
        Ok(calculate_frame_width(&state.widgets, id))
    })?)?;

    methods.set("GetHeight", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        state.resolve_rect_if_dirty(id);
        Ok(calculate_frame_height(&state.widgets, id))
    })?)?;

    methods.set("GetSize", lua.create_function(|lua, (ud, _explicit): (LightUserData, Option<bool>)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        state.resolve_rect_if_dirty(id);
        let width = calculate_frame_width(&state.widgets, id);
        let height = calculate_frame_height(&state.widgets, id);
        Ok((width, height))
    })?)?;

    Ok(())
}

/// Size setter methods: SetSize, SetWidth, SetHeight
fn add_size_setters(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetSize", lua.create_function(|lua, (ud, width, height): (LightUserData, f32, f32)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.set_size(width, height);
        }
        state.widgets.mark_rect_dirty(id);
        state.invalidate_layout_with_dependents(id);
        Ok(())
    })?)?;

    methods.set("SetWidth", lua.create_function(|lua, (ud, width): (LightUserData, f32)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.width = width;
        }
        state.widgets.mark_rect_dirty(id);
        state.invalidate_layout_with_dependents(id);
        Ok(())
    })?)?;

    methods.set("SetHeight", lua.create_function(|lua, (ud, height): (LightUserData, f32)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.height = height;
        }
        state.widgets.mark_rect_dirty(id);
        state.invalidate_layout_with_dependents(id);
        Ok(())
    })?)?;

    Ok(())
}

/// Compute effective scale by walking up the parent chain.
fn effective_scale(widgets: &crate::widget::WidgetRegistry, id: u64) -> f32 {
    let mut scale = 1.0f32;
    let mut current_id = Some(id);
    while let Some(cid) = current_id {
        if let Some(f) = widgets.get(cid) {
            scale *= f.scale;
            current_id = f.parent_id;
        } else {
            break;
        }
    }
    scale
}

/// Rect/position methods
fn add_rect_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_rect_full_methods(lua, methods)?;
    add_rect_edge_methods(lua, methods)?;
    Ok(())
}

/// GetRect, GetScaledRect, GetBounds
fn add_rect_full_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("GetRect", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let (screen_width, screen_height) = screen_dims(&state);
        let rect = compute_frame_rect(&state.widgets, id, screen_width, screen_height);
        let bottom = screen_height - rect.y - rect.height;
        Ok((rect.x, bottom, rect.width, rect.height))
    })?)?;

    methods.set("GetScaledRect", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let (screen_width, screen_height) = screen_dims(&state);
        let rect = compute_frame_rect(&state.widgets, id, screen_width, screen_height);
        let scale = effective_scale(&state.widgets, id);
        let left = rect.x * scale;
        let bottom = (screen_height - rect.y - rect.height) * scale;
        Ok((left, bottom, rect.width * scale, rect.height * scale))
    })?)?;

    methods.set("GetBounds", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let (screen_width, screen_height) = screen_dims(&state);
        let rect = compute_frame_rect(&state.widgets, id, screen_width, screen_height);
        let bottom = screen_height - rect.y - rect.height;
        Ok((rect.x, bottom, rect.width, rect.height))
    })?)?;

    Ok(())
}

/// GetLeft, GetRight, GetTop, GetBottom, GetCenter
fn add_rect_edge_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("GetLeft", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let (sw, sh) = screen_dims(&state);
        let rect = compute_frame_rect(&state.widgets, id, sw, sh);
        Ok(rect.x)
    })?)?;

    methods.set("GetRight", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let (sw, sh) = screen_dims(&state);
        let rect = compute_frame_rect(&state.widgets, id, sw, sh);
        Ok(rect.x + rect.width)
    })?)?;

    methods.set("GetTop", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let (sw, sh) = screen_dims(&state);
        let rect = compute_frame_rect(&state.widgets, id, sw, sh);
        Ok(sh - rect.y)
    })?)?;

    methods.set("GetBottom", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let (sw, sh) = screen_dims(&state);
        let rect = compute_frame_rect(&state.widgets, id, sw, sh);
        Ok(sh - rect.y - rect.height)
    })?)?;

    methods.set("GetCenter", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let (sw, sh) = screen_dims(&state);
        let rect = compute_frame_rect(&state.widgets, id, sw, sh);
        let cx = rect.x + rect.width / 2.0;
        let cy = sh - rect.y - rect.height / 2.0;
        Ok((cx, cy))
    })?)?;

    Ok(())
}

/// Fire OnShow on a frame and recursively on its visible children.
pub(crate) fn fire_on_show_recursive(
    lua: &Lua,
    id: u64,
) -> mlua::Result<()> {
    if let Some(handler) = crate::lua_api::script_helpers::get_script(lua, id, "OnShow") {
        let frame_val = frame_lud(id);
        if let Err(e) = handler.call::<()>(frame_val) {
            crate::lua_api::script_helpers::call_error_handler(lua, &e.to_string());
        }
    }

    let children: Vec<u64> = {
        let state_rc = get_sim_state(lua);
        let st = state_rc.borrow();
        st.widgets
            .get(id)
            .map(|f| {
                f.children
                    .iter()
                    .filter(|&&cid| st.widgets.get(cid).map(|c| c.visible).unwrap_or(false))
                    .copied()
                    .collect()
            })
            .unwrap_or_default()
    };

    for child_id in children {
        fire_on_show_recursive(lua, child_id)?;
    }

    Ok(())
}

/// Visibility methods: Show, Hide, IsVisible, IsShown, SetShown
fn add_visibility_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("Show", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let was_hidden = {
            let state = state_rc.borrow();
            state.widgets.get(id).map(|f| !f.visible).unwrap_or(false)
        };
        state_rc.borrow_mut().set_frame_visible(id, true);
        if was_hidden {
            fire_on_show_recursive(lua, id)?;
        }
        Ok(())
    })?)?;

    methods.set("Hide", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        state_rc.borrow_mut().set_frame_visible(id, false);
        Ok(())
    })?)?;

    methods.set("IsVisible", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let mut cur = id;
        loop {
            match state.widgets.get(cur) {
                Some(f) if f.visible => match f.parent_id {
                    Some(pid) => cur = pid,
                    None => return Ok(true),
                },
                _ => return Ok(false),
            }
        }
    })?)?;

    methods.set("IsShown", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let visible = state.widgets.get(id).map(|f| f.visible).unwrap_or(false);
        Ok(visible)
    })?)?;

    methods.set("SetShown", lua.create_function(|lua, (ud, shown): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        state_rc.borrow_mut().set_frame_visible(id, shown);
        Ok(())
    })?)?;

    Ok(())
}

/// Strata and level methods
fn add_strata_level_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_alpha_methods(lua, methods)?;
    add_strata_methods(lua, methods)?;
    add_level_methods(lua, methods)?;

    methods.set("SetToplevel", lua.create_function(|lua, (ud, toplevel): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(f) = state.widgets.get_mut(id) { f.toplevel = toplevel; }
        Ok(())
    })?)?;

    methods.set("IsToplevel", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        Ok(state_rc.borrow().widgets.get(id).map(|f| f.toplevel).unwrap_or(false))
    })?)?;

    Ok(())
}

/// Alpha transparency methods.
fn add_alpha_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetAlpha", lua.create_function(|lua, (ud, alpha): (LightUserData, f32)| {
        let id = lud_to_id(ud);
        let clamped = alpha.clamp(0.0, 1.0);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        let changed = state.widgets.get(id)
            .map(|f| f.alpha != clamped)
            .unwrap_or(false);
        if changed {
            let parent_eff = state.widgets.get(id)
                .and_then(|f| f.parent_id)
                .and_then(|pid| state.widgets.get(pid))
                .map(|p| p.effective_alpha)
                .unwrap_or(1.0);
            if let Some(frame) = state.widgets.get_mut_visual(id) {
                frame.alpha = clamped;
            }
            state.widgets.propagate_effective_alpha(id, parent_eff);
        }
        Ok(())
    })?)?;

    methods.set("GetAlpha", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.alpha).unwrap_or(1.0))
    })?)?;

    methods.set("GetEffectiveAlpha", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.effective_alpha).unwrap_or(1.0))
    })?)?;

    methods.set("SetAlphaFromBoolean", lua.create_function(|lua, (ud, flag): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        let new_alpha = if flag { 1.0 } else { 0.0 };
        let parent_eff = state.widgets.get(id)
            .and_then(|f| f.parent_id)
            .and_then(|pid| state.widgets.get(pid))
            .map(|p| p.effective_alpha)
            .unwrap_or(1.0);
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.alpha = new_alpha;
        }
        state.widgets.propagate_effective_alpha(id, parent_eff);
        Ok(())
    })?)?;

    Ok(())
}

/// Frame strata methods (major draw order).
fn add_strata_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetFrameStrata", lua.create_function(|lua, (ud, strata): (LightUserData, String)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        let Some(s) = crate::widget::FrameStrata::from_str(&strata) else {
            return Ok(());
        };
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.frame_strata = s;
            frame.has_fixed_frame_strata = true;
        }
        let mut queue: Vec<u64> = state.widgets.get(id)
            .map(|f| f.children.clone()).unwrap_or_default();
        while let Some(child_id) = queue.pop() {
            let Some(child) = state.widgets.get_mut_visual(child_id) else { continue };
            if child.has_fixed_frame_strata { continue; }
            child.frame_strata = s;
            queue.extend(child.children.iter().copied());
        }
        state.strata_buckets = None;
        Ok(())
    })?)?;

    methods.set("GetFrameStrata", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let strata = state.widgets.get(id)
            .map(|f| f.frame_strata.as_str())
            .unwrap_or("MEDIUM");
        Ok(strata.to_string())
    })?)?;

    methods.set("SetFixedFrameStrata", lua.create_function(|lua, (ud, fixed): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.has_fixed_frame_strata = fixed;
        }
        Ok(())
    })?)?;

    methods.set("HasFixedFrameStrata", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.has_fixed_frame_strata).unwrap_or(false))
    })?)?;

    Ok(())
}

/// Frame level methods (draw order within strata).
fn add_level_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetFrameLevel", lua.create_function(|lua, (ud, level): (LightUserData, i32)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.frame_level = level;
            frame.has_fixed_frame_level = true;
        }
        super::methods_hierarchy::propagate_strata_level_pub(&mut state.widgets, id);
        Ok(())
    })?)?;

    methods.set("GetFrameLevel", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.frame_level).unwrap_or(0))
    })?)?;

    methods.set("SetFixedFrameLevel", lua.create_function(|lua, (ud, fixed): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.has_fixed_frame_level = fixed;
        }
        Ok(())
    })?)?;

    methods.set("HasFixedFrameLevel", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.has_fixed_frame_level).unwrap_or(false))
    })?)?;

    Ok(())
}

/// Mouse and input methods
fn add_mouse_input_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetID", lua.create_function(|lua, (ud, user_id): (LightUserData, i32)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        if let Some(f) = state_rc.borrow_mut().widgets.get_mut(id) {
            f.user_id = user_id;
        }
        Ok(())
    })?)?;

    methods.set("GetID", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        Ok(state_rc.borrow().widgets.get(id).map(|f| f.user_id).unwrap_or(0))
    })?)?;

    methods.set("GetMapID", lua.create_function(|_, _ud: LightUserData| Ok(0))?)?;
    methods.set("SetMapID", lua.create_function(|_, (_ud, _map_id): (LightUserData, i32)| Ok(()))?)?;

    methods.set("EnableMouse", lua.create_function(|lua, (ud, enable): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(id) { frame.mouse_enabled = enable; }
        Ok(())
    })?)?;

    methods.set("IsMouseEnabled", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.mouse_enabled).unwrap_or(false))
    })?)?;

    methods.set("EnableMouseWheel", lua.create_function(|_, (_ud, _enable): (LightUserData, bool)| Ok(()))?)?;
    methods.set("IsMouseWheelEnabled", lua.create_function(|_, _ud: LightUserData| Ok(false))?)?;

    methods.set("EnableKeyboard", lua.create_function(|lua, (ud, enable): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(f) = state.widgets.get_mut(id) { f.keyboard_enabled = enable; }
        Ok(())
    })?)?;

    methods.set("IsKeyboardEnabled", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.keyboard_enabled).unwrap_or(false))
    })?)?;

    methods.set("RegisterForMouse", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;

    add_mouse_motion_methods(lua, methods)?;
    Ok(())
}

/// Mouse motion and click enabled methods.
fn add_mouse_motion_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("EnableMouseMotion", lua.create_function(|lua, (ud, enable): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(id) { frame.mouse_motion_enabled = enable; }
        Ok(())
    })?)?;

    methods.set("IsMouseMotionEnabled", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.mouse_motion_enabled).unwrap_or(false))
    })?)?;

    methods.set("SetMouseMotionEnabled", lua.create_function(|lua, (ud, enable): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(id) { frame.mouse_motion_enabled = enable; }
        Ok(())
    })?)?;

    methods.set("SetMouseClickEnabled", lua.create_function(|lua, (ud, enable): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(id) { frame.mouse_enabled = enable; }
        Ok(())
    })?)?;

    methods.set("IsMouseClickEnabled", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.mouse_enabled).unwrap_or(false))
    })?)?;

    Ok(())
}

/// Scale methods
fn add_scale_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("GetScale", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.scale).unwrap_or(1.0))
    })?)?;

    methods.set("SetScale", lua.create_function(|lua, (ud, scale): (LightUserData, f32)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        let parent_eff_scale = state.widgets.get(id)
            .and_then(|f| f.parent_id)
            .and_then(|pid| state.widgets.get(pid))
            .map(|p| p.effective_scale)
            .unwrap_or(1.0);
        if let Some(f) = state.widgets.get_mut_visual(id) {
            f.scale = scale;
        }
        state.widgets.propagate_effective_scale(id, parent_eff_scale);
        state.invalidate_layout_with_dependents(id);
        Ok(())
    })?)?;

    methods.set("GetEffectiveScale", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.effective_scale).unwrap_or(1.0))
    })?)?;

    methods.set("SetIgnoreParentScale", lua.create_function(|_, (_ud, _ignore): (LightUserData, bool)| Ok(()))?)?;
    methods.set("GetIgnoreParentScale", lua.create_function(|_, _ud: LightUserData| Ok(false))?)?;
    methods.set("SetIgnoreParentAlpha", lua.create_function(|_, (_ud, _ignore): (LightUserData, bool)| Ok(()))?)?;
    methods.set("GetIgnoreParentAlpha", lua.create_function(|_, _ud: LightUserData| Ok(false))?)?;

    Ok(())
}

/// Region/frame query methods
fn add_region_query_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("IsRectValid", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let has_anchors = state_rc.borrow().widgets.get(id)
            .map(|f| !f.anchors.is_empty()).unwrap_or(false);
        if !has_anchors {
            return Ok(false);
        }
        state_rc.borrow_mut().resolve_rect_if_dirty(id);
        Ok(true)
    })?)?;

    methods.set("IsObjectLoaded", lua.create_function(|_, _ud: LightUserData| Ok(true))?)?;
    methods.set("IsMouseOver", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(true))?)?;

    methods.set("IsMouseMotionFocus", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.hovered_frame == Some(id))
    })?)?;

    methods.set("StopAnimating", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;
    methods.set("GetSourceLocation", lua.create_function(|_, _ud: LightUserData| Ok(Value::Nil))?)?;
    methods.set("Intersects", lua.create_function(|_, (_ud, _region): (LightUserData, Value)| Ok(false))?)?;
    methods.set("IsDrawLayerEnabled", lua.create_function(|_, (_ud, _layer): (LightUserData, String)| Ok(true))?)?;
    methods.set("SetDrawLayerEnabled", lua.create_function(|_, (_ud, _layer, _enabled): (LightUserData, String, bool)| Ok(()))?)?;

    Ok(())
}

/// Check if a widget type is or inherits from the given type name.
fn widget_type_is_a(wt: crate::widget::WidgetType, type_name: &str) -> bool {
    use crate::widget::WidgetType;
    if wt.as_str().eq_ignore_ascii_case(type_name) {
        return true;
    }
    match type_name.to_ascii_lowercase().as_str() {
        "region" => true,
        "frame" => !matches!(wt, WidgetType::FontString | WidgetType::Texture | WidgetType::Line),
        "texture" => matches!(wt, WidgetType::Texture | WidgetType::Line),
        "line" => matches!(wt, WidgetType::Line),
        "button" => matches!(wt, WidgetType::Button | WidgetType::CheckButton),
        _ => false,
    }
}
