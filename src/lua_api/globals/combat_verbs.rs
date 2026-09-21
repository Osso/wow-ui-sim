//! Combat / cast verbs that mutate `SimState.casting`.
//!
//! Migrates action/cast globals off `GLOBAL_NIL_STUBS` onto the existing cast
//! pipeline so the simulator can exercise spell-in-flight UI:
//!
//! - `AttackTarget()`         — starts an "Auto Attack" cast marker
//! - `StopAttack()`           — clears the Auto Attack marker
//! - `CastSpell(id [, unit])`         — legacy signature, forwards to CastSpellByID
//! - `CastSpell(slot, bookType)`      — legacy spellbook signature
//! - `CastSpellByID(id [, unit])`     — starts/executes the spell
//! - `CastSpellByName(name [, unit])` — resolves and starts/executes the spell
//! - `ClickSpecialAbility(index)`     — index 1 => Auto Attack toggle,
//!   2 => Extra Attack marker
//! - `SpellTargetUnit(unit)`  — no-op when no cast pending; consumes the
//!   pending cast target when one exists
//! - `SpellIsTargeting()`     — false until the sim models spell targeting
//! - `SpellStopCasting()`     — clears the active cast marker and reports
//!   whether anything was interrupted
//! - `SpellCanTargetItem()`   — false until item-targeting cursor exists
//! - `SpellCanTargetItemID()` — false until item-targeting cursor exists
//! - `SpellStopTargeting()`   — no-op companion to `SpellIsTargeting`
//! - `UseAction(slot)`        — cast/execute the spell in an action bar slot
//! - `ActionButtonDown(id)` / `ActionButtonUp(id)` — main bar key dispatch
//! - `MultiActionButtonDown(barName, id)` / `MultiActionButtonUp(...)` —
//!   multi-bar key dispatch into the bar's `actionButtons` table
//! - `ExtraActionButtonKey(id, isDown)` — extra action bar key dispatch
//! - `TryUseActionButton(button, checkingFromDown)` — fires the action bound
//!   to a button during a key down, mirroring SecureActionButton_OnClick
//!
//! Registered from `register_tail_globals` in `register.rs` — runs after
//! `missing_surface` so the real Rust bodies overwrite any pre-existing
//! stub_nil entries that slipped through the stubs pass.

use crate::lua_api::env::WowLuaAppData;
use crate::lua_api::game_data::{
    self, CastingState, SpellCooldownState, SpellEffectResult, SpellTargetType,
};
use crate::lua_api::globals::spell_api::spell_cast_time;
use crate::lua_api::globals::spellbook_data;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, create_table,
    extract_frame_id, table_get, table_set,
};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::{FromStack, stack_val};
use crate::widget::AttributeValue;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, LuaResult, Val};

const BUTTON_STATE_PUSHED: &str = "PUSHED";
const BUTTON_STATE_NORMAL: &str = "NORMAL";

const DEFAULT_CAST_DURATION: f64 = 1.5;
const AUTO_ATTACK_NAME: &str = "Auto Attack";
const EXTRA_ATTACK_NAME: &str = "Extra Attack";
const DEFAULT_ICON: &str = "Interface/Icons/INV_Misc_QuestionMark";
const DEFAULT_GCD_SECONDS: f64 = 1.5;

fn spell_name(spell_id: u32) -> String {
    crate::spells::get_spell(spell_id)
        .map(|spell| spell.name.to_string())
        .unwrap_or_else(|| format!("Spell {spell_id}"))
}

fn spell_icon(spell_id: u32) -> String {
    crate::spells::get_spell(spell_id)
        .and_then(|spell| crate::manifest_interface_data::get_texture_path(spell.icon_file_data_id))
        .unwrap_or(DEFAULT_ICON)
        .to_string()
}

