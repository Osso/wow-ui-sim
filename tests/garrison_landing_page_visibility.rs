//! Host landing-page minimap records on Retail and Forever.
#![cfg(feature = "garrison-host-inputs")]
use wow_ui_sim::lua_api::WowLuaEnv;

const VISIBILITY: &str = r#"
    return C_Garrison.GetLandingPageGarrisonType() .. "|"
        .. tostring(C_Garrison.IsLandingPageMinimapButtonVisible(6)) .. "|"
        .. tostring(C_Garrison.IsLandingPageMinimapButtonVisible(262)) .. "|"
        .. tostring(C_Garrison.IsLandingPageMinimapButtonVisible(7)) .. "|"
        .. tostring(C_Garrison.IsLandingPageMinimapButtonVisible(111))
"#;

#[test]
fn landing_page_visibility_follows_supplied_records() {
    let env = WowLuaEnv::new().unwrap();
    assert_eq!(env.eval::<String>(VISIBILITY).unwrap(), "0|false|false|false|false");
    {
        let mut state = env.state().borrow_mut();
        let page = &mut state.garrison_landing_page;
        page.garrison_type = Some(6);
        page.minimap_button_records.insert(6, true);
        page.minimap_button_records.insert(7, false);
        page.minimap_button_records.insert(111, true);
    }
    // 262 shares record key 6; the covenant record also needs its condition.
    assert_eq!(env.eval::<String>(VISIBILITY).unwrap(), "6|true|true|false|false");
    {
        let mut state = env.state().borrow_mut();
        let page = &mut state.garrison_landing_page;
        page.player_condition_105386 = true;
        page.player_state_value = 3;
    }
    assert_eq!(env.eval::<String>(VISIBILITY).unwrap(), "6|true|true|false|true");
}

#[test]
fn landing_page_visibility_rejects_non_numeric_type() {
    let env = WowLuaEnv::new().unwrap();
    let rejected: bool = env
        .eval("return not pcall(C_Garrison.IsLandingPageMinimapButtonVisible, 'garrison')")
        .unwrap();
    assert!(rejected);
}
