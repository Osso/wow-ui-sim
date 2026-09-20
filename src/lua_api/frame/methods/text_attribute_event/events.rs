//! Event registration, script handlers, and hlist data structure helpers.

mod mouse_implied;
mod script_binding_args;

use super::super::forbidden_aspects::{
    ensure_forbidden_aspect_absent, read_keyboard_input_propagation,
};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, frame_id_from_stack, val_to_string};
use crate::lua_api::script_helpers::{
    ScriptBinding, get_script_binding as get_rilua_script_binding,
    remove_script_binding as remove_rilua_script_binding, set_script as set_rilua_script,
    set_script_binding as set_rilua_script_binding,
};
use crate::lua_bridge::stack_val;
use crate::widget::WidgetType;
use mouse_implied::imply_mouse_enabled_for_mouse_handler;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};
use script_binding_args as binding_args;

pub(super) fn register_event(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    ensure_forbidden_aspect_absent(state, id, "EventRegistrations", "RegisterEvent")?;
    let Some(event) = val_to_string(state, stack_val(state, 2)) else {
        return Err(runtime_error("RegisterEvent: event name required"));
    };
    let newly_registered = insert_registered_event_checked(state, id, &event)?;
    if newly_registered {
        rilua_hlist_register_individual(state, id, &event)?;
    }
    let restricted = crate::event::is_restricted_event(&event);
    state.push(Val::Bool(newly_registered && !restricted));
    Ok(1)
}
fn insert_registered_event_checked(state: &mut LuaState, id: u64, event: &str) -> LuaResult<bool> {
    mutate_registered_event_checked(state, id, event, RegisteredEventOp::Insert)
}
pub(super) fn register_unit_event(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    ensure_forbidden_aspect_absent(state, id, "EventRegistrations", "RegisterUnitEvent")?;
    let Some(event) = val_to_string(state, stack_val(state, 2)) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    ensure_registerable_event(state, id, &event)?;
    let Some(unit) = val_to_string(state, stack_val(state, 3)) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let valid_unit = super::unit_event::is_valid_unit_filter(&unit);
    let newly_registered = {
        let mut sim = borrow_state_mut(state)?;
        if valid_unit {
            sim.widgets.register_unit_event_listener(id, &event, &unit)
        } else {
            sim.widgets.register_event_listener(id, &event)
        }
    };
    if newly_registered {
        rilua_hlist_register_individual(state, id, &event)?;
    }
    state.push(Val::Bool(newly_registered));
    Ok(1)
}
pub(super) fn unregister_event(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    ensure_forbidden_aspect_absent(state, id, "EventRegistrations", "UnregisterEvent")?;
    let Some(event) = val_to_string(state, stack_val(state, 2)) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let was_registered = remove_registered_event_checked(state, id, &event)?;
    if was_registered {
        rilua_hlist_unregister_individual(state, id, &event)?;
    }
    state.push(Val::Bool(was_registered));
    Ok(1)
}
fn remove_registered_event_checked(state: &mut LuaState, id: u64, event: &str) -> LuaResult<bool> {
    mutate_registered_event_checked(state, id, event, RegisteredEventOp::Remove)
}
enum RegisteredEventOp {
    Insert,
    Remove,
}

