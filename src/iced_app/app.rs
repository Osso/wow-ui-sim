//! App struct definition and core initialization.

use iced::{Point, Size, Task};
use rilua::LuaApiMut;
use rustc_hash::{FxHashMap, FxHashSet};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

#[cfg(not(target_os = "linux"))]
use crate::inspector_server_stub as debug_server;
use crate::lua_api::WowLuaEnv;
use crate::lua_server;
use crate::render::{GlyphAtlas, WowFontSystem};
use crate::saved_variables::SavedVariablesManager;
use crate::texture::TextureManager;
use crate::widget::{Frame, WidgetType};
use debug_server::ScreenshotData;
#[cfg(target_os = "linux")]
use iced_layout_inspector::server as debug_server;

use super::Message;
use super::state::InspectorState;

type RenderResources = (
    Rc<RefCell<TextureManager>>,
    Rc<RefCell<WowFontSystem>>,
    Rc<RefCell<GlyphAtlas>>,
);

struct AppInitialSelections {
    xp_level: String,
    party_size: String,
    class: String,
    race: String,
    rot_level: String,
    movement: crate::config::MovementConfig,
}

const DEFAULT_FAST_TICK_MS: u64 = 16;
const GUI_STARTUP_ON_UPDATE_TICKS: usize = 16;

/// Debug visualization options.
#[derive(Default, Clone)]
pub struct DebugOptions {
    pub borders: bool,
    pub anchors: bool,
}

// Thread-local storage for init params
thread_local! {
    pub static INIT_ENV: RefCell<Option<WowLuaEnv>> = const { RefCell::new(None) };
    pub static INIT_DEBUG: RefCell<Option<DebugOptions>> = const { RefCell::new(None) };
    pub static INIT_SAVED_VARS: RefCell<Option<SavedVariablesManager>> = const { RefCell::new(None) };
    pub static INIT_EXEC_LUA: RefCell<Option<(String, bool)>> = const { RefCell::new(None) };
}

/// Fire the standard WoW startup events.
pub fn fire_startup_events(env: &Rc<RefCell<WowLuaEnv>>) {
    let env = env.borrow();
    let screen = env.state().borrow().screen_kind;
    crate::logging::println_elapsed("[Startup] beginning startup event dispatch");
    crate::startup::fire_startup_events_for_screen(&env, screen);
    crate::logging::println_elapsed("[Startup] startup event dispatch complete");
}

