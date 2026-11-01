//! Blizzard LoadOnDemand addon coverage shards.

use crate::common;

use std::collections::{BTreeMap, HashSet};
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_addon_closure_for_screen,
    discover_blizzard_addons, load_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_errors::grouped_errors_by_addon;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::settle_headless_startup;
use wow_ui_sim::toc::TocFile;
use wow_ui_sim::xml::{
    FrameXml, clear_templates, get_template, register_intrinsic_templates, register_template,
};

const KNOWN_LOAD_ON_DEMAND_RUNTIME_ERRORS: &[(&str, usize)] = &[
    ("Blizzard_AzeriteEssenceUI", 6),
    ("Blizzard_BoostTutorial", 8),
    ("Blizzard_EncounterJournal", 1),
    ("Blizzard_EventTrace", 4),
    ("Blizzard_ExpansionTrial", 4),
    ("Blizzard_HouseEditor", 1),
    ("Blizzard_ItemBeltFrame", 4),
    ("Blizzard_ItemInteractionUI", 6),
    ("Blizzard_MatchCelebrationPartyPoseUI", 1),
    ("Blizzard_Professions", 16),
    ("Blizzard_RuneforgeUI", 3),
    ("Blizzard_ScrappingMachineUI", 2),
    ("Blizzard_TimerunningCharacterCreate", 4),
];

#[derive(Clone, Debug, PartialEq, Eq)]
struct LoadOnDemandAddonClosure {
    root: String,
    addons: Vec<String>,
}

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn known_load_on_demand_runtime_error_counts() -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::from_iter(
        common::addon_coverage_baseline::KNOWN_ERRORS
            .iter()
            .map(|(addon, count)| ((*addon).to_string(), *count)),
    );
    counts.extend(
        KNOWN_LOAD_ON_DEMAND_RUNTIME_ERRORS
            .iter()
            .map(|(addon, count)| ((*addon).to_string(), *count)),
    );
    counts
}

fn actual_error_counts(grouped_errors: &BTreeMap<String, Vec<String>>) -> BTreeMap<String, usize> {
    grouped_errors
        .iter()
        .map(|(addon, errors)| (addon.clone(), errors.len()))
        .collect()
}

