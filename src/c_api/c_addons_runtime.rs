use crate::loader::LoadError;
use crate::lua_api::LoaderEnv;
use crate::lua_api::methods::{borrow_lua, borrow_state, create_string, state_handle};
use crate::screen::ScreenKind;
use crate::toc::TocFile;
use rilua::vm::state::LuaState;
use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};

pub(super) fn load_runtime_addon(state: &mut LuaState, addon_name: &str) -> Result<(), LoadError> {
    let loader_env = LoaderEnv::from_parts_active(
        borrow_lua(state).map_err(|err| LoadError::Lua(err.to_string()))?,
        state_handle(state).map_err(|err| LoadError::Lua(err.to_string()))?,
        state,
    );
    let previous_errors = loader_env.state().borrow().lua_error_records.len();
    crate::lua_api::workarounds::apply_for_runtime_addon_preload(&loader_env, addon_name);

    let result = load_runtime_addon_recursive(state, &loader_env, addon_name);
    let mut sim = loader_env.state().borrow_mut();
    if let Err(error) = &result {
        sim.mark_addon_initialization_failed(
            addon_name,
            format!("runtime addon {addon_name} failed: {error}"),
        );
    }
    let errors = sim.lua_error_records[previous_errors..].to_vec();
    for error in errors {
        sim.mark_addon_initialization_failed(
            error.addon_name.as_deref().unwrap_or(addon_name),
            format!(
                "runtime addon {addon_name} initialization failed: {}",
                error.message
            ),
        );
    }
    result
}

fn load_runtime_addon_recursive(
    state: &mut LuaState,
    loader_env: &LoaderEnv<'_>,
    addon_name: &str,
) -> Result<(), LoadError> {
    if is_addon_loaded_by_name(state, addon_name) || is_addon_loading_by_name(state, addon_name) {
        return Ok(());
    }

    load_runtime_addon_with_dependencies(state, loader_env, addon_name)
}

fn load_runtime_addon_with_dependencies(
    state: &mut LuaState,
    loader_env: &LoaderEnv<'_>,
    addon_name: &str,
) -> Result<(), LoadError> {
    let origin = crate::loader::runtime_load_addon_origin(
        borrow_state(state)
            .map(|sim| sim.xml_load_addon_depth)
            .unwrap_or(0),
    );
    crate::loader::trace_load_addon(origin, format!("begin {addon_name}"));
    let (toc_path, toc, screen) = read_runtime_addon_toc(state, addon_name, origin)?;
    let loading_guard = crate::loader::begin_addon_load(loader_env, addon_name, &toc);
    apply_mists_runtime_preload(loader_env, &toc, &toc_path, origin)?;
    load_runtime_addon_dependencies(state, loader_env, addon_name, &toc, screen, origin)?;
    let mut result =
        load_runtime_addon_files_and_mark(state, loader_env, addon_name, &toc, origin)?;
    loading_guard.commit_loaded();
    fire_addon_loaded(state, loader_env, addon_name);
    finalize_runtime_addon_load(
        loader_env,
        addon_name,
        origin,
        loading_guard.addon_index(),
        &mut result,
    );
    load_with_followers(state, loader_env, addon_name);
    Ok(())
}

pub(super) fn load_with_followers(state: &mut LuaState, loader_env: &LoaderEnv<'_>, target: &str) {
    if loader_env.state().borrow().load_with_followers.is_none() {
        let followers = scan_load_with_followers(state, loader_env);
        loader_env.state().borrow_mut().load_with_followers = Some(followers);
    }

    let followers = loader_env
        .state()
        .borrow()
        .load_with_followers
        .as_ref()
        .and_then(|map| map.get(target))
        .cloned()
        .unwrap_or_default();
    for follower in followers {
        if super::addon_is_disabled(state, &follower) {
            continue;
        }
        submit_load_with_follower(state, &follower);
    }
}