/// Application state.
pub struct App {
    pub(crate) env: Rc<RefCell<WowLuaEnv>>,
    pub(crate) log_messages: Vec<String>,
    pub(crate) command_input: String,
    pub(crate) texture_manager: Rc<RefCell<TextureManager>>,
    pub(crate) font_system: Rc<RefCell<WowFontSystem>>,
    pub(crate) glyph_atlas: Rc<RefCell<GlyphAtlas>>,
    pub(crate) hovered_frame: Option<u64>,
    pub(crate) pressed_frame: Option<u64>,
    pub(crate) mouse_down_frame: Option<u64>,
    pub(crate) right_mouse_down_frame: Option<u64>,
    /// Position where mouse button was pressed (for drag threshold detection).
    pub(crate) mouse_down_pos: Option<Point>,
    /// True once drag threshold is exceeded for the current press.
    pub(crate) dragging: bool,
    pub(crate) scroll_offset: f32,
    /// Current canvas size (updated each frame for layout calculations).
    pub(crate) screen_size: std::cell::Cell<Size>,
    /// True after WoW startup events have run against real canvas bounds.
    pub(crate) gui_startup_complete: std::cell::Cell<bool>,
    pub(crate) debug_rx: Option<mpsc::Receiver<debug_server::Command>>,
    pub(crate) pending_screenshot: Option<oneshot::Sender<Result<ScreenshotData, String>>>,
    pub(crate) lua_rx: Option<std::sync::mpsc::Receiver<lua_server::LuaCommand>>,
    /// Draw red debug borders around all frames when true.
    pub(crate) debug_borders: bool,
    /// Draw green anchor points on all frames when true.
    pub(crate) debug_anchors: bool,
    /// Cached merged quad batch (all strata combined), used by draw().
    pub(crate) cached_quads: RefCell<Option<(Size, std::sync::Arc<crate::render::QuadBatch>)>>,
    /// Per-strata cached quad batches. Index = FrameStrata::as_index().
    pub(crate) cached_strata_quads: RefCell<
        [Option<std::sync::Arc<crate::render::QuadBatch>>; crate::widget::FrameStrata::COUNT],
    >,
    /// Per-strata per-frame quad snapshots for incremental strata rebuild.
    pub(crate) cached_frame_snapshots: RefCell<
        [Option<std::collections::HashMap<u64, crate::render::FrameQuadSnapshot>>;
            crate::widget::FrameStrata::COUNT],
    >,
    /// Dirty frame IDs from the last timer tick. `None` means full rebuild needed.
    pub(crate) pending_dirty_ids: RefCell<Option<rustc_hash::FxHashSet<u64>>>,
    /// Cooldown widgets that were visually active on the previous timer tick.
    pub(crate) previous_active_cooldown_widget_ids: RefCell<FxHashSet<u64>>,
    /// Spatial grid for fast hit testing (rebuilt when layout changes).
    pub(crate) cached_hittable: RefCell<Option<super::hit_grid::HitGrid>>,
    /// Per-strata dirty bitmask — bit `i` means strata index `i` needs re-emit.
    pub(crate) strata_dirty: std::cell::Cell<u16>,
    /// True when texture loading was capped and more textures are pending.
    pub(crate) textures_pending: std::cell::Cell<bool>,
    /// Persistent queue of normalized pending texture paths (draw/tick shared).
    pub(crate) pending_texture_path_queue: RefCell<VecDeque<String>>,
    /// Membership set for `pending_texture_path_queue`.
    pub(crate) pending_texture_path_set: RefCell<FxHashSet<String>>,
    /// Per-strata pending requests keyed by normalized texture path.
    pub(crate) strata_pending_texture_requests: RefCell<
        [FxHashMap<String, Vec<crate::render::TextureRequest>>; crate::widget::FrameStrata::COUNT],
    >,
    /// Aggregated pending requests keyed by normalized texture path.
    pub(crate) pending_texture_requests_by_path:
        RefCell<FxHashMap<String, Vec<crate::render::TextureRequest>>>,
    /// Paths known-ready in GPU atlas from prior successful request handles.
    pub(crate) ready_texture_path_cache: RefCell<FxHashSet<String>>,
    /// Strata rebuilt or reset since an idle-tick texture warmup last found all
    /// their requests CPU-cached. CPU texture caches never evict, so only these
    /// strata can hold paths that still need a warmup check.
    pub(crate) texture_warmup_unsettled_strata: std::cell::Cell<u16>,
    /// Most recent main-thread phase that can block event handling.
    pub(crate) main_thread_phase: RefCell<(&'static str, std::time::Instant)>,
    /// Count of stale timer ticks dropped since the last key log.
    pub(crate) dropped_stale_timer_ticks: std::cell::Cell<u32>,
    /// When the last non-dropped timer tick was processed. Guarantees tick
    /// forward progress: under sustained main-thread saturation every tick
    /// arrives stale, and dropping all of them starves OnUpdate / C_Timer
    /// processing entirely.
    pub(crate) last_timer_tick_processed: std::cell::Cell<std::time::Instant>,
    /// Oldest age among dropped stale timer ticks since the last key log.
    pub(crate) oldest_dropped_timer_tick_age: std::cell::Cell<std::time::Duration>,
    /// FPS counter: frame count since last update (interior mutability for draw()).
    pub(crate) frame_count: std::cell::Cell<u32>,
    /// Total draw time accumulated since the last metrics refresh.
    pub(crate) draw_time_accum_ms: std::cell::Cell<f32>,
    /// Total timer tick time accumulated since the last metrics refresh.
    pub(crate) tick_time_accum_ms: std::cell::Cell<f32>,
    /// Count of timer ticks accumulated since the last metrics refresh.
    pub(crate) tick_count: std::cell::Cell<u32>,
    /// FPS counter: last FPS calculation time.
    pub(crate) fps_last_time: std::time::Instant,
    /// Current FPS value.
    pub(crate) fps: f32,
    /// Main-thread CPU time at `fps_last_time`, when the platform reports it.
    pub(crate) fps_last_main_thread_cpu: Option<std::time::Duration>,
    /// Title-bar frame metrics from the last one-second sample window.
    pub(crate) display_metrics: super::update::DisplayMetrics,
    /// Current mouse position in canvas coordinates.
    pub(crate) mouse_position: Option<Point>,
    /// Currently inspected frame ID.
    pub(crate) inspected_frame: Option<u64>,
    /// Whether the inspector panel is visible.
    pub(crate) inspector_visible: bool,
    /// Position of the inspector panel.
    pub(crate) inspector_position: Point,
    /// Inspector panel state (editable fields).
    pub(crate) inspector_state: InspectorState,
    /// Whether the frames panel is collapsed.
    pub(crate) frames_panel_collapsed: bool,
    /// Last time OnUpdate handlers were fired (for elapsed calculation).
    pub(crate) last_on_update_time: std::time::Instant,
    /// SavedVariables manager for persisting addon data on exit.
    pub(crate) saved_vars: Option<SavedVariablesManager>,
    /// Lua code to execute after first frame (from --exec-lua). Bool is
    /// `true` when `--exec-lua-secure` was passed (run under secureenv).
    pub(crate) pending_exec_lua: Option<(String, bool)>,
    /// Currently selected XP bar level label.
    pub(crate) selected_xp_level: String,
    /// Currently selected party size label.
    pub(crate) selected_party_size: String,
    /// Last time party health was ticked (random walk every 2 seconds).
    pub(crate) last_party_health_tick: std::time::Instant,
    /// Currently selected player class name (for picker display).
    pub(crate) selected_class: String,
    /// Currently selected player race name (for picker display).
    pub(crate) selected_race: String,
    /// Currently selected rot damage level label.
    pub(crate) selected_rot_level: String,
    /// Whether the options modal is visible.
    pub(crate) options_modal_visible: bool,
    /// Movement state toggles (mirrors SimState.movement for UI display).
    pub(crate) movement: crate::config::MovementConfig,
}

pub(crate) struct AppInit {
    pub(crate) env: Rc<RefCell<WowLuaEnv>>,
    pub(crate) log_messages: Vec<String>,
    pub(crate) texture_manager: Rc<RefCell<TextureManager>>,
    pub(crate) font_system: Rc<RefCell<WowFontSystem>>,
    pub(crate) glyph_atlas: Rc<RefCell<GlyphAtlas>>,
    pub(crate) cmd_rx: mpsc::Receiver<debug_server::Command>,
    pub(crate) lua_rx: std::sync::mpsc::Receiver<lua_server::LuaCommand>,
    pub(crate) debug_borders: bool,
    pub(crate) debug_anchors: bool,
    pub(crate) saved_vars: Option<SavedVariablesManager>,
    pub(crate) config: crate::config::SimConfig,
}

impl AppInitialSelections {
    fn from_config(config: &crate::config::SimConfig) -> Self {
        Self {
            xp_level: config.xp_level.clone(),
            party_size: config.party_size.to_string(),
            class: config.player_class.clone(),
            race: config.player_race.clone(),
            rot_level: config.rot_damage_level.clone(),
            movement: config.movement.clone(),
        }
    }
}

macro_rules! app_from_initial_state {
    ($init:ident, $selections:ident, $now:ident, $initial_screen_size:ident) => {
        App {
            env: $init.env,
            log_messages: $init.log_messages,
            command_input: String::new(),
            texture_manager: $init.texture_manager,
            font_system: $init.font_system,
            glyph_atlas: $init.glyph_atlas,
            hovered_frame: None,
            pressed_frame: None,
            mouse_down_frame: None,
            right_mouse_down_frame: None,
            mouse_down_pos: None,
            dragging: false,
            scroll_offset: 0.0,
            screen_size: std::cell::Cell::new($initial_screen_size),
            gui_startup_complete: std::cell::Cell::new(false),
            debug_rx: Some($init.cmd_rx),
            pending_screenshot: None,
            lua_rx: Some($init.lua_rx),
            debug_borders: $init.debug_borders,
            debug_anchors: $init.debug_anchors,
            cached_quads: RefCell::new(None),
            cached_strata_quads: RefCell::new(std::array::from_fn(|_| None)),
            cached_frame_snapshots: RefCell::new(std::array::from_fn(|_| None)),
            pending_dirty_ids: RefCell::new(None),
            previous_active_cooldown_widget_ids: RefCell::new(FxHashSet::default()),
            cached_hittable: RefCell::new(None),
            strata_dirty: std::cell::Cell::new((1u16 << crate::widget::FrameStrata::COUNT) - 1),
            textures_pending: std::cell::Cell::new(false),
            pending_texture_path_queue: RefCell::new(VecDeque::new()),
            pending_texture_path_set: RefCell::new(FxHashSet::default()),
            strata_pending_texture_requests: RefCell::new(std::array::from_fn(|_| {
                FxHashMap::default()
            })),
            pending_texture_requests_by_path: RefCell::new(FxHashMap::default()),
            ready_texture_path_cache: RefCell::new(FxHashSet::default()),
            texture_warmup_unsettled_strata: std::cell::Cell::new(ALL_STRATA_MASK),
            main_thread_phase: RefCell::new(("boot", $now)),
            dropped_stale_timer_ticks: std::cell::Cell::new(0),
            last_timer_tick_processed: std::cell::Cell::new($now),
            oldest_dropped_timer_tick_age: std::cell::Cell::new(std::time::Duration::ZERO),
            frame_count: std::cell::Cell::new(0),
            draw_time_accum_ms: std::cell::Cell::new(0.0),
            tick_time_accum_ms: std::cell::Cell::new(0.0),
            tick_count: std::cell::Cell::new(0),
            fps_last_time: $now,
            fps: 0.0,
            fps_last_main_thread_cpu: super::update::main_thread_cpu_time(),
            display_metrics: super::update::DisplayMetrics::default(),
            mouse_position: None,
            inspected_frame: None,
            inspector_visible: false,
            inspector_position: Point::new(100.0, 100.0),
            inspector_state: InspectorState::default(),
            frames_panel_collapsed: true,
            last_on_update_time: $now,
            saved_vars: $init.saved_vars,
            pending_exec_lua: INIT_EXEC_LUA.with(|cell| cell.borrow_mut().take()),
            selected_xp_level: $selections.xp_level,
            selected_party_size: $selections.party_size,
            last_party_health_tick: $now,
            selected_class: $selections.class,
            selected_race: $selections.race,
            selected_rot_level: $selections.rot_level,
            options_modal_visible: false,
            movement: $selections.movement,
        }
    };
}

impl App {
    pub fn title(_state: &Self) -> String {
        "WoW UI Simulator".to_string()
    }

