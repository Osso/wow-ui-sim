//! Initialization helpers for the WoW Lua environment.
//!
//! Standalone functions extracted from `env.rs` that are called during
//! `WowLuaEnv::new()` and from event/script dispatch paths.

mod bootstrap;
mod enums;
mod frames;
mod freeze_globals;
mod registry;
mod runtime;

use std::cell::RefCell;
use std::rc::Rc;

use super::state::SimState;
use crate::lua_api::methods::{registry_get, registry_set};
use crate::lua_api::script_helpers::protected_lua_pcall_state;
use crate::lua_api::taint::stamp_addon_taint_state;
use rilua::LuaApiMut;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const ORIGINAL_LOADSTRING_KEY: &str = "__original_loadstring";
const LOADSTRING_TAINT_MARKER: &str = "*** ForceTaint_Strong ***";
const LOADSTRING_SOURCE_TAINT_PREAMBLE: &[u8] =
    b"debug.setstacktaint(\"*** ForceTaint_Strong ***\")\n";
const DISABLE_GLOBAL_SLOTS_ENV: &str = "WOW_SIM_DISABLE_GLOBAL_SLOTS";

// Re-export public-within-crate symbols that env.rs and globals/ import.
pub(crate) use frames::init_builtin_frames;
pub(crate) use runtime::{
    addon_taint_name, is_blizzard_addon, record_addon_time, update_threshold_counters,
};

// Re-export the three functions reused by the EnvironmentCleanup restore workaround.
pub(crate) use bootstrap::init_runtime_surface_bootstrap;
pub(crate) use bootstrap::init_shared_bootstrap;
pub(crate) use enums::init_enum_globals;

/// Initialize the primary rilua state: seed registries, globals, frame methods, and taint.
pub(super) fn init_lua_state(
    lua: &mut rilua::Lua,
    state: Rc<RefCell<SimState>>,
) -> crate::Result<()> {
    registry::init_registry_tables(lua, &state)?;
    bootstrap::init_shared_bootstrap(lua)?;
    enums::init_enum_globals(lua)?;
    frames::init_frame_metatable(lua)?;
    // register_globals calls gc_stop at its entry; finalize_bootstrap_gc
    // below restores the collector once bootstrap is complete. Between
    // those two points the mark phase is paused.
    super::globals::register_globals(lua, state.clone())?;
    bootstrap::init_runtime_surface_bootstrap(lua)?;
    crate::lua_api::workarounds::apply_permanent_bootstrap(lua)?;
    crate::lua_api::workarounds::apply_temporary_bootstrap(lua)?;
    crate::c_api::c_curve_util::register(lua)?;
    crate::c_api::duration_text_binding::register(lua)?;
    #[cfg(feature = "client-wrath")]
    {
        crate::wrath::compat_bootstrap::init(lua)?;
        crate::wrath::compat_bootstrap::init_wrath_only_proxies(lua)?;
    }
    #[cfg(feature = "client-mists")]
    crate::mists::compat_bootstrap::init(lua)?;
    #[cfg(any(feature = "client-era", feature = "client-anniversary"))]
    crate::era::compat_bootstrap::init(lua)?;
    #[cfg(feature = "retail-12-1-0")]
    crate::ptr::compat_bootstrap::init(lua)?;
    #[cfg(feature = "client-wowforever")]
    {
        lua.exec(include_str!("../workarounds/temporary/securecopy.lua"))?;
        crate::loader::addon_modules::initialize(lua.state_mut())?;
    }
    // secureenv is shallow-copied from `_G` here. It keeps its copy of
    // the dangerous globals (dofile / loadfile / require / string.dump /
    // math.randomseed) so secure chunks — which Blizzard trusts —
    // retain them. Insecure addon code that reads through `_G` sees nil
    // after the cleanup below.
    super::globals::security::create_secure_environment(lua)?;
    #[cfg(feature = "retail-12-1-0")]
    super::script_object_transfer::install(lua)?;
    enable_taint_and_wrap_loadstring(lua)?;
    crate::loader::precompiled::init(lua)?;
    remove_sandbox_globals(lua)?;
    frames::init_frame_metatable(lua)?;
    finalize_bootstrap_gc(lua)?;
    // Opt-in until the "overwrite stable global" audit is complete.
    // Blizzard's SharedXMLBase utility files (Mixin / TableUtil /
    // EnumUtil / FunctionUtil / Compat) overwrite existing `_G`
    // entries during addon load; freezing `_G` rejects those writes
    // and breaks the entire addon pipeline. The infrastructure stays
    // available for measurement / selective freezing of known-stable
    // subtrees (Track 3).
    if std::env::var("WOW_SIM_FREEZE_GLOBALS").as_deref() == Ok("1") {
        freeze_globals::freeze_globals_with_live_shadow(lua)?;
    }
    install_global_slots(lua);
    Ok(())
}