fn mutate_registered_event_checked(
    state: &mut LuaState,
    id: u64,
    event: &str,
    op: RegisteredEventOp,
) -> LuaResult<bool> {
    ensure_registerable_event(state, id, event)?;
    let mut sim = borrow_state_mut(state)?;
    Ok(match op {
        RegisteredEventOp::Insert => sim.widgets.register_event_listener(id, event),
        RegisteredEventOp::Remove => sim.widgets.unregister_event_listener(id, event),
    })
}
fn ensure_registerable_event(state: &mut LuaState, id: u64, event: &str) -> LuaResult<()> {
    if crate::event::is_registerable_event(event) {
        return Ok(());
    }

    let sim = borrow_state(state)?;
    #[cfg(feature = "retail-12-1-0")]
    if event == "BATTLETAG_INVITE_SHOW" {
        let loading_blizzard_addon = sim
            .loading_addon_index
            .and_then(|idx| sim.addons.get(idx as usize))
            .is_some_and(|addon| addon.folder_name.starts_with("Blizzard_"));
        if loading_blizzard_addon {
            return Ok(());
        }
    }
    let frame_name = sim
        .widgets
        .get(id)
        .and_then(|frame| frame.name.clone())
        .unwrap_or_else(|| "Frame".to_string());
    Err(runtime_error(format!(
        "{}:RegisterEvent(): {}:RegisterEvent(): Attempt to register unknown event \"{}\"",
        frame_name, frame_name, event
    )))
}
pub(super) fn unregister_all_events(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    ensure_forbidden_aspect_absent(state, id, "EventRegistrations", "UnregisterAllEvents")?;
    {
        let mut sim = borrow_state_mut(state)?;
        sim.widgets.unregister_all_event_listeners(id);
    }
    rilua_hlist_unregister_all(state, id)?;
    Ok(0)
}
pub(super) fn register_all_events(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    ensure_forbidden_aspect_absent(state, id, "EventRegistrations", "RegisterAllEvents")?;
    {
        let mut sim = borrow_state_mut(state)?;
        sim.widgets.register_all_event_listener(id);
    }
    rilua_hlist_register_all(state, id)?;
    Ok(0)
}
pub(super) fn is_event_registered(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let event = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    let sim = borrow_state(state)?;
    let frame = sim.widgets.get(id);
    let registered = frame
        .map(|frame| frame_has_registered_event(frame, &event))
        .unwrap_or(false);
    let unit = frame.and_then(|frame| frame.registered_unit_events.get(&event).cloned());
    drop(sim);
    state.push(Val::Bool(registered));
    super::unit_event::push_registered_unit(state, unit.as_deref());
    Ok(2)
}
fn frame_has_registered_event(frame: &crate::widget::Frame, event: &str) -> bool {
    frame.registered_events.contains(event)
}
pub(super) fn register_event_callback(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    ensure_forbidden_aspect_absent(state, id, "EventRegistrations", "RegisterEventCallback")?;
    let Some(event) = val_to_string(state, stack_val(state, 2)) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    if !crate::event::is_callback_event(&event) {
        return Err(runtime_error(format!(
            "Frame:RegisterEventCallback(): Attempt to register unknown event \"{}\"",
            event
        )));
    }
    let callback = stack_val(state, 3);
    if !matches!(callback, Val::Function(_)) {
        state.push(Val::Bool(false));
        return Ok(1);
    }
    borrow_state_mut(state)?
        .widgets
        .register_event_listener(id, &event);
    super::callbacks::register_frame_event_callback(state, id, &event, callback)?;
    let restricted = crate::event::is_restricted_event(&event);
    rilua_hlist_register_individual(state, id, &event)?;
    state.push(Val::Bool(!restricted));
    Ok(1)
}
pub(super) fn register_unit_event_callback(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    ensure_forbidden_aspect_absent(state, id, "EventRegistrations", "RegisterUnitEventCallback")?;
    let Some(event) = val_to_string(state, stack_val(state, 2)) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let callback = stack_val(state, 3);
    if !matches!(callback, Val::Function(_)) {
        state.push(Val::Bool(false));
        return Ok(1);
    }
    let unit_filter = val_to_string(state, stack_val(state, 4));
    borrow_state_mut(state)?
        .widgets
        .register_event_listener(id, &event);
    rilua_hlist_register_individual(state, id, &event)?;
    super::callbacks::register_unit_callback(state, id, &event, callback, unit_filter.as_deref())?;
    let restricted = crate::event::is_restricted_event(&event);
    state.push(Val::Bool(!restricted));
    Ok(1)
}

