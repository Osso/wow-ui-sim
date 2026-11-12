#![cfg(feature = "gui")]

use crate::common;

use iced::Point;
use std::path::PathBuf;
use wow_ui_sim::iced_app::{build_hittable_rects, frame_collect::collect_hittable_frames};
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::settle_headless_startup;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn load_blizzard_screen(screen: ScreenKind) -> common::LockedEnv {
    common::lock_env(move || {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        env.set_screen_size(1024.0, 768.0);
        env.set_screen_mode(screen);

        let ui = blizzard_ui_dir();
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        for (name, toc_path) in &addons {
            if let Err(err) = load_addon(&env.loader_env(), toc_path) {
                panic!("[load {name}] FAILED: {err}");
            }
        }

        env.apply_post_load_workarounds();
        settle_headless_startup(&env);
        env
    })
}

fn hit_test_like_gui(env: &WowLuaEnv, pos: Point) -> Option<u64> {
    let mut state = env.state().borrow_mut();
    state.ensure_layout_rects();
    let strata_buckets = state
        .get_strata_buckets()
        .expect("visible strata buckets should exist")
        .clone();
    let collected = collect_hittable_frames(&state.widgets, &strata_buckets);
    let hittable = build_hittable_rects(&collected, &state.widgets);

    let rect_for = |id| {
        hittable
            .iter()
            .find_map(|(hid, rect, _)| (*hid == id).then_some(*rect))
    };

    let mut current = hittable
        .iter()
        .rev()
        .find_map(|(id, rect, _)| rect.contains(pos).then_some(*id))?;

    loop {
        let next = state.widgets.get(current).and_then(|frame| {
            frame.children.iter().rev().find_map(|child_id| {
                rect_for(*child_id)
                    .filter(|rect| rect.contains(pos))
                    .map(|_| *child_id)
            })
        });

        match next {
            Some(child_id) => current = child_id,
            None => return Some(current),
        }
    }
}

fn frame_chain(env: &WowLuaEnv, frame_id: u64) -> Vec<String> {
    let state = env.state().borrow();
    let mut chain = Vec::new();
    let mut current = Some(frame_id);

    while let Some(id) = current {
        let Some(frame) = state.widgets.get(id) else {
            break;
        };
        chain.push(frame.name.clone().unwrap_or_else(|| format!("#{id}")));
        current = frame.parent_id;
    }

    chain
}

fn frame_center(env: &WowLuaEnv, frame_path: &str) -> Point {
    let mut state = env.state().borrow_mut();
    state.ensure_layout_rects();

    let mut segments = frame_path.split('.');
    let root_name = segments.next().expect("frame path should have a root");
    let mut frame_id = state
        .widgets
        .get_id_by_name(root_name)
        .expect("root frame should exist in the widget registry");

    for segment in segments {
        frame_id = state
            .widgets
            .get(frame_id)
            .and_then(|frame| frame.children_keys.get(segment))
            .copied()
            .unwrap_or_else(|| panic!("frame path segment `{segment}` should exist: {frame_path}"));
    }

    let rect = state
        .widgets
        .get(frame_id)
        .and_then(|frame| frame.layout_rect)
        .expect("frame should have a layout rect");

    Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
}

#[test]
fn login_boot_hides_non_login_frontend_frames() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::Login);

        let stubs_present: bool = env
            .eval(
                r#"
                return type(GetSavedAccountName) == "function"
                    and type(SetSavedAccountName) == "function"
                    and type(GetSavedAccountList) == "function"
                    and type(SetUsesToken) == "function"
                    and type(WasScreenFirstDisplayed) == "function"
                    and type(C_Login.IsLoginReady) == "function"
                "#,
            )
            .expect("glue login stubs should be callable");
        assert!(stubs_present, "login boot should expose required glue account helpers");

        let missing_stub_errors: Vec<String> = env
            .state()
            .borrow()
            .lua_errors
            .iter()
            .filter(|msg| {
                msg.contains("GetSavedAccountName")
                    || msg.contains("IsLoginReady")
                    || msg.contains("WasScreenFirstDisplayed")
            })
            .cloned()
            .collect();
        assert!(
            missing_stub_errors.is_empty(),
            "login boot should not error on glue account helpers: {missing_stub_errors:#?}"
        );

        let account_login_visible: bool = env
            .eval("return AccountLogin ~= nil and AccountLogin:IsShown()")
            .expect("AccountLogin visibility should be queryable");
        assert!(account_login_visible, "login screen should show AccountLogin");

        // R62: Retail AccountLogin chat visibility is UNKNOWN. Its unconditional
        // shared definition must not be replaced or forced hidden. Other profiles
        // retain their independently supported visibility assertion.
        #[cfg(not(feature = "client-retail"))]
        {
        let chat_frame_visible: bool = env
            .eval("return ChatFrame1 ~= nil and ChatFrame1:IsShown()")
            .expect("ChatFrame1 visibility should be queryable");
        assert!(
            !chat_frame_visible,
            "plain login screen should not show the front-end chat frame"
        );
        }

        // R62 / tracker107: Retail chat-role dock visibility has the same
        // UNKNOWN native filtering/parent-visibility boundary as ChatFrame1.
        #[cfg(not(feature = "client-retail"))]
        {
        let chat_dock_visible: bool = env
            .eval("return GeneralDockManager ~= nil and GeneralDockManager:IsShown()")
            .expect("GeneralDockManager visibility should be queryable");
        assert!(
            !chat_dock_visible,
            "plain login screen should not show the chat dock"
        );
        }

        let char_customize_visible: bool = env
            .eval("return CharCustomizeFrame ~= nil and CharCustomizeFrame:IsShown()")
            .expect("CharCustomizeFrame visibility should be queryable");
        assert!(
            !char_customize_visible,
            "plain login screen should not show character customization"
        );
    }
}