fn start_cast(
    state: &mut LuaState,
    spell_id: u32,
    spell_name: &str,
    icon_path: &str,
    duration: f64,
) -> Option<u32> {
    let Ok(mut st) = borrow_state_mut(state) else {
        return None;
    };
    let now = st.start_time.elapsed().as_secs_f64();
    let cast_id = st.next_cast_id;
    st.next_cast_id = st.next_cast_id.wrapping_add(1);
    #[cfg(feature = "player-cast-durations")]
    crate::lua_api::spellcast_events::clear_replaced_specialization(&mut st);
    st.casting = Some(CastingState {
        spell_id,
        spell_name: spell_name.to_string(),
        icon_path: icon_path.to_string(),
        start_time: now,
        end_time: now + duration,
        cast_id,
        empower: None,
        delay_time: 0.0,
    });
    drop(st);
    #[cfg(feature = "player-cast-durations")]
    if let Err(error) = crate::lua_api::channeling::cancel_for_cast(state) {
        tracing::error!(%error, "failed to cancel replaced channel");
        return None;
    }
    Some(cast_id)
}

fn clear_cast_if_named(state: &mut LuaState, expected_name: &str) {
    let Ok(mut st) = borrow_state_mut(state) else {
        return;
    };
    if st
        .casting
        .as_ref()
        .is_some_and(|c| c.spell_name == expected_name)
    {
        st.casting = None;
    }
}

/// `AttackTarget()` — engage auto-attack on the current target.
fn attack_target(state: &mut LuaState) -> LuaResult<u32> {
    let _ = start_cast(state, 0, AUTO_ATTACK_NAME, DEFAULT_ICON, f64::INFINITY);
    Ok(0)
}

/// `StopAttack()` — drop the auto-attack marker if present.
fn stop_attack(state: &mut LuaState) -> LuaResult<u32> {
    clear_cast_if_named(state, AUTO_ATTACK_NAME);
    Ok(0)
}

fn stack_u32(state: &mut LuaState, index: i32) -> Option<u32> {
    match stack_val(state, index) {
        Val::Num(n) => Some(n as u32),
        _ => None,
    }
}

fn resolve_spell_id_by_name(name: &str) -> Option<u32> {
    crate::lua_api::globals::spellbook_data::find_spell_by_name(name)
}

fn instant_spell_cooldown_seconds(spell_id: u32) -> f64 {
    match spell_id {
        642 => 300.0,
        _ => 0.0,
    }
}

fn spell_can_execute_now(state: &mut LuaState, spell_id: u32) -> LuaResult<bool> {
    let target_type = game_data::spell_target_type(spell_id);
    let st = borrow_state(state)?;
    Ok(match target_type {
        SpellTargetType::Harmful => st
            .current_target
            .as_ref()
            .is_some_and(|target| target.is_enemy),
        SpellTargetType::Helpful | SpellTargetType::SelfOnly => true,
    })
}

fn start_gcd(state: &mut LuaState, duration: f64) {
    if duration <= 0.0 {
        return;
    }
    let Ok(mut st) = borrow_state_mut(state) else {
        return;
    };
    let now = st.start_time.elapsed().as_secs_f64();
    st.gcd = Some((now, duration));
}

fn start_spell_cooldown(state: &mut LuaState, spell_id: u32, duration: f64) {
    if duration <= 0.0 {
        return;
    }
    let Ok(mut st) = borrow_state_mut(state) else {
        return;
    };
    let now = st.start_time.elapsed().as_secs_f64();
    st.spell_cooldowns.insert(
        spell_id,
        SpellCooldownState {
            start: now,
            duration,
        },
    );
}

fn start_instant_spell_cooldowns(state: &mut LuaState, spell_id: u32) {
    match game_data::spell_target_type(spell_id) {
        SpellTargetType::Harmful | SpellTargetType::Helpful => {
            start_gcd(state, DEFAULT_GCD_SECONDS)
        }
        SpellTargetType::SelfOnly => {}
    }
    start_spell_cooldown(state, spell_id, instant_spell_cooldown_seconds(spell_id));
}

