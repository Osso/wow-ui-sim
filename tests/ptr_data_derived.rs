#![cfg(feature = "client-ptr")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn club_finder_loads_with_the_selected_class_catalog() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        observed_class_ids = {}
        local original = C_SpecializationInfo.GetNumSpecializationsForClassID
        C_SpecializationInfo.GetNumSpecializationsForClassID = function(id)
            table.insert(observed_class_ids, id)
            return original(id)
        end
    "#,
    )
    .unwrap();
    let root = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let source = std::fs::read_to_string(root.join("Blizzard_Communities/ClubFinder.lua")).unwrap();
    env.exec(&source).unwrap();
    env.exec(
        r#"
        assert(type(ClubsRecruitmentDialogMixin.UpdatedPostingInformationInit) == 'function')
        assert(#observed_class_ids == 13)
        for id = 1, 13 do
            assert(observed_class_ids[id] == id)
            assert(C_CreatureInfo.GetClassInfo(id).classID == id)
        end
    "#,
    )
    .unwrap();
}
