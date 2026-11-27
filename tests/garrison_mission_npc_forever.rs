#![cfg(feature = "client-wowforever")]
//! Forever `C_Garrison.IsAtGarrisonMissionNPC` reads `A_Admin.SetAtGarrisonMissionNPC`.
//! Its original callers (Blizzard_BFAMissionUI.lua:232, Blizzard_CovenantMissionUI.lua:280)
//! are in Blizzard_GarrisonUI, which Forever excludes (`## ExcludeLoadGameType:
//! camelot`), so the test calls the API.

#[test]
fn forever_mission_npc_query_follows_the_supplied_interaction() {
    crate::common::with_timeout(120, || {
        let env = crate::common::forever_game_env();
        let states: (bool, bool, bool) = env
            .eval(
                r#"
                A_Admin.SetAtGarrisonMissionNPC(true)
                local at = C_Garrison.IsAtGarrisonMissionNPC()
                A_Admin.SetAtGarrisonMissionNPC(false)
                local away = C_Garrison.IsAtGarrisonMissionNPC()
                A_Admin.SetAtGarrisonMissionNPC(true)
                return at, away, C_Garrison.IsAtGarrisonMissionNPC()
                "#,
            )
            .expect("the mission NPC query reads the supplied state");
        assert_eq!(states, (true, false, true));
    });
}
