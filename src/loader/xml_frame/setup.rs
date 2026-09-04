//! Frame setup: CreateFrame execution, XML property application, error recovery.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use crate::loader::LoadTiming;
use crate::loader::error::LoadError;
use crate::lua_api::LoaderEnv;
use crate::lua_api::frame::methods::forbidden_aspects;
use crate::lua_api::methods::{create_string, frame_ref, table_get, table_set};
use rilua::Val;

pub(super) struct SetupFrame<'a> {
    pub(super) widget_type: &'a str,
    pub(super) lua_code: &'a str,
    pub(super) name: &'a str,
    pub(super) explicit_parent: bool,
    pub(super) initial_hidden: bool,
    pub(super) frame: &'a crate::xml::FrameXml,
    pub(super) inherits: &'a str,
    pub(super) parent: &'a str,
    pub(super) intrinsic_base: Option<&'a str>,
}

#[derive(Default)]
struct FastCreateFrameProfile {
    fast_hits: u64,
    slow_fallbacks: u64,
    miss_reasons: BTreeMap<&'static str, u64>,
    miss_bodies: BTreeMap<String, u64>,
}

/// Execute CreateFrame Lua, apply XML properties, and record setup timing.
pub(super) fn setup_frame(
    env: &LoaderEnv<'_>,
    timing: &mut LoadTiming,
    setup: SetupFrame<'_>,
) -> Result<(), LoadError> {
    let setup_start = Instant::now();
    let exec_start = Instant::now();
    match exec_create_frame_code(env, &setup) {
        Ok(()) => {}
        Err(_)
            if recover_frame_after_partial_create_error(
                env,
                setup.name,
                setup.frame,
                setup.inherits,
                setup.parent,
            )? => {}
        Err(error) => return Err(error),
    }
    timing.frame_exec_lua_time += exec_start.elapsed();
    let props_start = Instant::now();
    let frame_id = created_frame_id(env, setup.name)?;
    ensure_parent_refs_registered(env, &setup, frame_id)?;
    apply_xml_properties_direct(env, frame_id, setup.frame, setup.inherits, setup.parent);
    apply_intrinsic_property(env, setup.intrinsic_base, frame_id);
    timing.frame_apply_props_time += props_start.elapsed();
    timing.xml_frame_setup_time += setup_start.elapsed();
    timing.frame_count += 1;
    Ok(())
}

pub(super) fn created_frame_id(env: &LoaderEnv<'_>, name: &str) -> Result<u64, LoadError> {
    env.state()
        .borrow()
        .widgets
        .get_id_by_name(name)
        .ok_or_else(|| LoadError::Lua(format!("Failed to locate created frame {name}")))
}

fn ensure_parent_refs_registered(
    env: &LoaderEnv<'_>,
    setup: &SetupFrame<'_>,
    frame_id: u64,
) -> Result<(), LoadError> {
    let parent_key = setup.frame.parent_key.as_deref();
    let parent_array = setup.frame.parent_array.as_deref();
    if parent_key.is_none() && parent_array.is_none() {
        return Ok(());
    }

    env.with_state(|state| {
        let parent_id = crate::lua_api::methods::borrow_state(state)?
            .widgets
            .get_id_by_name(setup.parent)
            .ok_or_else(|| crate::Error::Other(format!("missing parent '{}'", setup.parent)))?;
        if let Some(parent_key) = parent_key {
            crate::lua_api::globals::template::assign_parent_key(
                state, parent_id, parent_key, frame_id,
            )
            .map_err(|error| crate::Error::Other(error.to_string()))?;
        }
        if let Some(parent_array) = parent_array
            && !parent_array_contains_child(state, parent_id, parent_array, frame_id)?
        {
            crate::lua_api::globals::create_frame::append_parent_array_entry(
                state,
                parent_id,
                parent_array,
                frame_id,
            );
        }
        Ok::<(), crate::Error>(())
    })
    .map_err(|error| LoadError::Lua(error.to_string()))
}

