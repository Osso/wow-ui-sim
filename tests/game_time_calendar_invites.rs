#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use std::collections::HashMap;
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
    env.replace_calendar_day_events(HashMap::from([((0, 15), rows)]))
        .expect("replace calendar event rows");
}

fn assert_indicator(env: &WowLuaEnv, count: usize, shown: bool, flashing: bool) {
    env.exec(&format!(
        "assert(C_Calendar.GetNumPendingInvites() == {count}, 'calendar model count'); \
         assert(GameTimeFrame.pendingCalendarInvites == {count}, 'GameTimeFrame count'); \
         assert(GameTimeFrame.flashInvite == {flashing}, 'invite flashing state'); \
         assert(GameTimeCalendarInvitesTexture:IsShown() == {shown}, 'invite texture visibility'); \
         assert(GameTimeCalendarInvitesGlow:IsShown() == {shown}, 'invite glow visibility')"
    ))
    .expect("original GameTimeFrame reflects pending calendar invites");
}

#[test]
fn game_time_calendar_invites_follow_supplied_invited_events() {
    let env = game_time_env();
    env.exec(
        "CalendarInviteUpdateProbe = CreateFrame('Frame'); \
         CalendarInviteUpdateProbe:RegisterEvent('CALENDAR_UPDATE_PENDING_INVITES'); \
         CalendarInviteUpdateProbe:SetScript('OnEvent', function(self) \
             self.updates = (self.updates or 0) + 1 \
         end)",
    )
    .expect("install calendar update observer");
    assert_indicator(&env, 0, false, false);

    supply_invites(&env, &[0, 0]);
    assert_indicator(&env, 2, true, true);
    env.exec("assert(CalendarInviteUpdateProbe.updates == 1, 'two invites emitted update')")
        .expect("two pending invites notify listeners");

    supply_invites(&env, &[0, 1]);
    env.exec("assert(CalendarInviteUpdateProbe.updates == 2, 'one invite emitted update')")
        .expect("changed pending count notifies listeners");
    env.exec(
        "assert(C_Calendar.GetNumPendingInvites() == 1, 'updated calendar model count'); \
         assert(GameTimeFrame.pendingCalendarInvites == 2, 'original GameTimeFrame count'); \
         assert(GameTimeFrame.flashInvite == true, 'original invite flashing state'); \
         assert(GameTimeCalendarInvitesTexture:IsShown(), 'original invite texture visibility'); \
         assert(GameTimeCalendarInvitesGlow:IsShown(), 'original invite glow visibility')",
    )
    .expect("original GameTimeFrame retains its invite indicator until cleared");
}
