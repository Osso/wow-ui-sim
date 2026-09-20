//! C_SpecializationInfo implementation.

use crate::lua_api::game_data::CastingState;
use crate::lua_api::globals::real::specialization_helpers::push_specialization_identity;
use crate::lua_api::globals::spellbook_data;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_table};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use crate::specializations;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

use super::helpers::ensure_namespace;

type LuaTableRef = GcRef<Table>;
type RustLuaFn = rilua::vm::closure::RustFn;

const C_SPECIALIZATION_INFO_METHODS: &[(&str, RustLuaFn)] = &[
    #[cfg(feature = "client-wowforever")]
    ("GetAllClassIDs", c_spec_get_all_class_ids),
    ("GetSpecialization", c_spec_get_specialization),
    ("GetSpecializationInfo", c_spec_get_specialization_info),
    ("GetClassIDFromSpecID", c_spec_get_class_id_from_spec_id),
    (
        "GetNumSpecializationsForClassID",
        c_spec_get_num_specializations_for_class_id,
    ),
    ("IsInitialized", c_spec_is_initialized),
    (
        "CanPlayerUseTalentSpecUI",
        c_spec_can_player_use_talent_spec_ui,
    ),
    ("CanPlayerUseTalentUI", c_spec_can_player_use_talent_ui),
    ("GetActiveSpecGroup", c_spec_get_active_spec_group),
    #[cfg(feature = "client-mists")]
    ("GetTalentInfo", c_spec_get_talent_info),
    ("GetSpellsDisplay", c_spec_get_spells_display),
    (
        "GetSpecializationMasterySpells",
        c_spec_get_specialization_mastery_spells,
    ),
    ("GetSpecIDs", c_spec_get_spec_ids),
    ("SetSpecialization", c_spec_set_specialization),
];

pub(crate) const SPEC_ACTIVATION_SPELL_ID: u32 = 200749;
const SPEC_ACTIVATION_CAST_SECONDS: f64 = 1.5;

pub fn register_c_specialization_info(state: &mut LuaState) -> LuaResult<()> {
    // Reuse the existing global table if a workaround bootstrap already created
    // it, so its gap-filler shims (pvp talents) are not clobbered by replacing
    // the table with a fresh one.
    let t_ref = ensure_namespace(state, "C_SpecializationInfo")?;
    register_c_specialization_info_methods(state, t_ref)?;
    Ok(())
}

fn register_c_specialization_info_methods(
    state: &mut LuaState,
    table_ref: LuaTableRef,
) -> LuaResult<()> {
    for (name, rust_fn) in C_SPECIALIZATION_INFO_METHODS {
        // Re-registration (the post-cleanup restore runs this again after
        // addons load) must not mint fresh closures: the Blizzard deprecated
        // specialization shim copies these values into global aliases that
        // Blizzard code expects to stay identity-equal to the namespace
        // methods. Lua-closure gap fillers from workaround bootstraps are
        // still replaced on first registration.
        if !holds_rust_fn(state, table_ref, name) {
            table_set_rust_fn_static(state, table_ref, name, *rust_fn)?;
        }
    }
    Ok(())
}

