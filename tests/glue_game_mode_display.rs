#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
use crate::common;
use wow_ui_sim::c_api::game_mode_display::GameModeRow;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

#[test]
fn glue_display_uses_shared_cvar_rows_override_and_clear_callback() {
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.exec("assert(GetCVarDefault('currentGameMode') == '-1'); assert(type(C_GameRules.GetCurrentGameModeDisplayInfo()) ~= 'table')").unwrap();
    env.state().borrow_mut().game_rules.display.rows.insert(
        7,
        GameModeRow {
            glue_screen_name: "hostscreen".into(),
            logo: 123,
            logo_height: 40,
            logo_vertical_offset: -8,
            logo_shrunken_height: 20,
            logo_uses_dark_backdrop: true,
            character_create_extra_height: 3,
            character_create_outer_border: 456,
        },
    );
    env.state()
        .borrow_mut()
        .game_rules
        .display
        .rows
        .insert(8, Default::default());
    env.state()
        .borrow_mut()
        .game_rules
        .display
        .set_temporary_override(7);
    env.exec(
        r#"
        local d = C_GameRules.GetCurrentGameModeDisplayInfo()
        assert(C_GameRules.GetGameModeGlueScreenName() == 'hostscreen')
        assert(d.logo == 123 and d.logoHeight == 40 and d.logoVerticalOffset == -8)
        assert(d.logoShrunkenHeight == 20 and d.logoUsesDarkBackdrop == true)
        assert(d.characterCreateExtraHeight == 3 and d.characterCreateOuterBorder == 456)
        displayEvents = 0
        local f = CreateFrame('Frame'); f:RegisterEvent('GAME_MODE_DISPLAY_INFO_UPDATED')
        f:SetScript('OnEvent', function() displayEvents = displayEvents + 1 end)
        SetCVar('currentGameMode', 8)
        local zero = C_GameRules.GetCurrentGameModeDisplayInfo()
        assert(zero.logo == nil and zero.characterCreateOuterBorder == nil)
        assert(zero.logoHeight == 0 and zero.logoUsesDarkBackdrop == false)
        assert(C_GameRules.GetGameModeGlueScreenName() == nil and displayEvents > 0)
        SetCVar('currentGameMode', 7)
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .game_rules
        .display
        .set_temporary_override(8);
    env.exec("assert(C_GameRules.GetGameModeGlueScreenName() == nil)")
        .unwrap();
    env.state()
        .borrow_mut()
        .game_rules
        .display
        .clear_realm_list();
    env.exec("assert(C_GameRules.GetGameModeGlueScreenName() == 'hostscreen')")
        .unwrap();
    for screen in [ScreenKind::Login, ScreenKind::Game] {
        env.set_screen_mode(screen);
        env.exec("assert(type(C_GameRules.GetCurrentGameModeDisplayInfo()) == 'table')")
            .unwrap();
    }
    env.state().borrow_mut().game_rules.display.rows.insert(
        0,
        GameModeRow {
            glue_screen_name: "zero-row".into(),
            ..Default::default()
        },
    );
    env.state().borrow_mut().game_rules.display.rows.insert(
        -1,
        GameModeRow {
            glue_screen_name: "override-negative".into(),
            ..Default::default()
        },
    );
    env.exec("SetCVar('currentGameMode', 0); assert(C_GameRules.GetGameModeGlueScreenName() == 'zero-row'); assert(type(C_GameRules.GetCurrentGameModeDisplayInfo()) ~= 'table')").unwrap();
    env.state()
        .borrow_mut()
        .game_rules
        .display
        .set_temporary_override(-1);
    env.exec("assert(C_GameRules.GetGameModeGlueScreenName() == 'override-negative'); assert(type(C_GameRules.GetCurrentGameModeDisplayInfo()) == 'table')").unwrap();
    env.exec(
        "SetCVar('currentGameMode', -1); assert(C_GameRules.GetGameModeGlueScreenName() == nil)",
    )
    .unwrap();
    env.exec("A_Admin.SetActiveGameMode(7, 'changed-screen'); local d = C_GameRules.GetCurrentGameModeDisplayInfo(); assert(d.logo == 123 and d.logoUsesDarkBackdrop); assert(C_GameRules.GetGameModeGlueScreenName() == 'changed-screen')").unwrap();
    for value in [-1, 0, 999] {
        env.exec(&format!("SetCVar('currentGameMode', {value}); assert(type(C_GameRules.GetCurrentGameModeDisplayInfo()) ~= 'table')")).unwrap();
    }
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn glue_display_original_backdrop_consumer_tracks_host_flag() {
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.set_screen_mode(ScreenKind::Login);
    let path = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .unwrap()
        .join("Blizzard_GlueXML/Shared/GameLogoDarkBackdrop.lua");
    env.exec(&std::fs::read_to_string(path).unwrap()).unwrap();
    env.exec(
        r#"
        DisplayProbe = CreateFrame('Frame')
        Mixin(DisplayProbe, GameLogoDarkBackdropMixin)
        DisplayProbe.BackdropTexture = DisplayProbe:CreateTexture()
        DisplayProbe:SetScript('OnEvent', DisplayProbe.OnEvent)
        DisplayProbe:OnLoad()
        assert(not DisplayProbe.BackdropTexture:IsShown())
    "#,
    )
    .unwrap();
    env.state().borrow_mut().game_rules.display.rows.insert(
        7,
        GameModeRow {
            logo_uses_dark_backdrop: true,
            ..Default::default()
        },
    );
    env.exec("SetCVar('currentGameMode', 7); assert(DisplayProbe.BackdropTexture:IsShown())")
        .unwrap();
    env.state()
        .borrow_mut()
        .game_rules
        .display
        .rows
        .insert(8, Default::default());
    env.exec("SetCVar('currentGameMode', 8); assert(not DisplayProbe.BackdropTexture:IsShown())")
        .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
