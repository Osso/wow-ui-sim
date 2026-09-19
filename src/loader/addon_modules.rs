//! Forever's already-loaded addon module registry and native import boundary.

use std::collections::{HashMap, HashSet};
use std::path::{Component, Path};

use crate::lua_api::WowLuaAppData;
use crate::lua_api::methods::{create_table, registry_get, registry_set, table_get, table_set};
use crate::lua_bridge::stack_val;
use rilua::vm::proto::Proto;
use rilua::vm::state::LuaState;
use rilua::{Function, LuaApiMut, LuaResult, Val, runtime_error};

use super::module_import::{ImportError, ModuleIdentity, resolve_import};

const VALUES: &str = "__wow_addon_module_values";
const PROTOTYPE_ROOTS: &str = "__wow_addon_module_functions";

#[derive(Clone)]
struct FileOrigin {
    identity: ModuleIdentity,
    direct_deps: Vec<String>,
}

/// Metadata only: Lua values and prototype-owning closures live in GC roots.
#[derive(Clone, Default)]
pub(crate) struct ModuleRegistry {
    origins: HashMap<usize, FileOrigin>,
    completed: HashSet<String>,
    loading: HashMap<String, u64>,
    next_generation: u64,
}

/// Identifies one load attempt; a later begin supersedes an earlier attempt.
pub(crate) struct ModuleToken {
    key: String,
    generation: u64,
}

/// Install before secure-environment copying. The caller owns profile gating.
pub(crate) fn initialize(state: &mut LuaState) -> LuaResult<()> {
    let app = state
        .app_data_mut::<WowLuaAppData>()
        .ok_or_else(|| runtime_error("missing WowLuaAppData"))?;
    if app.addon_modules.is_none() {
        app.addon_modules = Some(ModuleRegistry::default());
        for key in [VALUES, PROTOTYPE_ROOTS] {
            let table = create_table(state);
            registry_set(state, key, table);
        }
    }
    state.register_function("require", require_module)
}

/// Register file provenance before execution and hide its previous completed value.
pub(crate) fn begin_file(
    state: &mut LuaState,
    function: &Function,
    addon: &str,
    relative_path: &Path,
    direct_deps: &[String],
) -> LuaResult<ModuleToken> {
    let identity = file_identity(addon, relative_path)?;
    let key = module_key(&identity);
    let proto = state
        .gc
        .closures
        .get(function.gc_ref())
        .and_then(|closure| closure.as_lua())
        .map(|closure| closure.proto.clone())
        .ok_or_else(|| runtime_error("addon module must be a Lua function"))?;
    let registry = module_registry_mut(state)?;
    registry.next_generation += 1;
    let generation = registry.next_generation;
    let origin = FileOrigin {
        identity,
        direct_deps: direct_deps.to_vec(),
    };
    register_prototypes(&proto, &origin, &mut registry.origins);
    registry.completed.remove(&key);
    registry.loading.insert(key.clone(), generation);

    // Root the top closure, not merely its Rust prototype pointer: GC must trace
    // constants in it and its nested prototypes, including after failed loads.
    let roots = registry_get(state, PROTOTYPE_ROOTS);
    table_set(
        state,
        roots,
        &generation.to_string(),
        Val::Function(function.gc_ref()),
    );
    let values = registry_get(state, VALUES);
    table_set(state, values, &key, Val::Nil);
    Ok(ModuleToken { key, generation })
}

/// Publish one original return value; nil membership is held separately.
pub(crate) fn finish_file(
    state: &mut LuaState,
    token: ModuleToken,
    first_return: Val,
) -> LuaResult<()> {
    let registry = module_registry_mut(state)?;
    if registry.loading.get(&token.key) != Some(&token.generation) {
        return Ok(());
    }
    registry.loading.remove(&token.key);
    registry.completed.insert(token.key.clone());
    // The loader has already popped the return tuple. Root its original value
    // before registry-key interning or any other Lua allocation.
    let saved_top = state.top;
    state.push(first_return);
    let values = registry_get(state, VALUES);
    table_set(state, values, &token.key, first_return);
    state.top = saved_top;
    Ok(())
}

/// Failed files stay unavailable. Escaped closures keep their file provenance.
pub(crate) fn abort_file(state: &mut LuaState, token: ModuleToken) -> LuaResult<()> {
    let registry = module_registry_mut(state)?;
    if registry.loading.get(&token.key) == Some(&token.generation) {
        registry.loading.remove(&token.key);
    }
    Ok(())
}

fn module_registry(state: &LuaState) -> LuaResult<&ModuleRegistry> {
    state
        .app_data::<WowLuaAppData>()
        .and_then(|app| app.addon_modules.as_ref())
        .ok_or_else(|| runtime_error("addon module registry not initialized"))
}

