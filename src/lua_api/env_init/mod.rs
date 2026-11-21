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
use crate::lua_api::workarounds::{
    copy_globals_into_secureenv_without_publication_evidence, copy_legacy_unit_globals_to_secureenv,
};
use crate::screen::ScreenKind;
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
pub(crate) use runtime::{record_addon_time, update_threshold_counters};

// Re-export the three functions reused by the EnvironmentCleanup restore workaround.
pub(crate) use bootstrap::init_runtime_surface_bootstrap;
pub(crate) use bootstrap::init_shared_bootstrap;
pub(crate) use enums::init_enum_globals;

/// Opt-in direct global reads only: never invoke getters or inspect/mutate metatables.
pub(crate) fn trace_debug_getter_values(lua: &mut rilua::Lua, phase: &str) {
    if std::env::var("WOW_SIM_TRACE_DEBUG_GETTERS").as_deref() != Ok("1") {
        return;
    }
    for name in ["GetCallstackHeight", "GetErrorCallstackHeight"] {
        let value = LuaApiMut::get_global_val(lua, name);
        eprintln!(
            "[DebugGetterTrace] phase={phase} global={name} type={} value={value:?}",
            value.type_name()
        );
    }
}

/// Initialize the primary rilua state: seed registries, globals, frame methods, and taint.
pub(super) fn init_lua_state(
    lua: &mut rilua::Lua,
    state: Rc<RefCell<SimState>>,
    bootstrap_screen: crate::screen::ScreenKind,
) -> crate::Result<()> {
    trace_debug_getter_values(lua, "init_entry");
    initialize_bootstrap_registry_and_secure_base(lua, &state)?;
    initialize_secure_environment_core(lua)?;
    let skip_chat_frame_placeholder = crate::client_profile::ACTIVE
        == crate::client_profile::ClientProfile::Mists
        && bootstrap_screen.is_glue();
    // register_globals calls gc_stop at its entry; finalize_bootstrap_gc
    // below restores the collector once bootstrap is complete. Between
    // those two points the mark phase is paused.
    trace_debug_getter_values(lua, "before_globals_register");
    super::globals::register_globals(lua, state.clone())?;
    trace_debug_getter_values(lua, "after_globals_register");
    register_runtime_surface_bootstrap(lua)?;
    register_permanent_bootstrap(lua)?;
    register_temporary_bootstrap(lua, skip_chat_frame_placeholder)?;
    register_runtime_api_functions(lua)?;
    trace_debug_getter_values(lua, "before_profile_bootstrap");
    register_profile_bootstrap(lua)?;
    trace_debug_getter_values(lua, "after_profile_bootstrap");
    // Replace typed quest-progress defaults after temporary bootstrap registration.
    super::globals::real::quest_dialog::register_progress_requirement_globals(lua)?;
    copy_legacy_unit_globals_to_secureenv(lua)?;
    let screen_kind = state.borrow().screen_kind;
    install_secure_runtime_surface(lua, screen_kind)?;
    finalize_lua_bootstrap(lua, &state)?;
    Ok(())
}

fn initialize_bootstrap_registry_and_secure_base(
    lua: &mut rilua::Lua,
    state: &Rc<RefCell<SimState>>,
) -> crate::Result<()> {
    registry::init_registry_tables(lua, state)?;
    bootstrap::init_shared_bootstrap(lua)?;
    if crate::client_profile::is_active_retail_12_1_5() {
        // R88ag/R88af.3 formatter; other profiles keep master's patch (timer_layout). Evidence: pr46-49-decomp-answers.md, sections R88ag, R88af.3.
        crate::lua_api::string_format::patch_string_format(lua)?;
    }
    super::globals::security::register_environment_helpers(lua)?;
    super::globals::missing_surface::register_secure_environment_base_globals(lua)?;
    LuaApiMut::register_function(lua, "forceinsecure", forceinsecure)?;
    if crate::client_profile::is_active_retail_12_1_5() {
        super::globals::utility_system_spell::register_secure_environment_base_globals(lua)?;
        crate::lua_api::workarounds::register_secure_environment_base(lua)?;
        // The secure environment snapshots `_G`, so install the native table first.
        super::globals::real::bit_library::register_all(lua)?;
        super::globals::real::math_extensions::register_retail_fast_random_alias(lua)?;
        crate::c_api::prepare_verified_string_library_copy(lua.state_mut());
    }
    Ok(())
}

