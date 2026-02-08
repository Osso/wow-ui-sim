//! XML file loading and element processing.

use crate::lua_api::LoaderEnv;
use crate::xml::{parse_xml_file, FrameXml, XmlElement};
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
    env: &LoaderEnv<'_>,
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
        lua_count += process_element(env, element, xml_dir, ctx, timing)?;
    }

    Ok(lua_count)
}

/// Process a single top-level XML element.
/// Returns the number of Lua files loaded (0 or 1, or recursive count for includes).
fn process_element(
    env: &LoaderEnv<'_>,
    element: &XmlElement,
    xml_dir: &Path,
    ctx: &AddonContext,
    timing: &mut LoadTiming,
) -> Result<usize, LoadError> {
    match element {
        XmlElement::Script(s) | XmlElement::ScriptLower(s) => {
            process_script(env, s, xml_dir, ctx, timing)
        }
        XmlElement::Include(i) | XmlElement::IncludeLower(i) => {
            process_include(env, i, xml_dir, ctx, timing)
        }
        XmlElement::Font(font) => {
            create_font_object(env, font)?;
            Ok(0)
        }
        XmlElement::FontFamily(font_family) => {
            create_font_family_object(env, font_family)?;
            Ok(0)
        }
        XmlElement::ScopedModifier(scoped) => {
            let mut count = 0;
            for child in &scoped.elements {
                count += process_element(env, child, xml_dir, ctx, timing)?;
            }
            Ok(count)
        }
        _ => {
            process_frame_element(env, element)?;
            Ok(0)
        }
    }
}

/// Process a Script element (file reference or inline code).
fn process_script(
    env: &LoaderEnv<'_>,
    s: &crate::xml::ScriptXml,
    xml_dir: &Path,
    ctx: &AddonContext,
    timing: &mut LoadTiming,
) -> Result<usize, LoadError> {
    if let Some(file) = &s.file {
        let script_path = resolve_path_with_fallback(xml_dir, ctx.addon_root, file);
        load_lua_file(env, &script_path, ctx, timing)?;
        Ok(1)
    } else if let Some(inline) = &s.inline {
        let table_clone = ctx.table.clone();
        let lua_start = Instant::now();
        env.exec_with_varargs(inline, "@inline", ctx.name, table_clone)
            .map_err(|e| LoadError::Lua(e.to_string()))?;
        timing.lua_exec_time += lua_start.elapsed();
        Ok(1)
    } else {
        Ok(0)
    }
}

/// Process an Include element (XML or Lua file).
fn process_include(
    env: &LoaderEnv<'_>,
    i: &crate::xml::IncludeXml,
    xml_dir: &Path,
    ctx: &AddonContext,
    timing: &mut LoadTiming,
) -> Result<usize, LoadError> {
    let include_path = resolve_path_with_fallback(xml_dir, ctx.addon_root, &i.file);
    if i.file.ends_with(".lua") {
        load_lua_file(env, &include_path, ctx, timing)?;
        Ok(1)
    } else {
        load_xml_file(env, &include_path, ctx, timing)
    }
}

/// Extract the FrameXml data and widget type string from an XmlElement.
fn resolve_frame_element(element: &XmlElement) -> Option<(&FrameXml, &'static str)> {
    match element {
        XmlElement::Frame(f) => Some((f, "Frame")),
        XmlElement::Button(f)
        | XmlElement::ItemButton(f)
        | XmlElement::DropDownToggleButton(f)
        | XmlElement::DropdownButton(f)
        | XmlElement::EventButton(f) => Some((f, "Button")),
        XmlElement::CheckButton(f) => Some((f, "CheckButton")),
        XmlElement::EditBox(f)
        | XmlElement::EventEditBox(f) => Some((f, "EditBox")),
        XmlElement::ScrollFrame(f)
        | XmlElement::EventScrollFrame(f) => Some((f, "ScrollFrame")),
        XmlElement::Slider(f) => Some((f, "Slider")),
        XmlElement::StatusBar(f) => Some((f, "StatusBar")),
        XmlElement::Cooldown(f) => Some((f, "Cooldown")),
        XmlElement::GameTooltip(f) => Some((f, "GameTooltip")),
        XmlElement::ColorSelect(f) => Some((f, "ColorSelect")),
        XmlElement::Model(f)
        | XmlElement::DressUpModel(f) => Some((f, "Model")),
        XmlElement::ModelScene(f) => Some((f, "ModelScene")),
        XmlElement::PlayerModel(f)
        | XmlElement::CinematicModel(f) => Some((f, "PlayerModel")),
        XmlElement::MessageFrame(f)
        | XmlElement::ScrollingMessageFrame(f) => Some((f, "MessageFrame")),
        XmlElement::SimpleHTML(f) => Some((f, "SimpleHTML")),
        XmlElement::EventFrame(f)
        | XmlElement::TaxiRouteFrame(f)
        | XmlElement::ModelFFX(f)
        | XmlElement::TabardModel(f)
        | XmlElement::UiCamera(f)
        | XmlElement::UnitPositionFrame(f)
        | XmlElement::OffScreenFrame(f)
        | XmlElement::Checkout(f)
        | XmlElement::FogOfWarFrame(f)
        | XmlElement::QuestPOIFrame(f)
        | XmlElement::ArchaeologyDigSiteFrame(f)
        | XmlElement::ScenarioPOIFrame(f)
        | XmlElement::UIThemeContainerFrame(f)
        | XmlElement::ContainedAlertFrame(f)
        | XmlElement::MapScene(f)
        | XmlElement::Line(f)
        | XmlElement::Browser(f)
        | XmlElement::MovieFrame(f)
        | XmlElement::WorldFrame(f) => Some((f, "Frame")),
        XmlElement::Minimap(f) => Some((f, "Minimap")),
        _ => None,
    }
}