fn module_registry_mut(state: &mut LuaState) -> LuaResult<&mut ModuleRegistry> {
    state
        .app_data_mut::<WowLuaAppData>()
        .and_then(|app| app.addon_modules.as_mut())
        .ok_or_else(|| runtime_error("addon module registry not initialized"))
}

fn file_identity(addon: &str, relative_path: &Path) -> LuaResult<ModuleIdentity> {
    let mut path = Vec::new();
    for component in relative_path.components() {
        match component {
            Component::Normal(value) => {
                let value = value.to_str().ok_or_else(no_module)?;
                path.push(value.to_owned());
            }
            Component::CurDir => {}
            Component::ParentDir if !path.is_empty() => {
                path.pop();
            }
            _ => return Err(no_module()),
        }
    }
    let filename = path.last_mut().ok_or_else(no_module)?;
    let file = Path::new(filename);
    if !file
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("lua"))
    {
        return Err(no_module());
    }
    *filename = file
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(no_module)?
        .to_owned();
    Ok(ModuleIdentity {
        addon: addon.to_owned(),
        path,
    })
}

fn module_key(identity: &ModuleIdentity) -> String {
    let mut key = identity.addon.to_ascii_lowercase();
    for component in &identity.path {
        key.push('\0');
        key.push_str(component);
    }
    key
}

fn register_prototypes(
    proto: &Proto,
    origin: &FileOrigin,
    origins: &mut HashMap<usize, FileOrigin>,
) {
    origins.insert(proto as *const Proto as usize, origin.clone());
    for nested in &proto.protos {
        register_prototypes(nested, origin, origins);
    }
}

fn caller_origin(state: &LuaState, registry: &ModuleRegistry) -> Option<FileOrigin> {
    // Stop at the nearest Lua frame: an unregistered dynamic chunk must not
    // inherit an older file caller's restrictions or relative directory.
    let frame = state.call_stack[..state.ci]
        .iter()
        .rev()
        .find(|frame| frame.is_lua)?;
    let Val::Function(function) = state.stack_get(frame.func) else {
        return None;
    };
    let closure = state.gc.closures.get(function)?.as_lua()?;
    let identity = closure.proto.as_ref() as *const Proto as usize;
    registry.origins.get(&identity).cloned()
}

fn no_module() -> rilua::LuaError {
    runtime_error(ImportError::NoModule.to_string())
}

