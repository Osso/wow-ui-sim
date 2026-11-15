//! Retail possession state writers. Upstream possession triggers remain UNKNOWN.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_get};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const UPDATE_POSSESS_BAR: &str = "UPDATE_POSSESS_BAR";
const DIRECT_POSSESSION_EFFECTS: [u32; 2] = [2, 6];
const IDENTITY_POSSESSION_EFFECT: u32 = 236;
const ALTERNATE_POSSESSION_EFFECTS: [u32; 2] = [28, 51];
const POSSESSION_CLASSIFICATION_FIELD: u32 = 3;

/// One host-supplied candidate; spell effects and identity fields are not backed elsewhere.
#[derive(Clone, Debug)]
pub struct PossessionCandidateInput {
    pub spell_id: u32,
    pub tested: bool,
    pub effects: Vec<u32>,
    pub identity_type_present: bool,
    pub compared_identity_first: u32,
    pub compared_identity_second: u32,
    pub classification_record_field: Option<u32>,
}

/// INFERRED current-player object input, empty until the host supplies it.
#[derive(Clone, Debug)]
pub struct PossessionObjectInput {
    pub identity_first: u32,
    pub identity_second: u32,
    pub candidates: Vec<PossessionCandidateInput>,
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if ACTIVE != ClientProfile::Retail {
        return Ok(builder);
    }
    builder
        .set_function("SetPossessionObject", set_possession_object)?
        .set_function("SetPossessionSpellID", set_possession_spell_id_admin)?
        .set_function("UpdatePossessionPlayer", update_possession_player_admin)
}

fn set_possession_object(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    let object = if matches!(value, Val::Nil) {
        None
    } else {
        Some(read_object_input(state, value)?)
    };
    borrow_state_mut(state)?.possession_object = object;
    Ok(0)
}

fn set_possession_spell_id_admin(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    write_possession_spell_id(state, spell_id)?;
    Ok(0)
}

fn update_possession_player_admin(state: &mut LuaState) -> LuaResult<u32> {
    let identity_first = u32::from_stack(state, 1)?;
    let identity_second = u32::from_stack(state, 2)?;
    update_possession_player(state, identity_first, identity_second)?;
    Ok(0)
}

fn update_possession_player(
    state: &mut LuaState,
    identity_first: u32,
    identity_second: u32,
) -> LuaResult<()> {
    if identity_first == 0 && identity_second == 0 {
        return write_possession_spell_id(state, 0);
    }
    let object = borrow_state(state)?.possession_object.clone();
    let Some(object) = object else {
        return Ok(());
    };
    if object.identity_first != identity_first || object.identity_second != identity_second {
        return Ok(());
    }
    let Some(spell_id) =
        first_accepted_spell(&object.candidates, (identity_first, identity_second))
    else {
        return Ok(());
    };
    write_possession_spell_id(state, spell_id)
}

fn write_possession_spell_id(state: &mut LuaState, spell_id: u32) -> LuaResult<()> {
    let changed = {
        let mut sim = borrow_state_mut(state)?;
        let old_spell_id = sim.possession_spell_id.unwrap_or(0);
        sim.possession_spell_id = Some(spell_id);
        old_spell_id != spell_id
    };
    if changed {
        dispatch_event_now(state, UPDATE_POSSESS_BAR, &[])?;
    }
    Ok(())
}

fn first_accepted_spell(
    candidates: &[PossessionCandidateInput],
    selected_identity: (u32, u32),
) -> Option<u32> {
    candidates
        .iter()
        .find(|candidate| candidate_is_accepted(candidate, selected_identity))
        .map(|candidate| candidate.spell_id)
}

fn candidate_is_accepted(
    candidate: &PossessionCandidateInput,
    selected_identity: (u32, u32),
) -> bool {
    if candidate.tested {
        return false;
    }
    if crate::spells::get_spell(candidate.spell_id).is_none() {
        return false;
    }
    if candidate.effects.is_empty() {
        return false;
    }
    has_direct_effect(candidate, selected_identity) || has_alternate_effect(candidate)
}

fn has_direct_effect(candidate: &PossessionCandidateInput, selected_identity: (u32, u32)) -> bool {
    candidate.effects.iter().any(|effect| match *effect {
        IDENTITY_POSSESSION_EFFECT => candidate_matches_identity(candidate, selected_identity),
        effect => DIRECT_POSSESSION_EFFECTS.contains(&effect),
    })
}

