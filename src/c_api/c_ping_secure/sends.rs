//! `C_PingSecure` sends and pending hit-test information (WM38, WM47).
//!
//! Every send applies the shared eligibility first. Only its cooldown check is
//! modelled: the feature, leader and squelch predicates (results 4, 3 and 6) are
//! known gaps (WM51-F2), and so is the cooldown-started callback (WM51-F1).
//! Branches that need unmodelled native state raise "not modeled" errors.

use crate::lua_api::globals::group_queries::is_attackable_unit;
use crate::lua_api::globals::real::protected_globals::refuse_tainted_caller;
use crate::lua_api::globals::unit_misc::unit_token_for_guid;
use crate::lua_api::globals::unit_stats::unit_reaction_in_state;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_table, table_set_static, val_to_string,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

/// WM47 Q4: exactly 2^-22; equality qualifies.
const POINT_THRESHOLD: f32 = 1.0 / 4_194_304.0;
/// WM38: the categories `SendPlayerSpellCategoryPing` accepts.
const SPELL_CATEGORIES: [i32; 3] = [4, 30, 1711];

// Enum.PingResult and Enum.PingSubjectType (PingConstantsDocumentation.lua).
const FAILED_GENERIC: u8 = 1;
const FAILED_SPAMMING: u8 = 2;
const FAILED_OUT_OF_PING_AREA: u8 = 5;
const FAILED_UNSPECIFIED: u8 = 7;
const ALERT_THREAT: u8 = 4;
const ALERT_NOT_THREAT: u8 = 5;

type Member = fn(&mut LuaState) -> LuaResult<u32>;

const SEND_MEMBERS: &[(&str, Member)] = &[
    ("ClearHitTestPingInfo", clear_hit_test_ping_info),
    ("SendHitTestPing", send_hit_test_ping),
    ("SendPlayerItemPing", send_player_item_ping),
    (
        "SendPlayerSpellCategoryPing",
        send_player_spell_category_ping,
    ),
    ("SendPlayerSpellPing", send_player_spell_ping),
    ("SendUnitPing", send_unit_ping),
];

/// WM38: the pending target GUID, position and transport GUID stored by hit-test
/// selection. The simulator has no world hit selection, so hosts supply it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PingPendingTarget {
    pub target_guid: String,
    pub position: [f32; 3],
    pub transport_guid: String,
}

/// WM38: the published table holds a numeric `result` and an optional `type`.
struct SendResult {
    result: u8,
    kind: Option<u8>,
}

impl SendResult {
    fn failed(result: u8) -> Self {
        Self { result, kind: None }
    }

    fn sent(kind: u8) -> Self {
        Self {
            result: 0,
            kind: Some(kind),
        }
    }
}

pub(super) fn register(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    for (name, member) in SEND_MEMBERS {
        table_set_rust_fn_static(state, table_ref, name, *member)?;
    }
    Ok(())
}

fn clear_hit_test_ping_info(state: &mut LuaState) -> LuaResult<u32> {
    if refuse_tainted_caller(state)? {
        return Ok(0);
    }
    borrow_state_mut(state)?.ping_pending = PingPendingTarget::default();
    Ok(0)
}

/// WM38/WM47 Q4: a pending GUID takes priority over a point; the send consumes
/// the pending information whatever it returns.
fn send_hit_test_ping(state: &mut LuaState) -> LuaResult<u32> {
    if refuse_tainted_caller(state)? {
        return Ok(0);
    }
    let kind = read_optional_type(state, 1)?;
    let pending = std::mem::take(&mut borrow_state_mut(state)?.ping_pending);
    let result = if !pending.target_guid.is_empty() {
        send_to_unit(state, "SendHitTestPing", &pending.target_guid, kind)?
    } else if pending.position.iter().any(|c| c.abs() >= POINT_THRESHOLD) {
        send_to_point(state)?
    } else {
        SendResult::failed(FAILED_GENERIC)
    };
    push_result(state, result)
}

fn send_unit_ping(state: &mut LuaState) -> LuaResult<u32> {
    if refuse_tainted_caller(state)? {
        return Ok(0);
    }
    let guid = read_guid(state)?;
    let kind = read_optional_type(state, 2)?;
    let result = send_to_unit(state, "SendUnitPing", &guid, kind)?;
    push_result(state, result)
}

fn send_player_item_ping(state: &mut LuaState) -> LuaResult<u32> {
    const MEMBER: &str = "SendPlayerItemPing";
    if refuse_tainted_caller(state)? {
        return Ok(0);
    }
    let item_id = read_required_id(state, MEMBER, "itemID")?;
    match admit_player_data(state, item_id)? {
        Some(result) => push_result(state, result),
        None => not_modeled(MEMBER, "toy ownership and item-use predicates (WM47 Q5)"),
    }
}

fn send_player_spell_ping(state: &mut LuaState) -> LuaResult<u32> {
    const MEMBER: &str = "SendPlayerSpellPing";
    if refuse_tainted_caller(state)? {
        return Ok(0);
    }
    let spell_id = read_required_id(state, MEMBER, "spellID")?;
    match admit_player_data(state, spell_id)? {
        Some(result) => push_result(state, result),
        None => not_modeled(MEMBER, "the spell effect-aura predicate (WM47 Q6)"),
    }
}

