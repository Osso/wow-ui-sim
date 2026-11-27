#![cfg(feature = "client-wowforever")]
//! Forever `C_PartyInfo.GetMinLevel` reads the supplied player and active-party
//! levels. Its only original caller (`ScenarioFinder_Shared.lua:58`) is in
//! Blizzard_GroupFinder, which Forever excludes (`## ExcludeLoadGameType:
//! camelot`), so the test calls the API.

use wow_ui_sim::lua_api::WowLuaEnv;

const MIN_LEVEL: &str = r#"
local playerLevel, memberLevel = ...
A_Admin.SetPlayerLevel(playerLevel)
A_Admin.SetPartySize(memberLevel and 1 or 0)
if memberLevel then A_Admin.SetPartyMember(1, "Member", 1, memberLevel) end
return C_PartyInfo.GetMinLevel()
"#;

fn min_level(env: &WowLuaEnv, player: i32, member: Option<i32>) -> i32 {
    let member = member.map_or_else(|| "nil".to_owned(), |level| level.to_string());
    let chunk = format!("return (function(...) {MIN_LEVEL} end)({player}, {member})");
    env.eval(&chunk).expect("GetMinLevel should read the supplied levels")
}

#[test]
fn forever_party_min_level_follows_the_supplied_roster() {
    crate::common::with_timeout(120, || {
        let env = crate::common::forever_game_env();
        assert_eq!(min_level(&env, 60, Some(49)), 49);
        assert_eq!(min_level(&env, 48, Some(50)), 48);
        assert_eq!(min_level(&env, 60, None), 60);
    });
}
