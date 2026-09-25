use std::time::{Duration, Instant};

use iced::mouse;
use iced::widget::shader::Program;
use iced::{Rectangle, Size};

use crate::lua_api::WowLuaEnv;

use super::app::{App, INIT_DEBUG, INIT_ENV, INIT_SAVED_VARS};
use super::{DebugOptions, Message};

const BENCHMARK_SIZE: Size = Size {
    width: 1024.0,
    height: 768.0,
};
const MAX_SETTLE_FRAMES: usize = 256;
const TEXTURE_IDLE_SETTLE_FRAMES: usize = 2;
const FORCED_TICK_INTERVAL: Duration = Duration::from_millis(17);

#[derive(Debug, Clone)]
pub struct SpellbookBenchmarkReport {
    pub startup_idle: BenchmarkPhase,
    pub first_open: BenchmarkPhase,
    pub first_close: BenchmarkPhase,
    pub second_open: BenchmarkPhase,
}

#[derive(Debug, Clone)]
pub struct LfgPanelBenchmarkReport {
    pub startup_idle: BenchmarkPhase,
    pub first_open: BenchmarkPhase,
    pub first_close: BenchmarkPhase,
    pub second_open: BenchmarkPhase,
}

#[derive(Debug, Clone)]
pub struct BenchmarkPhase {
    pub name: &'static str,
    pub keypress_elapsed: Duration,
    pub settle_elapsed: Duration,
    pub tick_elapsed: Duration,
    pub draw_elapsed: Duration,
    pub frames: usize,
    pub textures_loaded: usize,
    pub bc_textures_loaded: usize,
    pub max_pending_dirty_ids: usize,
    pub spellbook_shown: bool,
}

#[derive(Debug, Clone, Copy)]
struct PhaseOptions {
    name: &'static str,
    keypress: Option<&'static str>,
    expect_visible: bool,
    visible_name: &'static str,
    is_visible: fn(&App) -> crate::Result<bool>,
}

#[derive(Debug, Clone, Copy)]
struct FrameTelemetry {
    textures_loaded: usize,
    bc_textures_loaded: usize,
    uploaded_strata: usize,
}

#[derive(Debug, Default)]
struct SettleMetrics {
    tick_elapsed: Duration,
    draw_elapsed: Duration,
    frames: usize,
    textures_loaded: usize,
    bc_textures_loaded: usize,
    max_pending_dirty_ids: usize,
}

pub fn benchmark_spellbook_open_in_gui(env: WowLuaEnv) -> crate::Result<SpellbookBenchmarkReport> {
    let mut app = boot_benchmark_app(env);
    let startup_idle = benchmark_phase(&mut app, spellbook_phase("startup_idle", None, false))?;
    let first_open = benchmark_phase(&mut app, spellbook_phase("first_open", Some("S"), true))?;
    let first_close = benchmark_phase(&mut app, spellbook_phase("first_close", Some("S"), false))?;
    let second_open = benchmark_phase(&mut app, spellbook_phase("second_open", Some("S"), true))?;
    Ok(SpellbookBenchmarkReport {
        startup_idle,
        first_open,
        first_close,
        second_open,
    })
}

pub fn benchmark_lfg_panel_open_in_gui(env: WowLuaEnv) -> crate::Result<LfgPanelBenchmarkReport> {
    let mut app = boot_benchmark_app(env);
    let startup_idle = benchmark_phase(&mut app, lfg_panel_phase("startup_idle", None, false))?;
    let first_open = benchmark_phase(&mut app, lfg_panel_phase("first_open", Some("L"), true))?;
    let first_close = benchmark_phase(&mut app, lfg_panel_phase("first_close", Some("L"), false))?;
    let second_open = benchmark_phase(&mut app, lfg_panel_phase("second_open", Some("L"), true))?;
    Ok(LfgPanelBenchmarkReport {
        startup_idle,
        first_open,
        first_close,
        second_open,
    })
}

