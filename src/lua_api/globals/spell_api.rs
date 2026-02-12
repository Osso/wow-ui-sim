//! Spell-related C_* namespaces.
//!
//! - `C_SpellBook` - Spell book functions (backed by spellbook_data)
//! - `C_Spell` - Spell information functions
//! - `C_Traits` - Talent/loadout system (in traits_api.rs)

use super::spellbook_data;
use crate::lua_api::SimState;
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

pub fn register_spell_api(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();

    globals.set("C_SpellBook", register_c_spell_book(lua, Rc::clone(&state))?)?;
    globals.set("C_Spell", register_c_spell(lua, Rc::clone(&state))?)?;
    globals.set("C_Traits", super::traits_api::register_c_traits(lua, Rc::clone(&state))?)?;
    register_cast_globals(lua, state)?;

    Ok(())
}

/// C_SpellBook namespace - spell book functions backed by paladin spellbook data.
fn register_c_spell_book(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    register_c_spell_book_item_methods(&t, lua)?;
    register_c_spell_book_queries(&t, lua)?;
    register_c_spell_book_actions(&t, lua, state)?;
    Ok(t)
}

/// Item-level C_SpellBook methods (info, name, type, texture, cooldown, etc.).
fn register_c_spell_book_item_methods(t: &mlua::Table, lua: &Lua) -> Result<()> {
    t.set(
        "GetNumSpellBookSkillLines",
        lua.create_function(|_, ()| Ok(spellbook_data::num_skill_lines()))?,
    )?;
    t.set("GetSpellBookSkillLineInfo", lua.create_function(create_skill_line_info)?)?;
    t.set("GetSpellBookItemInfo", lua.create_function(create_spell_book_item_info)?)?;
    t.set("GetSpellBookItemName", lua.create_function(create_spell_book_item_name)?)?;
    t.set("GetSpellBookItemType", lua.create_function(create_spell_book_item_type)?)?;
    t.set(
        "GetSpellBookItemLevelLearned",
        lua.create_function(|_, (_slot, _bank): (i32, Option<i32>)| {
            Ok(1i32) // All spells learned at level 1 for now
        })?,
    )?;
    t.set(
        "IsSpellBookItemPassive",
        lua.create_function(|_, (slot, _bank): (i32, Option<i32>)| {
            Ok(spellbook_data::get_spell_at_slot(slot)
                .is_some_and(|(_, entry, _)| entry.is_passive))
        })?,
    )?;
    t.set("GetSpellBookItemCooldown", lua.create_function(create_spell_book_item_cooldown)?)?;
    t.set("GetSpellBookItemPowerCost", lua.create_function(|lua, (slot, _bank): (i32, Option<i32>)| {
        let Some((_, entry, _)) = spellbook_data::get_spell_at_slot(slot) else {
            return Ok(Value::Nil);
        };
        create_spell_power_cost(lua, entry.spell_id as i32)
    })?)?;
    t.set(
        "GetSpellBookItemAutoCast",
        lua.create_function(|_, (_slot, _bank): (i32, Option<i32>)| {
            Ok((false, false)) // autoCastAllowed, autoCastEnabled
        })?,
    )?;
    t.set(
        "GetSpellBookItemTexture",
        lua.create_function(|_, (slot, _bank): (i32, Option<i32>)| {
            let Some((_, entry, _)) = spellbook_data::get_spell_at_slot(slot) else {
                return Ok(Value::Nil);
            };
            let file_id = crate::spells::get_spell(entry.spell_id)
                .map(|s| s.icon_file_data_id)
                .unwrap_or(136243);
            Ok(Value::Integer(file_id as i64))
        })?,
    )?;
    Ok(())
}

