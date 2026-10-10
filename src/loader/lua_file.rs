//! Lua file loading functionality.

use crate::lua_api::LoaderEnv;
use crate::lua_api::globals::security::mark_secure_state;
use crate::lua_api::methods::create_string;
use crate::lua_api::script_helpers::call_error_handler_state;
use crate::lua_api::taint::stamp_addon_taint_state;
use rilua::LuaApiMut;
use rilua::vm::state::LuaState;
use std::path::Path;
use std::time::Instant;

use super::addon::AddonContext;
use super::bytecode_cache;
use super::bytecode_cache::PutResult;
use super::error::LoadError;
use super::{LoadTiming, LuaFileTiming};

/// Load a Lua file into the environment with addon varargs.
pub fn load_lua_file(
    env: &LoaderEnv<'_>,
    path: &Path,
    ctx: &AddonContext,
    timing: &mut LoadTiming,
) -> Result<(), LoadError> {
    let io_start = Instant::now();
    let bytes = std::fs::read(path).map_err(|e| LoadError::io_with_path(path, e))?;
    timing.io_time += io_start.elapsed();

    let chunk_name = wow_chunk_name(path);
    let patched_source = crate::lua_api::workarounds::patch_lua_source(&bytes, &chunk_name);

    let timing_before = LuaFileTimingSnapshot::from_load_timing(timing);
    let func = compile_lua_file(env, &patched_source, &chunk_name, timing)?;
    #[cfg(feature = "client-wowforever")]
    let module = begin_addon_module(env, &func, path, ctx)?;
    let execution = execute_lua_file(env, func, ctx, &chunk_name, timing);
    #[cfg(feature = "client-wowforever")]
    complete_addon_module(env, module, &execution)?;
    execution?;
    super::record_lua_file_timing(timing_before.diff(&chunk_name, timing));

    Ok(())
}

#[cfg(feature = "client-wowforever")]
fn begin_addon_module(
    env: &LoaderEnv<'_>,
    function: &rilua::Function,
    path: &Path,
    ctx: &AddonContext<'_>,
) -> Result<super::addon_modules::ModuleToken, LoadError> {
    let relative_path = path.strip_prefix(ctx.addon_root).map_err(|_| {
        LoadError::Lua(format!(
            "addon module path {} is outside {}",
            path.display(),
            ctx.addon_root.display()
        ))
    })?;
    let dependencies = read_addon_module_dependencies(env, ctx.name);
    env.with_state(|state| {
        super::addon_modules::begin_file(state, function, ctx.name, relative_path, &dependencies)
            .map_err(|error| LoadError::Lua(error.to_string()))
    })
}

#[cfg(feature = "client-wowforever")]
fn read_addon_module_dependencies(env: &LoaderEnv<'_>, name: &str) -> Vec<String> {
    let state = env.state().borrow();
    let addon = state
        .addons
        .iter()
        .find(|addon| addon.folder_name.eq_ignore_ascii_case(name));
    let Some(addon) = addon else {
        return Vec::new();
    };
    crate::toc::collect_metadata_lists(
        &addon.metadata,
        &[
            "Dep",
            "RequiredDep",
            "Dependencies",
            "RequiredDeps",
            "OptionalDep",
            "OptionalDeps",
        ],
    )
}

#[cfg(feature = "client-wowforever")]
fn complete_addon_module(
    env: &LoaderEnv<'_>,
    token: super::addon_modules::ModuleToken,
    execution: &Result<rilua::Val, LoadError>,
) -> Result<(), LoadError> {
    env.with_state(|state| {
        let result = match execution {
            Ok(value) => super::addon_modules::finish_file(state, token, *value),
            Err(_) => super::addon_modules::abort_file(state, token),
        };
        result.map_err(|error| LoadError::Lua(error.to_string()))
    })
}

struct LuaFileTimingSnapshot {
    compile_time: std::time::Duration,
    call_time: std::time::Duration,
    cache_hits: u32,
    cache_misses: u32,
    cache_lookup_misses: u32,
    cache_replay_failures: u32,
}

impl LuaFileTimingSnapshot {
    fn from_load_timing(timing: &LoadTiming) -> Self {
        Self {
            compile_time: timing.lua_compile_time,
            call_time: timing.lua_call_time,
            cache_hits: timing.cache_hits,
            cache_misses: timing.cache_misses,
            cache_lookup_misses: timing.cache_lookup_misses,
            cache_replay_failures: timing.cache_replay_failures,
        }
    }

