#![cfg(feature = "retail-12-0-5")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_12_0_1_strips_loose_file_textures_without_other_markup_changes() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local strip = C_StringUtil.StripTextureMarkupForLooseFiles
        assert(strip('A|TInterface\\Icons\\Test:16:16|tB') == 'AB')
        assert(strip('A|T12345:16:16|tB|Aatlas:16:16|a') == 'A|T12345:16:16|tB|Aatlas:16:16|a')
        assert(strip('é||Tloose.blp|t|cffffffffX|r') == 'é||Tloose.blp|t|cffffffffX|r')
        assert(strip('|Tbroken') == '|Tbroken')
    "#).unwrap();
}

#[test]
fn patch_12_0_1_death_recap_maximum_is_a_death_snapshot() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(C_DeathRecap.GetRecapMaxHealth() == 0)
        assert(C_DeathRecap.GetRecapMaxHealth(9999) == 0)
    "#).unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        for (recap_id, max_health) in [(41, 10000), (42, 25000)] {
            sim.death_recaps.push(wow_ui_sim::lua_api::state::DeathRecapEntry {
                recap_id, max_health, zone_name: "Raid".into(), killing_blows: vec![],
            });
        }
        sim.player.health_max = 99000;
    }
    env.exec(r#"
        assert(C_DeathRecap.GetRecapMaxHealth(41) == 10000)
        assert(C_DeathRecap.GetRecapMaxHealth(42) == 25000)
        assert(C_DeathRecap.GetRecapMaxHealth() == 25000)
    "#).unwrap();
}
