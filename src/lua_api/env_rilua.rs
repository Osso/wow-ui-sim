//! rilua-specific methods for WowLuaEnv.

use super::env::WowLuaEnv;
use super::loader_env::{LoaderEnv, create_addon_table};
use rilua::LuaApiMut;
use std::cell::{Ref, RefMut};

impl WowLuaEnv {
    fn with_dynamic_chunk_slots_disabled<R>(
        &self,
        f: impl FnOnce(&mut rilua::Lua) -> rilua::LuaResult<R>,
    ) -> rilua::LuaResult<R> {
        let mut lua = self.lua.borrow_mut();
        let saved_slots = lua.state_mut().global_slots.take();
        let result = f(&mut lua);
        lua.state_mut().global_slots = saved_slots;
        result
    }

    // ── rilua execution paths ────────────────────────────────────────

    /// Execute Lua code on rilua's VM.
    pub fn exec_rilua(&self, code: &str) -> rilua::LuaResult<()> {
        let func = self.load_rilua(code)?;
        let result = self.lua.borrow_mut().call_function(&func, &[]).map(|_| ());
        self.finish_host_lua_call(result)
    }

    /// Execute Lua code on rilua's VM with a custom chunk name.
    pub fn exec_rilua_named(&self, code: &str, name: &str) -> rilua::LuaResult<()> {
        let func = self.load_rilua_named(code, name)?;
        let result = self.lua.borrow_mut().call_function(&func, &[]).map(|_| ());
        self.finish_host_lua_call(result)
    }

    /// Compile Lua code on rilua and return a function handle.
    pub fn load_rilua(&self, code: &str) -> rilua::LuaResult<rilua::Function> {
        self.with_dynamic_chunk_slots_disabled(|lua| LuaApiMut::load(&mut *lua, code))
    }

    /// Compile Lua code on rilua with a custom chunk name.
    pub fn load_rilua_named(&self, code: &str, name: &str) -> rilua::LuaResult<rilua::Function> {
        self.with_dynamic_chunk_slots_disabled(|lua| {
            LuaApiMut::load_bytes(&mut *lua, code.as_bytes(), name)
        })
    }

    /// Compile Lua code, retarget its fenv to the registry secureenv, and
    /// run it — the end-to-end secure-addon load path used by files marked
    /// `[LoadIntoEnvironment secure]` in their TOC. Exposed primarily so
    /// integration tests can exercise fenv isolation without staging an
    /// entire addon directory.
    pub fn exec_rilua_secure(&self, code: &str) -> rilua::LuaResult<()> {
        let func = self.load_rilua(code)?;
        let result = {
            let mut lua = self.lua.borrow_mut();
            super::globals::security::mark_secure(&mut *lua, &func)?;
            lua.call_function(&func, &[]).map(|_| ())
        };
        self.finish_host_lua_call(result)
    }

    /// Dispatch between `exec` (default, runs under `_G`) and
    /// `exec_rilua_secure` (runs under secureenv). Lets callers that
    /// have a `secure: bool` toggle avoid an `if/else` at every use.
    pub fn exec_maybe_secure(&self, code: &str, secure: bool) -> crate::Result<()> {
        self.run_post_startup_exec_hook();
        if secure {
            Ok(self.exec_rilua_secure(code)?)
        } else {
            self.exec(code)
        }
    }

    /// Call a rilua function handle with arguments.
    pub fn call_rilua(
        &self,
        func: &rilua::Function,
        args: &[rilua::Val],
    ) -> rilua::LuaResult<Vec<rilua::Val>> {
        let result = self.lua.borrow_mut().call_function(func, args);
        self.finish_host_lua_call(result)
    }

    pub(crate) fn with_host_input_operation<T>(
        &self,
        operation: impl FnOnce() -> crate::Result<T>,
    ) -> crate::Result<T> {
        self.host_input_depth.set(self.host_input_depth.get() + 1);
        let result = operation();
        self.host_input_depth.set(self.host_input_depth.get() - 1);
        if self.host_input_depth.get() > 0 {
            return result;
        }
        let flush = super::editbox_update::flush_at_host_boundary(self);
        match result {
            Err(error) => Err(error),
            Ok(value) => {
                flush?;
                Ok(value)
            }
        }
    }

    fn finish_host_lua_call<T>(&self, result: rilua::LuaResult<T>) -> rilua::LuaResult<T> {
        let flush =
            super::editbox_update::flush_at_host_boundary(self).map_err(|error| match error {
                crate::Error::Lua(error) => error,
                error => rilua::runtime_error(error.to_string()),
            });
        match result {
            Err(error) => Err(error),
            Ok(value) => {
                flush?;
                Ok(value)
            }
        }
    }

