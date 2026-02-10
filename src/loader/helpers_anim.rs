//! Animation group code generation helpers.

use super::helpers::{apply_script_handlers, escape_lua_string};

/// Generate Lua code for creating an animation group and its child animations from XML.
pub fn generate_animation_group_code(
    anim_group: &crate::xml::AnimationGroupXml,
    frame_ref: &str,
) -> String {
    let mut code = String::new();

    code.push_str(&format!(
        "\n        do\n        local __ag = {frame_ref}:CreateAnimationGroup({}, {})\n        ",
        lua_opt_str(anim_group.name.as_deref()),
        lua_opt_str(anim_group.inherits.as_deref()),
    ));

    if let Some(parent_key) = &anim_group.parent_key {
        code.push_str(&format!("\n        {frame_ref}.{parent_key} = __ag\n        "));
    }
    emit_str_call(&mut code, "__ag", "SetLooping", anim_group.looping.as_deref());
    if anim_group.set_to_final_alpha == Some(true) {
        code.push_str("\n        __ag:SetToFinalAlpha(true)\n        ");
    }

    // Emit children from inherited templates first, then our own
    emit_inherited_anim_group_children(&mut code, anim_group, frame_ref);
    emit_anim_group_children(&mut code, anim_group, frame_ref);
    emit_anim_group_mixin(&mut code, anim_group);

    // Fire OnLoad after the animation group is fully configured (mixin + scripts).
    // TargetsVisibleWhilePlayingAnimGroupTemplate uses OnLoad to Hide() animation targets.
    code.push_str(r#"
        do local __onLoad = __ag:GetScript("OnLoad")
        if __onLoad then __onLoad(__ag) end end
        "#);

    code.push_str("\n        end\n        ");
    code
}

/// Emit child elements from inherited AnimationGroup templates.
fn emit_inherited_anim_group_children(
    code: &mut String,
    anim_group: &crate::xml::AnimationGroupXml,
    frame_ref: &str,
) {
    let Some(ref inherits) = anim_group.inherits else { return };
    let registry = crate::xml::anim_group_template_registry_read();
    for parent_name in inherits.split(',').map(|s| s.trim()) {
        if let Some(parent) = registry.get(parent_name) {
            // Apply looping from template if not overridden
            if anim_group.looping.is_none() {
                emit_str_call(code, "__ag", "SetLooping", parent.looping.as_deref());
            }
            emit_anim_group_children(code, parent, frame_ref);
        }
    }
}

/// Emit child elements (scripts, keyvalues, animations) for an animation group.
fn emit_anim_group_children(
    code: &mut String,
    anim_group: &crate::xml::AnimationGroupXml,
    frame_ref: &str,
) {
    for element in &anim_group.elements {
        match element {
            crate::xml::AnimationElement::Scripts(scripts) => {
                code.push_str(&generate_anim_group_scripts_code(scripts, "__ag"));
            }
            crate::xml::AnimationElement::KeyValues(kv) => {
                emit_anim_key_values(code, kv);
            }
            crate::xml::AnimationElement::Unknown => {}
            _ => {
                if let Some((type_str, xml)) = resolve_animation_element(element) {
                    code.push_str(&generate_animation_code(xml, type_str, frame_ref));
                }
            }
        }
    }
}

/// Emit KeyValue assignments for an animation group.
fn emit_anim_key_values(code: &mut String, kv: &crate::xml::KeyValuesXml) {
    for key_value in &kv.values {
        let value = match key_value.value_type.as_deref() {
            Some("number") => key_value.value.clone(),
            Some("boolean") => key_value.value.to_lowercase(),
            _ => format!("\"{}\"", escape_lua_string(&key_value.value)),
        };
        code.push_str(&format!("\n        __ag.{} = {}\n        ", key_value.key, value));
    }
}

/// Resolve an AnimationElement variant to its type string and XML data.
fn resolve_animation_element(
    element: &crate::xml::AnimationElement,
) -> Option<(&str, &crate::xml::AnimationXml)> {
    match element {
        crate::xml::AnimationElement::Alpha(a) => Some(("Alpha", a)),
        crate::xml::AnimationElement::Translation(a) => Some(("Translation", a)),
        crate::xml::AnimationElement::LineTranslation(a) => Some(("LineTranslation", a)),
        crate::xml::AnimationElement::Rotation(a) => Some(("Rotation", a)),
        crate::xml::AnimationElement::Scale(a) => Some(("Scale", a)),
        crate::xml::AnimationElement::LineScale(a) => Some(("LineScale", a)),
        crate::xml::AnimationElement::Path(a) => Some(("Path", a)),
        crate::xml::AnimationElement::FlipBook(a) => Some(("FlipBook", a)),
        crate::xml::AnimationElement::VertexColor(a) => Some(("VertexColor", a)),
        crate::xml::AnimationElement::TextureCoordTranslation(a) => {
            Some(("TextureCoordTranslation", a))
        }
        crate::xml::AnimationElement::Animation(a) => Some(("Animation", a)),
        _ => None,
    }
}

/// Emit Mixin() calls for an animation group, including inherited mixins.
fn emit_anim_group_mixin(code: &mut String, anim_group: &crate::xml::AnimationGroupXml) {
    let mixins = crate::xml::collect_anim_group_mixins(anim_group);
    for m in &mixins {
        code.push_str(&format!("\n        if {m} then Mixin(__ag, {m}) end\n        "));
    }
}

/// Format an optional string as a Lua string literal or "nil".
fn lua_opt_str(s: Option<&str>) -> String {
    match s.filter(|s| !s.is_empty()) {
        Some(s) => format!("\"{}\"", escape_lua_string(s)),
        None => "nil".to_string(),
    }
}

/// Append a Lua method call with a single numeric argument if the value is Some.
fn emit_num_call(code: &mut String, target: &str, method: &str, val: Option<f32>) {
    if let Some(v) = val {
        code.push_str(&format!("\n        {target}:{method}({v})\n        "));
    }
}

/// Append a Lua method call with a single string argument if the value is Some.
fn emit_str_call(code: &mut String, target: &str, method: &str, val: Option<&str>) {
    if let Some(v) = val {
        code.push_str(&format!(
            "\n        {target}:{method}(\"{}\")\n        ",
            escape_lua_string(v)
        ));
    }
}

/// Append a Lua method call with two numeric arguments if either value is Some.
fn emit_pair_call(
    code: &mut String,
    target: &str,
    method: &str,
    a: Option<f32>,
    b: Option<f32>,
    default: f32,
) {
    if a.is_some() || b.is_some() {
        code.push_str(&format!(
            "\n        {target}:{method}({}, {})\n        ",
            a.unwrap_or(default),
            b.unwrap_or(default)
        ));
    }
}

/// Generate Lua code for a single animation element within a group.
fn generate_animation_code(
    anim: &crate::xml::AnimationXml,
    anim_type: &str,
    _frame_ref: &str,
) -> String {
    let mut code = String::new();

    code.push_str(&format!(
        "\n        local __anim = __ag:CreateAnimation(\"{anim_type}\", {})\n        ",
        lua_opt_str(anim.name.as_deref()),
    ));

    if let Some(parent_key) = &anim.parent_key {
        code.push_str(&format!("\n        __ag.{parent_key} = __anim\n        "));
    }

    emit_num_call(&mut code, "__anim", "SetDuration", anim.duration);
    if let Some(order) = anim.order {
        code.push_str(&format!("\n        __anim:SetOrder({order})\n        "));
    }
    emit_num_call(&mut code, "__anim", "SetStartDelay", anim.start_delay);
    emit_num_call(&mut code, "__anim", "SetEndDelay", anim.end_delay);
    emit_str_call(&mut code, "__anim", "SetSmoothing", anim.smoothing.as_deref());
    emit_num_call(&mut code, "__anim", "SetFromAlpha", anim.from_alpha);
    emit_num_call(&mut code, "__anim", "SetToAlpha", anim.to_alpha);
    emit_pair_call(&mut code, "__anim", "SetOffset", anim.offset_x, anim.offset_y, 0.0);
    emit_pair_call(&mut code, "__anim", "SetScale", anim.scale_x, anim.scale_y, 1.0);
    emit_pair_call(&mut code, "__anim", "SetScaleFrom", anim.from_scale_x, anim.from_scale_y, 1.0);
    emit_pair_call(&mut code, "__anim", "SetScaleTo", anim.to_scale_x, anim.to_scale_y, 1.0);
    emit_num_call(&mut code, "__anim", "SetDegrees", anim.degrees);
    emit_str_call(&mut code, "__anim", "SetChildKey", anim.child_key.as_deref());
    emit_str_call(&mut code, "__anim", "SetTargetName", anim.target.as_deref());
    emit_str_call(&mut code, "__anim", "SetTargetKey", anim.target_key.as_deref());

    // FlipBook properties
    if let Some(rows) = anim.flip_book_rows {
        code.push_str(&format!("\n        __anim:SetFlipBookRows({rows})\n        "));
    }
    if let Some(cols) = anim.flip_book_columns {
        code.push_str(&format!("\n        __anim:SetFlipBookColumns({cols})\n        "));
    }
    if let Some(frames) = anim.flip_book_frames {
        code.push_str(&format!("\n        __anim:SetFlipBookFrames({frames})\n        "));
    }

    code
}

/// Generate Lua code for animation group script handlers (OnPlay, OnFinished, etc.).
fn generate_anim_group_scripts_code(
    scripts: &crate::xml::ScriptsXml,
    group_ref: &str,
) -> String {
    apply_script_handlers(group_ref, &[
        ("OnLoad", scripts.on_load.last()),
        ("OnPlay", scripts.on_play.last()),
        ("OnFinished", scripts.on_finished.last()),
        ("OnStop", scripts.on_stop.last()),
        ("OnLoop", scripts.on_loop.last()),
        ("OnPause", scripts.on_pause.last()),
    ])
}
