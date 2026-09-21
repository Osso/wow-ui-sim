//! Visibility, collapse-layout, and menu-open methods.

use super::helpers::{arg_bool, frame_id};
use crate::lua_api::frame::methods::methods_helpers::{
    can_change_protected_state_for, emit_addon_action_blocked,
};
use crate::lua_api::frame::methods::secret_origin::{require_shown_readable, unwrap_input};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, frame_ref};
use crate::lua_api::script_helpers::call_error_handler_state;
use crate::lua_api::script_helpers::get_script as get_rilua_script;
use crate::lua_bridge::stack_val;
use crate::widget::WidgetType;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Maximum handler invocations per Show/Hide call (6 cycles × 2 handlers).
const SHOW_HIDE_HANDLER_LIMIT: usize = 12;

/// Maximum cross-frame Show/Hide dispatch depth.
const GLOBAL_SHOW_HIDE_DEPTH_LIMIT: u32 = 40;

pub fn show(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "Show");
        return Ok(0);
    }
    if let Some(frame) = borrow_state_mut(state)?.widgets.get_mut(id) {
        frame.secret_shown = false;
    }
    show_or_hide(state, id, true)?;
    Ok(0)
}

pub fn hide(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "Hide");
        return Ok(0);
    }
    if let Some(frame) = borrow_state_mut(state)?.widgets.get_mut(id) {
        frame.secret_shown = false;
    }
    show_or_hide(state, id, false)?;
    Ok(0)
}

pub fn set_shown(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let (value, secret) = unwrap_input(state, stack_val(state, 2))?;
    let shown = !matches!(value, Val::Nil | Val::Bool(false));
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "SetShown");
        return Ok(0);
    }
    if let Some(frame) = borrow_state_mut(state)?.widgets.get_mut(id) {
        frame.secret_shown = secret;
    }
    show_or_hide(state, id, shown)?;
    Ok(0)
}

fn show_or_hide(state: &mut LuaState, id: u64, shown: bool) -> LuaResult<()> {
    let (needs_change, in_handler, parent_id) = read_show_hide_state(state, id, shown)?;
    if !needs_change {
        return Ok(());
    }

    let parent_visible = parent_id
        .map(|parent_id| {
            borrow_state(state)
                .map(|sim| sim.widgets.is_ancestor_visible(parent_id))
                .unwrap_or(false)
        })
        .unwrap_or(true);

    {
        let mut sim = borrow_state_mut(state)?;
        sim.set_frame_visible(id, shown);
    }

    if in_handler || !parent_visible {
        return Ok(());
    }

    let depth = borrow_state(state)?.global_show_hide_depth;
    if depth >= GLOBAL_SHOW_HIDE_DEPTH_LIMIT {
        return Ok(());
    }

    {
        let mut sim = borrow_state_mut(state)?;
        sim.global_show_hide_depth = depth + 1;
    }

    let result = drain_visibility_handlers(state, id, shown);

    reset_visibility_dispatch_state(state, id, depth)?;

    result?;
    super::size::mark_nearest_layout_parent_dirty(state, id);
    Ok(())
}

fn reset_visibility_dispatch_state(
    state: &mut LuaState,
    id: u64,
    previous_depth: u32,
) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut(id) {
        frame.show_hide_depth = 0;
    }
    sim.global_show_hide_depth = previous_depth;
    Ok(())
}

fn read_show_hide_state(
    state: &mut LuaState,
    id: u64,
    shown: bool,
) -> LuaResult<(bool, bool, Option<u64>)> {
    let sim = borrow_state(state)?;
    let frame = sim.widgets.get(id);
    let needs_change = frame.map(|frame| frame.visible != shown).unwrap_or(false);
    let in_handler = frame
        .map(|frame| frame.show_hide_depth > 0)
        .unwrap_or(false);
    let parent_id = if needs_change {
        frame.and_then(|frame| frame.parent_id)
    } else {
        None
    };
    Ok((needs_change, in_handler, parent_id))
}

fn drain_visibility_handlers(state: &mut LuaState, id: u64, initial_target: bool) -> LuaResult<()> {
    {
        let mut sim = borrow_state_mut(state)?;
        if let Some(frame) = sim.widgets.get_mut(id) {
            frame.show_hide_depth = 1;
        }
    }

    let mut target = initial_target;
    for _ in 0..SHOW_HIDE_HANDLER_LIMIT {
        let handler_name = if target { "OnShow" } else { "OnHide" };
        fire_visibility_handler_recursive(state, id, handler_name)?;

        let visible_after = borrow_state(state)?
            .widgets
            .get(id)
            .map(|frame| frame.visible)
            .unwrap_or(false);

        {
            let mut sim = borrow_state_mut(state)?;
            sim.set_frame_visible(id, target);
        }

        if visible_after == target {
            break;
        }

        target = visible_after;

        {
            let mut sim = borrow_state_mut(state)?;
            sim.set_frame_visible(id, target);
        }
    }

    Ok(())
}