fn send_player_spell_category_ping(state: &mut LuaState) -> LuaResult<u32> {
    const MEMBER: &str = "SendPlayerSpellCategoryPing";
    if refuse_tainted_caller(state)? {
        return Ok(0);
    }
    let category_id = read_required_id(state, MEMBER, "spellCategoryID")?;
    match admit_player_data(state, category_id)? {
        Some(result) => push_result(state, result),
        None if !SPELL_CATEGORIES.contains(&category_id) => {
            push_result(state, SendResult::failed(FAILED_GENERIC))
        }
        None => not_modeled(MEMBER, "the category cooldown query (WM47 Q3)"),
    }
}

/// WM38: eligibility, then a zero ID produces 7.
fn admit_player_data(state: &LuaState, id: i32) -> LuaResult<Option<SendResult>> {
    if !admit(state)? {
        return Ok(Some(SendResult::failed(FAILED_SPAMMING)));
    }
    Ok((id == 0).then(|| SendResult::failed(FAILED_UNSPECIFIED)))
}

/// WM38/WM47 Q7: eligibility (INFERRED first, WM51-F3), the empty GUID (7), then
/// unit-object resolution (5). Past that only the local player is modelled: it
/// bypasses the unit-flag rejection and redirection and passes the resource flag.
fn send_to_unit(
    state: &mut LuaState,
    member: &str,
    guid: &str,
    kind: Option<u8>,
) -> LuaResult<SendResult> {
    if !admit(state)? {
        return Ok(SendResult::failed(FAILED_SPAMMING));
    }
    if guid.is_empty() {
        return Ok(SendResult::failed(FAILED_UNSPECIFIED));
    }
    let Some(unit) = unit_token_for_guid(&borrow_state(state)?, guid) else {
        return Ok(SendResult::failed(FAILED_OUT_OF_PING_AREA));
    };
    if unit != "player" {
        return not_modeled(
            member,
            "non-self unit flags and redirection (WM47 Q7, WM51-F3)",
        );
    }
    let kind = match kind {
        Some(kind) => kind,
        None => contextual_unit_type(state, &unit)?,
    };
    record_submission(state)?;
    Ok(SendResult::sent(kind))
}

/// WM47 Q7: the point sender repeats eligibility before its distance check.
fn send_to_point(state: &LuaState) -> LuaResult<SendResult> {
    if !admit(state)? {
        return Ok(SendResult::failed(FAILED_SPAMMING));
    }
    not_modeled(
        "SendHitTestPing",
        "the point distance check (WM47 Q7, WM51-F7)",
    )
}

/// WM47 Q3: 4 when the target can attack the player with reaction at most 1.
fn contextual_unit_type(state: &mut LuaState, unit: &str) -> LuaResult<u8> {
    let threat = is_attackable_unit(state, unit)?
        && unit_reaction_in_state(&borrow_state(state)?, unit) <= 1;
    Ok(if threat {
        ALERT_THREAT
    } else {
        ALERT_NOT_THREAT
    })
}

fn admit(state: &LuaState) -> LuaResult<bool> {
    let mut sim = borrow_state_mut(state)?;
    let now = sim.cached_clock_ms();
    Ok(sim.ping_burst.admit(now))
}

/// WM47 Q1: bookkeeping follows local processing; there is no network model.
fn record_submission(state: &LuaState) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let now = sim.cached_clock_ms();
    sim.ping_burst.record_submission(now);
    Ok(())
}

fn push_result(state: &mut LuaState, result: SendResult) -> LuaResult<u32> {
    let table = create_table(state);
    state.push(table);
    table_set_static(state, table, "result", Val::Num(f64::from(result.result)));
    if let Some(kind) = result.kind {
        table_set_static(state, table, "type", Val::Num(f64::from(kind)));
    }
    Ok(1)
}

fn not_modeled<T>(member: &str, gap: &str) -> LuaResult<T> {
    Err(runtime_error(format!(
        "C_PingSecure.{member}: {gap} is not modeled"
    )))
}

/// WM38: signed 32-bit conversion of a number or numeric string, truncating
/// toward zero after the range check.
fn read_required_id(state: &LuaState, member: &str, name: &str) -> LuaResult<i32> {
    let value = unwrap_secret(state, stack_val(state, 1))?;
    read_number(state, value)
        .map(f64::trunc)
        .filter(|number| (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(number))
        .map(|number| number as i32)
        .ok_or_else(|| runtime_error(format!("Usage: C_PingSecure.{member}({name})")))
}

/// WM38: nil or missing is absent. Other values convert to an integer (0 when
/// conversion fails) and narrow to a byte, which must name a subject type (0-9).
fn read_optional_type(state: &LuaState, index: i32) -> LuaResult<Option<u8>> {
    let value = unwrap_secret(state, stack_val(state, index))?;
    if matches!(value, Val::Nil) {
        return Ok(None);
    }
    let kind = read_number(state, value).unwrap_or(0.0) as i64 as u8;
    // INFERRED: a failed range check follows the shared argument-error policy.
    if kind > 9 {
        return Err(runtime_error("invalid PingSubjectType"));
    }
    Ok(Some(kind))
}

fn read_number(state: &LuaState, value: Val) -> Option<f64> {
    match value {
        Val::Num(number) => Some(number),
        Val::Str(_) => val_to_string(state, value)?.trim().parse().ok(),
        _ => None,
    }
}

/// WM38: a required GUID string. The native GUID-text parser is a gap, so the
/// text is kept as given; INFERRED: other values are argument errors.
fn read_guid(state: &LuaState) -> LuaResult<String> {
    let value = unwrap_secret(state, stack_val(state, 1))?;
    match value {
        Val::Str(_) => Ok(val_to_string(state, value).unwrap_or_default()),
        _ => Err(runtime_error(
            "Usage: C_PingSecure.SendUnitPing(target [, type, isPlayerResource])",
        )),
    }
}
