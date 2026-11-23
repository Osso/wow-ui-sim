//! R88ad.7-10 Glue state through the original Retail Glue consumers.
//! Evidence labels: docs/specs/glue-state-r88ad.md.
#![cfg(all(feature = "gui", feature = "client-retail"))]

use crate::common;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::globals::real::glue_state::{
    GlueAudioHandle, GlueAudioRequest, GlueServerState, update_minimum_expansion_level,
};
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::settle_headless_startup;

fn login_env() -> common::LockedEnv {
    common::lock_env(|| {
        let env = WowLuaEnv::new().expect("Lua environment");
        env.set_screen_size(1024.0, 768.0);
        env.set_screen_mode(ScreenKind::Login);
        let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(
            env!("CARGO_MANIFEST_DIR"),
        ));
        for (name, toc) in discover_blizzard_addons_for_screen(&ui, ScreenKind::Login) {
            load_addon(&env.loader_env(), &toc)
                .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
        }
        env.apply_post_load_workarounds();
        settle_headless_startup(&env);
        env
    })
}

/// Run a statement chunk; the leading statement keeps `eval` from treating a
/// one-line chunk as an expression.
fn lua(env: &WowLuaEnv, code: &str) -> String {
    env.eval::<String>(&format!("local _ = nil\n{code}"))
        .unwrap_or_else(|err| panic!("{code}\n=> {err}"))
}

fn lua_error(env: &WowLuaEnv, code: &str) -> String {
    match env.exec(code) {
        Ok(()) => panic!("{code} should raise an error"),
        Err(err) => err.to_string(),
    }
}

fn seed_setting(env: &WowLuaEnv, name: &str, value: &str) {
    env.state().borrow().cvars.set_imported(name, value);
}

fn setting(env: &WowLuaEnv, name: &str) -> Option<String> {
    env.state().borrow().cvars.get(name)
}

fn audio_requests(env: &WowLuaEnv) -> Vec<GlueAudioRequest> {
    std::mem::take(&mut env.state().borrow_mut().glue_state.audio.requests)
}

fn music_requests(env: &WowLuaEnv) -> Vec<GlueAudioRequest> {
    audio_requests(env)
        .into_iter()
        .filter(|request| {
            matches!(
                request,
                GlueAudioRequest::StartMusic { .. } | GlueAudioRequest::StopMusic { .. }
            )
        })
        .collect()
}

// R88ad.7: GetSavedAccountName reads `accountName`, or the empty fallback.
const FILL_ACCOUNT: &str = r#"
    AccountLogin_UpdateSavedData(AccountLogin)
    return AccountLogin.UI.AccountEditBox:GetText()
"#;

#[test]
fn saved_account_name_fills_account_login_edit_box() {
    test_timeout! {
        let env = login_env();
        lua(&env, "AccountLogin.UI.AccountEditBox:SetText('') return ''");
        assert_eq!(lua(&env, FILL_ACCOUNT), "", "absent setting");
        seed_setting(&env, "accountName", "alpha");
        assert_eq!(lua(&env, FILL_ACCOUNT), "alpha");
        seed_setting(&env, "accountName", "beta");
        assert_eq!(lua(&env, FILL_ACCOUNT), "beta");
    }
}

fn login_as(env: &WowLuaEnv, account: &str, save_name: bool) {
    lua(
        env,
        &format!(
            "local ui = AccountLogin.UI
        ui.AccountEditBox:SetText({account:?})
        ui.PasswordEditBox:SetText('pw')
        ui.SaveAccountNameCheckButton:SetControlChecked({save_name})
        AccountLogin_Login()
        return ''"
        ),
    );
}

