//! Runtime template chain application: applies XML template inheritance to
//! frames created via `CreateFrame("Frame", name, parent, "TemplateName")`.

mod builders;
mod fast_types;
mod parser;
mod runtime;
mod runtime_loader_effects;
mod runtime_synthetic_children;

pub(super) use fast_types::{FastHandlerRef, FastLiteralValue, FastScriptInstall};

use super::helpers::{
    append_parent_array_entry, apply_frame_mixin_with_partitions, apply_frame_mixins,
    resolve_global_path,
};
use crate::lua_api::methods::{borrow_state, call_function_state, create_string, frame_ref};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use rustc_hash::FxHashSet;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

enum RuntimeTemplateOverrides<'a> {
    None,
    Frame(&'a crate::xml::FrameXml),
    Initializer(Val),
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub(crate) fn apply_runtime_template_chain(
    state: &mut LuaState,
    frame_id: u64,
    inherits: Option<&str>,
    fire_on_load: bool,
) -> LuaResult<()> {
    let overrides = RuntimeTemplateOverrides::None;
    apply_runtime_template_chain_impl(state, frame_id, inherits, fire_on_load, overrides)
}

pub(crate) fn replay_runtime_template_parent_links(
    state: &mut LuaState,
    frame_id: u64,
    inherits: Option<&str>,
) -> LuaResult<()> {
    let Some(inherits) = inherits.filter(|value| !value.trim().is_empty()) else {
        return Ok(());
    };
    let chain = crate::xml::get_template_chain(inherits);
    if chain.is_empty() {
        return Ok(());
    }
    apply_template_parent_links(state, frame_id, &chain)
}

pub(crate) use runtime_synthetic_children::ensure_runtime_slider_children;

pub(crate) fn apply_runtime_template_chain_with_frame_overrides(
    state: &mut LuaState,
    frame_id: u64,
    inherits: Option<&str>,
    fire_on_load: bool,
    frame: &crate::xml::FrameXml,
) -> LuaResult<()> {
    let overrides = RuntimeTemplateOverrides::Frame(frame);
    apply_runtime_template_chain_impl(state, frame_id, inherits, fire_on_load, overrides)
}

pub(super) fn apply_runtime_template_chain_with_initializer(
    state: &mut LuaState,
    frame_id: u64,
    inherits: Option<&str>,
    fire_on_load: bool,
    initializer: Val,
) -> LuaResult<()> {
    let overrides = match initializer {
        Val::Function(_) => RuntimeTemplateOverrides::Initializer(initializer),
        _ => RuntimeTemplateOverrides::None,
    };
    apply_runtime_template_chain_impl(state, frame_id, inherits, fire_on_load, overrides)
}

fn apply_runtime_template_chain_impl(
    state: &mut LuaState,
    frame_id: u64,
    inherits: Option<&str>,
    fire_on_load: bool,
    overrides: RuntimeTemplateOverrides<'_>,
) -> LuaResult<()> {
    let Some(inherits) = inherits.filter(|value| !value.trim().is_empty()) else {
        apply_runtime_template_overrides(state, frame_id, overrides)?;
        return Ok(());
    };
    let chain = crate::xml::get_template_chain(inherits);
    if chain.is_empty() {
        apply_runtime_template_overrides(state, frame_id, overrides)?;
        return Ok(());
    }

    let state_rc = sim_state_rc(state)?;
    let frame_name = frame_lookup_name(state, frame_id);
    apply_template_parent_links(state, frame_id, &chain)?;
    apply_chain_entries(state, frame_id, &chain)?;
    apply_runtime_template_overrides(state, frame_id, overrides)?;
    apply_runtime_template_loader_effects(state, frame_id, inherits, &frame_name, &chain)?;
    create_runtime_template_child_frames(state, &state_rc, frame_id, &frame_name, &chain)?;

    finalize_template_frame(
        state,
        &state_rc,
        frame_id,
        inherits,
        &frame_name,
        fire_on_load,
    )
}

fn apply_runtime_template_overrides(
    state: &mut LuaState,
    frame_id: u64,
    overrides: RuntimeTemplateOverrides<'_>,
) -> LuaResult<()> {
    match overrides {
        RuntimeTemplateOverrides::None => Ok(()),
        RuntimeTemplateOverrides::Frame(frame) => {
            crate::lua_api::frame::methods::forbidden_aspects::apply_xml_forbidden_aspects(
                state, frame_id, frame,
            )?;
            apply_template_partition_marker(state, frame_id, frame);
            apply_template_key_values(state, frame_id, frame.all_key_values());
            Ok(())
        }
        RuntimeTemplateOverrides::Initializer(initializer) => {
            let frame = frame_ref(state, frame_id)?;
            crate::lua_api::script_helpers::call_void_function_state(state, initializer, &[frame])
                .map_err(rilua::runtime_error)
        }
    }
}

fn create_runtime_template_child_frames(
    state: &mut LuaState,
    state_rc: &Rc<RefCell<crate::lua_api::SimState>>,
    frame_id: u64,
    frame_name: &str,
    chain: &[Arc<crate::xml::TemplateEntry>],
) -> LuaResult<()> {
    let mut deferred_on_loads = Vec::new();
    for entry in chain {
        runtime::create_template_child_frames(
            state,
            state_rc,
            frame_id,
            frame_name,
            frame_name,
            &entry.frame,
            &mut deferred_on_loads,
        )?;
    }
    fire_deferred_child_on_loads(state, &deferred_on_loads)?;
    Ok(())
}

fn fire_deferred_child_on_loads(state: &mut LuaState, child_ids: &[u64]) -> LuaResult<()> {
    for child_id in child_ids {
        runtime::fire_frame_on_load(state, *child_id)?;
        resolve_runtime_template_named_anchors(state, *child_id)?;
    }
    Ok(())
}

fn apply_runtime_template_loader_effects(
    state: &mut LuaState,
    frame_id: u64,
    inherits: &str,
    frame_name: &str,
    chain: &[Arc<crate::xml::TemplateEntry>],
) -> LuaResult<()> {
    let runtime_frame = crate::xml::FrameXml::default();
    runtime::apply_runtime_template_loader_effects(
        state,
        frame_id,
        frame_name,
        frame_name,
        &runtime_frame,
        Some(inherits),
    )?;
    runtime::repair_runtime_template_chain_layer_parent_keys(state, frame_id, frame_name, chain)
}

// ---------------------------------------------------------------------------
// Chain application
// ---------------------------------------------------------------------------

fn apply_template_parent_links(
    state: &mut LuaState,
    frame_id: u64,
    chain: &[Arc<crate::xml::TemplateEntry>],
) -> LuaResult<()> {
    let template_parent_key = chain
        .iter()
        .rev()
        .find_map(|entry| entry.frame.parent_key.as_deref());
    let template_parent_array = chain
        .iter()
        .rev()
        .find_map(|entry| entry.frame.parent_array.as_deref());
    let parent_id = borrow_state(state)
        .ok()
        .and_then(|sim| sim.widgets.get(frame_id).and_then(|frame| frame.parent_id));
    let Some(parent_id) = parent_id else {
        return Ok(());
    };
    if let Some(parent_key) = template_parent_key {
        crate::lua_api::globals::template::assign_parent_key(
            state, parent_id, parent_key, frame_id,
        )?;
    }
    if let Some(parent_array) = template_parent_array {
        append_parent_array_entry(state, parent_id, parent_array, frame_id);
    }
    Ok(())
}

fn apply_chain_entries(
    state: &mut LuaState,
    frame_id: u64,
    chain: &[Arc<crate::xml::TemplateEntry>],
) -> LuaResult<()> {
    for entry in chain {
        crate::lua_api::frame::methods::forbidden_aspects::apply_xml_forbidden_aspects(
            state,
            frame_id,
            &entry.frame,
        )?;
    }
    for entry in chain {
        runtime::ensure_runtime_button_texture_slots(state, frame_id, &entry.frame)?;
        apply_template_partition_marker(state, frame_id, &entry.frame);
        apply_frame_mixins(state, frame_id, entry.frame.combined_mixin().as_deref())?;
        let previous_local_source = install_template_local_source(state, entry.local_source);
        apply_block_mixins(state, frame_id, entry.frame.mixins())?;
        apply_template_key_values(state, frame_id, entry.frame.all_key_values());
        restore_template_local_source(state, previous_local_source);
        let entry_is_intrinsic = entry.frame.intrinsic == Some(true);
        if let Some(scripts) = entry.frame.scripts() {
            apply_template_scripts_impl(state, frame_id, scripts, entry_is_intrinsic)?;
        }
    }
    Ok(())
}

fn finalize_template_frame(
    state: &mut LuaState,
    state_rc: &Rc<RefCell<crate::lua_api::SimState>>,
    frame_id: u64,
    inherits: &str,
    frame_name: &str,
    fire_on_load: bool,
) -> LuaResult<()> {
    runtime::apply_runtime_template_direct_properties(state_rc, frame_id, inherits, frame_name);
    crate::lua_api::globals::template::repair_direct_child_parent_keys(state, frame_id)?;
    crate::lua_api::globals::template::repair_transparent_descendant_parent_key_aliases(
        state, frame_id,
    )?;
    crate::lua_api::globals::template::repair_descendant_name_aliases(state, frame_id)?;
    resolve_runtime_template_named_anchors(state, frame_id)?;
    if fire_on_load {
        runtime::fire_frame_on_load(state, frame_id)?;
    }

    // `OnLoad` handlers can re-anchor using named keys (for example after
    // reading runtime state). Re-resolve one more time after hooks run so late
    // named-key anchors are still fixed against freshly created sibling layers.
    resolve_runtime_template_named_anchors(state, frame_id)?;
    Ok(())
}

/// Re-resolve `$parent.X` style anchors for a runtime template frame subtree
/// once both child frames and layer regions exist.
///
/// `set_single_anchor` records the unresolved relative-key string when a sibling
/// hasn't been created yet (template child frames are created before layer
/// fontstrings/textures). Without this pass the anchors stay unresolved and
/// children fall back to anchoring against their parent.
pub(crate) fn resolve_runtime_template_named_anchors(
    state: &mut LuaState,
    frame_id: u64,
) -> LuaResult<()> {
    let mut sim = crate::lua_api::methods::borrow_state_mut(state)
        .map_err(|error| rilua::runtime_error(error.to_string()))?;
    let mut todo = vec![frame_id];
    let mut seen = FxHashSet::default();
    while let Some(current_id) = todo.pop() {
        if !seen.insert(current_id) {
            continue;
        }
        sim.widgets
            .resolve_named_anchor_targets_for_frame(current_id);
        sim.widgets.mark_rect_dirty(current_id);
        let child_ids = sim
            .widgets
            .get(current_id)
            .map(|frame| frame.children.clone())
            .unwrap_or_default();
        todo.extend(child_ids);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Script helpers
// ---------------------------------------------------------------------------

pub(super) fn apply_template_partition_marker(
    state: &mut LuaState,
    frame_id: u64,
    frame: &crate::xml::FrameXml,
) {
    if frame.use_forbidden_object_table {
        crate::lua_api::globals::create_frame::mark_frame_uses_forbidden_object_table(
            state, frame_id,
        );
    }
}

fn install_template_local_source(state: &mut LuaState, local_source: Option<Val>) -> Val {
    let globals = Val::Table(state.global);
    let previous =
        crate::lua_api::methods::table_get_static(state, globals, "__wow_loading_addon_table");
    if let Some(local_source) = local_source {
        crate::lua_api::methods::table_set_static(
            state,
            globals,
            "__wow_loading_addon_table",
            local_source,
        );
    }
    previous
}

fn restore_template_local_source(state: &mut LuaState, previous: Val) {
    let globals = Val::Table(state.global);
    crate::lua_api::methods::table_set_static(
        state,
        globals,
        "__wow_loading_addon_table",
        previous,
    );
}

pub(super) fn apply_block_mixins(
    state: &mut LuaState,
    frame_id: u64,
    mixins: Option<&crate::xml::MixinsXml>,
) -> LuaResult<()> {
    let Some(mixins) = mixins else {
        return Ok(());
    };
    for mixin in &mixins.entries {
        apply_frame_mixin_with_partitions(
            state,
            frame_id,
            &mixin.key,
            mixin.source.as_deref(),
            mixin.target_partition.as_deref(),
            mixin.inbound_partition.as_deref(),
            mixin.secure_delegates.unwrap_or(false),
        )?;
    }
    Ok(())
}

fn apply_template_key_values<'a>(
    state: &mut LuaState,
    frame_id: u64,
    key_values: impl Iterator<Item = &'a crate::xml::KeyValuesXml>,
) {
    let frame = frame_ref(state, frame_id).ok();
    let Some(frame) = frame else { return };

    for key_block in key_values {
        for entry in &key_block.values {
            let value = template_key_value(
                state,
                &entry.key,
                &entry.value,
                entry.value_type.as_deref(),
                entry.source.as_deref(),
            );
            apply_template_key_value(state, frame, &entry.key, value);
        }
    }
}

fn apply_template_key_value(state: &mut LuaState, frame: Val, key: &str, value: Val) {
    let helper = resolve_global_path(state, "__wow_xml_set_key_value");
    let key = create_string(state, key);
    let _ = call_function_state(state, helper, &[frame, key, value]);
}

pub(crate) fn apply_template_scripts(
    state: &mut LuaState,
    frame_id: u64,
    scripts: &crate::xml::ScriptsXml,
) -> LuaResult<()> {
    apply_template_scripts_impl(state, frame_id, scripts, false)
}

fn apply_template_scripts_impl(
    state: &mut LuaState,
    frame_id: u64,
    scripts: &crate::xml::ScriptsXml,
    intrinsic_default_scripts: bool,
) -> LuaResult<()> {
    if apply_fast_scripts(state, frame_id, scripts, intrinsic_default_scripts)? {
        return Ok(());
    }

    let script_code = if intrinsic_default_scripts {
        crate::loader::helpers::generate_intrinsic_scripts_code(scripts)
    } else {
        crate::loader::helpers::generate_scripts_code(scripts)
    };
    if script_code.trim().is_empty() {
        return Ok(());
    }

    let chunk = format!("local frame = ...\n{script_code}");
    let saved_slots = state.global_slots.take();
    let func =
        crate::loader::chunk_cache::load_chunk(state, &chunk, "template-scripts-no-global-slots")
            .map_err(|error| rilua::runtime_error(error.to_string()));
    state.global_slots = saved_slots;
    let func = func?;
    crate::lua_api::loader_env::apply_loading_scoped_fenv_state(state, &func)
        .map_err(|error| rilua::runtime_error(error.to_string()))?;
    let frame = frame_ref(state, frame_id)?;
    match crate::lua_api::script_helpers::call_void_function_state(
        state,
        Val::Function(func.gc_ref()),
        &[frame],
    ) {
        Ok(_) => {}
        Err(error) => return Err(rilua::runtime_error(error)),
    }
    Ok(())
}

pub(crate) fn scripts_support_fast_install(scripts: &crate::xml::ScriptsXml) -> bool {
    collect_fast_handlers(scripts, false).is_some()
}

pub(crate) fn first_fast_install_miss(scripts: &crate::xml::ScriptsXml) -> Option<String> {
    first_fast_handler_miss_in_group(base_method_only_handlers(scripts))
        .or_else(|| first_fast_handler_miss_in_group(pointer_method_only_handlers(scripts)))
        .or_else(|| first_fast_handler_miss_in_group(text_method_only_handlers(scripts)))
        .or_else(|| first_fast_handler_miss_in_group(state_method_only_handlers(scripts)))
}

fn apply_fast_scripts(
    state: &mut LuaState,
    frame_id: u64,
    scripts: &crate::xml::ScriptsXml,
    intrinsic_default_scripts: bool,
) -> LuaResult<bool> {
    let Some(installs) = collect_fast_handlers(scripts, intrinsic_default_scripts) else {
        return Ok(false);
    };
    if installs.is_empty() {
        return Ok(true);
    }

    for (handler_name, install) in installs {
        builders::install_fast_handler(state, frame_id, handler_name, install)?;
    }

    Ok(true)
}

fn collect_fast_handlers(
    scripts: &crate::xml::ScriptsXml,
    intrinsic_default_scripts: bool,
) -> Option<Vec<(&'static str, FastScriptInstall<'_>)>> {
    let mut handlers = Vec::new();
    collect_fast_handler_group(
        &mut handlers,
        base_method_only_handlers(scripts),
        intrinsic_default_scripts,
    )?;
    collect_fast_handler_group(
        &mut handlers,
        pointer_method_only_handlers(scripts),
        intrinsic_default_scripts,
    )?;
    collect_fast_handler_group(
        &mut handlers,
        text_method_only_handlers(scripts),
        intrinsic_default_scripts,
    )?;
    collect_fast_handler_group(
        &mut handlers,
        state_method_only_handlers(scripts),
        intrinsic_default_scripts,
    )?;
    Some(handlers)
}

type MethodOnlyScript<'a> = (&'static str, Option<&'a crate::xml::ScriptBodyXml>);
type FastHandler<'a> = (&'static str, Option<&'a crate::xml::ScriptBodyXml>);

fn collect_fast_handler_group<'a>(
    handlers: &mut Vec<(&'static str, FastScriptInstall<'a>)>,
    group: impl IntoIterator<Item = FastHandler<'a>>,
    intrinsic_default_scripts: bool,
) -> Option<()> {
    for (handler_name, script) in group {
        let Some(script) = script else {
            continue;
        };
        let install = fast_script_install(handler_name, script, intrinsic_default_scripts)?;
        handlers.push((handler_name, install));
    }
    Some(())
}

fn first_fast_handler_miss_in_group<'a>(
    group: impl IntoIterator<Item = FastHandler<'a>>,
) -> Option<String> {
    for (handler_name, script) in group {
        let Some(script) = script else {
            continue;
        };
        if fast_script_install(handler_name, script, false).is_none() {
            return Some(describe_fast_script_miss(handler_name, script));
        }
    }
    None
}