fn submit_load_with_follower(state: &mut LuaState, addon_name: &str) {
    let (top, base, ci) = (state.top, state.base, state.ci);
    let result = load_runtime_addon(state, addon_name);
    state.top = top;
    state.base = base;
    state.ci = ci;
    if let Err(error) = result {
        tracing::warn!("LoadWith follower {addon_name} failed: {error}");
    }
}

fn scan_load_with_followers(
    state: &LuaState,
    loader_env: &LoaderEnv<'_>,
) -> HashMap<String, Vec<String>> {
    let bases = {
        let sim = loader_env.state().borrow();
        if sim.addon_base_paths.is_empty() {
            super::default_runtime_addon_bases()
        } else {
            sim.addon_base_paths.clone()
        }
    };
    let addon_names = scan_addon_names(&bases);
    let declarations = read_load_with_declarations(state, addon_names);
    index_load_with_declarations(declarations)
}

fn scan_addon_names(bases: &[PathBuf]) -> Vec<String> {
    let mut addon_names = HashSet::new();
    for base in bases {
        collect_addon_names_from_root(base, &mut addon_names);
    }

    let mut addon_names: Vec<_> = addon_names.into_iter().collect();
    addon_names.sort();
    addon_names
}

fn collect_addon_names_from_root(base: &Path, addon_names: &mut HashSet<String>) {
    let entries = match std::fs::read_dir(base) {
        Ok(entries) => entries,
        Err(_) if !base.is_dir() => return,
        Err(error) => {
            tracing::warn!(
                "Cannot inspect addon root {} for LoadWith: {error}",
                base.display()
            );
            return;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                tracing::warn!(
                    "Cannot inspect an addon entry under {}: {error}",
                    base.display()
                );
                continue;
            }
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        addon_names.insert(name.to_owned());
    }
}

fn read_load_with_declarations(
    state: &LuaState,
    addon_names: Vec<String>,
) -> Vec<(String, Vec<String>)> {
    let mut declarations = Vec::new();
    for addon_name in addon_names {
        let Some(toc_path) = super::find_runtime_addon_toc_ignoring_screen(state, &addon_name)
        else {
            continue;
        };
        let toc = match TocFile::from_file(&toc_path) {
            Ok(toc) => toc,
            Err(error) => {
                tracing::warn!(
                    "Cannot read addon TOC {} for LoadWith: {error}",
                    toc_path.display()
                );
                continue;
            }
        };
        let targets = toc.load_with();
        if !targets.is_empty() {
            declarations.push((addon_name, targets));
        }
    }
    declarations
}

fn index_load_with_declarations(
    declarations: Vec<(String, Vec<String>)>,
) -> HashMap<String, Vec<String>> {
    let mut followers = HashMap::<String, Vec<String>>::new();
    for (follower, targets) in declarations {
        for target in targets {
            followers.entry(target).or_default().push(follower.clone());
        }
    }
    for names in followers.values_mut() {
        names.sort();
        names.dedup();
    }
    followers
}

fn read_runtime_addon_toc(
    state: &LuaState,
    addon_name: &str,
    origin: crate::loader::LoadAddonTraceOrigin,
) -> Result<(PathBuf, TocFile, ScreenKind), LoadError> {
    let toc_path = super::find_runtime_addon_toc_ignoring_screen(state, addon_name)
        .ok_or_else(|| missing_runtime_addon_error(addon_name))?;
    crate::loader::trace_load_addon(origin, format!("toc {}", toc_path.display()));
    let toc = TocFile::from_file(&toc_path).map_err(LoadError::Toc)?;
    let screen = borrow_state(state)
        .map(|sim| sim.screen_kind)
        .unwrap_or(ScreenKind::Game);
    if !toc.allows_screen(screen) {
        return Err(LoadError::WrongActiveInterface);
    }
    Ok((toc_path, toc, screen))
}

