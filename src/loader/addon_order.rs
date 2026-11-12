use crate::screen::ScreenKind;
use crate::toc::TocFile;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Sort addons by available `LoadManagers`, optional, and required TOC entries.
///
/// Addons whose dependencies aren't in the list are treated as having no deps (they load early).
/// Ties are broken alphabetically for deterministic output.
pub fn sort_addons_by_dependencies(addons: &mut Vec<(String, PathBuf)>) {
    let mut toc_map: HashMap<String, (PathBuf, TocFile)> = HashMap::new();
    for (name, toc_path) in addons.iter() {
        if let Ok(toc) = TocFile::from_file(toc_path) {
            toc_map.insert(name.clone(), (toc_path.clone(), toc));
        }
    }

    let available: HashSet<&str> = toc_map.keys().map(|name| name.as_str()).collect();
    let dependencies = build_dependency_graph(&toc_map, &available);
    let load_first = build_load_first_set(&toc_map);
    let sorted = kahns_sort(&dependencies, toc_map.len(), &load_first);

    let name_to_path: HashMap<&str, &PathBuf> = addons
        .iter()
        .map(|(name, path)| (name.as_str(), path))
        .collect();
    let mut result: Vec<(String, PathBuf)> = sorted
        .iter()
        .filter_map(|&name| {
            name_to_path
                .get(name)
                .map(|&path| (name.to_string(), path.clone()))
        })
        .collect();
    for (name, path) in addons.iter() {
        if !toc_map.contains_key(name) {
            result.push((name.clone(), path.clone()));
        }
    }
    *addons = result;
}

/// Order packaged Blizzard addons by their native startup phases.
///
/// Phase one emits `LoadFirst` and `UseSecureEnvironment` addons. Phase two
/// emits the remaining addons from the non-insecure packaged UI input.
pub(super) fn topological_sort_addons(
    addons: HashMap<String, (PathBuf, TocFile)>,
) -> Vec<(String, PathBuf)> {
    let extra_dependencies = HashMap::new();
    topological_sort_addons_with_extra_dependencies(addons, &extra_dependencies)
}

pub(super) fn topological_sort_addons_with_extra_dependencies(
    addons: HashMap<String, (PathBuf, TocFile)>,
    extra_dependencies: &HashMap<String, Vec<String>>,
) -> Vec<(String, PathBuf)> {
    topological_sort_addons_with_screen(addons, extra_dependencies, None)
}

pub(super) fn topological_sort_addons_with_extra_dependencies_for_screen(
    addons: HashMap<String, (PathBuf, TocFile)>,
    extra_dependencies: &HashMap<String, Vec<String>>,
    screen: ScreenKind,
) -> Vec<(String, PathBuf)> {
    topological_sort_addons_with_screen(addons, extra_dependencies, Some(screen))
}

fn topological_sort_addons_with_screen(
    mut addons: HashMap<String, (PathBuf, TocFile)>,
    extra_dependencies: &HashMap<String, Vec<String>>,
    screen: Option<ScreenKind>,
) -> Vec<(String, PathBuf)> {
    let mut result = Vec::with_capacity(addons.len());
    let mut loaded: HashSet<String> = HashSet::new();
    let mut visiting: HashSet<String> = HashSet::new();

    emit_early_addons(
        &mut addons,
        extra_dependencies,
        screen,
        &mut result,
        &mut loaded,
        &mut visiting,
    );
    emit_remaining_addons(
        &mut addons,
        extra_dependencies,
        screen,
        &mut result,
        &mut loaded,
        &mut visiting,
    );
    result
}

fn emit_early_addons(
    addons: &mut HashMap<String, (PathBuf, TocFile)>,
    extra_dependencies: &HashMap<String, Vec<String>>,
    screen: Option<ScreenKind>,
    result: &mut Vec<(String, PathBuf)>,
    loaded: &mut HashSet<String>,
    visiting: &mut HashSet<String>,
) {
    let mut early: Vec<String> = addons
        .iter()
        .filter_map(|(name, (_, toc))| {
            (toc.is_load_first() || toc.is_secure_env()).then_some(name.clone())
        })
        .collect();
    early.sort();
    for name in early {
        emit_addon_recursive(
            &name,
            addons,
            extra_dependencies,
            screen,
            result,
            loaded,
            visiting,
        );
    }
}

