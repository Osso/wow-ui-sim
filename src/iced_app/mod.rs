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

mod app;
mod button_vis;
pub mod frame_collect;
mod hit_grid;
mod keybinds;
pub mod layout;
mod message_frame_render;
mod nine_slice;
mod masking;
mod quad_builders;
mod render;
mod statusbar;
mod state;
mod tiling;
pub mod tooltip;
mod styles;
mod mouse;
mod update;
mod update_servers;
mod screenshot;
mod tree_dump;
mod view;

use std::path::PathBuf;

use iced::window::screenshot::Screenshot;

use crate::lua_api::WowLuaEnv;
use crate::saved_variables::SavedVariablesManager;

// Re-export public types
pub use app::App;
pub use layout::{anchor_position, compute_frame_rect, compute_frame_rect_cached, frame_position_from_anchor, CachedFrameLayout, LayoutCache};
pub use render::{build_quad_batch_for_registry, build_hittable_rects};
pub use state::{CanvasMessage, InspectorState};
pub use styles::palette;

pub use app::DebugOptions;
use app::{
    FALLBACK_TEXTURES_PATH, INIT_DEBUG, INIT_ENV, INIT_SAVED_VARS, INIT_TEXTURES,
    LOCAL_TEXTURES_PATH,
};

/// Application messages.
#[derive(Debug, Clone)]
pub enum Message {
    FireEvent(String),
    Scroll(f32, f32),
    ReloadUI,
    CommandInputChanged(String),
    ExecuteCommand,
    ProcessTimers,
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
    /// Keyboard input dispatched to Lua (WoW key name, e.g. "ESCAPE", "ENTER", "A")
    /// plus optional raw text for character input into focused EditBox.
    KeyPress(String, Option<String>),
    /// Player class changed via dropdown.
    PlayerClassChanged(String),
    /// Player race changed via dropdown.
    PlayerRaceChanged(String),
    /// Rot damage level changed via dropdown.
    RotDamageLevelChanged(String),
    /// Toggle options modal visibility.
    ToggleOptionsModal,
    /// Close options modal (backdrop click or Escape).
    CloseOptionsModal,
}

/// Run the iced UI with the given Lua environment.
pub fn run_iced_ui(
    env: WowLuaEnv,
    debug: DebugOptions,
    saved_vars: Option<SavedVariablesManager>,
    exec_lua: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Prefer local WebP textures, fall back to full repo
    let textures_path = if PathBuf::from(LOCAL_TEXTURES_PATH).exists() {
        PathBuf::from(LOCAL_TEXTURES_PATH)
    } else {
        PathBuf::from(FALLBACK_TEXTURES_PATH)
    };
    if let Some(code) = exec_lua {
        app::INIT_EXEC_LUA.with(|cell| *cell.borrow_mut() = Some(code));
    }
    run_iced_ui_with_textures(env, textures_path, debug, saved_vars)
}

/// Run the iced UI with the given Lua environment and textures path.
pub fn run_iced_ui_with_textures(
    env: WowLuaEnv,
    textures_path: PathBuf,
    debug: DebugOptions,
    saved_vars: Option<SavedVariablesManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Store in thread-local for the boot function
    INIT_ENV.with(|cell| *cell.borrow_mut() = Some(env));
    INIT_TEXTURES.with(|cell| *cell.borrow_mut() = Some(textures_path));
    INIT_DEBUG.with(|cell| *cell.borrow_mut() = Some(debug));
    if let Some(sv) = saved_vars {
        INIT_SAVED_VARS.with(|cell| *cell.borrow_mut() = Some(sv));
    }

    iced::application(App::boot, App::update, App::view)
        .title(App::title)
        .subscription(App::subscription)
        .run()?;

    Ok(())
}