/// Walk the Track 1 whitelist against post-bootstrap `_G` and stash the
/// resulting slot vector on `WowLuaAppData`. Runs after
/// `freeze_globals_with_live_shadow` so the captured values include any
/// post-freeze shadow bootstrapping, but before addon load so the
/// captured snapshot is the canonical pre-addon state.
fn install_global_slots(lua: &mut rilua::Lua) {
    install_global_slots_from_env(lua, std::env::var(DISABLE_GLOBAL_SLOTS_ENV).ok().as_deref());
}

fn install_global_slots_from_env(lua: &mut rilua::Lua, env_value: Option<&str>) {
    if global_slots_disabled_from_env(env_value) {
        return;
    }
    use super::env::WowLuaAppData;
    use super::global_slots;
    use rilua::LuaApiMut;
    let slots = global_slots::install(lua.state_mut());
    let app = lua
        .state_mut()
        .app_data_mut::<WowLuaAppData>()
        .expect("WowLuaEnv rilua app_data should always exist");
    app.global_slots = Some(slots);
}

fn global_slots_disabled_from_env(value: Option<&str>) -> bool {
    value == Some("1")
}

/// Run a full collection to drop bootstrap transients, then re-enable
/// the incremental collector. Called at the end of `init_lua_state`
/// (and should be called again by the binary after addon loading, with
/// a matching `gc_stop` before the addon loads).
fn finalize_bootstrap_gc(lua: &mut rilua::Lua) -> crate::Result<()> {
    use rilua::LuaApiMut;
    lua.gc_collect()?;
    lua.gc_restart();
    Ok(())
}

/// Enable Elune taint tracking and wrap loadstring as secure.
fn enable_taint_and_wrap_loadstring(lua: &mut rilua::Lua) -> crate::Result<()> {
    super::taint::enable_taint_mode(lua);
    let original_loadstring = LuaApiMut::get_global_val(lua, "loadstring");
    if matches!(original_loadstring, Val::Function(_)) {
        let state = lua.state_mut();
        registry_set(state, ORIGINAL_LOADSTRING_KEY, original_loadstring);
        LuaApiMut::set_global_val(lua, "loadstring_untainted", original_loadstring)?;
    }
    LuaApiMut::register_function(lua, "loadstring", tainting_loadstring)?;
    LuaApiMut::register_function(lua, "forceinsecure", forceinsecure)?;
    Ok(())
}

fn tainting_loadstring(state: &mut LuaState) -> LuaResult<u32> {
    let original = match registry_get(state, ORIGINAL_LOADSTRING_KEY) {
        Val::Function(_) => registry_get(state, ORIGINAL_LOADSTRING_KEY),
        _ => {
            return Err(runtime_error(
                "loadstring wrapper missing original implementation",
            ));
        }
    };
    let nargs = state.top.saturating_sub(state.base);
    let mut args = (0..nargs)
        .map(|index| state.stack_get(state.base + index))
        .collect::<Vec<_>>();
    let should_taint = !rilua::api::state_is_secure(state);
    if should_taint && let Some(source) = args.first_mut() {
        if let Some(wrapped_source) = wrap_loadstring_source(state, *source) {
            *source = wrapped_source;
        }
    }
    let results = protected_lua_pcall_state(state, original, &args)
        .map_err(|error| runtime_error(format!("loadstring wrapper failed: {error}")))?;
    if should_taint && let Some(Val::Function(func_ref)) = results.first().copied() {
        let func = rilua::Function::from_gc_ref(func_ref);
        stamp_addon_taint_state(state, &func, LOADSTRING_TAINT_MARKER);
    }
    let count = results.len() as u32;
    for value in results {
        state.push(value);
    }
    Ok(count)
}

fn wrap_loadstring_source(state: &mut LuaState, source: Val) -> Option<Val> {
    let Val::Str(source_ref) = source else {
        return None;
    };
    let source = state.gc.string_arena.get(source_ref)?;
    let mut wrapped_source = LOADSTRING_SOURCE_TAINT_PREAMBLE.to_vec();
    wrapped_source.extend_from_slice(source.data());
    Some(Val::Str(state.gc.intern_string(&wrapped_source)))
}

fn forceinsecure(state: &mut LuaState) -> LuaResult<u32> {
    taint_calling_frame(state, "");
    Ok(0)
}

fn taint_calling_frame(state: &mut LuaState, taint: &str) {
    let target = state.ci.checked_sub(1).unwrap_or(state.ci);
    if let Some(ci) = state.call_stack.get_mut(target) {
        ci.taint = Some(taint.to_string());
    }
}

