use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

#[test]
fn login_screen_updates_glue_login_state() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_mode(ScreenKind::Login);

    assert!(env.eval::<bool>("return InGlue()").unwrap());
    assert!(env.eval::<bool>("return C_Glue.IsOnGlueScreen()").unwrap());
    assert!(!env.eval::<bool>("return IsLoggedIn()").unwrap());

    let (aurora_state, connected_to_wow, wow_connection_state, has_realm_list): (
        i32,
        bool,
        i32,
        bool,
    ) = env.eval("return C_Login.GetState()").unwrap();
    let expected_aurora_state: i32 = env.eval("return LE_AURORA_STATE_NONE").unwrap();
    assert_eq!(aurora_state, expected_aurora_state);
    assert!(!connected_to_wow);
    assert_eq!(wow_connection_state, 0);
    assert!(!has_realm_list);
}

#[test]
fn character_select_screen_updates_glue_login_state() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_mode(ScreenKind::CharacterSelect);

    assert!(env.eval::<bool>("return InGlue()").unwrap());
    assert!(env.eval::<bool>("return C_Glue.IsOnGlueScreen()").unwrap());
    assert!(!env.eval::<bool>("return IsLoggedIn()").unwrap());

    let (_aurora_state, connected_to_wow, wow_connection_state, has_realm_list): (
        i32,
        bool,
        i32,
        bool,
    ) = env.eval("return C_Login.GetState()").unwrap();
    assert!(connected_to_wow);
    assert_eq!(wow_connection_state, 0);
    assert!(!has_realm_list);
}

#[test]
fn character_create_screen_updates_glue_login_state() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_mode(ScreenKind::CharacterCreate);

    assert!(env.eval::<bool>("return InGlue()").unwrap());
    assert!(env.eval::<bool>("return C_Glue.IsOnGlueScreen()").unwrap());
    assert!(!env.eval::<bool>("return IsLoggedIn()").unwrap());

    let (_aurora_state, connected_to_wow, wow_connection_state, has_realm_list): (
        i32,
        bool,
        i32,
        bool,
    ) = env.eval("return C_Login.GetState()").unwrap();
    assert!(connected_to_wow);
    assert_eq!(wow_connection_state, 0);
    assert!(!has_realm_list);
}

#[test]
fn glue_login_permanent_defaults_remain_registered() {
    let env = WowLuaEnv::new().unwrap();

    let defaults: (bool, bool, Option<String>, bool, bool, bool, bool) = env
        .eval(
            r#"
            local clearResult = C_Login.ClearLastError()
            return C_Login.IsLauncherLogin(),
                   C_Login.IsReconnectLoginPossible(),
                   C_Login.GetLastError(),
                   clearResult == nil,
                   C_Login.AttemptedLauncherLogin(),
                   C_Login.IsNewPlayer(),
                   C_Glue.IsFirstLoadThisSession()
        "#,
        )
        .expect("glue login defaults should be queryable");

    assert_eq!(defaults, (false, false, None, true, false, false, false));
}

#[test]
fn default_screen_size_matches_screen_globals_and_ui_parent() {
    let env = WowLuaEnv::new().unwrap();

    let screen_metrics: (f64, f64, f64, f64, i32, i32) = env
        .eval(
            r#"
        return GetScreenWidth(), GetScreenHeight(), UIParent:GetWidth(), UIParent:GetHeight(), GetPhysicalScreenSize()
    "#,
        )
        .unwrap();
    let (
        screen_width,
        screen_height,
        ui_parent_width,
        ui_parent_height,
        physical_width,
        physical_height,
    ) = screen_metrics;

    assert_eq!(screen_width, 1024.0);
    assert_eq!(screen_height, 768.0);
    assert_eq!(physical_width, 1024);
    assert_eq!(physical_height, 768);
    assert_eq!(ui_parent_width, 1024.0);
    assert_eq!(ui_parent_height, 768.0);
}

