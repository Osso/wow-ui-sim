//! Iced-based UI for rendering WoW frames.
//!
//! This module is split into several submodules:
//! - `app`: App struct and initialization
//! - `state`: State structs (InspectorState, CanvasMessage)
//! - `styles`: UI styling functions and color palette
//! - `layout`: Frame layout computation and anchor positioning
//! - `view`: App::view() and subscription methods
//! - `update`: App::update() and message handling
//! - `render`: Shader/canvas rendering implementations

// Always-compiled: no iced/GPU dependencies
pub mod frame_collect;
pub mod layout;

// GUI-only submodules
#[cfg(feature = "gui")]
mod app;
#[cfg(feature = "gui")]
mod app_icon;
#[cfg(feature = "gui")]
mod benchmark;
#[cfg(feature = "gui")]
mod button_vis;
#[cfg(feature = "gui")]
mod click_probe;
#[cfg(feature = "gui")]
mod hit_grid;
#[cfg(feature = "gui")]
mod keybinds;
#[cfg(feature = "gui")]
mod masking;
#[cfg(feature = "gui")]
mod message_frame_render;
#[cfg(feature = "gui")]
mod mouse;
#[cfg(feature = "gui")]
mod mouse_drag;
#[cfg(feature = "gui")]
mod mouse_sequence_probe;
#[cfg(feature = "gui")]
pub use mouse_sequence_probe::{MouseSequenceStep, run_headless_mouse_sequence};
#[cfg(feature = "gui")]
mod nine_slice;
#[cfg(feature = "gui")]
mod quad_builders;
#[cfg(feature = "gui")]
mod quad_builders_line;
#[cfg(feature = "gui")]
mod render;
#[cfg(feature = "gui")]
mod screenshot;
#[cfg(feature = "gui")]
mod simple_html_input;
#[cfg(feature = "gui")]
mod simple_html_render;
#[cfg(feature = "gui")]
mod slice_render;
#[cfg(feature = "gui")]
mod state;
#[cfg(feature = "gui")]
mod statusbar;
#[cfg(feature = "gui")]
mod strata_emit;
#[cfg(feature = "gui")]
mod styles;
#[cfg(feature = "gui")]
mod tiling;
#[cfg(feature = "gui")]
pub mod tooltip;
#[cfg(feature = "gui")]
mod tree_dump;
#[cfg(feature = "gui")]
mod update;
#[cfg(feature = "gui")]
mod update_helpers;
#[cfg(feature = "gui")]
mod update_runtime;
#[cfg(feature = "gui")]
mod update_servers;
#[cfg(feature = "gui")]
mod view;

// Always-compiled re-exports
pub use layout::{
    CachedFrameLayout, LayoutCache, anchor_position, compute_frame_rect, compute_frame_rect_cached,
    frame_position_from_anchor,
};

// GUI-only imports and re-exports
#[cfg(feature = "gui")]
use std::sync::OnceLock;
#[cfg(feature = "gui")]
use std::time::Instant;

#[cfg(feature = "gui")]
use iced::window::screenshot::Screenshot;

#[cfg(feature = "gui")]
use crate::lua_api::WowLuaEnv;
#[cfg(feature = "gui")]
use crate::saved_variables::SavedVariablesManager;

#[cfg(feature = "gui")]
pub use app::App;
#[cfg(all(test, feature = "gui", unix))]
pub(crate) use app::AppInit;
#[cfg(feature = "gui")]
pub use benchmark::{
    BenchmarkPhase, DurationStats, LfgPanelBenchmarkReport, PhaseStats, SpellbookBenchmarkReport,
    SpellbookCycleReport, SteadyStateOptions, SteadyStateRound, benchmark_lfg_panel_open_in_gui,
    benchmark_spellbook_cycles_in_gui, benchmark_spellbook_open_in_gui,
    benchmark_steady_state_in_gui, load_benchmark_ui_env,
};
#[cfg(feature = "gui")]
pub use click_probe::{NamedClick, run_headless_named_click_probe};
#[cfg(feature = "gui")]
pub use render::{DirtyStrataRebuildParams, rebuild_dirty_strata_batches_for_registry};
#[cfg(feature = "gui")]
pub use state::{CanvasMessage, InspectorState};
#[cfg(feature = "gui")]
pub use strata_emit::{
    RegistryQuadBatchParams, build_hittable_rects, build_quad_batch_for_registry,
    build_quad_batch_for_registry_with_quest_blobs,
};
#[cfg(feature = "gui")]
pub use styles::palette;

#[cfg(feature = "gui")]
pub use app::DebugOptions;
#[cfg(feature = "gui")]
use app::{INIT_DEBUG, INIT_ENV, INIT_SAVED_VARS};

