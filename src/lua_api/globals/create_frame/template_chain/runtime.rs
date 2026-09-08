use crate::lua_api::LoaderEnv;
use crate::lua_api::methods::{
    borrow_lua, borrow_state, borrow_state_mut, frame_ref, state_handle,
};
use crate::widget::WidgetType;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::cell::RefCell;
use std::rc::Rc;

type RuntimeButtonTextureSlot<'a> = (&'static str, Option<&'a crate::xml::TextureXml>);

pub(super) fn create_template_child_frames(
    state: &mut LuaState,
    state_rc: &Rc<RefCell<crate::lua_api::SimState>>,
    parent_id: u64,
    parent_name: &str,
    subst_parent: &str,
    frame: &crate::xml::FrameXml,
    deferred_on_loads: &mut Vec<u64>,
) -> LuaResult<()> {
    create_direct_child_frames(
        state,
        state_rc,
        parent_id,
        parent_name,
        subst_parent,
        frame,
        deferred_on_loads,
    )?;
    create_scroll_child_frames(
        state,
        state_rc,
        parent_id,
        parent_name,
        subst_parent,
        frame,
        deferred_on_loads,
    )?;
    Ok(())
}

fn create_direct_child_frames(
    state: &mut LuaState,
    state_rc: &Rc<RefCell<crate::lua_api::SimState>>,
    parent_id: u64,
    _parent_name: &str,
    subst_parent: &str,
    frame: &crate::xml::FrameXml,
    deferred_on_loads: &mut Vec<u64>,
) -> LuaResult<()> {
    frame.try_for_each_frame_element(|child_frame, child_tag| {
        create_template_child_frame(
            state,
            state_rc,
            parent_id,
            subst_parent,
            child_frame,
            child_tag,
            deferred_on_loads,
        )?;
        Ok::<(), rilua::LuaError>(())
    })
}

fn create_scroll_child_frames(
    state: &mut LuaState,
    state_rc: &Rc<RefCell<crate::lua_api::SimState>>,
    parent_id: u64,
    _parent_name: &str,
    subst_parent: &str,
    frame: &crate::xml::FrameXml,
    deferred_on_loads: &mut Vec<u64>,
) -> LuaResult<()> {
    let Some(scroll_child) = frame.scroll_child() else {
        return Ok(());
    };

    let mut registered_scroll_child = false;
    for child in &scroll_child.children {
        let Some((child_frame, child_tag)) = child.as_frame_data() else {
            continue;
        };
        let child_id = create_template_child_frame(
            state,
            state_rc,
            parent_id,
            subst_parent,
            child_frame,
            child_tag,
            deferred_on_loads,
        )?;
        if !registered_scroll_child && let Some(child_id) = child_id {
            let mut sim = borrow_state_mut(state)?;
            crate::lua_api::frame::methods::widget_scroll::assign_scroll_child(
                &mut sim, parent_id, child_id, false,
            );
            registered_scroll_child = true;
        }
    }

    Ok(())
}

fn create_template_child_frame(
    state: &mut LuaState,
    state_rc: &Rc<RefCell<crate::lua_api::SimState>>,
    parent_id: u64,
    subst_parent: &str,
    child_frame: &crate::xml::FrameXml,
    child_tag: &'static str,
    deferred_on_loads: &mut Vec<u64>,
) -> LuaResult<Option<u64>> {
    let Some((frame, widget_type_name, intrinsic)) =
        super::template_child_type(child_frame, child_tag)
    else {
        return Ok(None);
    };
    let inherited_chain = super::build_child_inherits(intrinsic, frame.inherits.as_deref());
    let child_name = super::template_child_name(frame.name.as_deref(), subst_parent);
    let child_id = instantiate_template_child(
        state,
        parent_id,
        frame,
        widget_type_name,
        inherited_chain.as_deref(),
        &child_name,
    )?;
    assign_child_parent_refs(state, parent_id, child_id, frame);
    apply_child_template_properties(state, child_id, frame, intrinsic)?;
    finalize_runtime_template_child(
        state,
        state_rc,
        parent_id,
        child_id,
        &child_name,
        subst_parent,
        frame,
        inherited_chain.as_deref(),
        deferred_on_loads,
    )?;
    Ok(Some(child_id))
}

