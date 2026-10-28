//! Explicit engraving availability (Mists/Era/Anniversary) and Mists query inputs.
//! Declarations: EngravingInfoDocumentation.lua:64-79,135-147,179-238,269-271.
use super::engraving_inputs::EngravingData;
use super::helpers::set_table_array;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set,
};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        ACTIVE,
        ClientProfile::Mists | ClientProfile::Era | ClientProfile::Anniversary
    ) {
        return Ok(());
    }
    let ns = super::ensure_namespace(state, "C_Engraving")?;
    // All three pinned EngravingInfoDocumentation.lua files declare this at 179-186.
    table_set_rust_fn_static(state, ns, "IsEngravingEnabled", is_engraving_enabled)?;
    if ACTIVE != ClientProfile::Mists {
        return Ok(());
    }
    table_set_rust_fn_static(state, ns, "GetEngravingModeEnabled", get_mode_enabled)?;
    table_set_rust_fn_static(state, ns, "GetCurrentRuneCast", get_current_rune)?;
    table_set_rust_fn_static(state, ns, "IsInventorySlotEngravable", is_slot_engravable)?;
    table_set_rust_fn_static(state, ns, "GetRuneForInventorySlot", get_slot_rune)?;
    table_set_rust_fn_static(
        state,
        ns,
        "IsInventorySlotEngravableByCurrentRuneCast",
        is_slot_engravable_by_current_cast,
    )?;
    table_set_rust_fn_static(state, ns, "RefreshRunesList", record_refresh_request)
}

fn require_supplied<T>(input: Option<T>, query: &str) -> LuaResult<T> {
    input.ok_or_else(|| runtime_error(format!("C_Engraving.{query}: unsupplied input")))
}

fn is_engraving_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let input = borrow_state(state)?.engraving.enabled;
    let enabled = if matches!(ACTIVE, ClientProfile::Era | ClientProfile::Anniversary) {
        // Era/Anniversary default: user-decided, not native evidence.
        input.unwrap_or(false)
    } else {
        require_supplied(input, "IsEngravingEnabled")?
    };
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn get_mode_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let input = borrow_state(state)?.engraving.mode_enabled;
    let enabled = require_supplied(input, "GetEngravingModeEnabled")?;
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn get_current_rune(state: &mut LuaState) -> LuaResult<u32> {
    let input = borrow_state(state)?.engraving.current_rune.clone();
    let rune = require_supplied(input, "GetCurrentRuneCast")?;
    push_optional_rune(state, rune.as_ref())
}

fn read_inventory_slot(state: &LuaState) -> LuaResult<(i32, i32)> {
    Ok((i32::from_stack(state, 1)?, i32::from_stack(state, 2)?))
}

fn is_slot_engravable(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_inventory_slot(state)?;
    let input = borrow_state(state)?
        .engraving
        .inventory_eligibility
        .get(&slot)
        .copied();
    let eligible = require_supplied(input, "IsInventorySlotEngravable")?;
    state.push(Val::Bool(eligible));
    Ok(1)
}

fn is_slot_engravable_by_current_cast(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_inventory_slot(state)?;
    let input = borrow_state(state)?
        .engraving
        .current_cast_eligibility
        .get(&slot)
        .copied();
    let eligible = require_supplied(input, "IsInventorySlotEngravableByCurrentRuneCast")?;
    state.push(Val::Bool(eligible));
    Ok(1)
}

fn get_slot_rune(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_inventory_slot(state)?;
    let input = borrow_state(state)?
        .engraving
        .inventory_runes
        .get(&slot)
        .cloned();
    let rune = require_supplied(input, "GetRuneForInventorySlot")?;
    push_optional_rune(state, rune.as_ref())
}

fn push_optional_rune(state: &mut LuaState, rune: Option<&EngravingData>) -> LuaResult<u32> {
    let value = rune.map_or(Val::Nil, |rune| encode_rune(state, rune));
    state.push(value);
    Ok(1)
}

/// Full pinned EngravingData declaration: EngravingInfoDocumentation.lua:329-340.
fn encode_rune(state: &mut LuaState, rune: &EngravingData) -> Val {
    let row = create_table(state);
    for (name, value) in [
        ("skillLineAbilityID", rune.skill_line_ability_id),
        ("itemEnchantmentID", rune.item_enchantment_id),
        ("iconTexture", rune.icon_texture),
        ("equipmentSlot", rune.equipment_slot),
        ("level", rune.level),
    ] {
        table_set(state, row, name, Val::Num(value as f64));
    }
    let name = create_string(state, &rune.name);
    table_set(state, row, "name", name);
    let spells = create_table(state);
    for (index, spell) in rune.learned_ability_spell_ids.iter().enumerate() {
        set_table_array(state, spells, index as i64 + 1, Val::Num(*spell as f64));
    }
    table_set(state, row, "learnedAbilitySpellIDs", spells);
    row
}

fn record_refresh_request(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.engraving.refresh_requests += 1;
    Ok(0)
}
