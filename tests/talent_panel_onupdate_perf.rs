#[cfg(feature = "optional-tests")]
use crate::common;

#[cfg(feature = "optional-tests")]
use std::path::PathBuf;
#[cfg(feature = "optional-tests")]
use std::time::{Duration, Instant};

#[cfg(feature = "optional-tests")]
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
#[cfg(feature = "optional-tests")]
use wow_ui_sim::screen::ScreenKind;
#[cfg(feature = "optional-tests")]
use wow_ui_sim::startup::settle_headless_startup;

#[cfg(feature = "optional-tests")]
const TALENT_PANEL_VISIBLE_ONUPDATE_ADDED_CEILING: usize = 192;
#[cfg(feature = "optional-tests")]
const TALENT_PANEL_ONUPDATE_AVG_BUDGET: Duration = Duration::from_millis(35);
#[cfg(feature = "optional-tests")]
const TALENT_PANEL_ONUPDATE_MAX_BUDGET: Duration = Duration::from_millis(75);
#[cfg(feature = "optional-tests")]
const TALENT_PANEL_ONUPDATE_DELTA_BUDGET: Duration = Duration::from_millis(20);
#[cfg(feature = "optional-tests")]
const BASELINE_TICKS: usize = 20;
#[cfg(feature = "optional-tests")]
const TALENT_STEADY_TICKS: usize = 24;
#[cfg(feature = "optional-tests")]
const TALENT_SETTLE_TICKS: usize = 10;

#[cfg(feature = "optional-tests")]
fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

#[cfg(feature = "optional-tests")]
fn load_settled_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];

    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("Failed to load Blizzard addon {name}: {err}"));
    }

    env.apply_post_load_workarounds();
    settle_headless_startup(&env);
    env
}

#[cfg(feature = "optional-tests")]
fn open_talent_panel(env: &WowLuaEnv) {
    env.exec("PlayerSpellsUtil.ToggleClassTalentFrame()")
        .expect("Talent panel toggle should succeed");
    let shown: bool = env
        .eval(
            "return PlayerSpellsFrame ~= nil and PlayerSpellsFrame:IsShown() and PlayerSpellsFrame.TalentsFrame ~= nil and PlayerSpellsFrame.TalentsFrame:IsShown()",
        )
        .expect("Talent panel visibility query should succeed");
    assert!(shown, "Talent panel should be visible after opening");
}

fn capture_visible_onupdate(env: &WowLuaEnv) -> (usize, Vec<String>) {
    let state = env.state().borrow();
    let mut visible_ids = state
        .on_update_frames
        .iter()
        .copied()
        .filter(|id| state.widgets.is_ancestor_visible(*id))
        .collect::<Vec<_>>();
    visible_ids.sort_unstable();
    let names = visible_ids
        .iter()
        .map(|id| {
            state
                .widgets
                .get(*id)
                .and_then(|frame| frame.name.clone().or_else(|| frame.parent_key.clone()))
                .unwrap_or_else(|| format!("id:{id}"))
        })
        .collect::<Vec<_>>();
    (visible_ids.len(), names)
}

#[cfg(feature = "optional-tests")]
fn sample_on_update_ticks(env: &WowLuaEnv, samples: usize, phase: &str) -> Vec<Duration> {
    (0..samples)
        .map(|tick| run_logged_headless_tick(env, phase, tick))
        .collect()
}

#[cfg(feature = "optional-tests")]
fn run_logged_headless_tick(env: &WowLuaEnv, phase: &str, tick: usize) -> Duration {
    eprintln!("[talent-tick] phase={phase} tick={tick} begin");
    let duration = run_headless_tick(env);
    eprintln!("[talent-tick] phase={phase} tick={tick} end");
    duration
}

#[cfg(feature = "optional-tests")]
fn run_headless_tick(env: &WowLuaEnv) -> Duration {
    let started = Instant::now();
    env.process_timers().unwrap();
    env.state().borrow_mut().ensure_layout_rects();
    env.fire_on_update(0.016).unwrap();
    started.elapsed()
}

