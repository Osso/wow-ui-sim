//! rilua RustFn equivalents for methods_text/, methods_attribute.rs, and methods_event.rs.
//!
//! Each function is a `RustFn` (`fn(&mut LuaState) -> LuaResult<u32>`) that mirrors
//! the corresponding mlua method. Complex operations are stubbed with TODO.

#[cfg(feature = "retail-12-1-0")]
mod access_restrictions;
mod attributes;
pub(crate) mod callbacks;
mod events;
mod helpers;
mod rolesets;
mod text;
mod unit_event;

use crate::lua_api::methods::call_function_state;
#[cfg(feature = "retail-12-1-0")]
use crate::lua_api::methods::{
    create_table, frame_id_from_stack, get_or_create_frame_fields, table_get, table_set,
    val_to_string,
};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::api::LuaApiMut;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

/// Register all text, attribute, and event RustFn methods on the given table.
pub fn register_all(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    register_text_methods(state, table)?;
    register_attribute_methods(state, table)?;
    register_event_methods(state, table)?;
    if cfg!(any(
        feature = "retail-12-1-0",
        feature = "client-wowforever"
    )) {
        rolesets::register(state, table)?;
    }
    #[cfg(feature = "retail-12-1-0")]
    register_patch_12_1_methods(state, table)?;
    Ok(())
}

#[cfg(feature = "retail-12-1-0")]
fn register_patch_12_1_methods(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    access_restrictions::register(state, table)?;
    table_set_rust_fn_static(state, table, "AddForbiddenAspects", add_forbidden_aspects)?;
    table_set_rust_fn_static(state, table, "GetForbiddenAspects", get_forbidden_aspects)?;
    table_set_rust_fn_static(
        state,
        table,
        "GetInheritableForbiddenAspects",
        get_inheritable_forbidden_aspects,
    )?;
    table_set_rust_fn_static(state, table, "GetObjectTable", get_object_table)?;
    table_set_rust_fn_static(
        state,
        table,
        "HasAnyForbiddenAspects",
        has_any_forbidden_aspects,
    )?;
    table_set_rust_fn_static(state, table, "SetOnUpdateMode", set_on_update_mode)?;
    table_set_rust_fn_static(state, table, "GetOnUpdateMode", get_on_update_mode)?;
    table_set_rust_fn_static(state, table, "ClearScripts", clear_scripts)?;
    Ok(())
}

#[cfg(feature = "retail-12-1-0")]
fn frame_fields_from_stack(state: &mut LuaState) -> LuaResult<Val> {
    let id = frame_id_from_stack(state, 1)?;
    Ok(get_or_create_frame_fields(state, id))
}

#[cfg(feature = "retail-12-1-0")]
fn stack_bitmask_arg(state: &mut LuaState, index: usize) -> u64 {
    match crate::lua_bridge::stack_val(state, index as i32) {
        Val::Num(value) if value > 0.0 => value as u64,
        _ => 0,
    }
}

