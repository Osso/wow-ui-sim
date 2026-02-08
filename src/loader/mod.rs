//! Addon loader - loads addons from TOC files.

mod addon;
mod button;
mod error;
pub(crate) mod helpers;
mod lua_file;
mod xml_file;
mod xml_fontstring;
mod xml_frame;
mod xml_texture;

use crate::lua_api::LoaderEnv;
use crate::saved_variables::SavedVariablesManager;
use crate::toc::TocFile;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub use error::LoadError;
pub use xml_frame::create_frame_from_xml;

/// Find the TOC file for an addon directory.
/// Prefers Mainline variant, then exact name match, then any non-Classic TOC.
pub fn find_toc_file(addon_dir: &Path) -> Option<PathBuf> {
    let addon_name = addon_dir.file_name()?.to_str()?;
    let toc_variants = [
        format!("{}_Mainline.toc", addon_name),
        format!("{}.toc", addon_name),
    ];
    for variant in &toc_variants {
        let toc_path = addon_dir.join(variant);
        if toc_path.exists() {
            return Some(toc_path);
        }
    }
    // Fallback: find any .toc file (skip Classic/TBC/etc.)
    if let Ok(entries) = std::fs::read_dir(addon_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "toc").unwrap_or(false) {
                let name = path.file_name().unwrap().to_str().unwrap();
                if !name.contains("_Cata")
                    && !name.contains("_Wrath")
                    && !name.contains("_TBC")
                    && !name.contains("_Vanilla")
                    && !name.contains("_Mists")
                {
                    return Some(path);
                }
            }
        }
    }
    None
}

/// Result of loading an addon.
#[derive(Debug)]
pub struct LoadResult {
    /// Addon name
    pub name: String,
    /// Number of Lua files loaded
    pub lua_files: usize,
    /// Number of XML files loaded
    pub xml_files: usize,
    /// Time breakdown
    pub timing: LoadTiming,
    /// Errors encountered (non-fatal)
    pub warnings: Vec<String>,
}

/// Timing breakdown for addon loading.
#[derive(Debug, Default, Clone)]
pub struct LoadTiming {
    /// Time reading files from disk
    pub io_time: Duration,
    /// Time parsing XML
    pub xml_parse_time: Duration,
    /// Time executing Lua
    pub lua_exec_time: Duration,
    /// Time loading SavedVariables
    pub saved_vars_time: Duration,
}

impl LoadTiming {
    pub fn total(&self) -> Duration {
        self.io_time + self.xml_parse_time + self.lua_exec_time + self.saved_vars_time
    }
}

/// Load an addon from its TOC file.
pub fn load_addon(env: &LoaderEnv<'_>, toc_path: &Path) -> Result<LoadResult, LoadError> {
    let toc = TocFile::from_file(toc_path)?;
    load_addon_from_toc(env, &toc)
}

/// Load an addon from its TOC file with saved variables support.
pub fn load_addon_with_saved_vars(
    env: &LoaderEnv<'_>,
    toc_path: &Path,
    saved_vars_mgr: &mut SavedVariablesManager,
) -> Result<LoadResult, LoadError> {
    let toc = TocFile::from_file(toc_path)?;
    load_addon_from_toc_with_saved_vars(env, &toc, saved_vars_mgr)
}

/// Load an addon from a parsed TOC.
pub fn load_addon_from_toc(env: &LoaderEnv<'_>, toc: &TocFile) -> Result<LoadResult, LoadError> {
    addon::load_addon_internal(env, toc, None)
}

/// Load an addon from a parsed TOC with saved variables support.
pub fn load_addon_from_toc_with_saved_vars(
    env: &LoaderEnv<'_>,
    toc: &TocFile,
    saved_vars_mgr: &mut SavedVariablesManager,
) -> Result<LoadResult, LoadError> {
    addon::load_addon_internal(env, toc, Some(saved_vars_mgr))
}

#[cfg(test)]
mod tests;
