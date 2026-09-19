//! WoW Lua environment.

use super::env_convert::{FromRiluaResults, unpack_eval_results};
use super::env_init::{init_builtin_frames, init_lua_state};
use super::state::SimState;
use crate::Result;
use crate::font::WowFontSystem;
use crate::lua_api::methods::{create_string, registry_get, registry_set, table_set};
use crate::xml::{clear_templates, register_intrinsic_templates};
use rilua::{LuaApi, LuaApiMut, Val};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TIMER_ID: AtomicU64 = AtomicU64::new(1);
const EVAL_RESULTS_REGISTRY_KEY: &str = "__wow_eval_results";

/// Generate a unique timer ID.
pub(crate) fn next_timer_id() -> u64 {
    NEXT_TIMER_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone)]
pub(crate) struct WowLuaAppData {
    pub(crate) sim_state: Rc<RefCell<SimState>>,
    pub(crate) lua: Option<Rc<RefCell<rilua::Lua>>>,
    pub(crate) font_system: Option<Rc<RefCell<WowFontSystem>>>,
    /// Sticky dirty bit for rebuilding `SimState::on_update_frames` from
    /// registry handler caches when incremental sync could not borrow state.
    pub(crate) on_update_cache_dirty: bool,
    /// Pre-interned handles for the hot-literal whitelist. Populated by
    /// `HotLiteralRegistry::install` during bootstrap (Track 1 sub-item 2).
    /// `None` on a fresh VM before the register-globals pass runs.
    pub(crate) hot_literals: Option<crate::lua_api::hot_literals::HotLiteralHandles>,
    /// Frozen slot vector for the Track 3 global-slot fast path.
    /// Populated by `global_slots::install` at the end of
    /// `init_lua_state`. `None` on a fresh VM before bootstrap runs.
    pub(crate) global_slots: Option<crate::lua_api::global_slots::GlobalSlotTable>,
    #[cfg(any(feature = "client-wowforever", test))]
    pub(crate) addon_modules: Option<crate::loader::addon_modules::ModuleRegistry>,
}

impl WowLuaAppData {
    fn new(sim_state: Rc<RefCell<SimState>>) -> Self {
        Self {
            sim_state,
            lua: None,
            font_system: None,
            on_update_cache_dirty: true,
            hot_literals: None,
            global_slots: None,
            #[cfg(any(feature = "client-wowforever", test))]
            addon_modules: None,
        }
    }
}

/// The WoW Lua environment.
pub struct WowLuaEnv {
    pub(crate) lua: Rc<RefCell<rilua::Lua>>,
    pub(crate) state: Rc<RefCell<SimState>>,
}

impl Drop for WowLuaEnv {
    fn drop(&mut self) {
        if let Ok(state) = self.state.try_borrow() {
            crate::lua_errors::print_suppressed_error_summary(&state);
        }
        clear_app_data_handles(&self.lua);
    }
}

impl WowLuaEnv {
    /// Create a new WoW Lua environment with the API initialized.
    pub fn new() -> Result<Self> {
        crate::logging::eprintln_elapsed("[Startup] WowLuaEnv::new begin");
        let state = timed_startup_phase("SimState::default complete", || {
            Rc::new(RefCell::new(SimState::default()))
        });
        crate::logging::init_process_start_time(state.borrow().start_time);

        let lua = timed_startup_phase("rilua VM created", || {
            Rc::new(RefCell::new(Self::new_rilua(Rc::clone(&state))))
        });

        timed_startup_phase("builtin frames initialized", || init_builtin_frames(&state));
        timed_startup_phase("template registry cleared", clear_templates);
        timed_startup_phase(
            "intrinsic templates registered",
            register_intrinsic_templates,
        );
        timed_startup_phase("initial app_data lua handle installed", || {
            install_app_data_lua_handle(&lua, &lua)
        });

        timed_startup_phase("init_lua_state complete", || {
            let mut lua_ref = lua.borrow_mut();
            init_lua_state(&mut lua_ref, Rc::clone(&state))
        })?;

        let env = Self { lua, state };
        env.install_final_runtime_globals();
        crate::logging::eprintln_elapsed("[Startup] WowLuaEnv::new complete");
        Ok(env)
    }

