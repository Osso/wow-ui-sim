//! R63 field publication; R112 empty host catalog/text-map simulator policy.
//! Native fresh rows/order and text validation/unknown-ID result await R65.
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct CreditsRow {
    pub is_default: bool,
    pub credits_file_id: u32,
    pub key_art: String,
    pub background: String,
    pub name: String,
    pub logo: u32,
    pub sound_kit: Option<u32>,
}
#[derive(Clone, Debug, Default)]
pub struct CreditsState {
    /// Explicit host order, not a reconstructed native row order/fresh catalog.
    pub rows: Vec<CreditsRow>,
    pub texts: HashMap<u32, String>,
}
pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(ACTIVE, ClientProfile::Ptr | ClientProfile::WowForever) {
        return Ok(());
    }
    table_set_rust_fn_static(state, state.global, "GetCreditsDisplayInfo", display)?;
    if borrow_state(state)?.screen_kind.is_glue() {
        table_set_rust_fn_static(state, state.global, "GetCreditsTextByFileDataID", text)?;
    } else {
        table_set(
            state,
            Val::Table(state.global),
            "GetCreditsTextByFileDataID",
            Val::Nil,
        );
    }
    Ok(())
}
fn display(state: &mut LuaState) -> LuaResult<u32> {
    let rows = borrow_state(state)?.credits.rows.clone();
    let result = create_table(state);
    let Val::Table(result_ref) = result else {
        unreachable!()
    };
    for (index, row) in rows.iter().enumerate() {
        let entry = create_table(state);
        table_set(state, entry, "isDefault", Val::Bool(row.is_default));
        table_set(
            state,
            entry,
            "creditsFileID",
            Val::Num(f64::from(row.credits_file_id)),
        );
        for (key, value) in [
            ("keyArt", &row.key_art),
            ("background", &row.background),
            ("name", &row.name),
        ] {
            let value = create_string(state, value);
            table_set(state, entry, key, value);
        }
        if row.logo != 0 {
            table_set(state, entry, "logo", Val::Num(f64::from(row.logo)));
        }
        if let Some(sound) = row.sound_kit {
            table_set(state, entry, "soundKit", Val::Num(f64::from(sound)));
        }
        if let Some(table) = state.gc.tables.get_mut(result_ref) {
            let _ = table.raw_set(Val::Num((index + 1) as f64), entry, &state.gc.string_arena);
        }
        state.gc.barrier_back(result_ref);
    }
    state.push(result);
    Ok(1)
}
fn text(state: &mut LuaState) -> LuaResult<u32> {
    // Bounded host-map input, NOT native validation or conversion semantics.
    let Val::Num(id) = stack_val(state, 1) else {
        return Err(rilua::runtime_error(
            "GetCreditsTextByFileDataID: argument outside simulator coverage (native validation pending R65)",
        ));
    };
    if !id.is_finite() || id.fract() != 0.0 || id < 0.0 || id > f64::from(u32::MAX) {
        return Err(rilua::runtime_error(
            "GetCreditsTextByFileDataID: argument outside simulator coverage (native validation pending R65)",
        ));
    }
    let text = borrow_state(state)?
        .credits
        .texts
        .get(&(id as u32))
        .cloned();
    if let Some(text) = text {
        let value = create_string(state, &text);
        state.push(value);
        Ok(1)
    } else {
        Ok(0)
    } // R112 neutral host-policy result, not native unknown-ID evidence.
}
