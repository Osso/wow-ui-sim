use super::{is_addon_loaded, load_one_blizzard_addon};
use std::collections::HashSet;
use std::path::PathBuf;
use wow_ui_sim::loader::{LoadTiming, discover_blizzard_addon_closure_for_screen};
use wow_ui_sim::logging;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::saved_variables::SavedVariablesManager;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

pub(super) fn load_required_blizzard_dependencies_for_addons(
    env: &WowLuaEnv,
    saved_vars: &mut Option<SavedVariablesManager>,
    screen: ScreenKind,
    addons: &[(String, PathBuf)],
) {
    let roots = required_blizzard_dependencies(addons);
    if roots.is_empty() {
        return;
    }

    let blizzard_ui_path = match wow_ui_sim::paths::default_blizzard_ui_addons_path() {
        Ok(path) => path,
        Err(error) => {
            logging::println_elapsed(&format!(
                "Skipping Blizzard addon dependencies for third-party addons: {error}"
            ));
            return;
        }
    };
    let root_refs = roots.iter().map(String::as_str).collect::<Vec<_>>();
    let dependencies =
        discover_blizzard_addon_closure_for_screen(&blizzard_ui_path, screen, &root_refs);
    if dependencies.is_empty() {
        return;
    }

    logging::println_elapsed(&format!(
        "Loading {} Blizzard addon dependencies for third-party addons...",
        dependencies.len()
    ));
    let verbose = std::env::var("WOW_SIM_VERBOSE").is_ok();
    let mut timing = LoadTiming::default();
    for (name, toc_path) in dependencies {
        if is_addon_loaded(env, &name) {
            continue;
        }
        load_one_blizzard_addon(
            env,
            &name,
            &toc_path,
            super::StartupAddonLoadKind::Full,
            saved_vars,
            verbose,
            &mut timing,
        );
    }
}

fn required_blizzard_dependencies(addons: &[(String, PathBuf)]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut dependencies = Vec::new();
    for (_, toc_path) in addons {
        let Ok(toc) = TocFile::from_file(toc_path) else {
            continue;
        };
        for dependency in toc.dependencies() {
            if dependency.starts_with("Blizzard_") && seen.insert(dependency.clone()) {
                dependencies.push(dependency);
            }
        }
    }
    dependencies
}

#[cfg(test)]
mod tests {
    use super::required_blizzard_dependencies;
    use std::path::{Path, PathBuf};

    fn write_addon_with_toc(root: &Path, name: &str, toc: &str) -> PathBuf {
        let addon_dir = root.join(name);
        std::fs::create_dir_all(&addon_dir).expect("create addon dir");
        let toc_path = addon_dir.join(format!("{name}.toc"));
        std::fs::write(&toc_path, toc).expect("write toc");
        std::fs::write(addon_dir.join("main.lua"), "").expect("write lua");
        toc_path
    }

    #[test]
    fn required_blizzard_dependencies_reads_hard_toc_dependencies() {
        let temp = tempfile::tempdir().expect("tempdir");
        let toc_path = write_addon_with_toc(
            temp.path(),
            "CraftSim",
            "## Interface: 120005\n## Dependencies: Blizzard_Professions, Ace3\nmain.lua\n",
        );

        let deps = required_blizzard_dependencies(&[("CraftSim".to_string(), toc_path)]);

        assert_eq!(deps, vec!["Blizzard_Professions"]);
    }
}