    fn install_final_runtime_globals(&self) {
        timed_startup_phase("final app_data lua handle installed", || {
            install_app_data_lua_handle(&self.lua, &self.lua)
        });
        timed_startup_phase("initial screen globals installed", || {
            self.install_initial_screen_size_globals()
        });
    }

    fn new_rilua(state: Rc<RefCell<SimState>>) -> rilua::Lua {
        let mut lua = rilua::Lua::new().expect("failed to create rilua Lua state");
        lua.state_mut().set_app_data(WowLuaAppData::new(state));
        lua
    }

    /// Execute Lua code.
    pub fn exec(&self, code: &str) -> Result<()> {
        self.exec_rilua(code)?;
        Ok(())
    }

    /// Borrow the active rilua VM.
    pub fn lua(&self) -> std::cell::Ref<'_, rilua::Lua> {
        self.rilua()
    }

    /// Execute Lua code with a custom chunk name (for better error messages and debugstack).
    pub fn exec_named(&self, code: &str, name: &str) -> Result<()> {
        self.exec_rilua_named(code, name)?;
        Ok(())
    }

    /// Execute Lua code and return the result.
    pub fn eval<T: FromRiluaResults>(&self, code: &str) -> Result<T> {
        {
            let mut lua = self.lua.borrow_mut();
            let state = lua.state_mut();
            registry_set(state, EVAL_RESULTS_REGISTRY_KEY, Val::Nil);
        }
        let body = Self::normalize_eval_body(code);
        let wrapped = format!(
            "local function __wow_eval()\n{body}\nend\ndebug.getregistry().{EVAL_RESULTS_REGISTRY_KEY} = {{ __wow_eval() }}"
        );
        let exec_result = self.exec_rilua(&wrapped);
        let packed_results = {
            let mut lua = self.lua.borrow_mut();
            let state = lua.state_mut();
            let packed = registry_get(state, EVAL_RESULTS_REGISTRY_KEY);
            registry_set(state, EVAL_RESULTS_REGISTRY_KEY, Val::Nil);
            packed
        };
        exec_result?;
        let lua = self.lua.borrow();
        let results = unpack_eval_results(lua.state(), packed_results)?;
        T::from_results(lua.state(), results)
    }

    fn normalize_eval_body(code: &str) -> String {
        let trimmed = code.trim();
        if trimmed.is_empty() {
            return String::new();
        }
        if Self::looks_like_lua_expression(trimmed) {
            format!("return {trimmed}")
        } else {
            code.to_string()
        }
    }

    fn looks_like_lua_expression(code: &str) -> bool {
        if code.contains('\n') || code.contains(';') {
            return false;
        }
        let lower = code.trim_start().to_ascii_lowercase();
        !matches!(
            lower.split_whitespace().next().unwrap_or_default(),
            "return" | "local" | "if" | "for" | "while" | "repeat" | "do" | "function"
        )
    }

    /// Create a Lua string value on the active rilua VM.
    pub fn lua_string(&self, text: &str) -> Val {
        let mut lua = self.lua.borrow_mut();
        create_string(lua.state_mut(), text)
    }

    /// Run a full GC cycle then re-enable the incremental collector.
    ///
    /// Pairs with [`Self::gc_stop`] to bracket bootstrap / addon-load
    /// allocations. The full collection drops transients allocated
    /// while the collector was paused; `gc_restart` resets the debt
    /// threshold so the incremental collector resumes normally.
    pub fn gc_restart_after_bootstrap(&self) -> crate::Result<()> {
        use rilua::LuaApiMut;
        let mut lua = self.lua.borrow_mut();
        lua.gc_collect()?;
        lua.gc_restart();
        Ok(())
    }

    /// Populate the `__addon_names` registry table mapping addon index → folder name.
    pub fn sync_addon_names_to_lua(&self) {
        let addon_names = {
            let state = self.state.borrow();
            state
                .addons
                .iter()
                .map(|addon| addon.folder_name.clone())
                .collect::<Vec<_>>()
        };

        let mut lua = self.lua.borrow_mut();
        let state = lua.state_mut();
        let table = registry_get(state, "__addon_names");
        for (index, addon_name) in addon_names.iter().enumerate() {
            let addon_name_val = create_string(state, addon_name);
            table_set(state, table, &index.to_string(), addon_name_val);
            if let Val::Table(table_ref) = table
                && let Some(table) = state.gc.tables.get_mut(table_ref)
            {
                let _ = table.raw_set(
                    Val::Num(index as f64),
                    addon_name_val,
                    &state.gc.string_arena,
                );
            }
        }
    }

    /// Restore globals that EnvironmentCleanup nil'd but later addons need.
    pub fn restore_post_cleanup_globals(&self) {
        let mut lua = self.rilua_mut();
        let _ = super::workarounds::restore_post_cleanup_globals(&mut lua, Rc::clone(&self.state));
    }

    pub fn sync_string_metatable_to_global_string(&self) {
        let mut lua = self.rilua_mut();
        let _ = super::env_init::sync_string_metatable_to_global_string(&mut lua);
    }

    /// Apply post-load workarounds for Blizzard code that depends on
    /// unimplemented engine features (AnimationGroups, EditMode, etc.).
    pub fn apply_post_load_workarounds(&self) {
        let start_time = self.state().borrow().start_time;
        let log = |message: &str| {
            eprintln!("{} {}", crate::logging::elapsed_prefix(start_time), message);
        };

        log("[Startup] apply_post_load_workarounds: begin");
        super::workarounds::apply(self);
        log("[Startup] apply_post_load_workarounds: workarounds complete");
        self.restore_post_cleanup_globals();
        log("[Startup] apply_post_load_workarounds: globals restored");
        #[cfg(feature = "retail-12-1-0")]
        crate::ptr::compat_bootstrap::apply_post_load(self);
        #[cfg(feature = "retail-12-1-0")]
        log("[Startup] apply_post_load_workarounds: patch 12.1 compatibility restored");
        let _ = self.exec(
            "rawset(_G, 'seterrorhandler', debug.newsecurefunction(rawget(_G, 'seterrorhandler')))",
        );
        log("[Startup] apply_post_load_workarounds: seterrorhandler restored");
    }

    pub fn apply_runtime_addon_load_workarounds(&self, addon_name: &str) {
        super::workarounds::apply_for_runtime_addon_load(&self.loader_env(), addon_name);
    }

    /// Apply workarounds that must run after startup events.
    pub fn apply_post_event_workarounds(&self) {
        if !self
            .state()
            .borrow_mut()
            .mark_post_event_workarounds_applied()
        {
            return;
        }
        super::workarounds::apply_post_event(self);
        #[cfg(feature = "retail-12-1-0")]
        crate::ptr::compat_bootstrap::apply_strict_removals(self);
    }
}

