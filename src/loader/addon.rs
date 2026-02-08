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

/// Initialize saved variables for an addon (WTF first, then JSON fallback).
fn init_saved_variables(
    env: &WowLuaEnv,
    toc: &TocFile,
    folder_name: &str,
    mgr: &mut SavedVariablesManager,
) -> Vec<String> {
    let mut warnings = Vec::new();
    match mgr.load_wtf_for_addon(env.lua(), folder_name) {
        Ok(count) if count > 0 => {
            tracing::debug!("Loaded {} WTF SavedVariables file(s) for {}", count, toc.name);
        }
        Ok(_) => {
            let saved_vars = toc.saved_variables();
            let saved_vars_per_char = toc.saved_variables_per_character();
            if (!saved_vars.is_empty() || !saved_vars_per_char.is_empty())
                && let Err(e) =
                    mgr.init_for_addon(env.lua(), folder_name, &saved_vars, &saved_vars_per_char)
                {
                    warnings.push(format!(
                        "Failed to initialize saved variables for {}: {}",
                        folder_name, e
                    ));
                }
        }
        Err(e) => {
            warnings.push(format!(
                "Failed to load WTF SavedVariables for {}: {}",
                folder_name, e
            ));
        }
    }
    warnings
}

/// Internal addon loading with optional saved variables.
pub fn load_addon_internal(
    env: &WowLuaEnv,
    toc: &TocFile,
    saved_vars_mgr: Option<&mut SavedVariablesManager>,
) -> Result<LoadResult, LoadError> {
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

    if let Some(mgr) = saved_vars_mgr {
        let sv_start = Instant::now();
        result
            .warnings
            .extend(init_saved_variables(env, toc, folder_name, mgr));
        result.timing.saved_vars_time = sv_start.elapsed();
    }

    let addon_table = env.create_addon_table().map_err(|e| LoadError::Lua(e.to_string()))?;
    let ctx = AddonContext {
        name: folder_name,
        table: addon_table,
        addon_root: &toc.addon_dir,
    };

    let overlay_dir = Path::new("Interface/AddOns").join(folder_name);

    for (file_rel, file) in toc.files.iter().zip(toc.file_paths()) {
        // Check local overlay first (./Interface/AddOns/{addon}/{file})
        let file = {
            let overlay = overlay_dir.join(file_rel);
            if overlay.exists() { overlay } else { file }
        };
        let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "lua" => {
                match load_lua_file(env, &file, &ctx, &mut result.timing) {
                    Ok(()) => result.lua_files += 1,
                    Err(e) => result.warnings.push(format!("{}: {}", file.display(), e)),
                }
                apply_cpp_mixin_stubs(env);
            }
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

/// Patch Lua mixin tables with methods normally provided by the C++ engine.
///
/// WoW's C++ engine provides OnLoad for certain base control button mixins.
/// The Lua side creates empty tables (e.g. `ModelSceneControlButtonMixin = {}`),
/// and derived mixins call `BaseMixin.OnLoad(self)` expecting the C++ method.
/// Runs after each .lua file so stubs are available before the next .xml file
/// creates frames that depend on them.
fn apply_cpp_mixin_stubs(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if ModelSceneControlButtonMixin and not ModelSceneControlButtonMixin.OnLoad then
            ModelSceneControlButtonMixin.OnLoad = function() end
        end
        if PerksModelSceneControlButtonMixin and not PerksModelSceneControlButtonMixin.OnLoad then
            PerksModelSceneControlButtonMixin.OnLoad = function() end
        end
        if PetActionBarMixin and PetActionBarMixin.Update and not PetActionBarMixin._update_guarded then
            PetActionBarMixin._update_guarded = true
            local origUpdate = PetActionBarMixin.Update
            PetActionBarMixin.Update = function(self)
                if not self.actionButtons or #self.actionButtons == 0 then return end
                return origUpdate(self)
            end
        end
        "#,
    );
}