fn require_module(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Str(name) = stack_val(state, 1) else {
        return Err(no_module());
    };
    let request = state
        .gc
        .string_arena
        .get(name)
        .and_then(|name| std::str::from_utf8(name.data()).ok())
        .ok_or_else(no_module)?
        .to_owned();
    let registry = module_registry(state)?;
    let origin = caller_origin(state, registry);
    let caller = origin.as_ref().map(|origin| &origin.identity);
    let dependencies = origin
        .as_ref()
        .map_or(&[][..], |origin| origin.direct_deps.as_slice());
    let identity = resolve_import(&request, caller, dependencies)
        .map_err(|error| runtime_error(error.to_string()))?;
    let key = module_key(&identity);
    if !registry.completed.contains(&key) {
        return Err(no_module());
    }
    let values = registry_get(state, VALUES);
    let value = table_get(state, values, &key);
    state.push(value);
    Ok(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua_api::WowLuaAppData;
    use crate::lua_api::state::SimState;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn environment() -> rilua::Lua {
        let mut lua = rilua::Lua::new().unwrap();
        lua.state_mut().set_app_data(WowLuaAppData {
            sim_state: Rc::new(RefCell::new(SimState::default())),
            lua: None,
            font_system: None,
            on_update_cache_dirty: true,
            hot_literals: None,
            global_slots: None,
            addon_modules: None,
        });
        initialize(lua.state_mut()).unwrap();
        lua
    }

    fn load_file(
        lua: &mut rilua::Lua,
        addon: &str,
        path: &str,
        dependencies: &[&str],
        code: &str,
    ) -> LuaResult<()> {
        let chunk = format!("@Interface/AddOns/{addon}/{path}");
        let function = lua.load_bytes(code.as_bytes(), &chunk)?;
        let deps = dependencies
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>();
        let token = begin_file(lua.state_mut(), &function, addon, Path::new(path), &deps)?;
        match lua.call_function(&function, &[]) {
            Ok(values) => finish_file(
                lua.state_mut(),
                token,
                values.first().copied().unwrap_or(Val::Nil),
            ),
            Err(error) => {
                abort_file(lua.state_mut(), token)?;
                Err(error)
            }
        }
    }

    fn run(lua: &mut rilua::Lua, code: &str) {
        let function = lua.load_bytes(code.as_bytes(), "=dynamic-test").unwrap();
        lua.call_function(&function, &[]).unwrap();
    }

    #[test]
    fn completed_values_preserve_table_function_false_nil_and_identity_after_gc() {
        let mut lua = environment();
        for (name, code) in [
            ("Table", "return { answer = 42 }"),
            (
                "Function",
                "local value = { answer = 71 }; return function() return value end",
            ),
            ("False", "return false"),
            ("Nil", "return"),
            ("Number", "return 17"),
            ("String", "return 'module text'"),
        ] {
            load_file(&mut lua, "Library", &format!("{name}.lua"), &[], code).unwrap();
        }
        lua.gc_collect().unwrap();
        lua.gc_collect().unwrap();
        run(
            &mut lua,
            r#"
            assert(require('Library.Table').answer == 42)
            assert(rawequal(require('Library.Table'), require('Library.Table')))
            assert(require('Library.Function')().answer == 71)
            assert(rawequal(require('Library.Function'), require('Library.Function')))
            assert(require('Library.False') == false)
            assert(select('#', require('Library.Nil')) == 1)
            assert(require('Library.Nil') == nil)
            assert(require('Library.Number') == 17)
            assert(require('Library.String') == 'module text')
        "#,
        );
    }

    #[test]
    fn unfinished_self_and_forward_modules_are_missing_until_completion() {
        let mut lua = environment();
        load_file(
            &mut lua,
            "Addon",
            "Core/First.lua",
            &[],
            r#"
            for _, name in ipairs({'Addon.Core.First', 'Addon.Core.Later'}) do
                local ok, err = pcall(require, name)
                assert(not ok)
                assert(string.find(err, 'Invalid import: No module with that name exists', 1, true))
            end
            return 41
        "#,
        )
        .unwrap();
        load_file(
            &mut lua,
            "Addon",
            "Core/Later.lua",
            &[],
            "return require('.First') + 1",
        )
        .unwrap();
        run(&mut lua, "assert(require('Addon.Core.Later') == 42)");
    }

    #[test]
    fn delayed_nested_closures_keep_their_origin_after_gc() {
        let mut lua = environment();
        load_file(
            &mut lua,
            "Addon",
            "Data/Value.lua",
            &[],
            "return { value = 23 }",
        )
        .unwrap();
        load_file(
            &mut lua,
            "Addon",
            "Core/Entry.lua",
            &[],
            r#"
            return function()
                return function() return require('..Data.Value') end
            end
        "#,
        )
        .unwrap();
        lua.gc_collect().unwrap();
        run(
            &mut lua,
            "assert(require('Addon.Core.Entry')()().value == 23)",
        );
    }

    #[test]
    fn required_and_optional_direct_dependencies_are_case_insensitive() {
        let mut lua = environment();
        load_file(
            &mut lua,
            "ExampleLibrary",
            "Formatting/Text.lua",
            &[],
            "return 123",
        )
        .unwrap();
        for (addon, dependency) in [
            ("Required", "examplelibrary"),
            ("Optional", "EXAMPLELIBRARY"),
        ] {
            load_file(
                &mut lua,
                addon,
                "Main.lua",
                &[dependency],
                "return require('ExampleLibrary.Formatting.Text')",
            )
            .unwrap();
        }
        run(
            &mut lua,
            "assert(require('Required.Main') == 123); assert(require('Optional.Main') == 123)",
        );
    }

    #[test]
    fn transitive_dependencies_do_not_authorize_delayed_file_callers() {
        let mut lua = environment();
        load_file(&mut lua, "Library", "Value.lua", &[], "return 7").unwrap();
        load_file(
            &mut lua,
            "Middle",
            "Main.lua",
            &["Library"],
            "return require('Library.Value')",
        )
        .unwrap();
        load_file(
            &mut lua,
            "Consumer",
            "Main.lua",
            &["Middle"],
            "return function() return require('Library.Value') end",
        )
        .unwrap();
        run(
            &mut lua,
            r#"
            local ok, err = pcall(require('Consumer.Main'))
            assert(not ok)
            assert(string.find(err, 'Invalid import: Modules from other addons may only be imported if the calling addon has a direct dependency on the addon being imported', 1, true))
        "#,
        );
    }

    #[test]
    fn dynamic_chunks_with_spoofed_file_source_are_dependency_exempt() {
        let mut lua = environment();
        load_file(&mut lua, "Library", "Value.lua", &[], "return 91").unwrap();
        load_file(&mut lua, "Consumer", "Main.lua", &[], r#"
            return function()
                local f = assert(loadstring("return require('Library.Value')", '@Interface/AddOns/Consumer/Main.lua'))
                return f()
            end
        "#).unwrap();
        run(&mut lua, "assert(require('Consumer.Main')() == 91)");
    }

    #[test]
    fn relative_imports_cannot_escape_addon_even_with_direct_dependency() {
        let mut lua = environment();
        load_file(&mut lua, "Library", "Value.lua", &[], "return 8").unwrap();
        load_file(&mut lua, "Consumer", "Core/Main.lua", &["Library"], r#"
            local ok, err = pcall(require, '...Library.Value')
            assert(not ok)
            assert(string.find(err, 'Invalid import: Relative imports may only be used within the same addon', 1, true))
            return true
        "#).unwrap();
    }

    #[test]
    fn aborted_files_are_missing_and_later_completion_can_replace_them() {
        let mut lua = environment();
        assert!(load_file(&mut lua, "Addon", "Main.lua", &[], "error('broken module')").is_err());
        run(&mut lua, "assert(not pcall(require, 'Addon.Main'))");
        load_file(&mut lua, "Addon", "Main.lua", &[], "return 52").unwrap();
        run(&mut lua, "assert(require('Addon.Main') == 52)");
        assert!(load_file(&mut lua, "Addon", "Main.lua", &[], "error('broken reload')").is_err());
        run(&mut lua, "assert(not pcall(require, 'Addon.Main'))");
    }

    #[test]
    fn module_path_case_is_preserved_while_addon_lookup_is_case_insensitive() {
        let mut lua = environment();
        load_file(&mut lua, "Addon", "Core/Value.lua", &[], "return 73").unwrap();
        run(
            &mut lua,
            "assert(require('aDdOn.Core.Value') == 73); assert(not pcall(require, 'Addon.core.Value'))",
        );
    }

    #[test]
    fn ephemeral_completion_values_survive_collection_and_restore_stack_top() {
        let mut lua = environment();
        lua.gc_set_pause(0);
        lua.gc_set_step_multiplier(1000);
        for index in 0..8 {
            let name = format!("Ephemeral{index}.lua");
            let function = lua
                .load_bytes(
                    b"local value = { text = 'rooted return' }; return function() return value end",
                    &format!("@Interface/AddOns/Addon/{name}"),
                )
                .unwrap();
            let token =
                begin_file(lua.state_mut(), &function, "Addon", Path::new(&name), &[]).unwrap();
            let returned = lua.call_function(&function, &[]).unwrap();
            let first = returned[0];
            drop(returned);
            let top = lua.state_mut().top;
            finish_file(lua.state_mut(), token, first).unwrap();
            assert_eq!(lua.state_mut().top, top);
            lua.gc_step(1000).unwrap();
            lua.gc_collect().unwrap();
            run(
                &mut lua,
                &format!("assert(require('Addon.Ephemeral{index}')().text == 'rooted return')"),
            );
        }
    }

    #[test]
    fn failed_file_escaped_closure_retains_direct_dependency_restriction() {
        let mut lua = environment();
        load_file(&mut lua, "Library", "Value.lua", &[], "return 4").unwrap();
        assert!(
            load_file(
                &mut lua,
                "Consumer",
                "Failed.lua",
                &[],
                r#"
            escaped = function() return require('Library.Value') end
            error('load failed after assigning closure')
        "#
            )
            .is_err()
        );
        lua.gc_collect().unwrap();
        run(
            &mut lua,
            r#"
            local ok, err = pcall(escaped)
            assert(not ok)
            assert(string.find(err, 'direct dependency', 1, true))
        "#,
        );
    }

    #[test]
    fn nearest_dynamic_frame_does_not_inherit_outer_relative_origin() {
        let mut lua = environment();
        load_file(&mut lua, "Addon", "Core/Value.lua", &[], "return 4").unwrap();
        load_file(&mut lua, "Addon", "Core/Entry.lua", &[], r#"
            return function()
                local dynamic = assert(loadstring("return require('.Value')"))
                local ok, err = pcall(dynamic)
                assert(not ok)
                assert(string.find(err, 'Relative imports may only be used within the same addon', 1, true))
            end
        "#).unwrap();
        run(&mut lua, "require('Addon.Core.Entry')()");
    }

    #[test]
    fn imports_and_reinitialization_do_not_reexecute_completed_files() {
        let mut lua = environment();
        load_file(
            &mut lua,
            "Addon",
            "Once.lua",
            &[],
            "executions = (executions or 0) + 1; return executions",
        )
        .unwrap();
        initialize(lua.state_mut()).unwrap();
        run(
            &mut lua,
            "assert(require('Addon.Once') == 1); assert(require('Addon.Once') == 1); assert(executions == 1)",
        );
    }

    #[test]
    fn separate_lua_states_do_not_share_completed_modules() {
        let mut first = environment();
        let mut second = environment();
        load_file(&mut first, "Addon", "Main.lua", &[], "return 17").unwrap();
        run(&mut second, "assert(not pcall(require, 'Addon.Main'))");
        run(&mut first, "assert(require('Addon.Main') == 17)");
    }
}