#[cfg(feature = "optional-tests")]
fn mean_duration(samples: &[Duration]) -> Duration {
    let total = samples
        .iter()
        .copied()
        .fold(Duration::ZERO, |sum, elapsed| sum + elapsed);
    total / u32::try_from(samples.len()).expect("sample set should fit u32")
}

#[cfg(feature = "optional-tests")]
fn max_duration(samples: &[Duration]) -> Duration {
    samples
        .iter()
        .copied()
        .max()
        .expect("sample set should not be empty")
}

#[test]
fn visible_onupdate_collector_tracks_child_parent_visibility() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.fire_on_update(0.016)
        .expect("Initial OnUpdate tick should settle registration");
    let baseline_count = capture_visible_onupdate(&env).0;

    env.exec(
        r#"
        VisibleOnUpdateRegressionTicks = 0
        local parent = CreateFrame("Frame", "VisibleOnUpdateRegressionParent", UIParent)
        local child = CreateFrame("Frame", "VisibleOnUpdateRegressionChild", parent)
        parent:Show()
        child:Show()
        child:SetScript("OnUpdate", function()
            VisibleOnUpdateRegressionTicks = VisibleOnUpdateRegressionTicks + 1
        end)
        assert(child:IsVisible(), "Shown child under shown parent should be visible")
        assert(type(child:GetScript("OnUpdate")) == "function", "Child should have an OnUpdate callback")
        "#,
    )
    .expect("Visible child OnUpdate fixture should register successfully");
    let child_name: String = env
        .eval("return VisibleOnUpdateRegressionChild:GetName()")
        .expect("Child should expose its actual global name");
    assert_eq!(child_name, "VisibleOnUpdateRegressionChild");

    env.fire_on_update(0.016)
        .expect("Visible child OnUpdate tick should succeed");
    let ticks: i32 = env
        .eval("return VisibleOnUpdateRegressionTicks")
        .expect("Visible child callback count should be readable");
    assert_eq!(ticks, 1, "Visible child should receive its first OnUpdate");
    let (visible_count, visible_names) = capture_visible_onupdate(&env);
    assert_eq!(
        visible_count,
        baseline_count + 1,
        "Collector should count the visible child that received OnUpdate: {visible_names:?}"
    );
    assert!(
        visible_names.contains(&child_name),
        "Collector should include the visible child's actual global name: {visible_names:?}"
    );

    env.exec(
        r#"
        VisibleOnUpdateRegressionParent:Hide()
        assert(VisibleOnUpdateRegressionChild:IsShown(), "Hiding parent should leave child locally shown")
        assert(not VisibleOnUpdateRegressionChild:IsVisible(), "Child under hidden parent should not be visible")
        "#,
    )
    .expect("Parent hide should make the locally shown child invisible");
    env.fire_on_update(0.016)
        .expect("Hidden parent OnUpdate tick should succeed");
    let ticks: i32 = env
        .eval("return VisibleOnUpdateRegressionTicks")
        .expect("Hidden child callback count should be readable");
    assert_eq!(
        ticks, 1,
        "Child under hidden parent should not receive OnUpdate"
    );
    let (hidden_count, hidden_names) = capture_visible_onupdate(&env);
    assert_eq!(
        hidden_count, baseline_count,
        "Collector should return to baseline when parent hides the child: {hidden_names:?}"
    );
    assert!(
        !hidden_names.contains(&child_name),
        "Collector should exclude the child under a hidden parent: {hidden_names:?}"
    );

    env.exec(
        r#"
        VisibleOnUpdateRegressionParent:Show()
        assert(VisibleOnUpdateRegressionChild:IsVisible(), "Re-showing parent should restore child visibility")
        "#,
    )
    .expect("Parent show should restore child visibility");
    env.fire_on_update(0.016)
        .expect("Restored visible child OnUpdate tick should succeed");
    let ticks: i32 = env
        .eval("return VisibleOnUpdateRegressionTicks")
        .expect("Restored child callback count should be readable");
    assert_eq!(
        ticks, 2,
        "Restored visible child should receive its second OnUpdate"
    );
    let (restored_count, restored_names) = capture_visible_onupdate(&env);
    assert_eq!(
        restored_count,
        baseline_count + 1,
        "Collector should count the restored visible child: {restored_names:?}"
    );
    assert!(
        restored_names.contains(&child_name),
        "Collector should include the restored child's actual global name: {restored_names:?}"
    );
}

