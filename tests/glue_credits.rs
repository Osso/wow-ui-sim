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
        -- Explicit host file contents, independent of the display catalog.
        assert(GetCreditsTextByFileDataID(17) == 'Host-supplied text')
        rows[2].name = 'changed'
        assert(GetCreditsDisplayInfo()[2].name == 'Host credits')
    "#).unwrap();
    env.set_screen_mode(ScreenKind::Game);
    env.exec("assert(type(GetCreditsDisplayInfo) == 'function'); assert(GetCreditsTextByFileDataID == nil)").unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn glue_credits_text_validates_r65_ids_and_reads_direct_host_files() {
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.set_screen_mode(ScreenKind::Login);
    {
        let mut sim = env.state().borrow_mut();
        sim.credits
            .texts
            .insert(17, "File not present in credits rows".into());
        sim.credits.texts.insert(-17, "Signed host ID".into());
        sim.credits.texts.insert(18, String::new());
    }
    env.exec(
        r#"
        local usage = 'Usage: local creditsText = GetCreditsTextByFileDataID(fileDataID)'
        local function fails(...)
            local ok, err = pcall(GetCreditsTextByFileDataID, ...)
            assert(not ok and tostring(err):find(usage, 1, true), tostring(err))
        end
        fails(); fails(nil); fails(false); fails({}); fails(function() end)
        fails('bad'); fails(0/0); fails(math.huge); fails(-math.huge)
        fails(2147483648); fails(-2147483649)
    "#,
    )
    .unwrap();
    // Return publication is VERIFIED on Forever. PTR uses the same approved
    // host-map implementation, but its complete native publisher is unproven.
    #[cfg(feature = "client-wowforever")]
    env.exec(
        r#"
        assert(GetCreditsTextByFileDataID(17.9) == 'File not present in credits rows')
        assert(GetCreditsTextByFileDataID('17') == 'File not present in credits rows')
        assert(GetCreditsTextByFileDataID(-17.9) == 'Signed host ID')
        assert(select('#', GetCreditsTextByFileDataID(18)) == 1)
        assert(GetCreditsTextByFileDataID(18) == '')
        assert(select('#', GetCreditsTextByFileDataID(19)) == 1)
        assert(GetCreditsTextByFileDataID(19) == nil)
    "#,
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
