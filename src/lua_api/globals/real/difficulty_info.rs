//! `GetDifficultyInfo` and `IsLegacyDifficulty` (InstanceDocumentation.lua,
//! Retail) over the pinned Retail 12.1.5.70961 `Difficulty` DB2 export.
//!
//! Name, instance type, toggle difficulty and player counts are DB2 columns.
//! INFERRED: R88g9 (native evidence pending): the bool returns read the
//! `Flags` column with the TrinityCore `DifficultyFlags` bit names below, the
//! instance type uses the `GetInstanceInfo` type strings, a zero
//! `ToggleDifficultyID` returns nil, and an unknown difficulty returns nothing.

use std::sync::OnceLock;

use crate::lua_api::methods::create_string;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

const DIFFICULTY_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/db2/retail-12.1.5.70961/Difficulty.csv"
));

const HEROIC_STYLE_LOCKOUTS: u32 = 0x01;
const CAN_SELECT: u32 = 0x04;
const CHALLENGE_MODE: u32 = 0x08;
const LFG_ONLY: u32 = 0x10;
const LEGACY: u32 = 0x20;
const DISPLAY_HEROIC: u32 = 0x40;
const DISPLAY_MYTHIC: u32 = 0x80;

const INSTANCE_TYPES: [&str; 6] = ["none", "party", "raid", "pvp", "arena", "scenario"];

struct Difficulty {
    id: i32,
    name: String,
    instance_type: usize,
    min_players: i32,
    max_players: i32,
    flags: u32,
    toggle_id: i32,
}

fn parse_row(line: &str) -> Difficulty {
    let fields: Vec<&str> = line.split(',').collect();
    let number = |index: usize| -> i64 {
        fields[index]
            .parse()
            .unwrap_or_else(|_| panic!("Difficulty.csv field {index}: {line}"))
    };
    Difficulty {
        id: number(0) as i32,
        name: fields[1].to_owned(),
        instance_type: number(2) as usize,
        min_players: number(6) as i32,
        max_players: number(7) as i32,
        flags: number(8) as u32,
        toggle_id: number(10) as i32,
    }
}

fn difficulties() -> &'static [Difficulty] {
    static ROWS: OnceLock<Vec<Difficulty>> = OnceLock::new();
    ROWS.get_or_init(|| DIFFICULTY_CSV.lines().skip(1).map(parse_row).collect())
}

fn difficulty_argument(state: &mut LuaState) -> LuaResult<Option<&'static Difficulty>> {
    let id = f64::from_stack(state, 1)
        .map_err(|_| runtime_error("Usage: expected difficultyID number"))?;
    Ok(difficulties().iter().find(|row| f64::from(row.id) == id))
}

fn get_difficulty_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(row) = difficulty_argument(state)? else {
        return Ok(0);
    };
    let flag = |bit: u32| Val::Bool(row.flags & bit != 0);
    let kind = INSTANCE_TYPES
        .get(row.instance_type)
        .copied()
        .unwrap_or("none");
    for text in [row.name.as_str(), kind] {
        let value = create_string(state, text);
        state.push(value);
    }
    let toggle = match row.toggle_id {
        0 => Val::Nil,
        id => Val::Num(f64::from(id)),
    };
    for value in [
        flag(HEROIC_STYLE_LOCKOUTS),
        flag(CHALLENGE_MODE),
        flag(DISPLAY_HEROIC),
        flag(DISPLAY_MYTHIC),
        toggle,
        flag(LFG_ONLY),
        Val::Num(f64::from(row.min_players)),
        Val::Num(f64::from(row.max_players)),
        flag(CAN_SELECT),
    ] {
        state.push(value);
    }
    Ok(11)
}

fn is_legacy_difficulty(state: &mut LuaState) -> LuaResult<u32> {
    let row = difficulty_argument(state)?;
    let value = row.map_or(Val::Nil, |row| Val::Bool(row.flags & LEGACY != 0));
    state.push(value);
    Ok(1)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetDifficultyInfo", get_difficulty_info)?;
    LuaApiMut::register_function(lua, "IsLegacyDifficulty", is_legacy_difficulty)
}
