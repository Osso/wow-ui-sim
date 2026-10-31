//! Group creation, timer updates, frame/font-string registration, and quest blob methods.

use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, create_table,
    extract_frame_id, frame_id_from_stack, get_or_create_frame_fields, table_get, table_get_static,
    table_set, table_set_static, val_to_string,
};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use crate::quest_poi_blobs;
use rilua::vm::closure::RustFn;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub fn register(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    register_methods(state, mt, GROUP_TIMER_METHODS)?;
    Ok(())
}

struct MethodBinding {
    name: &'static str,
    func: RustFn,
}

macro_rules! method {
    ($name:literal, $func:path) => {
        MethodBinding {
            name: $name,
            func: $func,
        }
    };
}

const GROUP_TIMER_METHODS: &[MethodBinding] = &[
    method!("GetOrCreateGroup", get_or_create_group),
    method!("ForceUpdateTimers", force_update_timers),
    method!("RegisterFontString", register_font_string),
    method!("RegisterFontStrings", register_font_strings),
    method!("RegisterBackgroundTexture", register_background_texture),
    method!("RegisterFrames", register_frames),
    method!("SetBorderAlpha", set_border_alpha),
    method!("SetBorderScalar", set_border_scalar),
    method!("SetBorderTexture", set_border_texture),
    method!("SetFillAlpha", set_fill_alpha),
    method!("SetOwningDialog", set_owning_dialog),
    method!("SetFillTexture", set_fill_texture),
    method!("DrawBlob", draw_blob),
    method!("SetToDefaults", set_to_defaults),
    method!("DrawNone", draw_none),
    method!("UpdateMouseOverTooltip", update_mouse_over_tooltip),
    method!("GetTooltipIndex", get_tooltip_index),
    method!("SetAlertContainer", set_alert_container),
    method!("SetDefaultText", set_default_text),
    method!("UpdateHeight", update_height),
    method!("SetSelectionTranslator", set_selection_translator),
    method!("SetItemButtonScale", set_item_button_scale),
    method!("UpdateItemContextMatching", update_item_context_matching),
    method!("RegisterForWidgetSet", register_for_widget_set),
    method!("UnregisterForWidgetSet", unregister_for_widget_set),
];

fn register_methods(
    state: &mut LuaState,
    mt: GcRef<Table>,
    methods: &[MethodBinding],
) -> LuaResult<()> {
    for method in methods {
        table_set_rust_fn_static(state, mt, method.name, method.func)?;
    }
    Ok(())
}

// ── GetOrCreateGroup ──────────────────────────────────────────────────────────

pub fn get_or_create_group(state: &mut LuaState) -> LuaResult<u32> {
    let self_table = stack_val(state, 1);
    let group_text = String::from_stack(state, 2)?;
    let order = read_order(state);
    let groups = get_or_create_groups_table(state, self_table);
    let result = find_or_insert_group(state, groups, &group_text, order);
    state.push(result);
    Ok(1)
}

fn read_order(state: &mut LuaState) -> f64 {
    match stack_val(state, 3) {
        Val::Num(v) => v,
        _ => 10.0,
    }
}

fn get_or_create_groups_table(state: &mut LuaState, self_table: Val) -> Val {
    match table_get(state, self_table, "groups") {
        t @ Val::Table(_) => t,
        _ => {
            let t = create_table(state);
            table_set(state, self_table, "groups", t);
            t
        }
    }
}

fn find_or_insert_group(state: &mut LuaState, groups: Val, group_text: &str, order: f64) -> Val {
    let Val::Table(groups_ref) = groups else {
        return Val::Nil;
    };
    let existing = state
        .gc
        .tables
        .get(groups_ref)
        .map(|t| t.array_slice().to_vec())
        .unwrap_or_default();

    if let Some(found) = find_existing_group(state, &existing, group_text) {
        return found;
    }
    insert_new_group(state, groups_ref, &existing, group_text, order)
}

fn find_existing_group(state: &mut LuaState, existing: &[Val], group_text: &str) -> Option<Val> {
    for &entry in existing {
        let name = table_get(state, entry, "groupText");
        if val_to_string(state, name).as_deref() == Some(group_text) {
            return Some(entry);
        }
    }
    None
}

