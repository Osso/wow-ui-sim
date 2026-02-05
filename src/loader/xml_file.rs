//! XML file loading and element processing.

use crate::lua_api::WowLuaEnv;
use crate::xml::{parse_xml_file, XmlElement};
use std::path::Path;
use std::time::Instant;

use super::addon::AddonContext;
use super::error::LoadError;
use super::helpers::resolve_path_with_fallback;
use super::lua_file::load_lua_file;
use super::xml_frame::create_frame_from_xml;
use super::LoadTiming;

/// Load an XML file, processing its elements.
/// Returns the number of Lua files loaded from Script elements.
pub fn load_xml_file(
    env: &WowLuaEnv,
    path: &Path,
    ctx: &AddonContext,
    timing: &mut LoadTiming,
) -> Result<usize, LoadError> {
    let xml_start = Instant::now();
    let ui = parse_xml_file(path)?;
    timing.xml_parse_time += xml_start.elapsed();

    let xml_dir = path.parent().unwrap_or(Path::new("."));
    let mut lua_count = 0;

    for element in &ui.elements {
        match element {
            XmlElement::Script(s) | XmlElement::ScriptLower(s) => {
                // Script can have file attribute or inline content
                if let Some(file) = &s.file {
                    let script_path = resolve_path_with_fallback(xml_dir, ctx.addon_root, file);
                    load_lua_file(env, &script_path, ctx, timing)?;
                    lua_count += 1;
                } else if let Some(inline) = &s.inline {
                    // Execute inline script with varargs
                    let table_clone = ctx.table.clone();
                    let lua_start = Instant::now();
                    env.exec_with_varargs(inline, "@inline", ctx.name, table_clone)
                        .map_err(|e| LoadError::Lua(e.to_string()))?;
                    timing.lua_exec_time += lua_start.elapsed();
                    lua_count += 1;
                }
            }
            XmlElement::Include(i) | XmlElement::IncludeLower(i) => {
                let include_path = resolve_path_with_fallback(xml_dir, ctx.addon_root, &i.file);
                // Check if it's a Lua file (some addons use Include for Lua files)
                if i.file.ends_with(".lua") {
                    load_lua_file(env, &include_path, ctx, timing)?;
                    lua_count += 1;
                } else {
                    lua_count += load_xml_file(env, &include_path, ctx, timing)?;
                }
            }
            XmlElement::Frame(f) => {
                create_frame_from_xml(env, f, "Frame", None)?;
            }
            XmlElement::Button(f) | XmlElement::ItemButton(f) => {
                create_frame_from_xml(env, f, "Button", None)?;
            }
            XmlElement::CheckButton(f) => {
                create_frame_from_xml(env, f, "CheckButton", None)?;
            }
            XmlElement::EditBox(f) => {
                create_frame_from_xml(env, f, "EditBox", None)?;
            }
            XmlElement::ScrollFrame(f) => {
                create_frame_from_xml(env, f, "ScrollFrame", None)?;
            }
            XmlElement::Slider(f) => {
                create_frame_from_xml(env, f, "Slider", None)?;
            }
            XmlElement::StatusBar(f) => {
                create_frame_from_xml(env, f, "StatusBar", None)?;
            }
            XmlElement::EventFrame(f) => {
                create_frame_from_xml(env, f, "Frame", None)?;
            }
            XmlElement::Texture(_) | XmlElement::FontString(_) => {
                // Top-level textures/fontstrings are templates
            }
            XmlElement::AnimationGroup(_) => {
                // Animation groups are templates
            }
            XmlElement::Actor(_) => {
                // Actor definitions for ModelScene
            }
            XmlElement::Font(font) => {
                // Font definitions - create font objects that can be referenced
                if let Some(name) = &font.name {
                    if !name.is_empty() {
                        let font_path = font.font.clone().unwrap_or_else(|| "Fonts/FRIZQT__.TTF".to_string());
                        // Escape backslashes for Lua string
                        let font_path_escaped = font_path.replace('\\', "/");
                        let font_height = font.height.unwrap_or(12.0);
                        let font_outline = font.outline.clone().unwrap_or_default();

                        // Create font object via Lua
                        let lua_code = format!(r#"
                            {name} = {{
                                __font = "{font_path}",
                                __height = {font_height},
                                __outline = "{font_outline}",
                                __r = 1.0,
                                __g = 1.0,
                                __b = 1.0,
                                SetTextColor = function(self, r, g, b)
                                    self.__r = r
                                    self.__g = g
                                    self.__b = b
                                end,
                                GetFont = function(self)
                                    return self.__font, self.__height, self.__outline
                                end,
                                SetFont = function(self, path, height, flags)
                                    self.__font = path
                                    if height then self.__height = height end
                                    if flags then self.__outline = flags end
                                end,
                                CopyFontObject = function(self, source)
                                    if source.__font then self.__font = source.__font end
                                    if source.__height then self.__height = source.__height end
                                    if source.__outline then self.__outline = source.__outline end
                                    if source.__r then self.__r = source.__r end
                                    if source.__g then self.__g = source.__g end
                                    if source.__b then self.__b = source.__b end
                                end,
                            }}
                        "#, name = name, font_path = font_path_escaped, font_height = font_height, font_outline = font_outline);

                        env.exec(&lua_code).map_err(|e| {
                            LoadError::Lua(format!("Failed to create font {}: {}", name, e))
                        })?;
                    }
                }
            }
            XmlElement::FontFamily(font_family) => {
                // FontFamily definitions - create font objects that can be referenced
                if let Some(name) = &font_family.name {
                    if !name.is_empty() {
                        // Create font object via Lua with default values
                        // FontFamily contains Member elements with Font children, but for simulation
                        // we just need a font object with the right methods
                        let lua_code = format!(r#"
                            {name} = {{
                                __font = "Fonts/FRIZQT__.TTF",
                                __height = 12.0,
                                __outline = "",
                                __r = 1.0,
                                __g = 1.0,
                                __b = 1.0,
                                __justifyH = "CENTER",
                                __justifyV = "MIDDLE",
                                SetTextColor = function(self, r, g, b)
                                    self.__r = r
                                    self.__g = g
                                    self.__b = b
                                end,
                                GetTextColor = function(self)
                                    return self.__r, self.__g, self.__b
                                end,
                                SetFont = function(self, font, height, flags)
                                    if font then self.__font = font end
                                    if height then self.__height = height end
                                    if flags then self.__outline = flags end
                                end,
                                GetFont = function(self)
                                    return self.__font, self.__height, self.__outline
                                end,
                                SetJustifyH = function(self, justify)
                                    self.__justifyH = justify
                                end,
                                GetJustifyH = function(self)
                                    return self.__justifyH
                                end,
                                SetJustifyV = function(self, justify)
                                    self.__justifyV = justify
                                end,
                                GetJustifyV = function(self)
                                    return self.__justifyV
                                end,
                                CopyFontObject = function(self, source)
                                    if source.__font then self.__font = source.__font end
                                    if source.__height then self.__height = source.__height end
                                    if source.__outline then self.__outline = source.__outline end
                                    if source.__r then self.__r = source.__r end
                                    if source.__g then self.__g = source.__g end
                                    if source.__b then self.__b = source.__b end
                                end,
                            }}
                        "#, name = name);

                        env.exec(&lua_code).map_err(|e| {
                            LoadError::Lua(format!("Failed to create font family {}: {}", name, e))
                        })?;
                    }
                }
            }
            XmlElement::Text(_) => {
                // Inline text content - ignored (comes from malformed XML or comments)
            }
            // Other frame types not yet fully supported - skip for now
            _ => {}
        }
    }

    Ok(lua_count)
}
