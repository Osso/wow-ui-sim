#[cfg(feature = "client-wowforever")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_all_class_ids_match_class_catalogue_and_are_independent() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local ids = C_SpecializationInfo.GetAllClassIDs()
        assert(#ids == 13)
        for id = 1, 13 do
            assert(ids[id] == id)
            assert(C_CreatureInfo.GetClassInfo(id).classID == id)
        end
        ids[1] = 999
        assert(C_SpecializationInfo.GetAllClassIDs()[1] == 1)
        assert(select('#', C_SpecializationInfo.GetAllClassIDs()) == 1)
    "#,
    )
    .unwrap();
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_club_finder_initialization_enumerates_classes() {
    let env = WowLuaEnv::new().unwrap();
    let root = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let source = std::fs::read_to_string(root.join("Blizzard_Communities/ClubFinder.lua")).unwrap();
    env.exec(&source).unwrap();
    env.exec(
        "assert(type(ClubsRecruitmentDialogMixin.UpdatedPostingInformationInit) == 'function')",
    )
    .unwrap();
}
