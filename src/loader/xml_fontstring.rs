//! FontString creation from XML definitions.

use crate::lua_api::WowLuaEnv;

use super::error::LoadError;
use super::helpers::{escape_lua_string, get_size_values, rand_id};

/// Create a fontstring from XML definition.
pub fn create_fontstring_from_xml(
    env: &WowLuaEnv,
    fontstring: &crate::xml::FontStringXml,
    parent_name: &str,
    draw_layer: &str,
) -> Result<(), LoadError> {
    // Skip virtual fontstrings
    if fontstring.is_virtual == Some(true) {
        return Ok(());
    }

    let fs_name = fontstring
        .name
        .clone()
        .map(|n| n.replace("$parent", parent_name))
        .unwrap_or_else(|| format!("__fs_{}", rand_id()));

    let inherits = fontstring.inherits.as_deref().unwrap_or("");

    let mut lua_code = format!(
        r#"
        local parent = {}
        local fs = parent:CreateFontString("{}", "{}", {})
        "#,
        parent_name,
        fs_name,
        draw_layer,
        if inherits.is_empty() {
            "nil".to_string()
        } else {
            format!("\"{}\"", inherits)
        }
    );

    // Resolve text: XML text attributes are localization keys looked up in global strings.
    let resolved_text = fontstring.text.as_ref().map(|key| {
        crate::global_strings::get_global_string(key)
            .map(|s| s.to_string())
            .unwrap_or_else(|| key.clone())
    });

    if let Some(text) = &resolved_text {
        lua_code.push_str(&format!(
            r#"
        fs:SetText("{}")
        "#,
            escape_lua_string(text)
        ));
    }

    // Set justification
    if let Some(justify_h) = &fontstring.justify_h {
        lua_code.push_str(&format!(
            r#"
        fs:SetJustifyH("{}")
        "#,
            justify_h
        ));
    }
    if let Some(justify_v) = &fontstring.justify_v {
        lua_code.push_str(&format!(
            r#"
        fs:SetJustifyV("{}")
        "#,
            justify_v
        ));
    }

    // Set text color
    if let Some(color) = &fontstring.color {
        let r = color.r.unwrap_or(1.0);
        let g = color.g.unwrap_or(1.0);
        let b = color.b.unwrap_or(1.0);
        let a = color.a.unwrap_or(1.0);
        lua_code.push_str(&format!(
            r#"
        fs:SetTextColor({}, {}, {}, {})
        "#,
            r, g, b, a
        ));
    }

    // Set size
    if let Some(size) = &fontstring.size {
        let (x, y) = get_size_values(size);
        if let (Some(x), Some(y)) = (x, y) {
            lua_code.push_str(&format!(
                r#"
        fs:SetSize({}, {})
        "#,
                x, y
            ));
        }
    }

    // Set anchors
    if let Some(anchors) = &fontstring.anchors {
        for anchor in &anchors.anchors {
            let point = &anchor.point;
            let relative_point = anchor.relative_point.as_deref().unwrap_or(point.as_str());

            let (x, y) = if let Some(offset) = &anchor.offset {
                if let Some(abs) = &offset.abs_dimension {
                    (abs.x.unwrap_or(0.0), abs.y.unwrap_or(0.0))
                } else {
                    (0.0, 0.0)
                }
            } else {
                (anchor.x.unwrap_or(0.0), anchor.y.unwrap_or(0.0))
            };

            // Handle relativeKey first, then relativeTo
            let rel = if let Some(key) = anchor.relative_key.as_deref() {
                // Handle $parent chains like "$parent.$parent.ScrollFrame"
                if key.contains("$parent") || key.contains("$Parent") {
                    let parts: Vec<&str> = key.split('.').collect();
                    let mut expr = String::new();
                    for part in parts {
                        if part == "$parent" || part == "$Parent" {
                            if expr.is_empty() {
                                expr = "parent".to_string();
                            } else {
                                expr = format!("{}:GetParent()", expr);
                            }
                        } else if !part.is_empty() {
                            expr = format!("{}[\"{}\"]", expr, part);
                        }
                    }
                    if expr.is_empty() { "parent".to_string() } else { expr }
                } else {
                    key.to_string()
                }
            } else {
                match anchor.relative_to.as_deref() {
                    Some("$parent") | None => "parent".to_string(),
                    Some(r) if r.contains("$parent") || r.contains("$Parent") => {
                        // Replace any $parent/$Parent pattern (e.g., $parent_Sibling, $parentColorSwatch)
                        r.replace("$parent", parent_name).replace("$Parent", parent_name)
                    }
                    Some(r) => r.to_string(),
                }
            };

            lua_code.push_str(&format!(
                r#"
        fs:SetPoint("{}", {}, "{}", {}, {})
        "#,
                point, rel, relative_point, x, y
            ));
        }
    }

    // Set setAllPoints
    if fontstring.set_all_points == Some(true) {
        lua_code.push_str(
            r#"
        fs:SetAllPoints(true)
        "#,
        );
    }

    // Set parentKey if specified
    if let Some(key) = &fontstring.parent_key {
        lua_code.push_str(&format!(
            r#"
        parent.{} = fs
        "#,
            key
        ));
    }

    env.exec(&lua_code).map_err(|e| {
        LoadError::Lua(format!(
            "Failed to create fontstring {} on {}: {}",
            fs_name, parent_name, e
        ))
    })?;

    // Set text and auto-size dimensions directly in Rust
    // This bypasses the Lua SetText method which doesn't trigger our Rust code
    // due to the fake metatable setup
    if let Some(text) = &resolved_text {
        let state = env.state();
        let mut state_ref = state.borrow_mut();
        if let Some(frame_id) = state_ref.widgets.get_id_by_name(&fs_name) {
            if let Some(frame) = state_ref.widgets.get_mut(frame_id) {
                frame.text = Some(text.clone());
                // Auto-size height to font size; width is measured by renderer
                if frame.height == 0.0 {
                    frame.height = frame.font_size.max(12.0);
                }
            }
        }
    }

    Ok(())
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
        create_fontstring_from_xml(&env, &fs, "TestFSParent", "ARTWORK").unwrap();

        // Verify Lua-side text is resolved
        let text: String = env.eval("return TestFSResolved:GetText()").unwrap();
        assert_eq!(text, "Load out of date AddOns");

        // Verify Rust-side text is resolved
        let state = env.state();
        let state_ref = state.borrow();
        let id = state_ref.widgets.get_id_by_name("TestFSResolved").unwrap();
        let frame = state_ref.widgets.get(id).unwrap();
        assert_eq!(frame.text.as_deref(), Some("Load out of date AddOns"));
    }
}
