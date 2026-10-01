//! FontString methods: GetFontString, SetFontString, CreateFontString.

use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, extract_frame_id, frame_id_from_stack, frame_ref,
    registry_table_or_create, sync_child_to_rilua, table_get, table_get_static, table_set,
    val_to_string,
};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::HashSet;

use super::shared::{bind_named_child_global, opt_string};

const BUTTON_TEXT_CHILD_KEYS: [&str; 3] = ["Text", "text", "ButtonText"];
const FONT_OBJECTS_REGISTRY_KEY: &str = "__font_objects";

/// GetFontString() -> fontstring
pub(super) fn get_font_string(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    if let Some(tid) = find_existing_text_child(state, id) {
        let val = frame_ref(state, tid)?;
        state.push(val);
        return Ok(1);
    }
    create_synthetic_text_child(state, id)
}

fn find_existing_text_child(state: &mut LuaState, id: u64) -> Option<u64> {
    let sim = borrow_state(state).ok()?;
    if let Some(frame) = sim.widgets.get(id) {
        for key in BUTTON_TEXT_CHILD_KEYS {
            if let Some(child_id) = frame.children_keys.get(key).copied() {
                return Some(child_id);
            }
        }
        for key in BUTTON_TEXT_CHILD_KEYS {
            if let Some(child_id) = find_child_by_parent_key(&sim, frame.children.as_slice(), key) {
                return Some(child_id);
            }
        }
    }
    let fallback_name = sim.widgets.get(id)?.name.as_ref()?.to_string() + "Text";
    let child_id = sim.widgets.get_id_by_name(&fallback_name)?;
    let child = sim.widgets.get(child_id)?;
    if child.parent_id == Some(id) && child.widget_type == crate::widget::WidgetType::FontString {
        Some(child_id)
    } else {
        None
    }
}

fn find_child_by_parent_key(
    sim: &crate::lua_api::state::SimState,
    children: &[u64],
    key: &str,
) -> Option<u64> {
    children.iter().copied().find(|child_id| {
        sim.widgets.get(*child_id).is_some_and(|child| {
            child.widget_type == crate::widget::WidgetType::FontString
                && child.parent_key.as_deref() == Some(key)
        })
    })
}

fn create_synthetic_text_child(state: &mut LuaState, id: u64) -> LuaResult<u32> {
    let fallback = {
        let sim = borrow_state(state)?;
        sim.widgets.get(id).map(|frame| {
            (
                matches!(
                    frame.widget_type,
                    crate::widget::WidgetType::Button | crate::widget::WidgetType::CheckButton
                ),
                frame.name.as_ref().map(|name| format!("{name}Text")),
                frame.text.clone(),
            )
        })
    };
    let Some((is_button, _child_name, text_value)) = fallback else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let has_normal_font_object = super::buttons::has_normal_font_object(state, id);
    if !is_button || (text_value.is_none() && !has_normal_font_object) {
        state.push(Val::Nil);
        return Ok(1);
    }

    let Some(child_id) = ensure_button_text_child(state, id)? else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let val = frame_ref(state, child_id)?;
    state.push(val);
    Ok(1)
}

pub(crate) fn ensure_button_text_child(state: &mut LuaState, id: u64) -> LuaResult<Option<u64>> {
    if let Some(tid) = find_existing_text_child(state, id) {
        return Ok(Some(tid));
    }

    let fallback = {
        let sim = borrow_state(state)?;
        sim.widgets.get(id).map(|frame| {
            (
                matches!(
                    frame.widget_type,
                    crate::widget::WidgetType::Button | crate::widget::WidgetType::CheckButton
                ),
                frame.name.as_ref().map(|name| format!("{name}Text")),
                frame.text.clone(),
            )
        })
    };

    let Some((is_button, child_name, text_value)) = fallback else {
        return Ok(None);
    };
    if !is_button {
        return Ok(None);
    }

    let child_id = register_font_string_child(state, id, child_name, text_value)?;
    let _ = sync_child_to_rilua(state, id, "Text", child_id);
    Ok(Some(child_id))
}

