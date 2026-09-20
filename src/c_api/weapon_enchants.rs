//! Weapon enchant snapshots shared by C_Item and the legacy temporary-enchant query.

use crate::lua_api::methods::{borrow_state, create_table, table_set, table_set_num};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::api::LuaApiMut;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// ItemEnchantType.Temporary in the client API.
pub const TEMPORARY_ENCHANT_TYPE: u32 = 2;

/// Current server-provided enchant data; time remaining is in milliseconds.
#[derive(Clone, Debug)]
pub struct WeaponEnchant {
    pub enchant_type: u32,
    pub time_left: f64,
    pub charges: u32,
    pub enchant_id: u32,
    pub icon_id: u32,
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    LuaApiMut::register_function(state, "GetWeaponEnchantInfo", legacy_info)?;
    if cfg!(feature = "client-wowforever") {
        let table = super::helpers::ensure_namespace(state, "C_Item")?;
        table_set_rust_fn_static(state, table, "GetWeaponEnchantInfo", weapon_info)?;
    }
    Ok(())
}

fn weapon_info(state: &mut LuaState) -> LuaResult<u32> {
    let slot = validate_weapon_slot(stack_val(state, 1))?;
    let enchants = borrow_state(state)?.weapon_enchants[slot].clone();
    let result = create_table(state);
    let Val::Table(array) = result else {
        unreachable!()
    };
    for (index, enchant) in enchants.iter().enumerate() {
        let entry = serialize_enchant(state, enchant);
        table_set_num(state, array, (index + 1) as f64, entry);
    }
    state.push(result);
    Ok(1)
}

fn validate_weapon_slot(value: Val) -> LuaResult<usize> {
    match value {
        Val::Num(0.0) => Ok(0),
        Val::Num(1.0) => Ok(1),
        Val::Num(2.0) => Ok(2),
        _ => Err(rilua::runtime_error(
            "weaponSlot must be MainHand (0), OffHand (1), or Ranged (2)",
        )),
    }
}

fn serialize_enchant(state: &mut LuaState, enchant: &WeaponEnchant) -> Val {
    let entry = create_table(state);
    table_set(state, entry, "hasEnchant", Val::Bool(true));
    for (key, value) in [
        ("enchantType", enchant.enchant_type as f64),
        ("timeLeft", enchant.time_left),
        ("charges", enchant.charges as f64),
        ("enchantID", enchant.enchant_id as f64),
        ("enchantIconID", enchant.icon_id as f64),
    ] {
        table_set(state, entry, key, Val::Num(value));
    }
    entry
}

fn legacy_info(state: &mut LuaState) -> LuaResult<u32> {
    let enchants = borrow_state(state)?.weapon_enchants.clone();
    // The legacy contract exposes main-hand and off-hand only, never ranged.
    for slot in [&enchants[0], &enchants[1]] {
        let temporary = slot
            .iter()
            .find(|enchant| enchant.enchant_type == TEMPORARY_ENCHANT_TYPE);
        state.push(Val::Bool(temporary.is_some()));
        for value in temporary.map_or([0.0; 3], |enchant| {
            [
                enchant.time_left,
                enchant.charges as f64,
                enchant.enchant_id as f64,
            ]
        }) {
            state.push(Val::Num(value));
        }
    }
    Ok(8)
}
