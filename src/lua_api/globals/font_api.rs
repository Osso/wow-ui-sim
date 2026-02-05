//! Font API functions for WoW UI simulation.
//!
//! Contains CreateFont, CreateFontFamily, GetFonts, GetFontInfo, and
//! standard WoW font object creation.

use mlua::{Lua, Result, Value};

/// Register all font API functions.
pub fn register_font_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // CreateFont(name) - Creates a named font object that can be used with SetFontObject
    let create_font = lua.create_function(|lua, name: Option<String>| {
        // Create a font object table with font properties and methods
        let font = lua.create_table()?;

        // Internal state for the font
        font.set("__fontPath", "Fonts\\FRIZQT__.TTF")?;
        font.set("__fontHeight", 12.0)?;
        font.set("__fontFlags", "")?;
        font.set("__textColorR", 1.0)?;
        font.set("__textColorG", 1.0)?;
        font.set("__textColorB", 1.0)?;
        font.set("__textColorA", 1.0)?;
        font.set("__shadowColorR", 0.0)?;
        font.set("__shadowColorG", 0.0)?;
        font.set("__shadowColorB", 0.0)?;
        font.set("__shadowColorA", 0.0)?;
        font.set("__shadowOffsetX", 0.0)?;
        font.set("__shadowOffsetY", 0.0)?;
        font.set("__justifyH", "CENTER")?;
        font.set("__justifyV", "MIDDLE")?;

        // SetFont(fontPath, height, flags)
        font.set(
            "SetFont",
            lua.create_function(
                |_, (this, path, height, flags): (mlua::Table, String, f64, Option<String>)| {
                    this.set("__fontPath", path)?;
                    this.set("__fontHeight", height)?;
                    this.set("__fontFlags", flags.unwrap_or_default())?;
                    Ok(())
                },
            )?,
        )?;

        // GetFont() -> fontPath, height, flags
        font.set(
            "GetFont",
            lua.create_function(|_, this: mlua::Table| {
                let path: String = this.get("__fontPath")?;
                let height: f64 = this.get("__fontHeight")?;
                let flags: String = this.get("__fontFlags")?;
                Ok((path, height, flags))
            })?,
        )?;

        // SetTextColor(r, g, b, a)
        font.set(
            "SetTextColor",
            lua.create_function(
                |_, (this, r, g, b, a): (mlua::Table, f64, f64, f64, Option<f64>)| {
                    this.set("__textColorR", r)?;
                    this.set("__textColorG", g)?;
                    this.set("__textColorB", b)?;
                    this.set("__textColorA", a.unwrap_or(1.0))?;
                    Ok(())
                },
            )?,
        )?;

        // GetTextColor() -> r, g, b, a
        font.set(
            "GetTextColor",
            lua.create_function(|_, this: mlua::Table| {
                let r: f64 = this.get("__textColorR")?;
                let g: f64 = this.get("__textColorG")?;
                let b: f64 = this.get("__textColorB")?;
                let a: f64 = this.get("__textColorA")?;
                Ok((r, g, b, a))
            })?,
        )?;

        // SetShadowColor(r, g, b, a)
        font.set(
            "SetShadowColor",
            lua.create_function(
                |_, (this, r, g, b, a): (mlua::Table, f64, f64, f64, Option<f64>)| {
                    this.set("__shadowColorR", r)?;
                    this.set("__shadowColorG", g)?;
                    this.set("__shadowColorB", b)?;
                    this.set("__shadowColorA", a.unwrap_or(1.0))?;
                    Ok(())
                },
            )?,
        )?;

        // GetShadowColor() -> r, g, b, a
        font.set(
            "GetShadowColor",
            lua.create_function(|_, this: mlua::Table| {
                let r: f64 = this.get("__shadowColorR")?;
                let g: f64 = this.get("__shadowColorG")?;
                let b: f64 = this.get("__shadowColorB")?;
                let a: f64 = this.get("__shadowColorA")?;
                Ok((r, g, b, a))
            })?,
        )?;

        // SetShadowOffset(x, y)
        font.set(
            "SetShadowOffset",
            lua.create_function(|_, (this, x, y): (mlua::Table, f64, f64)| {
                this.set("__shadowOffsetX", x)?;
                this.set("__shadowOffsetY", y)?;
                Ok(())
            })?,
        )?;

        // GetShadowOffset() -> x, y
        font.set(
            "GetShadowOffset",
            lua.create_function(|_, this: mlua::Table| {
                let x: f64 = this.get("__shadowOffsetX")?;
                let y: f64 = this.get("__shadowOffsetY")?;
                Ok((x, y))
            })?,
        )?;

        // SetJustifyH(justify)
        font.set(
            "SetJustifyH",
            lua.create_function(|_, (this, justify): (mlua::Table, String)| {
                this.set("__justifyH", justify)?;
                Ok(())
            })?,
        )?;

        // GetJustifyH() -> justify
        font.set(
            "GetJustifyH",
            lua.create_function(|_, this: mlua::Table| {
                let justify: String = this.get("__justifyH")?;
                Ok(justify)
            })?,
        )?;

        // SetJustifyV(justify)
        font.set(
            "SetJustifyV",
            lua.create_function(|_, (this, justify): (mlua::Table, String)| {
                this.set("__justifyV", justify)?;
                Ok(())
            })?,
        )?;

        // GetJustifyV() -> justify
        font.set(
            "GetJustifyV",
            lua.create_function(|_, this: mlua::Table| {
                let justify: String = this.get("__justifyV")?;
                Ok(justify)
            })?,
        )?;

        // SetSpacing(spacing)
        font.set(
            "SetSpacing",
            lua.create_function(|_, (this, spacing): (mlua::Table, f64)| {
                this.set("__spacing", spacing)?;
                Ok(())
            })?,
        )?;

        // GetSpacing() -> spacing
        font.set(
            "GetSpacing",
            lua.create_function(|_, this: mlua::Table| {
                let spacing: f64 = this.get("__spacing").unwrap_or(0.0);
                Ok(spacing)
            })?,
        )?;

        // CopyFontObject(fontObject or fontName)
        font.set(
            "CopyFontObject",
            lua.create_function(|lua, (this, src): (mlua::Table, Value)| {
                // If src is a string, look up the font object by name
                let src_table: Option<mlua::Table> = match src {
                    Value::String(name) => {
                        let name_str = name.to_string_lossy().to_string();
                        lua.globals()
                            .get::<Option<mlua::Table>>(name_str)
                            .ok()
                            .flatten()
                    }
                    Value::Table(t) => Some(t),
                    _ => None,
                };

                if let Some(src) = src_table {
                    // Copy all font properties from src to this
                    if let Ok(v) = src.get::<String>("__fontPath") {
                        this.set("__fontPath", v)?;
                    }
                    if let Ok(v) = src.get::<f64>("__fontHeight") {
                        this.set("__fontHeight", v)?;
                    }
                    if let Ok(v) = src.get::<String>("__fontFlags") {
                        this.set("__fontFlags", v)?;
                    }
                    for key in &[
                        "__textColorR",
                        "__textColorG",
                        "__textColorB",
                        "__textColorA",
                        "__shadowColorR",
                        "__shadowColorG",
                        "__shadowColorB",
                        "__shadowColorA",
                        "__shadowOffsetX",
                        "__shadowOffsetY",
                    ] {
                        if let Ok(v) = src.get::<f64>(*key) {
                            this.set(*key, v)?;
                        }
                    }
                    if let Ok(v) = src.get::<String>("__justifyH") {
                        this.set("__justifyH", v)?;
                    }
                    if let Ok(v) = src.get::<String>("__justifyV") {
                        this.set("__justifyV", v)?;
                    }
                }
                Ok(())
            })?,
        )?;

        // GetName() -> name
        font.set("__name", name.clone())?;
        font.set(
            "GetName",
            lua.create_function(|_, this: mlua::Table| {
                let name: Option<String> = this.get("__name").ok();
                Ok(name)
            })?,
        )?;

        // GetFontObjectForAlphabet(alphabet) -> returns self (font localization stub)
        // In WoW this returns a different font for different alphabets (Latin, Cyrillic, etc.)
        // For simulation, just return self
        font.set(
            "GetFontObjectForAlphabet",
            lua.create_function(|_, this: mlua::Table| Ok(this))?,
        )?;

        // Register the font globally if it has a name
        if let Some(ref n) = name {
            lua.globals().set(n.as_str(), font.clone())?;
        }

        Ok(font)
    })?;
    globals.set("CreateFont", create_font)?;

    // GetFonts() - returns a list of registered font names
    let get_fonts = lua.create_function(|lua, ()| {
        // Return an empty table - in simulation we don't track font objects
        lua.create_table()
    })?;
    globals.set("GetFonts", get_fonts)?;

    // GetFontInfo(fontName or fontObject) - returns font information for a registered font
    let get_font_info = lua.create_function(|lua, font_input: Value| {
        // Return a table with font information
        let info = lua.create_table()?;

        match font_input {
            Value::String(name) => {
                let name_str = name.to_string_lossy().to_string();
                info.set("name", name_str.clone())?;
                // Try to get the font object from globals
                if let Ok(font_obj) = lua.globals().get::<mlua::Table>(name_str) {
                    let height: f64 = font_obj.get("__fontHeight").unwrap_or(12.0);
                    let outline: String = font_obj.get("__fontFlags").unwrap_or_default();
                    info.set("height", height)?;
                    info.set("outline", outline)?;
                    // Add color info
                    let color = lua.create_table()?;
                    color.set("r", font_obj.get::<f64>("__textColorR").unwrap_or(1.0))?;
                    color.set("g", font_obj.get::<f64>("__textColorG").unwrap_or(1.0))?;
                    color.set("b", font_obj.get::<f64>("__textColorB").unwrap_or(1.0))?;
                    color.set("a", font_obj.get::<f64>("__textColorA").unwrap_or(1.0))?;
                    info.set("color", color)?;
                } else {
                    info.set("height", 12.0)?;
                    info.set("outline", "")?;
                }
            }
            Value::Table(font_obj) => {
                let name: String = font_obj.get("__name").unwrap_or_default();
                let height: f64 = font_obj.get("__fontHeight").unwrap_or(12.0);
                let outline: String = font_obj.get("__fontFlags").unwrap_or_default();
                info.set("name", name)?;
                info.set("height", height)?;
                info.set("outline", outline)?;
                // Add color info
                let color = lua.create_table()?;
                color.set("r", font_obj.get::<f64>("__textColorR").unwrap_or(1.0))?;
                color.set("g", font_obj.get::<f64>("__textColorG").unwrap_or(1.0))?;
                color.set("b", font_obj.get::<f64>("__textColorB").unwrap_or(1.0))?;
                color.set("a", font_obj.get::<f64>("__textColorA").unwrap_or(1.0))?;
                info.set("color", color)?;
            }
            _ => {
                info.set("name", "")?;
                info.set("height", 12.0)?;
                info.set("outline", "")?;
            }
        }
        Ok(info)
    })?;
    globals.set("GetFontInfo", get_font_info)?;

    // CreateFontFamily(name, members) - creates a font family with different fonts for different alphabets
    // members is an array of {alphabet, file, height, flags} tables
    let create_font_family =
        lua.create_function(|lua, (name, members): (String, mlua::Table)| {
            // Create a font object similar to CreateFont
            let font = lua.create_table()?;
            font.set("__name", name.clone())?;
            font.set("__fontPath", "Fonts\\FRIZQT__.TTF")?;
            font.set("__fontHeight", 12.0)?;
            font.set("__fontFlags", "")?;
            font.set("__textColorR", 1.0)?;
            font.set("__textColorG", 1.0)?;
            font.set("__textColorB", 1.0)?;
            font.set("__textColorA", 1.0)?;
            font.set("__shadowColorR", 0.0)?;
            font.set("__shadowColorG", 0.0)?;
            font.set("__shadowColorB", 0.0)?;
            font.set("__shadowColorA", 0.0)?;
            font.set("__shadowOffsetX", 0.0)?;
            font.set("__shadowOffsetY", 0.0)?;

            // Try to get font info from first member
            if let Ok(first_member) = members.get::<mlua::Table>(1) {
                if let Ok(file) = first_member.get::<String>("file") {
                    font.set("__fontPath", file)?;
                }
                if let Ok(height) = first_member.get::<f64>("height") {
                    font.set("__fontHeight", height)?;
                }
                if let Ok(flags) = first_member.get::<String>("flags") {
                    font.set("__fontFlags", flags)?;
                }
            }

            // SetFont(path, height, flags)
            font.set(
                "SetFont",
                lua.create_function(
                    |_, (this, path, height, flags): (mlua::Table, String, f64, Option<String>)| {
                        this.set("__fontPath", path)?;
                        this.set("__fontHeight", height)?;
                        this.set("__fontFlags", flags.unwrap_or_default())?;
                        Ok(())
                    },
                )?,
            )?;

            // GetFont() -> path, height, flags
            font.set(
                "GetFont",
                lua.create_function(|_, this: mlua::Table| {
                    let path: String = this.get("__fontPath")?;
                    let height: f64 = this.get("__fontHeight")?;
                    let flags: String = this.get("__fontFlags")?;
                    Ok((path, height, flags))
                })?,
            )?;

            // SetTextColor(r, g, b, a)
            font.set(
                "SetTextColor",
                lua.create_function(
                    |_, (this, r, g, b, a): (mlua::Table, f64, f64, f64, Option<f64>)| {
                        this.set("__textColorR", r)?;
                        this.set("__textColorG", g)?;
                        this.set("__textColorB", b)?;
                        this.set("__textColorA", a.unwrap_or(1.0))?;
                        Ok(())
                    },
                )?,
            )?;

            // GetTextColor() -> r, g, b, a
            font.set(
                "GetTextColor",
                lua.create_function(|_, this: mlua::Table| {
                    let r: f64 = this.get("__textColorR")?;
                    let g: f64 = this.get("__textColorG")?;
                    let b: f64 = this.get("__textColorB")?;
                    let a: f64 = this.get("__textColorA")?;
                    Ok((r, g, b, a))
                })?,
            )?;

            // SetShadowColor(r, g, b, a)
            font.set(
                "SetShadowColor",
                lua.create_function(
                    |_, (this, r, g, b, a): (mlua::Table, f64, f64, f64, Option<f64>)| {
                        this.set("__shadowColorR", r)?;
                        this.set("__shadowColorG", g)?;
                        this.set("__shadowColorB", b)?;
                        this.set("__shadowColorA", a.unwrap_or(1.0))?;
                        Ok(())
                    },
                )?,
            )?;

            // GetShadowColor() -> r, g, b, a
            font.set(
                "GetShadowColor",
                lua.create_function(|_, this: mlua::Table| {
                    let r: f64 = this.get("__shadowColorR")?;
                    let g: f64 = this.get("__shadowColorG")?;
                    let b: f64 = this.get("__shadowColorB")?;
                    let a: f64 = this.get("__shadowColorA")?;
                    Ok((r, g, b, a))
                })?,
            )?;

            // SetShadowOffset(x, y)
            font.set(
                "SetShadowOffset",
                lua.create_function(|_, (this, x, y): (mlua::Table, f64, f64)| {
                    this.set("__shadowOffsetX", x)?;
                    this.set("__shadowOffsetY", y)?;
                    Ok(())
                })?,
            )?;

            // GetShadowOffset() -> x, y
            font.set(
                "GetShadowOffset",
                lua.create_function(|_, this: mlua::Table| {
                    let x: f64 = this.get("__shadowOffsetX")?;
                    let y: f64 = this.get("__shadowOffsetY")?;
                    Ok((x, y))
                })?,
            )?;

            // GetName() -> name
            font.set(
                "GetName",
                lua.create_function(|_, this: mlua::Table| {
                    let name: Option<String> = this.get("__name").ok();
                    Ok(name)
                })?,
            )?;

            // GetFontObjectForAlphabet(alphabet) -> returns self
            font.set(
                "GetFontObjectForAlphabet",
                lua.create_function(|_, this: mlua::Table| Ok(this))?,
            )?;

            // CopyFontObject(fontObject or fontName)
            font.set(
                "CopyFontObject",
                lua.create_function(|lua, (this, src): (mlua::Table, Value)| {
                    let src_table: Option<mlua::Table> = match src {
                        Value::String(s) => lua
                            .globals()
                            .get::<Option<mlua::Table>>(s.to_string_lossy().to_string())
                            .ok()
                            .flatten(),
                        Value::Table(t) => Some(t),
                        _ => None,
                    };
                    if let Some(src) = src_table {
                        if let Ok(v) = src.get::<String>("__fontPath") {
                            this.set("__fontPath", v)?;
                        }
                        if let Ok(v) = src.get::<f64>("__fontHeight") {
                            this.set("__fontHeight", v)?;
                        }
                        if let Ok(v) = src.get::<String>("__fontFlags") {
                            this.set("__fontFlags", v)?;
                        }
                        for key in &[
                            "__textColorR",
                            "__textColorG",
                            "__textColorB",
                            "__textColorA",
                            "__shadowColorR",
                            "__shadowColorG",
                            "__shadowColorB",
                            "__shadowColorA",
                            "__shadowOffsetX",
                            "__shadowOffsetY",
                        ] {
                            if let Ok(v) = src.get::<f64>(*key) {
                                this.set(*key, v)?;
                            }
                        }
                    }
                    Ok(())
                })?,
            )?;

            // Register globally
            lua.globals().set(name.as_str(), font.clone())?;
            Ok(font)
        })?;
    globals.set("CreateFontFamily", create_font_family)?;

    Ok(())
}

