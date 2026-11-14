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
    create_secure_environment, create_secure_replay_environment, mark_secure, mark_secure_state,
};
pub(crate) use secure_env::{
    mark_secure_replay_state, native_registration_destinations, publish_global_value_with_flags,
    register_native_function_if_missing_with_flags, register_native_function_with_flags,
    register_pair_list_function, register_unknown_allowlist_function, secure_env_table,
    secure_replay_env_table, set_secure_env_key_state,
};

#[cfg(feature = "retail-12-1-0")]
pub(crate) use secret_values::register_retail_secret_values;

/// Register the secure-environment base helpers before the shared secure table is built.
pub(crate) fn register_environment_helpers(lua: &mut rilua::Lua) -> LuaResult<()> {
    environment::register_environment_helpers(lua)
}

/// Register security globals through their classified publication routes.
pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    if crate::client_profile::is_active_retail_12_1_5() {
        secure_env::register_native_function_with_flags(
            lua,
            "securecallmethod",
            securecallmethod::securecallmethod,
            7,
        )?;
    } else {
        // R88n.11b verifies flag 7 only for Retail 12.1.5.
        secure_env::register_unknown_allowlist_function(
            lua,
            "securecallmethod",
            securecallmethod::securecallmethod,
        )?;
    }
    secure_env::register_unknown_allowlist_function(
        lua,
        "issecurevariable",
        value_access::issecurevariable_override,
    )?;
    secure_env::register_simulator_secure_helper(
        lua,
        "__sim_mark_secret_value",
        value_access::mark_secret_values,
    )?;
    secure_env::register_simulator_secure_helper(
        lua,
        "__sim_mark_slot_taint",
        value_access::mark_slot_taint,
    )?;
    value_access::register_value_access_fallbacks(lua)?;
    #[cfg(feature = "retail-12-0-0")]
    retail_secret_helpers::register(lua)?;
    secret_values::register_scrub_fallbacks(lua)?;
    secure_handler::register_secure_handler_stubs(lua)?;
    state_drivers::register_state_driver_stubs(lua)?;
    // UNKNOWN (R88k.7): retained secure allowlist entry; native flags are not established.
    secure_env::register_unknown_allowlist_function(
        lua,
        "SecureCmdOptionParse",
        cmd_option::secure_cmd_option_parse,
    )?;
    Ok(())
}

/// Register the fallback when nil; Retail also publishes an existing function by its verified flag.
fn register_if_missing(
    lua: &mut rilua::Lua,
    name: &str,
    func: rilua::RustFn,
    retail_flags: u8,
) -> LuaResult<()> {
    if crate::client_profile::is_active_retail_12_1_5() {
        return secure_env::register_native_function_if_missing_with_flags(
            lua,
            name,
            func,
            retail_flags,
        );
    }

    if LuaApiMut::get_global_val(lua, name) == Val::Nil {
        // Other profiles retain the existing allowlist route; their flags remain UNKNOWN.
        secure_env::register_unknown_allowlist_function(lua, name, func)?;
    }
    Ok(())
}