#[cfg(feature = "gui")]
fn perf_logging_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("WOW_SIM_VERBOSE").is_ok())
}

/// Application messages.
#[cfg(feature = "gui")]
#[derive(Debug, Clone)]
pub enum Message {
    InitialWindowSized,
    CloseRequested,
    FireEvent(String),
    Scroll(f32, f32),
    ReloadUI,
    CommandInputChanged(String),
    ExecuteCommand,
    ProcessTimers(Instant),
    CanvasEvent(CanvasMessage),
    ScreenshotTaken(Screenshot),
    /// Tick for FPS display refresh.
    FpsTick,
    /// Close the inspector panel.
    InspectorClose,
    /// Inspector width input changed.
    InspectorWidthChanged(String),
    /// Inspector height input changed.
    InspectorHeightChanged(String),
    /// Inspector alpha input changed.
    InspectorAlphaChanged(String),
    /// Inspector frame level input changed.
    InspectorLevelChanged(String),
    /// Inspector visible checkbox toggled.
    InspectorVisibleToggled(bool),
    /// Inspector mouse enabled checkbox toggled.
    InspectorMouseEnabledToggled(bool),
    /// Apply inspector changes to the frame.
    InspectorApply,
    /// Toggle frames panel collapsed state.
    ToggleFramesPanel,
    /// XP bar level changed via dropdown.
    XpLevelChanged(String),
    /// Party size changed via dropdown.
    PartySizeChanged(String),
    /// Keyboard input dispatched to Lua (WoW key name, e.g. "ESCAPE", "ENTER", "A")
    /// plus optional raw text for character input into focused EditBox and
    /// the event capture timestamp.
    KeyPress(String, Option<String>, Instant),
    /// Physical key released (unmodified WoW key name).
    KeyRelease(String),
    /// Keyboard modifier state changed.
    ModifiersChanged(iced::keyboard::Modifiers),
    /// Player class changed via dropdown.
    PlayerClassChanged(String),
    /// Player race changed via dropdown.
    PlayerRaceChanged(String),
    /// Rot damage level changed via dropdown.
    RotDamageLevelChanged(String),
    /// Toggle options modal visibility.
    ToggleOptionsModal,
    /// Show or hide the simulator controls over the fixed game canvas.
    ToggleSimulatorControls,
    /// Close options modal (backdrop click or Escape).
    CloseOptionsModal,
    /// Movement state toggle changed (field name, new value).
    MovementToggled(&'static str, bool),
}

/// Run the iced UI with the given Lua environment.
#[cfg(feature = "gui")]
pub fn run_iced_ui(
    env: WowLuaEnv,
    debug: DebugOptions,
    saved_vars: Option<SavedVariablesManager>,
    exec_lua: Option<String>,
    exec_lua_secure: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    run_iced_ui_with_window_size(env, debug, saved_vars, exec_lua, exec_lua_secure, None)
}

/// Run the iced UI with an optional initial client area in physical pixels.
#[cfg(feature = "gui")]
pub fn run_iced_ui_with_window_size(
    env: WowLuaEnv,
    debug: DebugOptions,
    saved_vars: Option<SavedVariablesManager>,
    exec_lua: Option<String>,
    exec_lua_secure: bool,
    window_size: Option<(u32, u32)>,
) -> Result<(), Box<dyn std::error::Error>> {
    if window_size.is_some_and(|(width, height)| width == 0 || height == 0) {
        return Err("window dimensions must be positive".into());
    }
    let completion = app::UiCompletion::default();
    app::INIT_COMPLETION.with(|cell| *cell.borrow_mut() = Some(completion.clone()));
    INIT_ENV.with(|cell| *cell.borrow_mut() = Some(env));
    INIT_DEBUG.with(|cell| *cell.borrow_mut() = Some(debug));
    if let Some(sv) = saved_vars {
        INIT_SAVED_VARS.with(|cell| *cell.borrow_mut() = Some(sv));
    }
    if let Some(code) = exec_lua {
        app::INIT_EXEC_LUA.with(|cell| *cell.borrow_mut() = Some((code, exec_lua_secure)));
    }

    let mut settings = app_icon::settings();
    settings.visible = window_size.is_none();
    iced::application(
        move || App::boot_with_window_size(window_size),
        App::update,
        App::view,
    )
    .title(App::title)
    .window(settings)
    .subscription(App::subscription)
    .run()?;

    ui_completion_result(&completion).map_err(Into::into)
}

#[cfg(feature = "gui")]
fn ui_completion_result(completion: &app::UiCompletion) -> Result<(), String> {
    completion
        .borrow()
        .clone()
        .unwrap_or_else(|| Err("UI exited without completing session finalization".to_string()))
}