#[cfg(feature = "retail-12-1-0")]
fn add_forbidden_aspects(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut mask = 0;
    let count = state.top.saturating_sub(state.base);
    for index in 2..=count {
        mask |= stack_bitmask_arg(state, index);
    }
    crate::lua_api::frame::methods::forbidden_aspects::add_forbidden_aspects(state, id, mask)?;
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn get_forbidden_aspects(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mask =
        crate::lua_api::frame::methods::forbidden_aspects::stored_forbidden_aspects(state, id)?;
    state.push(Val::Num(mask as f64));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_inheritable_forbidden_aspects(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    // ScriptObjectPropagationPath is an enum (0/1), not our internal bitmask (1/2).
    let inheritance = match crate::lua_bridge::stack_val(state, 2) {
        Val::Num(0.0) => crate::lua_api::frame::methods::forbidden_aspects::INHERITANCE_PARENT,
        Val::Num(1.0) => crate::lua_api::frame::methods::forbidden_aspects::INHERITANCE_LAYOUT,
        _ => return Err(rilua::runtime_error("invalid ScriptObjectPropagationPath")),
    };
    let mask =
        crate::lua_api::frame::methods::forbidden_aspects::stored_inheritable_forbidden_aspects(
            state,
            id,
            inheritance,
        )?;
    state.push(Val::Num(mask as f64));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_object_table(state: &mut LuaState) -> LuaResult<u32> {
    let fields = frame_fields_from_stack(state)?;
    state.push(fields);
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn has_any_forbidden_aspects(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let aspects =
        crate::lua_api::frame::methods::forbidden_aspects::stored_forbidden_aspects(state, id)?;
    state.push(Val::Bool(aspects != 0));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn set_on_update_mode(state: &mut LuaState) -> LuaResult<u32> {
    let fields = frame_fields_from_stack(state)?;
    let value = crate::lua_bridge::stack_val(state, 2);
    let mode = val_to_string(state, value)
        .map(|mode| crate::lua_api::methods::create_string(state, &mode))
        .unwrap_or(value);
    table_set(state, fields, "__onUpdateMode", mode);
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn get_on_update_mode(state: &mut LuaState) -> LuaResult<u32> {
    let fields = frame_fields_from_stack(state)?;
    let mode = table_get(state, fields, "__onUpdateMode");
    if matches!(mode, Val::Nil) {
        let default_mode = crate::lua_api::methods::create_string(state, "RunWhenVisible");
        state.push(default_mode);
    } else {
        state.push(mode);
    }
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn clear_scripts(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    crate::lua_api::script_helpers::remove_all_scripts(state, id);
    Ok(0)
}

pub(crate) fn refresh_auto_text_height_after_width_change(state: &mut LuaState, id: u64) {
    text::refresh_auto_text_height_after_width_change(state, id);
}

pub(crate) fn refresh_auto_text_width_after_zero_width(state: &mut LuaState, id: u64) {
    text::update_auto_text_width(state, id);
}

pub(crate) fn get_text_num_lines(state: &mut LuaState) -> LuaResult<u32> {
    text::get_num_lines(state)
}

fn register_text_methods(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    register_plain_text(state, table)?;
    register_font_methods(state, table)?;
    register_text_layout(state, table)?;
    register_styled_text(state, table)
}

fn register_plain_text(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "SetText", text::set_text)?;
    table_set_rust_fn_static(state, table, "GetText", text::get_text)?;
    table_set_rust_fn_static(state, table, "ClearText", text::clear_text)?;
    table_set_rust_fn_static(state, table, "SetFormattedText", text::set_formatted_text)?;
    table_set_rust_fn_static(state, table, "ApplyDefaultText", text::apply_default_text)?;
    table_set_rust_fn_static(
        state,
        table,
        "TryApplyDefaultText",
        text::try_apply_default_text,
    )?;
    Ok(())
}

fn register_font_methods(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "SetFont", text::set_font)?;
    table_set_rust_fn_static(state, table, "GetFont", text::get_font)?;
    table_set_rust_fn_static(state, table, "SetFontObject", text::set_font_object)?;
    table_set_rust_fn_static(
        state,
        table,
        "SetFontObjectsToTry",
        text::set_font_objects_to_try,
    )?;
    table_set_rust_fn_static(state, table, "GetFontObject", text::get_font_object)?;
    table_set_rust_fn_static(state, table, "SetFontHeight", text::set_font_height)?;
    table_set_rust_fn_static(state, table, "SetTextHeight", text::set_text_height)?;
    table_set_rust_fn_static(state, table, "GetFontHeight", text::get_font_height)?;
    table_set_rust_fn_static(state, table, "SetShadowColor", text::set_shadow_color)?;
    table_set_rust_fn_static(state, table, "GetShadowColor", text::get_shadow_color)?;
    table_set_rust_fn_static(state, table, "SetShadowOffset", text::set_shadow_offset)?;
    table_set_rust_fn_static(state, table, "GetShadowOffset", text::get_shadow_offset)?;
    Ok(())
}

fn register_text_layout(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    register_text_measurement(state, table)?;
    register_text_alignment(state, table)?;
    register_text_wrapping(state, table)?;
    register_text_scaling(state, table)
}

fn register_text_measurement(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "GetStringWidth", text::get_string_width)?;
    table_set_rust_fn_static(state, table, "GetStringHeight", text::get_string_height)?;
    table_set_rust_fn_static(state, table, "GetWrappedWidth", text::get_wrapped_width)?;
    table_set_rust_fn_static(state, table, "GetTextWidth", text::get_text_width)?;
    table_set_rust_fn_static(state, table, "GetTextHeight", text::get_text_height)?;
    table_set_rust_fn_static(state, table, "GetContentHeight", text::get_content_height)?;
    table_set_rust_fn_static(state, table, "GetTextData", text::get_text_data)?;
    table_set_rust_fn_static(state, table, "GetLineHeight", text::get_line_height)?;
    table_set_rust_fn_static(state, table, "GetNumLines", text::get_num_lines)?;
    table_set_rust_fn_static(state, table, "IsTruncated", text::is_truncated)?;
    table_set_rust_fn_static(
        state,
        table,
        "GetUnboundedStringWidth",
        text::get_unbounded_string_width,
    )?;
    Ok(())
}

fn register_text_alignment(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "SetJustifyH", text::set_justify_h)?;
    table_set_rust_fn_static(state, table, "GetJustifyH", text::get_justify_h)?;
    table_set_rust_fn_static(state, table, "SetJustifyV", text::set_justify_v)?;
    table_set_rust_fn_static(state, table, "GetJustifyV", text::get_justify_v)?;
    Ok(())
}

fn register_text_wrapping(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "SetWordWrap", text::set_word_wrap)?;
    table_set_rust_fn_static(state, table, "GetWordWrap", text::get_word_wrap)?;
    table_set_rust_fn_static(state, table, "CanWordWrap", text::can_word_wrap)?;
    table_set_rust_fn_static(state, table, "SetMaxLines", text::set_max_lines)?;
    table_set_rust_fn_static(state, table, "GetMaxLines", text::get_max_lines)?;
    table_set_rust_fn_static(state, table, "SetNonSpaceWrap", text::set_non_space_wrap)?;
    table_set_rust_fn_static(state, table, "CanNonSpaceWrap", text::can_non_space_wrap)?;
    Ok(())
}

fn register_text_scaling(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    #[cfg(feature = "retail-12-0-0")]
    {
        table_set_rust_fn_static(
            state,
            table,
            "GetScaleAnimationMode",
            text::get_scale_animation_mode,
        )?;
        table_set_rust_fn_static(
            state,
            table,
            "SetScaleAnimationMode",
            text::set_scale_animation_mode,
        )?;
    }
    table_set_rust_fn_static(state, table, "GetTextScale", text::get_text_scale)?;
    table_set_rust_fn_static(state, table, "SetTextScale", text::set_text_scale)?;
    table_set_rust_fn_static(state, table, "SetTextToFit", text::set_text_to_fit)?;
    table_set_rust_fn_static(state, table, "ScaleTextToFit", text::scale_text_to_fit)?;
    Ok(())
}

fn register_styled_text(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    register_text_colors(state, table)?;
    register_text_hyperlinks(state, table)?;
    register_text_wrap_style(state, table)?;
    Ok(())
}

fn register_text_colors(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "SetTextColor", text::set_text_color)?;
    table_set_rust_fn_static(state, table, "GetTextColor", text::get_text_color)?;
    table_set_rust_fn_static(state, table, "SetFixedColor", text::set_fixed_color)?;
    Ok(())
}

fn register_text_hyperlinks(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table,
        "SetHyperlinksEnabled",
        text::set_hyperlinks_enabled,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "GetHyperlinksEnabled",
        text::get_hyperlinks_enabled,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "SetHyperlinkFormat",
        text::set_hyperlink_format,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "GetHyperlinkFormat",
        text::get_hyperlink_format,
    )?;
    Ok(())
}

