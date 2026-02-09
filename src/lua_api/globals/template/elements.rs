//! Template element creation: textures, fontstrings, thumb/button textures.

use crate::loader::helpers::{generate_set_point_code, resolve_lua_escapes};
use crate::loader::helpers_anim::generate_animation_group_code;
use mlua::Lua;

use super::{escape_lua_string, get_size_values, lua_global_ref, rand_id};

/// Create a texture from template XML.
///
/// `parent_name` is the actual Lua frame name (for parent reference).
/// `subst_parent` is the name used for `$parent` substitution in child names
/// (propagated through anonymous frames to the nearest named ancestor).
pub(super) fn create_texture_from_template(
    lua: &Lua,
    texture: &crate::xml::TextureXml,
    parent_name: &str,
    subst_parent: &str,
    draw_layer: &str,
    is_mask: bool,
) {
    let child_name = texture
        .name
        .as_ref()
        .map(|n| n.replace("$parent", subst_parent))
        .unwrap_or_else(|| format!("__tex_{}", rand_id()));

    let create_method = if is_mask { "CreateMaskTexture" } else { "CreateTexture" };
    let mut code = format!(
        r#"
        local parent = {}
        if parent then
            local tex = parent:{}("{}", "{}")
        "#,
        lua_global_ref(parent_name), create_method, escape_lua_string(&child_name), draw_layer,
    );

    // Apply mixins from inherited templates and direct mixin attribute
    let mixins = crate::xml::collect_texture_mixins(texture);
    for m in &mixins {
        code.push_str(&format!(
            "            if {} then Mixin(tex, {}) end\n",
            m, m
        ));
    }

    append_texture_properties(&mut code, texture, "tex");
    append_anchors_and_parent_refs(
        &mut code, &texture.anchors, texture.set_all_points,
        &texture.parent_key, &texture.parent_array,
        "tex", "parent", parent_name,
    );

    // WoW implicitly applies SetAllPoints to textures with no anchors
    if texture.anchors.is_none() && texture.set_all_points != Some(true) {
        code.push_str("            tex:SetAllPoints(true)\n");
    }

    if texture.name.is_some() {
        code.push_str(&format!("            _G[\"{}\"] = tex\n", escape_lua_string(&child_name)));
    }

    if texture.hidden == Some(true) {
        code.push_str("            tex:Hide()\n");
    }

    if let Some(a) = texture.alpha {
        code.push_str(&format!("            tex:SetAlpha({})\n", a));
    }

    if let Some(ref mode) = texture.alpha_mode {
        code.push_str(&format!("            tex:SetBlendMode(\"{}\")\n", mode));
    }

    append_deferred_mask_wiring(&mut code, is_mask, texture);

    code.push_str("        end\n");
    if let Err(e) = lua.load(&code).exec() {
        eprintln!("[create_texture] failed for '{}' on '{}': {}", child_name, parent_name, e);
    }

    apply_texture_animations(lua, texture, &child_name);
}

/// Process animation groups on a texture.
fn apply_texture_animations(lua: &Lua, texture: &crate::xml::TextureXml, child_name: &str) {
    let Some(anims) = &texture.animations else { return };
    let mut anim_code = format!("local frame = {}\n", lua_global_ref(child_name));
    for group in &anims.animations {
        if group.is_virtual == Some(true) {
            continue;
        }
        anim_code.push_str(&generate_animation_group_code(group, "frame"));
    }
    let _ = lua.load(&anim_code).exec();
}

/// Append deferred MaskedTextures wiring to Lua code.
///
/// Uses `C_Timer.After(0, ...)` because layers are processed before child frames —
/// referenced siblings (e.g. `HealthBar.MyHealPredictionBar.Fill`) may not exist yet.
/// Dotted paths get multi-level nil guards to avoid "attempt to index nil" errors.
fn append_deferred_mask_wiring(code: &mut String, is_mask: bool, texture: &crate::xml::TextureXml) {
    if !is_mask {
        return;
    }
    let Some(ref masked) = texture.masked_textures else { return };
    let mut mask_lines = Vec::new();
    for entry in &masked.entries {
        if let Some(ref key) = entry.child_key {
            mask_lines.push(safe_add_mask_texture_code("parent", key));
        }
    }
    if mask_lines.is_empty() {
        return;
    }
    code.push_str("            C_Timer.After(0, function()\n");
    for line in &mask_lines {
        code.push_str(&format!("                {line}\n"));
    }
    code.push_str("            end)\n");
}

