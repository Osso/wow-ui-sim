//! Native separator publication through the TOC-selected Camelot name consumer.

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_character_name_constants_reach_selected_name_util() {
    use crate::common::blizzard_addon_harness::{
        blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
    };

    let root = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&root);
    {
        let mut state = env.state().borrow_mut();
        state.player.name = "Dar".into();
        state.player.surname = Some("Mara".into());
    }
    let loaded = load_blizzard_addon_closure_into_env(&env, &root, &["Blizzard_FrameXMLUtil"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_FrameXMLUtil"));
    let result: String = env
        .eval(
            r#"
            local source = debug.getinfo(NameUtil.FormatUnitNameForDisplay, "S").source:gsub("\\", "/")
            assert(source:match("Blizzard_FrameXMLUtil/Camelot/NameUtil.lua$"), source)
            assert(NameUtil.FormatUnitNameForDisplay("player") == "Dar Mara")
            assert(NameUtil.GetUnmodifiedUnitFullName("player") == "Dar Mara")
            assert(NameUtil.GetFullNameWithoutRealm("Dar", "Mara") == "Dar Mara")
            local first, surname = NameUtil.SplitPlayerNameIntoParts("Dar Mara")
            assert(first == "Dar" and surname == "Mara")
            local constants = rawget(Constants, "CharacterNameSeparatorConsts")
            local separator = rawget(constants, "CHARACTERNAME_SURNAME_SEPARATOR")
            assert(type(separator) == "string" and #separator == 1 and separator:byte() == 32)
            assert(rawget(constants, "CHARACTERNAME_LINK_SEPARATOR") == "-")
            assert(rawget(constants, "CHARACTERNAME_REALMNAME_SEPARATOR") == "-")
            return "Dar Mara"
            "#,
        )
        .unwrap();
    assert_eq!(result, "Dar Mara");

    env.state().borrow_mut().player.surname = None;
    let unsurnamed: (String, String) = env
        .eval(
            "return NameUtil.FormatUnitNameForDisplay('player'), NameUtil.GetUnmodifiedUnitFullName('player')",
        )
        .unwrap();
    assert_eq!(unsurnamed, ("Dar".into(), "Dar".into()));
}