pub(crate) fn execute_spell_by_id(state: &mut LuaState, spell_id: u32) -> LuaResult<()> {
    if let Some(skill_line_id) = spellbook_data::profession_skill_line_for_spell(spell_id) {
        crate::logging::eprintln_elapsed(&format!(
            "[spellcast] REDIRECT spell_id={spell_id} profession_skill_line={skill_line_id}"
        ));
        crate::lua_api::globals::missing_surface::professions::open_trade_skill_for_skill_line(
            state,
            skill_line_id,
        )?;
        return Ok(());
    }

    if !spell_can_execute_now(state, spell_id)? {
        crate::logging::eprintln_elapsed(&format!(
            "[spellcast] BLOCKED spell_id={spell_id} reason=spell_can_execute_now_false"
        ));
        return Ok(());
    }

    if start_timed_spell_cast(state, spell_id) {
        return Ok(());
    }

    crate::logging::eprintln_elapsed(&format!("[spellcast] INSTANT spell_id={spell_id}"));
    start_instant_spell_cooldowns(state, spell_id);
    apply_spell_to_target(state, spell_id);
    Ok(())
}

fn start_timed_spell_cast(state: &mut LuaState, spell_id: u32) -> bool {
    let cast_time_ms = cast_time_ms_for_spell(spell_id);
    if cast_time_ms <= 0 {
        return false;
    }

    let spell_name = spell_name(spell_id);
    let icon_path = spell_icon(spell_id);
    start_gcd(state, DEFAULT_GCD_SECONDS);
    let Some(cast_id) = start_cast(
        state,
        spell_id,
        &spell_name,
        &icon_path,
        cast_time_ms as f64 / 1000.0,
    ) else {
        return false;
    };
    crate::logging::eprintln_elapsed(&format!(
        "[spellcast] START spell_id={spell_id} name={spell_name} duration_ms={cast_time_ms}"
    ));
    crate::lua_api::spellcast_events::fire_player_cast_start(state, cast_id, spell_id);
    true
}

fn cast_time_ms_for_spell(spell_id: u32) -> i32 {
    if crate::spells::get_spell(spell_id).is_some() {
        return spell_cast_time(spell_id as i32);
    }
    (DEFAULT_CAST_DURATION * 1000.0) as i32
}

fn apply_spell_to_target(state: &mut LuaState, spell_id: u32) {
    let Some(app_data) = state.app_data::<WowLuaAppData>().cloned() else {
        return;
    };
    match game_data::apply_spell_to_state(&app_data.sim_state, spell_id) {
        Some(SpellEffectResult::UnitHealthChanged(unit_id)) => {
            let unit = create_string(state, &unit_id);
            fire_named_event_state(state, "UNIT_HEALTH", &[unit]);
        }
        Some(SpellEffectResult::PlayerAurasChanged) => {
            let unit = create_string(state, "player");
            let update_info = aura_full_update_info(state);
            fire_named_event_state(state, "UNIT_AURA", &[unit, update_info]);
        }
        None => {}
    }
}

fn aura_full_update_info(state: &mut LuaState) -> Val {
    let info = create_table(state);
    table_set(state, info, "isFullUpdate", Val::Bool(true));
    info
}

fn read_action_slot(state: &mut LuaState, button: Val) -> Option<u32> {
    if let Val::Num(n) = table_get(state, button, "action") {
        return Some(n as u32);
    }
    let id = extract_frame_id(state, button)?;
    let sim = borrow_state(state).ok()?;
    match sim.widgets.get(id)?.attributes.get("action")? {
        AttributeValue::Number(n) => Some(*n as u32),
        _ => None,
    }
}

fn lookup_method(state: &mut LuaState, button: Val, name: &str) -> Val {
    let key = create_string(state, name);
    state.gettable(button, key).unwrap_or(Val::Nil)
}

fn current_button_state(state: &mut LuaState, button: Val) -> Option<String> {
    let getter = lookup_method(state, button, "GetButtonState");
    if !matches!(getter, Val::Function(_)) {
        return None;
    }
    let result = call_function_state(state, getter, &[button]).ok()?;
    let Val::Str(str_ref) = result else {
        return None;
    };
    let bytes = state.gc.string_arena.get(str_ref)?.data().to_vec();
    String::from_utf8(bytes).ok()
}

