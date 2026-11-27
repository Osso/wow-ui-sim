//! `GetDifficultyInfo` / `IsLegacyDifficulty` (InstanceDocumentation.lua,
//! Retail) over the pinned Retail 12.1.5.70961 Difficulty DB2 rows.
#![cfg(feature = "client-retail")]

use crate::common;
use wow_ui_sim::lua_api::WowLuaEnv;

const DIFFICULTY_ROWS: &str = r#"
    local function info(id) return { GetDifficultyInfo(id) } end
    local heroic = info(2)
    assert(heroic[1] == "Heroic" and heroic[2] == "party")
    assert(heroic[3] == true and heroic[5] == false and heroic[6] == false)
    assert(heroic[7] == nil and heroic[8] == false)
    assert(heroic[9] == 5 and heroic[10] == 5 and heroic[11] == true)
    local mythic = info(16)
    assert(mythic[1] == "Mythic" and mythic[2] == "raid")
    assert(mythic[5] == false and mythic[6] == true)
    assert(mythic[9] == 20 and mythic[10] == 20)
    local raid10 = info(3)
    assert(raid10[1] == "10 Player" and raid10[7] == 5)
    local flexHeroic = info(15)
    assert(flexHeroic[3] == false and flexHeroic[5] == true)
    assert(flexHeroic[9] == 10 and flexHeroic[10] == 30)
    assert(info(7)[8] == true)
    assert(select('#', GetDifficultyInfo(999)) == 0)
"#;

const LEGACY_ROWS: &str = r#"
    assert(IsLegacyDifficulty(3) == true)
    assert(IsLegacyDifficulty(9) == true)
    assert(IsLegacyDifficulty(14) == false)
    assert(IsLegacyDifficulty(16) == false)
    assert(IsLegacyDifficulty(999) == nil)
"#;

#[test]
fn instance_difficulty_banner_uses_supplied_difficulty_flags() {
    common::with_timeout(120, || {
        let env = WowLuaEnv::new().expect("Lua environment");
        common::load_game_startup_inventory(&env);
        env.sync_addon_names_to_lua();
        env.apply_post_load_workarounds();
        wow_ui_sim::startup::fire_startup_events_for_screen(
            &env,
            wow_ui_sim::screen::ScreenKind::Game,
        );
        env.exec(
            r#"
            A_Admin.SetManualClock(0)
            local banner = MinimapCluster.InstanceDifficulty
            for _, case in ipairs({ { 2, "HeroicTexture" }, { 16, "MythicTexture" } }) do
                A_Admin.SetInstanceInfo("Test Instance", "party", case[1], 5)
                A_Admin.FireEvent("UPDATE_INSTANCE_INFO")
                A_Admin.AdvanceTime(0.3)
                assert(banner.Default[case[2]]:IsShown(), case[2])
                local other = case[2] == "HeroicTexture" and "MythicTexture" or "HeroicTexture"
                assert(not banner.Default[other]:IsShown(), other)
            end
            "#,
        )
        .expect("instance banner textures follow each supplied difficulty");
    });
}

#[test]
fn difficulty_info_reads_the_pinned_difficulty_rows() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(DIFFICULTY_ROWS).expect("GetDifficultyInfo rows");
    env.exec(LEGACY_ROWS).expect("IsLegacyDifficulty rows");
}