#[test]
fn login_boot_skips_current_known_glue_errors() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::Login);

        let errors = env.state().borrow().lua_errors.clone();
        let unexpected: Vec<String> = errors
            .into_iter()
            .filter(|msg| {
                msg.contains("CHARACTER_DUPLICATE_LOGON")
                    || msg.contains("previewPanel")
                    || msg.contains("AlertFrame_SetDuration")
                    || msg.contains("PlayerLocation")
                    || msg.contains("QuickJoinToastButton")
            })
            .collect();

        assert!(
            unexpected.is_empty(),
            "login boot should not hit the current glue runtime gaps: {unexpected:#?}"
        );
    }
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_login_state_supports_glue_screen_transitions() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::Login);
        env.exec(r#"
            assert(GlueParent_IsScreenValid("login"))
            assert(GlueParent_GetBestScreen() == "login")
        "#).expect("Forever login consumers should accept the login-state table");

        env.set_screen_mode(ScreenKind::CharacterSelect);
        env.exec(r#"
            assert(not GlueParent_IsScreenValid("login"))
            assert(GlueParent_IsScreenValid("charselect"))
        "#).expect("Forever glue consumers should observe the connected screen state");

        env.set_screen_mode(ScreenKind::Login);
        env.exec(r#"
            assert(GlueParent_IsScreenValid("login"))
            assert(GlueParent_GetBestScreen() == "login")
        "#).expect("Forever glue consumers should observe the return to login");

        let login_state_errors: Vec<String> = env.state().borrow().lua_errors.iter()
            .filter(|error| error.contains("loginState"))
            .cloned()
            .collect();
        assert!(login_state_errors.is_empty(),
            "login boot should not fail to read login state: {login_state_errors:#?}");
    }
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_login_boot_has_no_lua_errors() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::Login);
        let errors = env.state().borrow().lua_errors.clone();
        assert!(errors.is_empty(), "Forever login Lua errors: {errors:#?}");
        assert!(env.eval::<bool>("return AccountLogin:IsShown() and GlueParent_GetBestScreen() == 'login'")
            .expect("login screen should remain selected and visible"));
    }
}

#[test]
fn login_boot_keeps_settings_preview_panels_wired_to_settings_panel() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::Login);

        let (font_preview, quest_preview): (bool, bool) = env
            .eval(
                r#"
                return SettingsPanel ~= nil and SettingsPanel.AccessibilityFontPreview ~= nil,
                       SettingsPanel ~= nil and SettingsPanel.QuestTextPreview ~= nil
                "#,
            )
            .expect("settings preview panel wiring should be queryable");

        assert!(
            font_preview,
            "login boot should keep SettingsPanel.AccessibilityFontPreview wired"
        );
        assert!(
            quest_preview,
            "login boot should keep SettingsPanel.QuestTextPreview wired"
        );
    }
}

