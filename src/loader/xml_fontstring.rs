//! FontString creation from XML definitions.

use super::error::LoadError;
use super::helpers::{
    escape_lua_string, generate_scripts_code_for_target, generate_set_point_code, get_size_values,
    lua_global_ref, lua_table_field_ref, resolve_child_name, resolve_lua_escapes,
};
use crate::lua_api::LoaderEnv;

/// Resolve a text key through the global strings table.
pub(super) fn resolve_fontstring_text(text_key: Option<&str>) -> Option<String> {
    text_key.map(|key| {
        crate::global_strings::get_global_string(key)
            .map(|s| resolve_lua_escapes(s).into_owned())
            .unwrap_or_else(|| key.to_string())
    })
}

/// Generate Lua code for fontstring visual properties (justification, color, size, wrapping).
fn generate_fontstring_visual_code(fs: &crate::xml::FontStringXml) -> String {
    let mut code = String::new();
    generate_fontstring_justify_color(&mut code, fs);
    generate_fontstring_size_and_flags(&mut code, fs);
    code
}

fn generate_fontstring_justify_color(code: &mut String, fs: &crate::xml::FontStringXml) {
    if let Some(justify_h) = &fs.justify_h {
        code.push_str(&format!(
            "\n        fs:SetJustifyH(\"{}\")\n        ",
            justify_h
        ));
    }
    if let Some(justify_v) = &fs.justify_v {
        code.push_str(&format!(
            "\n        fs:SetJustifyV(\"{}\")\n        ",
            justify_v
        ));
    }
    if let Some(color) = &fs.color {
        if let Some(named) = &color.color {
            // <Color color="NORMAL_FONT_COLOR"/>: a colour object global.
            code.push_str(&format!(
                "\n        do local c = _G[\"{named}\"]; if c and c.GetRGBA then fs:SetTextColor(c:GetRGBA()) end end\n        "
            ));
        } else {
            code.push_str(&format!(
                "\n        fs:SetTextColor({}, {}, {}, {})\n        ",
                color.r.unwrap_or(1.0),
                color.g.unwrap_or(1.0),
                color.b.unwrap_or(1.0),
                color.a.unwrap_or(1.0)
            ));
        }
    }
}

fn generate_fontstring_size_and_flags(code: &mut String, fs: &crate::xml::FontStringXml) {
    if let Some(size) = fs.size.last() {
        let (x, y) = get_size_values(size);
        match (x, y) {
            (Some(x), Some(y)) => {
                code.push_str(&format!("\n        fs:SetSize({}, {})\n        ", x, y))
            }
            (Some(x), None) => code.push_str(&format!("\n        fs:SetWidth({})\n        ", x)),
            (None, Some(y)) => code.push_str(&format!("\n        fs:SetHeight({})\n        ", y)),
            _ => {}
        }
    }
    if let Some(h) = fs.font_height.as_ref().and_then(|fh| fh.value()) {
        code.push_str(&format!(
            "\n        do local f,_,fl = fs:GetFont(); if f then fs:SetFont(f, {h}, fl) end end\n        "
        ));
    }
    if fs.word_wrap == Some(false) {
        code.push_str("\n        fs:SetWordWrap(false)\n        ");
    }
    if let Some(max_lines) = fs.max_lines
        && max_lines > 0
    {
        code.push_str(&format!(
            "\n        fs:SetMaxLines({})\n        ",
            max_lines
        ));
    }
    if fs.set_all_points == Some(true) {
        code.push_str("\n        fs:SetAllPoints(true)\n        ");
    }
}

/// Generate Lua code for fontstring parent references (parentKey, parentArray).
fn generate_fontstring_parent_code(fs: &crate::xml::FontStringXml) -> String {
    let mut code = String::new();

    if let Some(key) = &fs.parent_key {
        let parent_field = lua_table_field_ref("parent", key);
        code.push_str(&format!("\n        {parent_field} = fs\n        "));
    }

    if let Some(parent_array) = &fs.parent_array {
        let array_ref = lua_table_field_ref("parent", parent_array);
        code.push_str(&format!(
            "\n        {array_ref} = {array_ref} or {{}}\n        \
             table.insert({array_ref}, fs)\n        ",
        ));
    }

    code
}