fn set_button_state(state: &mut LuaState, button: Val, target: &str) {
    let setter = lookup_method(state, button, "SetButtonState");
    if !matches!(setter, Val::Function(_)) {
        return;
    }
    let target_val = create_string(state, target);
    let _ = call_function_state(state, setter, &[button, target_val]);
}

fn dispatch_action_for_button(state: &mut LuaState, button: Val) -> LuaResult<bool> {
    let Some(slot) = read_action_slot(state, button) else {
        return Ok(false);
    };
    let spell_id = {
        let sim = borrow_state(state)?;
        sim.action_bars.get(&slot).copied()
    };
    let Some(spell_id) = spell_id else {
        return Ok(false);
    };
    execute_spell_by_id(state, spell_id)?;
    Ok(true)
}

fn lookup_bar_button(state: &mut LuaState, bar_name: &str, id: u32) -> Val {
    let bar = LuaApiMut::get_global_val(state, bar_name);
    let buttons = table_get(state, bar, "actionButtons");
    let Val::Table(buttons_ref) = buttons else {
        return Val::Nil;
    };
    integer_table_entry(state, buttons_ref, id as i64)
}

fn integer_table_entry(
    state: &LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<Table>,
    key: i64,
) -> Val {
    state
        .gc
        .tables
        .get(table_ref)
        .map(|t| t.get_int(key))
        .unwrap_or(Val::Nil)
}

/// `CastSpellByID(spellId [, unit])` — set `SimState.casting` to the spell.
pub(crate) fn cast_spell_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let Some(spell_id) = stack_u32(state, 1) else {
        return Ok(0);
    };
    let _unit = Option::<String>::from_stack(state, 2)?;
    execute_spell_by_id(state, spell_id)?;
    Ok(0)
}

/// `CastSpell(spellId [, unit])` or `CastSpell(slot, bookType)` — legacy entry.
fn cast_spell(state: &mut LuaState) -> LuaResult<u32> {
    let Some(first_arg) = stack_u32(state, 1) else {
        return Ok(0);
    };

    if let Some(book_type) = Option::<String>::from_stack(state, 2)?
        && is_spellbook_book_type(&book_type)
    {
        if let Some(spell_id) = spell_id_for_spellbook_slot(first_arg) {
            execute_spell_by_id(state, spell_id)?;
        }
        return Ok(0);
    }

    let _unit = Option::<String>::from_stack(state, 2)?;
    execute_spell_by_id(state, first_arg)?;
    Ok(0)
}

fn is_spellbook_book_type(book_type: &str) -> bool {
    matches!(book_type, "spell" | "pet" | "professions")
}

fn spell_id_for_spellbook_slot(slot: u32) -> Option<u32> {
    let slot = i32::try_from(slot).ok()?;
    spellbook_data::get_spell_at_slot(slot).map(|(_, entry, _)| entry.spell_id)
}

/// `CastSpellByName(name [, unit])` — set `SimState.casting` to the named spell.
pub(crate) fn cast_spell_by_name(state: &mut LuaState) -> LuaResult<u32> {
    let Some(name) = Option::<String>::from_stack(state, 1)? else {
        return Ok(0);
    };
    if name.is_empty() {
        return Ok(0);
    }
    if let Some(spell_id) = resolve_spell_id_by_name(&name) {
        let _unit = Option::<String>::from_stack(state, 2)?;
        execute_spell_by_id(state, spell_id)?;
    } else {
        let _ = start_cast(state, 0, &name, DEFAULT_ICON, DEFAULT_CAST_DURATION);
    }
    Ok(0)
}

/// `ClickSpecialAbility(index)` — 1 = auto-attack toggle, 2 = extra attack,
/// other indices are silent no-ops.
fn click_special_ability(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = stack_u32(state, 1) else {
        return Ok(0);
    };
    match index {
        1 => {
            let _ = start_cast(state, 0, AUTO_ATTACK_NAME, DEFAULT_ICON, f64::INFINITY);
        }
        2 => {
            let _ = start_cast(
                state,
                0,
                EXTRA_ATTACK_NAME,
                DEFAULT_ICON,
                DEFAULT_CAST_DURATION,
            );
        }
        _ => {}
    }
    Ok(0)
}

