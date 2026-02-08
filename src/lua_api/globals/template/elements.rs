//! Template element creation: textures, fontstrings, thumb/button textures.

use crate::loader::helpers::{generate_animation_group_code, generate_set_point_code};
use mlua::Lua;

use super::{escape_lua_string, get_size_values, lua_global_ref, rand_id};

/// Create a texture from template XML.
pub(super) fn create_texture_from_template(
    lua: &Lua,
    texture: &crate::xml::TextureXml,
    parent_name: &str,
    draw_layer: &str,
    is_mask: bool,
) {
    let child_name = texture
        .name
        .as_ref()
        .map(|n| n.replace("$parent", parent_name))
        .unwrap_or_else(|| format!("__tex_{}", rand_id()));

    let mut code = format!(
        r#"
        local parent = {}
        if parent then
            local tex = parent:CreateTexture("{}", "{}")
        "#,
        lua_global_ref(parent_name), escape_lua_string(&child_name), draw_layer,
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

    let has_name = texture.name.is_some();
    // Set global reference for named textures, or temporarily for mask textures
    // so mark_mask_texture can look them up
    if has_name || is_mask {
        code.push_str(&format!("            _G[\"{}\"] = tex\n", escape_lua_string(&child_name)));
    }

    if texture.hidden == Some(true) {
        code.push_str("            tex:Hide()\n");
    }

    if let Some(ref mode) = texture.alpha_mode {
        code.push_str(&format!("            tex:SetBlendMode(\"{}\")\n", mode));
    }

    code.push_str("        end\n");
    let _ = lua.load(&code).exec();

    // Mark MaskTextures so the renderer skips them
    if is_mask {
        mark_mask_texture(lua, &child_name);
        // Remove temporary global for unnamed mask textures
        if !has_name {
            let _ = lua.globals().set(child_name.as_str(), mlua::Value::Nil);
        }
    }

    apply_texture_animations(lua, texture, &child_name);
}

/// Mark a texture widget as a MaskTexture (not rendered).
fn mark_mask_texture(lua: &Lua, name: &str) {
    // Extract the widget ID from the Lua global, then get SimState from UIParent
    let widget_id = lua.globals().get::<mlua::AnyUserData>(name).ok()
        .and_then(|ud| ud.borrow::<crate::lua_api::frame::FrameHandle>().ok().map(|h| h.id));
    let Some(id) = widget_id else {
        eprintln!("[mask] FAILED to find global for mask texture: {}", name);
        return;
    };
    eprintln!("[mask] Marking mask texture: {} (id={})", name, id);
    let Ok(parent_ud) = lua.globals().get::<mlua::AnyUserData>("UIParent") else { return };
    let Ok(handle) = parent_ud.borrow::<crate::lua_api::frame::FrameHandle>() else { return };
    if let Some(frame) = handle.state.borrow_mut().widgets.get_mut(id) {
        frame.is_mask = true;
    }
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
        code.push_str(&format!(
            "            {}:SetAtlas(\"{}\")\n",
            var,
            escape_lua_string(atlas)
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
        code.push_str(&format!("            {}.{} = {}\n", parent_var, parent_key, var));
    }
    if let Some(parent_array) = parent_array {
        code.push_str(&format!(
            "            {parent_var}.{parent_array} = {parent_var}.{parent_array} or {{}}\n\
             table.insert({parent_var}.{parent_array}, {var})\n"
        ));
    }
}

/// Create a fontstring from template XML.
pub(super) fn create_fontstring_from_template(
    lua: &Lua,
    fontstring: &crate::xml::FontStringXml,
    parent_name: &str,
    draw_layer: &str,
) {
    let child_name = fontstring
        .name
        .as_ref()
        .map(|n| n.replace("$parent", parent_name))
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
        let resolved = crate::global_strings::get_global_string(text_key)
            .unwrap_or(text_key.as_str());
        code.push_str(&format!(
            "            fs:SetText(\"{}\")\n",
            escape_lua_string(resolved)
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
) {
    let child_name = bar
        .name
        .as_ref()
        .map(|n| n.replace("$parent", parent_name))
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
    code.push_str(&format!("            parent.{} = bar\n", parent_key));

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
) {
    let child_name = thumb
        .name
        .as_ref()
        .map(|n| n.replace("$parent", parent_name))
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
        code.push_str(&format!("            parent.{} = thumb\n", parent_key));
    } else {
        code.push_str("            parent.ThumbTexture = thumb\n");
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
        .map(|n| n.replace("$parent", parent_name));

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
    let mut code = format!(
        r#"
        local parent = {}
        if parent and parent.{} then
            local tex = parent.{}
            if tex == nil then
                tex = parent:CreateTexture("{}", "ARTWORK")
            end
        "#,
        lua_global_ref(parent_name), setter_method, actual_parent_key, escape_lua_string(tex_name),
    );

    if let Some(size) = &texture.size {
        let (width, height) = get_size_values(size);
        if let (Some(w), Some(h)) = (width, height) {
            code.push_str(&format!("            tex:SetSize({}, {})\n", w, h));
        }
    }

    code.push_str(&format!("            parent.{} = tex\n", actual_parent_key));
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