fn finalize_runtime_template_child(
    state: &mut LuaState,
    state_rc: &Rc<RefCell<crate::lua_api::SimState>>,
    parent_id: u64,
    child_id: u64,
    child_name: &str,
    subst_parent: &str,
    frame: &crate::xml::FrameXml,
    _inherited_chain: Option<&str>,
    deferred_on_loads: &mut Vec<u64>,
) -> LuaResult<()> {
    let child_subst = child_runtime_subst(frame, child_name, subst_parent);
    apply_runtime_child_direct_properties(state_rc, child_id, frame, child_subst);
    ensure_runtime_button_texture_slots(state, child_id, frame)?;
    create_template_child_frames(
        state,
        state_rc,
        child_id,
        child_name,
        child_subst,
        frame,
        deferred_on_loads,
    )?;
    // `apply_runtime_template_chain()` already loaded inherited layers/extras
    // onto the runtime child. The finalize pass should only apply the
    // child frame's own direct loader-backed content or it duplicates named
    // inherited regions (for example `$parentBackground`).
    apply_runtime_template_loader_effects(state, child_id, child_name, child_subst, frame, None)?;
    repair_runtime_direct_layer_parent_keys(state, child_id, child_subst, frame)?;
    crate::lua_api::globals::template::repair_direct_child_parent_keys(state, child_id)?;
    crate::lua_api::globals::template::repair_transparent_wrapper_parent_key_aliases(
        state, child_id,
    )?;
    super::resolve_runtime_template_named_anchors(state, child_id)?;
    deferred_on_loads.push(child_id);
    publish_anonymous_wrapper_layer_keys_to_parent(state, parent_id, frame, child_subst)?;
    Ok(())
}

fn publish_anonymous_wrapper_layer_keys_to_parent(
    state: &mut LuaState,
    parent_id: u64,
    frame: &crate::xml::FrameXml,
    name_parent: &str,
) -> LuaResult<()> {
    if frame.name.is_some() || frame.parent_key.is_some() {
        return Ok(());
    }

    let repairs =
        collect_runtime_direct_layer_parent_key_repairs(state, parent_id, name_parent, frame)?;
    for (parent_key, child_id) in repairs {
        crate::lua_api::globals::template::assign_parent_key(
            state,
            parent_id,
            &parent_key,
            child_id,
        )?;
    }
    Ok(())
}

fn repair_runtime_direct_layer_parent_keys(
    state: &mut LuaState,
    frame_id: u64,
    name_parent: &str,
    frame: &crate::xml::FrameXml,
) -> LuaResult<()> {
    let repairs =
        collect_runtime_direct_layer_parent_key_repairs(state, frame_id, name_parent, frame)?;
    for (parent_key, child_id) in repairs {
        crate::lua_api::globals::template::assign_parent_key(
            state,
            frame_id,
            &parent_key,
            child_id,
        )?;
    }
    Ok(())
}

pub(super) fn repair_runtime_template_chain_layer_parent_keys(
    state: &mut LuaState,
    frame_id: u64,
    name_parent: &str,
    chain: &[std::sync::Arc<crate::xml::TemplateEntry>],
) -> LuaResult<()> {
    for entry in chain {
        repair_runtime_direct_layer_parent_keys(state, frame_id, name_parent, &entry.frame)?;
    }
    Ok(())
}

fn collect_runtime_direct_layer_parent_key_repairs(
    state: &LuaState,
    frame_id: u64,
    name_parent: &str,
    frame: &crate::xml::FrameXml,
) -> LuaResult<Vec<(String, u64)>> {
    let desired = direct_layer_parent_key_children(frame, name_parent);
    if desired.is_empty() {
        return Ok(Vec::new());
    }

    let sim = borrow_state(state)?;
    let Some(parent) = sim.widgets.get(frame_id) else {
        return Ok(Vec::new());
    };

    let repairs = desired
        .into_iter()
        .filter(|(parent_key, _)| parent.children_keys.get(parent_key).is_none())
        .filter_map(|(parent_key, child_name)| {
            let child_id = sim
                .widgets
                .get_id_by_name(&child_name)
                .or_else(|| sole_direct_texture_child(&sim, frame_id));
            child_id.map(|child_id| (parent_key, child_id))
        })
        .collect();
    Ok(repairs)
}

fn direct_layer_parent_key_children(
    frame: &crate::xml::FrameXml,
    name_parent: &str,
) -> Vec<(String, String)> {
    let mut children = Vec::new();
    for layers in frame.layers() {
        for layer in &layers.layers {
            for element in &layer.elements {
                if let Some((parent_key, child_name)) =
                    direct_layer_parent_key_child(element, name_parent)
                {
                    children.push((parent_key, child_name));
                }
            }
        }
    }
    children
}

