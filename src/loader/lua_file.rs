//! Lua file loading functionality.

use crate::lua_api::LoaderEnv;
use std::path::Path;
use std::time::Instant;

use super::addon::AddonContext;
use super::bytecode_cache;
use super::error::LoadError;
use super::LoadTiming;

/// Load a Lua file into the environment with addon varargs.
pub fn load_lua_file(
    env: &LoaderEnv<'_>,
    path: &Path,
    ctx: &AddonContext,
    timing: &mut LoadTiming,
) -> Result<(), LoadError> {
    let io_start = Instant::now();
    let bytes = std::fs::read(path)?;
    timing.io_time += io_start.elapsed();

    // Transform path to WoW-style for debugstack (libraries expect "AddOns/..." pattern)
    let path_str = path.display().to_string();
    let chunk_name = if let Some(pos) = path_str.find("AddOns/") {
        format!("@Interface/{}", &path_str[pos..])
    } else {
        format!("@{}", path_str)
    };
    // Clone the table since mlua moves it on call
    let table_clone = ctx.table.clone();
    let lua = env.lua();

    let lua_start = Instant::now();
    let func = if bytecode_cache::is_disabled() {
        compile_from_source(lua, &bytes, &chunk_name)?
    } else {
        load_cached_or_compile(lua, &bytes, &chunk_name, timing)?
    };

    func.call::<()>((ctx.name.to_string(), table_clone))
        .map_err(|e| LoadError::Lua(e.to_string()))?;
    timing.lua_exec_time += lua_start.elapsed();

    Ok(())
}

/// Try loading from bytecode cache; compile and cache on miss.
fn load_cached_or_compile(
    lua: &mlua::Lua,
    bytes: &[u8],
    chunk_name: &str,
    timing: &mut LoadTiming,
) -> Result<mlua::Function, LoadError> {
    let hash = bytecode_cache::content_hash(bytes, chunk_name);

    if let Some(bytecode) = bytecode_cache::get(hash) {
        // Bytecode found — try loading (may fail if Lua version changed)
        if let Ok(func) = lua
            .load(bytecode.as_slice())
            .set_name(chunk_name)
            .into_function()
        {
            timing.cache_hits += 1;
            return Ok(func);
        }
    }

    // Cache miss or invalid bytecode — compile from source
    timing.cache_misses += 1;
    let func = compile_from_source(lua, bytes, chunk_name)?;
    let bc = func.dump(false);
    bytecode_cache::put(hash, &bc);
    Ok(func)
}

/// Compile Lua source code into a function.
fn compile_from_source(
    lua: &mlua::Lua,
    bytes: &[u8],
    chunk_name: &str,
) -> Result<mlua::Function, LoadError> {
    let code = String::from_utf8_lossy(bytes);
    // Strip UTF-8 BOM if present (common in Windows-edited files)
    let code = code.strip_prefix('\u{feff}').unwrap_or(&code);
    lua.load(code)
        .set_name(chunk_name)
        .into_function()
        .map_err(|e| LoadError::Lua(e.to_string()))
}