fn fast_script_install<'a>(
    handler_name: &'static str,
    script: &'a crate::xml::ScriptBodyXml,
    intrinsic_default_scripts: bool,
) -> Option<FastScriptInstall<'a>> {
    let handler = fast_handler_ref(handler_name, script)?;
    if matches!(handler, FastHandlerRef::NoOp) {
        return Some(FastScriptInstall::Set(handler));
    }
    match script.intrinsic_order.as_deref() {
        Some("precall") => Some(FastScriptInstall::Intrinsic {
            handler,
            new_first: true,
        }),
        Some("postcall") => Some(FastScriptInstall::Intrinsic {
            handler,
            new_first: false,
        }),
        Some(_) => None,
        None => match script.inherit.as_deref() {
            Some("append") => Some(FastScriptInstall::Chain {
                handler,
                new_first: true,
            }),
            Some("prepend") => Some(FastScriptInstall::Chain {
                handler,
                new_first: false,
            }),
            Some(_) => None,
            None if intrinsic_default_scripts => Some(FastScriptInstall::Intrinsic {
                handler,
                new_first: true,
            }),
            None => Some(FastScriptInstall::Set(handler)),
        },
    }
}

fn fast_handler_ref<'a>(
    handler_name: &'static str,
    script: &'a crate::xml::ScriptBodyXml,
) -> Option<FastHandlerRef<'a>> {
    if let Some(method_name) = script.method.as_deref() {
        return Some(FastHandlerRef::Method(method_name));
    }
    if let Some(function_name) = script.function.as_deref() {
        return Some(FastHandlerRef::Function(function_name));
    }
    if let Some(body) = script.body.as_deref() {
        return parser::parse_inline_fast_handler(handler_name, body);
    }
    Some(FastHandlerRef::NoOp)
}