fn insert_new_group(
    state: &mut LuaState,
    groups_ref: GcRef<Table>,
    existing: &[Val],
    group_text: &str,
    order: f64,
) -> Val {
    let group = create_table(state);
    let name_val = create_string(state, group_text);
    let categories = create_table(state);
    table_set(state, group, "groupText", name_val);
    table_set(state, group, "order", Val::Num(order));
    table_set(state, group, "categories", categories);
    if let Some(table) = state.gc.tables.get_mut(groups_ref) {
        let _ = table.raw_set(
            Val::Num((existing.len() + 1) as f64),
            group,
            &state.gc.string_arena,
        );
    }
    state.gc.barrier_back(groups_ref);
    group
}

// ── ForceUpdateTimers ─────────────────────────────────────────────────────────

pub fn force_update_timers(state: &mut LuaState) -> LuaResult<u32> {
    let self_table = stack_val(state, 1);
    let active_timers = table_get(state, self_table, "activeTimers");
    let Val::Table(active_timers_ref) = active_timers else {
        return Ok(0);
    };
    let timers = state
        .gc
        .tables
        .get(active_timers_ref)
        .map(|t| t.hash_entries())
        .unwrap_or_default();
    for (_, timer) in timers {
        let update_fn = table_get_static(state, timer, "OnUpdate");
        if matches!(update_fn, Val::Function(_)) {
            let _ = call_function_state(state, update_fn, &[timer]);
        }
    }
    Ok(0)
}

// ── Frame/FontString registration ─────────────────────────────────────────────

pub fn register_font_strings(state: &mut LuaState) -> LuaResult<u32> {
    if call_fields_override_if_present(state, "RegisterFontStrings")? {
        return Ok(0);
    }
    let fields = frame_fields_from_self(state)?;
    let font_strings = collect_varargs_table(state, 2);
    table_set(state, fields, "fontStrings", font_strings);
    table_set_static(state, fields, "__registeredFontStrings", font_strings);
    Ok(0)
}

pub fn register_font_string(state: &mut LuaState) -> LuaResult<u32> {
    if call_fields_override_if_present(state, "RegisterFontString")? {
        return Ok(0);
    }
    let fields = frame_fields_from_self(state)?;
    let font_strings = ensure_registry_table(state, fields, "fontStrings");
    let font_string = stack_val(state, 2);
    set_table_entry(state, font_strings, font_string, Val::Bool(true));
    table_set_static(state, fields, "__registeredFontStrings", font_strings);
    Ok(0)
}

pub fn register_background_texture(state: &mut LuaState) -> LuaResult<u32> {
    if call_fields_override_if_present(state, "RegisterBackgroundTexture")? {
        return Ok(0);
    }
    let fields = frame_fields_from_self(state)?;
    let background = stack_val(state, 2);
    let texture_kit = stack_val(state, 3);
    table_set(state, fields, "backgroundTexture", background);
    table_set(state, fields, "textureKit", texture_kit);
    Ok(0)
}

pub fn register_frames(state: &mut LuaState) -> LuaResult<u32> {
    if call_fields_override_if_present(state, "RegisterFrames")? {
        return Ok(0);
    }
    let fields = frame_fields_from_self(state)?;
    let frames = collect_varargs_table(state, 2);
    table_set(state, fields, "frames", frames);
    Ok(0)
}

pub fn set_owning_dialog(state: &mut LuaState) -> LuaResult<u32> {
    if call_fields_override_if_present(state, "SetOwningDialog")? {
        return Ok(0);
    }
    let fields = frame_fields_from_self(state)?;
    let dialog = stack_val(state, 2);
    table_set(state, fields, "owningDialog", dialog);
    table_set(state, fields, "OwningDialog", dialog);
    Ok(0)
}

pub fn set_alert_container(state: &mut LuaState) -> LuaResult<u32> {
    if call_fields_override_if_present(state, "SetAlertContainer")? {
        return Ok(0);
    }
    let fields = frame_fields_from_self(state)?;
    table_set(state, fields, "alertContainer", stack_val(state, 2));
    Ok(0)
}