// R88ad.7: SetSavedAccountName and SetUsesToken write only present settings.
#[test]
fn account_login_writes_saved_account_settings() {
    test_timeout! {
        let env = login_env();
        login_as(&env, "zeta", true);
        login_as(&env, "zeta", false);
        assert_eq!(setting(&env, "accountName"), None, "absent name setting");
        assert_eq!(setting(&env, "g_accountUsesToken"), None, "absent token setting");
        seed_setting(&env, "accountName", "seed");
        seed_setting(&env, "g_accountUsesToken", "1");
        login_as(&env, "alpha", true);
        assert_eq!(setting(&env, "accountName").as_deref(), Some("alpha"));
        login_as(&env, "beta", true);
        assert_eq!(setting(&env, "accountName").as_deref(), Some("beta"));
        assert_eq!(setting(&env, "g_accountUsesToken").as_deref(), Some("1"));
        login_as(&env, "beta", false);
        assert_eq!(setting(&env, "accountName").as_deref(), Some(""));
        assert_eq!(setting(&env, "g_accountUsesToken").as_deref(), Some("0"));
    }
}

/// Edit the account field; OnTextChanged is delivered before the script returns to host.
fn edit_account(env: &WowLuaEnv, text: &str) {
    lua(
        env,
        &format!("AccountLogin.UI.AccountEditBox:SetText({text:?}) return ''"),
    );
}

// R88ad.7: ClearSavedAccountList empties a present `accountList` setting only.
#[test]
fn edited_account_name_clears_saved_account_list() {
    test_timeout! {
        let env = login_env();
        seed_setting(&env, "accountName", "alpha");
        edit_account(&env, "edited");
        assert_eq!(setting(&env, "accountName").as_deref(), Some(""));
        assert_eq!(setting(&env, "accountList"), None, "absent list setting");
        for (name, list) in [("alpha", "!alpha|beta"), ("delta", "delta|!omega")] {
            seed_setting(&env, "accountName", name);
            seed_setting(&env, "accountList", list);
            edit_account(&env, &format!("edited-{name}"));
            assert_eq!(setting(&env, "accountName").as_deref(), Some(""), "{name}");
            assert_eq!(setting(&env, "accountList").as_deref(), Some(""), "{list}");
        }
    }
}

const SELECTED_ACCOUNT: &str = r#"
    AccountLoginDropdown_SetupList()
    return tostring((AccountLogin_GetSelectedAccountString()))
"#;

// R88ad.7: GetSavedAccountList publishes the `accountList` setting text.
#[test]
fn saved_account_list_selects_dropdown_account() {
    test_timeout! {
        let env = login_env();
        assert_eq!(lua(&env, SELECTED_ACCOUNT), "nil", "absent list setting");
        seed_setting(&env, "accountList", "alpha|!beta");
        assert_eq!(lua(&env, SELECTED_ACCOUNT), "beta");
        seed_setting(&env, "accountList", "!gamma|delta");
        assert_eq!(lua(&env, SELECTED_ACCOUNT), "gamma");
    }
}

const REALM_TEXT: &str = r#"
    return table.concat({
        CharacterSelectUtil.GetFormattedCurrentRealmName(),
        tostring(CharacterSelectUtil.IsSameRealmAsCurrent(101)),
        tostring(select('#', GetServerName())),
    }, "|")
"#;

fn set_server(env: &WowLuaEnv, server: GlueServerState) {
    env.state().borrow_mut().glue_state.server = server;
}

// R88ad.8: GetServerName publishes zero values or the five-value tuple.
#[test]
fn server_name_formats_character_select_realm() {
    test_timeout! {
        let env = login_env();
        assert_eq!(lua(&env, REALM_TEXT), "|nil|0", "unavailable connection");
        set_server(&env, GlueServerState {
            available: true,
            name: Some("Alpha".into()),
            config: Some((false, true)),
            additional_connection_flag: false,
            realm_number: Some(101),
        });
        let rp = lua(&env, "return RP_PARENTHESES");
        assert_eq!(lua(&env, REALM_TEXT), format!("Alpha {rp}|true|5"));
        set_server(&env, GlueServerState {
            available: true,
            name: Some("Beta".into()),
            ..Default::default()
        });
        assert_eq!(lua(&env, REALM_TEXT), "Beta|nil|5");
    }
}

