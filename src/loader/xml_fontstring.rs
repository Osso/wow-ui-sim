//! FontString creation from XML definitions.

use crate::lua_api::LoaderEnv;

use super::error::LoadError;
use super::helpers::{escape_lua_string, generate_set_point_code, get_size_values, lua_global_ref, resolve_child_name, resolve_lua_escapes};

/// Resolve a text key through the global strings table.
fn resolve_fontstring_text(text_key: Option<&str>) -> Option<String> {
    text_key.map(|key| {
        crate::global_strings::get_global_string(key)
            .map(|s| resolve_lua_escapes(s))
            .unwrap_or_else(|| key.to_string())
    })
}

/// Generate Lua code for fontstring visual properties (justification, color, size, wrapping).
fn generate_fontstring_visual_code(fs: &crate::xml::FontStringXml) -> String {
    let mut code = String::new();

    if let Some(justify_h) = &fs.justify_h {
        code.push_str(&format!("\n        fs:SetJustifyH(\"{}\")\n        ", justify_h));
    }
    if let Some(justify_v) = &fs.justify_v {
        code.push_str(&format!("\n        fs:SetJustifyV(\"{}\")\n        ", justify_v));
    }

    if let Some(color) = &fs.color {
        code.push_str(&format!(
            "\n        fs:SetTextColor({}, {}, {}, {})\n        ",
            color.r.unwrap_or(1.0), color.g.unwrap_or(1.0),
            color.b.unwrap_or(1.0), color.a.unwrap_or(1.0)
        ));
    }

    if let Some(size) = fs.size.last() {
        let (x, y) = get_size_values(size);
        match (x, y) {
            (Some(x), Some(y)) => {
                code.push_str(&format!("\n        fs:SetSize({}, {})\n        ", x, y));
            }
            (Some(x), None) => {
                code.push_str(&format!("\n        fs:SetWidth({})\n        ", x));
            }
            (None, Some(y)) => {
                code.push_str(&format!("\n        fs:SetHeight({})\n        ", y));
            }
            _ => {}
        }
    }

    if fs.word_wrap == Some(false) {
        code.push_str("\n        fs:SetWordWrap(false)\n        ");
    }

    if let Some(max_lines) = fs.max_lines
        && max_lines > 0 {
            code.push_str(&format!("\n        fs:SetMaxLines({})\n        ", max_lines));
        }

    if fs.set_all_points == Some(true) {
        code.push_str("\n        fs:SetAllPoints(true)\n        ");
    }

    code
}

/// Generate Lua code for fontstring parent references (parentKey, parentArray).
fn generate_fontstring_parent_code(fs: &crate::xml::FontStringXml) -> String {
    let mut code = String::new();

    if let Some(key) = &fs.parent_key {
        code.push_str(&format!("\n        parent.{} = fs\n        ", key));
    }

    if let Some(parent_array) = &fs.parent_array {
        code.push_str(&format!(
            "\n        parent.{parent_array} = parent.{parent_array} or {{}}\n        \
             table.insert(parent.{parent_array}, fs)\n        ",
        ));
    }

    code
}

/// Sync fontstring text and auto-size height directly in Rust widget state.
fn sync_fontstring_text_to_rust(env: &LoaderEnv<'_>, fs_name: &str, text: &str) {
    let state = env.state();
    let mut state_ref = state.borrow_mut();
    if let Some(frame_id) = state_ref.widgets.get_id_by_name(fs_name)
        && let Some(frame) = state_ref.widgets.get_mut_visual(frame_id) {
            frame.text = Some(text.to_string());
            if frame.height == 0.0 {
                frame.height = frame.font_size.max(12.0);
            }
        }
}

/// Create a fontstring from XML definition.
pub fn create_fontstring_from_xml(
    env: &LoaderEnv<'_>,
    fontstring: &crate::xml::FontStringXml,
    parent_name: &str,
    draw_layer: &str,
    sub_level: i32,
) -> Result<(), LoadError> {
    if fontstring.is_virtual == Some(true) {
        return Ok(());
    }

    let fs_name = resolve_child_name(fontstring.name.as_deref(), parent_name, "__fs_");
    let resolved_text = resolve_fontstring_text(fontstring.text.as_deref());
    let lua_code = build_fontstring_lua(fontstring, parent_name, draw_layer, sub_level, &fs_name, &resolved_text);

    env.exec(&lua_code).map_err(|e| {
        LoadError::Lua(format!(
            "Failed to create fontstring {} on {}: {}",
            fs_name, parent_name, e
        ))
    })?;

    if let Some(text) = &resolved_text {
        sync_fontstring_text_to_rust(env, &fs_name, text);
    }

    Ok(())
}

/// Build the Lua code string that creates and configures a fontstring.
fn build_fontstring_lua(
    fontstring: &crate::xml::FontStringXml,
    parent_name: &str,
    draw_layer: &str,
    sub_level: i32,
    fs_name: &str,
    resolved_text: &Option<String>,
) -> String {
    let inherits = fontstring.inherits.as_deref().unwrap_or("");
    let mut code = format!(
        r#"
        local parent = {}
        local fs = parent:CreateFontString("{}", "{}", {})
        "#,
        lua_global_ref(parent_name),
        fs_name,
        draw_layer,
        if inherits.is_empty() { "nil".to_string() } else { format!("\"{}\"", inherits) }
    );
    if sub_level != 0 {
        code.push_str(&format!("\n        fs:SetDrawLayer(\"{}\", {})\n        ", draw_layer, sub_level));
    }
    if let Some(text) = resolved_text {
        code.push_str(&format!("\n        fs:SetText(\"{}\")\n        ", escape_lua_string(text)));
    }
    code.push_str(&generate_fontstring_visual_code(fontstring));
    code.push_str(&generate_fontstring_parent_code(fontstring));
    if let Some(anchors) = &fontstring.anchors {
        code.push_str(&generate_set_point_code(anchors, "fs", "parent", parent_name, "parent"));
    }
    if let Some(a) = fontstring.alpha {
        code.push_str(&format!("\n        fs:SetAlpha({})\n        ", a));
    }
    if fontstring.hidden == Some(true) {
        code.push_str("\n        fs:Hide()\n        ");
    }
    code
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
        create_fontstring_from_xml(&env.loader_env(), &fs, "TestFSParent", "ARTWORK", 0).unwrap();

        let text: String = env.eval("return TestFSResolved:GetText()").unwrap();
        assert_eq!(text, "Load out of date AddOns");

        let state = env.state();
        let state_ref = state.borrow();
        let id = state_ref.widgets.get_id_by_name("TestFSResolved").unwrap();
        let frame = state_ref.widgets.get(id).unwrap();
        assert_eq!(frame.text.as_deref(), Some("Load out of date AddOns"));
    }
}
