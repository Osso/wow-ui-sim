#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::c_api::c_calendar::day_event_record::CalendarDayEventRecord;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

#[path = "calendar_day_events/fixtures.rs"]
mod supplied_calendar_fixture;

fn game_time_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create Game environment");
    let ui = crate::common::blizzard_ui_dir();
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    wow_ui_sim::loader::register_load_with_addons(
        &env.loader_env(),
        &wow_ui_sim::loader::discover_all_blizzard_addons(&ui),
    );
    crate::common::supply_empty_forever_spellbook_outfits(&env);
    crate::common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

fn invite_record(id: usize, status: i32) -> CalendarDayEventRecord {
    let mut record = supplied_calendar_fixture::record();
    record.event_id = format!("game-time-{id}");
    record.invite_status = status;
    record
}

fn supply_invites(env: &WowLuaEnv, statuses: &[i32]) {
    let rows = statuses
        .iter()
        .enumerate()
        .map(|(index, status)| invite_record(index, *status))
        .collect();
    env.state()
        .borrow_mut()
        .calendar_day_events
        .insert((0, 15), rows);
    env.fire_event("CALENDAR_UPDATE_PENDING_INVITES")
        .expect("calendar invite update event");
}

fn assert_indicator(env: &WowLuaEnv, count: usize, shown: bool, flashing: bool) {
    env.exec(&format!(
        "assert(C_Calendar.GetNumPendingInvites() == {count}); \
         assert(GameTimeFrame.pendingCalendarInvites == {count}); \
         assert(GameTimeFrame.flashInvite == {flashing}); \
         assert(GameTimeCalendarInvitesTexture:IsShown() == {shown}); \
         assert(GameTimeCalendarInvitesGlow:IsShown() == {shown})"
    ))
    .expect("original GameTimeFrame reflects pending calendar invites");
}

#[test]
fn game_time_calendar_invites_follow_supplied_invited_events() {
    let env = game_time_env();
    assert_indicator(&env, 0, false, false);

    supply_invites(&env, &[0, 0]);
    assert_indicator(&env, 2, true, true);

    supply_invites(&env, &[0, 1]);
    env.exec(
        "assert(C_Calendar.GetNumPendingInvites() == 1); \
         assert(GameTimeFrame.pendingCalendarInvites == 2); \
         assert(GameTimeFrame.flashInvite == true); \
         assert(GameTimeCalendarInvitesTexture:IsShown()); \
         assert(GameTimeCalendarInvitesGlow:IsShown())",
    )
    .expect("original GameTimeFrame retains its invite indicator until cleared");
}