fn parent_array_contains_child(
    state: &mut rilua::vm::state::LuaState,
    parent_id: u64,
    key: &str,
    child_id: u64,
) -> Result<bool, crate::Error> {
    let parent =
        frame_ref(state, parent_id).map_err(|error| crate::Error::Other(error.to_string()))?;
    let Val::Table(array_ref) = table_get(state, parent, key) else {
        return Ok(false);
    };
    Ok(state
        .gc
        .tables
        .get(array_ref)
        .map(|table| {
            table.array_slice().iter().copied().any(|entry| {
                crate::lua_api::methods::extract_frame_id(state, entry) == Some(child_id)
            })
        })
        .unwrap_or(false))
}

fn recover_frame_after_partial_create_error(
    env: &LoaderEnv<'_>,
    name: &str,
    frame: &crate::xml::FrameXml,
    inherits: &str,
    parent: &str,
) -> Result<bool, LoadError> {
    let frame_exists = env.state().borrow().widgets.get_id_by_name(name).is_some();
    if !frame_exists {
        return Ok(false);
    }
    let parent_key = resolve_inherited_parent_key(frame, inherits);
    let parent_array = resolve_inherited_parent_array(frame, inherits);
    if parent_key.is_none() && parent_array.is_none() {
        return Ok(false);
    }
    let repair = build_parent_link_repair_script(
        parent,
        name,
        parent_key.as_deref(),
        parent_array.as_deref(),
    );
    env.exec(&repair).map_err(|repair_error| {
        LoadError::Lua(format!(
            "Recovered frame {name} exists but failed to repair parent links after partial CreateFrame error: {repair_error}"
        ))
    })?;
    Ok(true)
}

/// Build a Lua snippet that re-links `child` into `parent[parent_key]` and/or
/// `parent[parent_array]` after a partial CreateFrame error left the frame disconnected.
fn build_parent_link_repair_script(
    parent: &str,
    name: &str,
    parent_key: Option<&str>,
    parent_array: Option<&str>,
) -> String {
    let parent_ref = crate::loader::helpers::lua_global_ref(parent);
    let child_ref = crate::loader::helpers::lua_global_ref(name);
    let mut repair = format!("local parent = {parent_ref}\nlocal child = {child_ref}\n");
    repair.push_str("if parent and child then\n");
    if let Some(parent_key) = parent_key {
        repair.push_str(&format!("  parent[{parent_key:?}] = child\n"));
    }
    if let Some(parent_array) = parent_array {
        repair.push_str(&format!(
            "  parent[{parent_array:?}] = parent[{parent_array:?}] or {{}}\n"
        ));
        repair.push_str("  local already_present = false\n");
        repair.push_str(&format!(
            "  for _, existing in ipairs(parent[{parent_array:?}]) do\n"
        ));
        repair.push_str("    if existing == child then\n");
        repair.push_str("      already_present = true\n");
        repair.push_str("      break\n");
        repair.push_str("    end\n");
        repair.push_str("  end\n");
        repair.push_str("  if not already_present then\n");
        repair.push_str(&format!(
            "    table.insert(parent[{parent_array:?}], child)\n"
        ));
        repair.push_str("  end\n");
    }
    repair.push_str("end\n");
    repair
}

fn resolve_inherited_parent_key(frame: &crate::xml::FrameXml, inherits: &str) -> Option<String> {
    frame.parent_key.clone().or_else(|| {
        if inherits.is_empty() {
            return None;
        }
        crate::xml::get_template_chain(inherits)
            .iter()
            .rev()
            .find_map(|entry| entry.frame.parent_key.clone())
    })
}

fn resolve_inherited_parent_array(frame: &crate::xml::FrameXml, inherits: &str) -> Option<String> {
    frame.parent_array.clone().or_else(|| {
        if inherits.is_empty() {
            return None;
        }
        crate::xml::get_template_chain(inherits)
            .iter()
            .rev()
            .find_map(|entry| entry.frame.parent_array.clone())
    })
}

