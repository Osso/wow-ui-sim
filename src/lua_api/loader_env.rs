//! Lightweight loader environment for addon loading.

use super::env::WowLuaEnv;
use super::globals::security::mark_secure_state;
use super::methods::create_string;
use crate::Result;
use crate::lua_api::methods::create_table;
use crate::lua_api::script_helpers::call_error_handler_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::Val;
use rilua::vm::state::{GlobalSlotRuntime, LuaState};
use rilua::{Function, LuaApiMut};
use std::cell::{Ref, RefMut};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

const GLOBAL_SLOTS_DIAGNOSTIC_ENV: &str = "WOW_SIM_GLOBAL_SLOTS_DIAGNOSTIC";
const MAX_GLOBAL_SLOTS_DIAGNOSTICS: usize = 256;
static GLOBAL_SLOTS_DIAGNOSTIC_COUNT: AtomicUsize = AtomicUsize::new(0);

fn global_slots_diagnostics_enabled() -> bool {
    std::env::var_os(GLOBAL_SLOTS_DIAGNOSTIC_ENV).is_some()
}

fn emit_global_slots_diagnostic(
    state: &LuaState,
    phase: &str,
    source: &str,
    slots_before: bool,
    saved_slots: Option<&GlobalSlotRuntime>,
) {
    if !global_slots_diagnostics_enabled()
        || GLOBAL_SLOTS_DIAGNOSTIC_COUNT.fetch_add(1, Ordering::Relaxed)
            >= MAX_GLOBAL_SLOTS_DIAGNOSTICS
    {
        return;
    }

    let saved_root_table = match saved_slots {
        Some(runtime) if state.gc.tables.get(runtime.root_global).is_some() => "table",
        Some(_) => "missing",
        None => "none",
    };
    let active_slots = if state.global_slots.is_some() {
        "Some"
    } else {
        "None"
    };
    let saved_slots = if saved_slots.is_some() {
        "Some"
    } else {
        "None"
    };

    eprintln!(
        "[global-slots-diagnostic] phase={phase} source={source:?} slots_before={} active_slots={active_slots} saved_slots={saved_slots} saved_root_table={saved_root_table}",
        if slots_before { "Some" } else { "None" },
    );
}

use super::state::SimState;

pub struct LoaderEnv<'a> {
    lua: Rc<std::cell::RefCell<rilua::Lua>>,
    state: Rc<std::cell::RefCell<SimState>>,
    current_state: Option<NonNull<LuaState>>,
    _marker: PhantomData<&'a WowLuaEnv>,
}

impl<'a> LoaderEnv<'a> {
    pub fn new(env: &'a WowLuaEnv) -> Self {
        Self {
            lua: Rc::clone(&env.lua),
            state: Rc::clone(&env.state),
            current_state: None,
            _marker: PhantomData,
        }
    }

