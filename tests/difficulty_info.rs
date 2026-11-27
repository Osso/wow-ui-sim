//! `GetDifficultyInfo` / `IsLegacyDifficulty` (InstanceDocumentation.lua,
//! Retail) over the pinned Retail 12.1.5.70961 Difficulty DB2 rows.
#![cfg(feature = "client-retail")]

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
fn difficulty_info_reads_the_pinned_difficulty_rows() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(DIFFICULTY_ROWS).expect("GetDifficultyInfo rows");
    env.exec(LEGACY_ROWS).expect("IsLegacyDifficulty rows");
}