/// Execute CreateFrame Lua with OnLoad suppression (depth-counted for recursion).
fn exec_create_frame_code(env: &LoaderEnv<'_>, setup: &SetupFrame<'_>) -> Result<(), LoadError> {
    {
        let mut state = env.state().borrow_mut();
        state.create_frame_initial_hidden = Some(setup.initial_hidden);
        state.suppress_runtime_on_load_depth += 1;
    }
    let exec_result = if can_fast_create_frame(setup) {
        fast_create_frame(env, setup)
    } else {
        env.exec(setup.lua_code)
            .map_err(|e| LoadError::Lua(format!("Failed to create frame {}: {}", setup.name, e)))
    };
    {
        let mut state = env.state().borrow_mut();
        state.create_frame_initial_hidden = None;
        state.suppress_runtime_on_load_depth =
            state.suppress_runtime_on_load_depth.saturating_sub(1);
    }
    exec_result
}

fn can_fast_create_frame(setup: &SetupFrame<'_>) -> bool {
    let scripts_need_slow_path = setup.frame.scripts().is_some_and(|scripts| {
        !crate::lua_api::globals::create_frame::scripts_support_fast_install(scripts)
    });
    let miss_reasons = fast_create_miss_reasons(setup, scripts_need_slow_path);

    if !fast_create_frame_profiling_enabled() {
        return miss_reasons.is_empty();
    }

    let miss_body = fast_create_script_miss_body(setup, scripts_need_slow_path);
    record_fast_create_frame_profile(&miss_reasons, miss_body.as_deref());
    miss_reasons.is_empty()
}

fn fast_create_miss_reasons(
    setup: &SetupFrame<'_>,
    scripts_need_slow_path: bool,
) -> Vec<&'static str> {
    let mut miss_reasons = Vec::new();
    append_fast_create_miss_reasons(setup, scripts_need_slow_path, &mut miss_reasons);
    miss_reasons
}

fn append_fast_create_miss_reasons(
    setup: &SetupFrame<'_>,
    scripts_need_slow_path: bool,
    miss_reasons: &mut Vec<&'static str>,
) {
    append_fast_create_parent_misses(setup, miss_reasons);
    append_fast_create_xml_misses(setup, miss_reasons);
    if scripts_need_slow_path {
        miss_reasons.push("scripts");
    }
}

fn append_fast_create_parent_misses(setup: &SetupFrame<'_>, miss_reasons: &mut Vec<&'static str>) {
    if !setup.explicit_parent {
        miss_reasons.push("no_explicit_parent");
    }
    if setup.name == setup.parent {
        miss_reasons.push("root_frame_reuse");
    }
}

fn append_fast_create_xml_misses(setup: &SetupFrame<'_>, miss_reasons: &mut Vec<&'static str>) {
    if setup.frame.xml_attributes().is_some() {
        miss_reasons.push("xml_attributes");
    }
    if setup.frame.mixins().is_some() {
        miss_reasons.push("mixins_block");
    }
    if setup.frame.key_values().is_some() {
        miss_reasons.push("key_values");
    }
}

fn fast_create_script_miss_body(
    setup: &SetupFrame<'_>,
    scripts_need_slow_path: bool,
) -> Option<String> {
    if !scripts_need_slow_path {
        return None;
    }
    setup
        .frame
        .scripts()
        .and_then(crate::lua_api::globals::create_frame::first_fast_install_miss)
}

fn fast_create_frame(env: &LoaderEnv<'_>, setup: &SetupFrame<'_>) -> Result<(), LoadError> {
    env.with_state(|state| {
        let widget_type = fast_create_widget_type(setup.widget_type)?;
        let parent_id = fast_create_parent_id(state, setup.parent)?;
        let frame_id = create_fast_frame_instance(state, setup, widget_type, parent_id)?;

        apply_fast_frame_templates(state, setup, frame_id)?;
        attach_fast_frame_to_parent(state, setup, parent_id, frame_id)?;
        apply_fast_frame_mixins_and_scripts(state, setup, frame_id)?;

        Ok::<(), crate::Error>(())
    })
    .map_err(|error| LoadError::Lua(format!("Failed to create frame {}: {}", setup.name, error)))
}

