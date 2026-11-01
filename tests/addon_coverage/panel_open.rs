use std::collections::BTreeMap;

use super::*;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_errors::grouped_errors_by_addon;

struct PanelOpenCoverageCase {
    name: &'static str,
    open_lua: &'static str,
    expected_addon: &'static str,
    expected_frame: &'static str,
    expected_error_overrides: &'static [(&'static str, usize)],
}

const PANEL_OPEN_COVERAGE_CASES: &[PanelOpenCoverageCase] = &[
    PanelOpenCoverageCase {
        name: "achievement",
        open_lua: "ToggleAchievementFrame()",
        expected_addon: "Blizzard_AchievementUI",
        expected_frame: "AchievementFrame",
        expected_error_overrides: &[("<unknown>", 1)],
    },
    PanelOpenCoverageCase {
        name: "collections",
        open_lua: "ToggleCollectionsJournal(COLLECTIONS_JOURNAL_TAB_INDEX_MOUNTS)",
        expected_addon: "Blizzard_Collections",
        expected_frame: "CollectionsJournal",
        expected_error_overrides: &[("<unknown>", 29)],
    },
    PanelOpenCoverageCase {
        name: "encounter_journal",
        open_lua: "ToggleEncounterJournal()",
        expected_addon: "Blizzard_EncounterJournal",
        expected_frame: "EncounterJournal",
        expected_error_overrides: &[("Blizzard_EncounterJournal", 1)],
    },
];

fn known_panel_open_runtime_error_counts(case: &PanelOpenCoverageCase) -> BTreeMap<String, usize> {
    let mut counts = known_error_counts();
    for (addon_name, count) in case.expected_error_overrides {
        counts.insert((*addon_name).to_string(), *count);
    }
    counts
}

fn frame_is_shown(env: &WowLuaEnv, frame_name: &str) -> bool {
    env.eval(&format!(
        "return _G[{frame_name:?}] ~= nil and _G[{frame_name:?}]:IsShown()"
    ))
    .expect("frame visibility query should return")
}

fn is_addon_loaded(env: &WowLuaEnv, addon_name: &str) -> bool {
    env.eval(&format!("return C_AddOns.IsAddOnLoaded({addon_name:?})"))
        .expect("C_AddOns.IsAddOnLoaded should return")
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

#[test]
fn panel_open_runtime_baseline_overrides_known_side_loads() {
    let collections_case = PANEL_OPEN_COVERAGE_CASES
        .iter()
        .find(|case| case.name == "collections")
        .expect("collections coverage case should exist");
    let known_counts = known_panel_open_runtime_error_counts(collections_case);

    assert_eq!(known_counts.get("<unknown>"), Some(&29));
    assert_eq!(known_counts.get("Blizzard_Collections"), Some(&0));

    let encounter_journal_case = PANEL_OPEN_COVERAGE_CASES
        .iter()
        .find(|case| case.name == "encounter_journal")
        .expect("encounter journal coverage case should exist");
    let known_counts = known_panel_open_runtime_error_counts(encounter_journal_case);

    assert_eq!(known_counts.get("Blizzard_EncounterJournal"), Some(&1));
}

#[test]
fn panel_open_runtime_paths_stay_within_known_error_baseline() {
    let _full_ui = lock_full_ui();
    common::with_perf_lock(|| {
        common::with_timeout(600, move || {
            let env = WowLuaEnv::new().expect("Failed to create Lua environment");
            env.set_screen_size(1024.0, 768.0);
            env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];

            let known_blizzard_addons = load_panel_harness_blizzard_ui(&env);
            let mut case_failures = Vec::new();

            // 12340 has AchievementUI but no Collections/EncounterJournal addons.
            let cases = if cfg!(feature = "client-wrath") {
                &PANEL_OPEN_COVERAGE_CASES[..1]
            } else {
                PANEL_OPEN_COVERAGE_CASES
            };
            for case in cases {
                clear_lua_error_tracking(&env);
                env.exec(case.open_lua)
                    .unwrap_or_else(|error| panic!("{} opener should run: {error}", case.name));

                let addon_loaded = is_addon_loaded(&env, case.expected_addon);
                let frame_shown = frame_is_shown(&env, case.expected_frame);
                let known_counts = if cfg!(feature = "client-wrath") {
                    BTreeMap::new()
                } else {
                    known_panel_open_runtime_error_counts(case)
                };
                let state = env.state().borrow();
                let grouped_errors = grouped_errors_by_addon(&state);
                let actual_counts = actual_error_counts(&grouped_errors);
                let increases =
                    classify_error_count_increases_from_baseline(&known_counts, &actual_counts);
                let invalid_addons: Vec<_> = grouped_errors
                    .keys()
                    .filter(|addon_name| {
                        addon_name.as_str() != "<unknown>"
                            && addon_name.as_str() != case.expected_addon
                            && !known_blizzard_addons.contains(*addon_name)
                    })
                    .cloned()
                    .collect();
                drop(state);

                if !addon_loaded
                    || !frame_shown
                    || !invalid_addons.is_empty()
                    || !increases.is_empty()
                {
                    case_failures.push(format!(
                        "{}: loaded={}, frame_shown={}, increased=[{}], invalid_addons={:?}, actual counts=[{}]\n{}",
                        case.name,
                        addon_loaded,
                        frame_shown,
                        format_error_count_changes(&increases),
                        invalid_addons,
                        format_error_count_map(&actual_counts),
                        format_per_addon_report(&grouped_errors),
                    ));
                }
            }

            assert!(
                case_failures.is_empty(),
                "panel-open runtime paths exceeded the known per-addon Lua error baseline:\n{}",
                case_failures.join("\n\n"),
            );
        })
    })
}