fn direct_layer_parent_key_child(
    element: &crate::xml::LayerElement,
    name_parent: &str,
) -> Option<(String, String)> {
    match element {
        crate::xml::LayerElement::Texture(texture)
        | crate::xml::LayerElement::Line(texture)
        | crate::xml::LayerElement::MaskTexture(texture) => {
            let resolved = crate::xml::resolve_texture_inheritance(texture);
            let parent_key = resolved.parent_key.clone()?;
            let child_name = crate::loader::helpers::resolve_child_name(
                resolved.name.as_deref(),
                name_parent,
                "__tex_",
            );
            Some((parent_key, child_name))
        }
        crate::xml::LayerElement::FontString(font_string) => {
            let parent_key = font_string.parent_key.clone()?;
            let child_name = crate::loader::helpers::resolve_child_name(
                font_string.name.as_deref(),
                name_parent,
                "__fs_",
            );
            Some((parent_key, child_name))
        }
    }
}

fn sole_direct_texture_child(sim: &crate::lua_api::SimState, frame_id: u64) -> Option<u64> {
    let parent = sim.widgets.get(frame_id)?;
    let mut texture_children = parent.children.iter().copied().filter(|child_id| {
        sim.widgets
            .get(*child_id)
            .is_some_and(|child| child.widget_type == WidgetType::Texture)
    });
    let first = texture_children.next()?;
    texture_children.next().is_none().then_some(first)
}

fn child_runtime_subst<'a>(
    frame: &crate::xml::FrameXml,
    child_name: &'a str,
    subst_parent: &'a str,
) -> &'a str {
    if frame.name.is_some() {
        child_name
    } else {
        subst_parent
    }
}

fn instantiate_template_child(
    state: &mut LuaState,
    parent_id: u64,
    frame: &crate::xml::FrameXml,
    widget_type_name: &str,
    inherited_chain: Option<&str>,
    child_name: &str,
) -> LuaResult<u64> {
    let previous_hidden = {
        let mut sim = borrow_state_mut(state)?;
        let previous_hidden = sim.create_frame_initial_hidden;
        sim.create_frame_initial_hidden =
            Some(super::resolve_inherited_hidden(frame, inherited_chain));
        previous_hidden
    };

    let result = crate::lua_api::globals::create_frame::create_frame_instance(
        state,
        WidgetType::from_str(widget_type_name).ok_or_else(|| {
            rilua::runtime_error(format!("unknown widget type '{widget_type_name}'"))
        })?,
        widget_type_name,
        Some(child_name.to_owned()),
        Some(parent_id),
        true,
        frame.xml_id,
    );

    borrow_state_mut(state)?.create_frame_initial_hidden = previous_hidden;
    result
}

fn assign_child_parent_refs(
    state: &mut LuaState,
    parent_id: u64,
    child_id: u64,
    frame: &crate::xml::FrameXml,
) {
    if let Some(parent_key) = super::resolve_inherited_string(frame, |t| t.parent_key.as_ref()) {
        let _ = crate::lua_api::globals::template::assign_parent_key(
            state,
            parent_id,
            &parent_key,
            child_id,
        );
    }
    // Template-inherited parentArray is applied by the runtime template chain.
    // Register only the child frame's direct parentArray here to avoid duplicates.
    if let Some(parent_array) = frame.parent_array.as_ref() {
        crate::lua_api::globals::create_frame::append_parent_array_entry(
            state,
            parent_id,
            parent_array,
            child_id,
        );
    }
}

fn apply_child_template_properties(
    state: &mut LuaState,
    child_id: u64,
    frame: &crate::xml::FrameXml,
    intrinsic: Option<&str>,
) -> LuaResult<()> {
    let inherited_chain = super::build_child_inherits(intrinsic, frame.inherits.as_deref());
    if let Some(chain) = inherited_chain.as_deref() {
        super::apply_runtime_template_chain(state, child_id, Some(chain), false)?;
    }
    if let Some(intrinsic) = intrinsic {
        crate::lua_api::globals::template::set_intrinsic(state, child_id, intrinsic);
    }
    crate::lua_api::frame::methods::forbidden_aspects::apply_xml_forbidden_aspects(
        state, child_id, frame,
    )?;
    super::apply_template_partition_marker(state, child_id, frame);
    crate::lua_api::globals::create_frame::apply_frame_mixins(
        state,
        child_id,
        frame.combined_mixin().as_deref(),
    )?;
    super::apply_block_mixins(state, child_id, frame.mixins())?;
    super::apply_template_key_values(state, child_id, frame.all_key_values());
    if let Some(scripts) = frame.scripts() {
        super::apply_template_scripts(state, child_id, scripts)?;
    }
    Ok(())
}