/// Sync fontstring text directly in Rust widget state.
/// Height/width auto-sizing is handled by the Lua SetText path.
pub(super) fn sync_fontstring_text_to_rust(env: &LoaderEnv<'_>, fs_name: &str, text: &str) {
    let state = env.state();
    let mut state_ref = state.borrow_mut();
    if let Some(frame_id) = state_ref.widgets.get_id_by_name(fs_name)
        && let Some(frame) = state_ref.widgets.get_mut_visual(frame_id)
    {
        frame.text_stripped = Some(crate::render::strip_wow_markup(text));
        frame.text = Some(text.to_string());
    }
}

/// Generate the CreateFontString call and draw layer setup.
fn build_fontstring_create_code(
    fontstring: &crate::xml::FontStringXml,
    parent_ref_name: &str,
    draw_layer: &str,
    sub_level: i32,
    fs_name: &str,
) -> String {
    let inherits = fontstring.inherits.as_deref().unwrap_or("");
    let mut code = format!(
        r#"
        local parent = {}
        local fs = parent:CreateFontString("{}", "{}", {})
        "#,
        lua_global_ref(parent_ref_name),
        escape_lua_string(fs_name),
        escape_lua_string(draw_layer),
        if inherits.is_empty() {
            "nil".to_string()
        } else {
            format!("\"{}\"", escape_lua_string(inherits))
        }
    );
    if sub_level != 0 {
        code.push_str(&format!(
            "\n        fs:SetDrawLayer(\"{}\", {})\n        ",
            draw_layer, sub_level
        ));
    }
    code
}

fn generate_fontstring_mixin_code(fontstring: &crate::xml::FontStringXml) -> String {
    let mixins = crate::xml::collect_font_string_mixins(
        fontstring.inherits.as_deref(),
        fontstring.mixin.as_deref(),
    );
    if mixins.is_empty() {
        return String::new();
    }

    let mut code = String::new();
    for mixin in mixins {
        code.push_str(&format!(
            "\n        if {mixin} then Mixin(fs, {mixin}) end\n        "
        ));
    }
    code
}

/// Generate Lua code for fontstring text, anchors, alpha, and visibility.
fn build_fontstring_extra_code(
    fontstring: &crate::xml::FontStringXml,
    parent_name: &str,
    resolved_text: &Option<String>,
) -> String {
    let mut code = String::new();
    append_fontstring_font_code(&mut code, fontstring);
    append_fontstring_text_code(&mut code, resolved_text);
    code.push_str(&generate_fontstring_visual_code(fontstring));
    code.push_str(&generate_fontstring_parent_code(fontstring));
    append_fontstring_anchor_code(&mut code, fontstring, parent_name);
    append_fontstring_key_values_code(&mut code, fontstring);
    append_fontstring_scripts_code(&mut code, fontstring);
    append_fontstring_alpha_visibility_code(&mut code, fontstring);
    code
}

fn append_fontstring_font_code(code: &mut String, fontstring: &crate::xml::FontStringXml) {
    if let Some(font) = &fontstring.font {
        code.push_str(&format!(
            "\n        fs:SetFontObject(\"{}\")\n        ",
            escape_lua_string(font)
        ));
    }
}

fn append_fontstring_text_code(code: &mut String, resolved_text: &Option<String>) {
    if let Some(text) = resolved_text {
        code.push_str(&format!(
            "\n        fs:SetText(\"{}\")\n        ",
            escape_lua_string(text)
        ));
    }
}

