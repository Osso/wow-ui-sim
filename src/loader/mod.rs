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
use std::collections::HashMap;
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

/// Discover all Blizzard addons in a BlizzardUI directory, topologically sorted by dependencies.
///
/// Scans for `Blizzard_*` subdirectories, parses their TOC files, filters out `LoadOnDemand`
/// addons (unless required by a non-LOD addon), and returns them in dependency order.
pub fn discover_blizzard_addons(blizzard_ui_dir: &Path) -> Vec<(String, PathBuf)> {
    let entries = match std::fs::read_dir(blizzard_ui_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    // Parse all addon TOCs into two pools: normal and load-on-demand
    let mut addons: HashMap<String, (PathBuf, TocFile)> = HashMap::new();
    let mut lod_pool: HashMap<String, (PathBuf, TocFile)> = HashMap::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = path.file_name().unwrap().to_str().unwrap().to_string();
        if !dir_name.starts_with("Blizzard_") {
            continue;
        }
        let Some(toc_path) = find_toc_file(&path) else {
            continue;
        };
        let Ok(toc) = TocFile::from_file(&toc_path) else {
            continue;
        };
        if toc.is_glue_only() {
            continue;
        }
        if toc.is_load_on_demand() {
            lod_pool.insert(dir_name, (toc_path, toc));
        } else {
            addons.insert(dir_name, (toc_path, toc));
        }
    }

    // Pull LOD addons that are required by non-LOD addons
    pull_required_lod_addons(&mut addons, &mut lod_pool);

    topological_sort_addons(addons)
}

/// Recursively pull LoadOnDemand addons into the main set when required by loaded addons.
fn pull_required_lod_addons(
    addons: &mut HashMap<String, (PathBuf, TocFile)>,
    lod_pool: &mut HashMap<String, (PathBuf, TocFile)>,
) {
    let mut needed: Vec<String> = addons
        .values()
        .flat_map(|(_, toc)| toc.dependencies())
        .filter(|dep| lod_pool.contains_key(dep))
        .collect();

    while let Some(name) = needed.pop() {
        if addons.contains_key(&name) {
            continue;
        }
        if let Some((toc_path, toc)) = lod_pool.remove(&name) {
            // This LOD addon may itself depend on other LOD addons
            for dep in toc.dependencies() {
                if lod_pool.contains_key(&dep) {
                    needed.push(dep);
                }
            }
            addons.insert(name, (toc_path, toc));
        }
    }
}

/// Topologically sort addons by their declared dependencies (Kahn's algorithm).
/// Base UI addons are placed first in a fixed order, then remaining addons are sorted.
fn topological_sort_addons(
    mut addons: HashMap<String, (PathBuf, TocFile)>,
) -> Vec<(String, PathBuf)> {
    // Extract base UI addons in fixed order, pulling their non-base dependencies first.
    let base_set: std::collections::HashSet<&str> =
        BASE_UI_ADDONS.iter().copied().collect();
    let mut result = Vec::with_capacity(addons.len());
    let mut loaded: std::collections::HashSet<String> = std::collections::HashSet::new();
    for &base in BASE_UI_ADDONS {
        // Pull non-base dependencies before this base addon
        let deps = addons
            .get(base)
            .map(|(_, toc)| toc.dependencies())
            .unwrap_or_default();
        for dep in deps {
            pull_base_deps(&dep, &mut addons, &mut result, &mut loaded, &base_set);
        }
        if let Some((toc_path, _)) = addons.remove(base) {
            result.push((base.to_string(), toc_path));
            loaded.insert(base.to_string());
        }
    }

    // Sort remaining addons by declared dependencies
    let available: std::collections::HashSet<&str> =
        addons.keys().map(|s| s.as_str()).collect();
    let deps = build_dependency_graph(&addons, &available);
    let sorted = kahns_sort(&deps, addons.len());

    result.extend(sorted.into_iter().filter_map(|name| {
        let (toc_path, _) = addons.get(name)?;
        Some((name.to_string(), toc_path.clone()))
    }));

    result
}

/// Recursively pull non-base dependencies from the addon pool into the result list.
fn pull_base_deps(
    name: &str,
    addons: &mut HashMap<String, (PathBuf, TocFile)>,
    result: &mut Vec<(String, PathBuf)>,
    loaded: &mut std::collections::HashSet<String>,
    base_set: &std::collections::HashSet<&str>,
) {
    if loaded.contains(name) || base_set.contains(name) {
        return;
    }
    let deps = addons
        .get(name)
        .map(|(_, toc)| toc.dependencies())
        .unwrap_or_default();
    for dep in deps {
        pull_base_deps(&dep, addons, result, loaded, base_set);
    }
    if let Some((toc_path, _)) = addons.remove(name) {
        result.push((name.to_string(), toc_path));
        loaded.insert(name.to_string());
    }
}

/// Foundational addons that form the base UI layer.
/// In WoW these are loaded before all other addons as part of FrameXML.
/// They have circular declared dependencies, so we load them in this fixed order.
const BASE_UI_ADDONS: &[&str] = &[
    "Blizzard_SharedXMLBase",
    "Blizzard_SharedXML",
    "Blizzard_SharedXMLGame",
    "Blizzard_FrameXML",
    "Blizzard_UIParent",
];

/// Build a map of addon name -> list of available addon names it depends on.
/// Includes both required and optional dependencies (WoW loads optional deps
/// before the addon if they are present).
fn build_dependency_graph<'a>(
    addons: &'a HashMap<String, (PathBuf, TocFile)>,
    available: &std::collections::HashSet<&'a str>,
) -> HashMap<&'a str, Vec<&'a str>> {
    addons
        .iter()
        .map(|(name, (_, toc))| {
            let mut deps: Vec<&str> = toc
                .dependencies()
                .iter()
                .filter_map(|d| available.get(d.as_str()).copied())
                .collect();
            for d in toc.optional_deps() {
                if let Some(&dep) = available.get(d.as_str()) {
                    if !deps.contains(&dep) {
                        deps.push(dep);
                    }
                }
            }
            (name.as_str(), deps)
        })
        .collect()
}

/// Run Kahn's algorithm on a dependency graph. Returns names in topological order.
/// Ties are broken alphabetically for deterministic output.
fn kahns_sort<'a>(deps: &HashMap<&'a str, Vec<&'a str>>, count: usize) -> Vec<&'a str> {
    let mut in_degree: HashMap<&str, usize> = deps.keys().map(|&n| (n, 0)).collect();
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();
    for (&node, reqs) in deps {
        *in_degree.entry(node).or_default() = reqs.len();
        for &r in reqs {
            dependents.entry(r).or_default().push(node);
        }
    }

    // Seed queue with zero-dependency addons, sorted descending (pop takes last = smallest)
    let mut queue: Vec<&str> = in_degree
        .iter()
        .filter(|&(_, deg)| *deg == 0)
        .map(|(&name, _)| name)
        .collect();
    queue.sort_by(|a, b| b.cmp(a));

    let mut result = Vec::with_capacity(count);
    while let Some(name) = queue.pop() {
        result.push(name);
        for &dep in dependents.get(name).unwrap_or(&Vec::new()) {
            if let Some(deg) = in_degree.get_mut(dep) {
                *deg -= 1;
                if *deg == 0 {
                    let pos = queue.partition_point(|&x| x > dep);
                    queue.insert(pos, dep);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests;
