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

        let chat_frame_visible: bool = env
            .eval("return ChatFrame1 ~= nil and ChatFrame1:IsShown()")
            .expect("ChatFrame1 visibility should be queryable");
        assert!(
            !chat_frame_visible,
            "plain login screen should not show the front-end chat frame"
        );

        let chat_dock_visible: bool = env
            .eval("return GeneralDockManager ~= nil and GeneralDockManager:IsShown()")
            .expect("GeneralDockManager visibility should be queryable");
        assert!(
            !chat_dock_visible,
            "plain login screen should not show the chat dock"
        );

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
fn submit_login_through_ui(click_button: bool) {
    let env = load_blizzard_screen(ScreenKind::Login);
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
    env.state().borrow_mut().glue_login.characters = vec![fixture];
    for (path, text) in [
        ("AccountLogin.UI.AccountEditBox", "sim@example.invalid"),
        ("AccountLogin.UI.PasswordEditBox", "arbitrary-password"),
    ] {
        let hit = hit_test_like_gui(&env, frame_center(&env, path))
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
        let hit = hit_test_like_gui(&env, frame_center(&env, "AccountLogin.UI.LoginButton"))
            .expect("login button center should be hittable");
        env.send_click(hit)
            .expect("login button click should dispatch");
    } else {
        env.send_key_press("ENTER", None)
            .expect("Enter should submit the password edit box");
    }
    wow_ui_sim::startup::run_extra_update_ticks(&env, 3);
    let current: String = env
        .eval("return tostring(GlueParent.currentScreen)")
        .unwrap();
    let best: String = env
        .eval("return tostring(GlueParent_GetBestScreen())")
        .unwrap();
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
            {"name", CreateFrame("Frame")}}) do
            local ok, err = pcall(C_Login.Login, unpack(args))
            assert(not ok and err == usage, tostring(err))
        end
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
                    assert(a.hasSelectedExperiencePreset and not a.keepingCharacterListOnRealmJoin)
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
        sim.glue_login.characters = vec![fixture];
        sim.glue_login.realm_list_available = true;
    }
    env.exec(
        r#"
        policyListUpdates = 0
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("CHARACTER_LIST_UPDATE")
        listener:SetScript("OnEvent", function(_, _, count)
            assert(count == 1)
            policyListUpdates = policyListUpdates + 1
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
    env.exec(
        r#"
        C_Login.Login("replacement-one", policyPasswordBox)
        C_Login.Login("replacement-two", policyPasswordBox)
    "#,
    )
    .unwrap();
    for _ in 0..10 {
        env.fire_on_update(0.016).unwrap();
    }
    assert_eq!(env.eval::<i32>("return policyListUpdates").unwrap(), 1);
    assert_eq!(env.eval::<i32>("return GetNumCharacters()").unwrap(), 1);
}
