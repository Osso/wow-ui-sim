//! `C_AchievementTelemetry` records each report in order; nothing is sent.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::LuaResult;
use rilua::vm::state::LuaState;

/// One telemetry report from the achievement UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AchievementTelemetryEvent {
    ShowAchievements,
    LinkInClub(i32),
    LinkInWhisper(i32),
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    // Generated docs publish the namespace on every profile except Wrath.
    if ACTIVE == ClientProfile::Wrath {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_AchievementTelemetry")?;
    table_set_rust_fn_static(state, namespace, "ShowAchievements", show_achievements)?;
    table_set_rust_fn_static(state, namespace, "LinkAchievementInClub", link_in_club)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "LinkAchievementInWhisper",
        link_in_whisper,
    )
}

fn record(state: &mut LuaState, event: AchievementTelemetryEvent) -> LuaResult<u32> {
    borrow_state_mut(state)?.achievement_telemetry.push(event);
    Ok(0)
}

fn show_achievements(state: &mut LuaState) -> LuaResult<u32> {
    record(state, AchievementTelemetryEvent::ShowAchievements)
}

fn link_in_club(state: &mut LuaState) -> LuaResult<u32> {
    let achievement_id = i32::from_stack(state, 1)?;
    record(state, AchievementTelemetryEvent::LinkInClub(achievement_id))
}

fn link_in_whisper(state: &mut LuaState) -> LuaResult<u32> {
    let achievement_id = i32::from_stack(state, 1)?;
    record(
        state,
        AchievementTelemetryEvent::LinkInWhisper(achievement_id),
    )
}