pub fn set_default_text(state: &mut LuaState) -> LuaResult<u32> {
    if call_fields_override_if_present(state, "SetDefaultText")? {
        return Ok(0);
    }
    let fields = frame_fields_from_self(state)?;
    table_set(state, fields, "defaultText", stack_val(state, 2));
    Ok(0)
}

pub fn update_height(state: &mut LuaState) -> LuaResult<u32> {
    if call_fields_override_if_present(state, "UpdateHeight")? {
        return Ok(0);
    }
    Ok(0)
}

pub fn set_selection_translator(state: &mut LuaState) -> LuaResult<u32> {
    if call_fields_override_if_present(state, "SetSelectionTranslator")? {
        return Ok(0);
    }
    let fields = frame_fields_from_self(state)?;
    table_set(state, fields, "selectionTranslator", stack_val(state, 2));
    Ok(0)
}

pub fn set_item_button_scale(state: &mut LuaState) -> LuaResult<u32> {
    if call_fields_override_if_present(state, "SetItemButtonScale")? {
        return Ok(0);
    }
    let fields = frame_fields_from_self(state)?;
    table_set(state, fields, "itemButtonScale", stack_val(state, 2));
    Ok(0)
}

pub fn update_item_context_matching(state: &mut LuaState) -> LuaResult<u32> {
    if call_fields_override_if_present(state, "UpdateItemContextMatching")? {
        return Ok(0);
    }
    Ok(0)
}

pub fn register_for_widget_set(state: &mut LuaState) -> LuaResult<u32> {
    if call_fields_override_if_present(state, "RegisterForWidgetSet")? {
        return Ok(0);
    }
    let fields = frame_fields_from_self(state)?;
    let registration = create_table(state);
    table_set(state, registration, "widgetSetID", stack_val(state, 2));
    table_set(
        state,
        registration,
        "widgetLayoutFunction",
        stack_val(state, 3),
    );
    table_set(
        state,
        registration,
        "widgetInitFunction",
        stack_val(state, 4),
    );
    table_set(state, registration, "attachedUnitInfo", stack_val(state, 5));
    table_set(state, fields, "widgetSetRegistration", registration);
    Ok(0)
}

pub fn unregister_for_widget_set(state: &mut LuaState) -> LuaResult<u32> {
    if call_fields_override_if_present(state, "UnregisterForWidgetSet")? {
        return Ok(0);
    }
    let fields = frame_fields_from_self(state)?;
    table_set(state, fields, "widgetSetRegistration", Val::Nil);
    Ok(0)
}

fn frame_fields_from_self(state: &mut LuaState) -> LuaResult<Val> {
    let frame = stack_val(state, 1);
    let Some(id) = extract_frame_id(state, frame) else {
        return Ok(Val::Nil);
    };
    Ok(get_or_create_frame_fields(state, id))
}

fn call_fields_override_if_present(state: &mut LuaState, method_name: &str) -> LuaResult<bool> {
    let fields = frame_fields_from_self(state)?;
    let override_fn = table_get(state, fields, method_name);
    if !matches!(override_fn, Val::Function(_)) {
        return Ok(false);
    }
    let arg_count = state.top.saturating_sub(state.base) as i32;
    let args: Vec<Val> = (1..=arg_count)
        .map(|index| stack_val(state, index))
        .collect();
    let _ = call_function_state(state, override_fn, &args)?;
    Ok(true)
}

fn collect_varargs_table(state: &mut LuaState, start: i32) -> Val {
    let table = create_table(state);
    let mut out_index = 1i64;
    let mut input_index = start;
    loop {
        let value = stack_val(state, input_index);
        if value == Val::Nil {
            break;
        }
        set_array_entry(state, table, out_index, value);
        out_index += 1;
        input_index += 1;
    }
    table
}

fn ensure_registry_table(state: &mut LuaState, fields: Val, key: &str) -> Val {
    let existing = table_get(state, fields, key);
    if matches!(existing, Val::Table(_)) {
        return existing;
    }
    let table = create_table(state);
    table_set(state, fields, key, table);
    table
}