fn timed_startup_phase<T>(label: &str, action: impl FnOnce() -> T) -> T {
    let phase_start = std::time::Instant::now();
    let result = action();
    crate::logging::eprintln_elapsed(&format!(
        "[Startup] {label} in {:.2?}",
        phase_start.elapsed()
    ));
    result
}

fn install_app_data_lua_handle(lua: &Rc<RefCell<rilua::Lua>>, handle: &Rc<RefCell<rilua::Lua>>) {
    let mut lua_ref = lua.borrow_mut();
    let app_data = lua_ref
        .state_mut()
        .app_data_mut::<WowLuaAppData>()
        .expect("WowLuaEnv rilua app_data should always exist");
    app_data.lua = Some(Rc::clone(handle));
}

fn clear_app_data_handles(lua: &Rc<RefCell<rilua::Lua>>) {
    let Ok(mut lua_ref) = lua.try_borrow_mut() else {
        return;
    };

    let Some(app_data) = lua_ref.state_mut().app_data_mut::<WowLuaAppData>() else {
        return;
    };

    app_data.lua = None;
    app_data.font_system = None;
}

#[cfg(test)]
mod tests {
    use super::WowLuaEnv;
    use std::rc::Rc;

    #[test]
    fn drop_releases_rilua_vm() {
        let env = WowLuaEnv::new().expect("env");
        let weak_lua = Rc::downgrade(&env.lua);

        drop(env);

        assert!(
            weak_lua.upgrade().is_none(),
            "WowLuaEnv must not leave a self-cycle through rilua app_data"
        );
    }
}