fn format_per_addon_report(grouped_errors: &BTreeMap<String, Vec<String>>) -> String {
    let mut rows: Vec<_> = grouped_errors.iter().collect();
    rows.sort_by(|(left_name, left_errors), (right_name, right_errors)| {
        right_errors
            .len()
            .cmp(&left_errors.len())
            .then_with(|| left_name.cmp(right_name))
    });

    rows.into_iter()
        .map(|(addon_name, errors)| {
            let sample = errors.first().map(String::as_str).unwrap_or("<no sample>");
            format!("{addon_name}: {} error(s); sample: {sample}", errors.len())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_error_count_map(error_counts: &BTreeMap<String, usize>) -> String {
    error_counts
        .iter()
        .map(|(addon, count)| format!("{addon}: {count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_error_count_changes(changes: &[(String, usize, usize)]) -> String {
    changes
        .iter()
        .map(|(addon, known, actual)| format!("{addon}: known={known}, actual={actual}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn classify_error_count_increases_from_baseline(
    known: &BTreeMap<String, usize>,
    actual: &BTreeMap<String, usize>,
) -> Vec<(String, usize, usize)> {
    actual
        .iter()
        .filter_map(|(addon_name, actual_count)| {
            let known_count = known.get(addon_name).copied().unwrap_or(0);
            (actual_count > &known_count).then(|| (addon_name.clone(), known_count, *actual_count))
        })
        .collect()
}

fn discover_blizzard_lod_addon_tocs() -> Vec<(String, TocFile)> {
    discover_all_blizzard_addons(&blizzard_ui_dir())
        .into_iter()
        .filter_map(|(name, toc_path)| {
            let toc = TocFile::from_file(&toc_path).ok()?;
            (toc.is_load_on_demand()
                && toc.allows_screen(ScreenKind::Game)
                && !toc.is_ptr_only()
                && !toc.is_game_type_restricted())
            .then_some((name, toc))
        })
        .collect()
}

fn discover_blizzard_lod_addon_closures() -> Vec<LoadOnDemandAddonClosure> {
    let ui = blizzard_ui_dir();
    discover_blizzard_lod_addon_tocs()
        .into_iter()
        .map(|(root, _)| {
            let addons =
                discover_blizzard_addon_closure_for_screen(&ui, ScreenKind::Game, &[root.as_str()])
                    .into_iter()
                    .map(|(name, _)| name)
                    .collect();

            LoadOnDemandAddonClosure { root, addons }
        })
        .collect()
}

fn clear_lua_error_tracking(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.lua_errors.clear();
    state.lua_error_records.clear();
    state.lua_error_counts.clear();
}

fn silence_lua_error_handler(env: &WowLuaEnv) {
    env.exec("seterrorhandler(function() end)")
        .expect("seterrorhandler should accept a no-op test handler");
}

fn reset_template_state() {
    clear_templates();
    register_intrinsic_templates();
}

fn with_isolated_addon_coverage_state(f: impl FnOnce()) {
    reset_template_state();
    let result = panic::catch_unwind(AssertUnwindSafe(f));
    reset_template_state();
    if let Err(payload) = result {
        panic::resume_unwind(payload);
    }
}

fn load_startup_blizzard_ui(env: &WowLuaEnv) -> HashSet<String> {
    reset_template_state();
    if let Some(toc) = wow_ui_sim::client_profile::blizzard_ui_framexml_toc() {
        load_addon(&env.loader_env(), &toc).expect("profile FrameXML should load before addons");
    }
    let startup_addons = discover_blizzard_addons(&blizzard_ui_dir());
    let mut load_failures = Vec::new();
    for (name, toc_path) in &startup_addons {
        if let Err(error) = load_addon(&env.loader_env(), toc_path) {
            load_failures.push(format!("{name}: {error}"));
        }
    }

    assert!(
        load_failures.is_empty(),
        "startup Blizzard addon load should not have hard TOC load failures:\n{}",
        load_failures.join("\n"),
    );

    env.apply_post_load_workarounds();
    settle_headless_startup(env);
    silence_lua_error_handler(env);
    startup_addons.into_iter().map(|(name, _)| name).collect()
}

fn is_addon_loaded(env: &WowLuaEnv, addon_name: &str) -> bool {
    env.eval(&format!("return C_AddOns.IsAddOnLoaded({addon_name:?})"))
        .expect("C_AddOns.IsAddOnLoaded should return")
}

fn load_on_demand_shard_weight(
    addon_name: &str,
    known_runtime_counts: &BTreeMap<String, usize>,
) -> usize {
    known_runtime_counts
        .get(addon_name)
        .copied()
        .unwrap_or(0)
        .max(1)
}

fn closure_runtime_weight(
    closure: &LoadOnDemandAddonClosure,
    known_runtime_counts: &BTreeMap<String, usize>,
) -> usize {
    closure
        .addons
        .iter()
        .map(|addon_name| load_on_demand_shard_weight(addon_name, known_runtime_counts))
        .sum::<usize>()
        .max(1)
}

fn shard_load_on_demand_addon_closures(
    lod_closures: &[LoadOnDemandAddonClosure],
    shard_count: usize,
    known_runtime_counts: &BTreeMap<String, usize>,
) -> Vec<Vec<LoadOnDemandAddonClosure>> {
    let mut weighted_closures: Vec<_> = lod_closures
        .iter()
        .enumerate()
        .map(|(original_index, closure)| {
            (
                original_index,
                closure.clone(),
                closure_runtime_weight(closure, known_runtime_counts),
            )
        })
        .collect();

    weighted_closures.sort_by(
        |(left_index, _, left_weight), (right_index, _, right_weight)| {
            right_weight
                .cmp(left_weight)
                .then_with(|| left_index.cmp(right_index))
        },
    );

    let mut shard_weights = vec![0usize; shard_count];
    let mut shards: Vec<Vec<(usize, LoadOnDemandAddonClosure)>> = vec![Vec::new(); shard_count];
    for (original_index, closure, weight) in weighted_closures {
        let shard_index = (0..shard_count)
            .min_by_key(|&index| (shard_weights[index], shards[index].len(), index))
            .expect("shard_count should be non-zero");
        shard_weights[shard_index] += weight;
        shards[shard_index].push((original_index, closure));
    }

    shards
        .into_iter()
        .map(|mut shard| {
            shard.sort_by_key(|(original_index, _)| *original_index);
            shard.into_iter().map(|(_, closure)| closure).collect()
        })
        .collect()
}

fn closure_has_unloaded_addons(env: &WowLuaEnv, closure: &LoadOnDemandAddonClosure) -> bool {
    closure
        .addons
        .iter()
        .any(|addon_name| !is_addon_loaded(env, addon_name))
}

fn load_addon_root_for_closure(
    env: &WowLuaEnv,
    closure: &LoadOnDemandAddonClosure,
    load_failures: &mut Vec<String>,
) -> Option<String> {
    if !closure_has_unloaded_addons(env, closure) {
        return None;
    }

    let (loaded, reason) = load_runtime_addon_root(env, closure);
    if !loaded && !is_known_runtime_load_gap(&closure.root) {
        load_failures.push(format_load_failure(&closure.root, reason.as_deref()));
    }

    Some(closure.root.clone())
}

fn load_runtime_addon_root(
    env: &WowLuaEnv,
    closure: &LoadOnDemandAddonClosure,
) -> (bool, Option<String>) {
    env.eval(&format!("return C_AddOns.LoadAddOn({:?})", closure.root))
        .unwrap_or_else(|error| recover_loaded_addon_or_panic(env, closure, error))
}

fn recover_loaded_addon_or_panic(
    env: &WowLuaEnv,
    closure: &LoadOnDemandAddonClosure,
    error: impl std::fmt::Debug,
) -> (bool, Option<String>) {
    if is_addon_loaded(env, &closure.root) {
        return (true, None);
    }

    let load_addon_details = load_addon_debug_source(env);
    panic!(
        "{}: C_AddOns.LoadAddOn should return ({load_addon_details}): {error:?}",
        closure.root,
    )
}

fn load_addon_debug_source(env: &WowLuaEnv) -> String {
    env.eval(
        r#"
        local loadAddOn = C_AddOns and C_AddOns.LoadAddOn
        local info = type(loadAddOn) == "function" and debug.getinfo(loadAddOn, "S") or nil
        return type(loadAddOn) .. " " .. tostring(info and info.source)
        "#,
    )
    .unwrap_or_else(|_| "<unavailable>".to_string())
}

fn is_known_runtime_load_gap(addon_name: &str) -> bool {
    addon_name == "Blizzard_RuneforgeUI"
}

fn format_load_failure(addon_name: &str, reason: Option<&str>) -> String {
    format!(
        "{addon_name}: LoadAddOn returned false ({})",
        reason.unwrap_or("nil"),
    )
}

fn closure_runtime_failure_message(
    env: &WowLuaEnv,
    closure: &LoadOnDemandAddonClosure,
    representative: &str,
    startup_addons: &HashSet<String>,
    known_runtime_counts: &BTreeMap<String, usize>,
) -> Option<String> {
    let state = env.state().borrow();
    let grouped_errors = grouped_errors_by_addon(&state);
    eprintln!("Complete runtime Lua errors for {representative}: {grouped_errors:#?}");
    let actual_counts = actual_error_counts(&grouped_errors);
    let increases =
        classify_error_count_increases_from_baseline(known_runtime_counts, &actual_counts);
    let invalid_addons: Vec<_> = grouped_errors
        .keys()
        .filter(|addon_name| {
            addon_name.as_str() != "<unknown>"
                && !startup_addons.contains(*addon_name)
                && !closure.addons.contains(*addon_name)
        })
        .cloned()
        .collect();
    let unknown_count = grouped_errors.get("<unknown>").map_or(0, Vec::len);

    (unknown_count > 0 || !invalid_addons.is_empty() || !increases.is_empty()).then(|| {
        format!(
            "{representative}: increased [{}], invalid_addons={:?}, unknown_count={}, actual counts=[{}]\n{}",
            format_error_count_changes(&increases),
            invalid_addons,
            unknown_count,
            format_error_count_map(&actual_counts),
            format_per_addon_report(&grouped_errors),
        )
    })
}

fn record_load_on_demand_closure_failures(
    env: &WowLuaEnv,
    closure: &LoadOnDemandAddonClosure,
    startup_addons: &HashSet<String>,
    known_runtime_counts: &BTreeMap<String, usize>,
    closure_failures: &mut Vec<String>,
    load_failures: &mut Vec<String>,
) {
    clear_lua_error_tracking(env);
    let Some(representative) = load_addon_root_for_closure(env, closure, load_failures) else {
        return;
    };

    if let Some(failure) = closure_runtime_failure_message(
        env,
        closure,
        &representative,
        startup_addons,
        known_runtime_counts,
    ) {
        closure_failures.push(failure);
    }
}

fn run_load_on_demand_blizzard_addon_shard_body(shard_index: usize, shard_count: usize) {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    let startup_addons = load_startup_blizzard_ui(&env);
    let known_runtime_counts = known_load_on_demand_runtime_error_counts();
    let lod_closures = discover_blizzard_lod_addon_closures();
    let shards =
        shard_load_on_demand_addon_closures(&lod_closures, shard_count, &known_runtime_counts);
    let mut closure_failures = Vec::new();
    let mut load_failures = Vec::new();

    for closure in &shards[shard_index] {
        record_load_on_demand_closure_failures(
            &env,
            closure,
            &startup_addons,
            &known_runtime_counts,
            &mut closure_failures,
            &mut load_failures,
        );
    }

    assert!(
        load_failures.is_empty() && closure_failures.is_empty(),
        "LoadOnDemand Blizzard addon shard {} of {} exceeded baseline.\nload failures:\n{}\nruntime failures:\n{}",
        shard_index + 1,
        shard_count,
        load_failures.join("\n"),
        closure_failures.join("\n\n"),
    );
}

fn run_load_on_demand_blizzard_addon_shard_without_timeout(shard_index: usize, shard_count: usize) {
    with_isolated_addon_coverage_state(|| {
        common::with_perf_lock(|| {
            run_load_on_demand_blizzard_addon_shard_body(shard_index, shard_count);
        })
    })
}

fn run_load_on_demand_blizzard_addon_shard(shard_index: usize, shard_count: usize) {
    common::with_timeout(600, move || {
        run_load_on_demand_blizzard_addon_shard_without_timeout(shard_index, shard_count);
    })
}

#[cfg(feature = "client-wrath")]
#[test]
fn time_manager_renders_numeric_hour_through_original_dropdown() {
    common::with_timeout(180, || {
        with_isolated_addon_coverage_state(|| {
            let env = WowLuaEnv::new().expect("Lua environment");
            env.set_screen_size(1024.0, 768.0);
            env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
            env.exec(
                r#"
                SetCVar("timeMgrUseMilitaryTime", "0")
                SetCVar("timeMgrAlarmTime", "780")
                SetCVar("timeMgrAlarmEnabled", "0")
                "#,
            )
            .expect("concrete 1 PM alarm input");
            load_startup_blizzard_ui(&env);
            clear_lua_error_tracking(&env);

            let (loaded, reason): (bool, Option<String>) = env
                .eval("return C_AddOns.LoadAddOn('Blizzard_TimeManager')")
                .expect("original TimeManager runtime load returns");
            assert!(loaded, "TimeManager load: {reason:?}");
            let errors = grouped_errors_by_addon(&env.state().borrow());
            assert!(errors.is_empty(), "original TimeManager load: {errors:#?}");
            let hour: String = env
                .eval("return TimeManagerAlarmHourDropDownText:GetText()")
                .expect("original dropdown's rendered hour");
            assert_eq!(hour, "1", "native numeric hour reaches its FontString");
        });
    });
}

#[test]
fn load_on_demand_runtime_baseline_overrides_force_load_counts() {
    let known_runtime_counts = known_load_on_demand_runtime_error_counts();

    assert_eq!(known_runtime_counts.get("Blizzard_EventTrace"), Some(&4));
    assert_eq!(known_runtime_counts.get("Blizzard_Professions"), Some(&16));
    assert_eq!(known_runtime_counts.get("Blizzard_WorldMap"), Some(&2));
}

#[test]
fn shard_load_on_demand_addons_spreads_heavy_addons_across_shards() {
    let lod_closures = vec![
        LoadOnDemandAddonClosure {
            root: "Blizzard_Light".to_string(),
            addons: vec!["Blizzard_Light".to_string()],
        },
        LoadOnDemandAddonClosure {
            root: "Blizzard_HeavyA".to_string(),
            addons: vec!["Blizzard_HeavyA".to_string()],
        },
        LoadOnDemandAddonClosure {
            root: "Blizzard_HeavyB".to_string(),
            addons: vec![
                "Blizzard_HeavyB_Dependency".to_string(),
                "Blizzard_HeavyB".to_string(),
            ],
        },
        LoadOnDemandAddonClosure {
            root: "Blizzard_Medium".to_string(),
            addons: vec!["Blizzard_Medium".to_string()],
        },
    ];
    let known_runtime_counts = BTreeMap::from([
        ("Blizzard_HeavyA".to_string(), 100),
        ("Blizzard_HeavyB".to_string(), 90),
        ("Blizzard_Medium".to_string(), 10),
    ]);

    let shards = shard_load_on_demand_addon_closures(&lod_closures, 2, &known_runtime_counts);

    assert_eq!(shards.len(), 2);
    assert!(
        shards[0]
            .iter()
            .any(|closure| closure.root == "Blizzard_HeavyA")
    );
    assert!(
        shards[1]
            .iter()
            .any(|closure| closure.root == "Blizzard_HeavyB")
    );
    assert!(
        shards.iter().any(|shard| shard.iter().any(|closure| {
            closure.root == "Blizzard_HeavyB"
                && closure
                    .addons
                    .contains(&"Blizzard_HeavyB_Dependency".to_string())
        })),
        "dependency closures should stay together inside a single shard",
    );
}

#[test]
fn closure_has_unloaded_addons_checks_full_dependency_closure() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    let closure = LoadOnDemandAddonClosure {
        root: "Blizzard_First".to_string(),
        addons: vec![
            "Blizzard_First".to_string(),
            "Blizzard_Second".to_string(),
            "Blizzard_Third".to_string(),
        ],
    };

    assert!(closure_has_unloaded_addons(&env, &closure));
}

post_wrath_ui_fixture! {
#[test]
fn generic_trait_ui_runtime_load_survives_prior_force_load_process_state() {
    common::with_perf_lock(|| {
        common::with_timeout(600, move || {
            let env = WowLuaEnv::new().expect("Failed to create Lua environment");
            env.set_screen_size(1024.0, 768.0);
            env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];

            for (_, toc_path) in discover_all_blizzard_addons(&blizzard_ui_dir()) {
                let _ = load_addon(&env.loader_env(), &toc_path);
            }
            drop(env);

            let env = WowLuaEnv::new().expect("Failed to create Lua environment");
            env.set_screen_size(1024.0, 768.0);
            env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
            load_startup_blizzard_ui(&env);

            let was_loaded: bool = env
                .eval("return C_AddOns.IsAddOnLoaded(\"Blizzard_GenericTraitUI\")")
                .expect("precondition query should return");
            let (loaded, reason): (bool, Option<String>) = env
                .eval("return C_AddOns.LoadAddOn(\"Blizzard_GenericTraitUI\")")
                .expect("GenericTraitUI load should return");
            let now_loaded: bool = env
                .eval("return C_AddOns.IsAddOnLoaded(\"Blizzard_GenericTraitUI\")")
                .expect("postcondition query should return");

            assert!(
                loaded && now_loaded,
                "GenericTraitUI should load after a prior force-load pass; was_loaded={was_loaded}, loaded={loaded}, reason={reason:?}, now_loaded={now_loaded}",
            );
        })
    })
}
}

post_wrath_ui_fixture! {
#[test]
fn contribution_runtime_load_survives_post_startup_state() {
    with_isolated_addon_coverage_state(|| {
        common::with_perf_lock(|| {
            common::with_timeout(600, move || {
                let env = WowLuaEnv::new().expect("Failed to create Lua environment");
                env.set_screen_size(1024.0, 768.0);
                env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
                load_startup_blizzard_ui(&env);
                clear_lua_error_tracking(&env);

                let (collector_type, close_type): (String, String) = env
                    .eval(
                        "return type(C_ContributionCollector), type(C_ContributionCollector and C_ContributionCollector.Close)",
                    )
                    .expect("collector shape query should return");
                let (loaded, reason): (bool, Option<String>) = env
                    .eval("return C_AddOns.LoadAddOn(\"Blizzard_Contribution\")")
                    .expect("Blizzard_Contribution load should return");
                let state = env.state().borrow();
                let grouped_errors = grouped_errors_by_addon(&state);

                assert!(
                    loaded,
                    "Blizzard_Contribution should load after startup; collector_type={collector_type}, close_type={close_type}, reason={reason:?}, errors=\n{}",
                    format_per_addon_report(&grouped_errors),
                );
            })
        })
    });
}
}

#[test]
fn shard_14_runtime_load_survives_prior_runtime_shards_in_process() {
    common::with_timeout(600, move || {
        for shard_index in 9..14 {
            run_load_on_demand_blizzard_addon_shard_without_timeout(shard_index, 16);
        }
    })
}

#[test]
fn perf_lock_recovers_after_prior_panicking_holder() {
    let first = panic::catch_unwind(|| {
        common::with_perf_lock(|| panic!("intentional perf lock poison"));
    });
    assert!(first.is_err(), "first perf-lock holder should panic");

    let second = panic::catch_unwind(|| {
        common::with_perf_lock(|| {});
    });
    assert!(
        second.is_ok(),
        "perf lock should recover after a prior panic instead of poisoning later shards"
    );
}

#[test]
fn isolated_shard_runner_resets_template_state_after_panic() {
    let first = panic::catch_unwind(|| {
        with_isolated_addon_coverage_state(|| {
            register_template("PoisonTemplate", "Frame", FrameXml::default());
            assert!(
                get_template("PoisonTemplate").is_some(),
                "test setup should register the synthetic template before the panic"
            );
            panic!("intentional shard failure");
        });
    });
    assert!(first.is_err(), "first isolated shard should panic");

    with_isolated_addon_coverage_state(|| {
        assert!(
            get_template("PoisonTemplate").is_none(),
            "template registry should be reset before the next isolated shard runs"
        );
    });
}

macro_rules! lod_shard_test {
    ($name:ident, $index:expr) => {
        #[test]
        fn $name() {
            run_load_on_demand_blizzard_addon_shard($index, 16);
        }
    };
}

lod_shard_test!(
    load_on_demand_blizzard_addons_shard_1_stays_within_known_error_baseline_after_startup,
    0
);
lod_shard_test!(
    load_on_demand_blizzard_addons_shard_2_stays_within_known_error_baseline_after_startup,
    1
);
lod_shard_test!(
    load_on_demand_blizzard_addons_shard_3_stays_within_known_error_baseline_after_startup,
    2
);
lod_shard_test!(
    load_on_demand_blizzard_addons_shard_4_stays_within_known_error_baseline_after_startup,
    3
);
lod_shard_test!(
    load_on_demand_blizzard_addons_shard_5_stays_within_known_error_baseline_after_startup,
    4
);
lod_shard_test!(
    load_on_demand_blizzard_addons_shard_6_stays_within_known_error_baseline_after_startup,
    5
);
lod_shard_test!(
    load_on_demand_blizzard_addons_shard_7_stays_within_known_error_baseline_after_startup,
    6
);
lod_shard_test!(
    load_on_demand_blizzard_addons_shard_8_stays_within_known_error_baseline_after_startup,
    7
);
lod_shard_test!(
    load_on_demand_blizzard_addons_shard_9_stays_within_known_error_baseline_after_startup,
    8
);
lod_shard_test!(
    load_on_demand_blizzard_addons_shard_10_stays_within_known_error_baseline_after_startup,
    9
);
lod_shard_test!(
    load_on_demand_blizzard_addons_shard_11_stays_within_known_error_baseline_after_startup,
    10
);
lod_shard_test!(
    load_on_demand_blizzard_addons_shard_12_stays_within_known_error_baseline_after_startup,
    11
);
lod_shard_test!(
    load_on_demand_blizzard_addons_shard_13_stays_within_known_error_baseline_after_startup,
    12
);
lod_shard_test!(
    load_on_demand_blizzard_addons_shard_14_stays_within_known_error_baseline_after_startup,
    13
);
lod_shard_test!(
    load_on_demand_blizzard_addons_shard_15_stays_within_known_error_baseline_after_startup,
    14
);
lod_shard_test!(
    load_on_demand_blizzard_addons_shard_16_stays_within_known_error_baseline_after_startup,
    15
);
