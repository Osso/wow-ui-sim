//! Active-unit selection and supplied combat-text event payloads.

use super::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_CombatText")?;
    table_set_rust_fn_static(state, namespace, "GetActiveUnit", get_active_unit)?;
    table_set_rust_fn_static(state, namespace, "SetActiveUnit", set_active_unit)?;
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) {
        // Retail CombatTextDocumentation.lua:21-29.
        table_set_rust_fn_static(
            state,
            namespace,
            "GetCurrentEventInfo",
            get_current_event_info,
        )?;
    }
    Ok(())
}

fn get_active_unit(state: &mut LuaState) -> LuaResult<u32> {
    let unit = borrow_state(state)?.combat_text_active_unit.clone();
    let value = match unit {
        Some(unit) => create_string(state, &unit),
        None => Val::Nil,
    };
    state.push(value);
    Ok(1)
}

fn set_active_unit(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    borrow_state_mut(state)?.combat_text_active_unit = Some(unit);
    Ok(0)
}

/// Owned values avoid retaining unrooted Lua strings during synchronous dispatch.
#[derive(Debug, Clone)]
pub enum PayloadValue {
    Number(f64),
    Text(String),
    Nil,
    Boolean(bool),
}

fn get_current_event_info(state: &mut LuaState) -> LuaResult<u32> {
    let payload = borrow_state(state)?
        .current_combat_text
        .clone()
        .unwrap_or_default();
    let count = payload.len() as u32;
    for value in payload {
        let value = match value {
            PayloadValue::Number(number) => Val::Num(number),
            PayloadValue::Text(text) => create_string(state, &text),
            PayloadValue::Nil => Val::Nil,
            PayloadValue::Boolean(value) => Val::Bool(value),
        };
        state.push(value);
    }
    Ok(count)
}

/// Dispatch explicitly supplied simulator payloads; names and layouts come from WM49-C.
pub(crate) fn dispatch_supplied_event(state: &mut LuaState, args: &[Val]) -> LuaResult<()> {
    let name = String::from_stack(state, 2)?;
    let layouts = event_layouts(&name)
        .ok_or_else(|| rilua::runtime_error("unsupported combat text event"))?;
    let values = &args[1..];
    let layout = layouts
        .iter()
        .find(|layout| {
            layout.len() == values.len()
                && layout
                    .iter()
                    .zip(values)
                    .all(|(kind, value)| matches_payload(*kind, *value))
        })
        .ok_or_else(|| rilua::runtime_error("invalid combat text event payload"))?;
    let mut payload = Vec::with_capacity(values.len());
    for (index, (kind, value)) in layout.iter().zip(values).enumerate() {
        payload.push(match (kind, value) {
            (b's', Val::Str(_)) => PayloadValue::Text(String::from_stack(state, index as i32 + 3)?),
            (_, Val::Nil) => PayloadValue::Nil,
            (_, Val::Num(number)) => PayloadValue::Number(*number),
            (_, Val::Bool(value)) => PayloadValue::Boolean(*value),
            _ => unreachable!(),
        });
    }
    borrow_state_mut(state)?.current_combat_text = Some(payload);
    let result = crate::lua_api::globals::state_backed_queries::dispatch_event_now(
        state,
        "COMBAT_TEXT_UPDATE",
        &args[..1],
    );
    borrow_state_mut(state)?.current_combat_text = None;
    result
}

fn matches_payload(kind: u8, value: Val) -> bool {
    match (kind, value) {
        (b's', Val::Str(_) | Val::Nil) | (b'b', Val::Bool(_)) => true,
        (b'n', Val::Num(number)) => number.is_finite(),
        (b'i', Val::Num(number)) => number.is_finite() && number.fract() == 0.0,
        _ => false,
    }
}

fn event_layouts(name: &str) -> Option<&'static [&'static [u8]]> {
    Some(match name {
        "INTERRUPT" | "SPELL_CAST" | "SPELL_ACTIVE" => &[b"s"],
        "DAMAGE_CRIT" | "DAMAGE" | "SPELL_DAMAGE_CRIT" | "SPELL_DAMAGE" | "DAMAGE_SHIELD"
        | "EXTRA_ATTACKS" | "SPLIT_DAMAGE" | "HONOR_GAINED" => &[b"n"],
        "MISS" | "DODGE" | "PARRY" | "EVADE" | "IMMUNE" | "DEFLECT" | "REFLECT" | "SPELL_MISS"
        | "SPELL_DODGE" | "SPELL_PARRY" | "SPELL_EVADE" | "SPELL_IMMUNE" | "SPELL_DEFLECT"
        | "SPELL_REFLECT" => &[b""],
        "RESIST" | "ABSORB" => &[b"", b"ii", b"iib"],
        "BLOCK" | "SPELL_RESIST" | "SPELL_BLOCK" | "SPELL_ABSORB" => &[b"", b"iib"],
        "PERIODIC_HEAL" | "HEAL_CRIT" | "HEAL" | "PERIODIC_HEAL_CRIT" | "FACTION"
        | "ABSORB_ADDED" => &[b"si"],
        "ENERGIZE" | "PERIODIC_ENERGIZE" => &[b"is"],
        "PERIODIC_HEAL_ABSORB" | "HEAL_CRIT_ABSORB" | "HEAL_ABSORB" => &[b"sii"],
        "SPELL_AURA_END"
        | "SPELL_AURA_END_HARMFUL"
        | "SPELL_AURA_START"
        | "SPELL_AURA_START_HARMFUL"
        | "SPELL_DISPELLED"
        | "SPELL_EMPOWER_START"
        | "SPELL_EMPOWER_END"
        | "SPELL_EMPOWER_INTERRUPT" => &[b"s", b""],
        _ => return None,
    })
}