fn fast_create_widget_type(widget_type: &str) -> Result<crate::widget::WidgetType, crate::Error> {
    crate::widget::WidgetType::from_str(widget_type)
        .ok_or_else(|| crate::Error::Other(format!("unknown widget type '{}'", widget_type)))
}

fn fast_create_parent_id(
    state: &mut rilua::vm::state::LuaState,
    parent: &str,
) -> Result<u64, crate::Error> {
    crate::lua_api::methods::borrow_state(state)?
        .widgets
        .get_id_by_name(parent)
        .ok_or_else(|| crate::Error::Other(format!("missing parent '{}'", parent)))
}

fn create_fast_frame_instance(
    state: &mut rilua::vm::state::LuaState,
    setup: &SetupFrame<'_>,
    widget_type: crate::widget::WidgetType,
    parent_id: u64,
) -> Result<u64, crate::Error> {
    let frame_id = crate::lua_api::globals::create_frame::create_frame_instance(
        state,
        widget_type,
        setup.widget_type,
        Some(setup.name.to_string()),
        Some(parent_id),
        true,
        setup.frame.xml_id,
    )?;
    Ok(frame_id)
}

fn apply_fast_frame_templates(
    state: &mut rilua::vm::state::LuaState,
    setup: &SetupFrame<'_>,
    frame_id: u64,
) -> Result<(), crate::Error> {
    crate::lua_api::globals::create_frame::apply_runtime_template_chain_with_frame_overrides(
        state,
        frame_id,
        (!setup.inherits.is_empty()).then_some(setup.inherits),
        false,
        setup.frame,
    )
    .map_err(|error| crate::Error::Other(error.to_string()))
}

fn attach_fast_frame_to_parent(
    state: &mut rilua::vm::state::LuaState,
    setup: &SetupFrame<'_>,
    parent_id: u64,
    frame_id: u64,
) -> Result<(), crate::Error> {
    if let Some(parent_key) = setup.frame.parent_key.as_deref() {
        crate::lua_api::globals::template::assign_parent_key(
            state, parent_id, parent_key, frame_id,
        )
        .map_err(|error| crate::Error::Other(error.to_string()))?;
    }

    if let Some(parent_array) = setup.frame.parent_array.as_deref() {
        crate::lua_api::globals::create_frame::append_parent_array_entry(
            state,
            parent_id,
            parent_array,
            frame_id,
        );
    }

    Ok(())
}

fn apply_fast_frame_mixins_and_scripts(
    state: &mut rilua::vm::state::LuaState,
    setup: &SetupFrame<'_>,
    frame_id: u64,
) -> Result<(), crate::Error> {
    crate::lua_api::globals::create_frame::apply_frame_mixins(
        state,
        frame_id,
        setup.frame.combined_mixin().as_deref(),
    )
    .map_err(|error| crate::Error::Other(error.to_string()))?;
    if let Some(mixins) = setup.frame.mixins() {
        for mixin in &mixins.entries {
            crate::lua_api::globals::create_frame::apply_frame_mixin(
                state,
                frame_id,
                &mixin.key,
                mixin.source.as_deref(),
            )
            .map_err(|error| crate::Error::Other(error.to_string()))?;
        }
    }

    if let Some(scripts) = setup.frame.scripts() {
        crate::lua_api::globals::create_frame::apply_template_scripts(state, frame_id, scripts)
            .map_err(|error| crate::Error::Other(error.to_string()))?;
    }

    Ok(())
}