fn describe_fast_script_miss(handler_name: &str, script: &crate::xml::ScriptBodyXml) -> String {
    let body = script.body.as_deref().unwrap_or("");
    let body = body.trim().replace('\n', " ");
    if let Some(intrinsic_order) = script.intrinsic_order.as_deref() {
        format!("{handler_name}|intrinsic={intrinsic_order}|{body}")
    } else if let Some(inherit) = script.inherit.as_deref() {
        format!("{handler_name}|inherit={inherit}|{body}")
    } else if let Some(method_name) = script.method.as_deref() {
        format!("{handler_name}|method={method_name}|{body}")
    } else if let Some(function_name) = script.function.as_deref() {
        format!("{handler_name}|function={function_name}|{body}")
    } else {
        format!("{handler_name}|{body}")
    }
}

fn base_method_only_handlers(scripts: &crate::xml::ScriptsXml) -> [MethodOnlyScript<'_>; 8] {
    [
        ("OnLoad", scripts.on_load.last()),
        ("OnEvent", scripts.on_event.last()),
        ("OnUpdate", scripts.on_update.last()),
        ("OnClick", scripts.on_click.last()),
        ("PreClick", scripts.pre_click.last()),
        ("PostClick", scripts.post_click.last()),
        ("OnShow", scripts.on_show.last()),
        ("OnHide", scripts.on_hide.last()),
    ]
}