fn fire_visibility_handler_recursive(
    state: &mut LuaState,
    frame_id: u64,
    handler_name: &str,
) -> LuaResult<()> {
    let children = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(frame_id)
            .map(|frame| {
                frame
                    .children
                    .iter()
                    .filter(|&&child_id| {
                        sim.widgets
                            .get(child_id)
                            .map(|child| child.visible)
                            .unwrap_or(false)
                    })
                    .copied()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    for child_id in children {
        fire_visibility_handler_recursive(state, child_id, handler_name)?;
    }

    if let Some(handler) = get_rilua_script(state, frame_id, handler_name) {
        let Ok(frame) = frame_ref(state, frame_id) else {
            return Ok(());
        };
        if let Err(error_msg) =
            crate::lua_api::script_helpers::protected_lua_pcall_state(state, handler, &[frame])
        {
            call_error_handler_state(state, &error_msg);
        }
    }

    Ok(())
}

pub fn is_visible(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    require_shown_readable(state, id, true)?;
    let sim = borrow_state(state)?;
    let result = sim.widgets.is_ancestor_visible(id)
        && sim.widgets.get(id).is_some_and(|frame| {
            matches!(
                frame.widget_type,
                WidgetType::Texture | WidgetType::FontString | WidgetType::Line
            ) || frame.visible
        });
    drop(sim);
    state.push(Val::Bool(result));
    Ok(1)
}

pub fn is_shown(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    require_shown_readable(state, id, false)?;
    let sim = borrow_state(state)?;
    let result = sim.widgets.get(id).map(|f| f.visible).unwrap_or(false);
    drop(sim);
    state.push(Val::Bool(result));
    Ok(1)
}

pub fn set_collapses_layout(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let val = arg_bool(state, 2);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.collapses_layout = val;
    }
    Ok(0)
}

pub fn collapses_layout(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let sim = borrow_state(state)?;
    let result = sim
        .widgets
        .get(id)
        .map(|f| f.collapses_layout)
        .unwrap_or(false);
    drop(sim);
    state.push(Val::Bool(result));
    Ok(1)
}

pub fn is_collapsed(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let sim = borrow_state(state)?;
    let result = is_collapsed_impl(&sim, id);
    drop(sim);
    state.push(Val::Bool(result));
    Ok(1)
}

/// IsMenuOpen() — returns false (menus are never open in headless mode).
pub fn is_menu_open(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let fields = crate::lua_api::methods::get_or_create_frame_fields(state, id);
    let has_menu = crate::lua_api::methods::table_get(state, fields, "menu") != Val::Nil;
    state.push(Val::Bool(has_menu));
    Ok(1)
}

fn is_collapsed_impl(state: &crate::lua_api::SimState, id: u64) -> bool {
    let frame = match state.widgets.get(id) {
        Some(f) => f,
        None => return false,
    };
    if !frame.collapses_layout {
        return false;
    }
    let mut visible = frame.visible;
    let mut cur_parent = frame.parent_id;
    while visible {
        match cur_parent.and_then(|pid| state.widgets.get(pid)) {
            Some(p) if p.visible => cur_parent = p.parent_id,
            Some(_) => {
                visible = false;
            }
            None => break,
        }
    }
    !visible
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;
    use rilua::LuaApiMut;

    use super::read_show_hide_state;

    #[test]
    fn read_show_hide_state_skips_parent_lookup_for_noop_visibility_calls() {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        env.exec(
            r#"
            Parent = CreateFrame("Frame", "Parent", UIParent)
            Child = CreateFrame("Frame", "Child", Parent)
            Parent:Show()
            Child:Show()
            "#,
        )
        .expect("failed to create test frames");

        let child_id = env
            .state()
            .borrow()
            .widgets
            .get_id_by_name("Child")
            .expect("child should exist");

        let mut lua = env.rilua_mut();
        let (needs_change, in_handler, parent_id) =
            read_show_hide_state(lua.state_mut(), child_id, true)
                .expect("state query should succeed");

        assert!(
            !needs_change,
            "shown child should not need another Show/SetShown(true)"
        );
        assert!(
            !in_handler,
            "fresh child should not be in a show/hide handler"
        );
        assert!(
            parent_id.is_none(),
            "no-op visibility checks should not request parent visibility traversal"
        );
    }
}