/// Nil WoW-forbidden globals from `_G`: the filesystem / module
/// loaders (`dofile`, `loadfile`, and non-Forever `require`), the bytecode dumper
/// (`string.dump`), and the process-global RNG seeder
/// (`math.randomseed`).
///
/// Runs AFTER `create_secure_environment` so `__secureenv` keeps its
/// shallow copy — secure chunks (audited Blizzard code) retain these
/// tools. Insecure addon code that reads through `_G` sees nil.
///
/// Covered by `tests/sandbox_dangerous_globals.rs`.
fn remove_sandbox_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    use rilua::LuaApiMut;
    for name in ["dofile", "loadfile"] {
        LuaApiMut::set_global_val(lua, name, rilua::Val::Nil)?;
    }
    // Forever's require imports completed addon modules; it is not a filesystem loader.
    #[cfg(not(feature = "client-wowforever"))]
    LuaApiMut::set_global_val(lua, "require", rilua::Val::Nil)?;
    // `string` and `math` are shared tables: secureenv's shallow copy
    // holds the same reference. Niling a field would mutate both. Swap
    // `_G.string` and `_G.math` for fresh shallow copies minus the
    // dangerous fields — secureenv keeps the originals intact.
    lua.exec(
        r#"
        if string then
            local safe_string = {}
            for k, v in pairs(string) do safe_string[k] = v end
            safe_string.dump = nil
            rawset(_G, "string", safe_string)
        end
        if math then
            local safe_math = {}
            for k, v in pairs(math) do safe_math[k] = v end
            safe_math.randomseed = nil
            rawset(_G, "math", safe_math)
        end
        "#,
    )?;
    sync_string_metatable_to_global_string(lua)?;
    Ok(())
}

pub(crate) fn sync_string_metatable_to_global_string(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(
        r#"
        if type(string) == "table" then
            local string_meta = debug.getmetatable("")
            if type(string_meta) == "table" then
                string_meta.__index = string
            end
        end
        "#,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{global_slots_disabled_from_env, install_global_slots_from_env};
    use crate::lua_api::WowLuaEnv;
    use crate::lua_api::env::WowLuaAppData;
    use crate::lua_api::state::SimState;
    use rilua::{Lua, LuaApi, LuaApiMut};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn global_slots_disable_flag_parses_exact_one() {
        assert!(global_slots_disabled_from_env(Some("1")));
        assert!(!global_slots_disabled_from_env(Some("0")));
        assert!(!global_slots_disabled_from_env(Some("")));
        assert!(!global_slots_disabled_from_env(None));
    }

    #[test]
    fn install_global_slots_from_env_skips_when_disabled() {
        let mut lua = Lua::new().expect("fresh rilua VM");
        lua.state_mut().set_app_data(WowLuaAppData {
            sim_state: Rc::new(RefCell::new(SimState::default())),
            lua: None,
            font_system: None,
            on_update_cache_dirty: true,
            hot_literals: None,
            global_slots: None,
            addon_modules: None,
        });

        install_global_slots_from_env(&mut lua, Some("1"));

        let app = lua
            .state()
            .app_data::<WowLuaAppData>()
            .expect("app data should exist");
        assert!(app.global_slots.is_none());
    }

    #[test]
    fn settings_canvas_category_frames_show_only_when_opened() {
        let env = WowLuaEnv::new().expect("fresh wow lua env");

        let visible_states: (bool, bool, bool, bool) = env
            .eval(
                r#"
                local first = CreateFrame("Frame", "SettingsCanvasFirstProbe", UIParent)
                local second = CreateFrame("Frame", "SettingsCanvasSecondProbe", UIParent)
                first:Show()
                second:Show()

                local firstCategory = InterfaceOptions_AddCategory(first, "First Probe")
                local secondCategory = InterfaceOptions_AddCategory(second, "Second Probe")
                local hiddenAfterRegister = (not first:IsShown()) and (not second:IsShown())

                Settings.OpenToCategory(firstCategory:GetID())
                local firstOpen = first:IsShown() and not second:IsShown()

                Settings.OpenToCategory(secondCategory:GetID())
                local secondOpen = (not first:IsShown()) and second:IsShown()

                return hiddenAfterRegister, firstOpen, secondOpen, firstCategory:GetName() == "First Probe"
                "#,
            )
            .expect("settings canvas visibility probe should run");

        assert!(
            visible_states.0,
            "registering addon settings category frames should hide their canvases"
        );
        assert!(
            visible_states.1,
            "opening first category should show only its canvas"
        );
        assert!(
            visible_states.2,
            "opening second category should hide the previous canvas"
        );
        assert!(
            visible_states.3,
            "registered settings category should preserve its display name"
        );
    }
}