/// `SpellTargetUnit(unit)` — consume the pending cast target when a cast
/// is in flight. Silent no-op otherwise, matching retail behaviour where
/// `SpellTargetUnit` is only meaningful for a pending spell.
fn spell_target_unit(state: &mut LuaState) -> LuaResult<u32> {
    let Ok(st) = borrow_state_mut(state) else {
        return Ok(0);
    };
    if st.casting.is_none() {
        return Ok(0);
    }
    drop(st);
    let _ = Option::<String>::from_stack(state, 1);
    Ok(0)
}

/// `SpellIsTargeting()` — targeting cursor is not modeled yet.
fn spell_is_targeting(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

/// `SpellStopCasting()` — interrupt the active cast marker when one exists.
fn spell_stop_casting(state: &mut LuaState) -> LuaResult<u32> {
    let cast = borrow_state_mut(state)?.casting.take();
    #[cfg(feature = "player-cast-durations")]
    if let Some(cast) = &cast {
        let interrupted_by = {
            let sim = borrow_state(state)?;
            super::unit_misc::guid_for_unit(&sim, "player")
        };
        crate::lua_api::spellcast_events::fire_player_cast_interrupted(
            state,
            cast.cast_id,
            cast.spell_id,
            &interrupted_by,
        );
    }
    let stopped = cast.is_some();
    #[cfg(feature = "player-cast-durations")]
    let stopped = stopped || crate::lua_api::channeling::stop(state, false)?;
    state.push(Val::Bool(stopped));
    Ok(1)
}

/// `SpellCanTargetItem()` — item-targeting cursor is not modeled yet.
fn spell_can_target_item(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

/// `SpellCanTargetItemID()` — item-targeting cursor is not modeled yet.
fn spell_can_target_item_id(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

/// `SpellStopTargeting()` — silent no-op until targeting cursor state exists.
fn spell_stop_targeting(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn use_action(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = stack_u32(state, 1) else {
        crate::logging::eprintln_elapsed("[spellcast] UseAction ignored reason=missing_slot");
        return Ok(0);
    };
    let spell_id = {
        let st = borrow_state(state)?;
        st.action_bars.get(&slot).copied()
    };
    let Some(spell_id) = spell_id else {
        crate::logging::eprintln_elapsed(&format!(
            "[spellcast] UseAction slot={slot} ignored reason=empty_slot"
        ));
        return Ok(0);
    };
    crate::logging::eprintln_elapsed(&format!(
        "[spellcast] UseAction slot={slot} spell_id={spell_id}"
    ));
    execute_spell_by_id(state, spell_id)?;
    Ok(0)
}

/// `TryUseActionButton(button, checkingFromDown)` — fire the bound action when
/// `checkingFromDown` is truthy. Returns true if the slot resolved to a known
/// spell and the cast pipeline was invoked.
fn try_use_action_button(state: &mut LuaState) -> LuaResult<u32> {
    let button = stack_val(state, 1);
    let checking_from_down = matches!(stack_val(state, 2), Val::Bool(true));
    let used = if checking_from_down {
        dispatch_action_for_button(state, button)?
    } else {
        false
    };
    state.push(Val::Bool(used));
    Ok(1)
}

fn press_button_and_fire(state: &mut LuaState, button: Val) -> LuaResult<()> {
    if matches!(button, Val::Nil) {
        return Ok(());
    }
    let was_normal = current_button_state(state, button).as_deref() == Some(BUTTON_STATE_NORMAL);
    if was_normal {
        set_button_state(state, button, BUTTON_STATE_PUSHED);
    }
    let _ = dispatch_action_for_button(state, button)?;
    Ok(())
}

fn release_button_if_pushed(state: &mut LuaState, button: Val) -> LuaResult<()> {
    if matches!(button, Val::Nil) {
        return Ok(());
    }
    let was_pushed = current_button_state(state, button).as_deref() == Some(BUTTON_STATE_PUSHED);
    if !was_pushed {
        return Ok(());
    }
    set_button_state(state, button, BUTTON_STATE_NORMAL);
    Ok(())
}

fn action_button_down(state: &mut LuaState) -> LuaResult<u32> {
    let Some(id) = stack_u32(state, 1) else {
        return Ok(0);
    };
    let button = LuaApiMut::get_global_val(state, &format!("ActionButton{id}"));
    press_button_and_fire(state, button)?;
    Ok(0)
}

fn action_button_up(state: &mut LuaState) -> LuaResult<u32> {
    let Some(id) = stack_u32(state, 1) else {
        return Ok(0);
    };
    let button = LuaApiMut::get_global_val(state, &format!("ActionButton{id}"));
    release_button_if_pushed(state, button)?;
    Ok(0)
}

fn multi_bar_button_from_stack(state: &mut LuaState) -> LuaResult<Option<Val>> {
    let Some(bar_name) = Option::<String>::from_stack(state, 1)? else {
        return Ok(None);
    };
    let Some(id) = stack_u32(state, 2) else {
        return Ok(None);
    };
    Ok(Some(lookup_bar_button(state, &bar_name, id)))
}

fn multi_action_button_down(state: &mut LuaState) -> LuaResult<u32> {
    let Some(button) = multi_bar_button_from_stack(state)? else {
        return Ok(0);
    };
    press_button_and_fire(state, button)?;
    Ok(0)
}

fn multi_action_button_up(state: &mut LuaState) -> LuaResult<u32> {
    let Some(button) = multi_bar_button_from_stack(state)? else {
        return Ok(0);
    };
    release_button_if_pushed(state, button)?;
    Ok(0)
}

fn extra_action_button_key(state: &mut LuaState) -> LuaResult<u32> {
    let Some(id) = stack_u32(state, 1) else {
        return Ok(0);
    };
    let is_down = matches!(stack_val(state, 2), Val::Bool(true));
    let button = LuaApiMut::get_global_val(state, &format!("ExtraActionButton{id}"));
    if is_down {
        press_button_and_fire(state, button)?;
    } else {
        release_button_if_pushed(state, button)?;
    }
    Ok(0)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "AttackTarget", attack_target)?;
    LuaApiMut::register_function(lua, "StopAttack", stop_attack)?;
    LuaApiMut::register_function(lua, "CastSpell", cast_spell)?;
    LuaApiMut::register_function(lua, "CastSpellByID", cast_spell_by_id)?;
    LuaApiMut::register_function(lua, "CastSpellByName", cast_spell_by_name)?;
    LuaApiMut::register_function(lua, "UseAction", use_action)?;
    LuaApiMut::register_function(lua, "ActionButtonDown", action_button_down)?;
    LuaApiMut::register_function(lua, "ActionButtonUp", action_button_up)?;
    LuaApiMut::register_function(lua, "MultiActionButtonDown", multi_action_button_down)?;
    LuaApiMut::register_function(lua, "MultiActionButtonUp", multi_action_button_up)?;
    LuaApiMut::register_function(lua, "ExtraActionButtonKey", extra_action_button_key)?;
    LuaApiMut::register_function(lua, "TryUseActionButton", try_use_action_button)?;
    LuaApiMut::register_function(lua, "ClickSpecialAbility", click_special_ability)?;
    LuaApiMut::register_function(lua, "SpellTargetUnit", spell_target_unit)?;
    LuaApiMut::register_function(lua, "SpellIsTargeting", spell_is_targeting)?;
    LuaApiMut::register_function(lua, "SpellCanTargetItem", spell_can_target_item)?;
    LuaApiMut::register_function(lua, "SpellCanTargetItemID", spell_can_target_item_id)?;
    LuaApiMut::register_function(lua, "SpellStopCasting", spell_stop_casting)?;
    LuaApiMut::register_function(lua, "SpellStopTargeting", spell_stop_targeting)?;
    Ok(())
}