/// Generate Lua code that safely navigates a dotted childKey path and calls AddMaskTexture.
///
/// For `root="parent"` and `key="HealthBar.MyHealPredictionBar.Fill"`, produces:
/// `if parent.HealthBar and parent.HealthBar.MyHealPredictionBar and parent.HealthBar.MyHealPredictionBar.Fill then parent.HealthBar.MyHealPredictionBar.Fill:AddMaskTexture(tex) end`
fn safe_add_mask_texture_code(root: &str, key: &str) -> String {
    let parts: Vec<&str> = key.split('.').collect();
    let full_path = format!("{root}.{key}");
    if parts.len() <= 1 {
        return format!("if {full_path} then {full_path}:AddMaskTexture(tex) end");
    }
    let mut guards = Vec::new();
    let mut path = root.to_string();
    for part in &parts {
        path = format!("{path}.{part}");
        guards.push(path.clone());
    }
    let guard_str = guards.join(" and ");
    format!("if {guard_str} then {full_path}:AddMaskTexture(tex) end")
}

/// Append texture-specific property setters (size, file, atlas, color) to Lua code.
fn append_texture_properties(code: &mut String, texture: &crate::xml::TextureXml, var: &str) {
    if let Some(size) = &texture.size {
        let (width, height) = get_size_values(size);
        if let (Some(w), Some(h)) = (width, height) {
            code.push_str(&format!("            {}:SetSize({}, {})\n", var, w, h));
        }
    }
    if let Some(file) = &texture.file {
        code.push_str(&format!(
            "            {}:SetTexture(\"{}\")\n",
            var,
            escape_lua_string(file)
        ));
    }
    if let Some(atlas) = &texture.atlas {
        let use_atlas_size = texture.use_atlas_size.unwrap_or(false);
        code.push_str(&format!(
            "            {}:SetAtlas(\"{}\", {})\n",
            var,
            escape_lua_string(atlas),
            use_atlas_size
        ));
    }
    if let Some(color) = &texture.color {
        let r = color.r.unwrap_or(1.0);
        let g = color.g.unwrap_or(1.0);
        let b = color.b.unwrap_or(1.0);
        let a = color.a.unwrap_or(1.0);
        code.push_str(&format!(
            "            {}:SetColorTexture({}, {}, {}, {})\n",
            var, r, g, b, a
        ));
    }
}

/// Append anchors, setAllPoints, parentKey, and parentArray assignment to Lua code.
#[allow(clippy::too_many_arguments)]
fn append_anchors_and_parent_refs(
    code: &mut String,
    anchors: &Option<crate::xml::AnchorsXml>,
    set_all_points: Option<bool>,
    parent_key: &Option<String>,
    parent_array: &Option<String>,
    var: &str,
    parent_var: &str,
    parent_name: &str,
) {
    if let Some(anchors) = anchors {
        code.push_str(&generate_set_point_code(anchors, var, parent_var, parent_name, "nil"));
    }
    if set_all_points == Some(true) {
        code.push_str(&format!("            {}:SetAllPoints(true)\n", var));
    }
    if let Some(parent_key) = parent_key {
        let key_escaped = escape_lua_string(parent_key);
        code.push_str(&format!("            {parent_var}[\"{key_escaped}\"] = {var}\n"));
    }
    if let Some(parent_array) = parent_array {
        let arr_escaped = escape_lua_string(parent_array);
        code.push_str(&format!(
            "            {parent_var}[\"{arr_escaped}\"] = {parent_var}[\"{arr_escaped}\"] or {{}}\n\
             table.insert({parent_var}[\"{arr_escaped}\"], {var})\n"
        ));
    }
}

/// Create a fontstring from template XML.
///
/// `subst_parent` is the name used for `$parent` substitution (propagated
/// through anonymous frames).
pub(super) fn create_fontstring_from_template(
    lua: &Lua,
    fontstring: &crate::xml::FontStringXml,
    parent_name: &str,
    subst_parent: &str,
    draw_layer: &str,
) {
    let child_name = fontstring
        .name
        .as_ref()
        .map(|n| n.replace("$parent", subst_parent))
        .unwrap_or_else(|| format!("__fs_{}", rand_id()));

    let inherits = fontstring.inherits.as_deref().unwrap_or("");

    let mut code = format!(
        r#"
        local parent = {}
        if parent then
            local fs = parent:CreateFontString("{}", "{}", {})
        "#,
        lua_global_ref(parent_name),
        escape_lua_string(&child_name),
        draw_layer,
        if inherits.is_empty() {
            "nil".to_string()
        } else {
            format!("\"{}\"", inherits)
        }
    );

    append_fontstring_size_and_text(&mut code, fontstring);
    append_fontstring_justify_and_color(&mut code, fontstring);
    append_fontstring_shadow(&mut code, fontstring);
    append_anchors_and_parent_refs(
        &mut code,
        &fontstring.anchors,
        fontstring.set_all_points,
        &fontstring.parent_key,
        &fontstring.parent_array,
        "fs",
        "parent",
        parent_name,
    );
    append_fontstring_wrap_and_lines(&mut code, fontstring);

    if fontstring.name.is_some() {
        code.push_str(&format!("            _G[\"{}\"] = fs\n", escape_lua_string(&child_name)));
    }

    if fontstring.hidden == Some(true) {
        code.push_str("            fs:Hide()\n");
    }

    if let Some(a) = fontstring.alpha {
        code.push_str(&format!("            fs:SetAlpha({})\n", a));
    }

    code.push_str("        end\n");
    let _ = lua.load(&code).exec();
}