    pub fn from_parts(
        lua: Rc<std::cell::RefCell<rilua::Lua>>,
        state: Rc<std::cell::RefCell<SimState>>,
    ) -> LoaderEnv<'static> {
        Self::from_parts_with_state(lua, state, None)
    }

    pub fn from_parts_active(
        lua: Rc<std::cell::RefCell<rilua::Lua>>,
        state: Rc<std::cell::RefCell<SimState>>,
        current_state: &mut LuaState,
    ) -> LoaderEnv<'static> {
        Self::from_parts_with_state(lua, state, Some(NonNull::from(current_state)))
    }

    fn from_parts_with_state(
        lua: Rc<std::cell::RefCell<rilua::Lua>>,
        state: Rc<std::cell::RefCell<SimState>>,
        current_state: Option<NonNull<LuaState>>,
    ) -> LoaderEnv<'static> {
        LoaderEnv {
            lua,
            state,
            current_state,
            _marker: PhantomData,
        }
    }

    /// Compile generated Lua without slot opcodes, restoring the caller's
    /// runtime even when compilation fails. Nested calls capture and restore
    /// independently, so synchronous Lua callbacks cannot lose their parent
    /// slot runtime.
    fn with_global_slots_disabled<T>(
        state: &mut LuaState,
        source: &str,
        operation: impl FnOnce(&mut LuaState) -> T,
    ) -> T {
        let slots_before = state.global_slots.is_some();
        let saved_slots = state.global_slots.take();
        emit_global_slots_diagnostic(state, "capture", source, slots_before, saved_slots.as_ref());
        emit_global_slots_diagnostic(
            state,
            "compile-entry",
            source,
            slots_before,
            saved_slots.as_ref(),
        );
        let result = operation(state);
        emit_global_slots_diagnostic(
            state,
            "restore-before",
            source,
            slots_before,
            saved_slots.as_ref(),
        );
        state.global_slots = saved_slots;
        emit_global_slots_diagnostic(state, "restore-after", source, slots_before, None);
        result
    }

    fn load_dynamic_chunk_without_slots(
        state: &mut LuaState,
        code: &str,
        tag: &str,
    ) -> Result<rilua::Function> {
        let cache_tag = format!("{tag}-no-global-slots");
        Self::with_global_slots_disabled(state, &cache_tag, |state| {
            crate::loader::chunk_cache::load_chunk(state, code, &cache_tag)
                .map_err(|error| crate::Error::Other(error.to_string()))
        })
    }

    pub fn with_state<T, E>(
        &self,
        f: impl FnOnce(&mut LuaState) -> std::result::Result<T, E>,
    ) -> std::result::Result<T, E> {
        match self.current_state {
            Some(mut current_state) => {
                let state = unsafe { current_state.as_mut() };
                f(state)
            }
            None => {
                let mut lua = self.lua.borrow_mut();
                f(lua.state_mut())
            }
        }
    }

    fn loading_addon_uses_secure_env(&self) -> bool {
        let state = self.state.borrow();
        state
            .loading_addon_index
            .and_then(|idx| state.addons.get(idx as usize))
            .map(|addon| addon.use_secure_env)
            .unwrap_or(false)
    }

    pub fn exec(&self, code: &str) -> Result<()> {
        self.exec_chunk(code, true)
    }

    pub fn exec_public(&self, code: &str) -> Result<()> {
        self.exec_chunk(code, false)
    }

    fn exec_chunk(&self, code: &str, use_loading_environment: bool) -> Result<()> {
        self.with_state(|state| {
            let func = Self::load_dynamic_chunk_without_slots(state, code, "loader-exec")?;
            if use_loading_environment {
                if self.loading_addon_uses_secure_env() {
                    mark_secure_state(state, &func)?;
                }
                apply_loading_scoped_fenv_state(state, &func)?;
            }
            crate::lua_api::script_helpers::call_void_function_state(
                state,
                Val::Function(func.gc_ref()),
                &[],
            )
            .map_err(crate::Error::Other)?;
            Ok(())
        })
    }

    pub fn exec_with_varargs(
        &self,
        code: &str,
        name: &str,
        addon_name: &str,
        addon_table: Val,
    ) -> Result<()> {
        self.with_state(|state| {
            let func = Self::with_global_slots_disabled(state, name, |state| {
                LuaApiMut::load_bytes(state, code.as_bytes(), name)
            })?;
            let addon_name = create_string(state, addon_name);
            crate::lua_api::methods::call_function_state(
                state,
                Val::Function(func.gc_ref()),
                &[addon_name, addon_table],
            )?;
            Ok(())
        })
    }

    pub fn fire_event_with_args(&self, event: &str, args: &[Val]) -> Result<()> {
        let listeners = self.with_state(|state| {
            let slots_before = state.global_slots.is_some();
            emit_global_slots_diagnostic(state, "event-entry", event, slots_before, None);
            Ok::<Vec<u64>, crate::Error>(crate::lua_api::script_helpers::get_event_listeners(
                state, event,
            ))
        })?;
        for widget_id in listeners {
            let result: std::result::Result<(), crate::Error> = self.with_state(|state| {
                let handler =
                    crate::lua_api::script_helpers::get_script(state, widget_id, "OnEvent");
                let Some(handler) = handler else {
                    return Ok(());
                };
                let frame = crate::lua_api::methods::frame_ref(state, widget_id)?;
                let event_name = crate::lua_api::methods::create_string(state, event);
                let mut call_args = Vec::with_capacity(args.len() + 2);
                call_args.push(frame);
                call_args.push(event_name);
                call_args.extend_from_slice(args);
                if let Err(error) = crate::lua_api::script_helpers::call_void_function_state(
                    state, handler, &call_args,
                ) {
                    call_error_handler_state(state, &error);
                }
                Ok(())
            });
            if let Err(error) = result {
                self.with_state(|state| {
                    call_error_handler_state(state, &error.to_string());
                    Ok::<(), crate::Error>(())
                })?;
            }
        }
        Ok(())
    }

    pub fn restore_post_cleanup_globals(&self) -> crate::Result<()> {
        let mut lua = self.lua.borrow_mut();
        super::workarounds::restore_post_cleanup_globals(&mut lua, Rc::clone(&self.state))
    }

    pub fn create_addon_table(&self) -> Result<Val> {
        self.with_state(create_addon_table_state)
    }

    pub fn lua(&self) -> &Rc<std::cell::RefCell<rilua::Lua>> {
        &self.lua
    }

    pub fn rilua(&self) -> Ref<'_, rilua::Lua> {
        self.lua.borrow()
    }

    pub fn rilua_mut(&self) -> RefMut<'_, rilua::Lua> {
        self.lua.borrow_mut()
    }

    pub fn state(&self) -> &Rc<std::cell::RefCell<SimState>> {
        &self.state
    }
}

pub(crate) fn apply_loading_scoped_fenv_state(state: &mut LuaState, func: &Function) -> Result<()> {
    let scoped_env = {
        let sim = super::methods::borrow_state(state)?;
        sim.loading_scoped_script_env
    };
    let Some(Val::Table(env_ref)) = scoped_env else {
        return Ok(());
    };
    let env = rilua::Table::from_gc_ref(env_ref);
    rilua::api::state_set_fenv(state, func, &env)?;
    Ok(())
}

pub(crate) fn create_addon_table(lua: &mut rilua::Lua) -> Result<Val> {
    create_addon_table_state(lua.state_mut())
}

pub(crate) fn create_addon_table_state(state: &mut LuaState) -> Result<Val> {
    let table = create_table(state);
    let Val::Table(table_ref) = table else {
        unreachable!("create_table must return a table");
    };
    table_set_rust_fn_static(state, table_ref, "unpack", addon_table_unpack)?;
    Ok(table)
}

fn addon_table_unpack(state: &mut LuaState) -> rilua::LuaResult<u32> {
    let table = state.stack_get(state.base);
    let values = addon_table_values(state, table);
    for value in values {
        state.push(value);
    }
    Ok(4)
}

fn addon_table_values(state: &LuaState, table: Val) -> [Val; 4] {
    let Val::Table(table_ref) = table else {
        return [Val::Nil, Val::Nil, Val::Nil, Val::Nil];
    };
    let Some(table) = state.gc.tables.get(table_ref) else {
        return [Val::Nil, Val::Nil, Val::Nil, Val::Nil];
    };
    let values = table.array_slice();
    [
        values.first().copied().unwrap_or(Val::Nil),
        values.get(1).copied().unwrap_or(Val::Nil),
        values.get(2).copied().unwrap_or(Val::Nil),
        values.get(3).copied().unwrap_or(Val::Nil),
    ]
}