    /// Get access to the primary rilua Lua instance.
    pub fn rilua(&self) -> Ref<'_, rilua::Lua> {
        self.lua.borrow()
    }

    /// Get mutable access to the primary rilua Lua instance.
    pub(crate) fn rilua_mut(&self) -> RefMut<'_, rilua::Lua> {
        self.lua.borrow_mut()
    }

    // ── GC control ───────────────────────────────────────────────────

    /// Stop the Lua garbage collector (defer collection until restart).
    pub fn gc_stop(&self) {
        let mut lua = self.lua.borrow_mut();
        LuaApiMut::gc_stop(&mut *lua);
    }

    /// Restart the Lua garbage collector after a stop.
    pub fn gc_restart(&self) {
        let mut lua = self.lua.borrow_mut();
        LuaApiMut::gc_restart(&mut *lua);
    }

    /// Run a full garbage collection cycle.
    pub fn gc_collect(&self) {
        let mut lua = self.lua.borrow_mut();
        let _ = LuaApiMut::gc_collect(&mut *lua);
    }

    /// Run an incremental GC step with rilua's default budget.
    pub fn gc_step(&self) {
        let mut lua = self.lua.borrow_mut();
        let _ = LuaApiMut::gc_step(&mut *lua, 0);
    }

    /// Run an incremental GC step with an explicit work budget.
    ///
    /// `budget` is measured in GC work units (see `GCSTEPSIZE` in rilua
    /// — currently 1024). Returns whether the full cycle completed. Used
    /// by the OnUpdate tick path to bound collection work per frame.
    pub fn gc_step_with_budget(&self, budget: i64) -> crate::Result<bool> {
        let mut lua = self.lua.borrow_mut();
        Ok(LuaApiMut::gc_step(&mut *lua, budget)?)
    }

    // ── rilua global access ──────────────────────────────────────────

    /// Read a global variable from rilua's global table.
    pub fn get_rilua_global(&self, name: &str) -> rilua::Val {
        let mut lua = self.lua.borrow_mut();
        LuaApiMut::get_global_val(&mut *lua, name)
    }

    /// Set a global variable in rilua's global table.
    pub fn set_rilua_global(&self, name: &str, val: rilua::Val) -> rilua::LuaResult<()> {
        let mut lua = self.lua.borrow_mut();
        LuaApiMut::set_global_val(&mut *lua, name, val)
    }

    /// Register a Rust function as a global in rilua's Lua state.
    pub fn register_rilua_function(&self, name: &str, func: rilua::RustFn) -> rilua::LuaResult<()> {
        let mut lua = self.lua.borrow_mut();
        LuaApiMut::register_function(&mut *lua, name, func)
    }

    /// Build a loader-facing environment wrapper on top of the active rilua VM.
    pub fn loader_env(&self) -> LoaderEnv<'_> {
        LoaderEnv::new(self)
    }

    /// Create the private addon table passed as the second vararg to addon chunks.
    pub fn create_addon_table(&self) -> crate::Result<rilua::Val> {
        let mut lua = self.lua.borrow_mut();
        create_addon_table(&mut lua)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::WowFontSystem;
    use crate::lua_api::env::WowLuaAppData;
    use rilua::LuaApi;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn wow_lua_env_seeds_rilua_app_data_with_sim_state() {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        let rilua = env.rilua();
        let app_data = rilua
            .state()
            .app_data::<WowLuaAppData>()
            .expect("rilua app_data should be seeded");
        assert!(Rc::ptr_eq(&app_data.sim_state, env.state()));
    }

    #[test]
    fn rilua_global_set_get_roundtrip() {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        env.set_rilua_global("__test_val", rilua::Val::Num(42.0))
            .unwrap();
        assert_eq!(env.get_rilua_global("__test_val"), rilua::Val::Num(42.0));
    }

    #[test]
    fn rilua_global_nil_for_missing_key() {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        assert_eq!(
            env.get_rilua_global("__nonexistent_key_xyz"),
            rilua::Val::Nil
        );
    }

    #[test]
    fn register_rilua_function_callable_from_rilua() {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        fn add_one(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<u32> {
            let arg = match state.stack_get(state.base) {
                rilua::Val::Num(n) => n,
                _ => 0.0,
            };
            state.push(rilua::Val::Num(arg + 1.0));
            Ok(1)
        }
        env.register_rilua_function("__test_add_one", add_one)
            .unwrap();
        let func = env.load_rilua("return __test_add_one(5)").unwrap();
        let result = env.call_rilua(&func, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn exec_rilua_runs_code_on_rilua_vm() {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        env.exec_rilua("__rilua_test = 99").unwrap();
        assert_eq!(env.get_rilua_global("__rilua_test"), rilua::Val::Num(99.0));
    }

    #[test]
    fn load_rilua_and_call() {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        let func = env.load_rilua("return 2 + 3").unwrap();
        let results = env.call_rilua(&func, &[]).unwrap();
        assert_eq!(results, vec![rilua::Val::Num(5.0)]);
    }

    #[test]
    fn exec_rilua_named_sets_chunk_name() {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        env.exec_rilua_named("__rilua_named = true", "@test_chunk")
            .unwrap();
        assert_eq!(
            env.get_rilua_global("__rilua_named"),
            rilua::Val::Bool(true)
        );
    }

    #[test]
    fn set_font_system_updates_rilua_app_data() {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        let font_system = Rc::new(RefCell::new(WowFontSystem::new()));
        env.set_font_system(Rc::clone(&font_system));
        let rilua = env.rilua();
        let app_data = rilua
            .state()
            .app_data::<WowLuaAppData>()
            .expect("rilua app_data should be seeded");
        let stored = app_data
            .font_system
            .as_ref()
            .expect("font system should be stored in rilua app_data");
        assert!(Rc::ptr_eq(stored, &font_system));
    }

    #[test]
    fn wow_lua_env_public_surface_uses_rilua_values() {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");

        env.exec("fired = false").unwrap();
        env.exec(
            r#"
            local frame = CreateFrame("Frame")
            frame:RegisterEvent("PLAYER_LOGIN")
            frame:SetScript("OnEvent", function(_, _, value)
                fired = value == 17
            end)
        "#,
        )
        .unwrap();

        env.fire_event_with_args("PLAYER_LOGIN", &[rilua::Val::Num(17.0)])
            .unwrap();

        assert!(env.eval::<bool>("return fired").unwrap());
    }
}
