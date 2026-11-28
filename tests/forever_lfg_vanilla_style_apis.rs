//! Forever APIs read by the shipped `Blizzard_GroupFinder_VanillaStyle`
//! (spec: docs/specs/forever-lfg-vanilla-style-inputs.md).
#![cfg(feature = "client-wowforever")]

use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use common::panel_fixtures::{clear_recorded_lua_errors, recorded_lua_errors};
use wow_ui_sim::lua_api::WowLuaEnv;

const LOAD_VANILLA_STYLE: &str = r#"
    A_Admin.SetSavedLFGListRoles({ tank = true, healer = false, dps = true })
    local loaded, reason = C_AddOns.LoadAddOn("Blizzard_GroupFinder_VanillaStyle")
    assert(loaded, tostring(reason))
"#;

const SOLO_ROLE_STATE: &str = r#"
    local buttons = LFGListingFrame.SoloRoleButtons
    local roles = C_LFGListRoles.GetRoles()
    return table.concat({
        tostring(buttons.Tank.CheckButton:GetChecked()),
        tostring(buttons.Healer.CheckButton:GetChecked()),
        tostring(buttons.DPS.CheckButton:GetChecked()),
        tostring(roles.tank), tostring(roles.healer), tostring(roles.dps),
    }, ",")
"#;

fn with_vanilla_style(body: fn(&WowLuaEnv)) {
    common::with_perf_lock(move || {
        common::with_timeout(240, move || {
            with_blizzard_addon_smoke_shape(&[], &[], move |env, _loaded| {
                clear_recorded_lua_errors(env);
                env.exec(LOAD_VANILLA_STYLE).expect("VanillaStyle should load");
                body(env);
                let errors = recorded_lua_errors(env);
                assert!(errors.is_empty(), "Lua errors:\n{}", errors.join("\n"));
            });
        });
    });
}

#[test]
fn solo_role_buttons_follow_saved_roles_and_role_updates() {
    with_vanilla_style(|env| {
        let startup: String = env.eval(SOLO_ROLE_STATE).unwrap();
        assert_eq!(startup, "true,false,true,true,false,true");
        env.exec("C_LFGListRoles.SetRoles({ tank = false, healer = true, dps = false })")
            .unwrap();
        let updated: String = env.eval(SOLO_ROLE_STATE).unwrap();
        assert_eq!(updated, "false,true,false,false,true,false");
        let saved: String = env
            .eval("local r = C_LFGListRoles.GetSavedRoles() return tostring(r.tank)..tostring(r.healer)")
            .unwrap();
        assert_eq!(saved, "falsetrue");
    });
}

const CREATE_WITH_COMMENT: &str = r#"
    local comment = LFGListingFrame.ActivityView.Comment.EditBox
    comment:SetText("Weekly run, chill")
    local activityID = C_LFGList.GetAvailableActivities()[1]
    assert(C_LFGList.CreateListing({ activityIDs = { activityID } }))
    local entryComment = C_LFGList.GetActiveEntryInfo().comment
    C_LFGList.ClearCreationTextFields()
    return entryComment, comment:GetText()
"#;

#[test]
fn creation_fields_clear_and_reload_from_the_active_entry() {
    with_vanilla_style(|env| {
        let (entry_comment, cleared): (String, String) = env.eval(CREATE_WITH_COMMENT).unwrap();
        assert_eq!(entry_comment, "Weekly run, chill");
        assert_eq!(cleared, "");
        // LFG_LIST_ACTIVE_ENTRY_UPDATE -> LoadActiveEntry -> CopyActiveEntryInfoToCreationFields.
        env.process_timers().expect("tick timers");
        let reloaded: String = env
            .eval("return LFGListingFrame.ActivityView.Comment.EditBox:GetText()")
            .unwrap();
        assert_eq!(reloaded, "Weekly run, chill");
    });
}

const MEMBER_ROWS: &str = r#"
    A_Admin.SetPremadeListingMembers(1, {
        { name = "Brakka", level = 60, className = "Warrior", classFilename = "WARRIOR",
          assignedRole = "TANK", lfgRoles = { tank = true, healer = false, dps = true },
          isLeader = false },
        { name = "Selwyn", level = 58, className = "Priest", classFilename = "PRIEST",
          assignedRole = "HEALER", lfgRoles = { tank = false, healer = true, dps = false },
          isLeader = true },
    })
    C_LFGList.Search(C_LFGList.GetAvailableCategories()[1])
    local leader = C_LFGList.GetSearchResultLeaderInfo(1)
    local first = C_LFGList.GetSearchResultPlayerInfo(1, 1)
    return leader.name, leader.level, leader.lfgRoles.healer, first.classFilename,
        first.assignedRole, C_LFGList.GetSearchResultPlayerInfo(1, 3) == nil,
        C_LFGList.GetSearchResultLeaderInfo(2) == nil
"#;

#[test]
fn search_result_member_rows_publish_supplied_members() {
    with_vanilla_style(|env| {
        let rows: (String, i32, bool, String, String, bool, bool) = env.eval(MEMBER_ROWS).unwrap();
        assert_eq!(
            rows,
            ("Selwyn".into(), 58, true, "WARRIOR".into(), "TANK".into(), true, true)
        );
    });
}

const WHO_SORT: &str = r#"
    local function first() return C_FriendList.GetWhoInfo(1).fullName end
    C_FriendList.SortWho("name", false)
    local descending = first()
    C_FriendList.SortWho("name")
    local toggled = first()
    local badOrigin = pcall(C_FriendList.SendWho, "Alyth", "chat")
    C_FriendList.SendWho("Alyth", Enum.SocialWhoOrigin.Social, { classIDs = {}, raceIDs = {}, uiMapIDs = {} })
    return descending, toggled, badOrigin
"#;

#[test]
fn who_sort_orders_results_and_send_who_validates_origin() {
    with_vanilla_style(|env| {
        let (descending, toggled, bad_origin): (String, String, bool) =
            env.eval(WHO_SORT).unwrap();
        assert_eq!((descending.as_str(), toggled.as_str()), ("Brennor", "Alyth"));
        assert!(!bad_origin, "a string origin must be rejected");
    });
}

const NAV_AND_LOCKS: &str = r#"
    local valid = C_Map.IsMapValidForNavBarDropdown
    local before = C_RaidLocks.GetRedirectedDifficultyID(533, 3)
    A_Admin.SetRaidDifficultyRedirects({ { mapID = 533, difficultyID = 3, redirectedDifficultyID = 9 } })
    return valid(13), valid(1), valid(947), valid(75), valid(999999), before,
        C_RaidLocks.GetRedirectedDifficultyID(533, 3), C_RaidLocks.GetRedirectedDifficultyID(533, 4)
"#;

#[test]
fn nav_bar_dropdown_maps_and_supplied_difficulty_redirects() {
    with_vanilla_style(|env| {
        type Row = (bool, bool, bool, bool, bool, i32, i32, i32);
        let row: Row = env.eval(NAV_AND_LOCKS).unwrap();
        assert_eq!(row, (true, true, false, false, false, 3, 9, 4));
    });
}
