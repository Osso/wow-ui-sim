//! Miscellaneous C_* namespace APIs.
//!
//! Split into sub-modules for organization:
//! - `c_misc_api_core` - Core game system C_ namespaces
//! - `c_misc_api_ui` - UI-related C_ namespaces
//! - `c_misc_api_game` - Game menu stubs and global game functions

use mlua::{Lua, Result, Value};

/// Register all miscellaneous C_* namespace APIs.
pub fn register_c_misc_api(lua: &Lua) -> Result<()> {
    super::c_misc_api_core::register_all(lua)?;
    register_c_color_overrides(lua)?;
    register_tooltip_data_processor(lua)?;
    super::c_misc_api_ui::register_all(lua)?;
    super::c_misc_api_game::register_all(lua)?;
    Ok(())
}

fn register_c_color_overrides(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;

    t.set("GetColorForQuality", lua.create_function(|lua, quality: i32| {
        create_quality_color(lua, quality)
    })?)?;
    t.set("GetDefaultColorForQuality", lua.create_function(|lua, quality: i32| {
        create_quality_color(lua, quality)
    })?)?;
    t.set("GetColorOverrideInfo", lua.create_function(|_, _ot: i32| Ok(Value::Nil))?)?;
    t.set("ClearColorOverrides", lua.create_function(|_, ()| Ok(()))?)?;
    t.set("SetColorOverride", lua.create_function(|_, (_ot, _c): (i32, Value)| Ok(()))?)?;
    t.set("RemoveColorOverride", lua.create_function(|_, _ot: i32| Ok(()))?)?;

    lua.globals().set("C_ColorOverrides", t)?;
    Ok(())
}

fn create_quality_color(lua: &mlua::Lua, quality: i32) -> Result<Value> {
    let (r, g, b) = match quality {
        0 => (0.62, 0.62, 0.62), // Poor (gray)
        1 => (1.00, 1.00, 1.00), // Common (white)
        2 => (0.12, 1.00, 0.00), // Uncommon (green)
        3 => (0.00, 0.44, 0.87), // Rare (blue)
        4 => (0.64, 0.21, 0.93), // Epic (purple)
        5 => (1.00, 0.50, 0.00), // Legendary (orange)
        6 => (0.90, 0.80, 0.50), // Artifact (light gold)
        7 => (0.00, 0.80, 1.00), // Heirloom (light blue)
        8 => (0.00, 0.80, 1.00), // WoW Token
        _ => (1.00, 1.00, 1.00), // Default to white
    };
    let code = format!("return CreateColor({}, {}, {}, 1.0)", r, g, b);
    lua.load(&code).eval::<Value>()
}

fn register_tooltip_data_processor(lua: &Lua) -> Result<()> {
    lua.globals().set(
        "TooltipDataProcessor",
        lua.create_table_from([
            ("AddTooltipPostCall", Value::Function(
                lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
            )),
        ])?,
    )?;
    Ok(())
}