fn holds_rust_fn(state: &mut LuaState, table_ref: LuaTableRef, name: &'static str) -> bool {
    let key_ref = state.gc.intern_string_static(name.as_bytes());
    let current = state
        .gc
        .tables
        .get(table_ref)
        .map(|table| table.get_str(key_ref, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    match current {
        Val::Function(closure_ref) => matches!(
            state.gc.closures.get(closure_ref),
            Some(rilua::vm::closure::Closure::Rust(_))
        ),
        _ => false,
    }
}

fn c_spec_get_specialization(state: &mut LuaState) -> LuaResult<u32> {
    let active_spec_index = borrow_state(state)?.player.active_spec_index;
    state.push(Val::Num(active_spec_index as f64));
    Ok(1)
}

fn c_spec_get_specialization_info(state: &mut LuaState) -> LuaResult<u32> {
    let class_id = stack_val(state, 7);
    if cfg!(feature = "retail-12-0-0") && class_id != Val::Nil {
        match explicit_class_spec(class_id, stack_val(state, 1)) {
            Some(spec) => push_specialization_info(state, spec),
            None => push_specialization_defaults(state),
        }
        return Ok(10);
    }
    let requested_index = match stack_val(state, 1) {
        Val::Num(n) => n as i32,
        _ => 1,
    };
    let spec = requested_or_active_spec(state, requested_index);
    let Some(spec) = spec else {
        return Ok(0);
    };
    push_specialization_info(state, spec);
    Ok(10)
}

fn explicit_class_spec(class_id: Val, index: Val) -> Option<&'static specializations::SpecInfo> {
    let class_id = positive_integer(class_id)?;
    let index = positive_integer(index)?;
    specializations::specs_for_class(class_id).nth((index - 1) as usize)
}

fn positive_integer(value: Val) -> Option<u32> {
    let Val::Num(number) = value else {
        return None;
    };
    let in_range = number >= 1.0 && number <= u32::MAX as f64;
    if in_range && number.fract() == 0.0 {
        Some(number as u32)
    } else {
        None
    }
}

fn push_specialization_defaults(state: &mut LuaState) {
    for value in [
        Val::Num(0.0),
        Val::Nil,
        Val::Nil,
        Val::Nil,
        Val::Nil,
        Val::Nil,
        Val::Num(0.0),
        Val::Nil,
        Val::Num(0.0),
        Val::Bool(true),
    ] {
        state.push(value);
    }
}

#[cfg(feature = "client-wowforever")]
fn c_spec_get_all_class_ids(state: &mut LuaState) -> LuaResult<u32> {
    // Class catalogue membership is independent of specialization availability.
    let ids: Vec<u32> = (1..=crate::lua_api::game_data::CLASS_LABELS.len() as u32).collect();
    push_number_array(state, &ids);
    Ok(1)
}

fn c_spec_get_spec_ids(state: &mut LuaState) -> LuaResult<u32> {
    let class_id = match stack_val(state, 1) {
        Val::Num(n) if n > 0.0 => n as u32,
        _ => borrow_state(state)?.player.class_index as u32,
    };
    let spec_ids: Vec<u32> = specializations::specs_for_class(class_id)
        .map(|spec| spec.id)
        .collect();
    push_number_array(state, &spec_ids);
    Ok(1)
}

/// Push a Lua array table of the given numbers onto the stack.
fn push_number_array(state: &mut LuaState, values: &[u32]) {
    let array = create_table(state);
    let Val::Table(array_ref) = array else {
        unreachable!("create_table must return a table");
    };
    for (index, value) in values.iter().copied().enumerate() {
        if let Some(table) = state.gc.tables.get_mut(array_ref) {
            let _ = table.raw_set(
                Val::Num((index + 1) as f64),
                Val::Num(value as f64),
                &state.gc.string_arena,
            );
        }
    }
    state.gc.barrier_back(array_ref);
    state.push(array);
}

fn c_spec_get_class_id_from_spec_id(state: &mut LuaState) -> LuaResult<u32> {
    let spec_id = match stack_val(state, 1) {
        Val::Num(n) => n as u32,
        _ => 0,
    };
    let class_id = specializations::spec_by_id(spec_id)
        .map(|spec| spec.class_id as f64)
        .unwrap_or(0.0);
    state.push(Val::Num(class_id));
    Ok(1)
}

fn c_spec_get_num_specializations_for_class_id(state: &mut LuaState) -> LuaResult<u32> {
    let class_id = match stack_val(state, 1) {
        Val::Num(n) => n as u32,
        _ => 0,
    };
    let count = specializations::specs_for_class(class_id).count() as f64;
    state.push(Val::Num(count));
    Ok(1)
}

fn c_spec_is_initialized(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

fn c_spec_can_player_use_talent_spec_ui(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    state.push(Val::Nil);
    Ok(2)
}

fn c_spec_can_player_use_talent_ui(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    state.push(Val::Nil);
    Ok(2)
}

fn c_spec_get_active_spec_group(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(1.0));
    Ok(1)
}

#[cfg(feature = "client-mists")]
fn c_spec_get_talent_info(state: &mut LuaState) -> LuaResult<u32> {
    super::mists_talents::get_talent_info(state)
}

fn c_spec_get_spells_display(state: &mut LuaState) -> LuaResult<u32> {
    let spec_id = match stack_val(state, 1) {
        Val::Num(n) => n as i32,
        _ => 0,
    };
    let Some(spell_ids) = spec_display_spell_ids(spec_id) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    push_number_array(state, &spell_ids);
    Ok(1)
}

fn c_spec_get_specialization_mastery_spells(state: &mut LuaState) -> LuaResult<u32> {
    let requested_index = match stack_val(state, 1) {
        Val::Num(n) => n as i32,
        _ => 1,
    };
    let mastery_spell_ids = requested_or_active_spec(state, requested_index)
        .map(|spec| spec.mastery_spell_ids)
        .unwrap_or(&[]);
    push_number_array(state, mastery_spell_ids);
    Ok(1)
}

fn c_spec_set_specialization(state: &mut LuaState) -> LuaResult<u32> {
    let requested_index = match stack_val(state, 1) {
        Val::Num(n) => n as i32,
        _ => 0,
    };
    let can_set = player_spec_by_index(state, requested_index).is_some();
    if can_set {
        if let Some(cast_id) = start_specialization_change(state, requested_index.max(1))? {
            crate::lua_api::spellcast_events::fire_player_cast_start(
                state,
                cast_id,
                SPEC_ACTIVATION_SPELL_ID,
            );
        }
    }
    state.push(Val::Bool(can_set));
    Ok(1)
}

fn start_specialization_change(state: &mut LuaState, target_index: i32) -> LuaResult<Option<u32>> {
    let mut sim = borrow_state_mut(state)?;
    if sim.player.active_spec_index == target_index && sim.player.pending_spec_change.is_none() {
        return Ok(None);
    }

    sim.player.pending_spec_change = Some(target_index);
    let now = sim.start_time.elapsed().as_secs_f64();
    let cast_id = sim.next_cast_id;
    sim.next_cast_id = sim.next_cast_id.wrapping_add(1);
    sim.casting = Some(CastingState {
        spell_id: SPEC_ACTIVATION_SPELL_ID,
        spell_name: "Activate Specialization".to_string(),
        icon_path: String::new(),
        start_time: now,
        end_time: now + SPEC_ACTIVATION_CAST_SECONDS,
        cast_id,
        empower: None,
        delay_time: 0.0,
    });
    Ok(Some(cast_id))
}

fn requested_or_active_spec(
    state: &LuaState,
    requested_index: i32,
) -> Option<&'static specializations::SpecInfo> {
    let (class_id, active_spec_index) = {
        let sim = borrow_state(state).ok()?;
        (sim.player.class_index as u32, sim.player.active_spec_index)
    };
    let requested_spec_index = requested_index.max(1);
    specializations::specs_for_class(class_id)
        .nth((requested_spec_index - 1) as usize)
        .or_else(|| {
            let active_spec_index = active_spec_index.max(1);
            specializations::specs_for_class(class_id).nth((active_spec_index - 1) as usize)
        })
}