/// Append size and text setters for a fontstring.
fn append_fontstring_size_and_text(code: &mut String, fs: &crate::xml::FontStringXml) {
    if let Some(size) = fs.size.last() {
        let (width, height) = get_size_values(size);
        if let (Some(w), Some(h)) = (width, height) {
            code.push_str(&format!("            fs:SetSize({}, {})\n", w, h));
        }
    }
    if let Some(text_key) = &fs.text {
        let raw = crate::global_strings::get_global_string(text_key)
            .unwrap_or(text_key.as_str());
        let resolved = resolve_lua_escapes(raw);
        code.push_str(&format!(
            "            fs:SetText(\"{}\")\n",
            escape_lua_string(&resolved)
        ));
    }
}

/// Append justification and text color for a fontstring.
fn append_fontstring_justify_and_color(code: &mut String, fs: &crate::xml::FontStringXml) {
    if let Some(justify_h) = &fs.justify_h {
        code.push_str(&format!("            fs:SetJustifyH(\"{}\")\n", justify_h));
    }
    if let Some(justify_v) = &fs.justify_v {
        code.push_str(&format!("            fs:SetJustifyV(\"{}\")\n", justify_v));
    }
    if let Some(color) = &fs.color {
        let r = color.r.unwrap_or(1.0);
        let g = color.g.unwrap_or(1.0);
        let b = color.b.unwrap_or(1.0);
        let a = color.a.unwrap_or(1.0);
        code.push_str(&format!(
            "            fs:SetTextColor({}, {}, {}, {})\n",
            r, g, b, a
        ));
    }
}

/// Append shadow offset and color for a fontstring.
fn append_fontstring_shadow(code: &mut String, fs: &crate::xml::FontStringXml) {
    let Some(shadow) = &fs.shadow else { return };
    if let Some(offset) = &shadow.offset {
        let x = offset.x();
        let y = offset.y();
        code.push_str(&format!("            fs:SetShadowOffset({}, {})\n", x, y));
    }
    if let Some(color) = &shadow.color {
        let r = color.r.unwrap_or(0.0);
        let g = color.g.unwrap_or(0.0);
        let b = color.b.unwrap_or(0.0);
        let a = color.a.unwrap_or(1.0);
        code.push_str(&format!(
            "            fs:SetShadowColor({}, {}, {}, {})\n",
            r, g, b, a
        ));
    }
}

/// Append wordWrap and maxLines for a fontstring.
fn append_fontstring_wrap_and_lines(code: &mut String, fs: &crate::xml::FontStringXml) {
    if fs.word_wrap == Some(false) {
        code.push_str("            fs:SetWordWrap(false)\n");
    }
    if let Some(max_lines) = fs.max_lines
        && max_lines > 0 {
            code.push_str(&format!("            fs:SetMaxLines({})\n", max_lines));
        }
}

/// Create a bar texture from template XML (for StatusBars).
pub(super) fn create_bar_texture_from_template(
    lua: &Lua,
    bar: &crate::xml::TextureXml,
    parent_name: &str,
    subst_parent: &str,
) {
    let child_name = bar
        .name
        .as_ref()
        .map(|n| n.replace("$parent", subst_parent))
        .unwrap_or_else(|| format!("__bar_{}", rand_id()));

    let mut code = format!(
        r#"
        local parent = {}
        if parent and parent.SetStatusBarTexture then
            local bar = parent:CreateTexture("{}", "ARTWORK")
        "#,
        lua_global_ref(parent_name), escape_lua_string(&child_name),
    );

    append_texture_properties(&mut code, bar, "bar");
    code.push_str("            parent:SetStatusBarTexture(bar)\n");
    let parent_key = bar.parent_key.as_deref().unwrap_or("Bar");
    code.push_str(&format!("            parent[\"{}\"] = bar\n", escape_lua_string(parent_key)));

    if bar.name.is_some() {
        code.push_str(&format!("            _G[\"{}\"] = bar\n", escape_lua_string(&child_name)));
    }

    code.push_str("        end\n");
    let _ = lua.load(&code).exec();
}