    pub fn boot() -> (Self, Task<Message>) {
        let (env_rc, saved_vars) = Self::take_init_params();
        let config = crate::config::SimConfig::load();
        Self::apply_config_to_state(&env_rc, &config);

        let log_messages = Vec::new();

        let (texture_manager, font_system, glyph_atlas) = Self::init_rendering(&env_rc);
        let (cmd_rx, lua_rx) = Self::init_servers();
        let (debug_borders, debug_anchors) = Self::resolve_debug_flags();

        let app = Self::build_app(AppInit {
            env: env_rc,
            log_messages,
            texture_manager,
            font_system,
            glyph_atlas,
            cmd_rx,
            lua_rx,
            debug_borders,
            debug_anchors,
            saved_vars,
            config,
        });

        (app, Task::none())
    }

    /// Construct the App struct from initialized components.
    pub(crate) fn build_app(init: AppInit) -> Self {
        let now = std::time::Instant::now();
        let selections = AppInitialSelections::from_config(&init.config);
        let initial_screen_size = current_env_screen_size(&init.env);
        app_from_initial_state!(init, selections, now, initial_screen_size)
    }

    /// Apply saved config to SimState before startup events fire.
    fn apply_config_to_state(env_rc: &Rc<RefCell<WowLuaEnv>>, config: &crate::config::SimConfig) {
        use crate::lua_api::state::{CLASS_LABELS, RACE_DATA, ROT_DAMAGE_LEVELS};
        let env = env_rc.borrow();
        let mut state = env.state().borrow_mut();
        state.player.class_index = CLASS_LABELS
            .iter()
            .position(|&n| n == config.player_class)
            .map(|i| (i + 1) as i32)
            .unwrap_or(1);
        state.player.race_index = RACE_DATA
            .iter()
            .position(|(n, _, _)| *n == config.player_race)
            .unwrap_or(0);
        state.rot_damage_level = ROT_DAMAGE_LEVELS
            .iter()
            .position(|(l, _)| *l == config.rot_damage_level)
            .unwrap_or(0);
        state.player.movement = crate::lua_api::state::MovementState {
            moving: config.movement.moving,
            mounted: config.movement.mounted,
            flying: config.movement.flying,
            falling: config.movement.falling,
            swimming: config.movement.swimming,
        };
        crate::startup::resize_party_state(&mut state, usize::from(config.party_size.min(4)));
    }

