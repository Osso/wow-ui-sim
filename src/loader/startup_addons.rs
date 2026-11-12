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

pub fn uses_verified_load_with_semantics() -> bool {
    matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    )
}

/// Whether runtime loading resolves registered TOC child declarations.
/// PTR inclusion is inferred because parser-to-resolver linkage remains unknown.
pub fn resolves_load_with_children() -> bool {
    matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::Ptr
            | crate::client_profile::ClientProfile::WowForever
    )
}

pub fn startup_bootstrap_eligible(toc: &TocFile) -> bool {
    cfg!(any(
        feature = "retail-12-1-0",
        feature = "client-wowforever"
    )) && toc.is_load_on_demand()
        && toc.has_bootstrap_files()
}

/// Register `LoadWith` declarations without changing the public AddOn list.
pub fn register_load_with_addons(env: &LoaderEnv<'_>, addons: &[(String, PathBuf)]) {
    let registered_enabled = env
        .state()
        .borrow()
        .addons
        .iter()
        .map(|addon| (addon.folder_name.clone(), addon.enabled))
        .collect::<HashMap<_, _>>();
    let declarations = addons
        .iter()
        .filter_map(|(name, toc_path)| {
            let toc = match TocFile::from_file(toc_path) {
                Ok(toc) => toc,
                Err(error) => {
                    tracing::warn!(
                        "Cannot read addon TOC {} for LoadWith registration: {error}",
                        toc_path.display()
                    );
                    return None;
                }
            };
            let targets = toc.load_with();
            let enabled = registered_enabled
                .get(name)
                .copied()
                .unwrap_or_else(|| toc.default_enabled());
            Some((name.clone(), targets, enabled))
        })
        .collect::<Vec<_>>();
    register_load_with_declarations(env, &declarations);
}

/// Register already-parsed `LoadWith` declarations and invalidate their index.
pub fn register_load_with_declarations(
    env: &LoaderEnv<'_>,
    declarations: &[(String, Vec<String>, bool)],
) {
    let mut state = env.state().borrow_mut();
    for (follower, targets, enabled) in declarations {
        if let Some(index) = state
            .registered_load_with
            .iter()
            .position(|registration| registration.addon_name == *follower)
        {
            if targets.is_empty() {
                state.registered_load_with.remove(index);
            } else {
                let registration = &mut state.registered_load_with[index];
                registration.enabled = *enabled;
                registration.targets.clone_from(targets);
            }
        } else if !targets.is_empty() {
            state
                .registered_load_with
                .push(crate::lua_api::state::RegisteredLoadWith {
                    addon_name: follower.clone(),
                    enabled: *enabled,
                    targets: targets.clone(),
                });
        }
    }
    if !declarations.is_empty() {
        state.load_with_followers = None;
    }
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
    pull_startup_prerequisites(&mut addons, &mut pool, &dependencies, screen);
    let bootstrap_names = insert_bootstrap_nodes(&mut addons, &mut pool, screen);
    topological_sort_addons_with_extra_dependencies_for_screen(addons, &dependencies, screen)
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
    if !is_builtin_addon_folder(name) || is_addon_excluded_for_active_profile(name) {
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
    let toc = TocFile::from_file(toc_path)?;
    validate_startup_dependency_screens(env, toc_path, &toc)?;
    match kind {
        StartupAddonLoadKind::BootstrapOnly => load_addon_bootstrap_from_toc(env, &toc),
        StartupAddonLoadKind::Full => {
            let result = super::load_addon_path_with_toc(env, toc_path, &toc, saved_vars)?;
            let addon_name = toc_path
                .parent()
                .and_then(Path::file_name)
                .and_then(|name| name.to_str())
                .unwrap_or(&toc.name);
            if uses_verified_load_with_semantics() {
                fire_startup_addon_loaded(env, addon_name);
            }
            crate::c_api::c_addons::load_startup_addon_followers(env, addon_name);
            Ok(result)
        }
    }
}

fn fire_startup_addon_loaded(env: &LoaderEnv<'_>, addon_name: &str) {
    let name_value = env.with_state(|state| {
        Ok::<_, crate::Error>(crate::lua_api::methods::create_string(state, addon_name))
    });
    let name_value = match name_value {
        Ok(value) => value,
        Err(error) => {
            record_startup_addon_loaded_error(env, addon_name, error);
            return;
        }
    };
    let arguments =
        crate::loader::addon_loaded_arguments(&env.state().borrow(), addon_name, name_value);
    if let Err(error) = env.fire_event_with_args("ADDON_LOADED", &arguments) {
        record_startup_addon_loaded_error(env, addon_name, error);
    }
}

fn record_startup_addon_loaded_error(
    env: &LoaderEnv<'_>,
    addon_name: &str,
    error: impl std::fmt::Display,
) {
    let message = format!("Error firing ADDON_LOADED for {addon_name}: {error}");
    env.state()
        .borrow_mut()
        .mark_addon_initialization_failed(addon_name, message.clone());
    tracing::warn!("{message}");
}

fn validate_startup_dependency_screens(
    env: &LoaderEnv<'_>,
    toc_path: &Path,
    toc: &TocFile,
) -> Result<(), LoadError> {
    let Some(addon_root) = toc_path.parent().and_then(Path::parent) else {
        return Ok(());
    };
    let screen = env.state().borrow().screen_kind;
    let mut checked = HashSet::new();
    if let Some(name) = toc_path
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
    {
        checked.insert(name.to_owned());
    }
    validate_required_dependency_screens(env, addon_root, toc, screen, &mut checked)
}

fn validate_required_dependency_screens(
    env: &LoaderEnv<'_>,
    addon_root: &Path,
    toc: &TocFile,
    screen: ScreenKind,
    checked: &mut HashSet<String>,
) -> Result<(), LoadError> {
    for dependency in toc.dependencies_for_screen(screen) {
        let is_loaded = env
            .state()
            .borrow()
            .addons
            .iter()
            .any(|addon| addon.folder_name == dependency && addon.loaded);
        if is_loaded || !checked.insert(dependency.clone()) {
            continue;
        }
        let Some(dependency_path) = super::find_toc_file(&addon_root.join(&dependency)) else {
            continue;
        };
        let Ok(dependency_toc) = TocFile::from_file(&dependency_path) else {
            continue;
        };
        if !dependency_toc.allows_screen(screen) {
            return Err(LoadError::DepWrongActiveInterface(dependency));
        }
        validate_required_dependency_screens(env, addon_root, &dependency_toc, screen, checked)?;
    }
    Ok(())
}