/// Spell knowledge and lookup queries for C_SpellBook.
fn register_c_spell_book_queries(t: &mlua::Table, lua: &Lua) -> Result<()> {
    t.set("HasPetSpells", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set(
        "GetOverrideSpell",
        lua.create_function(|_, spell_id: i32| Ok(spell_id))?,
    )?;
    t.set(
        "IsSpellKnown",
        lua.create_function(|_, (spell_id, _bank): (i32, Option<i32>)| {
            Ok(spellbook_data::is_spell_known(spell_id as u32))
        })?,
    )?;
    t.set(
        "IsSpellInSpellBook",
        lua.create_function(
            |_, (spell_id, _bank, _include_overrides): (i32, Option<i32>, Option<bool>)| {
                Ok(spellbook_data::find_spell_slot(spell_id as u32).is_some())
            },
        )?,
    )?;
    t.set(
        "FindSpellBookSlotForSpell",
        lua.create_function(|_, (spell_id, _bank): (i32, Option<i32>)| {
            match spellbook_data::find_spell_slot(spell_id as u32) {
                Some((slot, bank)) => Ok(mlua::MultiValue::from_vec(vec![
                    Value::Integer(slot as i64),
                    Value::Integer(bank as i64),
                ])),
                None => Ok(mlua::MultiValue::new()),
            }
        })?,
    )?;
    Ok(())
}

/// CastSpellBookItem, PickupSpellBookItem, ToggleSpellBookItemAutoCast.
fn register_c_spell_book_actions(
    t: &mlua::Table,
    lua: &Lua,
    state: Rc<RefCell<SimState>>,
) -> Result<()> {
    let st = Rc::clone(&state);
    t.set(
        "CastSpellBookItem",
        lua.create_function(move |lua, (slot, _bank, _self_cast): (i32, Option<i32>, Option<bool>)| {
            let spell_id = spellbook_data::get_spell_at_slot(slot)
                .map(|(_, entry, _)| entry.spell_id);
            let Some(spell_id) = spell_id else { return Ok(()) };
            if st.borrow().casting.is_some() { return Ok(()) }
            cast_spell_by_id(&st, lua, spell_id)
        })?,
    )?;
    t.set(
        "PickupSpellBookItem",
        lua.create_function(|_, (_slot, _bank): (i32, Option<i32>)| Ok(()))?,
    )?;
    t.set(
        "ToggleSpellBookItemAutoCast",
        lua.create_function(|_, (_slot, _bank): (i32, Option<i32>)| Ok(()))?,
    )?;
    Ok(())
}

/// Shared cast logic: validate target, resolve cast time, start cast or apply instant.
fn cast_spell_by_id(
    state: &Rc<RefCell<SimState>>,
    lua: &Lua,
    spell_id: u32,
) -> mlua::Result<()> {
    use super::action_bar_api::{apply_instant_spell, start_cast, start_cooldowns};
    use crate::lua_api::game_data::validate_spell_target;

    // Validate target compatibility (borrow must be dropped before fire_ui_error
    // because the event handler chain borrows state again for AddMessage).
    let blocked_msg = {
        let s = state.borrow();
        validate_spell_target(spell_id, s.current_target.as_ref()).err()
    };
    if let Some(msg) = blocked_msg {
        eprintln!("[cast] Blocked: {} (spell {})", msg, spell_id);
        fire_ui_error(lua, msg)?;
        return Ok(());
    }

    let cast_time_ms = spell_cast_time(spell_id as i32);
    if cast_time_ms > 0 {
        start_cast(state, lua, spell_id, cast_time_ms)?;
    } else {
        apply_instant_spell(state, lua, spell_id)?;
    }
    start_cooldowns(state, lua, spell_id)?;
    Ok(())
}

/// Fire UI_ERROR_MESSAGE so the error frame shows the red text.
pub fn fire_ui_error(lua: &Lua, message: &str) -> mlua::Result<()> {
    let fire: mlua::Function = lua.globals().get("FireEvent")?;
    fire.call::<()>((
        lua.create_string("UI_ERROR_MESSAGE")?,
        1i32, // messageType (1 = generic)
        lua.create_string(message)?,
    ))
}

/// CastSpellByID / CastSpellByName globals (used by SecureTemplates SECURE_ACTIONS["spell"]).
fn register_cast_globals(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let g = lua.globals();

    let st = Rc::clone(&state);
    g.set(
        "CastSpellByID",
        lua.create_function(move |lua, (spell_id, _unit): (i32, Option<String>)| {
            if spell_id <= 0 { return Ok(()) }
            if st.borrow().casting.is_some() { return Ok(()) }
            cast_spell_by_id(&st, lua, spell_id as u32)
        })?,
    )?;

    g.set(
        "CastSpellByName",
        lua.create_function(move |lua, (name, _unit): (String, Option<String>)| {
            let spell_id = spellbook_data::find_spell_by_name(&name);
            let Some(spell_id) = spell_id else { return Ok(()) };
            if state.borrow().casting.is_some() { return Ok(()) }
            cast_spell_by_id(&state, lua, spell_id)
        })?,
    )?;

    Ok(())
}

fn create_spell_book_item_name(
    lua: &Lua,
    (slot, _bank): (i32, Option<i32>),
) -> Result<mlua::MultiValue> {
    let Some((_, entry, _)) = spellbook_data::get_spell_at_slot(slot) else {
        return Ok(mlua::MultiValue::new());
    };
    let spell = crate::spells::get_spell(entry.spell_id);
    let name = spell.map(|s| s.name).unwrap_or("Unknown");
    let subtext = spell.map(|s| s.subtext).unwrap_or("");
    Ok(mlua::MultiValue::from_vec(vec![
        Value::String(lua.create_string(name)?),
        Value::String(lua.create_string(subtext)?),
    ]))
}

fn create_spell_book_item_type(
    _: &Lua,
    (slot, _bank): (i32, Option<i32>),
) -> Result<mlua::MultiValue> {
    let Some((_, entry, skill_line)) = spellbook_data::get_spell_at_slot(slot) else {
        return Ok(mlua::MultiValue::new());
    };
    // itemType: 1=Spell, 2=FutureSpell (off-spec unlearned)
    let item_type = if skill_line.off_spec_id.is_some() {
        2
    } else {
        1
    };
    Ok(mlua::MultiValue::from_vec(vec![
        Value::Integer(item_type),
        Value::Integer(entry.spell_id as i64),
    ]))
}

fn create_spell_book_item_cooldown(lua: &Lua, (_slot, _bank): (i32, Option<i32>)) -> Result<Value> {
    let info = lua.create_table()?;
    info.set("startTime", 0.0)?;
    info.set("duration", 0.0)?;
    info.set("isEnabled", true)?;
    info.set("modRate", 1.0)?;
    Ok(Value::Table(info))
}

/// Build a SpellBookSkillLineInfo table for a skill line index.
fn create_skill_line_info(lua: &Lua, index: i32) -> Result<Value> {
    let Some(skill_line) = spellbook_data::get_skill_line(index) else {
        return Ok(Value::Nil);
    };
    let info = lua.create_table()?;
    info.set("name", skill_line.name)?;
    info.set("iconID", skill_line.icon_id as i64)?;
    info.set("itemIndexOffset", spellbook_data::skill_line_offset(index))?;
    info.set("numSpellBookItems", skill_line.spells.len() as i32)?;
    info.set("isGuild", false)?;
    info.set("shouldHide", false)?;
    if let Some(spec_id) = skill_line.spec_id {
        info.set("specID", spec_id)?;
    }
    if let Some(off_spec_id) = skill_line.off_spec_id {
        info.set("offSpecID", off_spec_id)?;
    }
    Ok(Value::Table(info))
}

/// Build a SpellBookItemInfo table for a slot index.
fn create_spell_book_item_info(lua: &Lua, (slot, _bank): (i32, Option<i32>)) -> Result<Value> {
    let Some((skill_line_idx, entry, skill_line)) = spellbook_data::get_spell_at_slot(slot) else {
        return Ok(Value::Nil);
    };
    let spell = crate::spells::get_spell(entry.spell_id);
    let name = spell.map(|s| s.name).unwrap_or("Unknown");
    let subtext = spell.map(|s| s.subtext).unwrap_or("");
    let icon_id = spell.map(|s| s.icon_file_data_id).unwrap_or(136243);
    let is_off_spec = skill_line.off_spec_id.is_some();
    let item_type = if is_off_spec { 2 } else { 1 }; // FutureSpell for off-spec

    let info = lua.create_table()?;
    info.set("actionID", entry.spell_id as i64)?;
    info.set("spellID", entry.spell_id as i64)?;
    info.set("itemType", item_type)?;
    info.set("name", name)?;
    info.set("subName", subtext)?;
    info.set("iconID", icon_id as i64)?;
    info.set("isPassive", entry.is_passive)?;
    info.set("isOffSpec", is_off_spec)?;
    info.set("skillLineIndex", skill_line_idx)?;
    Ok(Value::Table(info))
}

/// C_Spell namespace - spell information.
fn register_c_spell(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set("GetSpellInfo", lua.create_function(create_spell_info)?)?;
    t.set("GetSpellCharges", lua.create_function(create_spell_charges)?)?;
    t.set("IsSpellPassive", lua.create_function(|_, spell_id: i32| {
        Ok(spellbook_data::find_spell_slot(spell_id as u32)
            .and_then(|(slot, _)| spellbook_data::get_spell_at_slot(slot))
            .is_some_and(|(_, entry, _)| entry.is_passive))
    })?)?;
    t.set("GetOverrideSpell", lua.create_function(|_, spell_id: i32| Ok(spell_id))?)?;
    t.set("GetSchoolString", lua.create_function(create_school_string)?)?;
    t.set("GetSpellTexture", lua.create_function(create_spell_texture)?)?;
    t.set("GetSpellLink", lua.create_function(create_spell_link)?)?;
    t.set("GetSpellName", lua.create_function(create_spell_name)?)?;
    let st = Rc::clone(&state);
    t.set("GetSpellCooldown", lua.create_function(move |lua, spell_id: i32| {
        let s = st.borrow();
        let now = s.start_time.elapsed().as_secs_f64();
        let (start, duration) = super::action_bar_api::spell_cooldown_times(
            &s, spell_id as u32, now,
        );
        let info = lua.create_table()?;
        info.set("startTime", start)?;
        info.set("duration", duration)?;
        info.set("isEnabled", true)?;
        info.set("modRate", 1.0)?;
        Ok(info)
    })?)?;
    t.set("DoesSpellExist", lua.create_function(|_, spell_id: i32| {
        Ok(spell_id > 0 && crate::spells::get_spell(spell_id as u32).is_some())
    })?)?;
    t.set("RequestLoadSpellData", lua.create_function(|_, _spell_id: i32| Ok(()))?)?;
    t.set("IsAutoAttackSpell", lua.create_function(|_, _spell_id: i32| Ok(false))?)?;
    t.set("IsRangedAutoAttackSpell", lua.create_function(|_, _spell_id: i32| Ok(false))?)?;
    t.set("IsPressHoldReleaseSpell", lua.create_function(|_, _spell_id: i32| Ok(false))?)?;
    t.set("GetSpellLossOfControlCooldown", lua.create_function(|_, _spell_id: i32| Ok((0.0f64, 0.0f64)))?)?;
    t.set("GetMawPowerBorderAtlasBySpellID", lua.create_function(|_, _spell_id: i32| Ok(Value::Nil))?)?;
    t.set("GetSpellPowerCost", lua.create_function(create_spell_power_cost)?)?;

    Ok(t)
}

/// Return power cost info for a spell from the generated SpellPower database.
/// WoW returns an array of SpellPowerCostInfo tables with fields: type, name,
/// cost, minCost, costPercent, costPerSec, requiredAuraID, hasRequiredAura.
fn create_spell_power_cost(lua: &Lua, spell_id: i32) -> Result<Value> {
    let Some(costs) = crate::spell_power::get_spell_power(spell_id as u32) else {
        return Ok(Value::Nil);
    };
    let result = lua.create_table()?;
    for (i, cost) in costs.iter().enumerate() {
        let entry = lua.create_table()?;
        entry.set("type", cost.power_type as i32)?;
        entry.set("name", crate::spell_power::power_type_name(cost.power_type))?;
        entry.set("cost", cost.mana_cost)?;
        entry.set("minCost", cost.mana_cost)?;
        entry.set("costPercent", cost.cost_pct as f64)?;
        entry.set("costPerSec", cost.cost_per_sec as f64)?;
        entry.set("requiredAuraID", cost.required_aura_id as i64)?;
        entry.set("hasRequiredAura", cost.required_aura_id != 0)?;
        result.set(i as i64 + 1, entry)?;
    }
    Ok(Value::Table(result))
}

fn create_spell_texture(_: &Lua, spell_id: i32) -> Result<(i32, i32)> {
    let file_id = crate::spells::get_spell(spell_id as u32)
        .map(|s| s.icon_file_data_id)
        .unwrap_or(136243);
    Ok((file_id as i32, file_id as i32))
}

fn create_spell_link(lua: &Lua, spell_id: i32) -> Result<Value> {
    let name = crate::spells::get_spell(spell_id as u32)
        .map(|s| s.name)
        .unwrap_or("Unknown");
    let link = format!("|cff71d5ff|Hspell:{}|h[{}]|h|r", spell_id, name);
    Ok(Value::String(lua.create_string(&link)?))
}

fn create_spell_name(lua: &Lua, spell_id: i32) -> Result<Value> {
    let name = crate::spells::get_spell(spell_id as u32)
        .map(|s| s.name)
        .unwrap_or("Unknown");
    Ok(Value::String(lua.create_string(name)?))
}

/// Cast time in milliseconds for spells that have one (WoW API returns ms).
pub fn spell_cast_time(spell_id: i32) -> i32 {
    match spell_id {
        19750 => 1500,  // Flash of Light
        82326 => 2500,  // Holy Light
        7328 => 10000,  // Redemption (10s res)
        _ => 0,
    }
}

fn create_spell_info(lua: &Lua, spell_id: i32) -> Result<Value> {
    let info = lua.create_table()?;
    if let Some(spell) = crate::spells::get_spell(spell_id as u32) {
        info.set("name", spell.name)?;
        info.set("iconID", spell.icon_file_data_id as i64)?;
    } else {
        info.set("name", format!("Spell {}", spell_id))?;
        info.set("iconID", 136243)?;
    }
    info.set("spellID", spell_id)?;
    info.set("castTime", spell_cast_time(spell_id))?;
    info.set("minRange", 0)?;
    info.set("maxRange", 0)?;
    Ok(Value::Table(info))
}

fn create_spell_charges(lua: &Lua, _spell_id: i32) -> Result<Value> {
    let info = lua.create_table()?;
    info.set("currentCharges", 0)?;
    info.set("maxCharges", 0)?;
    info.set("cooldownStartTime", 0)?;
    info.set("cooldownDuration", 0)?;
    info.set("chargeModRate", 1.0)?;
    Ok(Value::Table(info))
}

fn create_school_string(lua: &Lua, school_mask: i32) -> Result<Value> {
    let name = match school_mask {
        1 => "Physical",
        2 => "Holy",
        4 => "Fire",
        8 => "Nature",
        16 => "Frost",
        32 => "Shadow",
        64 => "Arcane",
        _ => "Unknown",
    };
    Ok(Value::String(lua.create_string(name)?))
}