fn emit_remaining_addons(
    addons: &mut HashMap<String, (PathBuf, TocFile)>,
    extra_dependencies: &HashMap<String, Vec<String>>,
    screen: Option<ScreenKind>,
    result: &mut Vec<(String, PathBuf)>,
    loaded: &mut HashSet<String>,
    visiting: &mut HashSet<String>,
) {
    let mut remaining: Vec<String> = addons.keys().cloned().collect();
    remaining.sort();
    for name in remaining {
        emit_addon_recursive(
            &name,
            addons,
            extra_dependencies,
            screen,
            result,
            loaded,
            visiting,
        );
    }
}

fn emit_addon_recursive(
    name: &str,
    addons: &mut HashMap<String, (PathBuf, TocFile)>,
    extra_dependencies: &HashMap<String, Vec<String>>,
    screen: Option<ScreenKind>,
    result: &mut Vec<(String, PathBuf)>,
    loaded: &mut HashSet<String>,
    visiting: &mut HashSet<String>,
) {
    if loaded.contains(name) || !addons.contains_key(name) {
        return;
    }
    if !visiting.insert(name.to_string()) {
        return;
    }

    let deps = collect_emit_dependencies(name, addons, extra_dependencies, screen);
    for dep in deps {
        emit_addon_recursive(
            &dep,
            addons,
            extra_dependencies,
            screen,
            result,
            loaded,
            visiting,
        );
    }

    visiting.remove(name);

    if let Some((toc_path, _)) = addons.remove(name) {
        result.push((name.to_string(), toc_path));
        loaded.insert(name.to_string());
    }
}

fn collect_emit_dependencies(
    name: &str,
    addons: &HashMap<String, (PathBuf, TocFile)>,
    extra_dependencies: &HashMap<String, Vec<String>>,
    screen: Option<ScreenKind>,
) -> Vec<String> {
    let Some((_, toc)) = addons.get(name) else {
        return Vec::new();
    };

    let mut seen = HashSet::new();
    let mut deps = Vec::new();
    let optional = screen.map_or_else(
        || toc.optional_deps(),
        |screen| toc.optional_deps_for_screen(screen),
    );
    let required = screen.map_or_else(
        || toc.dependencies(),
        |screen| toc.dependencies_for_screen(screen),
    );
    append_emit_dependencies(&mut deps, &mut seen, toc.load_managers(), addons);
    append_emit_dependencies(&mut deps, &mut seen, optional, addons);
    append_emit_dependencies(&mut deps, &mut seen, required, addons);
    append_emit_dependencies(
        &mut deps,
        &mut seen,
        extra_dependencies.get(name).cloned().unwrap_or_default(),
        addons,
    );
    deps
}

fn append_emit_dependencies(
    deps: &mut Vec<String>,
    seen: &mut HashSet<String>,
    candidates: Vec<String>,
    addons: &HashMap<String, (PathBuf, TocFile)>,
) {
    for dep in candidates {
        if addons.contains_key(&dep) && seen.insert(dep.clone()) {
            deps.push(dep);
        }
    }
}

/// Build available manager and dependency edges for startup sorting.
fn build_dependency_graph<'a>(
    addons: &'a HashMap<String, (PathBuf, TocFile)>,
    available: &HashSet<&'a str>,
) -> HashMap<&'a str, Vec<&'a str>> {
    addons
        .iter()
        .map(|(name, (_, toc))| {
            let mut seen = HashSet::new();
            let mut deps = Vec::new();
            for candidate in toc
                .load_managers()
                .into_iter()
                .chain(toc.optional_deps())
                .chain(toc.dependencies())
            {
                if let Some(&dependency) = available.get(candidate.as_str())
                    && seen.insert(dependency)
                {
                    deps.push(dependency);
                }
            }
            (name.as_str(), deps)
        })
        .collect()
}

