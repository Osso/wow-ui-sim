//! Player-related API functions.
//!
//! This module provides WoW API functions related to:
//! - BattleNet features (BNFeaturesEnabled, BNConnected, BNGetFriendInfo, etc.)
//! - Specialization info (GetSpecialization, GetSpecializationInfo, etc.)
//! - Action bar functions (HasAction, GetActionInfo, GetActionTexture, etc.)

use crate::lua_api::SimState;
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Register all player-related API functions to the Lua globals table.
pub fn register_player_api(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    register_battlenet_functions(lua)?;
    register_specialization_functions(lua)?;
    register_action_bar_functions(lua, state)?;
    register_timerunning_functions(lua)?;
    register_economy_functions(lua)?;
    register_instance_functions(lua)?;
    register_character_functions(lua)?;
    register_cinematic_functions(lua)?;
    register_unit_functions(lua)?;
    Ok(())
}

fn register_timerunning_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set("PlayerIsTimerunning", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsPlayerAtEffectiveMaxLevel", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsXPUserDisabled", lua.create_function(|_, ()| Ok(false))?)?;
    Ok(())
}

/// Register BattleNet social functions.
fn register_battlenet_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("BNFeaturesEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("BNFeaturesEnabledAndConnected", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("BNConnected", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("BNGetFriendInfo", lua.create_function(|_, _index: i32| Ok(Value::Nil))?)?;
    globals.set("BNGetNumFriends", lua.create_function(|_, ()| Ok((0, 0)))?)?; // online, total
    globals.set("BNGetInfo", lua.create_function(|lua, ()| {
        // Return: presenceID, battleTag, toonID, currentBroadcast, bnetAFK, bnetDND, isRIDEnabled
        Ok((
            Value::Integer(0),
            Value::String(lua.create_string("SimPlayer#0000")?),
            Value::Nil,
            Value::String(lua.create_string("")?),
            Value::Boolean(false),
            Value::Boolean(false),
            Value::Boolean(false),
        ))
    })?)?;

    Ok(())
}

/// Register specialization query functions.
fn register_specialization_functions(lua: &Lua) -> Result<()> {
    register_spec_basic_queries(lua)?;
    register_spec_info_lookups(lua)?;
    Ok(())
}

/// Basic spec queries: GetSpecialization, GetSpecializationInfo, GetNumSpecializations.
fn register_spec_basic_queries(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("GetSpecialization", lua.create_function(|_, ()| Ok(1))?)?;
    globals.set("GetSpecializationInfo", lua.create_function(|lua, spec_index: i32| {
        let (id, name, role) = match spec_index {
            1 => (62, "Arcane", "DAMAGER"),
            2 => (63, "Fire", "DAMAGER"),
            3 => (64, "Frost", "DAMAGER"),
            _ => (62, "Arcane", "DAMAGER"),
        };
        Ok(mlua::MultiValue::from_vec(vec![
            Value::Integer(id),
            Value::String(lua.create_string(name)?),
            Value::String(lua.create_string("Spec description")?),
            Value::Integer(136116),
            Value::String(lua.create_string(role)?),
            Value::Integer(4),
        ]))
    })?)?;
    globals.set("GetNumSpecializations", lua.create_function(|_, ()| Ok(4))?)?;
    globals.set("GetSpecializationRole", lua.create_function(|lua, _spec_index: Option<i32>| {
        Ok(Value::String(lua.create_string("DAMAGER")?))
    })?)?;
    globals.set("GetSpecializationRoleByID", lua.create_function(|lua, _spec_id: i32| {
        Ok(Value::String(lua.create_string("DAMAGER")?))
    })?)?;
    globals.set(
        "GetNumSpecializationsForClassID",
        lua.create_function(|_, (_class_id, _sex): (Option<i32>, Option<i32>)| {
            Ok(_class_id.map_or(0, |_| 3i32))
        })?,
    )?;

    Ok(())
}

/// Spec info lookups by ID or class: GetSpecializationInfoByID, ForSpecID, ForClassID.
fn register_spec_info_lookups(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("GetSpecializationInfoByID", lua.create_function(|lua, _spec_id: i32| {
        Ok(mlua::MultiValue::from_vec(vec![
            Value::Integer(62),
            Value::String(lua.create_string("Arcane")?),
            Value::String(lua.create_string("Spec description")?),
            Value::Integer(136116),
            Value::String(lua.create_string("DAMAGER")?),
            Value::String(lua.create_string("MAGE")?),
        ]))
    })?)?;
    globals.set("GetSpecializationInfoForSpecID", lua.create_function(|lua, _spec_id: i32| {
        Ok(mlua::MultiValue::from_vec(vec![
            Value::Integer(62),
            Value::String(lua.create_string("Arcane")?),
            Value::String(lua.create_string("Spec description")?),
            Value::Integer(136116),
            Value::String(lua.create_string("DAMAGER")?),
            Value::String(lua.create_string("MAGE")?),
        ]))
    })?)?;
    globals.set("GetSpecializationInfoForClassID", lua.create_function(|lua, (_class_id, spec_index): (i32, i32)| {
        if !(1..=4).contains(&spec_index) {
            return Ok(mlua::MultiValue::new());
        }
        Ok(mlua::MultiValue::from_vec(vec![
            Value::Integer(62i64 + spec_index as i64 - 1),
            Value::String(lua.create_string("Spec")?),
            Value::String(lua.create_string("Description")?),
            Value::Integer(136116),
            Value::String(lua.create_string("DAMAGER")?),
            Value::Boolean(false),
            Value::Boolean(true),
        ]))
    })?)?;

    Ok(())
}