    /// Extract init params from thread-local storage.
    /// Fire startup events, apply post-event workarounds, and hide default-hidden frames.
    fn run_startup_sequence(env_rc: &Rc<RefCell<WowLuaEnv>>) {
        fire_startup_events(env_rc);
        let env_ref = env_rc.borrow();
        env_ref.apply_post_event_workarounds();
        crate::startup::settle_startup_animation_groups(&env_ref);
        crate::startup::process_pending_timers(&env_ref);
        Self::settle_gui_startup_on_update(&env_ref);
        crate::lua_api::workarounds::close_startup_special_windows_before_first_frame(&env_ref);
        let _ = crate::lua_api::hide_runtime_hidden_frames(env_ref.lua());
        env_ref.state().borrow_mut().widgets.rebuild_anchor_index();
    }

    fn settle_gui_startup_on_update(env_ref: &WowLuaEnv) {
        for _ in 0..GUI_STARTUP_ON_UPDATE_TICKS {
            crate::startup::fire_gui_startup_on_update_tick(env_ref);
            crate::startup::process_pending_timers(env_ref);
        }
    }

    pub(crate) fn ensure_gui_startup_for_canvas_size(&self, size: Size) {
        if self.gui_startup_complete.get() {
            return;
        }

        self.screen_size.set(size);
        self.env.borrow().set_screen_size(size.width, size.height);
        Self::run_startup_sequence(&self.env);
        {
            let env = self.env.borrow();
            let mut state = env.state().borrow_mut();
            state.initialize_render_state();
            state.ensure_layout_rects();
        }
        self.preload_initial_texture_requests();
        self.gui_startup_complete.set(true);
    }