/// Create standard WoW font objects that addons expect to exist
pub fn create_standard_font_objects(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // Helper to create a font object with specific properties
    let create_font_obj =
        |name: &str, height: f64, flags: &str, r: f64, g: f64, b: f64| -> Result<mlua::Table> {
            let font = lua.create_table()?;

            // Internal state
            font.set("__fontPath", "Fonts\\FRIZQT__.TTF")?;
            font.set("__fontHeight", height)?;
            font.set("__fontFlags", flags)?;
            font.set("__textColorR", r)?;
            font.set("__textColorG", g)?;
            font.set("__textColorB", b)?;
            font.set("__textColorA", 1.0)?;
            font.set("__shadowColorR", 0.0)?;
            font.set("__shadowColorG", 0.0)?;
            font.set("__shadowColorB", 0.0)?;
            font.set("__shadowColorA", 0.0)?;
            font.set("__shadowOffsetX", 0.0)?;
            font.set("__shadowOffsetY", 0.0)?;
            font.set("__justifyH", "CENTER")?;
            font.set("__justifyV", "MIDDLE")?;
            font.set("__name", name)?;

            // Add methods
            font.set(
                "SetFont",
                lua.create_function(
                    |_,
                     (this, path, height, flags): (
                        mlua::Table,
                        String,
                        f64,
                        Option<String>,
                    )| {
                        this.set("__fontPath", path)?;
                        this.set("__fontHeight", height)?;
                        this.set("__fontFlags", flags.unwrap_or_default())?;
                        Ok(())
                    },
                )?,
            )?;
            font.set(
                "GetFont",
                lua.create_function(|_, this: mlua::Table| {
                    let path: String = this.get("__fontPath")?;
                    let height: f64 = this.get("__fontHeight")?;
                    let flags: String = this.get("__fontFlags")?;
                    Ok((path, height, flags))
                })?,
            )?;
            font.set(
                "SetTextColor",
                lua.create_function(
                    |_, (this, r, g, b, a): (mlua::Table, f64, f64, f64, Option<f64>)| {
                        this.set("__textColorR", r)?;
                        this.set("__textColorG", g)?;
                        this.set("__textColorB", b)?;
                        this.set("__textColorA", a.unwrap_or(1.0))?;
                        Ok(())
                    },
                )?,
            )?;
            font.set(
                "GetTextColor",
                lua.create_function(|_, this: mlua::Table| {
                    let r: f64 = this.get("__textColorR")?;
                    let g: f64 = this.get("__textColorG")?;
                    let b: f64 = this.get("__textColorB")?;
                    let a: f64 = this.get("__textColorA")?;
                    Ok((r, g, b, a))
                })?,
            )?;
            font.set(
                "SetShadowColor",
                lua.create_function(
                    |_, (this, r, g, b, a): (mlua::Table, f64, f64, f64, Option<f64>)| {
                        this.set("__shadowColorR", r)?;
                        this.set("__shadowColorG", g)?;
                        this.set("__shadowColorB", b)?;
                        this.set("__shadowColorA", a.unwrap_or(1.0))?;
                        Ok(())
                    },
                )?,
            )?;
            font.set(
                "GetShadowColor",
                lua.create_function(|_, this: mlua::Table| {
                    let r: f64 = this.get("__shadowColorR")?;
                    let g: f64 = this.get("__shadowColorG")?;
                    let b: f64 = this.get("__shadowColorB")?;
                    let a: f64 = this.get("__shadowColorA")?;
                    Ok((r, g, b, a))
                })?,
            )?;
            font.set(
                "SetShadowOffset",
                lua.create_function(|_, (this, x, y): (mlua::Table, f64, f64)| {
                    this.set("__shadowOffsetX", x)?;
                    this.set("__shadowOffsetY", y)?;
                    Ok(())
                })?,
            )?;
            font.set(
                "GetShadowOffset",
                lua.create_function(|_, this: mlua::Table| {
                    let x: f64 = this.get("__shadowOffsetX")?;
                    let y: f64 = this.get("__shadowOffsetY")?;
                    Ok((x, y))
                })?,
            )?;
            font.set(
                "SetJustifyH",
                lua.create_function(|_, (this, justify): (mlua::Table, String)| {
                    this.set("__justifyH", justify)?;
                    Ok(())
                })?,
            )?;
            font.set(
                "GetJustifyH",
                lua.create_function(|_, this: mlua::Table| {
                    let justify: String = this.get("__justifyH")?;
                    Ok(justify)
                })?,
            )?;
            font.set(
                "SetJustifyV",
                lua.create_function(|_, (this, justify): (mlua::Table, String)| {
                    this.set("__justifyV", justify)?;
                    Ok(())
                })?,
            )?;
            font.set(
                "GetJustifyV",
                lua.create_function(|_, this: mlua::Table| {
                    let justify: String = this.get("__justifyV")?;
                    Ok(justify)
                })?,
            )?;
            font.set(
                "SetSpacing",
                lua.create_function(|_, (this, spacing): (mlua::Table, f64)| {
                    this.set("__spacing", spacing)?;
                    Ok(())
                })?,
            )?;
            font.set(
                "GetSpacing",
                lua.create_function(|_, this: mlua::Table| {
                    let spacing: f64 = this.get("__spacing").unwrap_or(0.0);
                    Ok(spacing)
                })?,
            )?;
            font.set(
                "CopyFontObject",
                lua.create_function(|_, (this, src): (mlua::Table, mlua::Table)| {
                    if let Ok(v) = src.get::<String>("__fontPath") {
                        this.set("__fontPath", v)?;
                    }
                    if let Ok(v) = src.get::<f64>("__fontHeight") {
                        this.set("__fontHeight", v)?;
                    }
                    if let Ok(v) = src.get::<String>("__fontFlags") {
                        this.set("__fontFlags", v)?;
                    }
                    for key in &[
                        "__textColorR",
                        "__textColorG",
                        "__textColorB",
                        "__textColorA",
                        "__shadowColorR",
                        "__shadowColorG",
                        "__shadowColorB",
                        "__shadowColorA",
                        "__shadowOffsetX",
                        "__shadowOffsetY",
                    ] {
                        if let Ok(v) = src.get::<f64>(*key) {
                            this.set(*key, v)?;
                        }
                    }
                    if let Ok(v) = src.get::<String>("__justifyH") {
                        this.set("__justifyH", v)?;
                    }
                    if let Ok(v) = src.get::<String>("__justifyV") {
                        this.set("__justifyV", v)?;
                    }
                    Ok(())
                })?,
            )?;
            font.set(
                "GetName",
                lua.create_function(|_, this: mlua::Table| {
                    let name: Option<String> = this.get("__name").ok();
                    Ok(name)
                })?,
            )?;
            // GetFontObjectForAlphabet(alphabet) - returns self for font localization
            font.set(
                "GetFontObjectForAlphabet",
                lua.create_function(|_, this: mlua::Table| Ok(this))?,
            )?;

            globals.set(name, font.clone())?;
            Ok(font)
        };

    // Standard font objects - white text
    create_font_obj("GameFontNormal", 12.0, "", 1.0, 0.82, 0.0)?; // Gold text
    create_font_obj("GameFontNormalSmall", 10.0, "", 1.0, 0.82, 0.0)?;
    create_font_obj("GameFontNormalLarge", 16.0, "", 1.0, 0.82, 0.0)?;
    create_font_obj("GameFontNormalHuge", 20.0, "", 1.0, 0.82, 0.0)?;

    // Highlighted (white) fonts
    create_font_obj("GameFontHighlight", 12.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("GameFontHighlightSmall", 10.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("GameFontHighlightSmallOutline", 10.0, "OUTLINE", 1.0, 1.0, 1.0)?;
    create_font_obj("GameFontHighlightLarge", 16.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("GameFontHighlightHuge", 20.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("GameFontHighlightOutline", 12.0, "OUTLINE", 1.0, 1.0, 1.0)?;

    // Disabled fonts (gray)
    create_font_obj("GameFontDisable", 12.0, "", 0.5, 0.5, 0.5)?;
    create_font_obj("GameFontDisableSmall", 10.0, "", 0.5, 0.5, 0.5)?;
    create_font_obj("GameFontDisableLarge", 16.0, "", 0.5, 0.5, 0.5)?;

    // Red/error fonts
    create_font_obj("GameFontRed", 12.0, "", 1.0, 0.1, 0.1)?;
    create_font_obj("GameFontRedSmall", 10.0, "", 1.0, 0.1, 0.1)?;
    create_font_obj("GameFontRedLarge", 16.0, "", 1.0, 0.1, 0.1)?;

    // Green fonts
    create_font_obj("GameFontGreen", 12.0, "", 0.1, 1.0, 0.1)?;
    create_font_obj("GameFontGreenSmall", 10.0, "", 0.1, 1.0, 0.1)?;
    create_font_obj("GameFontGreenLarge", 16.0, "", 0.1, 1.0, 0.1)?;

    // White fonts
    create_font_obj("GameFontWhite", 12.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("GameFontWhiteSmall", 10.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("GameFontWhiteTiny", 9.0, "", 1.0, 1.0, 1.0)?;

    // Black fonts
    create_font_obj("GameFontBlack", 12.0, "", 0.0, 0.0, 0.0)?;
    create_font_obj("GameFontBlackSmall", 10.0, "", 0.0, 0.0, 0.0)?;

    // Number fonts
    create_font_obj("NumberFontNormal", 14.0, "OUTLINE", 1.0, 1.0, 1.0)?;
    create_font_obj("NumberFontNormalSmall", 12.0, "OUTLINE", 1.0, 1.0, 1.0)?;
    create_font_obj("NumberFontNormalLarge", 16.0, "OUTLINE", 1.0, 1.0, 1.0)?;
    create_font_obj("NumberFontNormalHuge", 24.0, "OUTLINE", 1.0, 1.0, 1.0)?;
    create_font_obj("NumberFontNormalRightRed", 14.0, "OUTLINE", 1.0, 0.1, 0.1)?;
    create_font_obj("NumberFontNormalRightYellow", 14.0, "OUTLINE", 1.0, 1.0, 0.0)?;

    // Chat fonts
    create_font_obj("ChatFontNormal", 14.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("ChatFontSmall", 12.0, "", 1.0, 1.0, 1.0)?;

    // System fonts
    create_font_obj("SystemFont_Small", 10.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Med1", 12.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Med2", 13.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Med3", 14.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Large", 16.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Huge1", 20.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Huge2", 24.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Outline", 12.0, "OUTLINE", 1.0, 1.0, 1.0)?;
    create_font_obj(
        "SystemFont_OutlineThick_Huge2",
        24.0,
        "OUTLINE, THICKOUTLINE",
        1.0,
        1.0,
        1.0,
    )?;
    create_font_obj(
        "SystemFont_OutlineThick_Huge4",
        32.0,
        "OUTLINE, THICKOUTLINE",
        1.0,
        1.0,
        1.0,
    )?;
    create_font_obj(
        "SystemFont_OutlineThick_WTF",
        64.0,
        "OUTLINE, THICKOUTLINE",
        1.0,
        1.0,
        1.0,
    )?;
    create_font_obj("SystemFont_Shadow_Small", 10.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Shadow_Med1", 12.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Shadow_Med2", 13.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Shadow_Med3", 14.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Shadow_Large", 16.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Shadow_Large_Outline", 16.0, "OUTLINE", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Shadow_Huge1", 20.0, "", 1.0, 1.0, 1.0)?;

    // Tooltip fonts
    create_font_obj("GameTooltipHeader", 14.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("GameTooltipText", 12.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("GameTooltipTextSmall", 10.0, "", 1.0, 1.0, 1.0)?;

    // Subzone fonts
    create_font_obj("SubZoneTextFont", 26.0, "OUTLINE", 1.0, 0.82, 0.0)?;
    create_font_obj("PVPInfoTextFont", 20.0, "OUTLINE", 1.0, 0.1, 0.1)?;

    // Misc fonts
    create_font_obj("FriendsFont_Normal", 12.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("FriendsFont_Small", 10.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("FriendsFont_Large", 14.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("FriendsFont_UserText", 11.0, "", 1.0, 1.0, 1.0)?;

    Ok(())
}
