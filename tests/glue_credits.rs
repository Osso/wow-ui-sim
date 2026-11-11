#![cfg(any(feature = "client-ptr", feature = "client-wowforever"))]
use crate::common;
use wow_ui_sim::c_api::credits::CreditsRow;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
#[test]
fn glue_credits_projects_supplied_rows_and_profile_screen_catalogs() {
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.exec("assert(type(GetCreditsDisplayInfo) == 'function'); assert(GetCreditsTextByFileDataID == nil)").unwrap();
    // Empty rows are explicit host policy, not a native fresh-Glue assertion.
    env.exec(
        "assert(type(GetCreditsDisplayInfo()) == 'table' and next(GetCreditsDisplayInfo()) == nil)",
    )
    .unwrap();
    env.set_screen_mode(ScreenKind::Login);
    env.state().borrow_mut().credits.rows = vec![
        CreditsRow {
            is_default: true,
            sound_kit: Some(0),
            ..Default::default()
        },
        CreditsRow {
            credits_file_id: 17,
            logo: 23,
            name: "Host credits".into(),
            ..Default::default()
        },
    ];
    env.state()
        .borrow_mut()
        .credits
        .texts
        .insert(17, "Host-supplied text".into());
    env.exec(r#"
        assert(type(GetCreditsTextByFileDataID) == 'function')
        local rows = GetCreditsDisplayInfo()
        assert(#rows == 2 and rows[1].isDefault == true and rows[2].isDefault == false)
        assert(rows[1].creditsFileID == 0 and rows[1].keyArt == '' and rows[1].background == '' and rows[1].name == '')
        assert(rows[1].logo == nil and rows[1].soundKit == 0)
        assert(rows[2].creditsFileID == 17 and rows[2].logo == 23 and rows[2].soundKit == nil)
        -- Explicit host text seam; native known/unknown-ID return paths await R65.
        assert(GetCreditsTextByFileDataID(17) == 'Host-supplied text')
        rows[2].name = 'changed'
        assert(GetCreditsDisplayInfo()[2].name == 'Host credits')
    "#).unwrap();
    env.set_screen_mode(ScreenKind::Game);
    env.exec("assert(type(GetCreditsDisplayInfo) == 'function'); assert(GetCreditsTextByFileDataID == nil)").unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