fn set_array_entry(state: &mut LuaState, table: Val, index: i64, value: Val) {
    set_table_entry(state, table, Val::Num(index as f64), value);
}

fn set_table_entry(state: &mut LuaState, table: Val, key: Val, value: Val) {
    let Val::Table(table_ref) = table else { return };
    if let Some(t) = state.gc.tables.get_mut(table_ref) {
        let _ = t.raw_set(key, value, &state.gc.string_arena);
    }
    state.gc.barrier_back(table_ref);
}

// ── Quest blob fields ─────────────────────────────────────────────────────────

pub fn set_fill_texture(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_texture(state, BlobTextureField::Fill)
}

pub fn set_fill_alpha(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_alpha(state, BlobAlphaField::Fill)
}

pub fn set_border_texture(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_texture(state, BlobTextureField::Border)
}

pub fn set_border_alpha(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_alpha(state, BlobAlphaField::Border)
}

enum BlobTextureField {
    Fill,
    Border,
}

enum BlobAlphaField {
    Fill,
    Border,
}

fn set_blob_texture(state: &mut LuaState, field: BlobTextureField) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let texture = val_to_string(state, stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    let blob = sim.quest_blobs.entry(id).or_default();
    match field {
        BlobTextureField::Fill => blob.fill_texture = texture,
        BlobTextureField::Border => blob.border_texture = texture,
    }
    Ok(0)
}

fn set_blob_alpha(state: &mut LuaState, field: BlobAlphaField) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let alpha = match stack_val(state, 2) {
        Val::Num(value) => Some(value),
        _ => None,
    };
    let mut sim = borrow_state_mut(state)?;
    let blob = sim.quest_blobs.entry(id).or_default();
    match field {
        BlobAlphaField::Fill => blob.fill_alpha = alpha,
        BlobAlphaField::Border => blob.border_alpha = alpha,
    }
    Ok(0)
}

pub fn set_border_scalar(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let scalar = match stack_val(state, 2) {
        Val::Num(value) => Some(value),
        _ => None,
    };
    borrow_state_mut(state)?
        .quest_blobs
        .entry(id)
        .or_default()
        .border_scalar = scalar;
    Ok(0)
}

pub fn set_to_defaults(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    if !super::super::methods_helpers::can_change_protected_state_for(state, id) {
        super::super::methods_helpers::emit_addon_action_blocked(state, id, "SetToDefaults");
        return Ok(0);
    }
    {
        let mut sim = borrow_state_mut(state)?;
        if let Some(frame) = sim.widgets.get_mut_visual(id) {
            frame.minimap_player_texture = None;
            frame.minimap_ping_position = None;
        }
    }
    let fields = get_or_create_frame_fields(state, id);
    table_set_static(state, fields, "__minimap_zoom", Val::Num(0.0));
    Ok(0)
}

pub fn draw_none(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    borrow_state_mut(state)?
        .quest_blobs
        .entry(id)
        .or_default()
        .clear_active_quests();
    Ok(0)
}

pub fn draw_blob(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let quest_id = i32::from_stack(state, 2)? as u32;
    let active = bool::from_stack(state, 3).unwrap_or(true);
    if !active {
        return Ok(0);
    }
    let mut sim = borrow_state_mut(state)?;
    let blob = sim.quest_blobs.entry(id).or_default();
    blob.insert_active_quest(quest_id);
    Ok(0)
}

pub fn update_mouse_over_tooltip(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let x = f64::from_stack(state, 2)? as f32;
    let y = f64::from_stack(state, 3)? as f32;
    let hit = {
        let sim = borrow_state(state)?;
        sim.quest_blobs.get(&id).and_then(|blob_state| {
            quest_poi_blobs::hit_test_blobs(&blob_state.active_quests, blob_state.map_id, x, y)
        })
    };
    let Some((quest_id, count)) = hit else {
        return Ok(0);
    };
    state.push(Val::Num(quest_id as f64));
    state.push(Val::Num(count as f64));
    Ok(2)
}

pub fn get_tooltip_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 2)?;
    state.push(Val::Num(index as f64));
    Ok(1)
}