/// Set declarative frame properties directly in Rust after the Lua CreateFrame chunk.
fn apply_xml_properties_direct(
    env: &LoaderEnv<'_>,
    frame_id: u64,
    frame: &crate::xml::FrameXml,
    inherits: &str,
    parent: &str,
) {
    use crate::lua_api::globals::template::direct;
    let state = env.state();
    direct::apply_xml_size(state, frame_id, frame, inherits);
    direct::apply_xml_set_all_points(state, frame_id, frame, inherits);
    direct::apply_xml_anchors(state, frame_id, frame, inherits, parent);
    direct::apply_xml_frame_strata(state, frame_id, frame, inherits);
    direct::apply_xml_frame_level(state, frame_id, frame, inherits);
    direct::apply_xml_hidden(state, frame_id, frame, inherits);
    direct::apply_xml_toplevel(state, frame_id, frame, inherits);
    direct::apply_xml_alpha(state, frame_id, frame, inherits);
    direct::apply_xml_scale(state, frame_id, frame, inherits);
    direct::apply_xml_enable_mouse(state, frame_id, frame, inherits);
    direct::apply_xml_enable_keyboard(state, frame_id, frame, inherits);
    direct::apply_xml_propagate_mouse_input(state, frame_id, frame, inherits);
    direct::apply_xml_clips_children(state, frame_id, frame, inherits);
    direct::apply_xml_hit_rect_insets(state, frame_id, frame);
    direct::apply_xml_text_insets(state, frame_id, frame);
    direct::apply_xml_clamped_to_screen(state, frame_id, frame, inherits);
    direct::apply_xml_protected(state, frame_id, frame, inherits);
    direct::apply_xml_id(state, frame_id, frame);
    direct::apply_xml_letters(state, frame_id, frame, inherits);
    direct::apply_xml_slider_orientation(state, frame_id, frame, inherits);
    apply_xml_on_update_mode(env, frame_id, frame);
    apply_xml_forbidden_aspects(env, frame_id, frame);
}

fn apply_xml_on_update_mode(env: &LoaderEnv<'_>, frame_id: u64, frame: &crate::xml::FrameXml) {
    let Some(mode) = frame.on_update_mode.as_deref() else {
        return;
    };
    let normalized = match mode.to_ascii_lowercase().as_str() {
        "disabled" => "Disabled",
        "runwhenvisibleonce" => "RunWhenVisibleOnce",
        "runonce" => "RunOnce",
        "runalways" => "RunAlways",
        _ => "RunWhenVisible",
    };
    let _ = env.with_state(|state| {
        let frame = frame_ref(state, frame_id)?;
        let mode = create_string(state, normalized);
        table_set(state, frame, "__onUpdateMode", mode);
        Ok::<(), crate::Error>(())
    });
}

fn apply_xml_forbidden_aspects(env: &LoaderEnv<'_>, frame_id: u64, frame: &crate::xml::FrameXml) {
    let Some(forbidden_aspects) = frame.forbidden_aspects() else {
        return;
    };
    let mut mask = 0_u64;
    let mut parent_mask = 0_u64;
    let mut layout_mask = 0_u64;
    for aspect in &forbidden_aspects.aspects {
        let aspect_mask = forbidden_aspect_mask(&aspect.aspect);
        mask |= aspect_mask;
        let inheritance = forbidden_aspect_inheritance_mask(aspect.inheritance.as_deref());
        if inheritance & forbidden_aspects::INHERITANCE_PARENT != 0 {
            parent_mask |= aspect_mask;
        }
        if inheritance & forbidden_aspects::INHERITANCE_LAYOUT != 0 {
            layout_mask |= aspect_mask;
        }
    }
    if mask == 0 {
        return;
    }
    let _ = env.with_state(|state| {
        forbidden_aspects::set_forbidden_aspects(state, frame_id, mask);
        forbidden_aspects::set_inheritable_forbidden_aspects(
            state,
            frame_id,
            parent_mask,
            layout_mask,
        );
        Ok::<(), crate::Error>(())
    });
}