const POPUP_AFTER_LOGIN_STATE: &str = r#"
    if not TimerunningChoicePopup then
        C_AddOns.LoadAddOn("Blizzard_TimerunningCharacterCreate")
    end
    TimerunningChoicePopup:Show()
    TimerunningChoicePopupMixin.OnEvent(TimerunningChoicePopup, "LOGIN_STATE_CHANGED")
    return tostring(TimerunningChoicePopup:IsShown())
"#;

// R88ad.8: IsConnectedToServer reads the shared connection-availability flag.
#[test]
fn connection_flag_keeps_timerunning_choice_popup() {
    test_timeout! {
        let env = login_env();
        assert_eq!(lua(&env, POPUP_AFTER_LOGIN_STATE), "false", "unavailable");
        for name in ["Alpha", "Beta"] {
            set_server(&env, GlueServerState {
                available: true,
                name: Some(name.into()),
                ..Default::default()
            });
            assert_eq!(lua(&env, POPUP_AFTER_LOGIN_STATE), "true", "{name}");
        }
    }
}

const UPGRADE_LEVELS: &str = r#"
    local current, upgrade = AccountUpgradePanel_GetDisplayExpansionLevel()
    return tostring(current) .. "|" .. tostring(upgrade)
"#;

// R88ad.8: GetMinimumExpansionLevel publishes the shared state; the update
// routine stores a differing value before MIN_EXPANSION_LEVEL_UPDATED.
#[test]
fn minimum_expansion_level_selects_upgrade_panel_levels() {
    test_timeout! {
        let env = login_env();
        let absent = lua(&env, UPGRADE_LEVELS);
        lua(&env, "GLUE_STATE_MIN_EVENTS = 0
            local f = CreateFrame('Frame')
            f:RegisterEvent('MIN_EXPANSION_LEVEL_UPDATED')
            f:SetScript('OnEvent', function() GLUE_STATE_MIN_EVENTS = GLUE_STATE_MIN_EVENTS + 1 end)
            return ''");
        let mut seen = vec![absent];
        for level in [3, 10, 10] {
            update_minimum_expansion_level(&env, level).expect("update");
            seen.push(lua(&env, UPGRADE_LEVELS));
        }
        let current = lua(&env, "return tostring(GetClampedCurrentExpansionLevel())");
        assert!(current.parse::<u32>().expect("current level") > 3, "{current}");
        let kept = format!("{current}|");
        assert!(seen[0].starts_with(&kept), "empty host input: {seen:?}");
        assert!(seen[1].starts_with(&kept), "level 3 keeps {current}: {seen:?}");
        assert!(seen[2].starts_with("0|"), "level 10 selects Classic: {seen:?}");
        assert_eq!(seen[3], seen[2]);
        assert_eq!(lua(&env, "return tostring(GLUE_STATE_MIN_EVENTS)"), "2");
    }
}

const KOREAN_RATINGS_ON_LOAD: &str = r#"
    KoreanRatings:SetScript("OnUpdate", nil)
    KoreanRatings:UnregisterEvent("SCREEN_FIRST_DISPLAYED")
    KoreanRatingsMixin.OnLoad(KoreanRatings)
    return tostring(KoreanRatings:GetScript("OnUpdate") == KoreanRatingsMixin.OnUpdate)
        .. "|" .. tostring(KoreanRatings:IsEventRegistered("SCREEN_FIRST_DISPLAYED"))
"#;

// R88ad.8: WasScreenFirstDisplayed is true exactly at display state 2.
#[test]
fn screen_first_display_state_selects_korean_ratings_path() {
    test_timeout! {
        let env = login_env();
        for (display_state, expected) in [(0, "false|true"), (1, "false|true"), (2, "true|false")] {
            env.state().borrow_mut().screen_first_display_state = display_state;
            assert_eq!(lua(&env, KOREAN_RATINGS_ON_LOAD), expected, "state {display_state}");
        }
    }
}

const LOGIN_MODEL: &str = r#"
    if not GLUE_STATE_MODEL_HOOKED then
        GLUE_STATE_MODEL_HOOKED = true
        hooksecurefunc(LoginBackgroundModel, "SetModel", function(_, file)
            GLUE_STATE_MODEL_FILE = file
        end)
    end
    GetExpansionDisplayInfo = function()
        return { highResBackgroundID = 4210001, lowResBackgroundID = 4210002 }
    end
    SetLoginScreenModel(LoginBackgroundModel)
    return tostring(GLUE_STATE_MODEL_FILE)
"#;

// R88ad.9: an accepted high-resolution ID wins; otherwise the low one.
// The expansion display record is a fixture for the unmodelled catalog, and the
// hook only records the original consumer's SetModel argument.
#[test]
fn login_background_objects_select_login_model() {
    test_timeout! {
        let env = login_env();
        assert_eq!(lua(&env, LOGIN_MODEL), "4210002", "no background objects");
        for (accepted, expected) in [(4210001, "4210001"), (4219999, "4210002")] {
            env.state().borrow_mut().glue_state.login_background_file_ids = [accepted].into();
            assert_eq!(lua(&env, LOGIN_MODEL), expected, "accepted {accepted}");
        }
    }
}

const RESET_MODEL: &str =
    "ResetModel(LoginBackgroundModel) return tostring(SOUNDKIT.GLUESCREEN_INTRO)";

fn set_ambience(env: &WowLuaEnv, handle: GlueAudioHandle) {
    env.state().borrow_mut().glue_state.audio.ambience = handle;
}

// R88ad.10: PlayGlueAmbience replacement and fade conversion.
#[test]
fn reset_model_requests_glue_ambience() {
    test_timeout! {
        let env = login_env();
        audio_requests(&env);
        let kit: i32 = lua(&env, RESET_MODEL).parse().expect("intro kit");
        let first = audio_requests(&env);
        let [GlueAudioRequest::StartAmbience { kit: started, fade_ms: 4000, handle }] = first[..] else {
            panic!("absent ambience: {first:?}");
        };
        assert_eq!((started, env.state().borrow().glue_state.audio.requested_ambience_kit), (kit, kit));
        set_ambience(&env, GlueAudioHandle { id: handle, kit, active: true });
        lua(&env, RESET_MODEL);
        assert_eq!(audio_requests(&env), [], "active same kit");
        set_ambience(&env, GlueAudioHandle { id: 77, kit: kit + 1, active: true });
        lua(&env, RESET_MODEL);
        let replaced = audio_requests(&env);
        assert!(matches!(replaced[..], [GlueAudioRequest::StopAmbience { handle: 77 },
            GlueAudioRequest::StartAmbience { kit: k, fade_ms: 4000, .. }] if k == kit), "{replaced:?}");
    }
}

const SET_LOGIN_SCREEN: &str = r#"
    GlueParent_SetScreen("login")
    return tostring(SOUNDKIT.PLUNDERSTORM_QUEUE_SCREEN_MUSIC)
"#;

// R88ad.10: PlayGlueMusic retains positive kits and replaces differing handles.
#[test]
fn login_screen_change_requests_plunderstorm_music() {
    test_timeout! {
        let env = login_env();
        audio_requests(&env);
        lua(&env, SET_LOGIN_SCREEN);
        assert_eq!(music_requests(&env), [], "standard mode");
        env.state().borrow_mut().game_rules.active_game_mode = 2;
        let kit: i32 = lua(&env, SET_LOGIN_SCREEN).parse().expect("music kit");
        let first = music_requests(&env);
        assert!(matches!(first[..], [GlueAudioRequest::StopMusic { handle: 0 },
            GlueAudioRequest::StartMusic { kit: k, .. }] if k == kit), "{first:?}");
        assert_eq!(env.state().borrow().glue_state.audio.requested_music_kit, kit);
        env.state().borrow_mut().glue_state.audio.music = GlueAudioHandle { id: 5, kit, active: true };
        lua(&env, SET_LOGIN_SCREEN);
        assert_eq!(music_requests(&env), [], "active same kit");
        env.state().borrow_mut().glue_state.audio.music = GlueAudioHandle { id: 6, kit: kit + 1, active: true };
        lua(&env, SET_LOGIN_SCREEN);
        let replaced = music_requests(&env);
        assert!(matches!(replaced[..], [GlueAudioRequest::StopMusic { handle: 6 },
            GlueAudioRequest::StopMusic { handle: 0 },
            GlueAudioRequest::StartMusic { kit: k, .. }] if k == kit), "{replaced:?}");
    }
}

const PHOTOSENSITIVITY_TRY_SHOW: &str = "PhotosensitivityWarningFrame:TryShow() return ''";

fn seed_audio(env: &WowLuaEnv, music: (u32, i32), ambience: (u32, i32)) {
    let mut sim = env.state().borrow_mut();
    let audio = &mut sim.glue_state.audio;
    audio.music = GlueAudioHandle {
        id: music.0,
        kit: music.1,
        active: true,
    };
    audio.requested_music_kit = music.1;
    audio.music_stop_markers = [false; 2];
    audio.ambience = GlueAudioHandle {
        id: ambience.0,
        kit: ambience.1,
        active: true,
    };
    audio.requested_ambience_kit = ambience.1;
    audio.ambience_stop_marker = false;
}

// R88ad.10: StopGlueMusic/StopGlueAmbience mark and stop stored handles but
// keep the retained kits.
#[test]
fn photosensitivity_warning_stops_glue_audio() {
    test_timeout! {
        let env = login_env();
        for (music, ambience) in [((0, 0), (0, 0)), ((7, 101), (8, 301)), ((9, 202), (10, 302))] {
            seed_audio(&env, music, ambience);
            audio_requests(&env);
            lua(&env, PHOTOSENSITIVITY_TRY_SHOW);
            let stops = audio_requests(&env);
            let sim = env.state().borrow();
            let audio = &sim.glue_state.audio;
            assert_eq!(stops, [GlueAudioRequest::StopMusic { handle: music.0 }, GlueAudioRequest::StopMusic { handle: 0 }, GlueAudioRequest::StopAmbience { handle: ambience.0 }]);
            assert_eq!((audio.music_stop_markers, audio.ambience_stop_marker), ([true; 2], true));
            assert_eq!((audio.requested_music_kit, audio.requested_ambience_kit), (music.1, ambience.1));
        }
    }
}

// Argument contracts (R88ad.7, .9, .10) on the direct native globals.
#[test]
fn glue_state_globals_validate_arguments() {
    test_timeout! {
        let env = login_env();
        for (code, usage) in [
            ("SetSavedAccountName()", "Usage: SetSavedAccountName(accountName)"),
            ("SetSavedAccountName(true)", "Usage: SetSavedAccountName(accountName)"),
            ("SetUsesToken(nil)", "Usage: SetUsesToken(usesToken)"),
            ("GetLoginScreenBackground(1)", "Usage: local result = GetLoginScreenBackground(highResFileID, lowResFileID)"),
            ("PlayGlueMusic()", "Usage: PlayGlueMusic(soundKitID)"),
            ("PlayGlueMusic({})", "outside of expected range -2147483648 to 2147483647 - Usage: PlayGlueMusic(soundKitID)"),
            ("PlayGlueAmbience()", "Usage: PlayGlueAmbience(soundKitID [, fadeInTime])"),
            ("PlayGlueAmbience(1, 1/0)", "must be a finite number - Usage: PlayGlueAmbience(soundKitID [, fadeInTime])"),
            ("PlayGlueAmbience(1, {})", "outside of expected range -3.402823e+38 to 3.402823e+38 - Usage: PlayGlueAmbience(soundKitID [, fadeInTime])"),
        ] {
            let message = lua_error(&env, code);
            assert!(message.contains(usage), "{code}: {message}");
        }
        let counts = "local n = select('#', PlayGlueMusic(0)) + select('#', StopGlueMusic())
            n = n + select('#', PlayGlueAmbience(0)) + select('#', StopGlueAmbience())
            return tostring(n + select('#', SetUsesToken(false)))";
        assert_eq!(lua(&env, counts), "0");
    }
}

// R88ad.7: every accepted non-false SetUsesToken value writes `1`.
#[test]
fn set_uses_token_writes_decimal_truthiness() {
    test_timeout! {
        let env = login_env();
        seed_setting(&env, "g_accountUsesToken", "0");
        for value in ["0", "''", "{}", "true"] {
            lua(&env, &format!("SetUsesToken({value}) return ''"));
            assert_eq!(setting(&env, "g_accountUsesToken").as_deref(), Some("1"), "{value}");
            lua(&env, "SetUsesToken(false) return ''");
            assert_eq!(setting(&env, "g_accountUsesToken").as_deref(), Some("0"));
        }
        seed_setting(&env, "accountName", "x");
        lua(&env, "SetSavedAccountName(12.5) return ''");
        assert_eq!(setting(&env, "accountName").as_deref(), Some("12.5"));
    }
}

fn ambience_fade(env: &WowLuaEnv, args: &str) -> i32 {
    audio_requests(env);
    lua(env, &format!("PlayGlueAmbience({args}) return ''"));
    match audio_requests(env)[..] {
        [GlueAudioRequest::StartAmbience { fade_ms, .. }] => fade_ms,
        ref other => panic!("PlayGlueAmbience({args}): {other:?}"),
    }
}

// R88ad.10: absent fade is -1; a present fade is trunc(f32(fade) * 1000f32);
// an overflowing conversion is i32::MIN. Handles stay inactive, so every
// call submits a new request.
#[test]
fn play_glue_ambience_converts_fade_input() {
    test_timeout! {
        let env = login_env();
        assert_eq!(ambience_fade(&env, "301"), -1, "absent fade");
        assert_eq!(ambience_fade(&env, "302, 2.5"), 2500);
        assert_eq!(ambience_fade(&env, "303, 0.0019"), 1, "truncation");
        // 2147483.5 * 1000 is 2147483500 in f64 but rounds to 2147483520 in f32.
        assert_eq!(ambience_fade(&env, "304, 2147483.5"), 2_147_483_520, "single precision");
        assert_eq!(ambience_fade(&env, "305, 3000000"), i32::MIN, "positive overflow");
        assert_eq!(ambience_fade(&env, "306, -3000000"), i32::MIN, "negative overflow");
    }
}

// R88ad.10: both native music control flags set suppress the whole action.
#[test]
fn play_glue_music_both_control_flags_suppress_request() {
    test_timeout! {
        let env = login_env();
        env.state().borrow_mut().glue_state.audio.music_suppressed = [true; 2];
        for kit in [101, 202] {
            audio_requests(&env);
            lua(&env, &format!("PlayGlueMusic({kit}) return ''"));
            assert_eq!(audio_requests(&env), [], "suppressed {kit}");
            assert_eq!(env.state().borrow().glue_state.audio.requested_music_kit, 0);
        }
        env.state().borrow_mut().glue_state.audio.music_suppressed = [false; 2];
        lua(&env, "PlayGlueMusic(202) return ''");
        let started = audio_requests(&env);
        assert!(started.iter().any(|r| matches!(r, GlueAudioRequest::StartMusic { kit: 202, .. })), "{started:?}");
        assert_eq!(env.state().borrow().glue_state.audio.requested_music_kit, 202);
    }
}

const SERVER_BOOLEANS: &str = r#"
    local _, first, second, additional = GetServerName()
    return tostring(first) .. "|" .. tostring(second) .. "|" .. tostring(additional)
"#;

// R88ad.8: a Cfg_Configs row miss leaves all three Booleans false; a row hit
// supplies the first two and permits the additional connection Boolean.
#[test]
fn server_name_publishes_additional_connection_boolean_on_row_hit() {
    test_timeout! {
        let env = login_env();
        for (config, additional, expected) in [
            (None, true, "false|false|false"),
            (Some((true, false)), true, "true|false|true"),
            (Some((false, true)), false, "false|true|false"),
        ] {
            set_server(&env, GlueServerState {
                available: true,
                name: Some("Alpha".into()),
                config,
                additional_connection_flag: additional,
                realm_number: Some(101),
            });
            assert_eq!(lua(&env, SERVER_BOOLEANS), expected, "{config:?} {additional}");
        }
    }
}