    fn take_init_params() -> (Rc<RefCell<WowLuaEnv>>, Option<SavedVariablesManager>) {
        let env = INIT_ENV
            .with(|cell| cell.borrow_mut().take())
            .expect("WowLuaEnv not initialized");
        let saved_vars = INIT_SAVED_VARS.with(|cell| cell.borrow_mut().take());
        (Rc::new(RefCell::new(env)), saved_vars)
    }

    /// Create texture manager, font system, and glyph atlas.
    fn init_rendering(env_rc: &Rc<RefCell<WowLuaEnv>>) -> RenderResources {
        let mut tex_mgr =
            TextureManager::new().with_addons_paths(crate::paths::default_addons_paths());
        if Self::eager_startup_texture_preloads_enabled() {
            let class_name = {
                let env = env_rc.borrow();
                let state = env.state().borrow();
                crate::lua_api::state::CLASS_LABELS
                    .get((state.player.class_index - 1).max(0) as usize)
                    .copied()
                    .unwrap_or("Warrior")
                    .to_string()
            };
            let is_glue_screen = {
                let env = env_rc.borrow();
                env.state().borrow().screen_kind.is_glue()
            };
            tex_mgr.preload_talent_textures(790);
            tex_mgr.preload_talent_panel_textures(&class_name);
            if !is_glue_screen {
                Self::preload_non_glue_textures(&mut tex_mgr, &class_name);
            }
        }
        let texture_manager = Rc::new(RefCell::new(tex_mgr));
        let font_system = Rc::new(RefCell::new(WowFontSystem::new()));
        env_rc.borrow().set_font_system(Rc::clone(&font_system));
        let glyph_atlas = Rc::new(RefCell::new(GlyphAtlas::new()));
        (texture_manager, font_system, glyph_atlas)
    }

    fn preload_non_glue_textures(tex_mgr: &mut TextureManager, class_name: &str) {
        if let Some(skill_line) = crate::lua_api::globals::spellbook_data::get_skill_line(2)
            && skill_line.name != class_name
        {
            tex_mgr.preload_talent_panel_textures(skill_line.name);
        }
        tex_mgr.preload_game_hud_textures();
        tex_mgr.preload_playerspells_runtime_textures();
        tex_mgr.preload_spellbook_icons();
    }

