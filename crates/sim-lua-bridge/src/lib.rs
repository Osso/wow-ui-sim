//! Ergonomic bridge layer between rilua's stack API and wow-ui-sim.
//!
//! Provides typed extraction/injection traits and macros that give
//! mlua-like method registration ergonomics without FFI overhead.
//!
//! # Design
//!
//! rilua exposes `RustFn = fn(&mut LuaState) -> LuaResult<u32>`. Arguments
//! live at `state.stack[state.base - 1 .. state.top]` (base points to the
//! first register / arg 0). This layer wraps that convention with typed
//! [`FromStack`] and [`IntoStack`] traits so call sites don't manually
//! index the stack.

mod from_stack;
mod into_stack;
mod macros;
mod multivalue;
mod table_builder;

pub use from_stack::FrameArena;
pub use from_stack::FrameObject;
pub use from_stack::FrameRef;
pub use from_stack::FromMethodSelf;
pub use from_stack::FromStack;
pub use from_stack::stack_val;
pub use from_stack::val_to_i32;
pub use into_stack::IntoStack;
#[doc(hidden)]
pub use macros::push_lua_result;
#[doc(hidden)]
pub use macros::run_method_body;
pub use multivalue::MultiValue;
pub use table_builder::FrameIdentity;
pub use table_builder::TableBuilder;
pub use table_builder::create_frame_table;
pub use table_builder::namespace_peer;
pub use table_builder::set_namespace_peer;
pub use table_builder::table_set_namespace_member;
pub use table_builder::table_set_rust_fn;
pub use table_builder::table_set_rust_fn_static;
pub use table_builder::table_set_rust_fn_unpaired;