fn candidate_matches_identity(
    candidate: &PossessionCandidateInput,
    selected_identity: (u32, u32),
) -> bool {
    candidate.identity_type_present
        && candidate.compared_identity_first == selected_identity.0
        && candidate.compared_identity_second == selected_identity.1
}

fn has_alternate_effect(candidate: &PossessionCandidateInput) -> bool {
    candidate.classification_record_field == Some(POSSESSION_CLASSIFICATION_FIELD)
        && candidate
            .effects
            .iter()
            .any(|effect| ALTERNATE_POSSESSION_EFFECTS.contains(effect))
}

fn read_object_input(state: &mut LuaState, value: Val) -> LuaResult<PossessionObjectInput> {
    if !matches!(value, Val::Table(_)) {
        return Err(runtime_error(
            "A_Admin.SetPossessionObject: expected table or nil",
        ));
    }
    let identity_first = read_u32(table_get(state, value, "identityFirst"), "identityFirst")?;
    let identity_second = read_u32(table_get(state, value, "identitySecond"), "identitySecond")?;
    let candidate_list = table_get(state, value, "candidates");
    let candidate_values = read_array(state, candidate_list)?;
    let candidates = candidate_values
        .into_iter()
        .map(|candidate| read_candidate_input(state, candidate))
        .collect::<LuaResult<Vec<_>>>()?;
    Ok(PossessionObjectInput {
        identity_first,
        identity_second,
        candidates,
    })
}

fn read_candidate_input(state: &mut LuaState, value: Val) -> LuaResult<PossessionCandidateInput> {
    if !matches!(value, Val::Table(_)) {
        return Err(runtime_error("possession candidate must be a table"));
    }
    let spell_id = read_u32(table_get(state, value, "spellID"), "spellID")?;
    let tested = read_bool(table_get(state, value, "tested"), "tested")?;
    let effects = read_effect_values(state, value)?;
    let (identity_type_present, compared_identity_first, compared_identity_second) =
        read_candidate_identity(state, value)?;
    let classification_record_field = read_optional_u32(
        table_get(state, value, "classificationField"),
        "classificationField",
    )?;
    Ok(PossessionCandidateInput {
        spell_id,
        tested,
        effects,
        identity_type_present,
        compared_identity_first,
        compared_identity_second,
        classification_record_field,
    })
}

fn read_effect_values(state: &mut LuaState, candidate: Val) -> LuaResult<Vec<u32>> {
    let effects = table_get(state, candidate, "effects");
    read_array(state, effects)?
        .into_iter()
        .map(|effect| read_u32(effect, "effects"))
        .collect()
}

fn read_candidate_identity(state: &mut LuaState, candidate: Val) -> LuaResult<(bool, u32, u32)> {
    let type_present = read_bool(
        table_get(state, candidate, "identityTypePresent"),
        "identityTypePresent",
    )?;
    let first = read_u32(
        table_get(state, candidate, "comparedIdentityFirst"),
        "comparedIdentityFirst",
    )?;
    let second = read_u32(
        table_get(state, candidate, "comparedIdentitySecond"),
        "comparedIdentitySecond",
    )?;
    Ok((type_present, first, second))
}

fn read_array(state: &LuaState, value: Val) -> LuaResult<Vec<Val>> {
    let Val::Table(reference) = value else {
        return Err(runtime_error("possession input array must be a table"));
    };
    let table = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| runtime_error("possession input array is unavailable"))?;
    Ok((1..=table.len(&state.gc.string_arena))
        .map(|index| table.get_int(index as i64))
        .collect())
}

fn read_u32(value: Val, field: &str) -> LuaResult<u32> {
    match value {
        Val::Num(number)
            if (0.0..=f64::from(u32::MAX)).contains(&number) && number.fract() == 0.0 =>
        {
            Ok(number as u32)
        }
        _ => Err(runtime_error(format!(
            "possession input {field} must be an unsigned integer"
        ))),
    }
}

fn read_bool(value: Val, field: &str) -> LuaResult<bool> {
    match value {
        Val::Bool(value) => Ok(value),
        _ => Err(runtime_error(format!(
            "possession input {field} must be a boolean"
        ))),
    }
}

fn read_optional_u32(value: Val, field: &str) -> LuaResult<Option<u32>> {
    if matches!(value, Val::Nil) {
        Ok(None)
    } else {
        read_u32(value, field).map(Some)
    }
}