    fn eager_startup_texture_preloads_enabled() -> bool {
        std::env::var_os("WOW_SIM_EAGER_STARTUP_TEXTURES").is_some()
    }

    /// Start debug server and Lua REPL server.
    fn init_servers() -> (
        mpsc::Receiver<debug_server::Command>,
        std::sync::mpsc::Receiver<lua_server::LuaCommand>,
    ) {
        let (cmd_rx, _guard) = debug_server::init();
        #[cfg(target_os = "linux")]
        std::mem::forget(_guard);
        #[cfg(not(target_os = "linux"))]
        let _ = _guard;

        let lua_rx = lua_server::init();
        (cmd_rx, lua_rx)
    }

    /// Resolve debug border/anchor flags from CLI and env vars.
    fn resolve_debug_flags() -> (bool, bool) {
        let init_debug = INIT_DEBUG
            .with(|cell| cell.borrow_mut().take())
            .unwrap_or_default();
        let debug_elements = std::env::var("WOW_SIM_DEBUG_ELEMENTS").is_ok();
        let debug_borders =
            init_debug.borders || debug_elements || std::env::var("WOW_SIM_DEBUG_BORDERS").is_ok();
        let debug_anchors =
            init_debug.anchors || debug_elements || std::env::var("WOW_SIM_DEBUG_ANCHORS").is_ok();

        if debug_borders || debug_anchors {
            eprintln!(
                "[wow-sim] Debug mode: borders={} anchors={}",
                debug_borders, debug_anchors
            );
        }
        (debug_borders, debug_anchors)
    }

    pub(crate) fn set_main_thread_phase(&self, phase: &'static str) {
        crate::logging::set_blocking_phase(phase);
        *self.main_thread_phase.borrow_mut() = (phase, std::time::Instant::now());
    }
}

fn current_env_screen_size(env: &Rc<RefCell<WowLuaEnv>>) -> Size {
    let env = env.borrow();
    let state = env.state().borrow();
    Size::new(state.screen_width, state.screen_height)
}

pub(crate) const ALL_STRATA_MASK: u16 = (1 << crate::widget::FrameStrata::COUNT) - 1;

impl App {
    /// Mark specific strata as dirty (need quad re-emit + GPU re-upload).
    pub(crate) fn mark_strata_dirty(&self, mask: u16) {
        self.strata_dirty.set(self.strata_dirty.get() | mask);
    }

    /// Mark ALL strata as dirty (full rebuild).
    /// Also clears per-frame snapshot caches so the next rebuild is non-incremental.
    pub(crate) fn mark_all_strata_dirty(&self) {
        self.strata_dirty.set(ALL_STRATA_MASK);
        *self.cached_frame_snapshots.borrow_mut() = std::array::from_fn(|_| None);
        *self.pending_dirty_ids.borrow_mut() = None;
    }

    pub(crate) fn mark_active_cooldown_widgets_dirty(&self) {
        let env = self.env.borrow();
        let state = env.state().borrow();
        let active_ids: FxHashSet<u64> = active_cooldown_widget_ids(&state).into_iter().collect();
        let previous_ids = self.previous_active_cooldown_widget_ids.borrow();
        for id in active_ids.iter().chain(previous_ids.iter()) {
            state.widgets.mark_visual_dirty(*id);
        }
        drop(previous_ids);
        *self.previous_active_cooldown_widget_ids.borrow_mut() = active_ids;
    }