fn build_load_first_set(addons: &HashMap<String, (PathBuf, TocFile)>) -> HashSet<&str> {
    addons
        .iter()
        .filter_map(|(name, (_, toc))| toc.is_load_first().then_some(name.as_str()))
        .collect()
}

fn addon_priority_cmp(a: &str, b: &str, load_first: &HashSet<&str>) -> Ordering {
    match (load_first.contains(a), load_first.contains(b)) {
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        _ => b.cmp(a),
    }
}

fn insert_by_priority<'a>(queue: &mut Vec<&'a str>, name: &'a str, load_first: &HashSet<&'a str>) {
    let pos = queue.partition_point(|&existing| {
        addon_priority_cmp(existing, name, load_first) == Ordering::Less
    });
    queue.insert(pos, name);
}

/// Run Kahn's algorithm on a dependency graph. Returns names in topological order.
/// Ties are broken by `LoadFirst`, then alphabetically. If the remaining graph
/// contains a cycle, we still emit every addon by breaking the cycle using the
/// same priority order.
fn kahns_sort<'a>(
    deps: &HashMap<&'a str, Vec<&'a str>>,
    count: usize,
    load_first: &HashSet<&'a str>,
) -> Vec<&'a str> {
    let (mut in_degree, dependents) = build_kahn_state(deps);
    let mut queue = build_zero_degree_queue(&in_degree, load_first);

    let mut result = Vec::with_capacity(count);
    let mut emitted: HashSet<&str> = HashSet::new();
    while result.len() < count {
        let Some(name) = next_kahn_node(&mut queue, &in_degree, &emitted, load_first) else {
            break;
        };

        if !emitted.insert(name) {
            continue;
        }
        result.push(name);
        release_dependents(name, &dependents, &mut in_degree, &mut queue, load_first);
    }

    result
}

fn build_kahn_state<'a>(
    deps: &HashMap<&'a str, Vec<&'a str>>,
) -> (HashMap<&'a str, usize>, HashMap<&'a str, Vec<&'a str>>) {
    let mut in_degree: HashMap<&str, usize> = deps.keys().map(|&name| (name, 0)).collect();
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

    for (&node, reqs) in deps {
        *in_degree.entry(node).or_default() = reqs.len();
        for &requirement in reqs {
            dependents.entry(requirement).or_default().push(node);
        }
    }

    (in_degree, dependents)
}

fn release_dependents<'a>(
    name: &'a str,
    dependents: &HashMap<&'a str, Vec<&'a str>>,
    in_degree: &mut HashMap<&'a str, usize>,
    queue: &mut Vec<&'a str>,
    load_first: &HashSet<&'a str>,
) {
    let Some(nodes) = dependents.get(name) else {
        return;
    };

    for &dependent in nodes {
        if let Some(degree) = in_degree.get_mut(dependent) {
            *degree = degree.saturating_sub(1);
            if *degree == 0 {
                insert_by_priority(queue, dependent, load_first);
            }
        }
    }
}

fn build_zero_degree_queue<'a>(
    in_degree: &HashMap<&'a str, usize>,
    load_first: &HashSet<&'a str>,
) -> Vec<&'a str> {
    let mut queue: Vec<&str> = in_degree
        .iter()
        .filter(|&(_, deg)| *deg == 0)
        .map(|(&name, _)| name)
        .collect();
    queue.sort_by(|a, b| addon_priority_cmp(a, b, load_first));
    queue
}

fn next_kahn_node<'a>(
    queue: &mut Vec<&'a str>,
    in_degree: &HashMap<&'a str, usize>,
    emitted: &HashSet<&'a str>,
    load_first: &HashSet<&'a str>,
) -> Option<&'a str> {
    queue.pop().or_else(|| {
        in_degree
            .keys()
            .filter(|name| !emitted.contains(**name))
            .max_by(|a, b| addon_priority_cmp(a, b, load_first))
            .copied()
    })
}