fn append_fontstring_anchor_code(
    code: &mut String,
    fontstring: &crate::xml::FontStringXml,
    parent_name: &str,
) {
    if let Some(anchors) = &fontstring.anchors {
        code.push_str(&generate_set_point_code(
            anchors,
            "fs",
            "parent",
            parent_name,
            "parent",
        ));
    } else if fontstring.set_all_points != Some(true) {
        let default_point = default_fontstring_anchor_point(fontstring);
        code.push_str(&format!(
            "\n        fs:SetPoint(\"{default_point}\", parent, \"{default_point}\", 0, 0)\n        "
        ));
    }
}

fn default_fontstring_anchor_point(fontstring: &crate::xml::FontStringXml) -> &str {
    match fontstring.justify_h.as_deref() {
        Some("LEFT") => "LEFT",
        Some("RIGHT") => "RIGHT",
        _ => "CENTER",
    }
}

fn append_fontstring_key_values_code(code: &mut String, fontstring: &crate::xml::FontStringXml) {
    code.push_str(&super::xml_frame_codegen::generate_key_values_code(
        fontstring.key_values.as_ref(),
        "fs",
    ));
}

fn append_fontstring_scripts_code(code: &mut String, fontstring: &crate::xml::FontStringXml) {
    if let Some(scripts) = &fontstring.scripts {
        code.push_str(&generate_scripts_code_for_target("fs", scripts));
        if fontstring_onload_should_fire_immediately(scripts) {
            code.push_str(IMMEDIATE_FONTSTRING_ONLOAD_LUA);
        }
    }
}

const IMMEDIATE_FONTSTRING_ONLOAD_LUA: &str = r#"
        do
            local __onload = fs:GetScript("OnLoad")
            if __onload then
                local __ok, __err = pcall(__onload, fs)
                if not __ok then
                    local __report = debug.getregistry()["__report_script_error"]
                    if __report then
                        local __name = fs.GetName and fs:GetName() or "?"
                        __report("[OnLoad] " .. tostring(__name) .. ": " .. tostring(__err))
                    else
                        error(__err)
                    end
                end
            end
        end
        "#;

fn append_fontstring_alpha_visibility_code(
    code: &mut String,
    fontstring: &crate::xml::FontStringXml,
) {
    if let Some(a) = fontstring.alpha {
        code.push_str(&format!("\n        fs:SetAlpha({})\n        ", a));
    }
    if fontstring.hidden == Some(true) {
        code.push_str("\n        fs:Hide()\n        ");
    }
}

fn fontstring_onload_should_fire_immediately(scripts: &crate::xml::ScriptsXml) -> bool {
    let Some(script) = scripts.on_load.last() else {
        return false;
    };

    let has_function = script
        .function
        .as_ref()
        .is_some_and(|name| !name.is_empty());
    let has_inline_body = script
        .body
        .as_ref()
        .is_some_and(|body| !body.trim().is_empty());
    has_function || has_inline_body
}

/// Build the Lua code string that creates and configures a fontstring.
pub(super) fn build_fontstring_lua(
    fontstring: &crate::xml::FontStringXml,
    parent_ref_name: &str,
    subst_parent_name: &str,
    draw_layer: &str,
    sub_level: i32,
    fs_name: &str,
    resolved_text: &Option<String>,
) -> String {
    let mut code =
        build_fontstring_create_code(fontstring, parent_ref_name, draw_layer, sub_level, fs_name);
    code.push_str(&generate_fontstring_mixin_code(fontstring));
    code.push_str(&build_fontstring_extra_code(
        fontstring,
        subst_parent_name,
        resolved_text,
    ));
    code
}

/// Create a fontstring from XML definition.
pub fn create_fontstring_from_xml_with_ref(
    env: &LoaderEnv<'_>,
    fontstring: &crate::xml::FontStringXml,
    parent_name: &str,
    parent_ref_name: &str,
    draw_layer: &str,
    sub_level: i32,
) -> Result<(), LoadError> {
    if fontstring.is_virtual == Some(true) {
        return Ok(());
    }

    let fs_name = resolve_child_name(fontstring.name.as_deref(), parent_name, "__fs_");
    let resolved_text = resolve_fontstring_text(fontstring.text.as_deref());
    let lua_code = build_fontstring_lua(
        fontstring,
        parent_ref_name,
        parent_name,
        draw_layer,
        sub_level,
        &fs_name,
        &resolved_text,
    );

    env.exec(&lua_code).map_err(|e| {
        LoadError::Lua(format!(
            "Failed to create fontstring {} on {}: {}",
            fs_name, parent_name, e
        ))
    })?;

    sync_fontstring_child_to_rilua(env, parent_name, &fs_name, fontstring.parent_key.as_deref())?;

    if let Some(text) = &resolved_text {
        sync_fontstring_text_to_rust(env, &fs_name, text);
    }

    Ok(())
}

