//! Garrison landing-page query backed by a neutral host input.

use super::helpers::ensure_namespace;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const EMPTY_GARRISON_TYPE: u32 = 0;

#[derive(Clone, Debug)]
pub struct GarrisonLandingPage {
    /// Host-supplied type; no selection or lifecycle is modeled.
    pub garrison_type: Option<u32>,
}

impl Default for GarrisonLandingPage {
    fn default() -> Self {
        Self {
            // INFERRED: the neutral host input starts at the empty-path type.
            garrison_type: Some(EMPTY_GARRISON_TYPE),
        }
    }
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_Garrison")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetLandingPageGarrisonType",
        get_landing_page_garrison_type,
    )
}

fn get_landing_page_garrison_type(state: &mut LuaState) -> LuaResult<u32> {
    let garrison_type = borrow_state(state)?
        .garrison_landing_page
        .garrison_type
        .unwrap_or(EMPTY_GARRISON_TYPE);
    state.push(Val::Num(f64::from(garrison_type)));
    Ok(1)
}
