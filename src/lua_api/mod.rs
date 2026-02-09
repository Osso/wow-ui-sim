//! Lua API bindings implementing WoW's addon API.

pub mod animation;
mod builtin_frames;
mod env;
pub(crate) mod frame;
pub(crate) mod keybindings;
mod key_dispatch;
mod frame_methods;
pub mod globals;
mod globals_legacy;
mod layout;
pub(crate) mod loader_env;
pub mod message_frame;
pub(crate) mod script_helpers;
pub mod simple_html;
mod state;
pub mod tooltip;
pub(crate) mod workarounds;
pub(crate) mod workarounds_editmode;

// Re-export public types
pub use env::WowLuaEnv;
pub use layout::{
    anchor_position, compute_frame_rect, frame_position_from_anchor, get_parent_depth, LayoutRect,
};
pub use loader_env::LoaderEnv;
pub use message_frame::MessageFrameData;
pub use simple_html::SimpleHtmlData;
pub use state::{AddonInfo, PendingTimer, SimState, tick_party_health};
pub use tooltip::TooltipData;
pub use globals::global_frames::hide_runtime_hidden_frames;

// Crate-internal re-exports
pub(crate) use env::next_timer_id;