    fn diff(self, chunk_name: &str, timing: &LoadTiming) -> LuaFileTiming {
        LuaFileTiming {
            chunk_name: chunk_name.to_string(),
            compile_time: timing.lua_compile_time - self.compile_time,
            call_time: timing.lua_call_time - self.call_time,
            cache_hits: timing.cache_hits - self.cache_hits,
            cache_misses: timing.cache_misses - self.cache_misses,
            cache_lookup_misses: timing.cache_lookup_misses - self.cache_lookup_misses,
            cache_replay_failures: timing.cache_replay_failures - self.cache_replay_failures,
        }
    }
}

fn compile_lua_file(
    env: &LoaderEnv<'_>,
    patched_source: &[u8],
    chunk_name: &str,
    timing: &mut LoadTiming,
) -> Result<rilua::Function, LoadError> {
    let compile_start = Instant::now();
    let func_result = env.with_state(|state| {
        load_cached_or_compile_for_chunk(state, patched_source, chunk_name, timing)
    });
    let compile_elapsed = compile_start.elapsed();
    timing.lua_compile_time += compile_elapsed;
    timing.lua_exec_time += compile_elapsed;

    func_result
}

fn load_cached_or_compile_for_chunk(
    state: &mut LuaState,
    patched_source: &[u8],
    chunk_name: &str,
    timing: &mut LoadTiming,
) -> Result<rilua::Function, LoadError> {
    if chunk_name.starts_with("@Interface/") {
        return load_cached_or_compile(state, patched_source, chunk_name, timing);
    }

    let saved_slots = state.global_slots.take();
    let result = load_cached_or_compile(state, patched_source, chunk_name, timing);
    state.global_slots = saved_slots;
    result
}

fn execute_lua_file(
    env: &LoaderEnv<'_>,
    func: rilua::Function,
    ctx: &AddonContext,
    chunk_name: &str,
    timing: &mut LoadTiming,
) -> Result<rilua::Val, LoadError> {
    let call_start = Instant::now();
    let exec_result =
        env.with_state(|state| execute_compiled_lua_file(state, func, ctx, chunk_name));
    let call_elapsed = call_start.elapsed();
    timing.lua_call_time += call_elapsed;
    timing.lua_exec_time += call_elapsed;

    exec_result
}

fn execute_compiled_lua_file(
    state: &mut LuaState,
    func: rilua::Function,
    ctx: &AddonContext,
    chunk_name: &str,
) -> Result<rilua::Val, LoadError> {
    // The VM tracks this exact closure allocation and applies its addon taint on entry.
    if ctx.taint {
        stamp_addon_taint_state(state, &func, ctx.name)
            .map_err(|e| report_lua_load_error(state, e))?;
    }
    if ctx.use_secure_env {
        mark_secure_state(state, &func).map_err(|e| report_lua_load_error(state, e))?;
    }
    crate::lua_api::loader_env::apply_loading_scoped_fenv_state(state, &func)
        .map_err(|e| report_lua_load_error(state, e))?;
    let result = if ctx.taint {
        execute_budgeted_addon_file(state, func, ctx, chunk_name)
    } else {
        crate::loader::stack_taint::with_secure_stack(state, |state| {
            exec_addon_func(state, func, ctx)
        })
    };
    result.map_err(|e| contextual_lua_load_error(state, e, chunk_name))
}

fn execute_budgeted_addon_file(
    state: &mut LuaState,
    func: rilua::Function,
    ctx: &AddonContext,
    chunk_name: &str,
) -> Result<rilua::Val, LoadError> {
    #[cfg(feature = "retail-12-0-5")]
    return crate::lua_api::execution_budget::with_addon_budget(state, ctx.name, |state| {
        let logging = crate::lua_api::handler_timing::is_enabled();
        let before = logging
            .then(|| state.instruction_budget(ctx.name))
            .flatten();
        let result = exec_addon_func(state, func, ctx);
        if logging {
            let after = state.instruction_budget(ctx.name);
            let line = match &result {
                Ok(_) => crate::lua_api::execution_budget::format_file_budget_success(
                    ctx.name, chunk_name, before, after,
                ),
                Err(_) => crate::lua_api::execution_budget::format_file_budget_error(
                    ctx.name, chunk_name, before, after,
                ),
            };
            eprintln!("{line}");
        }
        result.map_err(|error| rilua::runtime_error(error.to_string()))
    })
    .map_err(|error| LoadError::Lua(error.to_string()));
    #[cfg(not(feature = "retail-12-0-5"))]
    {
        let _ = chunk_name;
        exec_addon_func(state, func, ctx)
    }
}

