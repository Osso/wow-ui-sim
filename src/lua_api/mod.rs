//! Lua API bindings implementing WoW's addon API.

mod builtin_frames;
mod env;
mod frame;
mod frame_methods;
mod globals;
mod globals_legacy;
mod layout;
mod state;

// Re-export public types
pub use env::WowLuaEnv;
pub use layout::{
    anchor_position, compute_frame_rect, frame_position_from_anchor, get_parent_depth, LayoutRect,
};
pub use state::{AddonInfo, PendingTimer, SimState};

// Crate-internal re-exports
pub(crate) use env::next_timer_id;