fn register_text_wrap_style(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table,
        "SetIndentedWordWrap",
        text::set_indented_word_wrap,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "GetIndentedWordWrap",
        text::get_indented_word_wrap,
    )?;
    Ok(())
}

fn register_attribute_methods(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    register_attribute_core(state, table)?;
    register_attribute_frame_refs(state, table)?;
    register_attribute_protection(state, table)?;
    register_attribute_behavior_flags(state, table)?;
    register_attribute_hit_rect(state, table)?;
    Ok(())
}

fn register_attribute_core(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "GetAttribute", attributes::get_attribute)?;
    table_set_rust_fn_static(state, table, "SetAttribute", attributes::set_attribute)?;
    table_set_rust_fn_static(
        state,
        table,
        "SetAttributeNoHandler",
        attributes::set_attribute_no_handler,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "ClearAttributes",
        attributes::clear_attributes,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "ExecuteAttribute",
        attributes::execute_attribute,
    )?;
    table_set_rust_fn_static(state, table, "ChildUpdate", attributes::child_update)?;
    register_call_method(state, table)?;
    Ok(())
}

fn register_call_method(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    let loader = state.load(CALL_METHOD_LUA)?;
    let closure = call_function_state(state, Val::Function(loader.gc_ref()), &[])?;
    let key = Val::Str(state.gc.intern_string_static(b"CallMethod"));
    if let Some(methods) = state.gc.tables.get_mut(table) {
        methods.raw_set(key, closure, &state.gc.string_arena)?;
    }
    state.gc.barrier_back(table);
    Ok(())
}