fn initialize_secure_environment_core(lua: &mut rilua::Lua) -> crate::Result<()> {
    // Legacy profiles still need this table for their post-bootstrap snapshot.
    // C API namespace peers are created only by the Retail 12.1.5-gated helpers.
    super::globals::security::create_secure_environment(lua)?;
    // R88n.10/R88n.11 part 1 verify Constants and Enum are installed after creation. Evidence: pr46-49-decomp-answers.md, sections R88n.10, R88n.11.
    crate::c_api::c_ui_color::register(lua)?;
    publish_retail_secure_template_method_binder(lua)?;
    super::globals::real::mixins::register(lua)?;
    publish_unknown_bootstrap_globals(lua)?;
    enums::init_enum_globals(lua)?;
    frames::init_frame_metatable(lua)
}

fn register_runtime_surface_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    trace_debug_getter_values(lua, "before_runtime_bootstrap");
    bootstrap::init_runtime_surface_bootstrap(lua)?;
    super::globals::real::display_scale::register(lua)?;
    trace_debug_getter_values(lua, "after_runtime_bootstrap");
    #[cfg(feature = "retail-12-1-0")]
    super::globals::security::register_retail_secret_values(lua)?;
    Ok(())
}

fn register_permanent_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    trace_debug_getter_values(lua, "before_permanent_bootstrap");
    crate::lua_api::workarounds::apply_permanent_bootstrap(lua)?;
    trace_debug_getter_values(lua, "after_permanent_bootstrap");
    #[cfg(feature = "retail-12-0-7")]
    super::globals::real::performance_inputs::register(lua)?;
    Ok(())
}

fn register_temporary_bootstrap(
    lua: &mut rilua::Lua,
    skip_chat_frame_placeholder: bool,
) -> crate::Result<()> {
    trace_debug_getter_values(lua, "before_temporary_bootstrap");
    #[cfg(any(feature = "client-retail", feature = "client-ptr"))]
    crate::c_api::c_combat_log::register_filters(lua)?;
    crate::lua_api::workarounds::apply_temporary_bootstrap(lua, skip_chat_frame_placeholder)?;
    trace_debug_getter_values(lua, "after_temporary_bootstrap");
    Ok(())
}

fn register_runtime_api_functions(lua: &mut rilua::Lua) -> crate::Result<()> {
    super::globals::real::heal_prediction::register(lua)?;
    super::globals::real::secure_helpers::register(lua)?;
    super::globals::real::totems::register(lua)?;
    super::globals::real::bank_reagents::register(lua)?;
    crate::c_api::action_combat_range_check::register(lua.state_mut())?;
    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    crate::c_api::performance_bar::register(lua.state_mut())?;
    crate::c_api::c_tts_settings::register(lua)?;
    crate::c_api::c_click_bindings::register(lua)?;
    #[cfg(feature = "retail-12-0-5")]
    crate::c_api::c_click_bindings_spell::register(lua.state_mut())?;
    crate::c_api::c_curve_util::register(lua)?;
    crate::c_api::seconds_formatter::register(lua)?;
    crate::c_api::duration_text_binding::register(lua)
}

fn register_profile_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(feature = "client-wrath")]
    {
        crate::wrath::compat_bootstrap::init(lua)?;
        crate::wrath::compat_bootstrap::init_wrath_only_proxies(lua)?;
    }
    #[cfg(feature = "client-mists")]
    {
        crate::mists::compat_bootstrap::init(lua)?;
        crate::c_api::mists_reputation_list::register(lua.state_mut())?;
        crate::c_api::item_spell::register_mists_legacy_globals(lua.state_mut())?;
    }
    #[cfg(any(feature = "client-era", feature = "client-anniversary"))]
    crate::era::compat_bootstrap::init(lua)?;
    #[cfg(feature = "retail-12-1-0")]
    crate::ptr::compat_bootstrap::init(lua)?;
    if crate::loader::addon_modules::enabled() {
        let publish_require =
            crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::WowForever;
        crate::loader::addon_modules::initialize(lua.state_mut(), publish_require)?;
    }
    #[cfg(feature = "client-wowforever")]
    rilua::table_security::register_table_security(lua)?;
    Ok(())
}

