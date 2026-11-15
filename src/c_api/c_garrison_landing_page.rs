//! Garrison landing-page query backed by a neutral host input.

use super::helpers::ensure_namespace;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const EMPTY_GARRISON_TYPE: u32 = 0;
const COVENANT_GARRISON_TYPE: u8 = 111;
const GARRISON_TYPE_KEY_MODULUS: f64 = 256.0;

#[derive(Clone, Debug)]
pub struct GarrisonLandingPage {
    /// Host-supplied type; no selection or lifecycle is modeled.
    pub garrison_type: Option<u32>,
    /// Host-supplied flags keyed by the low byte of the garrison type.
    pub minimap_button_records: std::collections::HashMap<u8, bool>,
    /// INFERRED host input for PlayerCondition 105386; no evaluator is modeled.
    pub player_condition_105386: bool,
    /// INFERRED to be the covenant ID; the native predicate checks for nonzero.
    pub player_state_value: u32,
}

impl Default for GarrisonLandingPage {
    fn default() -> Self {
        Self {
            // INFERRED: the neutral host input starts at the empty-path type.
            garrison_type: Some(EMPTY_GARRISON_TYPE),
            minimap_button_records: std::collections::HashMap::new(),
            // INFERRED: 105386 is inactive until the host supplies its result.
            player_condition_105386: false,
            // INFERRED: the neutral player-state input is zero.
            player_state_value: 0,
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
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsLandingPageMinimapButtonVisible",
        is_landing_page_minimap_button_visible,
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

fn is_landing_page_minimap_button_visible(state: &mut LuaState) -> LuaResult<u32> {
    let garrison_type = match stack_val(state, 1) {
        Val::Num(number) if number.is_finite() => number,
        _ => {
            return Err(runtime_error(
                "C_Garrison.IsLandingPageMinimapButtonVisible expects a finite number",
            ));
        }
    };
    let record_key = garrison_type.trunc().rem_euclid(GARRISON_TYPE_KEY_MODULUS) as u8;

    let sim_state = borrow_state(state)?;
    let landing_page = &sim_state.garrison_landing_page;
    let record_flag = landing_page
        .minimap_button_records
        .get(&record_key)
        .copied();
    let is_visible = match (record_key, record_flag) {
        (_, None) => false,
        (COVENANT_GARRISON_TYPE, Some(_)) => {
            landing_page.player_condition_105386 && landing_page.player_state_value != 0
        }
        (_, Some(state_flag)) => state_flag,
    };
    drop(sim_state);

    state.push(Val::Bool(is_visible));
    Ok(1)
}
