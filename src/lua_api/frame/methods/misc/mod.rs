//! rilua RustFn equivalents of the miscellaneous frame methods in `methods_misc.rs`.
//!
//! Each function signature is `pub fn name(state: &mut LuaState) -> LuaResult<u32>`
//! where the return value is the number of results pushed onto the stack.
//!
//! Methods that require mlua table/function support (frame_fields, resolve_and_extract,
//! SetToDefaults) are stubbed with a `// TODO` comment.

pub mod alpha_gradient;
pub mod attribute_stubs;
pub mod bounds;
pub mod drag_input;
pub mod draw_layer;
pub mod frame_buffer;
pub mod frame_level;
pub mod gamepad;
pub mod group_timer;
pub mod highlight;
pub mod propagation;
pub mod render_layers;
pub mod secret;

use rilua::LuaResult;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;

/// Register all miscellaneous frame methods onto the given metatable.
pub fn register_all(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    drag_input::register(state, mt)?;
    propagation::register(state, mt)?;
    gamepad::register(state, mt)?;
    alpha_gradient::register(state, mt)?;
    draw_layer::register(state, mt)?;
    frame_buffer::register(state, mt)?;
    bounds::register(state, mt)?;
    attribute_stubs::register(state, mt)?;
    frame_level::register(state, mt)?;
    secret::register(state, mt)?;
    render_layers::register(state, mt)?;
    highlight::register(state, mt)?;
    group_timer::register(state, mt)?;
    Ok(())
}