fn contextual_lua_load_error(
    state: &mut LuaState,
    error: LoadError,
    chunk_name: &str,
) -> LoadError {
    let LoadError::Lua(msg) = error else {
        return error;
    };

    let contextual = format!("{chunk_name}: {msg}");
    call_error_handler_state(state, &contextual);
    LoadError::Lua(contextual)
}

/// Transform path to WoW-style chunk name for debugstack.
fn wow_chunk_name(path: &Path) -> String {
    let path_str = path.display().to_string().replace('\\', "/");
    if let Some(pos) = path_str.find("AddOns/") {
        format!("@Interface/{}", &path_str[pos..])
    } else {
        format!("@{}", path_str)
    }
}

fn exec_addon_func(
    state: &mut LuaState,
    func: rilua::Function,
    ctx: &AddonContext,
) -> Result<rilua::Val, LoadError> {
    let name = create_string(state, ctx.name);
    crate::lua_api::methods::call_function_state(
        state,
        rilua::Val::Function(func.gc_ref()),
        &[name, ctx.table],
    )
    .map_err(|e| LoadError::Lua(e.to_string()))
}

/// Try loading from bytecode cache; compile and cache on miss.
///
/// NOTE on secureenv: this function returns a fresh `rilua::Function` handle
/// whether the compiled body came from the cache or from source. The caller
/// (`load_lua_file`) applies `mark_secure_state` to that handle *after* this
/// function returns, so cache-replayed chunks and fresh compilations are
/// indistinguishable from secureenv's point of view — both get their fenv
/// swapped before the chunk ever runs. That ordering must be preserved if
/// anyone wires actual cache `get`/`put` calls here.
fn load_cached_or_compile(
    lua: &mut LuaState,
    bytes: &[u8],
    chunk_name: &str,
    timing: &mut LoadTiming,
) -> Result<rilua::Function, LoadError> {
    let hash = bytecode_cache::content_hash(bytes, chunk_name);

    if !bytecode_cache::is_disabled() {
        match bytecode_cache::with_cached_bytecode(hash, |bytecode| {
            compile_with_rilua(lua, bytecode, chunk_name)
        }) {
            Some(result) => match result {
                Ok(func) => {
                    timing.cache_hits += 1;
                    return Ok(func);
                }
                Err(_) => {
                    timing.cache_replay_failures += 1;
                }
            },
            None => {
                timing.cache_lookup_misses += 1;
            }
        }
    }

    timing.cache_misses += 1;
    let func = compile_from_source(lua, bytes, chunk_name)?;
    if !bytecode_cache::is_disabled() {
        let bytecode = crate::loader::bytecode::dump_function(lua, &func)?;
        match bytecode_cache::put(hash, &bytecode) {
            PutResult::Stored => timing.cache_store_successes += 1,
            PutResult::Unchanged | PutResult::Skipped => {}
            PutResult::Failed => timing.cache_store_failures += 1,
        }
    }
    Ok(func)
}

fn report_lua_load_error(state: &mut LuaState, err: impl ToString) -> LoadError {
    let msg = err.to_string();
    call_error_handler_state(state, &msg);
    LoadError::Lua(msg)
}

/// Compile Lua source code into a function.
fn compile_from_source(
    lua: &mut LuaState,
    bytes: &[u8],
    chunk_name: &str,
) -> Result<rilua::Function, LoadError> {
    compile_with_rilua(lua, bytes, chunk_name).map_err(|e| report_lua_load_error(lua, e))
}