const CALL_METHOD_LUA: &str = r##"
return function(frame, methodName, ...)
    local callerTaint = debug.getstacktaint()
    forceinsecure()
    if callerTaint ~= nil then
        debug.setstacktaint(callerTaint)
    end
    if type(methodName) ~= "string" then
        error("Method name must be a string")
    end
    local method = frame[methodName]
    if type(method) ~= "function" then
        error("Invalid method '" .. methodName .. "'")
    end
    if callerTaint ~= nil then
        __sim_mark_secret_value(...)
    end

    local function pack(...)
        return { n = select("#", ...), ... }
    end
    local results = pack(method(frame, ...))
    if callerTaint ~= nil then
        for index = 1, select("#", ...) do
            local value = select(index, ...)
            if type(value) == "string" then
                __sim_mark_secret_value(value)
            end
        end
        for index = 1, results.n do
            local value = results[index]
            if type(value) == "string" then
                __sim_mark_secret_value(value)
            end
        end
    end
    return unpack(results, 1, results.n)
end
"##;

fn register_attribute_frame_refs(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "SetFrameRef", attributes::set_frame_ref)?;
    table_set_rust_fn_static(state, table, "GetFrameRef", attributes::get_frame_ref)?;
    Ok(())
}

fn register_attribute_protection(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "SetForbidden", attributes::set_forbidden)?;
    table_set_rust_fn_static(
        state,
        table,
        "HasAccessConstraints",
        attributes::has_access_constraints,
    )?;
    table_set_rust_fn_static(state, table, "IsForbidden", attributes::is_forbidden)?;
    table_set_rust_fn_static(
        state,
        table,
        "CanChangeProtectedState",
        attributes::can_change_protected_state,
    )?;
    Ok(())
}

fn register_attribute_behavior_flags(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    register_attribute_input_render_flags(state, table)?;
    register_attribute_motion_scripts(state, table)?;
    register_attribute_clip_children(state, table)?;
    Ok(())
}

fn register_attribute_input_render_flags(
    state: &mut LuaState,
    table: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table,
        "SetPassThroughButtons",
        attributes::set_pass_through_buttons,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "SetFlattensRenderLayers",
        attributes::set_flattens_render_layers,
    )?;
    Ok(())
}

fn register_attribute_motion_scripts(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table,
        "SetMotionScriptsWhileDisabled",
        attributes::set_motion_scripts_while_disabled,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "GetMotionScriptsWhileDisabled",
        attributes::get_motion_scripts_while_disabled,
    )?;
    Ok(())
}

fn register_attribute_clip_children(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table,
        "SetClipsChildren",
        attributes::set_clips_children,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "DoesClipChildren",
        attributes::does_clip_children,
    )?;
    Ok(())
}

fn register_attribute_hit_rect(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table,
        "SetHitRectInsets",
        attributes::set_hit_rect_insets,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "GetHitRectInsets",
        attributes::get_hit_rect_insets,
    )?;
    Ok(())
}

