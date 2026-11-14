//! [`TableBuilder`]: fluent builder for creating and populating Lua tables.

use rilua::LuaError;
use rilua::LuaResult;
use rilua::RuntimeError;
use rilua::Val;
use rilua::vm::closure::{Closure, RustClosure, RustFn};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table as RiluaTable;
use rilua::vm::value::Userdata;

use crate::IntoStack;

const NAMESPACE_PEERS_REGISTRY_KEY: &str = "__wow_secure_namespace_peers";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameIdentity {
    pub index: u32,
    pub generation: u32,
}

/// Link the ordinary and secure tables for a native namespace registration.
pub fn set_namespace_peer(
    state: &mut LuaState,
    ordinary: GcRef<RiluaTable>,
    secure: GcRef<RiluaTable>,
) -> LuaResult<()> {
    let peers = namespace_peers_table(state, true)?.ok_or_else(table_collected_error)?;
    set_peer(state, peers, ordinary, secure)?;
    set_peer(state, peers, secure, ordinary)
}

/// Return the paired table for a namespace, if registration linked one.
pub fn namespace_peer(
    state: &mut LuaState,
    table_ref: GcRef<RiluaTable>,
) -> LuaResult<Option<GcRef<RiluaTable>>> {
    let Some(peers) = namespace_peers_table(state, false)? else {
        return Ok(None);
    };
    let value = state
        .gc
        .tables
        .get(peers)
        .map(|table| table.get(Val::Table(table_ref), &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    Ok(match value {
        Val::Table(peer) => Some(peer),
        _ => None,
    })
}

/// Write one registration value into a namespace and its paired environment table.
pub fn table_set_namespace_member(
    state: &mut LuaState,
    table_ref: GcRef<RiluaTable>,
    key: Val,
    value: Val,
) -> LuaResult<()> {
    let peer = namespace_peer(state, table_ref)?;
    set_table_member(state, table_ref, key, value)?;
    if let Some(peer) = peer {
        set_table_member(state, peer, key, value)?;
    }
    Ok(())
}

fn namespace_peers_table(
    state: &mut LuaState,
    create: bool,
) -> LuaResult<Option<GcRef<RiluaTable>>> {
    let registry_key = state
        .gc
        .intern_string_static(NAMESPACE_PEERS_REGISTRY_KEY.as_bytes());
    let registry = state.registry;
    let existing = state
        .gc
        .tables
        .get(registry)
        .map(|table| table.get_str(registry_key, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    if let Val::Table(table_ref) = existing {
        return Ok(Some(table_ref));
    }
    if !create {
        return Ok(None);
    }

    let peers = state.gc.alloc_table(RiluaTable::new());
    let registry_table = state
        .gc
        .tables
        .get_mut(registry)
        .ok_or_else(table_collected_error)?;
    registry_table.raw_set(
        Val::Str(registry_key),
        Val::Table(peers),
        &state.gc.string_arena,
    )?;
    state.gc.barrier_back(registry);
    Ok(Some(peers))
}

fn set_peer(
    state: &mut LuaState,
    peers: GcRef<RiluaTable>,
    table_ref: GcRef<RiluaTable>,
    peer: GcRef<RiluaTable>,
) -> LuaResult<()> {
    let table = state
        .gc
        .tables
        .get_mut(peers)
        .ok_or_else(table_collected_error)?;
    table.raw_set(
        Val::Table(table_ref),
        Val::Table(peer),
        &state.gc.string_arena,
    )?;
    state.gc.barrier_back(peers);
    Ok(())
}

fn set_table_member(
    state: &mut LuaState,
    table_ref: GcRef<RiluaTable>,
    key: Val,
    value: Val,
) -> LuaResult<()> {
    let table = state
        .gc
        .tables
        .get_mut(table_ref)
        .ok_or_else(table_collected_error)?;
    table.raw_set(key, value, &state.gc.string_arena)?;
    state.gc.barrier_back(table_ref);
    Ok(())
}

// ---------------------------------------------------------------------------
// Free helper (also used by macros)
// ---------------------------------------------------------------------------

/// Register a named `RustFn` on an existing table `GcRef`.
///
/// Used by the `define_methods!` and `define_functions!` macros so they
/// do not need to go through a full `TableBuilder`.
///
/// Prefer [`table_set_rust_fn_static`] when `name` is a compile-time
/// literal — the pointer-keyed static intern cache short-circuits
/// the content hash on every repeat call.
pub fn table_set_rust_fn(
    state: &mut LuaState,
    table_ref: GcRef<RiluaTable>,
    name: &str,
    func: RustFn,
) -> LuaResult<()> {
    let key_ref = state.gc.intern_string(name.as_bytes());
    install_rust_fn(state, table_ref, key_ref, name, func, true)
}

/// Same as [`table_set_rust_fn`] but routes the key through
/// `intern_string_static`, skipping the content-hash lookup when the
/// pointer is already in the static intern cache. Use this whenever
/// `name` is a compile-time literal.
pub fn table_set_rust_fn_static(
    state: &mut LuaState,
    table_ref: GcRef<RiluaTable>,
    name: &'static str,
    func: RustFn,
) -> LuaResult<()> {
    let name_bytes: &'static [u8] = name.as_bytes();
    let key_ref = state.gc.intern_string_static(name_bytes);
    install_rust_fn(state, table_ref, key_ref, name, func, true)
}

/// Register a `RustFn` in only the requested namespace table.
///
/// Unlike [`table_set_rust_fn`], this does not mirror the write to a paired
/// namespace. Use it when the caller has already selected the destinations.
pub fn table_set_rust_fn_unpaired(
    state: &mut LuaState,
    table_ref: GcRef<RiluaTable>,
    name: &str,
    func: RustFn,
) -> LuaResult<()> {
    let key_ref = state.gc.intern_string(name.as_bytes());
    install_rust_fn(state, table_ref, key_ref, name, func, false)
}

fn install_rust_fn(
    state: &mut LuaState,
    table_ref: GcRef<RiluaTable>,
    key_ref: GcRef<rilua::vm::string::LuaString>,
    name: &str,
    func: RustFn,
    mirror_namespace_peer: bool,
) -> LuaResult<()> {
    let key = Val::Str(key_ref);
    let closure = Closure::Rust(RustClosure::new(func, name));
    let closure_ref = state.gc.alloc_closure(closure);
    let stack_slot = state.top;
    state.ensure_stack(stack_slot + 1);
    state.stack_set(stack_slot, Val::Function(closure_ref));
    state.top = stack_slot + 1;
    let value = Val::Function(closure_ref);
    let result = if mirror_namespace_peer {
        table_set_namespace_member(state, table_ref, key, value)
    } else {
        set_table_member(state, table_ref, key, value)
    };
    state.top = stack_slot;
    result
}

fn table_collected_error() -> LuaError {
    LuaError::Runtime(RuntimeError {
        message: "table has been collected".into(),
        level: 0,
        traceback: vec![],
    })
}

/// Allocate a Lua table with wow-ui-sim's frame backing metadata attached.
///
/// The returned table is still a normal Lua table: Lua code can `rawset`,
/// iterate, and attach a metatable as usual.
///
/// Pre-sizes hash part to 64 slots for typical frame properties/children.
/// Methods are accessed via metatable __index, not stored directly.
pub fn create_frame_table(state: &mut LuaState, index: u32, generation: u32) -> GcRef<RiluaTable> {
    let identity_ref = state
        .gc
        .alloc_userdata(Userdata::new(Box::new(FrameIdentity { index, generation })));

    let mut table = RiluaTable::with_sizes(0, 64);
    table.set_backing(Some((index, generation)));
    table
        .raw_set(
            Val::Num(0.0),
            Val::Userdata(identity_ref),
            &state.gc.string_arena,
        )
        .expect("numeric frame identity slot should be settable");
    state.gc.alloc_table(table)
}

// ---------------------------------------------------------------------------
// TableBuilder
// ---------------------------------------------------------------------------

/// Fluent builder for creating Lua tables with typed values and Rust functions.
///
/// ```rust,ignore
/// let t = TableBuilder::new(state)
///     .set("version", "1.0")?
///     .set_function("GetVersion", |_state| { ... })?
///     .build();
/// ```
pub struct TableBuilder<'a> {
    state: &'a mut LuaState,
    table_ref: GcRef<RiluaTable>,
}

impl<'a> TableBuilder<'a> {
    /// Allocates a new empty table and returns a builder for it.
    pub fn new(state: &'a mut LuaState) -> Self {
        let table_ref = state.gc.alloc_table(RiluaTable::new());
        Self { state, table_ref }
    }

    /// Wrap an existing table, such as one returned by `c_api::ensure_namespace`.
    pub fn from_table(state: &'a mut LuaState, table_ref: GcRef<RiluaTable>) -> Self {
        Self { state, table_ref }
    }

    /// Set a string-keyed entry to any value implementing [`IntoStack`].
    ///
    /// `IntoStack` is used only to convert the value to `Val`; no stack
    /// slot is consumed — the value is pushed and immediately popped to
    /// set the table entry.
    pub fn set(self, key: &str, value: impl IntoStack) -> LuaResult<Self> {
        let val = value_to_table_entry(self.state, value)?;
        let key_ref = self.state.gc.intern_string(key.as_bytes());
        table_set_namespace_member(self.state, self.table_ref, Val::Str(key_ref), val)?;
        Ok(self)
    }

    /// Set a string-keyed entry to a named Rust function.
    pub fn set_function(self, name: &str, func: RustFn) -> LuaResult<Self> {
        table_set_rust_fn(self.state, self.table_ref, name, func)?;
        Ok(self)
    }

    /// Return the underlying `GcRef` for direct use with the macro helpers.
    pub fn table_ref(&self) -> GcRef<RiluaTable> {
        self.table_ref
    }

    /// Finish building and return the table as a `Val::Table`.
    pub fn build(self) -> Val {
        Val::Table(self.table_ref)
    }
}

fn value_to_table_entry(state: &mut LuaState, value: impl IntoStack) -> LuaResult<Val> {
    let save_top = state.top;
    let count = value.into_stack(state)?;
    let val = match count {
        0 => Val::Nil,
        1 => state.stack[save_top],
        _ => {
            state.top = save_top;
            return Err(table_builder_value_error(count));
        }
    };
    state.top = save_top;
    Ok(val)
}

fn table_builder_value_error(count: u32) -> LuaError {
    LuaError::Runtime(RuntimeError {
        message: format!("table builder values must push exactly 0 or 1 Lua values, got {count}"),
        level: 0,
        traceback: vec![],
    })
}