fn forbidden_aspect_inheritance_mask(inheritance: Option<&str>) -> u64 {
    let Some(inheritance) = inheritance else {
        return forbidden_aspects::INHERITANCE_PARENT | forbidden_aspects::INHERITANCE_LAYOUT;
    };
    inheritance
        .split([',', ' ', '|'])
        .fold(0_u64, |mask, value| match value {
            "Parent" => mask | forbidden_aspects::INHERITANCE_PARENT,
            "Layout" => mask | forbidden_aspects::INHERITANCE_LAYOUT,
            _ => mask,
        })
}

fn forbidden_aspect_mask(aspect: &str) -> u64 {
    match aspect {
        "UntrustedScriptExecution" => 1,
        "UntrustedLayoutScriptExecution" => 2,
        "EventRegistrations" => 4,
        "AlwaysPropagateInput" => 8,
        "ScriptedInput" => 16,
        "QueryFocus" => 32,
        _ => 0,
    }
}

/// Set the `intrinsic` property on intrinsic frames (e.g. frame.intrinsic = "DropdownButton").
fn apply_intrinsic_property(env: &LoaderEnv<'_>, intrinsic_base: Option<&str>, frame_id: u64) {
    if let Some(base) = intrinsic_base {
        let _ = env.with_state(|state| {
            crate::lua_api::globals::template::set_intrinsic(state, frame_id, base);
            Ok::<(), crate::Error>(())
        });
    }
}

fn fast_create_frame_profiling_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("WOW_SIM_PROFILE_XML_FAST_PATH").is_ok())
}

fn fast_create_frame_profile() -> &'static Mutex<FastCreateFrameProfile> {
    static PROFILE: OnceLock<Mutex<FastCreateFrameProfile>> = OnceLock::new();
    PROFILE.get_or_init(|| Mutex::new(FastCreateFrameProfile::default()))
}

fn record_fast_create_frame_profile(miss_reasons: &[&'static str], miss_body: Option<&str>) {
    if !fast_create_frame_profiling_enabled() {
        return;
    }
    let Ok(mut profile) = fast_create_frame_profile().lock() else {
        return;
    };
    if miss_reasons.is_empty() {
        profile.fast_hits += 1;
        return;
    }
    profile.slow_fallbacks += 1;
    for reason in miss_reasons {
        *profile.miss_reasons.entry(reason).or_default() += 1;
    }
    if let Some(miss_body) = miss_body {
        *profile
            .miss_bodies
            .entry(miss_body.to_string())
            .or_default() += 1;
    }
}

pub(super) fn fast_create_frame_profile_report() -> Option<String> {
    if !fast_create_frame_profiling_enabled() {
        return None;
    }
    let Ok(profile) = fast_create_frame_profile().lock() else {
        return None;
    };
    let total = profile.fast_hits + profile.slow_fallbacks;
    if total == 0 {
        return Some("xml fast path: no frames recorded".to_string());
    }

    let mut reasons = profile
        .miss_reasons
        .iter()
        .map(|(reason, count)| (*reason, *count))
        .collect::<Vec<_>>();
    reasons.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

    let top = reasons
        .into_iter()
        .take(8)
        .map(|(reason, count)| format!("{reason}={count}"))
        .collect::<Vec<_>>()
        .join(", ");

    Some(format!(
        "xml fast path: hits={} slow={} total={} misses: {}",
        profile.fast_hits,
        profile.slow_fallbacks,
        total,
        if top.is_empty() {
            "none".to_string()
        } else {
            top
        }
    ))
}

pub(super) fn fast_create_frame_profile_body_report() -> Option<String> {
    if !fast_create_frame_profiling_enabled() {
        return None;
    }
    let Ok(profile) = fast_create_frame_profile().lock() else {
        return None;
    };
    if profile.miss_bodies.is_empty() {
        return None;
    }
    let mut bodies = profile
        .miss_bodies
        .iter()
        .map(|(body, count)| (body.as_str(), *count))
        .collect::<Vec<_>>();
    bodies.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    let top = bodies
        .into_iter()
        .take(8)
        .map(|(body, count)| format!("{count}x {body}"))
        .collect::<Vec<_>>()
        .join(" | ");
    Some(format!("xml fast path script misses: {top}"))
}