/// Per-round tick and draw CPU cost over a fixed number of settled frames.
#[derive(Debug, Clone)]
pub struct SteadyStateRound {
    pub tick: DurationStats,
    pub draw: DurationStats,
    /// Frames whose draw handed at least one stratum batch to the GPU.
    pub strata_upload_frames: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct DurationStats {
    pub p50: Duration,
    pub p90: Duration,
    pub mean: Duration,
}

#[derive(Debug, Clone, Copy)]
pub struct SteadyStateOptions {
    pub warmup_frames: usize,
    pub measured_frames: usize,
    pub rounds: usize,
}

/// Measure settled-UI frames through the real GUI tick and draw path.
///
/// Every frame forces a full OnUpdate interval, so the workload per frame is
/// independent of window pacing and host load. Draw covers the CPU side of the
/// shader program (quad rebuild and texture loading), not GPU submission.
pub fn benchmark_steady_state_in_gui(
    env: WowLuaEnv,
    options: SteadyStateOptions,
) -> crate::Result<Vec<SteadyStateRound>> {
    let mut app = boot_benchmark_app(env);
    benchmark_phase(&mut app, spellbook_phase("startup_idle", None, false))?;
    for _ in 0..options.warmup_frames {
        run_benchmark_frame(&mut app);
    }
    let rounds = (0..options.rounds)
        .map(|_| measure_steady_state_round(&mut app, options.measured_frames))
        .collect();
    Ok(rounds)
}

fn measure_steady_state_round(app: &mut App, frames: usize) -> SteadyStateRound {
    let mut ticks = Vec::with_capacity(frames);
    let mut draws = Vec::with_capacity(frames);
    let mut strata_upload_frames = 0;
    for _ in 0..frames {
        let (tick, draw, telemetry) = run_benchmark_frame(app);
        strata_upload_frames += usize::from(telemetry.uploaded_strata != 0);
        ticks.push(tick);
        draws.push(draw);
    }
    SteadyStateRound {
        tick: summarize_durations(ticks),
        draw: summarize_durations(draws),
        strata_upload_frames,
    }
}

fn summarize_durations(mut samples: Vec<Duration>) -> DurationStats {
    samples.sort_unstable();
    let percentile = |fraction: f64| {
        let index = ((samples.len() as f64 * fraction) as usize).min(samples.len() - 1);
        samples[index]
    };
    let total: Duration = samples.iter().sum();
    DurationStats {
        p50: percentile(0.5),
        p90: percentile(0.9),
        mean: total / samples.len() as u32,
    }
}

fn spellbook_phase(
    name: &'static str,
    keypress: Option<&'static str>,
    expect_visible: bool,
) -> PhaseOptions {
    panel_phase(
        name,
        keypress,
        expect_visible,
        "spellbook_shown",
        is_spellbook_shown,
    )
}

fn lfg_panel_phase(
    name: &'static str,
    keypress: Option<&'static str>,
    expect_visible: bool,
) -> PhaseOptions {
    panel_phase(
        name,
        keypress,
        expect_visible,
        "lfg_panel_shown",
        is_lfg_panel_shown,
    )
}

fn panel_phase(
    name: &'static str,
    keypress: Option<&'static str>,
    expect_visible: bool,
    visible_name: &'static str,
    is_visible: fn(&App) -> crate::Result<bool>,
) -> PhaseOptions {
    PhaseOptions {
        name,
        keypress,
        expect_visible,
        visible_name,
        is_visible,
    }
}

/// Load every discovered Blizzard UI addon into a fresh 1024x768 environment.
pub fn load_benchmark_ui_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("failed to create Lua environment");
    env.set_screen_size(BENCHMARK_SIZE.width, BENCHMARK_SIZE.height);
    let ui =
        crate::paths::default_blizzard_ui_addons_path().expect("missing Blizzard UI addon tree");
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    for (name, toc_path) in &crate::loader::discover_blizzard_addons(&ui) {
        if let Err(error) = crate::loader::load_addon(&env.loader_env(), toc_path) {
            eprintln!("[load {name}] FAILED: {error}");
        }
    }
    env.apply_post_load_workarounds();
    env
}

