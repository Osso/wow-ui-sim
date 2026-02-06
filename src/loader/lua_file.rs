//! Lua file loading functionality.

use crate::lua_api::WowLuaEnv;
use std::path::Path;
use std::time::Instant;

use super::addon::AddonContext;
use super::error::LoadError;
use super::LoadTiming;

/// Load a Lua file into the environment with addon varargs.
pub fn load_lua_file(
    env: &WowLuaEnv,
    path: &Path,
    ctx: &AddonContext,
    timing: &mut LoadTiming,
) -> Result<(), LoadError> {
    let io_start = Instant::now();
    // Use lossy UTF-8 conversion to handle files with invalid encoding
    let bytes = std::fs::read(path)?;
    let code = String::from_utf8_lossy(&bytes);
    timing.io_time += io_start.elapsed();

    // Strip UTF-8 BOM if present (common in Windows-edited files)
    let code = code.strip_prefix('\u{feff}').unwrap_or(&code);
    // Transform path to WoW-style for debugstack (libraries expect "AddOns/..." pattern)
    let path_str = path.display().to_string();
    let chunk_name = if let Some(pos) = path_str.find("AddOns/") {
        format!("@Interface/{}", &path_str[pos..])
    } else {
        format!("@{}", path_str)
    };
    // Clone the table since mlua moves it on call
    let table_clone = ctx.table.clone();

    let lua_start = Instant::now();
    env.exec_with_varargs(&code, &chunk_name, ctx.name, table_clone)
        .map_err(|e| LoadError::Lua(e.to_string()))?;
    timing.lua_exec_time += lua_start.elapsed();

    Ok(())
}