pub(super) fn apply_runtime_template_loader_effects(
    state: &mut LuaState,
    frame_id: u64,
    frame_name: &str,
    name_parent: &str,
    frame: &crate::xml::FrameXml,
    inherits: Option<&str>,
) -> LuaResult<()> {
    let loader_env = LoaderEnv::from_parts_active(borrow_lua(state)?, state_handle(state)?, state);
    let inherits = inherits.unwrap_or("");
    let mut timing = crate::loader::LoadTiming::default();
    super::runtime_loader_effects::apply_loader_chain_layers(
        &loader_env,
        inherits,
        frame_id,
        frame_name,
        name_parent,
        &mut timing,
    )?;
    super::runtime_loader_effects::apply_loader_frame_extras(
        &loader_env,
        frame,
        frame_id,
        frame_name,
        name_parent,
        inherits,
        &mut timing,
    )
}

pub(super) fn ensure_runtime_button_texture_slots(
    state: &mut LuaState,
    frame_id: u64,
    frame: &crate::xml::FrameXml,
) -> LuaResult<()> {
    if !is_runtime_button(state, frame_id)? {
        return Ok(());
    }

    let aliases = create_runtime_button_texture_slots(state, frame_id, frame)?;
    publish_runtime_button_texture_aliases(state, frame_id, aliases)
}

fn is_runtime_button(state: &LuaState, frame_id: u64) -> LuaResult<bool> {
    let sim = borrow_state(state)?;
    let is_button = sim
        .widgets
        .get(frame_id)
        .map(|widget| {
            matches!(
                widget.widget_type,
                WidgetType::Button | WidgetType::CheckButton
            )
        })
        .unwrap_or(false);
    Ok(is_button)
}

fn create_runtime_button_texture_slots(
    state: &mut LuaState,
    frame_id: u64,
    frame: &crate::xml::FrameXml,
) -> LuaResult<Vec<(String, u64)>> {
    let mut aliases = Vec::new();
    let mut sim = borrow_state_mut(state)?;
    for (key, texture) in runtime_button_texture_slots(frame) {
        if let Some(texture) = texture {
            let texture_id =
                crate::lua_api::frame::methods::methods_helpers::get_or_create_button_texture(
                    &mut sim, frame_id, key,
                );
            if let Some(parent_key) = texture.parent_key.as_deref()
                && parent_key != key
            {
                add_runtime_button_texture_alias(&mut sim, frame_id, parent_key, texture_id);
                aliases.push((parent_key.to_string(), texture_id));
            }
        }
    }
    Ok(aliases)
}

fn runtime_button_texture_slots(frame: &crate::xml::FrameXml) -> [RuntimeButtonTextureSlot<'_>; 6] {
    [
        ("NormalTexture", frame.normal_texture()),
        ("PushedTexture", frame.pushed_texture()),
        ("HighlightTexture", frame.highlight_texture()),
        ("DisabledTexture", frame.disabled_texture()),
        ("CheckedTexture", frame.checked_texture()),
        ("DisabledCheckedTexture", frame.disabled_checked_texture()),
    ]
}

fn add_runtime_button_texture_alias(
    sim: &mut crate::lua_api::SimState,
    frame_id: u64,
    parent_key: &str,
    texture_id: u64,
) {
    if let Some(button) = sim.widgets.get_mut_visual(frame_id) {
        button
            .children_keys
            .insert(parent_key.to_string(), texture_id);
    }
}

fn publish_runtime_button_texture_aliases(
    state: &mut LuaState,
    frame_id: u64,
    aliases: Vec<(String, u64)>,
) -> LuaResult<()> {
    for (parent_key, texture_id) in aliases {
        crate::lua_api::globals::template::assign_parent_key(
            state,
            frame_id,
            &parent_key,
            texture_id,
        )?;
    }
    Ok(())
}

