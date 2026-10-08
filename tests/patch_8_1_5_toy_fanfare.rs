//! Acquisition and clearing must agree across both toy APIs.
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_8_1_5_toy_fanfare_tracks_collection_transitions() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local id = 13379
        A_Admin.UncollectToy(id)
        assert(C_ToyBoxInfo.NeedsFanfare(id) == false)
        assert(C_ToyBoxInfo.NeedsFanfare(999999) == false)
        A_Admin.CollectToy(id)
        assert(PlayerHasToy(id))
        assert(C_ToyBoxInfo.NeedsFanfare(id) == true)
        assert(select(5, C_ToyBox.GetToyInfo(id)) == true)
        C_ToyBoxInfo.ClearFanfare(999999)
        assert(C_ToyBoxInfo.NeedsFanfare(id) == true)
        C_ToyBoxInfo.ClearFanfare(id)
        C_ToyBoxInfo.ClearFanfare(id)
        assert(C_ToyBoxInfo.NeedsFanfare(id) == false)
        assert(select(5, C_ToyBox.GetToyInfo(id)) == false)
        assert(PlayerHasToy(id))
        A_Admin.CollectToy(id)
        assert(C_ToyBoxInfo.NeedsFanfare(id) == false)
        A_Admin.UncollectToy(id)
        A_Admin.CollectToy(id)
        assert(C_ToyBoxInfo.NeedsFanfare(id) == true)
        A_Admin.UncollectToy(id)
        assert(C_ToyBoxInfo.NeedsFanfare(id) == false)
        assert(select(5, C_ToyBox.GetToyInfo(id)) == false)
    "#).unwrap();
}
