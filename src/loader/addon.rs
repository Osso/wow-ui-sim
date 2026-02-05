//! Addon loading internals.

use crate::lua_api::WowLuaEnv;
use crate::saved_variables::SavedVariablesManager;
use crate::toc::TocFile;
use mlua::Table;
use std::path::Path;
use std::time::Instant;

use super::error::LoadError;
use super::lua_file::load_lua_file;
use super::xml_file::load_xml_file;
use super::{LoadResult, LoadTiming};

/// Context for loading addon files (name, private table, and addon root for path resolution).
pub struct AddonContext<'a> {
    pub name: &'a str,
    pub table: Table,
    /// Addon root directory for fallback path resolution
    pub addon_root: &'a Path,
}

/// Internal addon loading with optional saved variables.
pub fn load_addon_internal(
    env: &WowLuaEnv,
    toc: &TocFile,
    saved_vars_mgr: Option<&mut SavedVariablesManager>,
) -> Result<LoadResult, LoadError> {
    // WoW passes the folder name (not Title) as the addon name vararg
    let folder_name = toc
        .addon_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&toc.name);

    let mut result = LoadResult {
        name: toc.name.clone(),
        lua_files: 0,
        xml_files: 0,
        timing: LoadTiming::default(),
        warnings: Vec::new(),
    };

    // Initialize saved variables before loading addon files
    if let Some(mgr) = saved_vars_mgr {
        let sv_start = Instant::now();
        // First try to load WTF saved variables from real WoW installation
        match mgr.load_wtf_for_addon(env.lua(), folder_name) {
            Ok(count) if count > 0 => {
                tracing::debug!("Loaded {} WTF SavedVariables file(s) for {}", count, toc.name);
            }
            Ok(_) => {
                // No WTF files found, fall back to JSON storage
                let saved_vars = toc.saved_variables();
                let saved_vars_per_char = toc.saved_variables_per_character();

                if !saved_vars.is_empty() || !saved_vars_per_char.is_empty() {
                    if let Err(e) = mgr.init_for_addon(
                        env.lua(),
                        folder_name,
                        &saved_vars,
                        &saved_vars_per_char,
                    ) {
                        result.warnings.push(format!(
                            "Failed to initialize saved variables for {}: {}",
                            folder_name, e
                        ));
                    }
                }
            }
            Err(e) => {
                result.warnings.push(format!(
                    "Failed to load WTF SavedVariables for {}: {}",
                    folder_name, e
                ));
            }
        }
        result.timing.saved_vars_time = sv_start.elapsed();
    }

    // Create the shared private table for this addon (WoW passes this as second vararg)
    let addon_table = env.create_addon_table().map_err(|e| LoadError::Lua(e.to_string()))?;
    let ctx = AddonContext {
        name: folder_name,
        table: addon_table,
        addon_root: &toc.addon_dir,
    };

    for file in toc.file_paths() {
        let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext {
            "lua" => match load_lua_file(env, &file, &ctx, &mut result.timing) {
                Ok(()) => result.lua_files += 1,
                Err(e) => result.warnings.push(format!("{}: {}", file.display(), e)),
            },
            "xml" => match load_xml_file(env, &file, &ctx, &mut result.timing) {
                Ok(count) => {
                    result.xml_files += 1;
                    result.lua_files += count;
                }
                Err(e) => result.warnings.push(format!("{}: {}", file.display(), e)),
            },
            _ => {
                result.warnings.push(format!("{}: unknown file type", file.display()));
            }
        }
    }

    Ok(result)
}
