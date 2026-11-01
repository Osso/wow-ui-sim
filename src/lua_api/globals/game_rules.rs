//! `C_GameRules` namespace backed by `SimState::game_rules`.
//!
//! Methods:
//!
//! - `IsGameRuleActive(name)`        — whether a rule is set.
//! - `GetGameRuleAsFloat(name)`      — float value (0 when missing).
//! - `GetGameRuleAsInt(name)`        — int value (0 when missing).
//! - `GetGameRuleAsString(name)`     — string value (`""` when missing).
//! - `IsPlunderstorm()`              — active_game_mode == Plunderstorm.
//! - `GetActiveGameMode()`           — the active `Enum.GameMode` value.
//! - `GetGameModeGlueScreenName()`   — glue screen string.
//!
//! Admin:
//! - `A_Admin.SetGameRule(name, value)` — value by Lua type: number →
//!   populates all three (int = as i64, float = as f64, string = str), string
//!   → string form only, bool → clears on false / sets 0/0/"" with true as
//!   marker. `nil` removes the rule.
//! - `A_Admin.SetActiveGameMode(mode, glueScreen?)` — mode is an integer;
//!   glueScreen defaults to `"CharacterSelect"`.

use crate::c_api::game_mode;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string, create_table};
use crate::lua_api::state::GameRuleValue;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Resolve the rule-key arg at stack index 1. Accepts either a string name
/// or an `Enum.GameRule` integer id (reverse-looked-up by scanning
/// `Enum.GameRule` for the matching value). Returns `None` when no key.
fn read_rule_key(state: &mut LuaState) -> LuaResult<Option<String>> {
    match crate::lua_bridge::stack_val(state, 1) {
        Val::Str(s) => Ok(state
            .gc
            .string_arena
            .get(s)
            .and_then(|lua_str| std::str::from_utf8(lua_str.data()).ok())
            .map(str::to_owned)),
        Val::Num(n) => Ok(resolve_rule_name_from_enum_id(state, n as i64)),
        _ => Ok(None),
    }
}

/// Reverse-lookup `Enum.GameRule` to find the rule name whose numeric id
/// matches `id`. Returns `None` when the enum table is missing or has no
/// entry with that id.
fn resolve_rule_name_from_enum_id(state: &mut LuaState, id: i64) -> Option<String> {
    let rule_table = resolve_enum_game_rule_table(state)?;
    find_rule_name_for_id(state, rule_table, id)
}

/// Resolve `_G.Enum.GameRule` to its backing table, or `None` if either
/// `Enum` or `Enum.GameRule` is missing / non-table.
fn resolve_enum_game_rule_table(
    state: &mut LuaState,
) -> Option<rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>> {
    let enum_key = state.gc.intern_string_static(b"Enum");
    let enum_val = state
        .gc
        .tables
        .get(state.global)?
        .get_str(enum_key, &state.gc.string_arena);
    let Val::Table(enum_table) = enum_val else {
        return None;
    };
    let rule_key = state.gc.intern_string_static(b"GameRule");
    let rule_val = state
        .gc
        .tables
        .get(enum_table)?
        .get_str(rule_key, &state.gc.string_arena);
    match rule_val {
        Val::Table(rule_table) => Some(rule_table),
        _ => None,
    }
}

/// Walk the hash part of `rule_table` (name → numeric id pairs) and
/// return the first name whose id matches. None when no entry matches.
fn find_rule_name_for_id(
    state: &mut LuaState,
    rule_table: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    id: i64,
) -> Option<String> {
    let mut key = Val::Nil;
    while let Some((next_key, next_value)) = state
        .gc
        .tables
        .get(rule_table)?
        .next(key, &state.gc.string_arena)
        .ok()?
    {
        if let (Val::Str(name_ref), Val::Num(value)) = (next_key, next_value) {
            if value as i64 == id {
                return state
                    .gc
                    .string_arena
                    .get(name_ref)
                    .and_then(|lua_str| std::str::from_utf8(lua_str.data()).ok())
                    .map(str::to_owned);
            }
        }
        key = next_key;
    }
    None
}

pub fn is_game_rule_active(state: &mut LuaState) -> LuaResult<u32> {
    let active = match read_rule_key(state)? {
        Some(name) => borrow_state(state)?.game_rules.rules.contains_key(&name),
        None => false,
    };
    state.push(Val::Bool(active));
    Ok(1)
}

pub fn get_game_rule_as_float(state: &mut LuaState) -> LuaResult<u32> {
    // Optional second arg is a fixed-point decimal-shift count: `shift = 2`
    // divides the stored integer by 100 (e.g. 125 → 1.25). Matches
    // Blizzard's `GetGameRuleAsFloat(ruleID, decimalPlaces)` contract.
    let shift = Option::<f64>::from_stack(state, 2)?.unwrap_or(0.0) as i32;
    let divisor = 10f64.powi(shift.max(0));
    let value = match read_rule_key(state)? {
        Some(name) => borrow_state(state)?
            .game_rules
            .rules
            .get(&name)
            .map(|r| r.as_float)
            .unwrap_or(0.0),
        None => 0.0,
    };
    state.push(Val::Num(value / divisor));
    Ok(1)
}