pub(super) fn set_propagate_keyboard_input(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    // TODO: combat lockdown check
    let propagate = matches!(stack_val(state, 2), Val::Bool(b) if b);
    if !propagate {
        ensure_forbidden_aspect_absent(
            state,
            id,
            "AlwaysPropagateInput",
            "SetPropagateKeyboardInput",
        )?;
    }
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut(id) {
        f.propagate_keyboard_input = propagate;
    }
    Ok(0)
}

pub(super) fn get_propagate_keyboard_input(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let propagate = read_keyboard_input_propagation(state, id)?;
    state.push(Val::Bool(propagate));
    Ok(1)
}

pub(super) fn set_script(state: &mut LuaState) -> LuaResult<u32> {
    let frame_id = frame_id_from_stack(state, 1)?;
    let handler_name = val_to_string(state, stack_val(state, 2))
        .ok_or_else(|| runtime_error("SetScript: handler name required"))?;
    let handler = stack_val(state, 3);
    ensure_script_supported(state, frame_id, &handler_name)?;
    if matches!(handler, Val::Nil) {
        remove_rilua_script_binding(state, frame_id, &handler_name, ScriptBinding::Normal);
    } else {
        if !matches!(handler, Val::Function(_)) {
            return Err(runtime_error(format!(
                "SetScript: handler for '{handler_name}' must be a function or nil"
            )));
        }
        set_rilua_script(state, frame_id, &handler_name, handler);
        imply_mouse_enabled_for_mouse_handler(state, frame_id, &handler_name)?;
    }
    Ok(0)
}

pub(super) fn get_script(state: &mut LuaState) -> LuaResult<u32> {
    let frame_id = frame_id_from_stack(state, 1)?;
    let handler_name = val_to_string(state, stack_val(state, 2))
        .ok_or_else(|| runtime_error("GetScript: handler name required"))?;
    let binding = binding_args::optional_script_binding_from_stack(state, 3)?;
    let handler =
        get_rilua_script_binding(state, frame_id, &handler_name, binding).unwrap_or(Val::Nil);
    state.push(handler);
    Ok(1)
}

pub(super) fn has_script(state: &mut LuaState) -> LuaResult<u32> {
    let frame_id = frame_id_from_stack(state, 1)?;
    let handler_name = val_to_string(state, stack_val(state, 2))
        .ok_or_else(|| runtime_error("HasScript: handler name required"))?;
    // HasScript reports whether the object *type* supports the handler, not
    // whether a handler is currently bound (use GetScript for that). This
    // matches the live client, including animations — e.g.
    // `Alpha:HasScript("OnFinished")` is always true regardless of binding.
    let has_script = script_supported(state, frame_id, &handler_name);
    state.push(Val::Bool(has_script));
    Ok(1)
}

pub(super) fn hook_script(state: &mut LuaState) -> LuaResult<u32> {
    let frame_id = frame_id_from_stack(state, 1)?;
    let handler_name = val_to_string(state, stack_val(state, 2))
        .ok_or_else(|| runtime_error("HookScript: handler name required"))?;
    ensure_script_supported(state, frame_id, &handler_name)?;
    let hook = stack_val(state, 3);
    let binding = binding_args::optional_script_binding_from_stack(state, 4)?;
    if !matches!(hook, Val::Function(_)) {
        return Err(runtime_error(format!(
            "HookScript: hook for '{handler_name}' must be a function"
        )));
    }
    if binding_args::reject_unsupported_hook_binding(state, binding) {
        return Ok(1);
    }
    let old = get_rilua_script_binding(state, frame_id, &handler_name, binding).unwrap_or(Val::Nil);
    let chained = binding_args::build_hooked_script(state, old, hook)?;
    set_rilua_script_binding(state, frame_id, &handler_name, binding, chained);
    imply_mouse_enabled_for_mouse_handler(state, frame_id, &handler_name)?;
    state.push(Val::Bool(true));
    Ok(1)
}

