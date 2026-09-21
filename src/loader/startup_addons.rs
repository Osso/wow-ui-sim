use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartupAddonLoadKind {
    Full,
    BootstrapOnly,
}

pub struct StartupAddon {
    pub name: String,
    pub toc_path: PathBuf,
    pub kind: StartupAddonLoadKind,
}

pub fn startup_bootstrap_eligible(toc: &TocFile) -> bool {
    cfg!(any(
        feature = "retail-12-1-0",
        feature = "client-wowforever"
    )) && toc.is_load_on_demand()
        && toc.has_bootstrap_files()
}

/// Keep bootstrap-only addons in the ordinary dependency-ordered startup stream.
pub fn discover_blizzard_startup_addons_for_screen(
    directory: &Path,
    screen: ScreenKind,
) -> Vec<StartupAddon> {
    let Some((mut addons, mut pool)) =
        discover_blizzard_addon_toc_pools_for_screen(directory, screen)
    else {
        return Vec::new();
    };
    let dependencies = implicit_blizzard_startup_dependencies();
    pull_required_dependency_addons(&mut addons, &mut pool, &dependencies);
    let bootstrap_names = insert_bootstrap_nodes(&mut addons, &mut pool, screen);
    pull_required_dependency_addons(&mut addons, &mut pool, &dependencies);
    promote_foundational_addons_to_load_first(&mut addons);
    topological_sort_addons_with_extra_dependencies(addons, &dependencies)
        .into_iter()
        .map(|(name, toc_path)| {
            let kind = if bootstrap_names.contains(&name) {
                StartupAddonLoadKind::BootstrapOnly
            } else {
                StartupAddonLoadKind::Full
            };
            StartupAddon {
                name,
                toc_path,
                kind,
            }
        })
        .collect()
}

fn insert_bootstrap_nodes(
    addons: &mut HashMap<String, (PathBuf, TocFile)>,
    pool: &mut HashMap<String, (PathBuf, TocFile)>,
    screen: ScreenKind,
) -> HashSet<String> {
    let bootstrap_names: HashSet<String> = pool
        .iter()
        .filter(|(name, (_, toc))| is_eligible_bootstrap_root(name, toc, screen))
        .map(|(name, _)| name.clone())
        .collect();
    for name in &bootstrap_names {
        if let Some(entry) = pool.remove(name) {
            addons.insert(name.clone(), entry);
        }
    }
    bootstrap_names
}

fn is_eligible_bootstrap_root(name: &str, toc: &TocFile, screen: ScreenKind) -> bool {
    if !name.starts_with("Blizzard_") || is_addon_excluded_for_active_profile(name) {
        return false;
    }
    if excluded_addons_for_screen(screen).contains(&name) {
        return false;
    }
    startup_bootstrap_eligible(toc)
}

pub fn load_startup_addon(
    env: &LoaderEnv<'_>,
    toc_path: &Path,
    kind: StartupAddonLoadKind,
    saved_vars: Option<&mut SavedVariablesManager>,
) -> Result<LoadResult, LoadError> {
    match kind {
        StartupAddonLoadKind::BootstrapOnly => {
            let toc = TocFile::from_file(toc_path)?;
            load_addon_bootstrap_from_toc(env, &toc)
        }
        StartupAddonLoadKind::Full => load_addon_path(env, toc_path, saved_vars),
    }
}