fn register_event_methods(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    register_event_registration(state, table)?;
    register_event_callbacks(state, table)?;
    register_event_keyboard_propagation(state, table)?;
    register_event_script_handlers(state, table)?;
    Ok(())
}

fn register_event_registration(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "RegisterEvent", events::register_event)?;
    table_set_rust_fn_static(
        state,
        table,
        "RegisterUnitEvent",
        events::register_unit_event,
    )?;
    table_set_rust_fn_static(state, table, "UnregisterEvent", events::unregister_event)?;
    table_set_rust_fn_static(state, table, "UnRegisterEvent", events::unregister_event)?;
    table_set_rust_fn_static(
        state,
        table,
        "UnregisterAllEvents",
        events::unregister_all_events,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "RegisterAllEvents",
        events::register_all_events,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "IsEventRegistered",
        events::is_event_registered,
    )?;
    Ok(())
}

fn register_event_callbacks(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    register_event_callback_listeners(state, table)?;
    register_named_callback_table(state, table)?;
    Ok(())
}

fn register_event_callback_listeners(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table,
        "RegisterEventCallback",
        events::register_event_callback,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "RegisterUnitEventCallback",
        events::register_unit_event_callback,
    )?;
    Ok(())
}

fn register_named_callback_table(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    register_callback_registry_methods(state, table)?;
    register_menu_selection_callbacks(state, table)?;
    register_menu_default_callbacks(state, table)?;
    register_menu_interaction_callbacks(state, table)?;
    Ok(())
}

fn register_callback_registry_methods(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table,
        "RegisterCallback",
        callbacks::register_callback,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "UnregisterCallback",
        callbacks::unregister_callback,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "AddStaticEventMethod",
        callbacks::add_static_event_method,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "TriggerEvent",
        callbacks::trigger_callback_event,
    )?;
    Ok(())
}

fn register_menu_selection_callbacks(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    register_menu_setup_callbacks(state, table)?;
    register_menu_selection_text_callbacks(state, table)?;
    Ok(())
}

fn register_menu_setup_callbacks(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "SetupMenu", callbacks::setup_menu)?;
    table_set_rust_fn_static(state, table, "SetDefaultText", callbacks::set_default_text)?;
    register_menu_selection_setters(state, table)?;
    table_set_rust_fn_static(
        state,
        table,
        "EnableRegenerateOnResponse",
        callbacks::enable_regenerate_on_response,
    )?;
    Ok(())
}

fn register_menu_selection_setters(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table,
        "SetSelectionTranslator",
        callbacks::set_selection_translator,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "SetSelectionText",
        callbacks::set_selection_text,
    )?;
    Ok(())
}

fn register_menu_selection_text_callbacks(
    state: &mut LuaState,
    table: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table,
        "GetSelectionText",
        callbacks::get_selection_text,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "UpdateToMenuSelections",
        callbacks::update_to_menu_selections,
    )?;
    Ok(())
}

fn register_menu_default_callbacks(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table,
        "SetDefaultCallback",
        callbacks::set_default_callback,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "SetIsDefaultCallback",
        callbacks::set_is_default_callback,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "SetUpdateCallback",
        callbacks::set_update_callback,
    )?;
    table_set_rust_fn_static(state, table, "NotifyUpdate", callbacks::notify_update)?;
    Ok(())
}

fn register_menu_interaction_callbacks(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table,
        "SetOnClickHandler",
        callbacks::set_on_click_handler,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "SetOnEnterHandler",
        callbacks::set_on_enter_handler,
    )?;
    Ok(())
}

fn register_event_keyboard_propagation(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table,
        "SetPropagateKeyboardInput",
        events::set_propagate_keyboard_input,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "GetPropagateKeyboardInput",
        events::get_propagate_keyboard_input,
    )?;
    Ok(())
}

fn register_event_script_handlers(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "SetScript", events::set_script)?;
    table_set_rust_fn_static(state, table, "GetScript", events::get_script)?;
    table_set_rust_fn_static(state, table, "HasScript", events::has_script)?;
    table_set_rust_fn_static(state, table, "HookScript", events::hook_script)?;
    Ok(())
}