fn register_font_string_child(
    state: &mut LuaState,
    id: u64,
    child_name: Option<String>,
    text_value: Option<String>,
) -> LuaResult<u64> {
    use crate::widget::{Frame, WidgetType};
    let mut font_string = Frame::new(WidgetType::FontString, child_name, Some(id));
    font_string.parent_key = Some("Text".to_string());
    if let Some(text_value) = text_value {
        font_string.text_stripped = Some(crate::render::strip_wow_markup(&text_value));
        font_string.text = Some(text_value);
    }
    super::super::methods_helpers::set_all_points_anchors_pub(&mut font_string, id);
    let child_id = font_string.id;
    let child_global_name = font_string.name.clone();

    let mut sim = borrow_state_mut(state)?;
    sim.widgets.register(font_string);
    sim.widgets.add_child(id, child_id);
    if let Some(button) = sim.widgets.get_mut_visual(id) {
        button.children_keys.insert("Text".to_string(), child_id);
    }
    sim.invalidate_strata_buckets();
    drop(sim);
    if let Some(child_global_name) = child_global_name {
        bind_named_child_global(state, &child_global_name, child_id)?;
    }
    Ok(child_id)
}

/// SetFontString(fontstring)
pub(super) fn set_font_string(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let fontstring_val = stack_val(state, 2);
    let fs_id_opt = extract_frame_id(state, fontstring_val);
    if let Some(fs_id) = fs_id_opt {
        attach_font_string_to_button(state, id, fs_id)?;
    } else {
        let mut sim = borrow_state_mut(state)?;
        if let Some(btn) = sim.widgets.get_mut_visual(id) {
            btn.children_keys.remove("Text");
        }
    }
    Ok(0)
}

fn attach_font_string_to_button(state: &mut LuaState, id: u64, fs_id: u64) -> LuaResult<()> {
    {
        let mut sim = borrow_state_mut(state)?;
        super::super::methods_hierarchy::reparent_widget(&mut sim.widgets, fs_id, Some(id));
        if let Some(fs) = sim.widgets.get_mut_visual(fs_id) {
            fs.anchors.clear();
            super::super::methods_helpers::set_all_points_anchors_pub(fs, id);
        }
        if let Some(btn) = sim.widgets.get_mut_visual(id) {
            btn.children_keys.insert("Text".to_string(), fs_id);
        }
        if let Some(fs) = sim.widgets.get_mut_visual(fs_id) {
            fs.parent_key = Some("Text".to_string());
        }
    }
    let _ = sync_child_to_rilua(state, id, "Text", fs_id);
    Ok(())
}

/// CreateFontString([name [, layer [, inherits]]]) -> fontstring
pub(super) fn create_font_string(state: &mut LuaState) -> LuaResult<u32> {
    use crate::widget::{DrawLayer, Frame, WidgetType};
    let parent_id = frame_id_from_stack(state, 1)?;
    let name_raw: Option<String> = Option::<String>::from_stack(state, 2)?;
    let layer = opt_string(state, 3);
    let inherits = opt_string(state, 4);

    let name = resolve_child_name(state, name_raw, parent_id);

    let mut fontstring = Frame::new(WidgetType::FontString, name.clone(), Some(parent_id));
    if let Some(layer_str) = layer {
        if let Some(draw_layer) = DrawLayer::from_str(&layer_str) {
            fontstring.draw_layer = draw_layer;
        }
    }
    let inherited_font_object = apply_font_inherit(state, &mut fontstring, inherits.as_deref());
    let child_id = fontstring.id;
    {
        let mut sim = borrow_state_mut(state)?;
        sim.widgets.register(fontstring);
        sim.widgets.add_child(parent_id, child_id);
        sim.invalidate_strata_buckets();
    }
    if let Some(ref n) = name {
        bind_named_child_global(state, n, child_id)?;
    }
    if let Some(font_object) = inherited_font_object {
        store_font_object_for_frame(state, child_id, font_object);
    }
    apply_font_string_template_mixins(state, child_id, inherits.as_deref())?;
    let val = frame_ref(state, child_id)?;
    state.push(val);
    Ok(1)
}