/// Process a frame-type XML element by dispatching to create_frame_from_xml.
fn process_frame_element(env: &LoaderEnv<'_>, element: &XmlElement) -> Result<(), LoadError> {
    if let Some((frame_xml, widget_type)) = resolve_frame_element(element) {
        create_frame_from_xml(env, frame_xml, widget_type, None)?;
    }
    Ok(())
}

/// Lua template for Font objects. Placeholders: {name}, {font_path}, {font_height},
/// {font_outline}, {justify_h}, {justify_v}.
const FONT_LUA_TEMPLATE: &str = r#"
{name} = {
    __font = "{font_path}",
    __height = {font_height},
    __outline = "{font_outline}",
    __r = 1.0, __g = 1.0, __b = 1.0,
    __justifyH = "{justify_h}",
    __justifyV = "{justify_v}",
    SetTextColor = function(self, r, g, b)
        self.__r = r; self.__g = g; self.__b = b
    end,
    GetFont = function(self)
        return self.__font, self.__height, self.__outline
    end,
    SetFont = function(self, path, height, flags)
        self.__font = path
        if height then self.__height = height end
        if flags then self.__outline = flags end
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
        if source.__justifyH then self.__justifyH = source.__justifyH end
        if source.__justifyV then self.__justifyV = source.__justifyV end
    end,
}
"#;

/// Create a Font object in Lua from XML definition.
fn create_font_object(
    env: &LoaderEnv<'_>,
    font: &crate::xml::FontXml,
) -> Result<(), LoadError> {
    let Some(name) = &font.name else { return Ok(()) };
    if name.is_empty() {
        return Ok(());
    }

    let font_path = font.font.as_deref().unwrap_or("Fonts/FRIZQT__.TTF").replace('\\', "/");
    let lua_code = FONT_LUA_TEMPLATE
        .replace("{name}", name)
        .replace("{font_path}", &font_path)
        .replace("{font_height}", &font.height.unwrap_or(12.0).to_string())
        .replace("{font_outline}", font.outline.as_deref().unwrap_or(""))
        .replace("{justify_h}", font.justify_h.as_deref().unwrap_or("CENTER"))
        .replace("{justify_v}", font.justify_v.as_deref().unwrap_or("MIDDLE"));

    env.exec(&lua_code).map_err(|e| {
        LoadError::Lua(format!("Failed to create font {}: {}", name, e))
    })
}

/// Create a FontFamily object in Lua from XML definition.
const FONT_FAMILY_LUA_TEMPLATE: &str = r#"
{name} = {
    __font = "Fonts/FRIZQT__.TTF",
    __height = 12.0,
    __outline = "",
    __r = 1.0, __g = 1.0, __b = 1.0,
    __justifyH = "CENTER",
    __justifyV = "MIDDLE",
    SetTextColor = function(self, r, g, b)
        self.__r = r; self.__g = g; self.__b = b
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
}
"#;

fn create_font_family_object(
    env: &LoaderEnv<'_>,
    font_family: &crate::xml::FontFamilyXml,
) -> Result<(), LoadError> {
    let Some(name) = &font_family.name else {
        return Ok(());
    };
    if name.is_empty() {
        return Ok(());
    }
    let lua_code = FONT_FAMILY_LUA_TEMPLATE.replace("{name}", name);
    env.exec(&lua_code).map_err(|e| {
        LoadError::Lua(format!("Failed to create font family {}: {}", name, e))
    })
}