fn pointer_method_only_handlers(scripts: &crate::xml::ScriptsXml) -> [MethodOnlyScript<'_>; 8] {
    [
        ("OnEnter", scripts.on_enter.last()),
        ("OnLeave", scripts.on_leave.last()),
        ("OnMouseDown", scripts.on_mouse_down.last()),
        ("OnMouseUp", scripts.on_mouse_up.last()),
        ("OnMouseWheel", scripts.on_mouse_wheel.last()),
        ("OnDragStart", scripts.on_drag_start.last()),
        ("OnDragStop", scripts.on_drag_stop.last()),
        ("OnReceiveDrag", scripts.on_receive_drag.last()),
    ]
}

fn text_method_only_handlers(scripts: &crate::xml::ScriptsXml) -> [MethodOnlyScript<'_>; 11] {
    [
        ("OnEnterPressed", scripts.on_enter_pressed.last()),
        ("OnEscapePressed", scripts.on_escape_pressed.last()),
        ("OnTabPressed", scripts.on_tab_pressed.last()),
        ("OnSpacePressed", scripts.on_space_pressed.last()),
        ("OnArrowPressed", scripts.on_arrow_pressed.last()),
        ("OnTextChanged", scripts.on_text_changed.last()),
        ("OnTextSet", scripts.on_text_set.last()),
        ("OnChar", scripts.on_char.last()),
        ("OnEditFocusGained", scripts.on_edit_focus_gained.last()),
        ("OnEditFocusLost", scripts.on_edit_focus_lost.last()),
        (
            "OnInputLanguageChanged",
            scripts.on_input_language_changed.last(),
        ),
    ]
}