pub fn get_game_rule_as_int(state: &mut LuaState) -> LuaResult<u32> {
    let value = match read_rule_key(state)? {
        Some(name) => borrow_state(state)?
            .game_rules
            .rules
            .get(&name)
            .map(|r| r.as_int)
            .unwrap_or(0),
        None => 0,
    };
    state.push(Val::Num(value as f64));
    Ok(1)
}

pub fn get_game_rule_as_string(state: &mut LuaState) -> LuaResult<u32> {
    let value = match read_rule_key(state)? {
        Some(name) => borrow_state(state)?
            .game_rules
            .rules
            .get(&name)
            .map(|r| r.as_string.clone())
            .unwrap_or_default(),
        None => String::new(),
    };
    let val = create_string(state, &value);
    state.push(val);
    Ok(1)
}

pub fn is_plunderstorm(state: &mut LuaState) -> LuaResult<u32> {
    let active = borrow_state(state)?.game_rules.active_game_mode == game_mode::PLUNDERSTORM;
    state.push(Val::Bool(active));
    Ok(1)
}

pub fn get_active_game_mode(state: &mut LuaState) -> LuaResult<u32> {
    let mode = borrow_state(state)?.game_rules.active_game_mode;
    state.push(Val::Num(mode as f64));
    Ok(1)
}

pub fn get_game_mode_glue_screen_name(state: &mut LuaState) -> LuaResult<u32> {
    let name = borrow_state(state)?.game_rules.glue_screen_name.clone();
    let val = create_string(state, &name);
    state.push(val);
    Ok(1)
}

fn get_num_displayed_game_modes(state: &mut LuaState) -> LuaResult<u32> {
    let _ = state;
    state.push(Val::Num(1.0));
    Ok(1)
}

fn get_displayed_game_mode_record_id_at_index(state: &mut LuaState) -> LuaResult<u32> {
    let _ = i32::from_stack(state, 1)?;
    state.push(Val::Num(1.0));
    Ok(1)
}

fn does_game_mode_have_promo(state: &mut LuaState) -> LuaResult<u32> {
    let _ = i32::from_stack(state, 1)?;
    state.push(Val::Bool(false));
    Ok(1)
}

fn is_game_mode_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let _ = i32::from_stack(state, 1)?;
    state.push(Val::Bool(true));
    Ok(1)
}

fn is_wowhack(state: &mut LuaState) -> LuaResult<u32> {
    let active = borrow_state(state)?.game_rules.active_game_mode == game_mode::WOWHACK;
    state.push(Val::Bool(active));
    Ok(1)
}

fn get_current_event_realm_queues(state: &mut LuaState) -> LuaResult<u32> {
    let queues = borrow_state(state)?.game_rules.event_realm_queues;
    state.push(Val::Num(f64::from(queues)));
    Ok(1)
}

/// `GameModeDisplayInfo` is nilable; no game-mode display records are modeled,
/// so every record ID reports no info and GameModeSelect uses the expansion logo.
fn get_game_mode_display_info_by_record_id(state: &mut LuaState) -> LuaResult<u32> {
    let _ = i32::from_stack(state, 1)?;
    state.push(Val::Nil);
    Ok(1)
}