fn player_spec_by_index(
    state: &LuaState,
    requested_index: i32,
) -> Option<&'static specializations::SpecInfo> {
    let class_id = borrow_state(state).ok()?.player.class_index as u32;
    let requested_spec_index = requested_index.max(1);
    specializations::specs_for_class(class_id).nth((requested_spec_index - 1) as usize)
}

fn push_specialization_info(state: &mut LuaState, spec: &specializations::SpecInfo) {
    push_specialization_identity(state, spec);
    state.push(Val::Num(spec.primary_stat as f64));
    state.push(Val::Num(0.0));
    state.push(Val::Nil);
    state.push(Val::Num(0.0));
    state.push(Val::Bool(true));
}

fn spec_display_spell_ids(spec_id: i32) -> Option<Vec<u32>> {
    for skill_line_index in 1..=spellbook_data::num_skill_lines() {
        let skill_line = spellbook_data::get_skill_line(skill_line_index)?;
        if skill_line.spec_id == Some(spec_id) {
            return Some(
                skill_line
                    .spells
                    .iter()
                    .map(|entry| entry.spell_id)
                    .collect(),
            );
        }
    }
    None
}

pub fn player_get_timerunning_season_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = borrow_state(state)?.timerunning_season_id.unwrap_or(0);
    state.push(Val::Num(id as f64));
    Ok(1)
}

pub fn player_is_timerunning(state: &mut LuaState) -> LuaResult<u32> {
    let active = borrow_state(state)?.timerunning_season_id.is_some();
    state.push(Val::Bool(active));
    Ok(1)
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;
    use rilua::LuaApiMut;

    /// Mastery spells come from the modeled ChrSpecialization data: for the
    /// default Paladin player, spec index 2 is Protection whose mastery is
    /// Divine Bulwark (76671).
    #[test]
    fn get_specialization_mastery_spells_returns_real_spell_ids() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let (count, first): (i32, i32) = env
            .eval(
                r#"
                local spells = C_SpecializationInfo.GetSpecializationMasterySpells(2)
                return #spells, spells[1]
                "#,
            )
            .expect("mastery spells should be queryable");

        assert_eq!(count, 1);
        assert_eq!(first, 76671);
    }

    /// Registering the C_SpecializationInfo namespace must merge into the
    /// existing global table rather than replace it, or the gap-filler shims
    /// installed by workaround bootstraps (pvp talents) are
    /// clobbered and Blizzard code that calls them errors at runtime.
    #[test]
    fn register_preserves_existing_specialization_info_shims() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_SpecializationInfo = C_SpecializationInfo or {}
            function C_SpecializationInfo.SentinelShim()
                return "kept"
            end
            "#,
        )
        .expect("install sentinel shim");

        {
            let mut lua = env.rilua_mut();
            super::register_c_specialization_info(lua.state_mut())
                .expect("register C_SpecializationInfo");
        }

        let kept: String = env
            .eval(
                r#"return (C_SpecializationInfo.SentinelShim and C_SpecializationInfo.SentinelShim()) or "lost""#,
            )
            .expect("sentinel shim should remain callable");
        assert_eq!(
            kept, "kept",
            "c_api registration clobbered a namespace shim"
        );

        let has_getspec: String = env
            .eval(r#"return type(C_SpecializationInfo.GetSpecialization)"#)
            .expect("GetSpecialization query");
        assert_eq!(
            has_getspec, "function",
            "c_api method missing after registration"
        );
    }
}