#[test]
fn screen_size_globals_follow_canvas_dimensions() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_size(813.0, 822.0);

    let (width, height, physical_width, physical_height): (f64, f64, i32, i32) = env
        .eval(
            r#"
        return GetScreenWidth(), GetScreenHeight(), GetPhysicalScreenSize()
    "#,
        )
        .unwrap();
    assert_eq!(width, 813.0);
    assert_eq!(height, 822.0);
    assert_eq!(physical_width, 813);
    assert_eq!(physical_height, 822);

    env.set_screen_size(1646.0, 822.0);
    let (width, height, physical_width, physical_height): (f64, f64, i32, i32) = env
        .eval(
            r#"
        return GetScreenWidth(), GetScreenHeight(), GetPhysicalScreenSize()
    "#,
        )
        .unwrap();
    assert_eq!(width, 1646.0);
    assert_eq!(height, 822.0);
    assert_eq!(physical_width, 1646);
    assert_eq!(physical_height, 822);
}

#[test]
fn screen_size_globals_report_ui_units_after_ui_parent_scale() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_size(1024.0, 768.0);

    let (width, height, physical_width, physical_height): (f64, f64, i32, i32) = env
        .eval(
            r#"
        UIParent:SetScale(0.64)
        return GetScreenWidth(), GetScreenHeight(), GetPhysicalScreenSize()
    "#,
        )
        .unwrap();

    assert!((width - 1600.0).abs() < 0.001);
    assert!((height - 1200.0).abs() < 0.001);
    assert_eq!(physical_width, 1024);
    assert_eq!(physical_height, 768);
}

#[test]
fn physical_display_input_matches_captured_wow_ui_dimensions() {
    let env = WowLuaEnv::new().expect("create environment");
    env.set_display_size(3440.0, 1440.0)
        .expect("set captured physical display");
    env.exec(
        r#"
        assert(SetCVar("uiScale", "0.79999995231628"))
        assert(SetCVar("useUiScale", "1"))
        local pw, ph = GetPhysicalScreenSize()
        assert(pw == 3440 and ph == 1440)
        assert(math.abs(GetScreenWidth() - 2293.333251953125) < 0.001,
               "physical pixels must not be used as base UI canvas units")
        assert(math.abs(GetScreenHeight() - 960.0001220703125) < 0.001)
        local left, bottom, width, height = UIParent:GetRect()
        assert(left == 0 and bottom == 0)
        assert(math.abs(width - 2293.333251953125) < 0.001)
        assert(math.abs(height - 960.0001220703125) < 0.001)
        assert(math.abs(UIParent:GetEffectiveScale() - 0.7999999523162842) < 0.00001)
        assert(math.abs(ConvertPixelsToUI(1440, 0.8) - 960) < 0.001)
        "#,
    )
    .expect("physical display and UI units must match the live capture");

    env.set_display_size(1920.0, 1080.0)
        .expect("resize physical display");
    env.exec(
        r#"
        local width, height = GetPhysicalScreenSize()
        assert(width == 1920 and height == 1080)
        assert(math.abs(GetScreenWidth() - 1706.6667) < 0.001)
        assert(math.abs(GetScreenHeight() - 960) < 0.001)
        "#,
    )
    .expect("display resize must retain scale and update aspect ratio");
}

#[test]
fn invalid_physical_display_dimensions_leave_screen_state_unchanged() {
    let env = WowLuaEnv::new().expect("create environment");
    for dimensions in [(0.0, 1440.0), (3440.0, 0.0), (f32::NAN, 1440.0), (3440.0, f32::INFINITY)] {
        assert!(env.set_display_size(dimensions.0, dimensions.1).is_err());
    }
    let metrics: (f64, f64, f64, f64) = env
        .eval("return GetScreenWidth(), GetScreenHeight(), GetPhysicalScreenSize()")
        .expect("read unchanged screen metrics");
    assert_eq!(metrics, (1024.0, 768.0, 1024.0, 768.0));
}