fn state_method_only_handlers(scripts: &crate::xml::ScriptsXml) -> [MethodOnlyScript<'_>; 10] {
    [
        ("OnKeyDown", scripts.on_key_down.last()),
        ("OnKeyUp", scripts.on_key_up.last()),
        ("OnValueChanged", scripts.on_value_changed.last()),
        ("OnEnable", scripts.on_enable.last()),
        ("OnDisable", scripts.on_disable.last()),
        ("OnSizeChanged", scripts.on_size_changed.last()),
        ("OnAttributeChanged", scripts.on_attribute_changed.last()),
        ("OnHyperlinkClick", scripts.on_hyperlink_click.last()),
        ("OnHyperlinkEnter", scripts.on_hyperlink_enter.last()),
        ("OnHyperlinkLeave", scripts.on_hyperlink_leave.last()),
    ]
}

fn template_key_value(
    state: &mut LuaState,
    key: &str,
    value: &str,
    value_type: Option<&str>,
    source: Option<&str>,
) -> Val {
    match (value_type, source) {
        (Some("number"), _) => value.parse::<f64>().map(Val::Num).unwrap_or(Val::Nil),
        (Some("boolean"), _) => Val::Bool(value.eq_ignore_ascii_case("true")),
        (Some("global"), _) => resolve_global_path(state, value),
        (Some("local"), _) | (_, Some("local")) => {
            let local_key = if value.is_empty() { key } else { value };
            resolve_local_template_path(state, local_key)
        }
        // Auto-detect numbers when type is not specified (WoW behavior)
        (None, _) if value.parse::<f64>().is_ok() => Val::Num(value.parse().unwrap()),
        _ => create_string(state, value),
    }
}

