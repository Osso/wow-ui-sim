//! rilua equivalents of security_api.rs, secure_env.rs, and loader_env.rs.
//!
//! # Security functions (security_api.rs counterpart)
//!
//! `securecallmethod`, taint helpers (`issecretvalue`, `canaccessvalue`,
//! `canaccessallvalues`, `canaccesstable`), scrub pass-throughs, SecureHandler
//! stubs, state/attribute driver stubs, and `SecureCmdOptionParse`.
//!
//! Functions provided by Elune's C runtime (`issecure`, `issecurevariable`,
//! `securecall`, `securecallfunction`, `forceinsecure`, `hooksecurefunc`,
//! `secureexecuterange`) are NOT registered here.
//!
//! # Secure environment (secure_env.rs counterpart)
//!
//! `create_secure_environment` and `mark_secure` mirror the old mlua
//! secureenv path: create a shallow copy of `_G`, give it its own `Enum`,
//! expose it as `__secureenv`, and retarget secure addon chunks to that env.
//!
//! # Loader environment (loader_env.rs / env_init.rs counterpart)
//!
//! `compile_chunk_rilua` — compiles Lua source bytes via rilua's pure-Rust
//! compiler (`compile_with_rilua`). Entry point for the rilua-side loading path.

use rilua::{LuaApiMut, LuaResult, Val};

mod cmd_option;
#[cfg(feature = "retail-12-0-5")]
pub(crate) use cmd_option::{resolve_cmd_option, resolve_cmd_option_with_unit};
mod environment;
mod loader_env;
#[cfg(feature = "retail-12-0-0")]
mod retail_secret_helpers;
pub(super) mod secret_values;
mod secure_env;
mod secure_handler;
mod securecallmethod;
mod state_drivers;
mod value_access;

pub use loader_env::{compile_chunk_rilua, exec_chunk_rilua};
pub use secure_env::{
    create_secure_environment, mark_secure, mark_secure_state, set_in_both_envs_rilua,
};
pub(crate) use secure_env::{secure_env_table, set_secure_env_key_state};

#[cfg(feature = "retail-12-1-0")]
pub(crate) use secret_values::register_retail_secret_values;

/// Register all security-related globals into rilua's global table.
///
/// Registers the same set as `security_api::register_security_functions` but
/// as rilua `RustFn`s. Elune C-runtime functions are excluded.
pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "securecallmethod", securecallmethod::securecallmethod)?;
    environment::register_environment_helpers(lua)?;
    LuaApiMut::register_function(
        lua,
        "issecurevariable",
        value_access::issecurevariable_override,
    )?;
    LuaApiMut::register_function(
        lua,
        "__sim_mark_secret_value",
        value_access::mark_secret_values,
    )?;
    LuaApiMut::register_function(lua, "__sim_mark_slot_taint", value_access::mark_slot_taint)?;
    value_access::register_value_access_fallbacks(lua)?;
    #[cfg(feature = "retail-12-0-0")]
    retail_secret_helpers::register(lua)?;
    secret_values::register_scrub_fallbacks(lua)?;
    secure_handler::register_secure_handler_stubs(lua)?;
    state_drivers::register_state_driver_stubs(lua)?;
    LuaApiMut::register_function(
        lua,
        "SecureCmdOptionParse",
        cmd_option::secure_cmd_option_parse,
    )?;
    Ok(())
}

/// Register a `RustFn` only if the global is currently nil.
///
/// Mirrors `security_api::set_if_missing` for the rilua path.
fn register_if_missing(lua: &mut rilua::Lua, name: &str, func: rilua::RustFn) -> LuaResult<()> {
    let existing = LuaApiMut::get_global_val(lua, name);
    if existing == Val::Nil {
        LuaApiMut::register_function(lua, name, func)?;
    }
    Ok(())
}
