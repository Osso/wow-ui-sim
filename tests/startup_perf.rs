#![cfg(feature = "gui")]

#[cfg(feature = "optional-tests")]
use crate::common;
#[cfg(feature = "optional-tests")]
use crate::perf_base_game;

#[cfg(feature = "optional-tests")]
use std::time::Duration;

#[path = "startup_perf/observation.rs"]
mod observation;
#[cfg(feature = "optional-tests")]
use observation::{AddonLoadObservation, encode_addon_observation};

#[cfg(feature = "optional-tests")]
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
#[cfg(feature = "optional-tests")]
use wow_ui_sim::lua_api::WowLuaEnv;
#[cfg(feature = "optional-tests")]
use wow_ui_sim::screen::ScreenKind;
#[cfg(feature = "optional-tests")]
use wow_ui_sim::startup::fire_startup_events_for_screen;

#[cfg(feature = "optional-tests")]
const FULL_GAME_STARTUP_BUDGET: Duration = Duration::from_secs(20);

#[cfg(feature = "optional-tests")]
struct StartupPhaseTimings {
    addon_load_elapsed: Duration,
    post_load_workarounds_elapsed: Duration,
    startup_events_elapsed: Duration,
}

#[cfg(feature = "optional-tests")]
struct LoadedGameUi {
    env: WowLuaEnv,
    phase_timings: StartupPhaseTimings,
    startup_elapsed: Duration,
    addon_observations: Vec<AddonLoadObservation>,
}

#[cfg(feature = "optional-tests")]
fn load_timed_game_ui() -> LoadedGameUi {
    let started = std::time::Instant::now();
    let env = perf_base_game::new_game_env();
    let (addon_load_elapsed, addon_observations) = load_game_ui_addons(&env);
    let post_load_workarounds_elapsed = apply_post_load_workarounds(&env);
    let startup_events_elapsed = fire_startup_events(&env);

    LoadedGameUi {
        env,
        phase_timings: StartupPhaseTimings {
            addon_load_elapsed,
            post_load_workarounds_elapsed,
            startup_events_elapsed,
        },
        startup_elapsed: started.elapsed(),
        addon_observations,
    }
}

#[cfg(feature = "optional-tests")]
fn fire_startup_events(env: &WowLuaEnv) -> Duration {
    let started = std::time::Instant::now();
    fire_startup_events_for_screen(env, ScreenKind::Game);
    started.elapsed()
}

#[cfg(feature = "optional-tests")]
fn apply_post_load_workarounds(env: &WowLuaEnv) -> Duration {
    let started = std::time::Instant::now();
    env.apply_post_load_workarounds();
    started.elapsed()
}

#[cfg(feature = "optional-tests")]
fn load_game_ui_addons(env: &WowLuaEnv) -> (Duration, Vec<AddonLoadObservation>) {
    let started = std::time::Instant::now();
    let addons =
        discover_blizzard_addons_for_screen(&perf_base_game::blizzard_ui_dir(), ScreenKind::Game);
    let mut observations = Vec::with_capacity(addons.len());
    for (name, toc_path) in &addons {
        let addon_started = std::time::Instant::now();
        let result = load_addon(&env.loader_env(), toc_path);
        let wall_elapsed = addon_started.elapsed();
        let result =
            result.unwrap_or_else(|err| panic!("Failed to load Blizzard addon {name}: {err}"));
        observations.push(AddonLoadObservation {
            expected_name: name.clone(),
            wall_elapsed,
            result,
        });
    }
    (started.elapsed(), observations)
}

#[cfg(feature = "optional-tests")]
fn emit_startup_observations(loaded_ui: &LoadedGameUi, startup_ready: bool) {
    for (index, observation) in loaded_ui.addon_observations.iter().enumerate() {
        let row = encode_addon_observation(index + 1, observation);
        eprintln!("[startup-observation] {row}");
    }
    let phases = &loaded_ui.phase_timings;
    let summary = serde_json::json!({
        "kind": "startup_summary",
        "addon_count": loaded_ui.addon_observations.len(),
        "startup_ready": startup_ready,
        "startup_elapsed_ns": loaded_ui.startup_elapsed.as_nanos(),
        "addon_load_elapsed_ns": phases.addon_load_elapsed.as_nanos(),
        "post_load_workarounds_elapsed_ns": phases.post_load_workarounds_elapsed.as_nanos(),
        "startup_events_elapsed_ns": phases.startup_events_elapsed.as_nanos(),
        "budget_ns": FULL_GAME_STARTUP_BUDGET.as_nanos(),
    });
    eprintln!("[startup-observation] {summary}");
}

#[cfg(feature = "optional-tests")]
#[test]
fn full_game_startup_stays_under_budget() {
    perf_test_timeout! {
            let loaded_ui = load_timed_game_ui();
            let env = &loaded_ui.env;
            let phase_timings = &loaded_ui.phase_timings;
            let startup_elapsed = loaded_ui.startup_elapsed;

            let startup_ready: bool = env
                .eval("return UIParent ~= nil and PlayerFrame ~= nil and IsLoggedIn()")
                .unwrap();
            emit_startup_observations(&loaded_ui, startup_ready);
            assert!(
                startup_ready,
                "timed startup should produce a settled logged-in game UI"
            );

            eprintln!(
                "full game startup baseline: {:.2?} (addons {:.2?}, post-load {:.2?}, startup events {:.2?}; budget {:.2?})",
                startup_elapsed,
                phase_timings.addon_load_elapsed,
                phase_timings.post_load_workarounds_elapsed,
                phase_timings.startup_events_elapsed,
                FULL_GAME_STARTUP_BUDGET
            );

            assert!(
                startup_elapsed < FULL_GAME_STARTUP_BUDGET,
                "full game startup took {:.2?}, exceeding budget {:.2?}",
                startup_elapsed,
                FULL_GAME_STARTUP_BUDGET
            );
    }
}