fn ensure_script_supported(state: &LuaState, frame_id: u64, handler_name: &str) -> LuaResult<()> {
    if script_supported(state, frame_id, handler_name) {
        return Ok(());
    }
    Err(runtime_error(format!(
        "invalid script handler '{handler_name}'"
    )))
}

fn script_supported(state: &LuaState, frame_id: u64, handler_name: &str) -> bool {
    let Ok(sim) = borrow_state(state) else {
        return false;
    };
    let Some(frame) = sim.widgets.get(frame_id) else {
        return false;
    };
    script_supported_for_widget(
        frame.widget_type,
        frame.object_type_name.as_deref(),
        handler_name,
    )
}

fn script_supported_for_widget(
    widget_type: WidgetType,
    object_type_name: Option<&str>,
    handler_name: &str,
) -> bool {
    // Animation objects support only their own handler set — they do NOT share
    // the frame-style common handlers (OnEvent, OnShow, OnEnter, ...). Real
    // client (12.0.5) rejects those on animations and animation groups.
    if let Some(name) = object_type_name {
        if is_animation_group_object_type(name) {
            return is_animation_group_script_handler(handler_name);
        }
        if is_animation_object_type(name) {
            return is_animation_script_handler(handler_name);
        }
    }
    is_widget_agnostic_script_handler(handler_name)
        || is_widget_specific_script_handler(widget_type, object_type_name, handler_name)
}

fn is_animation_group_object_type(object_type_name: &str) -> bool {
    object_type_name == "AnimationGroup"
}

fn is_animation_object_type(object_type_name: &str) -> bool {
    matches!(
        object_type_name,
        "Alpha"
            | "Translation"
            | "Scale"
            | "Rotation"
            | "LineTranslation"
            | "LineScale"
            | "Path"
            | "FlipBook"
            | "VertexColor"
            | "Animation"
    )
}

/// Handlers supported on every Animation subtype (Alpha, Translation, ...).
/// Matches real-client ground truth captured by `docs/addons/AnimScriptProbe`.
fn is_animation_script_handler(handler_name: &str) -> bool {
    matches!(
        handler_name,
        "OnLoad" | "OnUpdate" | "OnPlay" | "OnPause" | "OnStop" | "OnFinished"
    )
}

/// AnimationGroup supports the Animation handler set plus OnLoop.
fn is_animation_group_script_handler(handler_name: &str) -> bool {
    is_animation_script_handler(handler_name) || handler_name == "OnLoop"
}

fn is_widget_agnostic_script_handler(handler_name: &str) -> bool {
    is_common_script_handler(handler_name)
        || is_keyboard_script_handler(handler_name)
        || is_hyperlink_script_handler(handler_name)
}

fn is_widget_specific_script_handler(
    widget_type: WidgetType,
    object_type_name: Option<&str>,
    handler_name: &str,
) -> bool {
    match handler_name {
        "OnClick" | "PreClick" | "PostClick" => {
            matches!(widget_type, WidgetType::Button | WidgetType::CheckButton)
        }
        "OnDoubleClick" => true,
        "OnEnable" | "OnDisable" => supports_enable_disable_script(widget_type),
        "OnTextChanged"
        | "OnTextSet"
        | "OnEditFocusGained"
        | "OnEditFocusLost"
        | "OnInputLanguageChanged"
        | "OnCursorChanged" => widget_type == WidgetType::EditBox,
        "OnCooldownDone" => matches!(widget_type, WidgetType::Cooldown),
        "OnValueChanged" => matches!(widget_type, WidgetType::Slider | WidgetType::StatusBar),
        "OnVerticalScroll" | "OnHorizontalScroll" | "OnScrollRangeChanged" => {
            matches!(widget_type, WidgetType::ScrollFrame | WidgetType::EditBox)
        }
        "OnColorSelect" => matches!(widget_type, WidgetType::ColorSelect),
        "OnTooltipCleared"
        | "OnTooltipAddMoney"
        | "OnTooltipSetDefaultAnchor"
        | "OnTooltipSetFrameStack"
        | "OnTooltipSetQuest"
        | "OnTooltipSetSpell"
        | "OnTooltipSetItem"
        | "OnTooltipSetUnit"
        | "OnTooltipSetFramestack" => widget_type == WidgetType::GameTooltip,
        "OnModelLoaded" | "OnModelCleared" | "OnDressModel" => {
            supports_model_script_handler(widget_type, object_type_name)
        }
        _ => false,
    }
}