#[test]
fn login_editboxes_gain_focus_when_clicking_their_visible_centers() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::Login);

        let account_center = frame_center(&env, "AccountLogin.UI.AccountEditBox");
        let password_center = frame_center(&env, "AccountLogin.UI.PasswordEditBox");

        let account_hit = hit_test_like_gui(&env, account_center)
            .expect("account edit box center should hit some widget");
        let password_hit = hit_test_like_gui(&env, password_center)
            .expect("password edit box center should hit some widget");

        env.send_click(account_hit)
            .expect("clicking the account edit box center should dispatch");
        let account_has_focus: bool = env
            .eval("return AccountLogin.UI.AccountEditBox:HasFocus()")
            .expect("AccountEditBox focus should be queryable");
        assert!(
            account_has_focus,
            "account edit box should gain focus when clicked; hit chain={:?}",
            frame_chain(&env, account_hit)
        );

        env.send_click(password_hit)
            .expect("clicking the password edit box center should dispatch");
        let password_has_focus: bool = env
            .eval("return AccountLogin.UI.PasswordEditBox:HasFocus()")
            .expect("PasswordEditBox focus should be queryable");
        assert!(
            password_has_focus,
            "password edit box should gain focus when clicked; hit chain={:?}",
            frame_chain(&env, password_hit)
        );
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn enter_credentials_and_submit(env: &WowLuaEnv, click_button: bool) {
    for (path, text) in [
        ("AccountLogin.UI.AccountEditBox", "sim@example.invalid"),
        ("AccountLogin.UI.PasswordEditBox", "arbitrary-password"),
    ] {
        let hit = hit_test_like_gui(env, frame_center(env, path))
            .expect("edit box center should be hittable");
        env.send_click(hit).expect("edit box click should dispatch");
        for character in text.chars() {
            let text = character.to_string();
            env.send_key_press(&text.to_uppercase(), Some(&text))
                .expect("typing should reach the focused edit box");
        }
        assert_eq!(
            env.eval::<String>(&format!("return {path}:GetText()"))
                .expect("typed text should be readable"),
            text
        );
    }
    if click_button {
        let hit = hit_test_like_gui(env, frame_center(env, "AccountLogin.UI.LoginButton"))
            .expect("login button center should be hittable");
        env.send_click(hit)
            .expect("login button click should dispatch");
    } else {
        env.send_key_press("ENTER", None)
            .expect("Enter should submit the password edit box");
    }
    wow_ui_sim::startup::run_extra_update_ticks(env, 3);
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn submit_login_through_ui(click_button: bool) {
    let env = load_blizzard_screen(ScreenKind::Login);
    #[cfg(feature = "client-wowforever")]
    env.exec("SetCVar('hasSelectedExperiencePreset', '1')")
        .unwrap();
    // Explicit host fixture: login never borrows the unrelated social roster.
    let fixture = env
        .state()
        .borrow()
        .bnet_friends
        .iter()
        .flat_map(|friend| &friend.game_accounts)
        .find(|account| account.client_program == "WoW")
        .unwrap()
        .clone();
    env.state().borrow_mut().glue_login.characters = vec![fixture.into()];
    env.state().borrow_mut().glue_login.connection_info =
        wow_ui_sim::c_api::c_login::ConnectionInfo {
            ready: true,
            character_limit: 2,
        };
    enter_credentials_and_submit(&env, click_button);
    let current: String = env
        .eval("return tostring(GlueParent.currentScreen)")
        .unwrap();
    let best: String = env
        .eval("return tostring(GlueParent_GetBestScreen())")
        .unwrap();
    assert!(
        env.eval::<bool>("return C_GameRules.GetGameModeGlueScreenName() == nil")
            .unwrap()
    );
    assert_eq!(
        best, "charselect",
        "original GlueParent must apply the fallback"
    );
    let (character_select, login): (bool, bool) = env
        .eval("return CharacterSelect ~= nil and CharacterSelect:IsShown(), AccountLogin:IsShown()")
        .expect("screen visibility should be readable");
    assert!(
        character_select && !login,
        "submit should show character select and hide login; character_select={character_select}, login={login}, current={current}, best={best}, errors={:#?}",
        env.state().borrow().lua_errors
    );
    let errors = env.state().borrow().lua_errors.clone();
    assert!(
        errors.is_empty(),
        "login submission should not introduce Lua errors: {errors:#?}"
    );
}

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn login_submit_enter_reaches_character_select() {
    test_timeout! { submit_login_through_ui(false); }
}

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn login_submit_button_reaches_character_select() {
    test_timeout! { submit_login_through_ui(true); }
}

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn login_api_validates_and_clears_password_synchronously() {
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.set_screen_mode(ScreenKind::Login);
    env.exec(
        r#"
        local usage = 'Usage: C_Login.Login("accountName", AccountLoginPasswordEdit)'
        local box = CreateFrame("EditBox")
        for _, args in ipairs({{false, box}, {{}, box}, {"name", false}, {"name", {}},
            {"name", nil}}) do
            local ok, err = pcall(C_Login.Login, unpack(args))
            assert(not ok and err == usage, tostring(err))
        end
        local ok, err = pcall(C_Login.Login, "name", CreateFrame("Frame"))
        assert(not ok and err:find("Wrong object type for function", 1, true))
        box:SetText("arbitrary-password")
        assert(select('#', C_Login.Login(42, box)) == 0)
        assert(box:GetText() == "")
        assert(select('#', C_Login.Login("", box)) == 0)
    "#,
    )
    .expect("R40 argument contract and synchronous password clearing");
    assert!(env.state().borrow().glue_login.characters.is_empty());
}

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn login_policy_publishes_states_and_empty_list_payload() {
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.set_screen_mode(ScreenKind::Login);
    env.exec(r#"
        loginObservations = {}
        listObservations = {}
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("LOGIN_STATE_CHANGED")
        listener:RegisterEvent("CHARACTER_LIST_UPDATE")
        listener:SetScript("OnEvent", function(_, event, ...)
            if event == "LOGIN_STATE_CHANGED" then
                assert(select('#', ...) == 0)
                local a, connected, connection, realms, waiting = C_Login.GetState()
                if type(a) == "table" then
                    local count = 0
                    for _ in pairs(a) do count = count + 1 end
                    assert(count == 9)
                    assert(not a.superDistrictChoicePending and not a.waitingForRuleSets)
                    assert(not a.hasSelectedExperiencePreset and not a.keepingCharacterListOnRealmJoin)
                    a, connected, connection, realms, waiting = a.auroraState,
                        a.connectedToWoW, a.wowConnectionState, a.hasRealmList, a.waitingForRealmList
                else
                    assert(select('#', C_Login.GetState()) == 5)
                end
                assert(connection == LE_WOW_CONNECTION_STATE_NONE)
                assert(not realms and not waiting)
                loginObservations[a] = connected and "wow" or "bnet"
            else
                assert(select('#', ...) == 1)
                table.insert(listObservations, (...))
            end
        end)
        local box = CreateFrame("EditBox")
        box:SetText("anything")
        C_Login.Login("any-account", box)
        assert(loginObservations[LE_AURORA_STATE_CONNECTING] == "bnet")
    "#).unwrap();
    // Drive to completion without asserting a native frame count or callback ordering.
    for _ in 0..10 {
        env.fire_on_update(0.016).unwrap();
    }
    env.exec(
        r#"
        assert(loginObservations[LE_AURORA_STATE_CONNECTED] == "bnet")
        assert(loginObservations[LE_AURORA_STATE_NONE] == "wow")
        assert(#listObservations == 1 and listObservations[1] == 0)
        assert(GetNumCharacters() == 0)
    "#,
    )
    .unwrap();
}

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn login_policy_replaces_pending_attempt_and_cancels_on_screen_boot() {
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.set_screen_mode(ScreenKind::Login);
    let fixture = env
        .state()
        .borrow()
        .bnet_friends
        .iter()
        .flat_map(|friend| &friend.game_accounts)
        .find(|account| account.client_program == "WoW")
        .unwrap()
        .clone();
    {
        let mut sim = env.state().borrow_mut();
        sim.glue_login.characters = vec![fixture.into()];
        sim.glue_login.realm_list_available = true;
    }
    env.exec(
        r#"
        policyListUpdates = 0
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("CHARACTER_LIST_UPDATE")
        listener:SetScript("OnEvent", function(_, _, count)
            policyListUpdates = policyListUpdates + 1
            assert(count == 1)
        end)
        policyPasswordBox = CreateFrame("EditBox")
        C_Login.Login("first", policyPasswordBox)
        local state, _, _, realmList = C_Login.GetState()
        if type(state) == "table" then realmList = state.hasRealmList end
        assert(realmList == true)
    "#,
    )
    .unwrap();
    env.fire_on_update(0.016).unwrap();
    // Explicit host boot cancels the offline policy, not a claimed native logout.
    env.set_screen_mode(ScreenKind::Login);
    for _ in 0..10 {
        env.fire_on_update(0.016).unwrap();
    }
    assert_eq!(env.eval::<i32>("return policyListUpdates").unwrap(), 0);
    assert_eq!(env.state().borrow().glue_login.characters.len(), 1);
    assert!(env.state().borrow().glue_login.realm_list_available);
    env.exec(r#"C_Login.Login("replacement-one", policyPasswordBox)"#)
        .unwrap();
    env.fire_on_update(0.016).unwrap();
    env.exec(
        r#"
        C_Login.Login("replacement-two", policyPasswordBox)
        local state = C_Login.GetState()
        if type(state) == "table" then state = state.auroraState end
        assert(state == LE_AURORA_STATE_CONNECTING)
    "#,
    )
    .unwrap();
    for _ in 0..10 {
        env.fire_on_update(0.016).unwrap();
    }
    assert_eq!(env.eval::<i32>("return policyListUpdates").unwrap(), 1);
    assert_eq!(env.eval::<i32>("return GetNumCharacters()").unwrap(), 1);
    let errors = env.state().borrow().lua_errors.clone();
    assert!(
        errors.is_empty(),
        "policy callbacks should succeed: {errors:#?}"
    );
}

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn login_queue_uses_host_snapshots_separate_from_wow_connection_state() {
    use wow_ui_sim::c_api::c_login::{LogonQueue, update_logon_queue};
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.set_screen_mode(ScreenKind::Login);
    assert_eq!(
        env.eval::<(bool, f64, f64)>("return C_Login.GetLogonQueueInfo()")
            .unwrap(),
        (false, 0.0, 0.0)
    );
    env.exec(
        r#"
        queueUpdateEvents = 0
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("LOGIN_STATE_CHANGED")
        listener:SetScript("OnEvent", function(_, _, ...)
            queueUpdateEvents = queueUpdateEvents + 1
            assert(select('#', ...) == 0)
        end)
    "#,
    )
    .unwrap();
    let queue = LogonQueue {
        server_position: 5,
        estimated_seconds: 90,
    };
    update_logon_queue(&env, Some(queue)).unwrap();
    update_logon_queue(&env, Some(queue)).unwrap();
    assert_eq!(env.eval::<i32>("return queueUpdateEvents").unwrap(), 2);
    env.fire_on_update(30.0).unwrap();
    assert_eq!(
        env.eval::<(bool, f64, f64)>("return C_Login.GetLogonQueueInfo()")
            .unwrap(),
        (true, 4.0, 90.0)
    );
    env.exec(
        r#"
        assert(select('#', C_Login.GetLogonQueueInfo()) == 3)
        local state, _, connection = C_Login.GetState()
        if type(state) == "table" then connection = state.wowConnectionState end
        assert(connection == LE_WOW_CONNECTION_STATE_NONE)
    "#,
    )
    .unwrap();
    for position in [1, 0, -1, i32::MIN] {
        update_logon_queue(
            &env,
            Some(LogonQueue {
                server_position: position,
                estimated_seconds: 90,
            }),
        )
        .unwrap();
        assert_eq!(
            env.eval::<(bool, i32, i32)>("return C_Login.GetLogonQueueInfo()")
                .unwrap(),
            (false, 0, 0)
        );
        assert_eq!(env.eval::<i32>("return queueUpdateEvents").unwrap(), 2);
    }
    update_logon_queue(&env, None).unwrap();
    env.exec(r#"C_Login.Login("offline", CreateFrame("EditBox"))"#)
        .unwrap();
    assert_eq!(
        env.eval::<(bool, f64, f64)>("return C_Login.GetLogonQueueInfo()")
            .unwrap(),
        (false, 0.0, 0.0)
    );
    let errors = env.state().borrow().lua_errors.clone();
    assert!(
        errors.is_empty(),
        "queue callbacks should succeed: {errors:#?}"
    );
}

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn login_connection_info_controls_character_creation_capacity() {
    use wow_ui_sim::c_api::c_login::ConnectionInfo;
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.set_screen_mode(ScreenKind::Login);
    assert!(!env.eval::<bool>("return CanCreateCharacter()").unwrap());
    let fixture = env
        .state()
        .borrow()
        .bnet_friends
        .iter()
        .flat_map(|friend| &friend.game_accounts)
        .find(|account| account.client_program == "WoW")
        .unwrap()
        .clone();
    for (ready, limit, populated, expected) in [
        (false, 2, true, false),
        (true, 0, false, false),
        (true, -1, false, false),
        (true, 1, false, true),
        (true, 1, true, false),
        (true, 2, true, true),
    ] {
        let mut row = fixture.clone();
        if !populated {
            row.wow_account_guid.clear();
        }
        {
            let mut sim = env.state().borrow_mut();
            sim.glue_login.characters = vec![row.into()];
            sim.glue_login.connection_info = ConnectionInfo {
                ready,
                character_limit: limit,
            };
        }
        env.exec(r#"C_Login.Login("offline", CreateFrame("EditBox"))"#)
            .unwrap();
        assert!(
            !env.eval::<bool>("return CanCreateCharacter()").unwrap(),
            "connection info has not arrived"
        );
        for _ in 0..10 {
            env.fire_on_update(0.016).unwrap();
        }
        assert_eq!(
            env.eval::<bool>("return CanCreateCharacter()").unwrap(),
            expected
        );
        assert_eq!(
            env.eval::<i32>("return select('#', CanCreateCharacter())")
                .unwrap(),
            1
        );
    }
}

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn login_submit_empty_host_roster_reaches_character_create() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::Login);
        #[cfg(feature = "client-wowforever")]
        env.exec("SetCVar('hasSelectedExperiencePreset', '1')").unwrap();
        assert!(env.state().borrow().glue_login.characters.is_empty());
        wow_ui_sim::startup::run_extra_update_ticks(&env, 3);
        assert!(env.eval::<bool>(r#"
            return GlueParent.currentScreen == "login" and AccountLogin:IsShown()
                and (not CharacterCreateFrame or not CharacterCreateFrame:IsShown())
                and GetNumCharacters() == 0
        "#).unwrap(), "empty-roster boot must remain on login before credentials");
        env.state().borrow_mut().glue_login.connection_info =
            wow_ui_sim::c_api::c_login::ConnectionInfo { ready: true, character_limit: 1 };
        // Coordinator #124: observe the transition before the original synchronous
        // CharacterCreate OnShow. Do not replace or suppress any vendor callback.
        thread_local! {
            static OBSERVED_STATE: std::cell::RefCell<
                std::rc::Weak<std::cell::RefCell<wow_ui_sim::lua_api::state::SimState>>
            > = const { std::cell::RefCell::new(std::rc::Weak::new()) };
        }
        OBSERVED_STATE.with(|slot| *slot.borrow_mut() = std::rc::Rc::downgrade(env.state()));
        env.register_rilua_function("__login_test_error_count", |state| {
            let count = OBSERVED_STATE.with(|slot| {
                slot.borrow().upgrade().expect("live observed environment").borrow().lua_errors.len()
            });
            state.push(rilua::Val::Num(count as f64));
            Ok(1)
        }).unwrap();
        env.exec(r#"
            local owner = {}
            EventRegistry:RegisterCallback("GlueParent.ScreenChanged", function(_, screen)
                if screen ~= "charcreate" then return end
                local aurora, connected, connection = C_Login.GetState()
                if type(aurora) == "table" then
                    aurora, connected, connection = aurora.auroraState,
                        aurora.connectedToWoW, aurora.wowConnectionState
                end
                emptyRosterTransition = {
                    screen = screen, currentScreen = GlueParent.currentScreen,
                    aurora = aurora, connected = connected, connection = connection,
                    count = GetNumCharacters(), errors = __login_test_error_count(),
                }
            end, owner)
        "#).unwrap();
        enter_credentials_and_submit(&env, false);
        assert!(env.eval::<bool>(r#"
            local s = emptyRosterTransition
            return s and s.screen == "charcreate" and s.currentScreen == "charcreate"
                and s.aurora == LE_AURORA_STATE_NONE and s.connected
                and s.connection == LE_WOW_CONNECTION_STATE_NONE
                and s.count == 0 and s.errors == 0
        "#).unwrap(), "empty roster must reach an error-free authenticated charcreate transition");
        // Later setup belongs to main's R67. Retain diagnostics, but make no
        // assertion that character-creation setup has succeeded.
        let errors = env.state().borrow().lua_errors.clone();
        if !errors.is_empty() {
            eprintln!("post-transition CharacterCreate setup diagnostics (R67): {errors:#?}");
        }
    }
}

#[test]
#[cfg(any(feature = "client-ptr", feature = "client-wowforever"))]
fn glue_feature_status_uses_host_record_and_ordered_empty_events() {
    use wow_ui_sim::lua_api::globals::real::glue_feature_status::{
        GlueFeatureStatus, deliver_status,
    };
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.set_screen_mode(ScreenKind::Login);
    env.exec(
        r#"
        assert(IsNameReservationOnly() == false)
        assert(select('#', IsNameReservationOnly()) == 1)
        featureEvents = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('TIMERUNNING_SEASON_UPDATE')
        listener:RegisterEvent('AVAILABLE_GAME_MODES_UPDATED')
        listener:SetScript('OnEvent', function(_, event, ...)
            table.insert(featureEvents, {event, select('#', ...)})
        end)
    "#,
    )
    .unwrap();
    for value in [true, true, false] {
        deliver_status(
            &env,
            GlueFeatureStatus {
                name_reservation_only: value,
            },
        )
        .unwrap();
        assert_eq!(
            env.eval::<bool>("return IsNameReservationOnly()").unwrap(),
            value
        );
    }
    env.exec(
        r#"
        assert(#featureEvents == 6)
        for i = 1, 6, 2 do
            assert(featureEvents[i][1] == 'TIMERUNNING_SEASON_UPDATE')
            assert(featureEvents[i + 1][1] == 'AVAILABLE_GAME_MODES_UPDATED')
            assert(featureEvents[i][2] == 0 and featureEvents[i + 1][2] == 0)
        end
    "#,
    )
    .unwrap();
    deliver_status(
        &env,
        GlueFeatureStatus {
            name_reservation_only: true,
        },
    )
    .unwrap();
    env.set_screen_mode(ScreenKind::CharacterSelect);
    assert!(
        env.eval::<bool>("return IsNameReservationOnly()").unwrap(),
        "screen/connection reset must not replace feature-system status"
    );
    env.set_screen_mode(ScreenKind::Game);
    env.set_screen_mode(ScreenKind::Login);
    assert!(env.eval::<bool>("return IsNameReservationOnly()").unwrap());
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
#[cfg(feature = "client-ptr")]
fn glue_ptr_connection_fields_project_independent_host_inputs() {
    use wow_ui_sim::c_api::c_login::ptr_state::PtrConnectionState;
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.set_screen_mode(ScreenKind::Login);
    for bits in 0..8 {
        let input = PtrConnectionState {
            connection_object_needs_connect: bits & 1 != 0,
            pending_connect: bits & 2 != 0,
            wow_queue: bits & 4 != 0,
            ..Default::default()
        };
        let expected = if bits & 3 != 0 {
            2
        } else if bits & 4 != 0 {
            3
        } else {
            1
        };
        env.state().borrow_mut().glue_login.ptr_connection = input;
        assert_eq!(
            env.eval::<i32>("return C_Login.GetState().wowConnectionState")
                .unwrap(),
            expected
        );
    }
    for bits in 0..8 {
        env.state().borrow_mut().glue_login.ptr_connection = PtrConnectionState {
            aurora_state: 5,
            connected_to_wow: bits & 1 != 0,
            realm_list_present: bits & 2 != 0,
            realm_list_available: bits & 4 != 0,
            ..Default::default()
        };
        assert_eq!(
            env.eval::<i32>("return C_Login.GetState().auroraState")
                .unwrap(),
            if bits & 1 != 0 { 1 } else { 5 }
        );
        assert_eq!(
            env.eval::<bool>("return C_Login.GetState().hasRealmList")
                .unwrap(),
            bits & 6 == 6
        );
    }
    for bits in 0..64 {
        let rule_gate = bits & 1 != 0;
        let rule_wait = bits & 2 != 0;
        let district_gate = bits & 4 != 0;
        let district_pending = bits & 8 != 0;
        let realm_wait = bits & 16 != 0;
        let keep_list = bits & 32 != 0;
        env.state().borrow_mut().glue_login.ptr_connection = PtrConnectionState {
            rule_sets_enabled: rule_gate,
            waiting_for_rule_sets: rule_wait,
            super_district_enabled: district_gate,
            super_district_choice_pending: district_pending,
            waiting_for_realm_list: realm_wait,
            keeping_character_list_on_realm_join: keep_list,
            ..Default::default()
        };
        let observed: (bool, bool, bool, bool) = env
            .eval(
                r#"
            local s = C_Login.GetState()
            return s.waitingForRuleSets, s.superDistrictChoicePending,
                s.waitingForRealmList, s.keepingCharacterListOnRealmJoin
        "#,
            )
            .unwrap();
        let district = district_gate && district_pending;
        assert_eq!(
            observed,
            (
                rule_gate && rule_wait,
                district,
                realm_wait && !district,
                keep_list
            )
        );
    }
    env.set_screen_mode(ScreenKind::Login);
    env.exec(
        r#"
        local s = C_Login.GetState()
        local count = 0
        for _ in pairs(s) do count = count + 1 end
        assert(count == 9 and select('#', C_Login.GetState()) == 1)
        assert(s.auroraState == 1 and not s.connectedToWoW and s.wowConnectionState == 1)
        assert(not s.hasRealmList and not s.waitingForRealmList and not s.waitingForRuleSets)
        assert(not s.superDistrictChoicePending and not s.keepingCharacterListOnRealmJoin)
        assert(s.hasSelectedExperiencePreset)
    "#,
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn glue_character_selection_uses_resolved_slots_and_native_events() {
    use wow_ui_sim::c_api::c_login::character_roster::CharacterSlot;
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.set_screen_mode(ScreenKind::Login);
    env.exec(
        r#"
        selectionEvents = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('UPDATE_SELECTED_CHARACTER')
        listener:SetScript('OnEvent', function(_, event, ...)
            table.insert(selectionEvents, {select('#', ...), ..., GetCharacterSelection()})
        end)
        selectionPassword = CreateFrame('EditBox')
        C_Login.Login('offline', selectionPassword)
        assert(GetCharacterSelection() == 0)
        assert(select('#', SelectCharacter(1)) == 0)
        assert(#selectionEvents == 0 and GetNumCharacters() == 0)
    "#,
    )
    .unwrap();
    let fixture = env
        .state()
        .borrow()
        .bnet_friends
        .iter()
        .flat_map(|friend| &friend.game_accounts)
        .find(|account| account.client_program == "WoW")
        .unwrap()
        .clone();
    let mut second = fixture.clone();
    second.wow_account_guid = "host-character-two".into();
    env.state().borrow_mut().glue_login.characters = vec![
        fixture.into(),
        second.into(),
        CharacterSlot::default(),
        CharacterSlot {
            character_id: "host-unresolved".into(),
            record: None,
        },
    ];
    env.exec(
        r#"
        assert(GetNumCharacters() == 4)
        assert(GetCharacterSelection() == 0)
        assert(select('#', SelectCharacter(2.9)) == 0)
        assert(GetCharacterSelection() == 2)
        SelectCharacter(0.5)
        assert(GetCharacterSelection() == 1 and #selectionEvents == 2)
        assert(GetCharacterSelection() == 1)
        SelectCharacter(-9)
        SelectCharacter(99)
        assert(#selectionEvents == 2)
        SelectCharacter(2)
        SelectCharacter(-9)
        assert(GetCharacterSelection() == 1 and #selectionEvents == 4)
        SelectCharacter(2)
        SelectCharacter(3) -- Empty ID: no search and no change.
        SelectCharacter(4) -- Unresolved nonzero ID: no search and no change.
        assert(GetCharacterSelection() == 2 and #selectionEvents == 5)
        SelectCharacter(99)
        assert(GetCharacterSelection() == 1 and #selectionEvents == 6)
        for i, expected in ipairs({2, 1, 2, 1, 2, 1}) do
            assert(selectionEvents[i][1] == 1)
            assert(selectionEvents[i][2] == expected and selectionEvents[i][3] == expected)
        end
        local function bad(...)
            local ok, message = pcall(SelectCharacter, ...)
            assert(not ok and message:find('Usage: SelectCharacter(index)', 1, true))
        end
        bad(); bad(false); bad({}); bad('not a number')
        assert(#selectionEvents == 6 and GetCharacterSelection() == 1)
    "#,
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn glue_character_list_resets_selection_and_resolves_only_host_candidate() {
    use wow_ui_sim::c_api::c_login::character_roster::{CharacterSlot, resolve_saved_candidate};
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.set_screen_mode(ScreenKind::Login);
    let fixture = env
        .state()
        .borrow()
        .bnet_friends
        .iter()
        .flat_map(|friend| &friend.game_accounts)
        .find(|account| account.client_program == "WoW")
        .unwrap()
        .clone();
    let valid = CharacterSlot::from(fixture.clone());
    let alternatives = vec![valid.clone(), CharacterSlot::default(), valid.clone()];
    assert_eq!(resolve_saved_candidate(&alternatives, Some(2)), Some(2));
    assert_eq!(resolve_saved_candidate(&alternatives, Some(1)), Some(2));
    assert_eq!(resolve_saved_candidate(&alternatives, Some(99)), Some(0));
    let mut mismatched = valid;
    mismatched.character_id = "different-record-id".into();
    assert!(mismatched.resolved_record().is_none());
    let slots = vec![
        CharacterSlot::default(),
        fixture.into(),
        CharacterSlot {
            character_id: "unresolved".into(),
            record: None,
        },
    ];
    assert_eq!(resolve_saved_candidate(&[], None), None);
    for candidate in [None, Some(-1), Some(0), Some(1), Some(2), Some(99)] {
        assert_eq!(resolve_saved_candidate(&slots, candidate), Some(1));
    }
    let unresolved = vec![CharacterSlot::default(), CharacterSlot::default()];
    assert_eq!(resolve_saved_candidate(&unresolved, Some(1)), Some(0));
    {
        let mut sim = env.state().borrow_mut();
        sim.glue_login.characters = slots;
        sim.glue_login.saved_character_index = Some(2);
        sim.glue_login.connection_info = wow_ui_sim::c_api::c_login::ConnectionInfo {
            ready: true,
            character_limit: 2,
        };
    }
    env.exec(
        r#"
        local password = CreateFrame('EditBox')
        listSelection = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('CHARACTER_LIST_UPDATE')
        listener:SetScript('OnEvent', function(_, event, ...)
            table.insert(listSelection, {select('#', ...), ..., GetCharacterSelection()})
        end)
        C_Login.Login('offline', password)
        SelectCharacter(2)
        assert(GetCharacterSelection() == 2)
    "#,
    )
    .unwrap();
    wow_ui_sim::startup::run_extra_update_ticks(&env, 3);
    env.exec(
        r#"
        assert(#listSelection == 1)
        assert(listSelection[1][1] == 1 and listSelection[1][2] == 3)
        assert(listSelection[1][3] == 0 and GetCharacterSelection() == 0)
        assert(GetCharacterGUID(3) == 'unresolved')
        assert(not CanCreateCharacter()) -- Nonzero unresolved IDs also consume capacity.
    "#,
    )
    .unwrap();
    let sim = env.state().borrow();
    assert_eq!(sim.glue_login.saved_character_index, Some(2));
    assert_eq!(sim.glue_login.resolved_saved_character_index, Some(1));
    assert!(sim.lua_errors.is_empty());
}

#[test]
#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
fn glue_login_host_roster_does_not_borrow_social_accounts_before_submission() {
    use wow_ui_sim::c_api::c_login::character_roster::CharacterSlot;
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    let direct_count = env.eval::<i32>("return GetNumCharacters()").unwrap();
    assert!(
        direct_count > 0,
        "direct-screen fixture must be nonempty for this contrast"
    );
    let fixture = env
        .state()
        .borrow()
        .bnet_friends
        .iter()
        .flat_map(|friend| &friend.game_accounts)
        .find(|account| account.client_program == "WoW")
        .unwrap()
        .clone();
    let guid = fixture.wow_account_guid.clone();
    env.set_screen_mode(ScreenKind::Login);
    env.exec(r#"
        assert(GetNumCharacters() == 0 and GetCharacterSelection() == 0)
        preloginSelectionEvents = 0
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('UPDATE_SELECTED_CHARACTER')
        listener:SetScript('OnEvent', function() preloginSelectionEvents = preloginSelectionEvents + 1 end)
    "#).unwrap();
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    env.exec(
        "SelectCharacter(1); assert(GetCharacterSelection() == 0 and preloginSelectionEvents == 0)",
    )
    .unwrap();
    let host_count = direct_count + 2;
    let mut slots = vec![CharacterSlot::default(); host_count as usize];
    slots[host_count as usize - 1] = fixture.into();
    env.state().borrow_mut().glue_login.characters = slots;
    assert_eq!(
        env.eval::<i32>("return GetNumCharacters()").unwrap(),
        host_count
    );
    assert_eq!(
        env.eval::<String>(&format!("return GetCharacterGUID({host_count})"))
            .unwrap(),
        guid
    );
    for screen in [
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
        ScreenKind::Game,
    ] {
        env.set_screen_mode(screen);
        assert_eq!(
            env.eval::<i32>("return GetNumCharacters()").unwrap(),
            direct_count
        );
    }
    env.set_screen_mode(ScreenKind::Login);
    assert_eq!(
        env.eval::<i32>("return GetNumCharacters()").unwrap(),
        host_count
    );
    env.state().borrow_mut().glue_login.characters.clear();
    assert_eq!(env.eval::<i32>("return GetNumCharacters()").unwrap(), 0);
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
#[cfg(any(feature = "client-ptr", feature = "client-wowforever"))]
fn glue_experience_preset_reads_real_cvar_numeric_prefix() {
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.set_screen_mode(ScreenKind::Login);
    let default =
        if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Ptr {
            "1"
        } else {
            "0"
        };
    assert_eq!(
        env.eval::<String>("return GetCVarDefault('hasSelectedExperiencePreset')")
            .unwrap(),
        default
    );
    assert_eq!(
        env.eval::<bool>("return C_Login.GetState().hasSelectedExperiencePreset")
            .unwrap(),
        default == "1"
    );
    env.exec(
        r#"
        SetCVar('hasSelectedExperiencePreset', '0')
        presetLoginEvents = 0
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('CVAR_UPDATE')
        listener:RegisterEvent('LOGIN_STATE_CHANGED')
        listener:SetScript('OnEvent', function(_, event, ...)
            if event == 'LOGIN_STATE_CHANGED' then
                presetLoginEvents = presetLoginEvents + 1
            else
                local name, value = ...
                if name == 'hasSelectedExperiencePreset' then
                    assert(select('#', ...) == 2)
                    presetObservedValue = value
                    presetObservedState = C_Login.GetState().hasSelectedExperiencePreset
                end
            end
        end)
    "#,
    )
    .unwrap();
    for (value, expected) in [
        ("1", true),
        ("0", false),
        ("true", false),
        ("false", false),
        ("junk", false),
        ("1junk", true),
        ("0x10", true),
        ("0x0junk", false),
        ("-1", true),
        ("-0", false),
        (" 8", true),
        ("08", true),
        ("8junk", true),
        ("+0x10", false),
        ("-0x10", false),
        (" 0x10", false),
        ("+ 1", false),
    ] {
        env.exec(&format!(
            "SetCVar('hasSelectedExperiencePreset', {value:?})"
        ))
        .unwrap();
        assert_eq!(
            env.eval::<String>("return GetCVar('hasSelectedExperiencePreset')")
                .unwrap(),
            value
        );
        assert_eq!(
            env.eval::<bool>("return C_Login.GetState().hasSelectedExperiencePreset")
                .unwrap(),
            expected
        );
        assert_eq!(
            env.eval::<String>("return presetObservedValue").unwrap(),
            value
        );
        assert_eq!(
            env.eval::<bool>("return presetObservedState").unwrap(),
            expected
        );
    }
    assert_eq!(
        env.eval::<i32>("return presetLoginEvents").unwrap(),
        0,
        "generic CVar writes must not synthesize a login-state event"
    );
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn glue_character_timerunning_season_uses_requested_record_without_active_fallback() {
    use wow_ui_sim::c_api::c_login::character_roster::CharacterSlot;
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    assert!(
        env.eval::<bool>("return rawget(_G, 'GetCharacterTimerunningSeasonID') == nil")
            .unwrap()
    );
    env.set_screen_mode(ScreenKind::Login);
    let fixture = env
        .state()
        .borrow()
        .bnet_friends
        .iter()
        .flat_map(|friend| &friend.game_accounts)
        .find(|account| account.client_program == "WoW")
        .unwrap()
        .clone();
    let mut slots = Vec::new();
    for (guid, season) in [
        ("host-one", Some(7)),
        ("host-two", Some(12)),
        ("host-zero", Some(0)),
        ("host-default", None),
        // Opaque host identity tests numeric-to-string conversion, not native GUID grammar.
        ("123", Some(13)),
    ] {
        let mut account = fixture.clone();
        account.wow_account_guid = guid.into();
        let mut slot = CharacterSlot::from(account);
        slot.record.as_mut().unwrap().timerunning_season_id = season;
        slots.push(slot);
    }
    slots.push(CharacterSlot {
        character_id: "host-unresolved".into(),
        record: None,
    });
    {
        let mut sim = env.state().borrow_mut();
        sim.glue_login.characters = slots;
        sim.timerunning_season_id = Some(99);
    }
    env.exec(r#"
        SelectCharacter(1)
        assert(GetCharacterTimerunningSeasonID('host-one') == 7)
        assert(GetCharacterTimerunningSeasonID('host-two', 'ignored') == 12)
        assert(select('#', GetCharacterTimerunningSeasonID('host-two')) == 1)
        for _, guid in ipairs({'host-zero', 'host-default', 'host-unresolved', 'unknown'}) do
            assert(GetCharacterTimerunningSeasonID(guid) == nil)
            assert(select('#', GetCharacterTimerunningSeasonID(guid)) == 1)
        end
        assert(GetCharacterTimerunningSeasonID(1) == nil) -- GUID conversion, not slot index.
        assert(GetCharacterTimerunningSeasonID(123) == 13)
        assert(GetActiveTimerunningSeasonID() == 99)
        local function bad(...)
            local ok, message = pcall(GetCharacterTimerunningSeasonID, ...)
            assert(not ok and message:find('Usage: local timerunningSeasonID = GetCharacterTimerunningSeasonID(guid)', 1, true))
        end
        bad(); bad(nil); bad(false); bad({})
    "#).unwrap();
    env.set_screen_mode(ScreenKind::Game);
    assert!(
        env.eval::<bool>("return rawget(_G, 'GetCharacterTimerunningSeasonID') == nil")
            .unwrap()
    );
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
#[cfg(any(feature = "client-ptr", feature = "client-wowforever"))]
fn glue_experience_preset_setter_orders_native_cvar_and_login_events() {
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.set_screen_mode(ScreenKind::Login);
    env.exec(r#"
        SetCVar('hasSelectedExperiencePreset', '0')
        presetEvents = {}
        local f = CreateFrame('Frame')
        for _, event in ipairs({'CVAR_UPDATE', 'LOGIN_STATE_CHANGED', 'GAME_RULES_CHANGED'}) do f:RegisterEvent(event) end
        f:SetScript('OnEvent', function(_, event, ...)
            table.insert(presetEvents, event)
            assert(C_Login.GetState().hasSelectedExperiencePreset)
            if event == 'CVAR_UPDATE' then
                assert(select('#', ...) == 2)
                local name, value = ...
                assert(name == 'hasSelectedExperiencePreset' and value == '1')
            else assert(select('#', ...) == 0) end
        end)
        assert(Enum.ForeverExperiencePreset.Classic == 0 and Enum.ForeverExperiencePreset.Enhanced == 1)
        assert(select('#', C_GameRules.SetForeverExperiencePresetPreLogin(257)) == 0)
        assert(table.concat(presetEvents, ',') == 'CVAR_UPDATE,LOGIN_STATE_CHANGED')
        presetEvents = {}
        C_GameRules.SetForeverExperiencePresetPreLogin(1)
        assert(table.concat(presetEvents, ',') == 'LOGIN_STATE_CHANGED')
    "#).unwrap();
    assert!(env.state().borrow().game_rules.experience_preset_changed);
    assert_eq!(
        env.state().borrow().game_rules.classic_experience_chosen,
        Some(false)
    );
    for (input, classic) in [
        ("256", true),
        ("0.9", true),
        ("1.9", false),
        ("'257'", false),
        ("true", true),
        ("'junk'", true),
    ] {
        env.exec(&format!("presetEvents = {{}}; C_GameRules.SetForeverExperiencePresetPreLogin({input}); assert(table.concat(presetEvents, ',') == 'LOGIN_STATE_CHANGED')")).unwrap();
        assert_eq!(
            env.state().borrow().game_rules.classic_experience_chosen,
            Some(classic)
        );
    }
    env.exec(r#"
        for _, input in ipairs({2, 255, 258, 2147483648, -2147483649}) do
            presetEvents = {}
            assert(not pcall(C_GameRules.SetForeverExperiencePresetPreLogin, input))
            assert(#presetEvents == 0)
        end
        local ok, message = pcall(C_GameRules.SetForeverExperiencePresetPreLogin)
        assert(not ok and message:find('Usage: C_GameRules.SetForeverExperiencePresetPreLogin(preset)', 1, true))
    "#).unwrap();
    assert_eq!(
        env.state().borrow().game_rules.classic_experience_chosen,
        Some(true)
    );
    assert!(env.state().borrow().lua_errors.is_empty());
    env.set_screen_mode(ScreenKind::Game);
    assert!(
        env.eval::<bool>(
            "return type(C_GameRules.SetForeverExperiencePresetPreLogin) == 'function'"
        )
        .unwrap()
    );
}

#[test]
#[cfg(feature = "client-ptr")]
fn glue_preset_login_notification_survives_synchronous_cvar_callback_error() {
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.set_screen_mode(ScreenKind::Login);
    env.exec(r#"
        SetCVar('hasSelectedExperiencePreset', '0')
        local callback = C_FunctionContainers.CreateCallback(function() error('preset callback probe') end)
        RegisterEventCallback('CVAR_UPDATE', callback)
        loginNotified = 0
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('LOGIN_STATE_CHANGED')
        frame:SetScript('OnEvent', function() loginNotified = loginNotified + 1 end)
        local ok, message = pcall(C_GameRules.SetForeverExperiencePresetPreLogin, 1)
        assert(not ok and message:find('preset callback probe', 1, true))
        assert(loginNotified == 1)
        assert(GetCVar('hasSelectedExperiencePreset') == '1')
        UnregisterEventCallback('CVAR_UPDATE', callback)
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
fn glue_login_settling_does_not_force_direct_character_select_refresh() {
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.set_screen_mode(ScreenKind::Login);
    env.exec(
        r#"
        forcedSelectLoads, forcedSelectShows, forcedListRequests = 0, 0, 0
        CharacterSelect = {
            OnLoad = function() forcedSelectLoads = forcedSelectLoads + 1 end,
            OnShow = function() forcedSelectShows = forcedSelectShows + 1 end,
        }
        CharacterSelectListUtil = {
            GetCharacterListUpdate = function() forcedListRequests = forcedListRequests + 1 end,
        }
    "#,
    )
    .unwrap();
    settle_headless_startup(&env);
    let observed: (i32, i32, i32) = env
        .eval("return forcedSelectLoads, forcedSelectShows, forcedListRequests")
        .unwrap();
    assert_eq!(observed, (0, 0, 0));
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[cfg(feature = "client-retail")]
#[test]
fn glue_retail_chat_source_preserves_r62_verified_asset_facts() {
    let ui = blizzard_ui_dir();
    let xml =
        std::fs::read_to_string(ui.join("Blizzard_ChatFrameBase/Mainline/FloatingChatFrame.xml"))
            .unwrap();
    let declaration = xml
        .lines()
        .find(|line| line.contains("name=\"ChatFrame1\""))
        .unwrap();
    assert!(declaration.contains("hidden=\"true\""));
    let chat =
        std::fs::read_to_string(ui.join("Blizzard_ChatFrameBase/Mainline/FloatingChatFrame.lua"))
            .unwrap();
    assert!(chat.contains("frame:SetShown(AllowChatFramesToShow(frame))"));
    let parent =
        std::fs::read_to_string(ui.join("Blizzard_GlueParent/Mainline/GlueParent.lua")).unwrap();
    let login = parent
        .lines()
        .find(|line| line.contains("[\"login\"]"))
        .unwrap();
    assert!(login.contains("AccountLogin") && !login.contains("allowChat"));
    let policy = parent
        .split("function AllowChatFramesToShow(chatFrame)")
        .nth(1)
        .unwrap()
        .split("\nend")
        .next()
        .unwrap();
    assert!(policy.contains("if chatFrame and info and info.allowChat then"));
    assert!(policy.contains("return false;"));
    // Static source facts do not establish the live winner or visible pixels.
}
