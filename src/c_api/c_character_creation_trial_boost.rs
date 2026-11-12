//! `C_CharacterCreation.GetTrialBoostStartingLevel` and its Glue inputs.

use crate::c_api::helpers::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, table_set};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

use super::c_character_creation_trial_boost_data::RETAIL_CHARACTER_SERVICE_INFO_ROWS;

const DEFAULT_BOOST_TYPE_SELECTOR: i32 = 2;
const LEVEL_BOOST_SERVICE_TYPE: i32 = 1;
const GENERIC_SERVICE_TYPE: i32 = 0;
const ZERO_BOOST_TYPE: i32 = 0;
const NAMESPACE: &'static str = "C_CharacterCreation";
const GETTER: &'static str = "GetTrialBoostStartingLevel";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CharacterServiceInfoRow {
    pub service_type: i32,
    pub boost_type: i32,
    pub boost_level: i32,
}

#[derive(Clone, Debug)]
pub struct TrialBoostLevelState {
    selector: i32,
    rows: Vec<CharacterServiceInfoRow>,
}

impl Default for TrialBoostLevelState {
    fn default() -> Self {
        let rows = if ACTIVE == ClientProfile::Retail {
            RETAIL_CHARACTER_SERVICE_INFO_ROWS
                .iter()
                .map(
                    |&(service_type, boost_type, boost_level)| CharacterServiceInfoRow {
                        service_type,
                        boost_type,
                        boost_level,
                    },
                )
                .collect()
        } else {
            Vec::new()
        };

        Self {
            selector: DEFAULT_BOOST_TYPE_SELECTOR,
            rows,
        }
    }
}

impl TrialBoostLevelState {
    /// Store the selector supplied by the Glue feature-system-status receiver.
    /// This setter always replaces the value and does not dispatch an event.
    pub fn set_selector(&mut self, selector: i32) {
        self.selector = selector;
    }

    /// Replace host-supplied rows without changing their iteration order.
    pub fn replace_rows(&mut self, rows: Vec<CharacterServiceInfoRow>) {
        self.rows = rows;
    }

    pub fn character_service_info_rows(&self) -> &[CharacterServiceInfoRow] {
        &self.rows
    }

    pub(super) fn starting_level(&self) -> i32 {
        self.rows
            .iter()
            .find(|row| {
                let is_matching_level_boost =
                    row.service_type == LEVEL_BOOST_SERVICE_TYPE && row.boost_type == self.selector;
                let is_matching_zero_service = self.selector == ZERO_BOOST_TYPE
                    && row.service_type == GENERIC_SERVICE_TYPE
                    && row.boost_type == ZERO_BOOST_TYPE;
                is_matching_level_boost || is_matching_zero_service
            })
            .map_or(0, |row| row.boost_level)
    }
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    update_screen_registration(state)
}

pub(crate) fn update_screen_registration(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        return Ok(());
    }

    let is_glue = borrow_state(state)?.screen_kind.is_glue();
    let namespace = ensure_namespace(state, NAMESPACE)?;
    if is_glue {
        table_set_rust_fn_static(state, namespace, GETTER, get_trial_boost_starting_level)?;
    } else {
        table_set(state, Val::Table(namespace), GETTER, Val::Nil);
    }
    Ok(())
}

fn get_trial_boost_starting_level(state: &mut LuaState) -> LuaResult<u32> {
    let level = borrow_state(state)?.trial_boost_level.starting_level();
    state.push(Val::Num(f64::from(level)));
    Ok(1)
}