fn supports_model_script_handler(widget_type: WidgetType, object_type_name: Option<&str>) -> bool {
    matches!(
        widget_type,
        WidgetType::Model | WidgetType::ModelScene | WidgetType::PlayerModel
    ) || object_type_name == Some("ModelSceneActor")
}

fn supports_enable_disable_script(widget_type: WidgetType) -> bool {
    matches!(
        widget_type,
        WidgetType::Frame
            | WidgetType::Button
            | WidgetType::CheckButton
            | WidgetType::EditBox
            | WidgetType::Slider
            | WidgetType::ScrollFrame
    )
}

fn is_common_script_handler(handler_name: &str) -> bool {
    // NOTE: the animation handlers (OnPlay/OnPause/OnStop/OnFinished/OnLoop) are
    // intentionally NOT here. They are gated per animation type in
    // `script_supported_for_widget`; the live client rejects them on
    // non-animation widgets such as Frame.
    matches!(
        handler_name,
        "OnLoad"
            | "OnEvent"
            | "OnUpdate"
            | "OnShow"
            | "OnHide"
            | "OnEnter"
            | "OnLeave"
            | "OnMouseDown"
            | "OnMouseUp"
            | "OnMouseWheel"
            | "OnDragStart"
            | "OnDragStop"
            | "OnReceiveDrag"
            | "OnSizeChanged"
            | "OnAttributeChanged"
    )
}

fn is_keyboard_script_handler(handler_name: &str) -> bool {
    matches!(
        handler_name,
        "OnEnterPressed"
            | "OnEscapePressed"
            | "OnTabPressed"
            | "OnSpacePressed"
            | "OnChar"
            | "OnKeyDown"
            | "OnKeyUp"
            | "OnGamePadStick"
            | "OnGamepadStick"
    )
}

fn is_hyperlink_script_handler(handler_name: &str) -> bool {
    matches!(
        handler_name,
        "OnHyperlinkClick" | "OnHyperlinkEnter" | "OnHyperlinkLeave"
    )
}

/// Insert `id` into the per-event hlist stored in registry["__event_individual"][event].
pub(super) fn rilua_hlist_register_individual(
    state: &mut LuaState,
    id: u64,
    event: &str,
) -> LuaResult<()> {
    let individual_val = crate::lua_api::methods::registry_get(state, "__event_individual");
    let Val::Table(individual) = individual_val else {
        return Ok(());
    };
    let event_key = state.gc.intern_string(event.as_bytes());
    let existing = state
        .gc
        .tables
        .get(individual)
        .map(|t| t.get_str(event_key, &state.gc.string_arena));
    let event_tbl = match existing {
        Some(Val::Table(t)) => t,
        _ => {
            let new_tbl = state.gc.alloc_table(Table::new());
            if let Some(t) = state.gc.tables.get_mut(individual) {
                let _ = t.raw_set(
                    Val::Str(event_key),
                    Val::Table(new_tbl),
                    &state.gc.string_arena,
                );
            }
            state.gc.barrier_back(individual);
            new_tbl
        }
    };
    rilua_hlist_insert(state, event_tbl, id)
}

/// Remove `id` from the per-event hlist.
pub(super) fn rilua_hlist_unregister_individual(
    state: &mut LuaState,
    id: u64,
    event: &str,
) -> LuaResult<()> {
    let individual_val = crate::lua_api::methods::registry_get(state, "__event_individual");
    let Val::Table(individual) = individual_val else {
        return Ok(());
    };
    let event_key = state.gc.intern_string(event.as_bytes());
    let existing = state
        .gc
        .tables
        .get(individual)
        .map(|t| t.get_str(event_key, &state.gc.string_arena));
    if let Some(Val::Table(event_tbl)) = existing {
        rilua_hlist_remove(state, event_tbl, id)?;
    }
    Ok(())
}

