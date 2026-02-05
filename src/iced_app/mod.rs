//! Iced-based UI for rendering WoW frames.
//!
//! This module is split into several submodules:
//! - `app`: App struct and initialization
//! - `state`: State structs (InspectorState, TextOverlay, CanvasMessage)
//! - `styles`: UI styling functions and color palette
//! - `layout`: Frame layout computation and anchor positioning
//! - `view`: App::view() and subscription methods
//! - `update`: App::update() and message handling
//! - `render`: Shader/canvas rendering implementations

mod app;
mod layout;
mod render;
mod state;
mod styles;
mod update;
mod view;

use std::path::PathBuf;

use iced::window::screenshot::Screenshot;

use crate::lua_api::WowLuaEnv;

// Re-export public types
pub use app::App;
pub use layout::{anchor_position, compute_frame_rect, frame_position_from_anchor};
pub use render::build_quad_batch_for_registry;
pub use state::{CanvasMessage, InspectorState, TextOverlay};
pub use styles::palette;

use app::{FALLBACK_TEXTURES_PATH, INIT_ENV, INIT_TEXTURES, LOCAL_TEXTURES_PATH};

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
}

/// Run the iced UI with the given Lua environment.
pub fn run_iced_ui(env: WowLuaEnv) -> Result<(), Box<dyn std::error::Error>> {
    // Prefer local WebP textures, fall back to full repo
    let textures_path = if PathBuf::from(LOCAL_TEXTURES_PATH).exists() {
        PathBuf::from(LOCAL_TEXTURES_PATH)
    } else {
        PathBuf::from(FALLBACK_TEXTURES_PATH)
    };
    run_iced_ui_with_textures(env, textures_path)
}

/// Run the iced UI with the given Lua environment and textures path.
pub fn run_iced_ui_with_textures(
    env: WowLuaEnv,
    textures_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Store in thread-local for the boot function
    INIT_ENV.with(|cell| *cell.borrow_mut() = Some(env));
    INIT_TEXTURES.with(|cell| *cell.borrow_mut() = Some(textures_path));

    iced::application(App::boot, App::update, App::view)
        .title(App::title)
        .subscription(App::subscription)
        .window_size((1024.0, 768.0))
        .run()?;

    Ok(())
}