/// Extract a slot number from a Lua Value (integer or number).
fn slot_from_value(v: &Value) -> Option<u32> {
    match v {
        Value::Integer(n) => Some(*n as u32),
        Value::Number(n) => Some(*n as u32),
        _ => None,
    }
}

/// Look up the texture path for an action bar slot.
fn action_texture_path(state: &SimState, slot: u32) -> Option<String> {
    let spell_id = state.action_bars.get(&slot)?;
    let spell = crate::spells::get_spell(*spell_id)?;
    let path = crate::manifest_interface_data::get_texture_path(spell.icon_file_data_id)?;
    Some(format!("Interface\\{}", path.replace('/', "\\")))
}

/// Register action bar query functions backed by SimState.
fn register_action_bar_functions(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    register_action_bar_stateful(lua, state)?;
    register_action_bar_stubs(lua)?;
    Ok(())
}

/// Stateful action bar functions that query SimState.
fn register_action_bar_stateful(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();

    let st = Rc::clone(&state);
    globals.set("HasAction", lua.create_function(move |_, slot: Value| {
        let s = st.borrow();
        Ok(slot_from_value(&slot).is_some_and(|n| s.action_bars.contains_key(&n)))
    })?)?;

    let st = Rc::clone(&state);
    globals.set("GetActionInfo", lua.create_function(move |lua, slot: Value| {
        let s = st.borrow();
        let Some(n) = slot_from_value(&slot) else { return Ok(mlua::MultiValue::new()) };
        let Some(&spell_id) = s.action_bars.get(&n) else { return Ok(mlua::MultiValue::new()) };
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string("spell")?),
            Value::Integer(spell_id as i64),
            Value::String(lua.create_string("spell")?),
        ]))
    })?)?;

    let st = Rc::clone(&state);
    globals.set("GetActionTexture", lua.create_function(move |lua, slot: Value| {
        let s = st.borrow();
        let Some(n) = slot_from_value(&slot) else { return Ok(Value::Nil) };
        match action_texture_path(&s, n) {
            Some(path) => Ok(Value::String(lua.create_string(&path)?)),
            None => Ok(Value::Nil),
        }
    })?)?;

    let st = Rc::clone(&state);
    globals.set("IsUsableAction", lua.create_function(move |_, slot: Value| {
        let s = st.borrow();
        let has = slot_from_value(&slot).is_some_and(|n| s.action_bars.contains_key(&n));
        Ok((has, false))
    })?)?;

    Ok(())
}

/// Stateless action bar stub functions.
fn register_action_bar_stubs(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set("GetActionText", lua.create_function(|_, _slot: Value| Ok(Value::Nil))?)?;
    globals.set("GetActionCount", lua.create_function(|_, _slot: Value| Ok(0))?)?;
    globals.set("GetActionCooldown", lua.create_function(|_, _slot: Value| {
        Ok((0.0_f64, 0.0_f64, 1, 1.0_f64))
    })?)?;
    globals.set("IsConsumableAction", lua.create_function(|_, _slot: Value| Ok(false))?)?;
    globals.set("IsStackableAction", lua.create_function(|_, _slot: Value| Ok(false))?)?;
    globals.set("IsAttackAction", lua.create_function(|_, _slot: Value| Ok(false))?)?;
    globals.set("IsAutoRepeatAction", lua.create_function(|_, _slot: Value| Ok(false))?)?;
    globals.set("IsCurrentAction", lua.create_function(|_, _slot: Value| Ok(false))?)?;
    globals.set("GetActionCharges", lua.create_function(|_, _slot: Value| {
        Ok((0, 0, 0.0_f64, 0.0_f64, 1.0_f64))
    })?)?;
    globals.set("GetPossessInfo", lua.create_function(|_, _index: Value| Ok(Value::Nil))?)?;
    globals.set("GetActionBarPage", lua.create_function(|_, ()| Ok(1))?)?;
    globals.set("GetBonusBarOffset", lua.create_function(|_, ()| Ok(0))?)?;
    globals.set("GetOverrideBarIndex", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    globals.set("GetVehicleBarIndex", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    globals.set("GetTempShapeshiftBarIndex", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    globals.set("IsPossessBarVisible", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("SetActionUIButton", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    Ok(())
}

/// Economy functions: money, trade, buyback.
fn register_economy_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set("GetMoney", lua.create_function(|_, ()| Ok(0i64))?)?;
    globals.set("GetTargetTradeMoney", lua.create_function(|_, ()| Ok(0i64))?)?;
    globals.set("GetNumBuybackItems", lua.create_function(|_, ()| Ok(0i32))?)?;
    Ok(())
}

/// Instance/dungeon info functions.
fn register_instance_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    // GetInstanceInfo() -> name, instanceType, difficultyID, difficultyName,
    //   maxPlayers, dynamicDifficulty, isDynamic, instanceID, instanceGroupSize, LfgDungeonID
    globals.set("GetInstanceInfo", lua.create_function(|lua, ()| {
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string("")?),     // name
            Value::String(lua.create_string("none")?),  // instanceType
            Value::Integer(0),                          // difficultyID
            Value::String(lua.create_string("")?),      // difficultyName
            Value::Integer(0),                          // maxPlayers
            Value::Integer(0),                          // dynamicDifficulty
            Value::Boolean(false),                      // isDynamic
            Value::Integer(0),                          // instanceID
            Value::Integer(0),                          // instanceGroupSize
            Value::Integer(0),                          // LfgDungeonID
        ]))
    })?)?;
    Ok(())
}