pub(super) fn resolve_child_name(
    state: &mut LuaState,
    name_raw: Option<String>,
    parent_id: u64,
) -> Option<String> {
    name_raw.map(|n| {
        let sim = borrow_state(state).ok();
        if let Some(sim) = sim {
            crate::lua_api::globals::create_frame::apply_parent_sub(&n, Some(parent_id), &sim)
        } else {
            n
        }
    })
}

fn apply_font_inherit(
    state: &mut LuaState,
    fontstring: &mut crate::widget::Frame,
    inherits: Option<&str>,
) -> Option<Val> {
    let Some(inherits) = inherits else {
        return None;
    };
    let mut visited = HashSet::new();
    apply_font_inherit_names(state, fontstring, inherits, &mut visited)
}

fn apply_font_inherit_names(
    state: &mut LuaState,
    fontstring: &mut crate::widget::Frame,
    inherits: &str,
    visited: &mut HashSet<String>,
) -> Option<Val> {
    let mut inherited_font_object = None;
    for name in inherits.split(',').map(str::trim) {
        if name.is_empty() || !visited.insert(name.to_string()) {
            continue;
        }
        if let Some(font_object) = apply_font_object_by_name(state, fontstring, name) {
            inherited_font_object.get_or_insert(font_object);
            continue;
        }
        if let Some(template) = crate::xml::get_font_string_template(name) {
            let template_font_object =
                apply_font_string_template(state, fontstring, &template, visited);
            inherited_font_object = inherited_font_object.or(template_font_object);
        }
    }
    inherited_font_object
}

fn apply_font_object_by_name(
    state: &mut LuaState,
    fontstring: &mut crate::widget::Frame,
    name: &str,
) -> Option<Val> {
    let font_object = table_get(state, Val::Table(state.global), name);
    if !matches!(font_object, Val::Table(_)) {
        return None;
    }

    apply_font_object_fields(state, fontstring, font_object.clone());
    Some(font_object)
}

fn apply_font_string_template(
    state: &mut LuaState,
    fontstring: &mut crate::widget::Frame,
    template: &crate::xml::FontStringXml,
    visited: &mut HashSet<String>,
) -> Option<Val> {
    let mut inherited_font_object = None;
    if let Some(inherits) = template.inherits.as_deref() {
        inherited_font_object = apply_font_inherit_names(state, fontstring, inherits, visited);
    }
    if let Some(font) = template.font.as_deref() {
        let font_object = apply_font_inherit_names(state, fontstring, font, visited);
        inherited_font_object = inherited_font_object.or(font_object);
    }
    apply_font_string_template_fields(state, fontstring, template);
    inherited_font_object
}

fn store_font_object_for_frame(state: &mut LuaState, frame_id: u64, font_object: Val) {
    let store = registry_table_or_create(state, FONT_OBJECTS_REGISTRY_KEY);
    table_set(state, store, &frame_id.to_string(), font_object);
}

fn apply_font_string_template_fields(
    state: &mut LuaState,
    fontstring: &mut crate::widget::Frame,
    template: &crate::xml::FontStringXml,
) {
    #[cfg(feature = "retail-12-0-5")]
    if let Some(enabled) = template.smooth_scaling {
        fontstring.font_string_smooth_scaling = enabled;
    }
    if let Some(justify_h) = template.justify_h.as_deref() {
        fontstring.justify_h = crate::widget::TextJustify::from_wow_str(justify_h);
    }
    if let Some(justify_v) = template.justify_v.as_deref() {
        fontstring.justify_v = crate::widget::TextJustify::from_wow_str(justify_v);
    }
    if let Some(height) = template
        .font_height
        .as_ref()
        .and_then(|font_height| font_height.value())
    {
        fontstring.font_size = height as f32;
    }
    if let Some(color) = template_color(state, template.color.as_ref()) {
        fontstring.text_color = color;
    }
}