fn resolve_local_template_path(state: &mut LuaState, path: &str) -> Val {
    let globals = Val::Table(state.global);
    let local_source =
        crate::lua_api::methods::table_get_static(state, globals, "__wow_loading_addon_table");
    resolve_table_path(state, local_source, path)
}

fn resolve_table_path(state: &mut LuaState, root: Val, path: &str) -> Val {
    let mut current = root;
    for segment in path
        .split('.')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
    {
        let Val::Table(table_ref) = current else {
            return Val::Nil;
        };
        let key = state.gc.intern_string(segment.as_bytes());
        current = state
            .gc
            .tables
            .get(table_ref)
            .map(|table| table.get_str(key, &state.gc.string_arena))
            .unwrap_or(Val::Nil);
    }
    current
}

#[cfg(test)]
mod tests {
    use super::{first_fast_install_miss, parser, scripts_support_fast_install};
    use crate::xml::{ScriptBodyXml, ScriptsXml};

    #[test]
    fn scripts_support_fast_install_for_character_frame_tooltip_body() {
        let body = r#"
                        GameTooltip:SetOwner(self, "ANCHOR_RIGHT");
                        GameTooltip:SetText(MicroButtonTooltipText(CHARACTER_INFO, "TOGGLECHARACTER0"), 1.0,1.0,1.0 );
                    "#;
        assert!(
            parser::parse_inline_fast_handler("OnEnter", body).is_some(),
            "parser miss"
        );
        let scripts = ScriptsXml {
            on_enter: vec![ScriptBodyXml {
                body: Some(body.to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(
            scripts_support_fast_install(&scripts),
            "miss={:?}",
            first_fast_install_miss(&scripts)
        );
    }

    #[test]
    fn parser_supports_chat_config_prefix_conditional_suffix_sequence() {
        let body = r#"
                        HideUIPanel(ChatConfigFrame);
                        if ( IsCombatLog(FCF_GetCurrentChatFrame()) ) then
                            Blizzard_CombatLog_RefreshGlobalLinks();
                            C_CombatLog.ApplyFilterSettings(Blizzard_CombatLog_CurrentSettings);
                            C_CombatLog.RefilterEntries();
                        end
                        PlaySound(SOUNDKIT.GS_TITLE_OPTION_OK);
                    "#;
        assert!(
            parser::parse_inline_fast_handler("OnClick", body).is_some(),
            "parser miss"
        );
        let scripts = ScriptsXml {
            on_click: vec![ScriptBodyXml {
                body: Some(body.to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(
            scripts_support_fast_install(&scripts),
            "miss={:?}",
            first_fast_install_miss(&scripts)
        );
    }
}

// ---------------------------------------------------------------------------
// Small utility helpers
// ---------------------------------------------------------------------------

pub(super) fn template_child_name(name: Option<&str>, subst_parent: &str) -> String {
    name.map(|name| name.replace("$parent", subst_parent))
        .unwrap_or_else(|| format!("__tpl_{}", crate::loader::helpers::rand_id()))
}

pub(super) fn build_child_inherits(
    intrinsic: Option<&str>,
    inherits: Option<&str>,
) -> Option<String> {
    match (intrinsic, inherits.filter(|value| !value.trim().is_empty())) {
        (Some(base), Some(inherits)) => Some(format!("{base}, {inherits}")),
        (Some(base), None) => Some(base.to_string()),
        (None, Some(inherits)) => Some(inherits.to_string()),
        (None, None) => None,
    }
}

pub(super) fn resolve_inherited_hidden(
    frame: &crate::xml::FrameXml,
    inherits: Option<&str>,
) -> bool {
    if let Some(hidden) = frame.hidden {
        return hidden;
    }

    let Some(inherits) = inherits.filter(|value| !value.trim().is_empty()) else {
        return false;
    };

    crate::xml::get_template_chain(inherits)
        .iter()
        .find_map(|entry| entry.frame.hidden)
        .unwrap_or(false)
}

pub(super) fn frame_lookup_name(state: &LuaState, frame_id: u64) -> String {
    borrow_state(state)
        .ok()
        .and_then(|sim| {
            sim.widgets
                .get(frame_id)
                .and_then(|frame| frame.name.clone())
        })
        .unwrap_or_else(|| format!("__frame_{frame_id}"))
}

pub(super) fn sim_state_rc(state: &LuaState) -> LuaResult<Rc<RefCell<crate::lua_api::SimState>>> {
    state
        .app_data::<crate::lua_api::env::WowLuaAppData>()
        .map(|app| app.sim_state.clone())
        .ok_or_else(|| rilua::runtime_error("missing WowLuaAppData"))
}

fn template_child_type<'a>(
    frame: &'a crate::xml::FrameXml,
    tag: &'static str,
) -> Option<(&'a crate::xml::FrameXml, &'static str, Option<&'static str>)> {
    match tag {
        "DropDownToggleButton" => Some((frame, "Button", Some("DropDownToggleButton"))),
        "EventButton" => Some((frame, "Button", Some("EventButton"))),
        _ => crate::xml::widget_type_for_tag(tag)
            .map(|(widget_type, intrinsic)| (frame, widget_type, intrinsic)),
    }
}

fn resolve_inherited_string(
    frame: &crate::xml::FrameXml,
    project: impl Fn(&crate::xml::FrameXml) -> Option<&String>,
) -> Option<String> {
    if let Some(value) = project(frame) {
        return Some(value.clone());
    }
    let inherits = frame.inherits.as_deref()?;
    crate::xml::get_template_chain(inherits)
        .iter()
        .find_map(|entry| project(&entry.frame).cloned())
}