#[cfg(feature = "optional-tests")]
#[test]
fn talent_panel_open_visible_onupdate_handlers_stay_within_ceiling() {
    perf_test_timeout! {
            let env = load_settled_game_ui();
            run_headless_tick(&env);
            let before_count = capture_visible_onupdate(&env).0;

            open_talent_panel(&env);
            for _ in 0..TALENT_SETTLE_TICKS {
                run_headless_tick(&env);
            }

            let (after_count, visible_names) = capture_visible_onupdate(&env);
            let added_count = after_count.saturating_sub(before_count);

            assert!(
                added_count <= TALENT_PANEL_VISIBLE_ONUPDATE_ADDED_CEILING,
                "talent-panel open should add at most {} visible OnUpdate handlers over settled baseline (before {}, after {}, added {}): {:?}",
                TALENT_PANEL_VISIBLE_ONUPDATE_ADDED_CEILING,
                before_count,
                after_count,
                added_count,
                visible_names
            );
    }
}

#[cfg(feature = "optional-tests")]
#[test]
fn talent_panel_open_onupdate_tick_latency_stays_under_budget() {
    perf_test_timeout! {
            let env = load_settled_game_ui();
            let baseline_samples = sample_on_update_ticks(&env, BASELINE_TICKS, "baseline");
            let baseline_avg = mean_duration(&baseline_samples);

            open_talent_panel(&env);
            for tick in 0..TALENT_SETTLE_TICKS {
                run_logged_headless_tick(&env, "settle", tick);
            }

            let talent_samples = sample_on_update_ticks(&env, TALENT_STEADY_TICKS, "steady");
            let talent_avg = mean_duration(&talent_samples);
            let talent_max = max_duration(&talent_samples);

            eprintln!(
                "talent panel OnUpdate baseline avg={:.2?}, talent avg={:.2?}, talent max={:.2?} (budgets avg<{:.2?}, max<{:.2?}, delta<{:.2?})",
                baseline_avg,
                talent_avg,
                talent_max,
                TALENT_PANEL_ONUPDATE_AVG_BUDGET,
                TALENT_PANEL_ONUPDATE_MAX_BUDGET,
                TALENT_PANEL_ONUPDATE_DELTA_BUDGET
            );

            assert!(
                talent_avg <= TALENT_PANEL_ONUPDATE_AVG_BUDGET,
                "talent-panel open average OnUpdate tick took {:.2?}, exceeding budget {:.2?}",
                talent_avg,
                TALENT_PANEL_ONUPDATE_AVG_BUDGET
            );
            assert!(
                talent_max <= TALENT_PANEL_ONUPDATE_MAX_BUDGET,
                "talent-panel open max OnUpdate tick took {:.2?}, exceeding budget {:.2?}",
                talent_max,
                TALENT_PANEL_ONUPDATE_MAX_BUDGET
            );
            assert!(
                talent_avg <= baseline_avg + TALENT_PANEL_ONUPDATE_DELTA_BUDGET,
                "talent-panel open average OnUpdate tick regressed too far over baseline (baseline {:.2?}, talent {:.2?}, delta budget {:.2?})",
                baseline_avg,
                talent_avg,
                TALENT_PANEL_ONUPDATE_DELTA_BUDGET
            );
    }
}