fn template_color(
    state: &mut LuaState,
    color: Option<&crate::xml::ColorXml>,
) -> Option<crate::widget::Color> {
    let color = color?;
    if let Some(name) = color.color.as_deref() {
        return named_color(state, name);
    }
    Some(crate::widget::Color::new(
        color.r.unwrap_or(1.0),
        color.g.unwrap_or(1.0),
        color.b.unwrap_or(1.0),
        color.a.unwrap_or(1.0),
    ))
}

fn named_color(state: &mut LuaState, name: &str) -> Option<crate::widget::Color> {
    let table = table_get(state, Val::Table(state.global), name);
    if !matches!(table, Val::Table(_)) {
        return None;
    }
    Some(crate::widget::Color::new(
        font_field_number(state, table.clone(), "r", "r").unwrap_or(1.0) as f32,
        font_field_number(state, table.clone(), "g", "g").unwrap_or(1.0) as f32,
        font_field_number(state, table.clone(), "b", "b").unwrap_or(1.0) as f32,
        font_field_number(state, table, "a", "a").unwrap_or(1.0) as f32,
    ))
}

fn apply_font_string_template_mixins(
    state: &mut LuaState,
    child_id: u64,
    inherits: Option<&str>,
) -> LuaResult<()> {
    let mixins = crate::xml::collect_font_string_mixins(inherits, None);
    if mixins.is_empty() {
        return Ok(());
    }

    crate::lua_api::globals::create_frame::apply_frame_mixins(
        state,
        child_id,
        Some(&mixins.join(",")),
    )
}

pub(crate) fn apply_font_object_fields(
    state: &mut LuaState,
    fontstring: &mut crate::widget::Frame,
    font_object: Val,
) {
    let fields = read_font_object_fields(state, font_object);
    apply_font_object_snapshot(fontstring, &fields);
}

pub(crate) struct FontObjectFields {
    pub(crate) font: Option<String>,
    pub(crate) font_size: Option<f32>,
    pub(crate) spacing: Option<f32>,
    pub(crate) font_outline: Option<crate::widget::TextOutline>,
    pub(crate) justify_h: Option<crate::widget::TextJustify>,
    pub(crate) justify_v: Option<crate::widget::TextJustify>,
    pub(crate) text_color: Option<crate::widget::Color>,
    pub(crate) shadow_color: Option<crate::widget::Color>,
    pub(crate) shadow_offset: Option<(f32, f32)>,
}

pub(crate) fn read_font_object_fields(state: &mut LuaState, font_object: Val) -> FontObjectFields {
    FontObjectFields {
        font: font_field_string(state, font_object.clone(), "__fontPath", "__font"),
        font_size: font_field_number(state, font_object.clone(), "__fontHeight", "__height")
            .map(|height| height as f32),
        spacing: font_field_number(state, font_object.clone(), "__spacing", "__spacing")
            .map(|spacing| spacing as f32),
        font_outline: font_field_string(state, font_object.clone(), "__fontFlags", "__outline")
            .map(|outline| crate::widget::TextOutline::from_wow_str(&outline)),
        justify_h: font_field_string(state, font_object.clone(), "__justifyH", "__justifyH")
            .map(|justify| crate::widget::TextJustify::from_wow_str(&justify)),
        justify_v: font_field_string(state, font_object.clone(), "__justifyV", "__justifyV")
            .map(|justify| crate::widget::TextJustify::from_wow_str(&justify)),
        text_color: read_color(state, font_object.clone(), TEXT_COLOR_FIELD_KEYS),
        shadow_color: read_color(state, font_object.clone(), SHADOW_COLOR_FIELD_KEYS),
        shadow_offset: read_shadow_offset(state, font_object),
    }
}

const TEXT_COLOR_FIELD_KEYS: [&str; 4] = [
    "__textColorR",
    "__textColorG",
    "__textColorB",
    "__textColorA",
];

const SHADOW_COLOR_FIELD_KEYS: [&str; 4] = [
    "__shadowColorR",
    "__shadowColorG",
    "__shadowColorB",
    "__shadowColorA",
];