/// Create a thumb texture from template XML (for sliders).
pub(super) fn create_thumb_texture_from_template(
    lua: &Lua,
    thumb: &crate::xml::TextureXml,
    parent_name: &str,
    subst_parent: &str,
) {
    let child_name = thumb
        .name
        .as_ref()
        .map(|n| n.replace("$parent", subst_parent))
        .unwrap_or_else(|| format!("__thumb_{}", rand_id()));

    let mut code = format!(
        r#"
        local parent = {}
        if parent and parent.SetThumbTexture then
            local thumb = parent:CreateTexture("{}", "ARTWORK")
        "#,
        lua_global_ref(parent_name), escape_lua_string(&child_name),
    );

    if let Some(size) = &thumb.size {
        let (width, height) = get_size_values(size);
        if let (Some(w), Some(h)) = (width, height) {
            code.push_str(&format!("            thumb:SetSize({}, {})\n", w, h));
        }
    }
    if let Some(file) = &thumb.file {
        code.push_str(&format!(
            "            thumb:SetTexture(\"{}\")\n",
            escape_lua_string(file)
        ));
    }

    code.push_str("            parent:SetThumbTexture(thumb)\n");
    if let Some(parent_key) = &thumb.parent_key {
        code.push_str(&format!("            parent[\"{}\"] = thumb\n", escape_lua_string(parent_key)));
    } else {
        code.push_str("            parent[\"ThumbTexture\"] = thumb\n");
    }

    if thumb.name.is_some() {
        code.push_str(&format!("            _G[\"{}\"] = thumb\n", escape_lua_string(&child_name)));
    }

    code.push_str("        end\n");
    let _ = lua.load(&code).exec();
}

/// Create a button texture from template XML (NormalTexture, PushedTexture, etc.).
pub(super) fn create_button_texture_from_template(
    lua: &Lua,
    texture: &crate::xml::TextureXml,
    parent_name: &str,
    subst_parent: &str,
    parent_key: &str,
    setter_method: &str,
) {
    let default_parent_key = format!("{}Texture", parent_key);
    let actual_parent_key = texture
        .parent_key
        .as_deref()
        .unwrap_or(&default_parent_key);

    let child_name = texture
        .name
        .as_ref()
        .map(|n| n.replace("$parent", subst_parent));

    let tex_name = child_name
        .clone()
        .unwrap_or_else(|| format!("__tex_{}", rand_id()));

    let code = build_button_texture_code(
        texture,
        parent_name,
        setter_method,
        actual_parent_key,
        &tex_name,
        child_name.is_some(),
    );
    let _ = lua.load(&code).exec();
}

/// Build Lua code to create or reuse a button texture with properties applied.
fn build_button_texture_code(
    texture: &crate::xml::TextureXml,
    parent_name: &str,
    setter_method: &str,
    actual_parent_key: &str,
    tex_name: &str,
    is_named: bool,
) -> String {
    let key_escaped = escape_lua_string(actual_parent_key);
    let mut code = format!(
        r#"
        local parent = {}
        if parent and parent.{} then
            local tex = parent["{}"]
            if tex == nil then
                tex = parent:CreateTexture("{}", "ARTWORK")
            end
        "#,
        lua_global_ref(parent_name), setter_method, key_escaped, escape_lua_string(tex_name),
    );

    if let Some(size) = &texture.size {
        let (width, height) = get_size_values(size);
        if let (Some(w), Some(h)) = (width, height) {
            code.push_str(&format!("            tex:SetSize({}, {})\n", w, h));
        }
    }

    code.push_str(&format!("            parent[\"{key_escaped}\"] = tex\n"));
    code.push_str(&format!("            parent:{}(tex)\n", setter_method));

    if let Some(file) = &texture.file {
        code.push_str(&format!(
            "            tex:SetTexture(\"{}\")\n",
            escape_lua_string(file)
        ));
    }
    if let Some(atlas) = &texture.atlas {
        let use_atlas_size = texture.use_atlas_size.unwrap_or(false);
        code.push_str(&format!(
            "            tex:SetAtlas(\"{}\", {})\n",
            escape_lua_string(atlas),
            use_atlas_size
        ));
    }

    if is_named {
        code.push_str(&format!("            _G[\"{}\"] = tex\n", escape_lua_string(tex_name)));
    }

    code.push_str("        end\n");
    code
}
