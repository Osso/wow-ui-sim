//! Frame handle and methods for WoW frame LightUserData.

mod handle;
pub(crate) mod metatable;
pub(crate) mod methods;

pub use handle::{extract_frame_id, frame_lud, get_sim_state, lud_to_id};
pub(crate) use methods::fire_on_show_recursive;
pub(crate) use methods::methods_hierarchy::propagate_strata_level_pub;