/// Insert `id` into the all-events hlist stored in registry["__event_all"].
pub(super) fn rilua_hlist_register_all(state: &mut LuaState, id: u64) -> LuaResult<()> {
    let all_val = crate::lua_api::methods::registry_get(state, "__event_all");
    if let Val::Table(all_tbl) = all_val {
        rilua_hlist_insert(state, all_tbl, id)?;
    }
    Ok(())
}

/// Remove `id` from all individual event hlists and the all-events hlist.
pub(super) fn rilua_hlist_unregister_all(state: &mut LuaState, id: u64) -> LuaResult<()> {
    remove_from_individual_hlists(state, id)?;
    remove_from_all_hlist(state, id)
}

fn remove_from_individual_hlists(state: &mut LuaState, id: u64) -> LuaResult<()> {
    let individual_val = crate::lua_api::methods::registry_get(state, "__event_individual");
    let Val::Table(individual) = individual_val else {
        return Ok(());
    };
    let sub_tables: Vec<GcRef<Table>> = state
        .gc
        .tables
        .get(individual)
        .map(|t| {
            t.hash_entries()
                .into_iter()
                .filter_map(|(_, v)| if let Val::Table(t) = v { Some(t) } else { None })
                .collect()
        })
        .unwrap_or_default();
    for event_tbl in sub_tables {
        rilua_hlist_remove(state, event_tbl, id)?;
    }
    Ok(())
}

fn remove_from_all_hlist(state: &mut LuaState, id: u64) -> LuaResult<()> {
    let all_val = crate::lua_api::methods::registry_get(state, "__event_all");
    if let Val::Table(all_tbl) = all_val {
        rilua_hlist_remove(state, all_tbl, id)?;
    }
    Ok(())
}

/// hlist insert: append id to array, record index in "_s" sub-table.
pub(super) fn rilua_hlist_insert(
    state: &mut LuaState,
    tbl: GcRef<Table>,
    id: u64,
) -> LuaResult<()> {
    let set = rilua_hlist_set(state, tbl);
    let already = state
        .gc
        .tables
        .get(set)
        .map(|t| t.get_int(id as i64) != Val::Nil)
        .unwrap_or(false);
    if already {
        return Ok(());
    }
    let n = state.gc.tables.get(tbl).map(|t| t.array_len()).unwrap_or(0) + 1;
    if let Some(t) = state.gc.tables.get_mut(tbl) {
        let _ = t.raw_set(
            Val::Num(n as f64),
            Val::Num(id as f64),
            &state.gc.string_arena,
        );
    }
    state.gc.barrier_back(tbl);
    if let Some(s) = state.gc.tables.get_mut(set) {
        let _ = s.raw_set(
            Val::Num(id as f64),
            Val::Num(n as f64),
            &state.gc.string_arena,
        );
    }
    state.gc.barrier_back(set);
    Ok(())
}

/// hlist remove: swap-remove to keep array dense.
pub(super) fn rilua_hlist_remove(
    state: &mut LuaState,
    tbl: GcRef<Table>,
    id: u64,
) -> LuaResult<()> {
    let set = rilua_hlist_set(state, tbl);
    let idx = find_hlist_index(state, set, id);
    let Some(idx) = idx else {
        return Ok(());
    };
    let n = state.gc.tables.get(tbl).map(|t| t.array_len()).unwrap_or(0);
    if idx != n {
        swap_last_to_slot(state, tbl, set, idx, n);
    }
    clear_hlist_tail(state, tbl, set, n, id);
    Ok(())
}

fn find_hlist_index(state: &LuaState, set: GcRef<Table>, id: u64) -> Option<usize> {
    state
        .gc
        .tables
        .get(set)
        .and_then(|t| match t.get_int(id as i64) {
            Val::Num(n) if n > 0.0 => Some(n as usize),
            _ => None,
        })
}