fn load_runtime_addon_dependencies(
    state: &mut LuaState,
    loader_env: &LoaderEnv<'_>,
    addon_name: &str,
    toc: &TocFile,
    screen: ScreenKind,
    origin: crate::loader::LoadAddonTraceOrigin,
) -> Result<(), LoadError> {
    for dependency in runtime_foundation_dependencies(state, addon_name) {
        crate::loader::trace_load_addon(origin, format!("{addon_name} -> foundation {dependency}"));
        load_runtime_addon_recursive(state, loader_env, dependency)?;
    }
    for dependency in toc.optional_deps_for_screen(screen) {
        crate::loader::trace_load_addon(origin, format!("{addon_name} -> optional {dependency}"));
        if !super::addon_is_disabled(state, &dependency) {
            // Native R13 ignores failures from optional dependency loads.
            let _ = load_runtime_addon_recursive(state, loader_env, &dependency);
        }
    }
    for dependency in toc.dependencies_for_screen(screen) {
        crate::loader::trace_load_addon(origin, format!("{addon_name} -> dep {dependency}"));
        if super::addon_is_disabled(state, &dependency) {
            return Err(disabled_dep_error(&dependency));
        }
        match load_runtime_addon_recursive(state, loader_env, &dependency) {
            Err(LoadError::WrongActiveInterface) => {
                return Err(LoadError::DepWrongActiveInterface(dependency));
            }
            result => result?,
        }
    }
    Ok(())
}

fn load_runtime_addon_files_and_mark(
    state: &mut LuaState,
    loader_env: &LoaderEnv<'_>,
    addon_name: &str,
    toc: &TocFile,
    origin: crate::loader::LoadAddonTraceOrigin,
) -> Result<crate::loader::LoadResult, LoadError> {
    crate::loader::trace_load_addon(origin, format!("files {addon_name}"));
    let result = load_runtime_addon_files(state, loader_env, addon_name, toc)?;
    loader_env
        .state()
        .borrow_mut()
        .mark_addon_initialized(addon_name);
    crate::lua_api::workarounds::apply_for_runtime_addon_load(loader_env, addon_name);
    Ok(result)
}

fn finalize_runtime_addon_load(
    loader_env: &LoaderEnv<'_>,
    addon_name: &str,
    origin: crate::loader::LoadAddonTraceOrigin,
    addon_index: u16,
    result: &mut crate::loader::LoadResult,
) {
    crate::loader::append_pending_nested_addon_diagnostics(loader_env, addon_index, result);
    crate::loader::trace_load_result_diagnostics(origin, addon_name, result);
    retain_runtime_load_warning(loader_env, addon_name, result);
    route_finalized_runtime_addon_diagnostics(loader_env, result.diagnostics());
    loader_env.state().borrow_mut().invalidate_strata_buckets();
    crate::loader::trace_load_addon(origin, format!("event {addon_name}"));
    crate::loader::trace_load_addon(origin, format!("loaded {addon_name}"));
}

fn retain_runtime_load_warning(
    loader_env: &LoaderEnv<'_>,
    addon_name: &str,
    result: &crate::loader::LoadResult,
) {
    if let Some(warning) = result.warnings.first() {
        loader_env
            .state()
            .borrow_mut()
            .mark_addon_initialization_failed(
                addon_name,
                format!("runtime addon {addon_name} failed: {warning}"),
            );
    }
}

fn route_finalized_runtime_addon_diagnostics(
    loader_env: &LoaderEnv<'_>,
    diagnostics: crate::loader::LoadDiagnostics,
) {
    if diagnostics.is_empty() {
        return;
    }

    let mut state = loader_env.state().borrow_mut();
    if let Some(parent_addon_index) = state.loading_addon_stack.iter().rev().nth(1).copied() {
        state
            .pending_nested_addon_diagnostics
            .entry(parent_addon_index)
            .or_default()
            .extend(diagnostics);
    } else {
        state.runtime_addon_diagnostics.extend(diagnostics);
    }
}

#[cfg(feature = "client-mists")]
fn apply_mists_runtime_preload(
    loader_env: &LoaderEnv<'_>,
    toc: &crate::toc::TocFile,
    toc_path: &std::path::Path,
    origin: crate::loader::LoadAddonTraceOrigin,
) -> Result<(), LoadError> {
    if let Some(result) =
        crate::mists::character_frame_preload::ensure_before_addon(loader_env, toc, toc_path)?
    {
        crate::loader::trace_load_result_diagnostics(origin, &result.name, &result);
        retain_runtime_load_warning(loader_env, &result.name, &result);
    }
    Ok(())
}