pub(crate) fn apply_font_object_snapshot(
    fontstring: &mut crate::widget::Frame,
    fields: &FontObjectFields,
) {
    if let Some(path) = &fields.font {
        fontstring.font = Some(path.clone());
    }
    if let Some(height) = fields.font_size {
        fontstring.font_size = height;
    }
    if let Some(spacing) = fields.spacing {
        fontstring.text_line_spacing = spacing;
    }
    if let Some(outline) = fields.font_outline {
        fontstring.font_outline = outline;
    }
    if let Some(justify_h) = fields.justify_h {
        fontstring.justify_h = justify_h;
    }
    if let Some(justify_v) = fields.justify_v {
        fontstring.justify_v = justify_v;
    }
    apply_font_object_colors_and_shadow(fontstring, fields);
}

fn apply_font_object_colors_and_shadow(
    fontstring: &mut crate::widget::Frame,
    fields: &FontObjectFields,
) {
    if let Some(text_color) = fields.text_color {
        fontstring.text_color = text_color;
    }
    if let Some(shadow_color) = fields.shadow_color {
        fontstring.shadow_color = shadow_color;
    }
    if let Some(shadow_offset) = fields.shadow_offset {
        fontstring.shadow_offset = shadow_offset;
    }
}

pub(crate) fn font_object_snapshot_changes_frame(
    frame: &crate::widget::Frame,
    fields: &FontObjectFields,
) -> bool {
    fields
        .font
        .as_ref()
        .is_some_and(|font| frame.font.as_deref() != Some(font.as_str()))
        || fields
            .font_size
            .is_some_and(|font_size| (frame.font_size - font_size).abs() > f32::EPSILON)
        || fields
            .spacing
            .is_some_and(|spacing| frame.text_line_spacing != spacing)
        || fields
            .font_outline
            .is_some_and(|outline| frame.font_outline != outline)
        || fields
            .justify_h
            .is_some_and(|justify_h| frame.justify_h != justify_h)
        || fields
            .justify_v
            .is_some_and(|justify_v| frame.justify_v != justify_v)
        || font_object_colors_or_shadow_change(frame, fields)
}

fn font_object_colors_or_shadow_change(
    frame: &crate::widget::Frame,
    fields: &FontObjectFields,
) -> bool {
    fields
        .text_color
        .is_some_and(|color| frame.text_color != color)
        || fields
            .shadow_color
            .is_some_and(|color| frame.shadow_color != color)
        || fields
            .shadow_offset
            .is_some_and(|shadow_offset| frame.shadow_offset != shadow_offset)
}

fn font_field_string(
    state: &mut LuaState,
    table: Val,
    primary: &'static str,
    fallback: &'static str,
) -> Option<String> {
    let primary_value = table_get_static(state, table.clone(), primary);
    match primary_value {
        Val::Str(_) => val_to_string(state, primary_value),
        _ => {
            let fallback_value = table_get_static(state, table, fallback);
            val_to_string(state, fallback_value)
        }
    }
}

fn font_field_number(
    state: &mut LuaState,
    table: Val,
    primary: &'static str,
    fallback: &'static str,
) -> Option<f64> {
    match table_get_static(state, table.clone(), primary) {
        Val::Num(value) => Some(value),
        _ => match table_get_static(state, table, fallback) {
            Val::Num(value) => Some(value),
            _ => None,
        },
    }
}

fn read_color(
    state: &mut LuaState,
    table: Val,
    keys: [&'static str; 4],
) -> Option<crate::widget::Color> {
    let r = font_field_number(state, table.clone(), keys[0], keys[0])?;
    let g = font_field_number(state, table.clone(), keys[1], keys[1])?;
    let b = font_field_number(state, table.clone(), keys[2], keys[2])?;
    let a = font_field_number(state, table, keys[3], keys[3]).unwrap_or(1.0);
    Some(crate::widget::Color::new(
        r as f32, g as f32, b as f32, a as f32,
    ))
}

fn read_shadow_offset(state: &mut LuaState, table: Val) -> Option<(f32, f32)> {
    let x = font_field_number(state, table.clone(), "__shadowOffsetX", "__shadowOffsetX")?;
    let y = font_field_number(state, table, "__shadowOffsetY", "__shadowOffsetY")?;
    Some((x as f32, y as f32))
}