fn boot_benchmark_app(env: WowLuaEnv) -> App {
    INIT_ENV.with(|cell| *cell.borrow_mut() = Some(env));
    INIT_DEBUG.with(|cell| *cell.borrow_mut() = Some(DebugOptions::default()));
    INIT_SAVED_VARS.with(|cell| *cell.borrow_mut() = None);

    let (app, _task) = App::boot();
    app
}

fn benchmark_phase(app: &mut App, options: PhaseOptions) -> crate::Result<BenchmarkPhase> {
    let keypress_elapsed = dispatch_keypress(app, options.keypress);
    let settle_started = Instant::now();
    let settle_metrics = collect_settle_metrics(app, options)?;
    let visible = (options.is_visible)(app)?;
    if visible != options.expect_visible {
        return Err(crate::Error::Other(format!(
            "{} ended with {}={} expected={}",
            options.name, options.visible_name, visible, options.expect_visible
        )));
    }

    Ok(BenchmarkPhase {
        name: options.name,
        keypress_elapsed,
        settle_elapsed: settle_started.elapsed(),
        tick_elapsed: settle_metrics.tick_elapsed,
        draw_elapsed: settle_metrics.draw_elapsed,
        frames: settle_metrics.frames,
        textures_loaded: settle_metrics.textures_loaded,
        bc_textures_loaded: settle_metrics.bc_textures_loaded,
        max_pending_dirty_ids: settle_metrics.max_pending_dirty_ids,
        spellbook_shown: visible,
    })
}

fn collect_settle_metrics(app: &mut App, options: PhaseOptions) -> crate::Result<SettleMetrics> {
    let mut metrics = SettleMetrics::default();
    let mut texture_idle_frames = 0usize;
    let mut last_texture_pending_count = texture_pending_path_count(app);

    while !is_quiescent(app, texture_idle_frames) {
        if metrics.frames >= MAX_SETTLE_FRAMES {
            return Err(crate::Error::Other(format!(
                "{} did not settle after {} frames ({})",
                options.name,
                MAX_SETTLE_FRAMES,
                quiescence_summary(app)
            )));
        }
        let (tick_dur, draw_dur, frame_telemetry) = run_benchmark_frame(app);
        let current_texture_pending_count = texture_pending_path_count(app);
        texture_idle_frames = next_texture_idle_frame_count(
            app,
            texture_idle_frames,
            last_texture_pending_count,
            current_texture_pending_count,
        );
        last_texture_pending_count = current_texture_pending_count;
        metrics.tick_elapsed += tick_dur;
        metrics.draw_elapsed += draw_dur;
        metrics.textures_loaded += frame_telemetry.textures_loaded;
        metrics.bc_textures_loaded += frame_telemetry.bc_textures_loaded;
        metrics.max_pending_dirty_ids = metrics.max_pending_dirty_ids.max(pending_dirty_count(app));
        metrics.frames += 1;
    }

    Ok(metrics)
}

fn next_texture_idle_frame_count(
    app: &App,
    texture_idle_frames: usize,
    last_texture_pending_count: usize,
    current_texture_pending_count: usize,
) -> usize {
    if has_dirty_render_work(app) || current_texture_pending_count != last_texture_pending_count {
        return 0;
    }
    texture_idle_frames.saturating_add(1)
}

fn dispatch_keypress(app: &mut App, keypress: Option<&'static str>) -> Duration {
    let Some(keypress) = keypress else {
        return Duration::ZERO;
    };
    let started = Instant::now();
    let _ = app.update(Message::KeyPress(
        keypress.to_string(),
        None,
        Instant::now(),
    ));
    started.elapsed()
}