/// Character info functions: titles, item level, RPE state, inventory.
fn register_character_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set("GetCurrentTitle", lua.create_function(|_, ()| Ok(0i32))?)?;
    globals.set("GetAverageItemLevel", lua.create_function(|_, ()| Ok((0.0_f64, 0.0_f64, 0.0_f64)))?)?;
    globals.set("IsPlayerInRPE", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("GetInventoryItemQuality", lua.create_function(|_, _args: mlua::MultiValue| Ok(Value::Nil))?)?;
    globals.set("GetInventoryItemTexture", lua.create_function(|_, _args: mlua::MultiValue| Ok(Value::Nil))?)?;
    globals.set("GetInventoryItemID", lua.create_function(|_, _args: mlua::MultiValue| Ok(Value::Nil))?)?;
    globals.set("GetInventoryItemLink", lua.create_function(|_, _args: mlua::MultiValue| Ok(Value::Nil))?)?;
    globals.set("GetInventoryItemCount", lua.create_function(|_, _args: mlua::MultiValue| Ok(0i32))?)?;
    globals.set("GetSpecializationRoleEnum", lua.create_function(|_, ()| Ok(0i32))?)?;
    globals.set("GetPlayerTradeMoney", lua.create_function(|_, ()| Ok(0i64))?)?;
    globals.set("IsInventoryItemLocked", lua.create_function(|_, _args: mlua::MultiValue| Ok(false))?)?;
    globals.set("GetRestrictedAccountData", lua.create_function(|_, ()| Ok((false, false, false)))?)?;
    globals.set("UnitStat", lua.create_function(|_, _args: mlua::MultiValue| Ok((0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64)))?)?;
    globals.set("MerchantFrame_UpdateGuildBankRepair", lua.create_function(|_, ()| Ok(()))?)?;
    globals.set("MerchantFrame_UpdateCanRepairAll", lua.create_function(|_, ()| Ok(()))?)?;
    globals.set("SetItemButtonDesaturated", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    globals.set("EquipmentFlyout_UpdateFlyout", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    globals.set("EquipmentFlyout_SetTooltipAnchor", lua.create_function(|_, _args: mlua::MultiValue| Ok(false))?)?;
    globals.set("GameTooltip_SuppressAutomaticCompareItem", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    Ok(())
}

/// Cinematic/cutscene control stubs.
fn register_cinematic_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set("MouseOverrideCinematicDisable", lua.create_function(|_, ()| Ok(()))?)?;
    globals.set("MouseOverrideCinematicEnable", lua.create_function(|_, ()| Ok(()))?)?;
    globals.set("GetCursorMoney", lua.create_function(|_, ()| Ok(0i64))?)?;
    globals.set("GetNumTitles", lua.create_function(|_, ()| Ok(0i32))?)?;
    // GetItemLevelColor(itemLevel) -> r, g, b
    globals.set("GetItemLevelColor", lua.create_function(|_, _ilvl: Value| {
        Ok((1.0_f64, 1.0_f64, 1.0_f64))
    })?)?;
    // GetDifficultyInfo(id) -> name, groupType, isHeroic, isChallengeMode, toggleDifficultyID
    globals.set("GetDifficultyInfo", lua.create_function(|lua, _id: Value| {
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string("")?),
            Value::String(lua.create_string("")?),
            Value::Boolean(false),
            Value::Boolean(false),
            Value::Integer(0),
        ]))
    })?)?;
    // BreakUpLargeNumbers(amount) -> formatted string
    globals.set("BreakUpLargeNumbers", lua.create_function(|_, amount: Value| {
        let s = match amount {
            Value::Integer(n) => n.to_string(),
            Value::Number(n) => format!("{:.0}", n),
            _ => "0".to_string(),
        };
        Ok(s)
    })?)?;
    Ok(())
}

/// Unit query functions used by UnitFrame/PetFrame code.
fn register_unit_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set("PetUsesPetFrame", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("UnitIsPossessed", lua.create_function(|_, _unit: Value| Ok(false))?)?;
    globals.set("GetNumShapeshiftForms", lua.create_function(|_, ()| Ok(0i32))?)?;
    Ok(())
}