/// Compile Lua source code using rilua's compiler (pure Rust).
///
/// This compiles source or cached bytecode and returns a rilua Function handle.
pub fn compile_with_rilua<L: LuaApiMut>(
    lua: &mut L,
    bytes: &[u8],
    chunk_name: &str,
) -> Result<rilua::Function, LoadError> {
    // Strip UTF-8 BOM if present
    let bytes = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        bytes
    };
    lua.load_bytes(bytes, chunk_name)
        .map_err(|e| LoadError::Lua(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_cache_chunk_name(prefix: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        format!("@{prefix}_{}_{}", std::process::id(), nanos)
    }

    #[cfg(feature = "retail-12-0-5")]
    #[test]
    fn startup_file_budget_success_preserves_meter_continuity_returns_and_effects() {
        let env = crate::lua_api::WowLuaEnv::new().unwrap();
        let ctx = AddonContext {
            name: "SuccessfulFileBudgetProbe",
            table: env.create_addon_table().unwrap(),
            addon_root: Path::new("Interface/AddOns/SuccessfulFileBudgetProbe"),
            use_secure_env: false,
            taint: true,
        };
        let mut lua = env.rilua_mut();
        let state = lua.state_mut();
        state.set_instruction_budget(ctx.name, Some(1_000));
        let first_chunk = "@Interface/AddOns/SuccessfulFileBudgetProbe/First.lua";
        let first = compile_from_source(
            state,
            b"local _, addon = ...; addon.total = 19; return addon.total",
            first_chunk,
        )
        .unwrap();
        let first_before = state.instruction_budget(ctx.name).unwrap();
        assert_eq!(first_before.used, 0);
        assert_eq!(
            execute_compiled_lua_file(state, first, &ctx, first_chunk).unwrap(),
            rilua::Val::Num(19.0)
        );
        let first_after = state.instruction_budget(ctx.name).unwrap();
        assert_eq!(first_after.limit, Some(1_000));
        assert!(first_after.used > first_before.used);

        let second_chunk = "@Interface/AddOns/SuccessfulFileBudgetProbe/Second.lua";
        let second = compile_from_source(
            state,
            b"local _, addon = ...; local function add() addon.total = addon.total + 23 end; add(); return addon.total",
            second_chunk,
        )
        .unwrap();
        let second_before = state.instruction_budget(ctx.name).unwrap();
        assert_eq!(second_before.used, first_after.used);
        assert_eq!(second_before.limit, first_after.limit);
        assert_eq!(
            execute_compiled_lua_file(state, second, &ctx, second_chunk).unwrap(),
            rilua::Val::Num(42.0)
        );
        let second_after = state.instruction_budget(ctx.name).unwrap();
        assert_eq!(second_after.limit, Some(1_000));
        assert!(second_after.used > second_before.used);
        assert!(second_after.used < 1_000);
        let rilua::Val::Table(table_ref) = ctx.table else {
            panic!("expected addon private table");
        };
        let table = rilua::Table::from_gc_ref(table_ref);
        let total_key = create_string(state, "total");
        assert_eq!(
            table.raw_get(state, total_key).unwrap(),
            rilua::Val::Num(42.0)
        );
    }

    #[cfg(feature = "retail-12-0-5")]
    #[test]
    fn startup_file_budget_error_preserves_cumulative_usage_and_loader_outcome() {
        let env = crate::lua_api::WowLuaEnv::new().unwrap();
        let ctx = AddonContext {
            name: "FileBudgetProbe",
            table: env.create_addon_table().unwrap(),
            addon_root: Path::new("Interface/AddOns/FileBudgetProbe"),
            use_secure_env: false,
            taint: true,
        };
        let mut lua = env.rilua_mut();
        let state = lua.state_mut();
        state.set_instruction_budget(ctx.name, Some(100));
        let chunk = "@Interface/AddOns/FileBudgetProbe/Settings/GroupTools.lua";
        let first = compile_from_source(state, b"return 42", chunk).unwrap();
        assert_eq!(
            execute_compiled_lua_file(state, first, &ctx, chunk).unwrap(),
            rilua::Val::Num(42.0)
        );
        let before = state.instruction_budget(ctx.name).unwrap();
        assert!(before.used > 0 && before.used < 100);

        let failing = compile_from_source(
            state,
            b"local _, addon = ...; addon.entered = true; while true do end; addon.completed = true",
            chunk,
        ).unwrap();
        let error = execute_compiled_lua_file(state, failing, &ctx, chunk).unwrap_err();
        let LoadError::Lua(message) = error else {
            panic!("file exhaustion must remain a Lua load error");
        };
        assert!(message.starts_with(&format!("{chunk}: ")));
        assert!(message.contains("instruction budget exhausted for owner 'FileBudgetProbe'"));
        let after = state.instruction_budget(ctx.name).unwrap();
        assert_eq!(after.limit, Some(100));
        assert_eq!(after.used, 100);

        let next = compile_from_source(
            state,
            b"local _, addon = ...; addon.next_file = true",
            chunk,
        )
        .unwrap();
        assert!(matches!(
            execute_compiled_lua_file(state, next, &ctx, chunk),
            Err(LoadError::Lua(_))
        ));
        assert_eq!(state.instruction_budget(ctx.name).unwrap().used, after.used);
        drop(lua);
        assert_eq!(env.eval::<i32>("return 7").unwrap(), 7);
        let rilua::Val::Table(table_ref) = ctx.table else {
            panic!("expected addon private table");
        };
        let mut lua = env.rilua_mut();
        let state = lua.state_mut();
        let table = rilua::Table::from_gc_ref(table_ref);
        for (key, expected) in [
            ("entered", rilua::Val::Bool(true)),
            ("completed", rilua::Val::Nil),
            ("next_file", rilua::Val::Nil),
        ] {
            let key = create_string(state, key);
            assert_eq!(table.raw_get(state, key).unwrap(), expected);
        }
    }

    #[test]
    fn wow_chunk_name_normalizes_windows_addon_paths() {
        let path = Path::new(
            r"C:\repo\vendor\wow-ui-source\Interface\AddOns\Blizzard_UIParent\Mainline\UIParent.lua",
        );
        assert_eq!(
            wow_chunk_name(path),
            "@Interface/AddOns/Blizzard_UIParent/Mainline/UIParent.lua"
        );
    }

    #[test]
    fn wow_chunk_name_normalizes_windows_blizzard_ui_paths_for_patches() {
        let path =
            Path::new(r"C:\repo\Interface\BlizzardUI\Blizzard_UIParent\Mainline\UIParent.lua");
        assert!(wow_chunk_name(path).ends_with("/UIParent.lua"));
    }

    #[test]
    fn rilua_compilation_matches_source_semantics() {
        let mut lua = rilua::Lua::new().unwrap();
        let func = compile_with_rilua(&mut lua, b"return 40 + 2", "@test")
            .expect("rilua should compile simple expression");
        let results = lua.call_function(&func, &[]).unwrap();
        assert_eq!(results, vec![rilua::Val::Num(42.0)]);
    }

    #[test]
    fn rilua_compilation_strips_bom() {
        let mut lua = rilua::Lua::new().unwrap();
        let source = b"\xEF\xBB\xBFreturn 1";
        let func = compile_with_rilua(&mut lua, source, "@bom_test")
            .expect("rilua should handle BOM-prefixed source");
        let results = lua.call_function(&func, &[]).unwrap();
        assert_eq!(results, vec![rilua::Val::Num(1.0)]);
    }

    #[test]
    fn bytecode_replayed_function_accepts_setfenv() {
        use rilua::LuaApiMut;

        let mut lua = rilua::Lua::new().unwrap();
        let func_from_source =
            compile_with_rilua(&mut lua, b"return MARK_SECURE_PROBE", "@cache_test")
                .expect("source compile should succeed");

        let proto = {
            let state = lua.state_mut();
            let closure = state
                .gc
                .closures
                .get(func_from_source.gc_ref())
                .expect("closure exists");
            match closure {
                rilua::vm::closure::Closure::Lua(cl) => cl.proto.clone(),
                rilua::vm::closure::Closure::Rust(_) => panic!("expected Lua closure"),
            }
        };

        let bytecode = {
            let state = lua.state_mut();
            rilua::vm::dump::dump(&proto, Some(&state.gc.string_arena), false)
        };

        let func_from_bytecode = compile_with_rilua(&mut lua, &bytecode, "@cache_test")
            .expect("bytecode replay should succeed");

        let env_table = LuaApiMut::create_table(&mut lua);
        {
            let state = lua.state_mut();
            let sentinel = rilua::Val::Str(state.gc.intern_string_static(b"from-secureenv"));
            let key = rilua::Val::Str(state.gc.intern_string_static(b"MARK_SECURE_PROBE"));
            env_table.raw_set(state, key, sentinel).unwrap();
        }

        rilua::api::state_set_fenv(lua.state_mut(), &func_from_bytecode, &env_table)
            .expect("state_set_fenv should accept bytecode-replayed closure");

        let results = lua.call_function(&func_from_bytecode, &[]).unwrap();
        let resolved = results.into_iter().next().expect("chunk returns a value");
        let rilua::Val::Str(s) = resolved else {
            panic!("expected string, got {resolved:?}");
        };
        assert_eq!(
            lua.val_as_bytes(rilua::Val::Str(s)).unwrap(),
            b"from-secureenv"
        );
    }

    #[test]
    fn load_cached_or_compile_hits_bytecode_cache_on_second_load() {
        let chunk_name = unique_cache_chunk_name("lua_file_cache");
        let source = format!("return {:?}", chunk_name);

        let mut first_lua = rilua::Lua::new().unwrap();
        let mut first_timing = LoadTiming::default();
        let first_func = {
            let state = first_lua.state_mut();
            load_cached_or_compile(state, source.as_bytes(), &chunk_name, &mut first_timing)
        }
        .expect("first compile should succeed");
        let first_results = first_lua.call_function(&first_func, &[]).unwrap();
        let first_value = first_results
            .into_iter()
            .next()
            .expect("first call returns value");
        assert_eq!(first_timing.cache_hits, 0);
        assert_eq!(first_timing.cache_misses, 1);
        assert_eq!(
            first_lua.val_as_bytes(first_value).unwrap(),
            chunk_name.as_bytes()
        );

        let mut second_lua = rilua::Lua::new().unwrap();
        let mut second_timing = LoadTiming::default();
        let second_func = {
            let state = second_lua.state_mut();
            load_cached_or_compile(state, source.as_bytes(), &chunk_name, &mut second_timing)
        }
        .expect("second compile should succeed");
        let second_results = second_lua.call_function(&second_func, &[]).unwrap();
        let second_value = second_results
            .into_iter()
            .next()
            .expect("second call returns value");
        assert_eq!(
            second_timing.cache_hits, 1,
            "second load should reuse cached bytecode"
        );
        assert_eq!(second_timing.cache_misses, 0);
        assert_eq!(
            second_lua.val_as_bytes(second_value).unwrap(),
            chunk_name.as_bytes()
        );
    }

    #[test]
    fn load_cached_or_compile_counts_bytecode_cache_lookup_misses() {
        let chunk_name = unique_cache_chunk_name("lua_file_cache_lookup_miss");
        let source = format!("return {:?}", chunk_name);

        let mut lua = rilua::Lua::new().unwrap();
        let mut timing = LoadTiming::default();
        {
            let state = lua.state_mut();
            load_cached_or_compile(state, source.as_bytes(), &chunk_name, &mut timing)
        }
        .expect("source compile should succeed");

        assert_eq!(timing.cache_hits, 0);
        assert_eq!(timing.cache_misses, 1);
        assert_eq!(timing.cache_lookup_misses, 1);
        assert_eq!(timing.cache_replay_failures, 0);
        assert_eq!(timing.cache_store_successes, 1);
        assert_eq!(timing.cache_store_failures, 0);
    }

    #[test]
    fn load_cached_or_compile_counts_bytecode_replay_failures() {
        let chunk_name = unique_cache_chunk_name("lua_file_cache_replay_failure");
        let source = format!("return {:?}", chunk_name);
        let hash = bytecode_cache::content_hash(source.as_bytes(), &chunk_name);
        bytecode_cache::put(hash, b"not valid lua bytecode");

        let mut lua = rilua::Lua::new().unwrap();
        let mut timing = LoadTiming::default();
        {
            let state = lua.state_mut();
            load_cached_or_compile(state, source.as_bytes(), &chunk_name, &mut timing)
        }
        .expect("source compile should recover after cache replay failure");

        assert_eq!(timing.cache_hits, 0);
        assert_eq!(timing.cache_misses, 1);
        assert_eq!(timing.cache_lookup_misses, 0);
        assert_eq!(timing.cache_replay_failures, 1);
        assert_eq!(timing.cache_store_successes, 1);
        assert_eq!(timing.cache_store_failures, 0);
    }
}
