//! Frame handle and methods for WoW frame userdata.

mod handle;
mod methods;

pub use handle::FrameHandle;
pub(crate) use methods::fire_on_show_recursive;