pub(super) fn apply_runtime_template_direct_properties(
    state: &Rc<RefCell<crate::lua_api::SimState>>,
    frame_id: u64,
    inherits: &str,
    frame_name: &str,
) {
    let frame = crate::xml::FrameXml::default();
    apply_runtime_child_direct_properties_with_inherits(
        state, frame_id, &frame, inherits, frame_name,
    );
}

fn apply_runtime_child_direct_properties(
    state: &Rc<RefCell<crate::lua_api::SimState>>,
    frame_id: u64,
    frame: &crate::xml::FrameXml,
    frame_name: &str,
) {
    let inherits = frame.inherits.as_deref().unwrap_or("");
    apply_runtime_child_direct_properties_with_inherits(
        state, frame_id, frame, inherits, frame_name,
    );
}

fn apply_runtime_child_direct_properties_with_inherits(
    state: &Rc<RefCell<crate::lua_api::SimState>>,
    frame_id: u64,
    frame: &crate::xml::FrameXml,
    inherits: &str,
    frame_name: &str,
) {
    crate::lua_api::globals::template::direct::apply_xml_size(state, frame_id, frame, inherits);
    crate::lua_api::globals::template::direct::apply_xml_set_all_points(
        state, frame_id, frame, inherits,
    );
    crate::lua_api::globals::template::direct::apply_xml_anchors(
        state, frame_id, frame, inherits, frame_name,
    );
    crate::lua_api::globals::template::direct::apply_xml_hidden(state, frame_id, frame, inherits);
    crate::lua_api::globals::template::direct::apply_xml_alpha(state, frame_id, frame, inherits);
    crate::lua_api::globals::template::direct::apply_xml_scale(state, frame_id, frame, inherits);
    crate::lua_api::globals::template::direct::apply_xml_propagate_mouse_input(
        state, frame_id, frame, inherits,
    );
    crate::lua_api::globals::template::direct::apply_xml_clips_children(
        state, frame_id, frame, inherits,
    );
    crate::lua_api::globals::template::direct::apply_xml_frame_level(
        state, frame_id, frame, inherits,
    );
    crate::lua_api::globals::template::direct::apply_xml_frame_strata(
        state, frame_id, frame, inherits,
    );
    normalize_edit_mode_selection_layers(state, frame_id, inherits);
    crate::lua_api::globals::template::direct::apply_xml_protected(
        state, frame_id, frame, inherits,
    );
}

fn normalize_edit_mode_selection_layers(
    state: &Rc<RefCell<crate::lua_api::SimState>>,
    frame_id: u64,
    inherits: &str,
) {
    if !inherits
        .split(',')
        .map(str::trim)
        .any(|name| name.starts_with("EditModeSystemSelection"))
    {
        return;
    }

    let mut sim = state.borrow_mut();
    let Some((parent_strata, parent_level)) = sim
        .widgets
        .get(frame_id)
        .and_then(|frame| frame.parent_id)
        .and_then(|parent_id| sim.widgets.get(parent_id))
        .map(|parent| (parent.frame_strata, parent.frame_level))
    else {
        return;
    };

    if let Some(frame) = sim.widgets.get_mut_visual(frame_id) {
        frame.has_fixed_frame_strata = false;
        frame.frame_strata = parent_strata;
        frame.has_fixed_frame_level = false;
        frame.frame_level = parent_level + frame.frame_level_offset.unwrap_or(1);
    }
    crate::lua_api::frame::methods::methods_hierarchy::propagate_strata_level_pub(
        &mut sim.widgets,
        frame_id,
    );
}

pub(super) fn fire_frame_on_load(state: &mut LuaState, frame_id: u64) -> LuaResult<()> {
    let frame = frame_ref(state, frame_id)?;
    for handler in
        crate::lua_api::script_helpers::get_scripts_for_dispatch(state, frame_id, "OnLoad")
    {
        call_handler_with_frame(state, handler, frame)?;
    }
    crate::lua_api::frame::methods::core_state::size::mark_nearest_layout_parent_dirty(
        state, frame_id,
    );
    Ok(())
}

fn call_handler_with_frame(state: &mut LuaState, handler: Val, frame: Val) -> LuaResult<()> {
    let Val::Function(_) = handler else {
        return Ok(());
    };
    match crate::lua_api::script_helpers::call_void_function_state(state, handler, &[frame]) {
        Ok(_) => Ok(()),
        Err(err) => Err(rilua::runtime_error(err)),
    }
}
