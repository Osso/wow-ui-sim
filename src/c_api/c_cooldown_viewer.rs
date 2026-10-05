//! `C_CooldownViewer` cooldown entries backed by explicit simulator state.
//!
//! Entries are keyed by cooldownID. No DB2 cooldown-set data is loaded, so the
//! table starts empty; callers (tests, future spec/talent models) populate it.

use super::helpers::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{borrow_state, create_table, table_set};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

/// One `CooldownViewerCooldown` (CooldownViewerDocumentation.lua).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CooldownViewerCooldown {
    pub cooldown_id: i32,
    pub spell_id: Option<i32>,
    pub spell_category_id: Option<i32>,
    pub override_spell_id: Option<i32>,
    pub override_tooltip_spell_id: Option<i32>,
    /// 1-based inventory slot (`luaIndex`).
    pub equip_slot: Option<i32>,
    /// 1-based buff slot (`luaIndex`).
    pub buff_slot: Option<i32>,
    pub linked_spell_ids: Vec<i32>,
    pub self_aura: bool,
    pub has_aura: bool,
    pub charges: bool,
    pub is_known: bool,
    pub is_invisible: bool,
    /// `Enum.CooldownSetSpellFlags` bitmask.
    pub flags: i32,
    /// `Enum.CooldownViewerCategory` value.
    pub category: i32,
    /// INFERRED host-declared alert capabilities, not derived from flags/categories.
    pub valid_alert_types: Vec<u8>,
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_CooldownViewer")?;
    #[cfg(feature = "retail-12-0-0")]
    table_set_rust_fn_static(
        state,
        namespace,
        "GetValidAlertTypes",
        get_valid_alert_types,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetCooldownViewerCategorySet",
        get_category_set,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetCooldownViewerCooldownInfo",
        get_cooldown_info,
    )
}

#[cfg(feature = "retail-12-0-0")]
fn get_valid_alert_types(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Num(id) = value else {
        return Err(runtime_error("cooldownID must be a number"));
    };
    if !id.is_finite() || id.fract() != 0.0 || id < i32::MIN as f64 || id > i32::MAX as f64 {
        return Err(runtime_error("cooldownID must be an i32 integer"));
    }
    let alerts = borrow_state(state)?
        .cooldown_viewer_cooldowns
        .get(&(id as i32))
        .map(|entry| entry.valid_alert_types.clone())
        .unwrap_or_default();
    let table = create_table(state);
    state.push(table);
    for (index, alert) in alerts.iter().enumerate() {
        set_table_array(state, table, index as i64 + 1, Val::Num(f64::from(*alert)));
    }
    Ok(1)
}

fn required_number(state: &LuaState, index: i32, name: &str) -> LuaResult<i32> {
    match stack_val(state, index) {
        Val::Num(value) if value.is_finite() => Ok(value as i32),
        _ => Err(runtime_error(format!("{name} must be a number"))),
    }
}

fn get_category_set(state: &mut LuaState) -> LuaResult<u32> {
    let category = required_number(state, 1, "category")?;
    let allow_unlearned = Option::<bool>::from_stack(state, 2)?.unwrap_or(false);
    // INFERRED: set order is ascending cooldownID; real order is DB2 set order.
    let ids: Vec<i32> = borrow_state(state)?
        .cooldown_viewer_cooldowns
        .values()
        .filter(|cd| cd.category == category && (cd.is_known || allow_unlearned))
        .map(|cd| cd.cooldown_id)
        .collect();
    let table = create_table(state);
    for (index, id) in ids.into_iter().enumerate() {
        set_table_array(state, table, index as i64 + 1, Val::Num(f64::from(id)));
    }
    state.push(table);
    Ok(1)
}

fn get_cooldown_info(state: &mut LuaState) -> LuaResult<u32> {
    let cooldown_id = required_number(state, 1, "cooldownID")?;
    let Some(cooldown) = borrow_state(state)?
        .cooldown_viewer_cooldowns
        .get(&cooldown_id)
        .cloned()
    else {
        return Ok(0);
    };
    let table = publish_cooldown(state, &cooldown);
    state.push(table);
    Ok(1)
}

fn publish_cooldown(state: &mut LuaState, cooldown: &CooldownViewerCooldown) -> Val {
    let table = create_table(state);
    state.push(table);
    let linked = create_table(state);
    for (index, id) in cooldown.linked_spell_ids.iter().enumerate() {
        set_table_array(state, linked, index as i64 + 1, Val::Num(f64::from(*id)));
    }
    table_set(state, table, "linkedSpellIDs", linked);
    for (name, value) in [
        ("cooldownID", Some(cooldown.cooldown_id)),
        ("spellID", cooldown.spell_id),
        ("spellCategoryID", cooldown.spell_category_id),
        ("overrideSpellID", cooldown.override_spell_id),
        ("overrideTooltipSpellID", cooldown.override_tooltip_spell_id),
        ("equipSlot", cooldown.equip_slot),
        ("buffSlot", cooldown.buff_slot),
        ("flags", Some(cooldown.flags)),
        ("category", Some(cooldown.category)),
    ] {
        if let Some(value) = value {
            table_set(state, table, name, Val::Num(f64::from(value)));
        }
    }
    for (name, value) in [
        ("selfAura", cooldown.self_aura),
        ("hasAura", cooldown.has_aura),
        ("charges", cooldown.charges),
        ("isKnown", cooldown.is_known),
        ("isInvisible", cooldown.is_invisible),
    ] {
        table_set(state, table, name, Val::Bool(value));
    }
    state.pop();
    table
}