fn install_on_c_game_rules(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_c_game_rules_table(state);
    let entries: &[(&'static str, rilua::vm::closure::RustFn)] = &[
        ("IsGameRuleActive", is_game_rule_active),
        ("GetGameRuleAsFloat", get_game_rule_as_float),
        ("GetGameRuleAsInt", get_game_rule_as_int),
        ("GetGameRuleAsString", get_game_rule_as_string),
        ("IsPlunderstorm", is_plunderstorm),
        ("GetActiveGameMode", get_active_game_mode),
        ("GetGameModeGlueScreenName", get_game_mode_glue_screen_name),
        ("GetNumDisplayedGameModes", get_num_displayed_game_modes),
        (
            "GetDisplayedGameModeRecordIDAtIndex",
            get_displayed_game_mode_record_id_at_index,
        ),
        ("DoesGameModeHavePromo", does_game_mode_have_promo),
        ("IsGameModeEnabled", is_game_mode_enabled),
    ];
    for (name, func) in entries {
        table_set_rust_fn(state, table_ref, name, *func)?;
    }
    // GameRulesDocumentation.lua declares these on every profile except Wrath,
    // which ships no generated documentation.
    if ACTIVE != ClientProfile::Wrath {
        table_set_rust_fn(state, table_ref, "IsStandard", game_mode::is_standard)?;
        table_set_rust_fn(
            state,
            table_ref,
            "IsMultiActionBarVisibilityForced",
            crate::c_api::game_rules_visibility::is_forced,
        )?;
        table_set_rust_fn(
            state,
            table_ref,
            "GetGameModeDisplayInfoByRecordID",
            get_game_mode_display_info_by_record_id,
        )?;
        table_set_rust_fn(state, table_ref, "IsWoWHack", is_wowhack)?;
        table_set_rust_fn(
            state,
            table_ref,
            "GetCurrentEventRealmQueues",
            get_current_event_realm_queues,
        )?;
    }
    Ok(())
}

/// Resolve `_G.C_GameRules` to a `GcRef<Table>`, creating the namespace
/// table if it doesn't exist yet. Mirrors the pattern used by
/// `ensure_c_*_table` in the other `C_*` modules.
fn ensure_c_game_rules_table(
    state: &mut LuaState,
) -> rilua::vm::gc::arena::GcRef<rilua::vm::table::Table> {
    let key = state.gc.intern_string_static(b"C_GameRules");
    let global = state.global;
    let existing = state
        .gc
        .tables
        .get(global)
        .map(|t| t.get_str(key, &state.gc.string_arena));
    if let Some(Val::Table(r)) = existing {
        return r;
    }
    let new_val = create_table(state);
    let Val::Table(new_ref) = new_val else {
        unreachable!("create_table must return a table");
    };
    if let Some(global_table) = state.gc.tables.get_mut(global) {
        let _ = global_table.raw_set(Val::Str(key), new_val, &state.gc.string_arena);
    }
    state.gc.barrier_back(global);
    new_ref
}

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    use rilua::LuaApiMut;
    install_on_c_game_rules(lua.state_mut())?;
    crate::c_api::ptr_i4_game_rules::register(lua.state_mut())
}

// ── Admin setters (exposed via admin_zone_economy) ────────────────────────────

/// Rule value snapshot read from the Lua stack before acquiring the mutable
/// SimState borrow.
enum RuleOp {
    Remove,
    Set(GameRuleValue),
}

fn read_rule_op(state: &mut LuaState) -> RuleOp {
    match stack_val(state, 2) {
        Val::Num(n) => RuleOp::Set(rule_value_from_number(n)),
        Val::Bool(true) => RuleOp::Set(rule_value_true_marker()),
        Val::Str(s) => RuleOp::Set(rule_value_from_string(state, s)),
        Val::Nil | Val::Bool(false) => RuleOp::Remove,
        _ => RuleOp::Remove,
    }
}

fn rule_value_from_number(n: f64) -> GameRuleValue {
    GameRuleValue {
        integer_value: (n.is_finite()
            && n.fract() == 0.0
            && n >= i64::MIN as f64
            && n < -(i64::MIN as f64))
            .then_some(n as i64),
        as_float: n,
        as_int: n as i64,
        as_string: format!("{}", n),
    }
}

fn rule_value_true_marker() -> GameRuleValue {
    GameRuleValue {
        integer_value: None,
        as_float: 1.0,
        as_int: 1,
        as_string: "true".into(),
    }
}

fn rule_value_from_string(
    state: &mut LuaState,
    s: rilua::vm::gc::arena::GcRef<rilua::vm::string::LuaString>,
) -> GameRuleValue {
    let text = state
        .gc
        .string_arena
        .get(s)
        .and_then(|lua_str| std::str::from_utf8(lua_str.data()).ok())
        .map(str::to_owned)
        .unwrap_or_default();
    let parsed_f = text.parse::<f64>().unwrap_or(0.0);
    let parsed_i = text.parse::<i64>().unwrap_or(parsed_f as i64);
    GameRuleValue {
        integer_value: None,
        as_float: parsed_f,
        as_int: parsed_i,
        as_string: text,
    }
}

/// Install a game rule by name + Lua value.
///
/// - `nil` / `false`: removes the rule.
/// - number: stored as all three forms (`as_float = n`, `as_int = n as i64`,
///   `as_string = "n"`).
/// - string: stored as-is; float/int forms try to parse the string, default 0.
/// - boolean `true`: stored as a marker with `("true", 1, 1.0)`.
pub fn admin_set_game_rule(state: &mut LuaState) -> LuaResult<u32> {
    let Some(name) = Option::<String>::from_stack(state, 1)? else {
        return Ok(0);
    };
    let op = read_rule_op(state);
    let mut sim = borrow_state_mut(state)?;
    match op {
        RuleOp::Remove => {
            sim.game_rules.rules.remove(&name);
        }
        RuleOp::Set(value) => {
            sim.game_rules.rules.insert(name, value);
        }
    }
    Ok(0)
}

pub fn admin_set_active_game_mode(state: &mut LuaState) -> LuaResult<u32> {
    let mode =
        Option::<f64>::from_stack(state, 1)?.map_or(game_mode::STANDARD, |value| value as i32);
    let glue = Option::<String>::from_stack(state, 2)?
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "CharacterSelect".into());
    let mut sim = borrow_state_mut(state)?;
    sim.game_rules.active_game_mode = mode;
    sim.game_rules.glue_screen_name = glue;
    Ok(0)
}
