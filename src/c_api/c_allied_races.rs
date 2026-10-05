//! `C_AlliedRaces` surface consumed by `Blizzard_AlliedRacesUI`.
//!
//! State source:
//!
//! - `state.allied_races: HashMap<raceID, AlliedRaceInfo>` —
//!   `GetRaceInfoByID(raceID)` returns the matching row, or nil for
//!   unknown ids. `AlliedRacesFrameMixin:LoadRaceData` short-circuits on
//!   nil; otherwise it pulls model ids, names, atlases, and the banner
//!   color out of the returned table.
//! - `state.allied_races[raceID].racial_abilities: Vec<AlliedRaceRacialAbility>`
//!   — `GetAllRacialAbilitiesFromID(raceID)` returns a sequence of
//!   `{name, description, icon}` tables, or nil for unknown ids.
//!   `AlliedRacesFrameMixin:RacialAbilitiesData` short-circuits on nil and
//!   otherwise iterates with `ipairs`.
//!
//! `bannerColor` is wrapped in `CreateColor` so the returned table carries
//! the `ColorMixin:GetRGB` method the addon calls.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{
    borrow_state, call_function_state, create_string, create_table, create_table_with_fields,
    table_set_num, table_set_static,
};
use crate::lua_api::state::{AlliedRaceInfo, AlliedRaceRacialAbility};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register_c_allied_races_surface(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_AlliedRaces")?;
    table_set_rust_fn_static(state, ns, "GetRaceInfoByID", get_race_info_by_id)?;
    table_set_rust_fn_static(
        state,
        ns,
        "GetAllRacialAbilitiesFromID",
        get_all_racial_abilities_from_id,
    )?;
    Ok(())
}

fn get_race_info_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let Ok(race_id) = i64::from_stack(state, 1) else {
        return Ok(0);
    };
    let Some(info) = borrow_state(state)?.allied_races.get(&race_id).cloned() else {
        return Ok(0);
    };
    let table = build_allied_race_info_table(state, &info);
    state.push(table);
    Ok(1)
}

fn get_all_racial_abilities_from_id(state: &mut LuaState) -> LuaResult<u32> {
    let Ok(race_id) = i64::from_stack(state, 1) else {
        return Ok(0);
    };
    let Some(abilities) = borrow_state(state)?
        .allied_races
        .get(&race_id)
        .map(|info| info.racial_abilities.clone())
    else {
        return Ok(0);
    };
    let table = build_racial_ability_sequence(state, &abilities);
    state.push(table);
    Ok(1)
}

fn build_allied_race_info_table(state: &mut LuaState, info: &AlliedRaceInfo) -> Val {
    // Run the collecting Lua callback before creating host-held DTO values.
    let (r, g, b) = info.banner_color;
    let banner_color = create_color_mixin(state, r, g, b);
    let male_name = create_string(state, &info.male_name);
    let female_name = create_string(state, &info.female_name);
    let description = create_string(state, &info.description);
    let race_file_string = create_string(state, &info.race_file_string);
    let crest_atlas = create_string(state, &info.crest_atlas);
    let model_background_atlas = create_string(state, &info.model_background_atlas);
    let achievement_ids = build_achievement_id_sequence(state, &info.achievement_ids);
    create_table_with_fields(
        state,
        &[
            ("raceID", Val::Num(info.race_id as f64)),
            ("maleModelID", Val::Num(info.male_model_id as f64)),
            ("femaleModelID", Val::Num(info.female_model_id as f64)),
            ("achievementIds", achievement_ids),
            ("maleName", male_name),
            ("femaleName", female_name),
            ("description", description),
            ("raceFileString", race_file_string),
            ("crestAtlas", crest_atlas),
            ("modelBackgroundAtlas", model_background_atlas),
            ("bannerColor", banner_color),
        ],
    )
}

fn build_racial_ability_sequence(
    state: &mut LuaState,
    abilities: &[AlliedRaceRacialAbility],
) -> Val {
    build_sequence(state, abilities, build_racial_ability_entry)
}

fn build_racial_ability_entry(state: &mut LuaState, ability: &AlliedRaceRacialAbility) -> Val {
    let entry = create_table(state);
    let name = create_string(state, &ability.name);
    let description = create_string(state, &ability.description);
    table_set_static(state, entry, "name", name);
    table_set_static(state, entry, "description", description);
    table_set_static(state, entry, "icon", Val::Num(ability.icon as f64));
    entry
}

fn build_achievement_id_sequence(state: &mut LuaState, ids: &[i64]) -> Val {
    build_sequence(state, ids, |_, id| Val::Num(*id as f64))
}

fn build_sequence<T, F>(state: &mut LuaState, items: &[T], mut to_val: F) -> Val
where
    F: FnMut(&mut LuaState, &T) -> Val,
{
    let sequence = create_table(state);
    let Val::Table(sequence_ref) = sequence else {
        unreachable!("create_table must return a table");
    };
    for (index, item) in items.iter().enumerate() {
        let val = to_val(state, item);
        table_set_num(state, sequence_ref, (index + 1) as f64, val);
    }
    sequence
}

pub(crate) fn create_color_mixin(state: &mut LuaState, r: f64, g: f64, b: f64) -> Val {
    let create_color_key = state.gc.intern_string(b"CreateColor");
    let create_color = state
        .gc
        .tables
        .get(state.global)
        .map(|globals| globals.get_str(create_color_key, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    match call_function_state(
        state,
        create_color,
        &[Val::Num(r), Val::Num(g), Val::Num(b), Val::Num(1.0)],
    ) {
        Ok(color) => color,
        Err(_) => fallback_color_table(state, r, g, b),
    }
}

fn fallback_color_table(state: &mut LuaState, r: f64, g: f64, b: f64) -> Val {
    create_table_with_fields(
        state,
        &[
            ("r", Val::Num(r)),
            ("g", Val::Num(g)),
            ("b", Val::Num(b)),
            ("a", Val::Num(1.0)),
        ],
    )
}