fn run_benchmark_frame(app: &mut App) -> (Duration, Duration, FrameTelemetry) {
    app.last_on_update_time = Instant::now() - FORCED_TICK_INTERVAL;

    let tick_started = Instant::now();
    let _ = app.update(Message::ProcessTimers(Instant::now()));
    let tick_elapsed = tick_started.elapsed();

    let draw_started = Instant::now();
    let primitive = <&App as Program<Message>>::draw(
        &&*app,
        &(),
        mouse::Cursor::Unavailable,
        Rectangle::with_size(BENCHMARK_SIZE),
    );
    let draw_elapsed = draw_started.elapsed();

    (
        tick_elapsed,
        draw_elapsed,
        FrameTelemetry {
            textures_loaded: primitive.textures.len(),
            bc_textures_loaded: primitive.bc_textures.len(),
            uploaded_strata: primitive.strata_batches.iter().flatten().count(),
        },
    )
}

fn is_quiescent(app: &App, texture_idle_frames: usize) -> bool {
    !has_dirty_render_work(app) && textures_are_quiescent(app, texture_idle_frames)
}

fn has_dirty_render_work(app: &App) -> bool {
    app.strata_dirty.get() != 0
        || app
            .pending_dirty_ids
            .borrow()
            .as_ref()
            .is_some_and(|ids| !ids.is_empty())
}

fn textures_are_quiescent(app: &App, texture_idle_frames: usize) -> bool {
    !app.textures_pending.get() || texture_idle_frames >= TEXTURE_IDLE_SETTLE_FRAMES
}

fn texture_pending_path_count(app: &App) -> usize {
    app.pending_texture_path_set.borrow().len()
}

fn pending_dirty_count(app: &App) -> usize {
    app.pending_dirty_ids
        .borrow()
        .as_ref()
        .map_or(0, rustc_hash::FxHashSet::len)
}

fn quiescence_summary(app: &App) -> String {
    format!(
        "strata_dirty=0x{:x} textures_pending={} pending_dirty_ids={}",
        app.strata_dirty.get(),
        app.textures_pending.get(),
        pending_dirty_count(app)
    )
}

fn is_spellbook_shown(app: &App) -> crate::Result<bool> {
    let env = app.env.borrow();
    env.eval::<bool>("return PlayerSpellsFrame ~= nil and PlayerSpellsFrame:IsShown()")
}

fn is_lfg_panel_shown(app: &App) -> crate::Result<bool> {
    let env = app.env.borrow();
    env.eval::<bool>(
        "return PVEFrame ~= nil and PVEFrame:IsShown() and GroupFinderFrame ~= nil and GroupFinderFrame:IsShown()",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spellbook_phase_sets_expected_metadata() {
        let phase = spellbook_phase("first_open", Some("S"), true);

        assert_eq!(phase.name, "first_open");
        assert_eq!(phase.keypress, Some("S"));
        assert!(phase.expect_visible);
        assert_eq!(phase.visible_name, "spellbook_shown");
    }

    #[test]
    fn summarize_durations_reports_percentiles_and_mean() {
        let samples = (1..=10).rev().map(Duration::from_millis).collect();

        let stats = summarize_durations(samples);

        assert_eq!(stats.p50, Duration::from_millis(6));
        assert_eq!(stats.p90, Duration::from_millis(10));
        assert_eq!(stats.mean, Duration::from_micros(5500));
    }

    #[test]
    fn settle_metrics_defaults_to_zero() {
        let metrics = SettleMetrics::default();

        assert_eq!(metrics.tick_elapsed, Duration::ZERO);
        assert_eq!(metrics.draw_elapsed, Duration::ZERO);
        assert_eq!(metrics.frames, 0);
        assert_eq!(metrics.textures_loaded, 0);
        assert_eq!(metrics.bc_textures_loaded, 0);
        assert_eq!(metrics.max_pending_dirty_ids, 0);
    }
}