fn swap_last_to_slot(
    state: &mut LuaState,
    tbl: GcRef<Table>,
    set: GcRef<Table>,
    idx: usize,
    n: usize,
) {
    let last_id = state
        .gc
        .tables
        .get(tbl)
        .and_then(|t| match t.get_int(n as i64) {
            Val::Num(lid) => Some(lid as u64),
            _ => None,
        });
    let Some(lid) = last_id else {
        return;
    };
    if let Some(t) = state.gc.tables.get_mut(tbl) {
        let _ = t.raw_set(
            Val::Num(idx as f64),
            Val::Num(lid as f64),
            &state.gc.string_arena,
        );
    }
    state.gc.barrier_back(tbl);
    if let Some(s) = state.gc.tables.get_mut(set) {
        let _ = s.raw_set(
            Val::Num(lid as f64),
            Val::Num(idx as f64),
            &state.gc.string_arena,
        );
    }
    state.gc.barrier_back(set);
}

fn clear_hlist_tail(state: &mut LuaState, tbl: GcRef<Table>, set: GcRef<Table>, n: usize, id: u64) {
    if let Some(t) = state.gc.tables.get_mut(tbl) {
        let _ = t.raw_set(Val::Num(n as f64), Val::Nil, &state.gc.string_arena);
    }
    state.gc.barrier_back(tbl);
    if let Some(s) = state.gc.tables.get_mut(set) {
        let _ = s.raw_set(Val::Num(id as f64), Val::Nil, &state.gc.string_arena);
    }
    state.gc.barrier_back(set);
}

/// Get or create the "_s" set sub-table of a hlist table.
pub(super) fn rilua_hlist_set(state: &mut LuaState, tbl: GcRef<Table>) -> GcRef<Table> {
    let key_ref = state.gc.intern_string_static(b"_s");
    let existing = state
        .gc
        .tables
        .get(tbl)
        .map(|t| t.get_str(key_ref, &state.gc.string_arena));
    if let Some(Val::Table(s)) = existing {
        return s;
    }
    let new_set = state.gc.alloc_table(Table::new());
    if let Some(t) = state.gc.tables.get_mut(tbl) {
        let _ = t.raw_set(
            Val::Str(key_ref),
            Val::Table(new_set),
            &state.gc.string_arena,
        );
    }
    state.gc.barrier_back(tbl);
    new_set
}

#[cfg(test)]
mod tests {
    use super::{frame_has_registered_event, script_supported_for_widget};
    use crate::widget::{Frame, WidgetType};

    #[test]
    fn frame_has_registered_event_uses_set_membership() {
        let mut frame = Frame::new(WidgetType::Frame, Some("EventFrame".to_string()), None);
        frame.registered_events.insert("PLAYER_LOGIN".to_string());

        assert!(frame_has_registered_event(&frame, "PLAYER_LOGIN"));
        assert!(!frame_has_registered_event(&frame, "PLAYER_LOGOUT"));
    }

    #[test]
    fn script_supported_for_widget_combines_shared_and_widget_specific_handlers() {
        assert!(script_supported_for_widget(
            WidgetType::Frame,
            None,
            "OnShow"
        ));
        assert!(script_supported_for_widget(
            WidgetType::EditBox,
            None,
            "OnTextChanged"
        ));
        assert!(!script_supported_for_widget(
            WidgetType::Frame,
            None,
            "OnTextChanged"
        ));
        assert!(script_supported_for_widget(
            WidgetType::PlayerModel,
            None,
            "OnModelLoaded"
        ));
        assert!(script_supported_for_widget(
            WidgetType::Frame,
            Some("ModelSceneActor"),
            "OnModelLoaded"
        ));
        assert!(!script_supported_for_widget(
            WidgetType::Frame,
            None,
            "OnModelLoaded"
        ));
    }
}