fn install_secure_runtime_surface(
    lua: &mut rilua::Lua,
    screen_kind: ScreenKind,
) -> crate::Result<()> {
    super::globals::real::protected_globals::install(lua)?;
    // temporary: master's `_G` snapshot where native publication is UNKNOWN.
    // Master took it after protected-global wrapping, before color overrides.
    copy_globals_into_secureenv_without_publication_evidence(lua)?;
    // Register the model after secureenv construction. The R88y pass applies Evidence: pr46-49-decomp-answers.md, sections R88y.
    // verified destinations for the active screen. Evidence: pr46-49-decomp-answers.md, sections R88y.y1.
    crate::c_api::c_color_overrides::register(lua)?;
    #[cfg(feature = "forbidden-aspects")]
    super::script_object_transfer::install(lua)?;
    enable_taint_and_wrap_loadstring(lua)?;
    crate::loader::precompiled::init(lua)?;
    // temporary: lt-635e. Keep the full snapshot private to secure-mode and replay chunks.
    super::globals::security::create_secure_replay_environment(lua)?;
    remove_sandbox_globals(lua)?;
    let unmodeled =
        crate::c_api::publish_verified_secure_namespace_members(lua.state_mut(), screen_kind)?;
    if unmodeled > 0 {
        tracing::debug!(
            unmodeled_members = unmodeled,
            "R88y verified members have no simulator implementation"
        );
    }
    frames::init_frame_metatable(lua)
}

fn finalize_lua_bootstrap(
    lua: &mut rilua::Lua,
    state: &Rc<RefCell<SimState>>,
) -> crate::Result<()> {
    trace_debug_getter_values(lua, "before_final_gc");
    finalize_bootstrap_gc(lua)?;
    trace_debug_getter_values(lua, "after_final_gc");
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
    frames::mark_bootstrap_frame_placeholders(state);
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
    Ok(())
}

fn publish_retail_secure_template_method_binder(lua: &mut rilua::Lua) -> LuaResult<()> {
    if !crate::client_profile::is_active_retail_12_1_5() {
        return Ok(());
    }

    // Secure template factories use this simulator helper; it is not a native base name.
    let binder = LuaApiMut::get_global_val(lua, "__wow_bind_xml_method");
    if !matches!(binder, Val::Function(_)) {
        return Err(runtime_error(
            "secure template method binder is missing from ordinary globals",
        ));
    }
    super::globals::security::set_secure_env_key_state(
        lua.state_mut(),
        "__wow_bind_xml_method",
        binder,
    )?;
    Ok(())
}

fn publish_unknown_bootstrap_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    // UNKNOWN (R88k.7): retain existing secure bootstrap values where no Retail native entry replaces them. Evidence: pr46-49-decomp-answers.md, sections R88k.7.
    let names: &[&str] =
        if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail {
            &["CreateAndInitFromMixin"]
        } else {
            &["Mixin", "CreateFromMixins", "CreateAndInitFromMixin"]
        };
    for name in names {
        let value = LuaApiMut::get_global_val(lua, name);
        super::globals::security::set_secure_env_key_state(lua.state_mut(), name, value)?;
    }
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
    if let Some(source) = args.first_mut()
        && let Some(stripped) = strip_utf8_bom(state, *source)
    {
        *source = stripped;
    }
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
        stamp_addon_taint_state(state, &func, LOADSTRING_TAINT_MARKER)?;
    }
    let count = results.len() as u32;
    for value in results {
        state.push(value);
    }
    Ok(count)
}

/// WoW's luaL_loadbuffer skips a leading UTF-8 BOM (3.3.5a lauxlib; Retail
/// 12.1.5 observation in Meorawr/elune#11).
fn strip_utf8_bom(state: &mut LuaState, source: Val) -> Option<Val> {
    let Val::Str(source_ref) = source else {
        return None;
    };
    let rest = state
        .gc
        .string_arena
        .get(source_ref)?
        .data()
        .strip_prefix(b"\xEF\xBB\xBF")?
        .to_vec();
    Some(Val::Str(state.gc.intern_string(&rest)))
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

/// WM105 §7: `forceinsecure()` installs the literal `*** TaintForced ***`
/// source, not an empty one, so the aura gate still denies a forced caller.
const FORCED_TAINT_SOURCE: &str = "*** TaintForced ***";

fn forceinsecure(state: &mut LuaState) -> LuaResult<u32> {
    taint_calling_frame(state, FORCED_TAINT_SOURCE);
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

    #[test]
    fn loadstring_skips_a_leading_utf8_bom() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let result: String = env
            .eval(
                r#"
                local f, err = loadstring("\239\187\191return 1")
                local inner, inner_err = loadstring("return 1 \239\187\191")
                return tostring(f and f()) .. "|" .. tostring(err) .. "|" .. tostring(inner)
                "#,
            )
            .expect("loadstring probe should run");
        assert_eq!(result, "1|nil|nil");
    }
}
