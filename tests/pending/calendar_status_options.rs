//! Pending WM27 native evidence for EventGetStatusOptions. Restore both consumer tests with its producer.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn calendar_invite_status_menu_uses_supplied_options(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::{CalendarEditableAccess, CalendarInviteRecord};
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    let name: String = env.eval("return UnitName('player')").unwrap();
    let invite = CalendarInviteRecord { invite_id: "one".into(), name: Some(name), invite_status: Some(0), mod_status: Some("CREATOR".into()), ..Default::default() };
    env.state().borrow_mut().calendar_context_policy.editable = Some(CalendarEditableAccess { local_edit_credential: Some(1), invites: vec![invite], ..Default::default() });
    env.exec(r#"
        CalendarCreateEventFrame.mode = 'edit'
        calendarStatusProbe = CreateFrame('Button', 'CalendarStatusProbe', UIParent)
        calendarStatusProbe.inviteIndex = 1
        CalendarCreateEventInviteListButton_OnClick(calendarStatusProbe, 'RightButton')
        assert(Menu.GetManager():GetOpenMenu())
        assert(#C_Calendar.EventGetStatusOptions(1) == 6)
        Menu.GetManager():CloseMenus()
    "#).unwrap();
    env.state().borrow_mut().calendar_context_policy.editable.as_mut().unwrap().invites[0].invite_status = Some(1);
    env.exec(r#"
        CalendarCreateEventInviteListButton_OnClick(calendarStatusProbe, 'RightButton')
        assert(Menu.GetManager():GetOpenMenu())
        assert(#C_Calendar.EventGetStatusOptions(1) == 5)
        Menu.GetManager():CloseMenus()
    "#).unwrap();
}
}

prefork_full_ui_case! {
fn calendar_view_invite_menu_tracks_pending_event_identity(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::{CalendarEditableAccess, CalendarInviteRecord};
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    let invite = CalendarInviteRecord { invite_id: "one".into(), invite_status: Some(0), ..Default::default() };
    env.state().borrow_mut().calendar_context_policy.editable = Some(CalendarEditableAccess { event_id: "event-a".into(), local_edit_credential: Some(1), invites: vec![invite], ..Default::default() });
    env.exec(r#"
        calendarPendingProbe = CreateFrame('Button', 'CalendarPendingProbe', UIParent)
        calendarPendingProbe.inviteIndex = 1
        CalendarViewEventFrame.myInviteIndex = 1
        Menu.GetManager():CloseMenus()
        CalendarViewEventInviteListButton_OnClick(calendarPendingProbe, 'RightButton')
        assert(not Menu.GetManager():GetOpenMenu())
    "#).unwrap();
    env.state().borrow_mut().calendar_pending_invite_event_ids.insert("event-a".into());
    env.exec(r#"
        CalendarViewEventInviteListButton_OnClick(calendarPendingProbe, 'RightButton')
        assert(Menu.GetManager():GetOpenMenu())
        Menu.GetManager():CloseMenus()
    "#).unwrap();
}
}
