//! `C_CooldownViewer` cooldown entries backed by explicit simulator state.
//!
//! Entries are keyed by cooldownID. No DB2 cooldown-set data is loaded, so the
//! table starts empty; callers (tests, future spec/talent models) populate it.

use super::helpers::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{borrow_state, create_table, table_set};
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_api::methods::{borrow_state_mut, table_get, val_to_string};
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_bridge::TableBuilder;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use rilua::vm::table::Table;
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

/// Supplied `GroupBuffItem`; Retail omits `displaySpellID` from its records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupBuffItem {
    pub spell_id: i32,
    pub display_spell_id: Option<i32>,
    pub name: String,
    pub icon_id: u32,
    pub flags: i32,
    pub is_known: bool,
}

/// Explicit host availability, not a class/spec eligibility inference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CooldownViewerAvailability {
    pub is_available: bool,
    pub failure_reason: String,
}

impl Default for CooldownViewerAvailability {
    fn default() -> Self {
        Self {
            is_available: false,
            failure_reason: "Availability not supplied".to_owned(),
        }
    }
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_CooldownViewer")?;
    if cfg!(any(
        feature = "retail-12-0-0",
        feature = "client-wowforever"
    )) {
        table_set_rust_fn_static(
            state,
            namespace,
            "IsCooldownViewerAvailable",
            query_availability,
        )?;
    }
    // Retail and Forever CooldownViewerDocumentation.lua:43-50,149-159.
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        table_set_rust_fn_static(state, namespace, "GetGroupBuffItems", get_group_buff_items)?;
        register_group_buff_flags(state)?;
    }
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

fn get_group_buff_items(state: &mut LuaState) -> LuaResult<u32> {
    let entries = borrow_state(state)?.cooldown_viewer_group_buffs.clone();
    let array = create_table(state);
    for (index, entry) in entries.iter().enumerate() {
        let row = create_table(state);
        for (key, value) in [
            ("spellID", Some(entry.spell_id)),
            ("flags", Some(entry.flags)),
        ] {
            if let Some(value) = value {
                table_set(state, row, key, Val::Num(f64::from(value)));
            }
        }
        if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::WowForever {
            if let Some(display_spell_id) = entry.display_spell_id {
                table_set(
                    state,
                    row,
                    "displaySpellID",
                    Val::Num(f64::from(display_spell_id)),
                );
            }
        }
        table_set(state, row, "iconID", Val::Num(f64::from(entry.icon_id)));
        let name = crate::lua_api::methods::create_string(state, &entry.name);
        table_set(state, row, "name", name);
        table_set(state, row, "isKnown", Val::Bool(entry.is_known));
        set_table_array(state, array, index as i64 + 1, row);
    }
    state.push(array);
    Ok(1)
}

fn register_group_buff_flags(state: &mut LuaState) -> LuaResult<()> {
    let enums = ensure_namespace(state, "Enum")?;
    let flags = crate::c_api::ensure_namespace_table_field(state, enums, "GroupBuffItemFlags")?;
    crate::c_api::set_namespace_member(state, Val::Table(flags), "HideByDefault", Val::Num(1.0))
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetGroupBuffItems", set_group_buff_items)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_group_buff_items(state: &mut LuaState) -> LuaResult<u32> {
    let items = read_group_buff_items(state, stack_val(state, 1))?;
    borrow_state_mut(state)?.cooldown_viewer_group_buffs = items;
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn read_group_buff_items(state: &mut LuaState, value: Val) -> LuaResult<Vec<GroupBuffItem>> {
    let Val::Table(array_ref) = value else {
        return Err(runtime_error(
            "A_Admin.SetGroupBuffItems: expected an item array",
        ));
    };
    let length = state
        .gc
        .tables
        .get(array_ref)
        .ok_or_else(|| runtime_error("A_Admin.SetGroupBuffItems: expected an item array"))?
        .len(&state.gc.string_arena);
    (1..=length)
        .map(|index| {
            let value = state
                .gc
                .tables
                .get(array_ref)
                .map(|array| array.get_int(index as i64))
                .unwrap_or(Val::Nil);
            read_group_buff_item(state, value)
        })
        .collect()
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn read_group_buff_item(state: &mut LuaState, value: Val) -> LuaResult<GroupBuffItem> {
    let Val::Table(table) = value else {
        return Err(invalid_group_buff_field("item"));
    };
    let display_spell_id = optional_group_buff_integer(state, table, "displaySpellID")?;
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::WowForever
        && display_spell_id.is_none()
    {
        return Err(invalid_group_buff_field("displaySpellID"));
    }
    Ok(GroupBuffItem {
        spell_id: group_buff_integer(state, table, "spellID")?,
        display_spell_id,
        name: group_buff_string(state, table, "name")?,
        icon_id: group_buff_icon_id(state, table)?,
        flags: group_buff_integer(state, table, "flags")?,
        is_known: group_buff_bool(state, table, "isKnown")?,
    })
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn group_buff_integer(state: &mut LuaState, table: GcRef<Table>, key: &str) -> LuaResult<i32> {
    match table_get(state, Val::Table(table), key) {
        Val::Num(value)
            if value.is_finite()
                && value.fract() == 0.0
                && (i32::MIN as f64..=i32::MAX as f64).contains(&value) =>
        {
            Ok(value as i32)
        }
        _ => Err(invalid_group_buff_field(key)),
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn optional_group_buff_integer(
    state: &mut LuaState,
    table: GcRef<Table>,
    key: &str,
) -> LuaResult<Option<i32>> {
    match table_get(state, Val::Table(table), key) {
        Val::Nil => Ok(None),
        Val::Num(value)
            if value.is_finite()
                && value.fract() == 0.0
                && (i32::MIN as f64..=i32::MAX as f64).contains(&value) =>
        {
            Ok(Some(value as i32))
        }
        _ => Err(invalid_group_buff_field(key)),
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn group_buff_icon_id(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<u32> {
    match table_get(state, Val::Table(table), "iconID") {
        Val::Num(value)
            if value.is_finite()
                && value.fract() == 0.0
                && (0.0..=u32::MAX as f64).contains(&value) =>
        {
            Ok(value as u32)
        }
        _ => Err(invalid_group_buff_field("iconID")),
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn group_buff_string(state: &mut LuaState, table: GcRef<Table>, key: &str) -> LuaResult<String> {
    let value = table_get(state, Val::Table(table), key);
    val_to_string(state, value).ok_or_else(|| invalid_group_buff_field(key))
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn group_buff_bool(state: &mut LuaState, table: GcRef<Table>, key: &str) -> LuaResult<bool> {
    match table_get(state, Val::Table(table), key) {
        Val::Bool(value) => Ok(value),
        _ => Err(invalid_group_buff_field(key)),
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn invalid_group_buff_field(key: &str) -> rilua::LuaError {
    runtime_error(format!("A_Admin.SetGroupBuffItems: invalid {key}"))
}

fn query_availability(state: &mut LuaState) -> LuaResult<u32> {
    let availability = borrow_state(state)?.cooldown_viewer_availability.clone();
    state.push(Val::Bool(availability.is_available));
    let reason = crate::lua_api::methods::create_string(state, &availability.failure_reason);
    state.push(reason);
    Ok(2)
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
