//! `C_MajorFactions` Renown surface consumed by
//! `Blizzard_ActionBar/Shared/ReputationBar.lua`.
//!
//! State sources:
//!
//! - `state.major_factions: HashMap<factionID, MajorFactionData>` —
//!   `GetMajorFactionData(factionID)` returns the matching row, or nil when
//!   the id isn't registered. `ReputationStatusBarMixin:Update` reads
//!   `renownLevel` / `renownLevelThreshold` to drive the bar's blue Renown
//!   look.
//! - `state.major_faction_renown_levels: HashMap<factionID, Vec<RenownLevelInfo>>`
//!   — `GetRenownLevels(factionID)` returns a Lua sequence built from the
//!   matching vec. The mixin uses the **last** entry's `level` to clamp the
//!   bar via `:GetMaxLevel()`. An unknown id yields an empty sequence.

mod renown_rewards;
pub use renown_rewards::RenownRewardInfo;

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{
    borrow_state, call_function_state, create_string, create_table, create_table_with_fields,
    table_set_num, table_set_static,
};
use crate::lua_api::state::{MajorFactionData, RenownLevelInfo};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register_c_major_factions_surface(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_MajorFactions")?;
    table_set_rust_fn_static(state, ns, "GetMajorFactionData", get_major_faction_data)?;
    table_set_rust_fn_static(state, ns, "GetRenownLevels", get_renown_levels)?;
    table_set_rust_fn_static(state, ns, "GetMajorFactionIDs", get_major_faction_ids)?;
    Ok(())
}

fn get_major_faction_data(state: &mut LuaState) -> LuaResult<u32> {
    let Ok(faction_id) = i64::from_stack(state, 1) else {
        return Ok(0);
    };
    let Some(data) = borrow_state(state)?
        .major_factions
        .get(&faction_id)
        .cloned()
    else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let table = build_major_faction_data_table(state, &data);
    state.push(table);
    Ok(1)
}

fn get_major_faction_ids(state: &mut LuaState) -> LuaResult<u32> {
    let expansion = i32::from_stack(state, 1).ok();
    let mut ids: Vec<i64> = {
        let s = borrow_state(state)?;
        s.major_factions
            .values()
            .filter(|d| match expansion {
                Some(exp) => d.expansion_filter == exp,
                None => true,
            })
            .map(|d| d.faction_id)
            .collect()
    };
    ids.sort_unstable();
    let sequence = create_table(state);
    let Val::Table(sequence_ref) = sequence else {
        unreachable!("create_table must return a table");
    };
    for (index, id) in ids.iter().enumerate() {
        table_set_num(
            state,
            sequence_ref,
            (index + 1) as f64,
            Val::Num(*id as f64),
        );
    }
    state.push(sequence);
    Ok(1)
}

fn get_renown_levels(state: &mut LuaState) -> LuaResult<u32> {
    let Ok(faction_id) = i64::from_stack(state, 1) else {
        return Ok(0);
    };
    let levels = borrow_state(state)?
        .major_faction_renown_levels
        .get(&faction_id)
        .cloned()
        .unwrap_or_default();
    let sequence = create_table(state);
    let Val::Table(sequence_ref) = sequence else {
        unreachable!("create_table must return a table");
    };
    for (index, level) in levels.iter().enumerate() {
        let entry = build_renown_level_entry(state, level);
        table_set_num(state, sequence_ref, (index + 1) as f64, entry);
    }
    state.push(sequence);
    Ok(1)
}

fn build_major_faction_data_table(state: &mut LuaState, data: &MajorFactionData) -> Val {
    let table = create_table(state);
    set_major_faction_numbers(state, table, data);
    set_major_faction_descriptive_fields(state, table, data);
    table
}

fn set_major_faction_descriptive_fields(state: &mut LuaState, table: Val, data: &MajorFactionData) {
    let name = create_string(state, &data.name);
    let texture_kit = create_string(state, &data.texture_kit);
    let unlock_description = optional_string_val(state, data.unlock_description.as_ref());
    let faction_font_color = create_faction_font_color(state, data);
    table_set_static(state, table, "name", name);
    table_set_static(state, table, "isUnlocked", Val::Bool(data.is_unlocked));
    table_set_static(state, table, "unlockDescription", unlock_description);
    table_set_static(state, table, "textureKit", texture_kit);
    table_set_static(state, table, "factionFontColor", faction_font_color);
}

fn set_major_faction_numbers(state: &mut LuaState, table: Val, data: &MajorFactionData) {
    set_major_faction_identity_numbers(state, table, data);
    set_major_faction_progress_numbers(state, table, data);
    set_major_faction_sound_numbers(state, table, data);
}

fn set_major_faction_identity_numbers(state: &mut LuaState, table: Val, data: &MajorFactionData) {
    table_set_static(state, table, "factionID", number(data.faction_id));
    table_set_static(
        state,
        table,
        "expansionFilter",
        number(data.expansion_filter as i64),
    );
    table_set_static(
        state,
        table,
        "expansionID",
        number(data.expansion_filter as i64),
    );
    table_set_static(state, table, "maxLevel", number(data.max_level as i64));
    table_set_static(state, table, "uiPriority", number(data.ui_priority as i64));
}

fn set_major_faction_progress_numbers(state: &mut LuaState, table: Val, data: &MajorFactionData) {
    table_set_static(
        state,
        table,
        "renownLevel",
        number(data.renown_level as i64),
    );
    table_set_static(
        state,
        table,
        "renownReputationEarned",
        number(data.renown_reputation_earned as i64),
    );
    table_set_static(
        state,
        table,
        "renownLevelThreshold",
        number(data.renown_level_threshold as i64),
    );
}

fn set_major_faction_sound_numbers(state: &mut LuaState, table: Val, data: &MajorFactionData) {
    table_set_static(
        state,
        table,
        "celebrationSoundKit",
        number(data.celebration_sound_kit as i64),
    );
    table_set_static(
        state,
        table,
        "renownFanfareSoundKitID",
        number(data.renown_fanfare_sound_kit_id as i64),
    );
}

fn number(value: i64) -> Val {
    Val::Num(value as f64)
}

fn create_faction_font_color(state: &mut LuaState, data: &MajorFactionData) -> Val {
    let (r, g, b) = data.faction_font_color;
    let color = create_color_mixin(state, r as f64, g as f64, b as f64);
    create_table_with_fields(state, &[("color", color)])
}

fn create_color_mixin(state: &mut LuaState, r: f64, g: f64, b: f64) -> Val {
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
        Err(_) => create_table_with_fields(
            state,
            &[
                ("r", Val::Num(r)),
                ("g", Val::Num(g)),
                ("b", Val::Num(b)),
                ("a", Val::Num(1.0)),
            ],
        ),
    }
}

fn optional_string_val(state: &mut LuaState, text: Option<&String>) -> Val {
    match text {
        Some(s) => create_string(state, s),
        None => Val::Nil,
    }
}

fn build_renown_level_entry(state: &mut LuaState, level: &RenownLevelInfo) -> Val {
    create_table_with_fields(
        state,
        &[
            ("factionID", Val::Num(level.faction_id as f64)),
            ("level", Val::Num(level.level as f64)),
            ("locked", Val::Bool(level.locked)),
            ("isMilestone", Val::Bool(level.is_milestone)),
            ("isCapstone", Val::Bool(level.is_capstone)),
        ],
    )
}