    /// Determine how often the timer subscription should tick.
    ///
    /// Returns `Some(interval)` when periodic work is needed (animations,
    /// casting, pending C_Timers, rot damage), or `None` when fully idle
    /// (only user input triggers redraws).
    ///
    /// OnUpdate handlers cause a 1-second tick when any frames have them
    /// registered (e.g., buff duration countdown text).
    pub(crate) fn compute_tick_interval(&self) -> Option<std::time::Duration> {
        let env = self.env.borrow();
        let state = env.state().borrow();

        // Fast tick: playing visual animations, active cast, or dirty quads.
        let animation_wake = next_animation_wake(&state);
        let has_animations = animation_wake == Some(std::time::Duration::ZERO);
        let has_casting = state.casting.is_some();
        let has_cooldowns = has_active_cooldowns(&state);
        let is_glue_screen = state.screen_kind.is_glue();
        if has_animations
            || has_casting
            || has_cooldowns
            || self.strata_dirty.get() != 0
            || self.textures_pending.get()
        {
            return Some(fast_tick_interval());
        }
        drop(state);

        // Timer tick: wake up when next C_Timer fires or an unseen animation
        // reaches a loop/finish boundary (min 16ms)
        let wake_delay = [env.next_timer_delay(), animation_wake]
            .into_iter()
            .flatten()
            .min();
        if let Some(delay) = wake_delay {
            return Some(stable_timer_interval(delay));
        }
        drop(env);

        // Slow heartbeat: rot damage needs 2s ticks
        if self.selected_rot_level != "Off" {
            return Some(std::time::Duration::from_secs(2));
        }

        // Glue screens rely on interactive OnUpdate handlers for character
        // rotation and similar motion even when nothing else is animating.
        if is_glue_screen {
            return Some(std::time::Duration::from_millis(33));
        }

        // Idle heartbeat: fire OnUpdate handlers at 1s intervals even when
        // nothing else is active (e.g., buff duration countdown text).
        Some(std::time::Duration::from_secs(1))
    }
}

/// Quantize a pending-timer delay into one of a few fixed tick intervals.
///
/// iced keys `time::every` subscription identity by the interval value, and
/// the subscription set is re-evaluated after every `update()`. Returning the
/// raw remaining time produces a different interval on each update, which
/// tears down and recreates the tick stream — resetting its first-fire delay
/// — every time any message arrives. Under continuous input (mouse movement,
/// IPC commands) the stream is recreated before it ever fires and timer
/// ticks starve completely: pending `C_Timer.After` chains stall, the
/// FPS display freezes at its last value, and anything waiting on a timer
/// (e.g. CoreBehaviorProbe's blocker cleanup) never runs. Fixed buckets keep
/// the interval — and therefore the subscription — stable while a timer's
/// remaining time shrinks; the stream is recreated at most once per bucket
/// boundary, and each bucket interval is at most the remaining time, so
/// timers still fire within one tick of their deadline.
pub(crate) fn stable_timer_interval(delay: std::time::Duration) -> std::time::Duration {
    const BUCKETS_MS: &[u64] = &[1000, 250, 50, 16];
    let delay_ms = delay.as_millis().min(u128::from(u64::MAX)) as u64;
    let bucket = BUCKETS_MS
        .iter()
        .copied()
        .find(|&bucket| delay_ms >= bucket)
        .unwrap_or(16);
    std::time::Duration::from_millis(bucket)
}

fn fast_tick_interval() -> std::time::Duration {
    let fast_tick_ms = std::env::var("WOW_SIM_TICK_MS")
        .ok()
        .as_deref()
        .and_then(parse_fast_tick_ms)
        .unwrap_or(DEFAULT_FAST_TICK_MS);
    std::time::Duration::from_millis(fast_tick_ms)
}

fn parse_fast_tick_ms(value: &str) -> Option<u64> {
    let tick_ms = value.trim().parse::<u64>().ok()?;
    (tick_ms > 0).then_some(tick_ms)
}

/// Delay until a playing animation next needs a tick: zero when one is
/// visible or has an OnUpdate handler, otherwise the nearest loop/finish
/// boundary of unseen ones (timer-only groups or ones under a transparent
/// parent).
fn next_animation_wake(state: &crate::lua_api::SimState) -> Option<std::time::Duration> {
    state
        .animation_groups
        .iter()
        .filter(|(_, g)| {
            g.playing
                && !g.paused
                && g.has_visual_effects()
                && state.widgets.is_ancestor_visible(g.owner_frame_id)
        })
        .filter_map(|(&group_id, g)| {
            let draws_nothing = g.is_timer_only() || is_hidden_by_parent_alpha(&state.widgets, g);
            let is_unseen = draws_nothing && !group_has_on_update_handler(state, group_id);
            if is_unseen {
                g.time_to_next_boundary()
            } else {
                Some(std::time::Duration::ZERO)
            }
        })
        .min()
}

/// OnUpdate on the group or one of its animations runs every tick, so the
/// group needs frame-rate ticks even when nothing it animates can show.
fn group_has_on_update_handler(state: &crate::lua_api::SimState, group_id: u64) -> bool {
    state.on_update_frames.iter().any(|frame_id| {
        let owning_group = state
            .anim_frame_to_group
            .get(frame_id)
            .copied()
            .or_else(|| state.anim_frame_to_anim.get(frame_id).map(|&(id, _)| id));
        owning_group == Some(group_id)
    })
}

/// True when nothing the group animates can show: the owner's parent has
/// effective alpha 0 and no animated frame escapes it via ignoreParentAlpha.
fn is_hidden_by_parent_alpha(
    widgets: &crate::widget::WidgetRegistry,
    group: &crate::lua_api::animation::AnimGroupState,
) -> bool {
    let targets_owner_only = group.animations.iter().all(|a| a.child_key.is_none());
    let Some(owner) = widgets.get(group.owner_frame_id) else {
        return false;
    };
    let parent_alpha = owner
        .parent_id
        .and_then(|parent_id| widgets.get(parent_id))
        .map(|parent| parent.effective_alpha);
    let parent_is_transparent = parent_alpha.is_some_and(|alpha| alpha <= 0.0);
    targets_owner_only && parent_is_transparent && !subtree_ignores_parent_alpha(widgets, owner)
}

fn subtree_ignores_parent_alpha(
    widgets: &crate::widget::WidgetRegistry,
    frame: &crate::widget::Frame,
) -> bool {
    frame.ignore_parent_alpha
        || frame
            .children
            .iter()
            .filter_map(|&child_id| widgets.get(child_id))
            .any(|child| subtree_ignores_parent_alpha(widgets, child))
}

/// Check if any GCD, spell, or visible Cooldown widgets are still active.
fn has_active_cooldowns(state: &crate::lua_api::SimState) -> bool {
    let now = state.start_time.elapsed().as_secs_f64();
    if let Some((start, dur)) = state.gcd {
        if now < start + dur {
            return true;
        }
    }
    if state
        .spell_cooldowns
        .values()
        .any(|cd| now < cd.start + cd.duration)
    {
        return true;
    }
    has_active_cooldown_widget(state, now)
}

pub(super) fn active_cooldown_widget_ids(state: &crate::lua_api::SimState) -> Vec<u64> {
    let now = state.start_time.elapsed().as_secs_f64();
    state
        .widgets
        .cooldown_ids()
        .filter(|&id| is_visible_active_cooldown_widget(state, id, now))
        .collect()
}

fn has_active_cooldown_widget(state: &crate::lua_api::SimState, now: f64) -> bool {
    state
        .widgets
        .cooldown_ids()
        .any(|id| is_visible_active_cooldown_widget(state, id, now))
}

fn is_visible_active_cooldown_widget(state: &crate::lua_api::SimState, id: u64, now: f64) -> bool {
    state
        .widgets
        .get(id)
        .is_some_and(|frame| is_active_cooldown_widget(frame, now))
        && state.widgets.is_ancestor_visible(id)
}

fn is_active_cooldown_widget(frame: &Frame, now: f64) -> bool {
    matches!(frame.widget_type, WidgetType::Cooldown)
        && !frame.cooldown_paused
        && frame.cooldown_duration > 0.0
        && cooldown_elapsed_since_start(frame, now) < frame.cooldown_duration
}

fn cooldown_elapsed_since_start(frame: &Frame, now: f64) -> f64 {
    let mod_rate = if frame.cooldown_mod_rate > 0.0 {
        frame.cooldown_mod_rate
    } else {
        1.0
    };
    (now - frame.cooldown_start) * mod_rate
}

impl Drop for App {
    fn drop(&mut self) {
        if let Some(ref saved_vars) = self.saved_vars {
            let env = self.env.borrow();
            let mut lua = env.rilua_mut();
            match saved_vars.save_all(lua.state_mut()) {
                Ok(()) => crate::logging::eprintln_elapsed(
                    "[wow-sim] SavedVariables saved to simulator storage",
                ),
                Err(e) => crate::logging::eprintln_elapsed(&format!(
                    "[wow-sim] SavedVariables save error: {e}"
                )),
            }
        }
    }
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod app_tests;