fn sync_fontstring_child_to_rilua(
    env: &LoaderEnv<'_>,
    parent_name: &str,
    fs_name: &str,
    parent_key: Option<&str>,
) -> Result<(), LoadError> {
    let Some(parent_key) = parent_key else {
        return Ok(());
    };
    let (parent_id, child_id) = {
        let sim = env.state().borrow();
        let Some(parent_id) = sim.widgets.get_id_by_name(parent_name) else {
            return Ok(());
        };
        let Some(child_id) = sim.widgets.get_id_by_name(fs_name) else {
            return Ok(());
        };
        (parent_id, child_id)
    };
    env.with_state(|state| {
        crate::lua_api::globals::template::assign_parent_key(state, parent_id, parent_key, child_id)
            .map_err(|e| crate::Error::Other(e.to_string()))
    })
    .map_err(|e| LoadError::Lua(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua_api::WowLuaEnv;
    use crate::xml::FontStringXml;

    #[test]
    fn xml_fontstring_text_resolves_global_string_key() {
        let env = WowLuaEnv::new().unwrap();
        env.exec(r#"CreateFrame("Frame", "TestFSParent", UIParent)"#)
            .unwrap();

        let fs = FontStringXml {
            name: Some("TestFSResolved".to_string()),
            text: Some("ADDON_FORCE_LOAD".to_string()),
            ..Default::default()
        };
        create_fontstring_from_xml_with_ref(
            &env.loader_env(),
            &fs,
            "TestFSParent",
            "TestFSParent",
            "ARTWORK",
            0,
        )
        .unwrap();

        let text: String = env.eval("return TestFSResolved:GetText()").unwrap();
        assert_eq!(text, "Load out of date AddOns");

        let state = env.state();
        let state_ref = state.borrow();
        let id = state_ref.widgets.get_id_by_name("TestFSResolved").unwrap();
        let frame = state_ref.widgets.get(id).unwrap();
        assert_eq!(frame.text.as_deref(), Some("Load out of date AddOns"));
    }

    #[test]
    fn fontstring_create_code_escapes_generated_child_names() {
        let code = build_fontstring_create_code(
            &FontStringXml::default(),
            "UIParent",
            "ARTWORK",
            0,
            r#"Parent-|TInterface\AddOns\Addon\Icon:0|t.__fs_1"#,
        );

        assert!(
            code.contains(
                r#"CreateFontString("Parent-|TInterface\\AddOns\\Addon\\Icon:0|t.__fs_1""#
            )
        );
    }

    #[test]
    fn xml_fontstring_direct_mixin_applies_to_created_fontstring() {
        let env = WowLuaEnv::new().unwrap();
        env.exec(
            r#"
            CreateFrame("Frame", "TestFSMixinParent", UIParent)
            TestFontStringMixin = {}
            function TestFontStringMixin:Describe()
                return "mixed"
            end
            "#,
        )
        .unwrap();

        let fs = FontStringXml {
            name: Some("TestFSMixed".to_string()),
            mixin: Some("TestFontStringMixin".to_string()),
            ..Default::default()
        };
        create_fontstring_from_xml_with_ref(
            &env.loader_env(),
            &fs,
            "TestFSMixinParent",
            "TestFSMixinParent",
            "ARTWORK",
            0,
        )
        .unwrap();

        let result: String = env.eval("return TestFSMixed:Describe()").unwrap();
        assert_eq!(result, "mixed");
    }
}