#[cfg(not(feature = "client-mists"))]
fn apply_mists_runtime_preload(
    _loader_env: &LoaderEnv<'_>,
    _toc: &crate::toc::TocFile,
    _toc_path: &std::path::Path,
    _origin: crate::loader::LoadAddonTraceOrigin,
) -> Result<(), LoadError> {
    Ok(())
}

fn load_runtime_addon_files(
    state: &mut LuaState,
    loader_env: &LoaderEnv<'_>,
    _addon_name: &str,
    toc: &crate::toc::TocFile,
) -> Result<crate::loader::LoadResult, LoadError> {
    if !toc.loads_as_blizzard_code() {
        return crate::loader::load_addon_from_toc(loader_env, toc);
    }

    let saved_taints = crate::lua_api::taint::clear_active_stack_taint(state);
    let result = crate::loader::load_addon_from_toc(loader_env, toc);
    crate::lua_api::taint::restore_active_stack_taint(state, saved_taints);
    result
}

fn runtime_foundation_dependencies(state: &LuaState, addon_name: &str) -> Vec<&'static str> {
    if !addon_name.starts_with("Blizzard_") {
        return Vec::new();
    }

    let screen_kind = borrow_state(state)
        .ok()
        .map(|sim| sim.screen_kind)
        .unwrap_or(crate::screen::ScreenKind::Game);
    let foundations = match screen_kind {
        crate::screen::ScreenKind::Game => super::GAME_RUNTIME_FOUNDATIONS,
        crate::screen::ScreenKind::Login
        | crate::screen::ScreenKind::CharacterSelect
        | crate::screen::ScreenKind::CharacterCreate => super::GLUE_RUNTIME_FOUNDATIONS,
    };
    let end = foundations
        .iter()
        .position(|candidate| *candidate == addon_name)
        .unwrap_or(foundations.len());
    foundations[..end]
        .iter()
        .copied()
        .filter(|dependency| super::find_runtime_addon_toc(state, dependency).is_some())
        .collect()
}

fn is_addon_loaded_by_name(state: &LuaState, addon_name: &str) -> bool {
    borrow_state(state)
        .ok()
        .and_then(|sim| {
            sim.addons
                .iter()
                .find(|addon| addon.folder_name == addon_name)
                .map(|addon| addon.loaded)
        })
        .unwrap_or(false)
}

fn is_addon_loading_by_name(state: &LuaState, addon_name: &str) -> bool {
    borrow_state(state)
        .map(|sim| sim.is_addon_loading(addon_name))
        .unwrap_or(false)
}

fn missing_runtime_addon_error(addon_name: &str) -> LoadError {
    LoadError::Io(io::Error::new(
        io::ErrorKind::NotFound,
        format!("runtime addon not found: {addon_name}"),
    ))
}

fn disabled_dep_error(dependency: &str) -> LoadError {
    LoadError::DepDisabled(dependency.to_string())
}

fn fire_addon_loaded(state: &mut LuaState, loader_env: &LoaderEnv<'_>, addon_name: &str) {
    let saved_top = state.top;
    let addon_name_val = create_string(state, addon_name);
    let arguments = crate::loader::addon_loaded_arguments(
        &loader_env.state().borrow(),
        addon_name,
        addon_name_val,
    );
    if let Err(error) = loader_env.fire_event_with_args("ADDON_LOADED", &arguments) {
        loader_env
            .state()
            .borrow_mut()
            .mark_addon_initialization_failed(
                addon_name,
                format!(
                    "runtime addon {addon_name} initialization callback dispatch failed: {error}"
                ),
            );
    }
    state.top = saved_top;
}

#[cfg(test)]
#[path = "c_addons_runtime_tests.rs"]
mod tests;
