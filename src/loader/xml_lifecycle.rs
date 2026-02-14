//! Lifecycle script firing for XML-created frames (OnLoad, OnShow).

use crate::lua_api::LoaderEnv;

use super::precompiled;

/// Fire OnLoad and OnShow lifecycle scripts after the frame is fully configured.
pub fn fire_lifecycle_scripts(env: &LoaderEnv<'_>, name: &str) {
    let fns = precompiled::get(env.lua());
    if let Err(e) = fns.fire_onload.call::<()>(name) {
        eprintln!("[OnLoad] {} error: {}", name, e);
    }
    if let Err(e) = fns.fire_onshow.call::<()>(name) {
        eprintln!("[OnShow] {} error: {}", name, e);
    }
}
